# The EEVDF Algorithm

**EEVDF** (Earliest Eligible Virtual Deadline First) is a proportional-share scheduling algorithm. It guarantees that every task receives CPU time proportional to its assigned weight (priority), while keeping individual task latency bounded.

EEVDF was described by Stoica et al. in 1995 and was adopted as the default Linux scheduler in kernel version 6.6. Moss implements EEVDF as well.

## The Core Idea: Virtual Time

The key insight behind EEVDF (and its predecessor, WFQ/WF²Q) is the concept of a **virtual clock**. The virtual clock runs at a rate inversely proportional to the number of runnable tasks.

When there are N tasks all with equal weight, each task should get 1/N of the CPU. The virtual clock represents "how much CPU time would have been received in an ideal fluid scheduler."

```
Real time:     0────────────────────────────────────── 1s
Virtual time:  0────────────────────────────────────── N seconds
               (advances N times slower than real time)
```

More precisely:

```
vclock += (real_elapsed_ns << VT_FIXED_SHIFT) / sum_of_weights
```

Where `VT_FIXED_SHIFT` is a fixed-point scaling factor for precision.

## Virtual Eligibility and Deadlines

Each task has two virtual timestamps:

- **Virtual eligible time** (`v_eligible`): The virtual time at which the task becomes eligible to run. A task becomes eligible when it has "used its share" of CPU up to this point.
- **Virtual deadline** (`v_deadline`): The virtual time by which the task should complete its current request.

```
v_deadline = max(v_eligible, vclock) + (slice_length / weight)
```

Where `slice_length` is how long the task wants to run (default: ~4 ms) and `weight` is the task's priority weight.

A task is **eligible** when `v_eligible <= vclock`. Only eligible tasks can be selected by the scheduler.

## The Scheduling Decision

At each scheduling point, the scheduler picks the **eligible task with the earliest virtual deadline**:

```
next_task = eligible_tasks.min_by_key(|t| t.v_deadline)
```

This is implemented as a min-heap (or red-black tree) sorted by `v_deadline`.

### Intuition

Consider two tasks with equal weight:
- Task A has been sleeping (no CPU usage) → it will have an earlier `v_eligible` and earlier `v_deadline`
- Task B has been running continuously → it will have a later deadline

EEVDF naturally gives Task A (the well-behaved sleeping task) priority when it wakes up, since it hasn't used its share yet. But Task B is never starved — it will eventually have the earliest deadline among running tasks.

## Example: Two Equal-Weight Tasks

```
vclock: 0   1   2   3   4   5   6   7   8
        ────────────────────────────────────►
A runs: ████████    ████████    ████████
B runs:         ████████    ████████
```

Each task gets alternating time slices. The virtual clock advances at half the real rate (two tasks, equal weight).

## Example: Weighted Tasks (2:1 ratio)

If Task A has weight 2 and Task B has weight 1:

```
vclock: 0   1   2   3   4   5   6
        ────────────────────────────►
A runs: █████████████   ██████████
B runs:              ███         ███
```

Task A gets twice as much CPU as Task B because its larger weight causes its deadline to advance more slowly.

## Implementation in Moss

Moss implements the EEVDF scheduler in `src/sched/mod.rs`. Key data structures:

```rust
pub struct SchedState {
    // Runnable tasks sorted by v_deadline
    run_q: BinaryHeap<SchedTask, MinKey>,

    // Sleeping tasks (waiting to become runnable)
    wait_q: Vec<WaitingTask>,

    // Virtual clock for this CPU
    vclock: u64,

    // Sum of weights of all runnable tasks
    total_weight: u64,

    // Force immediate rescheduling
    force_resched: AtomicBool,
}
```

### Task Selection

```rust
pub fn pick_next_task(&mut self) -> Option<Arc<Task>> {
    let vclock = self.vclock;

    // Find the eligible task with earliest deadline
    self.run_q.iter()
        .filter(|t| t.v_eligible <= vclock)
        .min_by_key(|t| t.v_deadline)
        .map(|t| self.run_q.remove(t))
}
```

### Updating the Virtual Clock

```rust
pub fn update_clock(&mut self, elapsed_ns: u64) {
    if self.total_weight > 0 {
        self.vclock += (elapsed_ns << VT_FIXED_SHIFT) / self.total_weight;
    }
}
```

### Enqueueing a Woken Task

When a sleeping task wakes up:

```rust
pub fn enqueue_task(&mut self, task: Arc<Task>) {
    let vclock = self.vclock;

    // Eligible immediately (it's been sleeping, so it hasn't used CPU)
    task.sched.v_eligible = vclock;

    // Deadline = vclock + slice/weight
    task.sched.v_deadline = vclock + DEFAULT_SLICE / task.weight;

    self.run_q.push(SchedTask::new(task));
    self.total_weight += task.weight;
}
```

## Preemption

When a new task becomes runnable with an earlier deadline than the current task, the scheduler sets a **resched flag**:

```rust
if new_task.v_deadline < current_task.v_deadline {
    set_need_resched();  // Forces a context switch soon
}
```

The actual preemption happens at the next scheduling point (timer interrupt or system call return).

## Exercises

1. Two tasks have weights 1 and 3. Over a 1-second period, how much CPU time does each task receive?

2. What happens to a task's `v_eligible` and `v_deadline` when it has been sleeping for a long time? Is this fair?

3. EEVDF makes a scheduling decision in O(log N) time. What alternative algorithms could you use for the run queue, and what would their performance characteristics be?
