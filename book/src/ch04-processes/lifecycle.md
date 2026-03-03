# Task Lifecycle

A task goes through a well-defined sequence of states from creation to final cleanup.

## Task States

```rust
pub enum TaskState {
    Running,    // Currently executing on a CPU
    Runnable,   // Ready to run, waiting for a CPU
    Woken,      // Recently woken up, about to be Runnable
    Sleeping,   // Waiting for an event (I/O, signal, timer)
    Stopped,    // Suspended by SIGSTOP
    Finished,   // Exited, waiting to be reaped
}
```

### State Transitions

```
         fork()
           │
           ▼
        Runnable ◄─────────────── Woken
           │                        ▲
    Scheduled on CPU           wake_up()
           │                        │
           ▼                        │
        Running ───── blocks ────► Sleeping
           │
     returns from syscall / timer tick
           │
           ▼
        Runnable (re-queued by scheduler)

           │ exit()
           ▼
        Finished ──── wait4() by parent ──► Freed
```

### Running vs Runnable

The distinction between `Running` and `Runnable` is important:
- **Running**: The task is the `current_task()` on some CPU right now. There is no context switch needed to execute it.
- **Runnable**: The task is ready but waiting in the scheduler's run queue for a CPU.

Only one task per CPU can be in the `Running` state.

## Sleeping and Waking

When a task needs to wait for something (a file read to complete, a lock to become available, a timer to fire), it transitions to `Sleeping`:

```rust
// Simplified async sleep
pub async fn wait_for_event() {
    // Register a waker — when the event occurs, this waker is called
    let waker = register_waker(current_task().make_waker());

    // Check if event already happened (avoid sleeping unnecessarily)
    if event_already_happened() { return; }

    // Suspend this task
    yield_to_scheduler().await;

    // When we resume here, the event has occurred
}
```

The **waker** mechanism comes from Rust's `async/await` system. When the event occurs (e.g., data arrives on a socket), the kernel calls `waker.wake()`, which transitions the task from `Sleeping` to `Woken` and re-queues it in the scheduler's run queue.

This async model is significantly more efficient than the traditional "sleep on a wait queue" approach in C kernels because:
- No kernel stack is needed while sleeping (the future holds the state)
- The compiler verifies no spinlocks are held across `.await` points

## The Exit Sequence

When a task calls `exit()` or `exit_group()`:

1. **Release resources**: Close file descriptors, release memory mappings
2. **Reparent orphaned children**: Children whose parent exits are reparented to `init` (PID 1)
3. **Notify parent**: Send `SIGCHLD` to the parent thread group and wake any `wait4()` callers
4. **Become a zombie**: Transition to `Finished` state (zombie)

A **zombie** process has exited but has not yet been reaped by its parent. The `ThreadGroup` struct is kept alive so the parent can collect the exit code via `wait4()`.

### Why Zombies?

Zombies exist because Unix's process model requires that a parent can always call `wait4(pid)` to get the child's exit status, even if the parent is slow to do so. The kernel must hold the exit status somewhere until the parent reads it.

If the parent exits without calling `wait4()`, the children are reparented to `init`, which periodically calls `wait4()` to clean up zombies.

## Reaping

When a parent calls `wait4(pid)`:

```rust
pub async fn sys_wait4(pid: i32, status_ptr: UA, options: i32) -> Result<Pid> {
    loop {
        // Check if any child has exited
        if let Some(child) = find_exited_child(pid) {
            let exit_code = child.exit_code();

            // Write exit status to user space
            if !status_ptr.is_null() {
                copy_to_user(status_ptr, &encode_wait_status(exit_code)).await?;
            }

            // Free the child's thread group
            drop(child);

            return Ok(child.tgid);
        }

        // No exited child yet — sleep until SIGCHLD
        wait_for_sigchld().await;
    }
}
```

After the parent reaps the zombie, the child's `ThreadGroup` reference count drops to zero and its memory is freed.

## The Init Process (PID 1)

PID 1 is special: it is the ancestor of all processes and the reaper of last resort. It must:
- Never exit (if PID 1 exits, the kernel panics — there's nowhere to reparent orphans)
- Periodically call `wait4(-1, ...)` to reap any orphaned zombies

In a typical Moss setup, PID 1 is a shell or an init system that manages the rest of the system.

## Exercises

1. What is a zombie process? Why is it necessary? How do you detect zombie processes on a running system?

2. If a process exits while having multiple children, and those children later exit, who reaps them? What happens if PID 1 doesn't call `wait4`?

3. Why is it important that `arc` reference count reaches zero only after the parent reaps the zombie? What resource would be prematurely freed otherwise?
