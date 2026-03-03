# The Idle Task

When no runnable tasks exist on a CPU, the CPU cannot simply stop — it needs to do *something*. The **idle task** is a special task that runs when the run queue is empty.

## Purpose

The idle task serves two purposes:

1. **CPU power management**: The idle task executes a "wait for interrupt" instruction, halting the CPU until an interrupt arrives. This significantly reduces power consumption.

2. **Scheduling infrastructure**: The scheduler always has a task to "run." This simplifies the scheduler's logic — it never needs to handle the case of a completely empty run queue.

## Implementation in Moss

Each CPU has its own dedicated idle task created during initialization:

```rust
pub fn create_idle_task(cpu_id: CpuId) -> Arc<Task> {
    let task = Task::new_kernel_task("idle", idle_loop);
    // Mark as lowest possible priority
    task.sched.weight = IDLE_WEIGHT;
    task
}

async fn idle_loop() -> ! {
    loop {
        // Attempt to steal work from other CPUs
        if let Some(stolen_task) = try_steal_task() {
            schedule_task(stolen_task);
            yield_to_scheduler().await;
            continue;
        }

        // Nothing to do — wait for an interrupt
        arch::wait_for_interrupt();
        // After waking from WFI, check the run queue again
        yield_to_scheduler().await;
    }
}
```

## The WFI Instruction

`arch::wait_for_interrupt()` executes the AArch64 `wfi` (Wait For Interrupt) instruction:

```asm
wfi     // CPU enters low-power state until an interrupt occurs
```

After `wfi`, the CPU is in a low-power state where it consumes minimal power. It resumes normal execution when:
- A timer interrupt fires
- An IPI arrives (new task was placed on this CPU's queue)
- Any other hardware interrupt fires

Without `wfi`, the idle loop would be a busy-wait (`loop { }`) that keeps all CPU transistors switching at full rate — wasting power.

## Idle Priority

The idle task has the lowest possible scheduling priority. It only runs when no other task is runnable:

```rust
// In the scheduler's pick_next_task():
pub fn pick_next_task(&mut self) -> Arc<Task> {
    // Try to find a runnable non-idle task
    self.run_q.iter()
        .filter(|t| t.v_eligible <= self.vclock)
        .min_by_key(|t| t.v_deadline)
        .cloned()
        // Fall back to idle task if nothing is runnable
        .unwrap_or_else(|| self.idle.clone())
}
```

The idle task is never placed in the run queue — it's retrieved directly as a fallback.

## CPU Utilization Measurement

The time each CPU spends running the idle task (vs. other tasks) is how utilization is measured:

```
CPU Utilization = 1 - (idle_time / total_time)
```

Monitoring tools like `top` and `vmstat` report this metric. When a system is at 100% CPU utilization, the idle task never runs.

## Exercises

1. What happens if the idle task exits? Why is it implemented as `loop { ... }` rather than exiting when done?

2. On a system with 4 CPUs and 3 runnable tasks, how many CPUs are running the idle task? What does `top` report as CPU utilization?

3. Modern CPUs have different "C-states" (power states deeper than `wfi`). What trade-off exists between deeper C-states (more power savings) and interrupt response latency?
