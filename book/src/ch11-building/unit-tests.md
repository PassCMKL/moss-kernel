# Unit Tests

Unit tests in Moss run on the **host machine** (x86_64 Linux or macOS) without requiring QEMU. They test architecture-independent code in `libkernel`.

## Running Unit Tests

```bash
just test-unit
# or equivalently:
cargo test -p libkernel
```

Output:
```
running 47 tests
test memory::buddy::tests::alloc_single_page ... ok
test memory::buddy::tests::alloc_and_free ... ok
test memory::buddy::tests::coalesce_buddies ... ok
test memory::proc_vm::tests::vma_insert_no_overlap ... ok
test memory::proc_vm::tests::vma_find ... ok
...
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured
```

## What Is Tested

The unit tests cover:

- **Buddy allocator**: Allocation at various orders, freeing and coalescing, out-of-memory handling
- **Slab allocator**: Object allocation and freeing across multiple slab pages
- **Page table construction**: Building and walking multi-level AArch64 page tables (the algorithm is tested on x86_64 even though the hardware format is AArch64)
- **VMA tree**: Virtual memory area insertion, overlap detection, splitting and merging
- **Address types**: VA/PA/UA conversions and arithmetic

## Why Host Tests?

Running tests on the host machine (without QEMU) is much faster than kernel tests. A unit test run takes 2–5 seconds. A kernel test run requires:
1. Booting QEMU (~2 seconds)
2. Kernel initialization (~1 second)
3. Running tests
4. Shutting down QEMU

Unit tests are ideal for rapid iteration during development.

## Writing a Unit Test

Tests in `libkernel` use the standard Rust `#[test]` attribute:

```rust
// In libkernel/src/memory/buddy.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_order_0() {
        let mut allocator = BuddyAllocator::new_for_test(1024 * 1024);

        let page = allocator.alloc(0).expect("allocation failed");
        assert!(page.0 % PAGE_SIZE == 0, "page must be page-aligned");

        allocator.free(page, 0);
        // After freeing, should be able to allocate again
        let page2 = allocator.alloc(0).expect("second allocation failed");
        assert_eq!(page, page2);  // Same page returned (buddy allocator is deterministic)
    }

    #[test]
    fn test_coalesce() {
        let mut allocator = BuddyAllocator::new_for_test(4 * PAGE_SIZE);

        let p0 = allocator.alloc(0).unwrap();
        let p1 = allocator.alloc(0).unwrap();

        // Both should be freed and coalesced into an order-1 block
        allocator.free(p0, 0);
        allocator.free(p1, 0);

        // Now should be able to allocate an order-1 block
        let large = allocator.alloc(1).expect("order-1 alloc failed");
        assert!(large.0 % (2 * PAGE_SIZE) == 0);
    }
}
```

## Test Infrastructure

The `BuddyAllocator::new_for_test` constructor creates an allocator backed by a `Vec<u8>` on the heap, rather than physical pages. This allows the allocator's algorithms to be tested without any OS support.

Similarly, the page table tests use a mock "physical memory" that maps page numbers to `Vec<u8>` buffers, allowing the multi-level table code to be tested without actual hardware.

## Exercises

1. Run `just test-unit`. How many tests pass? Are there any failures?

2. Add a test for the buddy allocator that allocates 4 order-0 pages, frees them in reverse order, and verifies that all four are coalesced into an order-2 block.

3. The page table unit tests run the AArch64 page table code on an x86_64 host. What assumptions does this require? What would break if the host were big-endian?
