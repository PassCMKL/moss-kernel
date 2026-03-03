# Exception Levels on AArch64

AArch64 defines four **Exception Levels** (ELs) that form a privilege hierarchy. Each level can only be entered via an exception from a lower level (or by explicit transition during boot).

## The Four Exception Levels

```
EL3  ──  Secure Monitor
         ● Manages Secure vs. Non-Secure world separation (TrustZone)
         ● Typically used by firmware (e.g., ARM Trusted Firmware)
         ● Not used by Moss

EL2  ──  Hypervisor
         ● Runs virtual machine managers (e.g., KVM, Xen)
         ● Can intercept and emulate hardware for guest VMs
         ● Not used by Moss (QEMU takes care of this)

EL1  ──  Kernel (Privileged OS)
         ● Where Moss runs
         ● Can access all system registers
         ● Page tables, interrupt control, cache management

EL0  ──  User Applications
         ● Unprivileged code
         ● Cannot access hardware directly
         ● Cannot modify page tables
         ● Limited to registers X0–X30 and the stack
```

Moss assumes it starts at EL1. QEMU (acting as a hypervisor or EFI firmware) sets up the initial EL level before jumping to the kernel.

## Why Multiple Levels?

The separation of levels enforces the principle of least privilege:

1. **EL0 → EL1 barrier**: User programs cannot corrupt the kernel. A buggy user program cannot modify page tables, unmask interrupts, or access other processes' memory.

2. **EL1 → EL2 barrier** (when applicable): A guest OS kernel cannot access the real hardware or the hypervisor's data structures.

3. **EL2 → EL3 barrier**: The hypervisor cannot access the secure world's secrets (cryptographic keys, DRM content).

## Entering a Higher Exception Level

The only way to move to a higher EL is through an exception:

| Exception type | Typical cause | Handler EL |
|---|---|---|
| Synchronous | SVC, page fault, undefined instruction | EL1 (from EL0) or EL2 |
| IRQ | Hardware interrupt | EL1 or EL2 |
| FIQ | High-priority interrupt | EL1 or EL3 |
| SError | System error (e.g., bus error) | EL1 or higher |

When an exception is taken from EL0 to EL1:
- The CPU saves the exception return address in `ELR_EL1`
- The CPU saves the processor state in `SPSR_EL1`
- The CPU switches the stack pointer to `SP_EL1` (the kernel stack)
- Execution jumps to the appropriate entry in the exception vector table

## Returning to a Lower Level: ERET

To return from EL1 to EL0, the kernel uses the `eret` (Exception Return) instruction:
- PC ← `ELR_EL1` (return to where the exception occurred)
- PSTATE ← `SPSR_EL1` (restore the previous processor state, including EL0 mode)

The `eret` instruction is atomic — there is no moment in time where the processor is "between" EL1 and EL0.

## System Registers at EL1

At EL1, the kernel has access to system control registers that EL0 code cannot read or write:

| Register | Purpose |
|---|---|
| `TTBR0_EL1` | Translation Table Base Register 0 (user page tables) |
| `TTBR1_EL1` | Translation Table Base Register 1 (kernel page tables) |
| `TCR_EL1` | Translation Control Register (VA size, granule, etc.) |
| `VBAR_EL1` | Vector Base Address Register (exception vector table) |
| `MAIR_EL1` | Memory Attribute Indirection Register (cache policies) |
| `SCTLR_EL1` | System Control Register (MMU on/off, caches, etc.) |
| `ESR_EL1` | Exception Syndrome Register (why did the exception occur?) |
| `FAR_EL1` | Fault Address Register (which VA caused a fault?) |
| `DAIF` | Debug, Abort, IRQ, FIQ mask bits |

## Exercises

1. Why does AArch64 need separate `SP_EL0` and `SP_EL1` stack pointers? What would happen if both ELs shared the same stack?

2. When an exception is taken from EL0 to EL1, what information does `SPSR_EL1` need to save so that `eret` can return correctly?

3. What is TrustZone, and how does EL3 enable it? Why might a mobile device's keystore use TrustZone?
