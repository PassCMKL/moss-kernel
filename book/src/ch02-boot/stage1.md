# Stage 1: Early Initialization

`arch_init_stage1()` is the first Rust function called during boot. It runs with a minimal temporary stack and with the MMU partially configured. Its job is to create a safe environment for the rest of the kernel.

## What Stage 1 Must Accomplish

At the start of Stage 1, the hardware state is:

- MMU: partially on — only an identity map (physical == virtual) for kernel code
- Memory allocator: none
- Stack: a small temporary stack set up by assembly
- Devices: not initialized
- Heap: none

By the end of Stage 1, the kernel must have:

- A stable kernel virtual address space (TTBR1 set up)
- The DTB remapped into the fixmap region
- A working early memory allocator (Smalloc)
- A new kernel stack

## Step-by-Step Walkthrough

### 1. Parse the Device Tree Blob

The DTB is passed as a physical address in `X0` by the bootloader. Stage 1's first job is to parse it just enough to discover physical memory regions:

```rust
// Simplified view
let dtb_pa = PA(x0);
let memory_regions = parse_dtb_memory(dtb_pa);
```

The DTB parser walks the device tree structure, looking for `memory` nodes that describe the available RAM.

### 2. Set Up the Identity Map (TTBR0)

AArch64 has two Translation Table Base Registers:
- **TTBR0**: Maps the lower half of virtual address space (user addresses, `0x0000_...`)
- **TTBR1**: Maps the upper half (kernel addresses, `0xffff_...`)

During Stage 1, the kernel code is running via an identity map loaded into TTBR0 — virtual addresses equal physical addresses. This works because the kernel image is loaded at a physical address that happens to also be a valid virtual address at this stage.

### 3. Build the Kernel Virtual Address Space (TTBR1)

The long-term kernel virtual address space is built in TTBR1. This is the kernel's permanent home in the upper half of the 64-bit address space:

```
0xffff_8000_0000_0000  ─  Kernel image (.text, .data, .bss)
0xffff_9000_0000_0000  ─  Fixmap region (temporary mappings)
0xffff_b800_0000_0000  ─  Per-CPU kernel stacks
0xffff_d000_0000_0000  ─  MMIO remapping region
0xffff_e000_0000_0000  ─  Exception vector table
```

Moss creates the TTBR1 page tables by hand during Stage 1, using statically allocated memory (since the allocator doesn't exist yet).

### 4. Remap the DTB into the Fixmap

The DTB is needed throughout early boot. The fixmap is a small region of virtual address space reserved for temporary kernel mappings. The DTB is remapped there so it can be accessed via a stable virtual address after the identity map is removed.

### 5. Initialize Smalloc (Early Allocator)

Before the frame allocator can be set up, the kernel needs some memory to hold data structures. **Smalloc** is a very simple allocator backed by statically allocated arrays. It provides `alloc()` and nothing else — it cannot free memory.

Smalloc's purpose is to allocate the initial data structures needed to *set up* the real allocator, which happens in Stage 2.

### 6. Return the New Stack Pointer

At the end of Stage 1, a proper per-CPU kernel stack has been mapped in the TTBR1 address space. Stage 1 returns this new stack pointer to the assembly code, which switches to it before calling Stage 2.

```asm
// In start.s (simplified)
bl      arch_init_stage1   // returns new SP in X0
mov     sp, x0             // switch to new stack
bl      arch_init_stage2
```

## Why Two Stages?

Stage 1 operates with almost no infrastructure. It cannot use dynamic allocation, cannot print to a console (the UART driver isn't initialized), and cannot handle exceptions (the exception vector table isn't set up). The split allows us to clearly separate:

- **Stage 1**: Minimum viable environment (MMU, stack, early allocator)
- **Stage 2**: Full kernel initialization using the infrastructure from Stage 1

## Source Reference

The complete implementation lives in:
- `src/arch/arm64/boot/mod.rs` — Stage 1 Rust code
- `src/arch/arm64/boot/start.s` — Assembly entry point
- `src/arch/arm64/memory/mmu.rs` — Page table construction

## Exercises

1. Why must the identity map in TTBR0 be kept active during Stage 1? When can it be safely removed?

2. The kernel image is linked to a virtual address in the upper half (`0xffff_8000_...`) but loaded at a lower physical address. How does the identity map bridge this gap?

3. What would happen if Moss tried to call `println!()` during Stage 1 before the UART driver is initialized?
