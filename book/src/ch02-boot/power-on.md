# From Power-On to Kernel

## The Role of the Bootloader

When a computer is powered on, the CPU begins executing instructions at a hardwired address — typically in read-only firmware (BIOS, UEFI, or an embedded ROM). On a system running Moss under QEMU, this firmware is **QEMU's built-in UEFI/EFI**, which performs basic hardware discovery and then loads a **bootloader**.

For Moss, the bootloader is typically GRUB or the EFI stub loader embedded in the kernel image itself. The bootloader's job is to:

1. Load the kernel binary into memory at the correct address
2. Load the **Device Tree Blob (DTB)** — a structured description of the hardware
3. Pass control to the kernel's entry point with the DTB address in a register

## The Device Tree Blob

On ARM systems, there is no standardized mechanism (like PCI configuration space on x86) for hardware to announce itself to the OS. Instead, a **Device Tree Blob (DTB)** describes the hardware topology: where the RAM is, what peripherals exist, what interrupt lines they use, what base addresses they're mapped at.

The bootloader places the DTB at a known physical address and passes this address to the kernel. Moss reads the DTB during early boot to discover:

- Physical memory regions
- UART (serial console) base address
- Interrupt controller (GIC) base address
- Timer parameters

## The Entry Point: `start.s`

The kernel's entry point is AArch64 assembly code in `src/arch/arm64/boot/start.s`. When the bootloader jumps to the kernel, the CPU is at EL1 (kernel privilege level) with:

- The MMU **disabled** — all addresses are physical
- No valid stack — the stack pointer is garbage
- `X0` holding the DTB physical address
- Minimal CPU state otherwise initialized

The assembly entry point must establish enough of a runtime environment for Rust code to execute before it can hand off to Rust.

## Why Assembly?

The very first instructions cannot be written in Rust because:

1. Rust code requires a valid stack — the assembly must set one up first
2. Rust code may require static data to be in certain memory locations — the MMU must be configured before Rust's memory model makes sense
3. Setting the stack pointer and configuring the MMU requires privileged register writes that are more naturally expressed in assembly

Once the stack is set up and the MMU is enabled, control passes to Rust.

## A Simplified View of the Boot Sequence

```
Power on
  └─> Firmware (QEMU EFI)
        └─> Bootloader
              └─> start.s (assembly entry point)
                    ├─ Save DTB address from X0
                    ├─ Set up temporary stack
                    ├─ arch_init_stage1() ── sets up MMU, returns new SP
                    ├─ Switch to new stack pointer
                    └─ arch_init_stage2() ── full kernel init
                          └─ kmain() ── first user process
```

The next two sections walk through Stage 1 and Stage 2 in detail.

## Exercises

1. What physical address does QEMU's EFI firmware typically load the AArch64 kernel to? (Hint: look at QEMU's documentation for the `virt` machine type.)

2. Why is the MMU disabled when the bootloader first jumps to the kernel? What problems would arise if the MMU were already enabled with an unknown page table?

3. What information might a Device Tree Blob contain that a kernel absolutely needs before it can do anything useful?
