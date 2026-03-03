# Sending and Receiving Signals

## Sending Signals

Signals can be sent by:

1. **User programs**: via `kill(pid, signum)` or `tkill(tid, signum)` syscalls
2. **The kernel**: when a hardware exception occurs (`SIGSEGV`, `SIGFPE`, `SIGILL`, etc.)
3. **The terminal driver**: `Ctrl+C` → `SIGINT`, `Ctrl+Z` → `SIGTSTP`
4. **The kernel on events**: `SIGCHLD` when a child exits, `SIGALRM` when a timer fires

### The `kill` Syscall

```rust
pub fn sys_kill(pid: i32, signum: i32) -> Result<()> {
    let sig = Signal::from_int(signum)?;

    // Determine target
    let target = if pid > 0 {
        SignalTarget::Process(Pid::from(pid as u32))
    } else if pid == 0 {
        SignalTarget::ProcessGroup(current_pgid())
    } else if pid == -1 {
        SignalTarget::AllProcesses  // Broadcast (restricted to permitted targets)
    } else {
        SignalTarget::ProcessGroup(Pgid::from((-pid) as u32))
    };

    // Check permission: can we send this signal to the target?
    check_signal_permission(&target, sig)?;

    // Deliver the signal
    send_signal(target, sig)
}
```

### Permission Checks

A process can send signals to another process if:
- It has root privileges (EUID = 0)
- Its real UID or effective UID matches the target's real UID or saved-set-UID

This prevents unprivileged processes from sending signals to processes owned by other users.

## Signal Delivery: Pending Signals

When a signal is sent to a process, it becomes **pending**. Pending signals are stored in a bitmask per thread group:

```rust
pub struct ThreadGroup {
    pub pending_signals: AtomicSigSet,  // Signals pending for the group
    // ...
}

pub struct Task {
    pub pending_signals: AtomicSigSet,  // Thread-specific signals
    pub sig_mask: SpinLock<SigSet>,     // Blocked signals for this thread
}
```

A signal can be pending at either level:
- **Thread-specific** (via `tkill`): only the targeted thread can receive it
- **Process-wide** (via `kill`): any thread in the group can receive it

## Signal Masking

A thread can **block** signals using `rt_sigprocmask`. Blocked signals remain pending but are not delivered until unblocked:

```rust
pub fn sys_rt_sigprocmask(how: i32, new_mask: UA, old_mask: UA) -> Result<()> {
    let task = current_task();
    let mut mask = task.sig_mask.lock();

    // Return old mask if requested
    if !old_mask.is_null() {
        copy_to_user(old_mask, &*mask).await?;
    }

    // Apply new mask based on `how`
    match how {
        SIG_BLOCK =>   { *mask |= read_user_mask(new_mask)?; }
        SIG_UNBLOCK => { *mask &= !read_user_mask(new_mask)?; }
        SIG_SETMASK => { *mask = read_user_mask(new_mask)?; }
        _ => return Err(EINVAL),
    }

    Ok(())
}
```

Note: `SIGKILL` and `SIGSTOP` cannot be masked — the kernel ignores attempts to block them.

## Actual Delivery: When Do Signals Fire?

A pending, unmasked signal is delivered at the next **safe delivery point**:

1. **Returning to user space from a syscall**: Before executing `eret`, the kernel checks for pending signals.
2. **Waking from sleep**: If a task is sleeping and receives a signal, it wakes up and the signal is delivered before re-entering user space.

The delivery happens in `do_signal()`:

```rust
pub fn do_signal(state: &mut ExceptionState) {
    let task = current_task();

    // Find a pending, non-masked signal
    let sig = find_deliverable_signal(&task);
    let action = task.thread_group.sig_actions.lock().get(sig);

    match action {
        SigAction::Default => {
            handle_default_action(sig, state);
        }
        SigAction::Ignore => {
            // Discard the signal
        }
        SigAction::Handler(handler_fn) => {
            setup_user_signal_frame(state, sig, handler_fn);
        }
    }
}
```

## Interrupting Sleeping Syscalls

When a signal arrives while a task is sleeping in a syscall, the syscall must be interrupted. The `.interruptible()` async combinator handles this:

```rust
pub async fn interruptible<F: Future>(future: F) -> Result<F::Output> {
    select! {
        result = future => Ok(result),
        _ = wait_for_signal() => Err(EINTR),
    }
}
```

When interrupted, the syscall returns `-EINTR` to user space. Libc automatically restarts many syscalls (`SA_RESTART` flag), but the program can also check for `EINTR` and restart manually.

## Exercises

1. What is a "signal race"? Give an example of code that has a race between checking for signals and sleeping.

2. What happens when a blocked signal's default action is to terminate the process, and the signal is later unblocked? When is the signal delivered?

3. What is `sigwaitinfo()`? How does it differ from installing a signal handler? What kind of program would prefer `sigwaitinfo()`?
