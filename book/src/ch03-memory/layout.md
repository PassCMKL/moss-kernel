# The Memory Layout

Before examining the allocators, let's understand how Moss organizes its virtual address space.

## The 64-Bit Address Space Split

AArch64 provides a 64-bit virtual address space, but current hardware only uses 48 bits (256 TiB). Critically, the address space is split into two halves based on the top bits of the address:

- **Lower half** (`0x0000_0000_0000_0000` – `0x0000_7fff_ffff_ffff`): User space. Mapped via **TTBR0_EL1**.
- **Upper half** (`0xffff_0000_0000_0000` – `0xffff_ffff_ffff_ffff`): Kernel space. Mapped via **TTBR1_EL1**.

This is enforced by hardware. User processes can only access the lower half. Any attempt to access a kernel address from user space causes an immediate fault.

## Kernel Virtual Memory Map

Moss divides the kernel half of the address space into distinct regions:

```
Virtual Address                      Region
─────────────────────────────────────────────────────────────
0xffff_0000_0000_0000
  │                    Logical Memory Map (direct physical map)
  │                    Every physical page is accessible here
  │                    VA = 0xffff_0000_0000_0000 + PA
0xffff_8000_0000_0000
  │                    Kernel Image (.text, .data, .bss, .rodata)
  │                    ~512 MiB window
0xffff_8100_0000_0000
  │                    VDSO (user-accessible kernel page)
  │                    Mapped into every process's address space
0xffff_9000_0000_0000
  │                    Fixmap Region
  │                    Small window for temporary kernel mappings
  │                    (used during boot for the DTB, etc.)
0xffff_b800_0000_0000
  │                    Per-CPU Kernel Stacks
  │                    One 32 KiB guard-paged stack per CPU
0xffff_d000_0000_0000
  │                    MMIO Remap Region
  │                    Device memory (UART, GIC, etc.) mapped here
0xffff_e000_0000_0000
  │                    Exception Vector Table
  │                    Page-aligned, 4 KiB
0xffff_f000_0000_0000
  └─                   (unused / reserved)
```

## The Logical Memory Map

The most important region is the **logical memory map** at the bottom of the kernel address space. This is a direct mapping of all physical memory: physical address `P` is accessible at virtual address `0xffff_0000_0000_0000 + P`.

This means the kernel can access any physical page without needing to set up a special mapping first. It is the foundation of the frame allocator, the slab allocator, and virtually all kernel memory access.

Converting between physical and virtual addresses in this region is trivial:

```rust
pub fn pa_to_va(pa: PA) -> VA {
    VA(pa.0 + PHYSICAL_MAP_BASE)
}

pub fn va_to_pa(va: VA) -> PA {
    PA(va.0 - PHYSICAL_MAP_BASE)
}
```

This is sometimes called a **linear map** or **direct map**.

## User Space Layout

Each process has its own virtual address space in the lower half. Moss sets up user space with a conventional Unix layout:

```
Virtual Address         Region
──────────────────────────────────────────────────
0x0000_0000_0000_0000   NULL page (unmapped, catches null deref)
0x0000_0000_0001_0000   Program text and data (load address varies)
                        Dynamic linker loads main binary here
0x0000_5000_0000_0000   Program segments (with ASLR bias)
0x0000_7000_0000_0000   Dynamic library regions (libc, etc.)
0x0000_7fff_f000_0000   User stack (grows downward)
0x0000_7fff_ffff_f000   vDSO (kernel-provided shared library)
```

The actual addresses are randomized with ASLR (Address Space Layout Randomization) to make exploitation harder. Moss applies a fixed bias for dynamic-linked programs:
- Main program: `0x5000_0000_0000`
- Dynamic libraries (libc): `0x7000_0000_0000`

## Exercises

1. Why is the kernel mapped in the upper half of the address space and user programs in the lower half? What would go wrong if they overlapped?

2. Convert the physical address `0x4000_1000` to its logical memory map virtual address (assuming `PHYSICAL_MAP_BASE = 0xffff_0000_0000_0000`).

3. What is ASLR? Why does it make exploitation harder? Can it be defeated, and if so, how?
