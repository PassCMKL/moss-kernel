# Scheduling Concepts

Before examining EEVDF, let's establish the core concepts and trade-offs in CPU scheduling.

## Why Is Scheduling Hard?

If every process just ran to completion, there would be no need for a scheduler. But in practice:
- Multiple processes want the CPU simultaneously
- Processes have different priorities and resource needs
- Processes block waiting for I/O, then resume
- Interactive processes need low latency; batch processes need high throughput
- A malicious process might try to monopolize the CPU

The scheduler must balance **fairness**, **throughput**, **latency**, and **starvation-prevention** simultaneously.

## Preemption vs. Cooperative Scheduling

### Cooperative Scheduling

In cooperative scheduling, a process runs until it voluntarily yields the CPU (by calling `yield()` or blocking on I/O). The process controls when context switches happen.

**Pros**: Simple to implement, predictable timing.
**Cons**: A buggy or malicious process that never yields can starve all other processes.

### Preemptive Scheduling

In preemptive scheduling, the kernel can forcibly remove a process from the CPU at any time — typically triggered by a timer interrupt. The process has no say in when it gets preempted.

Moss uses preemptive scheduling. The timer fires every ~4 ms, giving the scheduler a chance to pick a new task.

## Scheduling Metrics

| Metric | Definition | Who cares |
|---|---|---|
| **Throughput** | Tasks completed per second | Batch workloads |
| **Latency** | Time from ready to first execution | Interactive apps |
| **Response time** | Time to complete a task | End users |
| **Fairness** | Does every task get its fair share? | Multi-user systems |
| **Starvation** | Does any task never get the CPU? | All workloads |
| **Overhead** | How much CPU does the scheduler itself use? | All workloads |

## Task Categories

Schedulers typically distinguish between:

- **CPU-bound tasks**: Mostly compute, rarely block (e.g., video encoding)
- **I/O-bound tasks**: Frequently block waiting for I/O (e.g., a web server waiting for network)
- **Interactive tasks**: Need low latency for human-perceptible responsiveness (e.g., text editor)

A good scheduler should give I/O-bound and interactive tasks low latency (they don't use much CPU anyway) while giving CPU-bound tasks high throughput.

## Context Switching

A **context switch** is the act of saving one task's state and restoring another's. On AArch64, this involves:

1. Saving the current task's general-purpose registers (X0–X30), stack pointer, and program counter
2. Updating the scheduler's bookkeeping (CPU time used, etc.)
3. Loading the next task's registers
4. Switching page tables (TTBR0) if the new task is in a different process
5. Flushing the TLB (if switching processes)

Context switches have real cost — typically 1–10 microseconds. A scheduler that context-switches too frequently wastes CPU on overhead. One that context-switches too rarely has poor interactive latency.

## The Run Queue

The scheduler maintains a **run queue** — a data structure holding all runnable tasks. The key operations are:

- **Enqueue**: Add a newly runnable task
- **Dequeue/pick_next**: Remove and return the next task to run
- **Update**: Adjust a task's position after it has run for a while

Different scheduling algorithms use different data structures for the run queue:
- Round-robin: a simple FIFO queue
- Priority scheduling: a priority heap (min-heap on priority)
- EEVDF: a red-black tree ordered by virtual deadline

## Scheduling Points

The scheduler runs at well-defined **scheduling points**:

1. **Timer interrupt**: Every timer tick (~4 ms), the scheduler checks if the current task's time slice is expired
2. **System call return**: Before returning to user space after a syscall, check if a higher-priority task became runnable
3. **Wakeup**: When a sleeping task wakes up, it may preempt the current task if it has higher priority
4. **Explicit yield**: When a task calls `sched_yield()`

## Exercises

1. What are the trade-offs between a short time slice (e.g., 1 ms) and a long time slice (e.g., 100 ms)?

2. A video game requires frames to be rendered every 16.7 ms (60 FPS). What scheduling properties does the game thread need?

3. How could a cooperative scheduler be exploited by a malicious process? How does preemptive scheduling prevent this?
