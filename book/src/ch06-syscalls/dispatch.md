# Syscall Dispatch

When a user program executes `svc #0`, control transfers to the kernel's exception handler. This section traces the exact path from hardware entry to the Rust syscall function.

## The Exception Entry Path

The entry point for EL0 synchronous exceptions (which includes syscalls) is in `src/arch/arm64/exceptions/exceptions.s`. The assembly:

1. Saves all user registers to the stack
2. Reads the exception syndrome register to determine if this is a syscall
3. Calls the Rust handler

```asm
// EL0_Sync entry (simplified)
el0_sync:
    // Save all user registers (X0–X30, SP_EL0, ELR_EL1, SPSR_EL1)
    sub sp, sp, #ExceptionState_SIZE
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    // ... save all registers ...

    // Read exception class from ESR_EL1
    mrs x0, esr_el1
    lsr x0, x0, #26          // Extract EC (Exception Class) bits
    cmp x0, #0x15            // 0x15 = SVC instruction from AArch64
    b.eq handle_syscall_entry

    // Not a syscall — handle as page fault or other exception
    b handle_exception
```

## Saving User State

Before any kernel code can run, the complete user register state must be saved. The `ExceptionState` struct on the kernel stack captures everything needed to return to user space:

```rust
#[repr(C)]
pub struct ExceptionState {
    pub x: [u64; 31],     // X0–X30 general purpose registers
    pub elr_el1: u64,     // Exception Link Register (user return address)
    pub spsr_el1: u64,    // Saved Program Status Register
    pub sp_el0: u64,      // User stack pointer
    pub tpid_el0: u64,    // Thread-Local Storage pointer
}
```

## The Rust Dispatch Function

After saving state, the assembly calls `handle_syscall()` with the `ExceptionState` pointer:

```rust
// src/arch/arm64/exceptions/mod.rs
pub async fn handle_syscall(state: &mut ExceptionState) {
    let nr = state.x[8] as usize;  // X8 = syscall number

    let result: i64 = match nr {
        // Process management
        NR_EXIT       => sys_exit(state.x[0] as i32).await,
        NR_FORK       => sys_fork(state).await,
        NR_EXECVE     => sys_execve(
            UA(state.x[0] as usize),  // path
            UA(state.x[1] as usize),  // argv
            UA(state.x[2] as usize),  // envp
        ).await,

        // File I/O
        NR_READ  => sys_read(state.x[0] as i32,
                             UA(state.x[1] as usize),
                             state.x[2] as usize).await,
        NR_WRITE => sys_write(state.x[0] as i32,
                              UA(state.x[1] as usize),
                              state.x[2] as usize).await,
        NR_OPEN  => sys_open(UA(state.x[0] as usize),
                             state.x[1] as i32,
                             state.x[2] as u32).await,

        // ... 100+ more syscalls ...

        // Unknown syscall
        _ => {
            warn!("Unknown syscall {}", nr);
            -ENOSYS as i64
        }
    };

    // Store return value in X0 for return to user space
    state.x[0] = result as u64;
}
```

## Syscall Numbers

Syscall numbers are defined by the Linux AArch64 ABI. Moss uses the same numbers so that programs compiled for Linux work without modification:

```rust
// Subset of Linux AArch64 syscall numbers
pub const NR_READ:     usize = 63;
pub const NR_WRITE:    usize = 64;
pub const NR_OPEN:     usize = 56; // openat
pub const NR_CLOSE:    usize = 57;
pub const NR_EXIT:     usize = 93;
pub const NR_FORK:     usize = 1079;
pub const NR_EXECVE:   usize = 221;
pub const NR_MMAP:     usize = 222;
// ...
```

## Kernel Work Queue

Syscall handlers in Moss are `async fn`. After saving user state, the kernel does not directly call the handler. Instead, it spawns the handler as a kernel **work item**:

```rust
// After saving exception state:
let work = async move {
    handle_syscall(&mut state).await;
    // On completion, restore state and return to user space
    dispatch_userspace_task(state);
};

// Enqueue the work item — the scheduler will run it
kernel_work_queue().push(work);
```

This allows the kernel to handle multiple concurrent syscalls (even from the same CPU) by interleaving them as async tasks.

## The Return Path

After the syscall completes, `dispatch_userspace_task()` restores user state and executes `eret`:

```rust
pub fn dispatch_userspace_task(state: ExceptionState) -> ! {
    // Check for pending signals before returning
    deliver_pending_signals();

    // Check if we should reschedule
    if need_resched() {
        schedule();
    }

    // Restore user registers and return to EL0
    unsafe { restore_and_eret(state) }
}
```

The final `eret` is the mirror of the `svc` that started the syscall — it atomically restores the user context and returns to EL0.

## Exercises

1. What would happen if the kernel forgot to save X0–X7 before calling the syscall handler? How would this manifest as a user-visible bug?

2. Why is the syscall number in X8 rather than X0? What happens to X0 after the syscall?

3. A user program calls `read(fd, buf, 1024)` and the kernel needs to perform disk I/O. Trace the complete path: from user's `read()` call through the kernel, through sleeping, through waking up, and back to user space.
