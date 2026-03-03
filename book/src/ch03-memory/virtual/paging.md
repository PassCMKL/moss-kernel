# AArch64 Paging

AArch64 uses a **multi-level page table** to translate virtual addresses to physical addresses. Understanding the hardware mechanism is essential before looking at Moss's software implementation.

## Why Paging?

Physical memory is a limited resource. With paging, the OS can:

1. **Isolate processes**: Each process has its own page tables, so it cannot access another process's memory.
2. **Overcommit**: Give processes more virtual memory than there is physical RAM.
3. **Share memory**: Map the same physical page into multiple processes (e.g., shared libraries).
4. **Control permissions**: Mark pages read-only, executable, no-execute, etc.

## The MMU and TLB

The **Memory Management Unit (MMU)** is a hardware unit in the CPU that intercepts every memory access and translates the virtual address to a physical address by walking the page tables. This happens transparently — software just uses virtual addresses.

Page table walks are slow (multiple memory accesses). To avoid repeating them, the MMU caches recent translations in the **Translation Lookaside Buffer (TLB)**. A TLB miss triggers a hardware page table walk; a TLB hit returns the translation immediately.

When the kernel modifies page tables, it must **flush the TLB** for affected entries to prevent stale translations.

## AArch64 4-Level Page Table

AArch64 uses a 4-level page table for 48-bit virtual addresses. Each level is an array of 512 entries (indexed by 9 bits of the virtual address).

A 48-bit virtual address is decoded as follows:

```
Bit: 47      39 38      30 29      21 20      12 11       0
     ┌─────────┬──────────┬──────────┬──────────┬──────────┐
     │  L0 idx │  L1 idx  │  L2 idx  │  L3 idx  │  offset  │
     │  (9 bit)│  (9 bit) │  (9 bit) │  (9 bit) │ (12 bit) │
     └─────────┴──────────┴──────────┴──────────┴──────────┘
```

The translation process:

```
CR3 (TTBR0/TTBR1) → L0 Table (4KB)
                         │
                    L0[bits 47:39] → L0 Entry → Physical address of L1 Table
                                                       │
                                                  L1[bits 38:30] → L1 Entry
                                                  (can be 1GB huge page)
                                                              │
                                                         L2[bits 29:21] → L2 Entry
                                                         (can be 2MB huge page)
                                                                     │
                                                                L3[bits 20:12] → Physical Page
                                                                                       +
                                                                                 offset[11:0]
                                                                                       =
                                                                                Physical Address
```

Each level reduces the space overhead compared to a flat page table. For a 48-bit address space with 4 KiB pages, a flat table would need 512 GiB of memory. With 4 levels, a process only needs tables for its actually-used memory regions.

## Page Table Entries

Each entry in a level table (L0, L1, L2, or L3) is a 64-bit value:

```
Bits    Meaning
────────────────────────────────────────
[0]     Valid (1 = entry is valid)
[1]     Table bit (1 = points to next level; 0 = block/page)
[11:2]  Attributes (see below)
[47:12] Physical address of next table or block
[63:48] More attributes (Access flag, Dirty, Privileged No-Execute, etc.)
```

Key attribute bits for a leaf entry (page):

| Bit | Meaning |
|---|---|
| UXN | User Execute Never (1 = page is not executable from EL0) |
| PXN | Privileged Execute Never (1 = page is not executable from EL1) |
| AF | Access Flag (set to 1 when page is first accessed) |
| SH | Shareability (determines TLB sharing between CPUs) |
| AP | Access Permissions (user/privileged, read-only/read-write) |
| AttrIdx | Index into MAIR_EL1 memory attribute register |

## Huge Pages

Both L1 and L2 entries can be **block descriptors** (also called huge pages) instead of pointers to the next table:

- L1 block = 1 GiB of contiguous physical memory
- L2 block = 2 MiB of contiguous physical memory

Huge pages are valuable for the kernel's linear map and for large mappings (e.g., the kernel image itself), because they require fewer page table levels and reduce TLB pressure.

## Two Page Table Registers

AArch64 has two page table base registers:

- **TTBR0_EL1**: Used for virtual addresses with bit 63 = 0 (user space)
- **TTBR1_EL1**: Used for virtual addresses with bit 63 = 1 (kernel space)

On a context switch between user processes, only TTBR0 changes (pointing to the new process's page tables). TTBR1 always points to the kernel's page tables, which are shared among all processes.

## Exercises

1. A 48-bit address space has `2^48 / 4096 = 68 billion` pages. A flat page table (one 8-byte entry per page) would need 512 GiB. How large is the actual maximum page table for a single process using Moss's 4-level scheme?

2. If a user process touches only 10 MiB of memory, how many page table pages does it actually need?

3. What does a TLB shootdown cost on a 64-core system compared to a single-core system? Why?
