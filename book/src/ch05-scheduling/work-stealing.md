# Work Stealing (SMP)

With per-CPU run queues, a CPU might have many runnable tasks while another CPU is idle. **Work stealing** allows idle CPUs to take tasks from busy CPUs, keeping all CPUs productive.

## The Load Imbalance Problem

Consider 8 CPUs and 16 tasks. Ideally, each CPU would run 2 tasks simultaneously. But naive per-CPU queues could leave some CPUs idle if tasks are unevenly distributed:

```
CPU 0: [T1][T2][T3][T4][T5][T6]  ← overloaded
CPU 1: [T7][T8][T9][T10]          ← loaded
CPU 2: []                          ← idle!
CPU 3: []                          ← idle!
```

Work stealing allows CPU 2 and 3 to steal tasks from CPU 0 and 1.

## How Moss Places New Tasks

When a task is created or woken up, it needs to be placed on a CPU's run queue. Moss uses a lightweight heuristic: place the task on the **least loaded CPU**.

To avoid a global scan (too expensive), Moss maintains an atomic **"least-tasked CPU info" word** that tracks which CPU has the fewest tasks:

```rust
struct GlobalSchedInfo {
    // Encoded: (cpu_id, task_count) for the least-loaded CPU
    least_tasked: AtomicU64,
}
```

When a new task is created:
```rust
pub fn spawn_task(task: Arc<Task>) {
    let target_cpu = global_sched.least_tasked_cpu();
    target_cpu.sched.enqueue(task);

    // If target_cpu is not the current CPU, send an IPI to wake it up
    if target_cpu != current_cpu() {
        send_ipi(target_cpu, IpiMessage::NewTask);
    }
}
```

This ensures new tasks immediately go to the CPU best positioned to run them.

## Work Stealing Implementation

When a CPU's run queue is empty (it's about to run the idle task), it tries to steal from other CPUs:

```rust
pub fn try_steal_task() -> Option<Arc<Task>> {
    // Look at all other CPUs and find the busiest one
    let busiest = find_busiest_cpu()?;

    // Lock the busiest CPU's run queue
    let stolen = busiest.sched.run_q.steal_half();

    // Take half the tasks (to avoid ping-ponging)
    stolen
}
```

Stealing half the tasks (rather than one) reduces the frequency of future steal operations, lowering overhead.

## The Steal Lock

Accessing another CPU's run queue requires synchronization — the owner CPU might be modifying it simultaneously. Moss uses a lightweight spinlock that is only acquired during steal operations (not during normal scheduling, keeping the common case fast).

## IPI-Based Wakeup

When a new task is placed on a remote CPU's queue, the owner CPU might be in the idle loop (waiting for interrupts). An **Inter-Processor Interrupt (IPI)** is used to wake it up:

```rust
// Sent when placing a task on a remote CPU:
send_ipi(target_cpu, IpiMessage::NewTask);

// Handler on the receiving CPU:
fn handle_ipi(msg: IpiMessage) {
    match msg {
        IpiMessage::NewTask => {
            // A new task was placed on our queue — reschedule!
            force_reschedule();
        }
        IpiMessage::FlushTlb => {
            arch::flush_tlb_all();
        }
        // ...
    }
}
```

The IPI ensures the idle CPU wakes up and runs the new task with minimal latency.

## Migration and Affinity

Moss does not yet implement CPU affinity (pinning a task to a specific CPU). All tasks can migrate freely. This simplifies the implementation but may cause cache performance issues in real workloads — a task that moves between CPUs loses its L1/L2 cache state.

Advanced schedulers like Linux's CFS have elaborate **load balancing** mechanisms that take cache topology, NUMA nodes, and affinity hints into account.

## Exercises

1. Why does Moss steal "half" of a busy CPU's tasks rather than just one? What problem would stealing only one task cause on a very imbalanced system?

2. If two idle CPUs both try to steal from the same busy CPU simultaneously, what could go wrong? How does the spinlock prevent this?

3. What is NUMA (Non-Uniform Memory Access)? Why might a work-stealing scheduler perform poorly on a NUMA system, and how could it be improved?
