# Signal Handlers

A **signal handler** is a user-space function that the kernel calls when a signal is delivered. Signal handlers allow programs to respond to events (like `SIGINT`) rather than accepting the default action (termination).

## Installing a Signal Handler

```c
// C code (user space)
#include <signal.h>

void my_sigint_handler(int signum) {
    write(1, "Caught SIGINT!\n", 15);
    // Don't call printf — it's not async-signal-safe!
}

int main() {
    struct sigaction sa = {
        .sa_handler = my_sigint_handler,
        .sa_flags = SA_RESTART,
    };
    sigemptyset(&sa.sa_mask);
    sigaction(SIGINT, &sa, NULL);

    // Now Ctrl+C calls my_sigint_handler instead of terminating
    while (1) pause();
}
```

In the kernel, `sigaction` stores the handler in the thread group's signal action table:

```rust
pub struct SignalActionState {
    actions: [SigAction; NSIG],
}

pub enum SigAction {
    Default,                  // Default kernel action
    Ignore,                   // Silently discard
    Handler(UserFn),          // User-space function pointer
    RestartableHandler(UserFn), // Same, but syscalls auto-restart
}
```

## How Signal Handlers Are Called

Calling a user-space function from the kernel is non-trivial. The CPU is currently in kernel mode (EL1) and the signal handler is in user space (EL0). The kernel cannot simply call it — it must set up a fake stack frame so the CPU returns to the handler.

### Setting Up the Signal Frame

Before returning to user space, the kernel modifies the saved `ExceptionState` on the kernel stack:

```rust
fn setup_user_signal_frame(
    state: &mut ExceptionState,
    sig: Signal,
    handler: UserFn,
) {
    // Save the original user-space context to the user's stack
    // so the handler can return and the original code can resume
    let user_sp = state.sp_el0;
    let sig_frame = SignalFrame {
        saved_context: *state,  // The original registers
        sig: sig as u32,
    };

    // Push the signal frame onto the user's stack
    let new_sp = user_sp - size_of::<SignalFrame>();
    copy_to_user(UA(new_sp), &sig_frame).unwrap();

    // Make the CPU "return" to the signal handler
    state.elr_el1 = handler.0;   // Handler's address becomes the return PC
    state.sp_el0 = new_sp;        // Updated stack pointer
    state.x[0] = sig as u64;      // Signal number in X0 (first argument)

    // Set X30 (link register) to the signal return trampoline
    // (so the handler "returns" to rt_sigreturn)
    state.x[30] = SIG_RETURN_TRAMPOLINE;
}
```

After `eret`, the CPU jumps to the signal handler function with:
- `X0` = signal number
- `SP` pointing to the signal frame on the user stack
- `X30` = address of the return trampoline

### Returning from the Handler

When the handler returns, it returns to the signal return trampoline (code in the vDSO):

```asm
// vDSO trampoline code
sig_return_trampoline:
    mov x8, #139       // __NR_rt_sigreturn
    svc #0             // Call rt_sigreturn syscall
```

The `rt_sigreturn` syscall:
1. Reads the `SignalFrame` from the user's stack
2. Restores the original registers from it
3. Returns to the original code as if the signal never happened

```rust
pub fn sys_rt_sigreturn(state: &mut ExceptionState) {
    // Read the signal frame from the user's stack
    let sp = state.sp_el0;
    let sig_frame: SignalFrame = read_user_struct(UA(sp)).unwrap();

    // Restore the original context
    *state = sig_frame.saved_context;

    // Continue execution at the original instruction
    // (state.elr_el1 now points to where we were interrupted)
}
```

## Async Signal Safety

Signal handlers can interrupt any point in a program's execution — including library functions like `malloc` or `printf`. If a signal handler calls the same function that was interrupted, the function's state may be corrupted.

**Async-signal-safe functions** are those that can be called from signal handlers. The POSIX standard defines a specific list, which includes:
- `read`, `write`, `close`
- `signal`, `sigaction`
- `exit`, `_exit`
- `kill`

Notably NOT async-signal-safe:
- `malloc` / `free` (uses internal locks)
- `printf` (uses internal buffers)
- `errno` (not technically, but implementations vary)

## The `SA_RESTART` Flag

When a signal interrupts a blocking syscall, the syscall normally returns `-EINTR`. The `SA_RESTART` flag changes this: the kernel automatically restarts the syscall after the signal handler returns.

This is usually the desired behavior — a `read()` waiting for data should not fail just because a signal arrived.

## Exercises

1. Why can't `printf` be called safely from a signal handler? What specifically would break?

2. What is the alternative signal stack (`sigaltstack`)? When is it necessary?

3. A signal handler sets a global flag: `volatile sig_atomic_t got_sigint = 1;`. Why is `volatile` necessary here? What is `sig_atomic_t`?
