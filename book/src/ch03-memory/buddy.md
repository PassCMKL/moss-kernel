# The Buddy Allocator

The buddy allocator is the primary allocator for **physical pages** in Moss. It replaces Smalloc once the kernel has enough infrastructure to manage the full physical address space.

## The Problem with Simple Allocators

Suppose we have 1 GiB of RAM and need to allocate and free pages of different sizes in arbitrary order. A bump allocator cannot free memory. A linked-list allocator is slow (O(n) search). We need something that is:

- Fast to allocate (close to O(1))
- Fast to free (and merge adjacent free blocks)
- Low fragmentation

The buddy allocator achieves all three.

## How the Buddy Allocator Works

The buddy allocator manages memory as a hierarchy of **orders**. Order 0 is one page (4 KiB). Order 1 is two contiguous pages (8 KiB). Order N is 2^N contiguous pages.

```
Physical memory (16 pages total):

Order 3 (32KB): [page0...page7][page8...page15]
Order 2 (16KB): [page0..page3][page4..page7][page8..page11][page12..page15]
Order 1 (8KB):  [p0,p1][p2,p3][p4,p5][p6,p7][p8,p9]...
Order 0 (4KB):  [p0][p1][p2][p3][p4][p5][p6][p7]...
```

Each order has a **free list** of blocks of that size. To allocate 8 KiB (order 1):

1. Check the order-1 free list. If there's a block, return it.
2. If not, check order 2. Split the 16 KiB block into two 8 KiB buddies. Add one to the free list, return the other.
3. If order 2 is empty too, go to order 3. Split it, and so on.

### Freeing: The Key Innovation

When a block is freed, the allocator checks if its **buddy** is also free. Two blocks are buddies if:
- They are the same size
- Their starting addresses differ by exactly their size
- They would form a valid larger block if merged

```
Block at page 2 (order 0) is freed.
Its buddy is page 3 (they'd form an order-1 block at page 2).
Is page 3 free? Yes!
→ Merge to form a free order-1 block at page 2.

Is the buddy of [page2,page3] (i.e., [page0,page1]) free? Yes!
→ Merge to form a free order-2 block at page 0.

Continue up the tree until the buddy is not free.
```

This **coalescing** ensures that the allocator doesn't suffer from external fragmentation as long as allocations and frees are roughly matched.

## Implementation in Moss

Moss's buddy allocator tracks physical page frames. Each physical page has a `PageFrame` struct in a large array indexed by page number:

```rust
pub struct PageFrame {
    order: u8,       // Current allocation order (or the order if free)
    state: u8,       // FREE, ALLOCATED, RESERVED
}
```

The free lists are implemented as intrusive linked lists — the next/prev pointers are stored *within* the free `PageFrame` structs themselves, so no additional allocation is needed for list metadata.

### Allocation Flow

```rust
pub fn alloc_pages(order: usize) -> Option<PA> {
    // Walk up from requested order to find a free block
    for current_order in order..=MAX_ORDER {
        if let Some(block) = free_list[current_order].pop() {
            // Split down to the requested order
            split_block(block, current_order, order);
            return Some(block);
        }
    }
    None  // Out of memory
}
```

### Free Flow

```rust
pub fn free_pages(pa: PA, order: usize) {
    // Try to merge with buddy
    loop {
        let buddy_pa = pa ^ (1 << (PAGE_SHIFT + order));
        if buddy_is_free(buddy_pa, order) {
            // Remove buddy from free list, merge
            free_list[order].remove(buddy_pa);
            pa = min(pa, buddy_pa);  // New block starts at lower address
            order += 1;
        } else {
            break;
        }
    }
    free_list[order].push(pa);
}
```

## Fragmentation

The buddy allocator has excellent resistance to **external fragmentation** (unusable gaps between allocations) because free blocks are always merged.

However, it can suffer from **internal fragmentation**: if you need 5 pages, you must allocate an order-3 block (8 pages), wasting 3 pages. For small allocations (much less than a page), the slab allocator is used instead.

## Performance Characteristics

| Operation | Complexity |
|---|---|
| Allocate | O(log N) — scan up to MAX_ORDER levels |
| Free/Coalesce | O(log N) — merge up to MAX_ORDER levels |
| Space overhead | O(N) — one metadata byte per page |

For a system with 1 GiB of RAM (262,144 pages), MAX_ORDER is typically 11, so allocations and frees require at most 11 steps.

## Exercises

1. Manually walk through allocating 12 pages (three 4-page allocations) from a 32-page buddy allocator, then freeing all three. Draw the free lists at each step.

2. What is the maximum order the buddy allocator needs to support for a system with 4 GiB of RAM and 4 KiB pages?

3. The buddy allocator finds a block's buddy using a simple XOR operation. Explain why `pa ^ (page_size * 2)` always gives the correct buddy address.
