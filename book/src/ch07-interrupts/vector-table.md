# The Exception Vector Table

The **exception vector table** is a page-aligned block of code at a fixed virtual address. When any exception occurs, the CPU jumps to one of 16 entries in this table based on the exception type and the current exception level.

## Structure

The vector table has four groups of four entries each:

```
Offset    Exception type             Source
────────────────────────────────────────────────────────────
+0x000    Synchronous               Current EL, SP_EL0 stack
+0x080    IRQ                       Current EL, SP_EL0 stack
+0x100    FIQ                       Current EL, SP_EL0 stack
+0x180    SError                    Current EL, SP_EL0 stack

+0x200    Synchronous               Current EL, SP_EL1 stack
+0x280    IRQ                       Current EL, SP_EL1 stack
+0x300    FIQ                       Current EL, SP_EL1 stack
+0x380    SError                    Current EL, SP_EL1 stack

+0x400    Synchronous               Lower EL (AArch64)
+0x480    IRQ                       Lower EL (AArch64)
+0x500    FIQ                       Lower EL (AArch64)
+0x580    SError                    Lower EL (AArch64)

+0x600    Synchronous               Lower EL (AArch32)
+0x680    IRQ                       Lower EL (AArch32)
+0x700    FIQ                       Lower EL (AArch32)
+0x780    SError                    Lower EL (AArch32)
```

Each entry is 128 bytes (32 instructions), giving room for a short trampoline that saves state and jumps to the actual handler.

## Moss's Key Entries

Moss primarily handles:

- **`+0x400` (EL0_Sync)**: Synchronous exceptions from user space — system calls, page faults, illegal instructions
- **`+0x480` (EL0_IRQ)**: Hardware interrupts while running user space code
- **`+0x280` (EL1_IRQ)**: Hardware interrupts while running kernel code
- **`+0x200` (EL1_Sync)**: Kernel page faults (e.g., during `copy_from_user`)

## What Each Entry Does

Each entry is a small assembly trampoline. Here is the EL0_Sync entry:

```asm
// src/arch/arm64/exceptions/exceptions.s
.balign 0x80
el0_sync:
    // Save user space registers to the kernel stack
    sub  sp, sp, #ExceptionState_SIZE
    stp  x0, x1,   [sp, #0x00]
    stp  x2, x3,   [sp, #0x10]
    stp  x4, x5,   [sp, #0x20]
    stp  x6, x7,   [sp, #0x30]
    stp  x8, x9,   [sp, #0x40]
    stp  x10, x11, [sp, #0x50]
    stp  x12, x13, [sp, #0x60]
    stp  x14, x15, [sp, #0x70]
    stp  x16, x17, [sp, #0x80]
    stp  x18, x19, [sp, #0x90]
    stp  x20, x21, [sp, #0xa0]
    stp  x22, x23, [sp, #0xb0]
    stp  x24, x25, [sp, #0xc0]
    stp  x26, x27, [sp, #0xd0]
    stp  x28, x29, [sp, #0xe0]
    str  x30,      [sp, #0xf0]

    // Save special registers
    mrs  x0, elr_el1      // Exception return address
    mrs  x1, spsr_el1     // Saved processor state
    mrs  x2, sp_el0       // User stack pointer
    mrs  x3, tpidr_el0    // TLS pointer
    stp  x0, x1, [sp, #ExceptionState_ELR_OFFSET]
    stp  x2, x3, [sp, #ExceptionState_SP_OFFSET]

    // Call Rust handler with pointer to the saved state
    mov  x0, sp
    bl   handle_el0_sync
```

## Exception Class Determination

Inside `handle_el0_sync`, the kernel reads the Exception Syndrome Register (`ESR_EL1`) to determine what type of exception occurred:

```rust
pub fn handle_el0_sync(state: &mut ExceptionState) {
    let esr = read_sysreg!(esr_el1);
    let ec = (esr >> 26) & 0x3f;  // Exception Class

    match ec {
        0x15 => handle_syscall(state),     // SVC from AArch64
        0x20 | 0x21 => handle_inst_fault(state),  // Instruction abort
        0x24 | 0x25 => handle_data_fault(state),  // Data abort (page fault)
        0x07 => handle_fp_access(state),   // FP/SIMD access
        _ => {
            error!("Unknown EL0 sync exception: EC={:#x}", ec);
            deliver_signal(SIGILL);
        }
    }
}
```

## Installing the Vector Table

The vector table address is loaded into `VBAR_EL1` during Stage 2:

```rust
// Install exception vector table
let vbar = &raw const EXCEPTION_VECTORS as u64;
unsafe { write_sysreg!(vbar_el1, vbar) };
// Instruction barrier: ensure the write is visible before we execute anything
unsafe { asm!("isb") };
```

After this, any exception will jump to the correct entry.

## Exercises

1. Why must each vector table entry be exactly 128 bytes? What would happen if an entry needed more code?

2. What is the difference between an IRQ taken from EL0 and one taken from EL1? Why does Moss need separate handlers for these?

3. The vector table is at a fixed virtual address. What security advantage does randomizing this address (via KASLR) provide?
