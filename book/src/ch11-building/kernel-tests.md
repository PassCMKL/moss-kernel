# Kernel Tests

Kernel tests (ktests) run **inside the kernel** in QEMU. They test subsystems that require a running kernel — the scheduler, real page tables, IPC, signals, and more.

## Running Kernel Tests

```bash
just test-kunit
```

This builds the kernel with the `test` feature enabled, boots it in QEMU, runs all `#[ktest]` functions, reports results, and shuts down.

Expected output:

```
[MOSS] Running kernel tests...
[KTEST] test_page_mapping ... ok
[KTEST] test_frame_alloc ... ok
[KTEST] test_slab_alloc ... ok
[KTEST] test_fork_basic ... ok
[KTEST] test_signal_delivery ... ok
[KTEST] test_pipe_read_write ... ok
...
[KTEST] All 23 tests passed!
[MOSS] Powering off.
```

## The `#[ktest]` Macro

Kernel tests are marked with `#[ktest]` from the `moss-macros` crate:

```rust
// src/testing/mod.rs usage example
#[ktest]
async fn test_page_mapping() {
    // Test that mapping a page and reading it back works
    let frame = frame_alloc().expect("no memory");
    let va = map_kernel_page(frame);

    // Write a pattern to the physical page
    let ptr = va.as_mut_ptr::<u64>();
    unsafe { *ptr = 0xDEAD_BEEF_CAFE_1234 };

    // Read it back
    assert_eq!(unsafe { *ptr }, 0xDEAD_BEEF_CAFE_1234);

    // Cleanup
    unmap_kernel_page(va);
    frame_free(frame);
}
```

The macro expands to:
1. A regular function named `test_page_mapping`
2. A static registration that adds it to a test list collected at link time
3. An async wrapper that provides the test execution context

## Async Kernel Tests

Kernel tests are `async fn`, which means they can:
- Test operations that require sleeping (disk I/O, timer wait)
- Test signal delivery (an async test can be interrupted by a signal)
- Use the same async primitives as the kernel itself

```rust
#[ktest]
async fn test_nanosleep() {
    let before = clock_gettime(CLOCK_MONOTONIC);
    nanosleep(Duration::from_millis(10)).await;
    let after = clock_gettime(CLOCK_MONOTONIC);

    let elapsed = after - before;
    assert!(elapsed >= Duration::from_millis(10), "sleep was too short");
    assert!(elapsed < Duration::from_millis(50), "sleep was too long");
}
```

## Test Execution Context

During kernel boot, after all subsystems are initialized but before spawning `init`, the kernel runs:

```rust
#[cfg(feature = "test")]
fn run_ktests() {
    for test_fn in KTEST_TABLE {
        print!("[KTEST] {} ... ", test_fn.name);
        let result = run_async(test_fn.func());
        match result {
            Ok(()) => println!("ok"),
            Err(e) => println!("FAILED: {:?}", e),
        }
    }
}
```

If any test panics, the kernel catches the panic (rather than halting), records the failure, and continues with the remaining tests.

## What Kernel Tests Cover

- Memory management: page allocation/deallocation, CoW faults, VMA operations
- Process management: fork, exec, exit, wait
- Signals: delivery, masking, handlers
- Filesystem: creating/reading/writing/deleting files in tmpfs
- Synchronization: spinlocks, mutexes, condition variables
- Timer: nanosleep accuracy, timer expiry

## Exercises

1. Run `just test-kunit`. How many kernel tests exist? How long does the test run take?

2. Write a kernel test that verifies copy-on-write works: fork a process, have the child write to a shared page, and verify the parent's copy is unchanged.

3. What would happen if a kernel test called `malloc` (from user space) instead of `frame_alloc` (kernel allocation)? Would this work?
