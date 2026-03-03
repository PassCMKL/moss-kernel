# Per-CPU State

Moss maintains separate scheduler state for each CPU. This is a critical design decision: a global run queue would require a lock that every CPU acquires on every context switch, creating a serious bottleneck on multi-core systems.

## The `SchedState` Structure

Each CPU has its own `SchedState` instance, stored in per-CPU memory:

```rust
pub struct SchedState {
    /// Runnable tasks — min-heap by v_deadline
    pub run_q: BinaryHeap<SchedTask, MinKey>,

    /// Sleeping/blocked tasks — waiting for events
    pub wait_q: Vec<WaitingTask>,

    /// Current virtual time for this CPU
    pub vclock: u64,

    /// Sum of weights of all runnable tasks on this CPU
    pub total_weight: u64,

    /// Signal: should we switch tasks ASAP?
    pub force_resched: AtomicBool,

    /// The currently running task on this CPU
    pub current: Option<Arc<Task>>,

    /// The idle task (runs when run_q is empty)
    pub idle: Arc<Task>,
}
```

## The Scheduling Loop

The core scheduling loop runs at every scheduling point:

```rust
pub fn schedule() {
    let cpu = current_cpu();
    let sched = cpu.sched_state();

    // Update the virtual clock with time elapsed since last schedule
    let elapsed = sched.timer.elapsed_since_last_tick();
    sched.update_clock(elapsed);

    // Update current task's CPU time accounting
    if let Some(task) = &sched.current {
        task.stime.fetch_add(elapsed, Ordering::Relaxed);
    }

    // Pick the next task
    let next = sched.pick_next_task()
        .unwrap_or_else(|| sched.idle.clone());

    if next.tid != sched.current_tid() {
        // Context switch needed
        context_switch(sched.current.take(), next);
    }
}
```

## Context Switching

The actual context switch is an architecture-specific operation. On AArch64:

```rust
pub fn context_switch(prev: Option<Arc<Task>>, next: Arc<Task>) {
    // Save previous task's registers to its ArchTaskState
    if let Some(prev) = &prev {
        arch::save_task_state(prev);
    }

    // Switch to next task's address space (if different process)
    if should_switch_address_space(&prev, &next) {
        next.address_space().activate();
    }

    // Update TLS pointer (TPID_EL0)
    arch::set_tls(next.tls_ptr);

    // Restore next task's registers
    arch::restore_task_state(&next);

    // Update current task pointer
    set_current_task(next);
}
```

The register save/restore is the most critical part. On AArch64, saving state means writing X0–X30, SP_EL0, and ELR_EL1 to the `ExceptionState` struct in the task:

```rust
pub struct ExceptionState {
    pub x: [u64; 31],      // General-purpose registers X0–X30
    pub elr_el1: u64,      // Return address (where to resume)
    pub spsr_el1: u64,     // Status register
    pub sp_el0: u64,       // User stack pointer
    pub tpid_el0: u64,     // TLS pointer
}
```

## Timer-Driven Preemption

The per-CPU timer fires at a fixed interval (default ~4 ms in Moss). On each tick:

1. The timer interrupt fires, saving the current context to the stack
2. The interrupt handler calls `sched.tick()`
3. `tick()` updates the virtual clock and checks if the current task's time slice is expired
4. If expired (or a higher-priority task is waiting), set `force_resched = true`
5. On return from the interrupt, if `force_resched` is set, call `schedule()`

```rust
pub fn tick(&mut self, elapsed_ns: u64) {
    self.update_clock(elapsed_ns);

    let current = self.current.as_ref().unwrap();
    let current_deadline = current.sched.v_deadline;

    // Is there a task with an earlier deadline?
    if let Some(next) = self.run_q.peek() {
        if next.v_deadline < current_deadline {
            self.force_resched.store(true, Ordering::Release);
        }
    }
}
```

## The `current_task()` Function

Throughout the kernel, code frequently needs to access "the task currently running on this CPU." Moss provides:

```rust
pub fn current_task() -> Arc<Task> {
    // Read the per-CPU current-task pointer
    per_cpu().sched.current.clone().expect("no current task")
}
```

This is implemented efficiently: the per-CPU data structure is at a fixed virtual address specific to each CPU, so accessing it requires no atomic operations or locks.

## Exercises

1. Why do per-CPU run queues reduce lock contention compared to a global run queue?

2. What data in `SchedState` could be accessed from multiple CPUs (e.g., during work stealing)? How must this data be protected?

3. On a system with 8 CPUs, task A is running on CPU 0, and task B (with higher priority) becomes runnable on CPU 3. How does CPU 0 learn about B and preempt A?
