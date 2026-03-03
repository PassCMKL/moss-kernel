# Saving and Restoring State

When an exception occurs, the CPU's current register values must be saved so that, after the exception is handled, the original code can resume exactly where it left off. This is called **context saving**.

## What Needs to Be Saved

On AArch64, the following state must be saved when an exception is taken from EL0:

| State | Register/Location | Saved by |
|---|---|---|
| General registers X0–X30 | Stack | Software (assembly) |
| Program counter | `ELR_EL1` | Hardware automatically |
| Processor state (PSTATE) | `SPSR_EL1` | Hardware automatically |
| User stack pointer | `SP_EL0` | Software |
| Thread-local storage pointer | `TPIDR_EL0` | Software |

The hardware saves `ELR_EL1` and `SPSR_EL1` automatically. Everything else must be saved by the exception entry assembly code before calling the Rust handler.

## The `ExceptionState` Struct

```rust
#[repr(C)]  // Must match the assembly layout exactly
pub struct ExceptionState {
    pub x: [u64; 31],     // X0–X30 (31 registers × 8 bytes = 248 bytes)
    pub elr_el1: u64,     // Exception Link Register
    pub spsr_el1: u64,    // Saved Program Status Register
    pub sp_el0: u64,      // User stack pointer
    pub tpid_el0: u64,    // TLS pointer
}
// Total: 31*8 + 4*8 = 280 bytes
```

This struct is placed directly on the **kernel stack** when an exception occurs. The assembly entry code builds it by pushing registers to the stack in the exact order the struct expects.

## Why `#[repr(C)]`?

By default, Rust may reorder struct fields for efficiency. `#[repr(C)]` forces the fields to be in declaration order with C-compatible alignment. This is essential here because the assembly code accesses specific offsets into the struct:

```asm
// Assembly accesses ExceptionState by fixed offsets
str x30, [sp, #240]        // X30 at offset 240 (= 30 * 8)
stp x0, x1, [sp, #280]    // ELR_EL1 and SPSR_EL1 at offset 280
```

If Rust reordered the fields, these offsets would be wrong.

## The Save Operation

The assembly saves registers in a tight loop using `stp` (Store Pair) instructions, which write two registers to memory in a single instruction:

```asm
stp  x0, x1,   [sp, #0]
stp  x2, x3,   [sp, #16]
// ... and so on
```

This is carefully unrolled to minimize overhead. On AArch64, there is no instruction that saves all registers at once (unlike x86's `pusha`), so each pair must be saved explicitly.

## The Restore Operation

Restoring state is the mirror image. Before `eret`, the kernel loads user registers from the `ExceptionState` on the stack:

```asm
// Load special registers
ldp  x0, x1, [sp, #ExceptionState_ELR_OFFSET]
msr  elr_el1, x0    // Restore return address
msr  spsr_el1, x1   // Restore processor state
ldp  x2, x3, [sp, #ExceptionState_SP_OFFSET]
msr  sp_el0, x2     // Restore user stack pointer
msr  tpidr_el0, x3  // Restore TLS pointer

// Load general registers
ldp  x0, x1,   [sp, #0]
ldp  x2, x3,   [sp, #16]
// ...
ldr  x30, [sp, #240]
add  sp, sp, #ExceptionState_SIZE

// Return to user space
eret
```

The `add sp, sp, #ExceptionState_SIZE` deallocates the saved state from the kernel stack. After `eret`, the CPU is back at EL0 with the original register values.

## Modifying Saved State

Because `ExceptionState` is on the kernel stack and passed to Rust as a mutable reference, the kernel can modify the saved register values before returning to user space.

This is how syscall results are communicated:

```rust
pub fn complete_syscall(state: &mut ExceptionState, result: i64) {
    // Write the result into the saved X0 — it will be restored on eret
    state.x[0] = result as u64;
}
```

And how `fork` gives different return values to parent and child:

```rust
// Child gets 0 from fork
fn setup_fork_child(child_state: &mut ExceptionState) {
    child_state.x[0] = 0;  // Fork returns 0 to child
}
```

## The Overhead Cost

Saving and restoring 35 registers (31 general + 4 special) on every exception entry/exit takes time — roughly 30-60 nanoseconds on a modern ARM core. This is the fundamental cost of the user–kernel boundary.

Optimizations like vDSO reduce the frequency of syscalls for common operations (clock reads, etc.) to minimize this overhead.

## Exercises

1. What is `SPSR_EL1` used for? What specific bits does it contain, and why are they needed for `eret` to work correctly?

2. Why is it important to save X30 (the link register)? What would happen if it were not saved?

3. When the kernel modifies a signal handler's saved state (to make it look like the signal handler was called), which saved registers does it need to change? Why?
