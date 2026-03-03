# Virtual Memory

Virtual memory is one of the most important abstractions an operating system provides. Each process believes it has exclusive access to a large, flat address space — typically the full 48-bit lower half on AArch64. In reality, physical RAM is shared among all processes, and the hardware MMU translates each process's virtual addresses to physical addresses on the fly.

This section covers:

- [AArch64 Paging](./paging.md) — how the hardware translates addresses
- [Page Tables in Moss](./page-tables.md) — Moss's page table implementation
- [Address Types: VA, PA, UA](./address-types.md) — Rust types for safe address manipulation
- [Page Fault Handling](./page-faults.md) — what happens when a page isn't present
- [Copy-on-Write](./cow.md) — sharing pages efficiently
- [Kernel–User Data Transfers](./copy-user.md) — safely moving data across the privilege boundary
