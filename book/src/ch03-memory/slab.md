# The Slab Allocator

The buddy allocator is ideal for allocating whole pages, but a kernel spends most of its time allocating much smaller objects: a 128-byte task struct, a 64-byte file descriptor, a 32-byte VMA entry. Allocating a full 4 KiB page for each of these would waste enormous amounts of memory.

The **slab allocator** solves this by pre-dividing pages into fixed-size slots, reusing them efficiently.

## Core Idea

A slab is a contiguous region of memory (one or more pages) divided into equal-sized **object slots**:

```
One slab page (4 KiB), slot size = 64 bytes:
┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬── ...
│ obj0 │ obj1 │ obj2 │ obj3 │ obj4 │ obj5 │ obj6 │
└──────┴──────┴──────┴──────┴──────┴──────┴──────┴──

Slots per page: 4096 / 64 = 64 objects
```

The slab allocator maintains a free list of available slots within each slab. Allocating an object is just: pop from the free list and return the slot's address.

## Slab Caches

Each object size has its own **slab cache**. Common sizes are powers of two: 8, 16, 32, 64, 128, 256, 512, 1024, 2048 bytes. When you call `kmalloc(48)`, the allocator rounds up to 64 and serves you from the 64-byte cache.

```
Slab Cache Structure:
┌─────────────────────────────────────────┐
│ Cache for 64-byte objects               │
│                                         │
│  Full slabs:  [slab1][slab2]            │
│  Partial:     [slab3 (32/64 used)]      │
│  Empty:       [slab4 (0/64 used)]       │
└─────────────────────────────────────────┘
```

When all objects in a partial slab are freed, the slab becomes empty. Empty slabs can be returned to the buddy allocator (freeing the underlying pages).

## Per-CPU Caches

In a multi-CPU system, having all CPUs share a single slab cache would require locking on every allocation. Moss uses **per-CPU caches** to eliminate this contention:

```
CPU 0's view:         CPU 1's view:
┌────────────┐        ┌────────────┐
│ Local cache│        │ Local cache│
│  (no lock) │        │  (no lock) │
└─────┬──────┘        └─────┬──────┘
      │                     │
      └─────────┬───────────┘
            ┌───▼────────────┐
            │ Shared slab    │
            │ (with lock)    │
            └────────────────┘
```

Each CPU maintains a small magazine (a fixed array of pointers) to recently freed objects. On allocation, it takes from the magazine without any locking. When the magazine is empty, it refills from the shared slab, taking a batch of objects to reduce lock frequency.

## Implementation in Moss

Moss's slab allocator lives in `libkernel/src/memory/allocators/`. Because it's in `libkernel`, it can be tested on the host machine without QEMU.

The slab allocator backs Rust's global allocator in the kernel:

```rust
// When Rust code does Box::new(task), this is called:
unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    SLAB_ALLOCATOR.alloc(layout.size(), layout.align())
}

unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    SLAB_ALLOCATOR.free(ptr, layout.size())
}
```

This means `Box<T>`, `Vec<T>`, `Arc<T>`, `String`, and all standard Rust heap types work in the kernel once the slab allocator is live.

## Slab Coloring

A subtle optimization in slab allocators is **coloring**: offsetting the start of object slots within each slab by a small amount. Without coloring, objects at the same position in different slabs tend to map to the same CPU cache lines, causing cache conflicts.

By starting slab N with an offset of `(N % cache_line_size)` bytes, consecutive slab accesses use different cache lines, reducing conflict misses.

Moss's slab allocator uses this technique for better cache performance.

## Summary: Allocator Hierarchy

```
Allocation Request
        │
        ▼
Size < 4 KiB? ──Yes──> Slab Allocator ──> Per-CPU cache
        │                                     │
        No                             (miss) │
        │                                     ▼
        │                              Shared slab cache
        │                                     │
        │                             (empty slab) │
        ▼                                     ▼
Buddy Allocator ◄──────────────── Request pages
```

## Exercises

1. Why is it wasteful to use the buddy allocator directly for small kernel objects like `struct Task`?

2. If a slab cache's objects are 128 bytes and a page is 4 KiB, how many objects fit per slab? What percentage of the page is wasted on alignment/metadata?

3. What is "slab reclaim" and why is it important under memory pressure? When should the kernel return empty slabs to the buddy allocator versus holding them for future use?
