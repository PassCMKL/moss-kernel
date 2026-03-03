# The User–Kernel Boundary

Every interaction between a user program and the kernel crosses a **privilege boundary**. This boundary is enforced by hardware and is the foundation of process isolation.

## Why the Boundary Exists

User code runs at **EL0** (Exception Level 0) on AArch64 — the lowest privilege level. At EL0:
- The MMU is active with user-space page tables
- Privileged instructions (e.g., modifying page tables, writing MMIO registers) are illegal and cause faults
- Kernel memory is mapped but not accessible (marked as privileged-only in page table attributes)

Kernel code runs at **EL1** (Exception Level 1):
- All memory is accessible
- Privileged instructions work normally
- Full hardware control

This separation means a bug in a user program cannot corrupt the kernel or other processes.

## Crossing the Boundary: SVC

The only way for user code to enter the kernel intentionally is via the `svc` (Supervisor Call) instruction:

```asm
// User program performing a syscall (e.g., write(1, "hello\n", 6))
mov x8, #64        // Syscall number: __NR_write = 64
mov x0, #1         // Argument 0: fd = 1 (stdout)
adr x1, msg        // Argument 1: buf = pointer to "hello\n"
mov x2, #6         // Argument 2: count = 6
svc #0             // Enter kernel
// Kernel executes write syscall, returns to here
// Return value is now in X0
```

When `svc #0` executes:
1. The CPU atomically switches to EL1
2. The PC jumps to the EL0_Sync entry in the exception vector table
3. The CPU saves the user-space return address in `ELR_EL1`
4. The CPU saves the user-space processor state in `SPSR_EL1`

## The Syscall Convention (AArch64 Linux ABI)

Moss follows the standard Linux AArch64 syscall convention:

| Register | Role |
|---|---|
| X8 | Syscall number |
| X0 | Argument 0 (also return value) |
| X1 | Argument 1 |
| X2 | Argument 2 |
| X3 | Argument 3 |
| X4 | Argument 4 |
| X5 | Argument 5 |
| X0 (after) | Return value (negative = error code) |

This matches Linux exactly, which is why programs compiled for Linux work on Moss.

## Error Handling Convention

Syscalls return a single 64-bit value in X0:
- Non-negative value: success (specific meaning depends on the syscall)
- `-1` to `-4095`: failure (the negated `errno` value)

User-space libc wrappers interpret negative return values and set the global `errno` variable:

```c
// libc wrapper for read():
ssize_t read(int fd, void *buf, size_t count) {
    ssize_t ret = syscall(__NR_read, fd, buf, count);
    if (ret < 0) {
        errno = -ret;  // e.g., -22 → errno = EINVAL
        return -1;
    }
    return ret;
}
```

## Returning to User Space

After the kernel handles the syscall, it needs to return to user space at the instruction after `svc #0`. On AArch64, this uses the `eret` (Exception Return) instruction:

```asm
// In kernel exception return path:
eret   // Restores PC from ELR_EL1, status from SPSR_EL1, switches to EL0
```

`eret` is the reverse of the exception entry: it atomically:
1. Restores the PC from `ELR_EL1`
2. Restores the processor state from `SPSR_EL1`
3. Switches back to EL0

Before `eret`, the kernel sets X0 to the return value and checks for pending signals and rescheduling.

## Exercises

1. Why must the syscall boundary be enforced by hardware rather than by software convention?

2. What happens if a user program executes a privileged instruction (like writing to TTBR0) at EL0?

3. Some architectures use a different mechanism for syscalls (e.g., x86 uses `syscall`/`sysret` instructions). What properties are common across all syscall mechanisms?
