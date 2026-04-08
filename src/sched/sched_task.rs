// src/sched/sched_task.rs
//
// Level 2: Round-Robin Scheduler
//
// DESIGN RATIONALE:
// The existing EEVDF scheduler tracks virtual runtime and "lag" — how much
// CPU time a task has consumed relative to its fair share.  While EEVDF is
// optimal for throughput and latency-sensitive workloads, it adds complexity
// and can deprioritise a task that ran recently (even if no other tasks exist).
//
// Our Round-Robin (RR) implementation is intentionally simple:
//   - Every ready task receives a fixed TIME_SLICE (10 ms).
//   - When a task exhausts its slice it moves to the back of the run queue.
//   - No history, lag, or virtual deadline is tracked — FIFO ordering only.
//   - Starvation is impossible: every task is guaranteed a turn in O(N) time.
//
// COMPARISON with EEVDF (see write-up for full analysis):
//   - RR max waiting time  = (N-1) × TIME_SLICE  (predictable, bounded)
//   - EEVDF waiting time   = variable (depends on lag; newly-woken tasks may
//                            wait longer than expected)
//   - RR completion time   = fair for equal-burst tasks; no starvation
//   - EEVDF completion     = better for mixed workloads (shorter tasks finish
//                            first due to deadline-driven selection)
//
// Conclusion: For homogeneous tasks RR is simpler and equally fair.
// For heterogeneous or interactive workloads EEVDF wins on latency.
//
// METRICS:
// SchedTask records total_wait_time and total_runtime using wall-clock
// Instants.  Call print_task_metrics() at task exit to observe them.

use alloc::boxed::Box;
use core::{cmp::Ordering, ops::Deref, time::Duration};

use crate::{
    drivers::timer::{self, Instant},
    process::owned::OwnedTask,
    sched::SCHED_WEIGHT_BASE,
};

/// Fixed time quantum for Round-Robin scheduling.
const RR_TIME_SLICE: Duration = Duration::from_millis(10);

// ---------------------------------------------------------------------------
// Per-task Round-Robin scheduling state
// ---------------------------------------------------------------------------

/// Embedded scheduling state and metrics for a single task.
///
/// Lifecycle of a task through the scheduler:
///
///   1. Task is placed on the run queue  →  `on_enqueue(now)`
///   2. Task is selected to run          →  `on_dispatch(now)`
///   3. Scheduler ticks while running    →  `tick(now)` → returns true when
///                                          the time slice expires
///   4. Task is preempted or yields      →  `on_preempt(now)` (called from
///                                          `SchedulableTask::re_enqueue`)
///   5. Repeat from step 1.
#[derive(Debug)]
pub struct SchedTask {
    /// Timestamp when this task was last placed on the run queue.
    /// Used to compute `total_wait_time` when the task is dispatched.
    enqueue_time: Option<Instant>,
    /// Timestamp when the current CPU time slice began.
    /// Used by `tick()` to determine when the slice expires.
    slice_start: Option<Instant>,
    /// Total time this task has spent waiting in the run queue.
    pub total_wait_time: Duration,
    /// Total CPU time this task has consumed across all slices.
    pub total_runtime: Duration,
}

impl SchedTask {
    pub const fn new() -> Self {
        Self {
            enqueue_time: None,
            slice_start: None,
            total_wait_time: Duration::ZERO,
            total_runtime: Duration::ZERO,
        }
    }

    /// Record that this task has been placed on the run queue.
    /// Starts the wait-time accounting window.
    pub fn on_enqueue(&mut self, now: Instant) {
        self.enqueue_time = Some(now);
        self.slice_start = None;
    }

    /// Record that this task has been selected to run.
    /// Closes the wait-time window and opens the CPU-time window.
    pub fn on_dispatch(&mut self, now: Instant) {
        if let Some(enq) = self.enqueue_time.take() {
            self.total_wait_time = self.total_wait_time.saturating_add(now - enq);
        }
        self.slice_start = Some(now);
    }

    /// Called on every scheduler tick while this task is running.
    /// Returns `true` if the Round-Robin time slice has expired.
    pub fn tick(&self, now: Instant) -> bool {
        if let Some(start) = self.slice_start {
            now - start >= RR_TIME_SLICE
        } else {
            false
        }
    }

    /// Record that this task was preempted or voluntarily yielded.
    /// Closes the CPU-time accounting window and accumulates runtime.
    pub fn on_preempt(&mut self, now: Instant) {
        if let Some(start) = self.slice_start.take() {
            self.total_runtime = self.total_runtime.saturating_add(now - start);
        }
    }
}

// ---------------------------------------------------------------------------
// SchedulableTask — the run-queue element
// ---------------------------------------------------------------------------

/// A schedulable wrapper around a kernel task.
///
/// Provides the interface expected by [`RunQueue`] and the scheduler core
/// ([`SchedState`]).
///
/// # Virtual-time fields
/// `v_eligible` is always 0 so every task passes the run-queue eligibility
/// filter.  `v_deadline` is set to the current tick count at enqueue time,
/// giving FIFO ordering: the task that has been waiting longest (lowest
/// timestamp) is selected first.
pub struct SchedulableTask {
    /// Exclusively-owned, CPU-local task state.
    pub task: Box<OwnedTask>,

    /// Virtual eligibility time.  Always 0 in Round-Robin — every task is
    /// immediately eligible.
    pub v_eligible: u128,

    /// Virtual deadline used for FIFO ordering.
    /// Set to `now.ticks()` each time the task enters the run queue so that
    /// the task queued earliest has the numerically lowest value and is
    /// therefore selected first by `min_by(compare_with)`.
    pub v_deadline: u128,

    /// Round-Robin scheduling state and accumulated metrics.
    pub sched: SchedTask,
}

// SAFETY: The scheduler only accesses a SchedulableTask from a single CPU at
// a time (protected by the per-CPU SchedState borrow).
unsafe impl Send for SchedulableTask {}

impl Deref for SchedulableTask {
    type Target = OwnedTask;

    fn deref(&self) -> &Self::Target {
        &self.task
    }
}

impl SchedulableTask {
    /// Wrap an `OwnedTask` in a new `SchedulableTask` ready for insertion into
    /// a run queue.
    pub fn new(task: Box<OwnedTask>) -> Box<Self> {
        Box::new(Self {
            task,
            v_eligible: 0,
            v_deadline: 0,
            sched: SchedTask::new(),
        })
    }

    /// Scheduling weight.  All non-idle tasks have equal weight in Round-Robin.
    pub fn weight(&self) -> i32 {
        SCHED_WEIGHT_BASE
    }

    /// Compare scheduling priority with another task.
    ///
    /// In Round-Robin, the task with the earlier enqueue time (lower
    /// `v_deadline`) has higher priority and will be selected first.
    pub fn compare_with(&self, other: &SchedulableTask) -> Ordering {
        self.v_deadline.cmp(&other.v_deadline)
    }

    /// Called when this task is first placed onto the run queue (e.g. on
    /// creation or after a wakeup from sleep).
    ///
    /// * Closes any open runtime window via `on_preempt` (no-op on first call).
    /// * Sets `v_deadline` to the current timestamp for FIFO ordering.
    /// * Starts the wait-time accounting window.
    ///
    /// The `_vclock` parameter is accepted for API compatibility with the
    /// run-queue interface but is unused in Round-Robin.
    pub fn inserting_into_runqueue(&mut self, _vclock: u128) {
        if let Some(now) = timer::now() {
            self.sched.on_preempt(now); // close any open runtime window
            self.v_deadline = now.ticks() as u128;
            self.sched.on_enqueue(now);
        }
        self.v_eligible = 0;
    }

    /// Called when this task is preempted and immediately re-added to the
    /// back of the run queue.
    ///
    /// Updates `v_deadline` to the current timestamp so the task goes to the
    /// back of the FIFO queue (tasks waiting since before this moment have
    /// earlier/lower timestamps and will run first).
    pub fn re_enqueue(&mut self, now: Instant) {
        self.sched.on_preempt(now); // accumulate runtime for this slice
        self.v_deadline = now.ticks() as u128; // go to back of queue
        self.sched.on_enqueue(now); // start new wait window
        self.v_eligible = 0;
    }

    /// Called just before this task begins executing on the CPU.
    /// Closes the wait-time accounting window opened by `inserting_into_runqueue`.
    pub fn about_to_execute(&mut self, now: Instant) {
        self.sched.on_dispatch(now);
    }

    /// Called on every scheduler tick while this task is running.
    /// Returns `true` if the Round-Robin time slice has expired.
    pub fn tick(&self, now: Instant) -> bool {
        self.sched.tick(now)
    }
}

// ---------------------------------------------------------------------------
// Metrics helpers
// ---------------------------------------------------------------------------

/// Log accumulated scheduling metrics for a task via the kernel log.
/// Intended to be called at task exit or from a diagnostic command.
pub fn print_task_metrics(task: &SchedulableTask) {
    let s = &task.sched;
    log::info!(
        "[RR metrics] pid={} total_runtime={:?}  total_wait={:?}",
        task.descriptor().tgid().value(),
        s.total_runtime,
        s.total_wait_time,
    );
}
