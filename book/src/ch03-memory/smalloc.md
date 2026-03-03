# The Early Allocator (Smalloc)

When the kernel first starts, there is no memory allocator. But setting up the real allocator requires allocating memory. This chicken-and-egg problem is solved by **Smalloc** — a tiny, static, non-freeing bootstrap allocator.

## Design

Smalloc is as simple as an allocator can be:

```rust
struct Smalloc {
    memory: [u8; SMALLOC_SIZE],  // Statically allocated buffer in .bss
    offset: usize,               // Current allocation pointer
}

impl Smalloc {
    pub fn alloc(&mut self, size: usize, align: usize) -> *mut u8 {
        // Round up to alignment
        let aligned = (self.offset + align - 1) & !(align - 1);
        let new_offset = aligned + size;
        assert!(new_offset <= SMALLOC_SIZE, "Smalloc exhausted");
        self.offset = new_offset;
        &mut self.memory[aligned]
    }
    // No free() — memory is never reclaimed
}
```

Allocations are made by bumping a pointer forward. There is no `free()` because:
1. The allocations made during early boot are permanent (page tables, per-CPU data)
2. Implementing `free()` would require metadata and complexity we don't yet have

## What Smalloc Is Used For

Smalloc provides memory for exactly the structures that the frame allocator needs to track itself:

- The **frame allocator's metadata array** (one entry per physical page)
- Initial **page table pages** for the kernel address space
- The **DTB copy** in the fixmap region

Once the frame allocator is initialized, Smalloc is no longer used. All subsequent allocations go through the frame allocator or slab allocator.

## Limitations

- Fixed size: Smalloc's backing buffer is statically sized. If early boot needs more memory than this buffer provides, the kernel panics.
- No freeing: all Smalloc memory is permanently allocated
- Thread safety: Smalloc is only used on the primary CPU during single-threaded boot

## The Bump Allocator Pattern

Smalloc is an instance of a well-known pattern called a **bump allocator** (or arena allocator):

```
Initial state:
┌────────────────────────────────┐
│          free                  │
└────────────────────────────────┘
↑ offset = 0

After first allocation (16 bytes):
┌────────┬───────────────────────┐
│ alloc1 │       free            │
└────────┴───────────────────────┘
         ↑ offset = 16

After second allocation (32 bytes):
┌────────┬────────────────┬──────┐
│ alloc1 │    alloc2      │ free │
└────────┴────────────────┴──────┘
                          ↑ offset = 48
```

Bump allocators are extremely fast (just a pointer comparison and increment) and are used in many performance-sensitive contexts beyond OS kernels — game engines, web servers, and compilers all use arena allocation for temporary data.

## Exercises

1. What is the advantage of a bump allocator over `malloc`? What is the disadvantage?

2. How large should `SMALLOC_SIZE` be? What happens if it's too small? Too large?

3. An alternative to Smalloc is to use a portion of the physical memory that the DTB told us about, before setting up any page tables. What complications would this approach have?
