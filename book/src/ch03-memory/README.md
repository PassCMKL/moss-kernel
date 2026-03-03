# Chapter 3: Memory Management

Memory management is the heart of any operating system. The kernel must:

1. Track which physical memory (RAM) is in use and which is free
2. Create the illusion that each process has its own private address space
3. Handle faults when a process accesses a page that isn't yet loaded
4. Protect processes from each other — one process cannot read another's memory

This chapter covers both **physical memory** management (how the kernel tracks RAM) and **virtual memory** (how the kernel creates isolated per-process address spaces).

## Learning Objectives

By the end of this chapter you should be able to:

- Explain the difference between physical and virtual addresses
- Describe how a buddy allocator works
- Draw the AArch64 page table hierarchy
- Trace the path from a virtual address to a physical address
- Explain copy-on-write and demand paging

## Structure

This chapter is split into two major sections:

**Physical Memory:**
- [The Memory Layout](./layout.md)
- [The Early Allocator (Smalloc)](./smalloc.md)
- [The Buddy Allocator](./buddy.md)
- [The Slab Allocator](./slab.md)

**Virtual Memory:**
- [AArch64 Paging](./virtual/paging.md)
- [Page Tables in Moss](./virtual/page-tables.md)
- [Address Types: VA, PA, UA](./virtual/address-types.md)
- [Page Fault Handling](./virtual/page-faults.md)
- [Copy-on-Write](./virtual/cow.md)
- [Kernel–User Data Transfers](./virtual/copy-user.md)
