# Stage 2: Kernel Proper

After Stage 1 establishes a stable address space and a working stack, `arch_init_stage2()` completes the full kernel initialization. By the end of Stage 2, the kernel is ready to run user processes.

## Stage 2 Prerequisites

Entering Stage 2, the kernel has:
- TTBR1 active with the permanent kernel address space
- The identity map in TTBR0 still active (will be removed early in Stage 2)
- A working early allocator (Smalloc)
- No exception handlers, no heap, no scheduler, no console

## Step-by-Step Walkthrough

### 1. Disable the Identity Map

The first thing Stage 2 does is remove the temporary TTBR0 identity map. From this point on, any access to a low virtual address will fault. This is intentional — it catches bugs where code accidentally uses physical addresses as virtual addresses.

### 2. Initialize the Frame Allocator

The frame allocator manages physical memory pages (frames). It is the foundation of all memory allocation in the kernel.

Moss uses a **buddy allocator** for physical memory. The buddy system divides memory into power-of-two-sized blocks. When a block is freed, it is merged with its "buddy" (the adjacent block of the same size) if the buddy is also free.

```
Physical memory: [page0][page1][page2][page3][page4]...

Free blocks at order 0 (4KB):  {page0, page2, page4, ...}
Free blocks at order 1 (8KB):  {page0+page1, page4+page5, ...}
Free blocks at order 2 (16KB): {page0..page3, ...}
```

The frame allocator is initialized with the memory regions discovered from the DTB in Stage 1, minus the regions already used by:
- The kernel image itself
- The DTB
- The early Smalloc allocations

### 3. Initialize the Slab Allocator

The frame allocator is great for allocating whole pages (4KB minimum), but the kernel frequently needs much smaller allocations — a 64-byte task struct, a 128-byte file descriptor, and so on. The **slab allocator** provides this.

The slab allocator maintains per-size caches. Each cache holds a set of **slabs** (one or more pages). Within each slab, objects of a fixed size are pre-allocated. This eliminates internal fragmentation for common kernel object sizes.

Once the slab allocator is live, Rust's global allocator (`alloc::`) is backed by it — `Box<T>`, `Vec<T>`, `Arc<T>`, and similar types all work.

### 4. Set Up Per-CPU Data

Moss supports multiple CPUs (SMP). Each CPU needs its own:
- Kernel stack (set up in Stage 1)
- Scheduler run queue and wait queue
- Slab allocator cache (for locality)
- Exception state save area

Per-CPU data is stored at a fixed virtual address that each CPU maps to its own private physical page. Reading `per_cpu().scheduler` on CPU 0 returns CPU 0's scheduler; the same address on CPU 1 returns CPU 1's scheduler.

### 5. Install the Exception Vector Table

The exception vector table is a page-aligned block of code where the CPU jumps on any exception (interrupt, system call, page fault, etc.). It must be installed before interrupts are enabled.

```rust
// Install the vector table address into the VBAR_EL1 register
write_sysreg!(vbar_el1, &EXCEPTION_VECTORS as *const _ as u64);
```

Each entry in the vector table is 128 bytes of instructions. Moss's vector table is in `src/arch/arm64/exceptions/exceptions.s`. Once installed, the kernel can safely receive hardware interrupts.

### 6. Probe Devices from the DTB

Stage 2 walks the Device Tree looking for hardware it recognizes, then initializes drivers for each device found:

- **UART**: Sets up the serial console so `kprint!()` works
- **Interrupt Controller (GIC)**: Enables hardware interrupt routing
- **System Timer**: Configures the per-CPU timer that drives the scheduler

Device probing happens in `src/drivers/` and uses the device tree compatible strings (e.g., `"arm,pl011"`) to match devices to drivers.

### 7. Enable Interrupts

Now that the exception vector table and interrupt controller are initialized, Stage 2 enables CPU interrupts. The timer will begin firing, and the scheduler can start preempting tasks.

### 8. Initialize the VDSO

The **vDSO** (Virtual Dynamically Shared Object) is a small shared library that the kernel maps into every process's address space. It provides fast paths for certain system calls (like `clock_gettime`) that can be served without a full kernel entry by reading memory-mapped kernel state.

Moss initializes the VDSO page during Stage 2 and maps it at a fixed address in every process's address space.

### 9. Call `kmain()`

With all subsystems initialized, Stage 2 calls `kmain()` in `src/main.rs`. This function:

1. Parses the kernel command line (passed by the bootloader via the DTB)
2. Mounts the initial filesystem (rootfs)
3. Launches the idle task for this CPU
4. Spawns the `init` process (PID 1)
5. Calls `sched_start()` to begin scheduling

From this point on, the kernel is event-driven: it services interrupts, handles system calls from user processes, and manages resources on their behalf.

## The `kmain()` Function

```rust
// src/main.rs (simplified)
pub async fn kmain(cmdline: &str) {
    // Parse and apply kernel command line options
    let opts = parse_cmdline(cmdline);

    // Mount the root filesystem
    mount_rootfs(&opts).await;

    // Launch init (PID 1)
    spawn_init().await;

    // The idle loop — runs when no other tasks are ready
    loop {
        arch::wait_for_interrupt();
    }
}
```

## Exercises

1. What happens if the frame allocator is initialized before the exception vector table is installed? Give a specific scenario where this ordering would cause a crash.

2. Why does the slab allocator exist on top of the frame allocator rather than directly allocating pages for every kernel object?

3. What is the purpose of the vDSO? Look up how Linux implements `clock_gettime` in the vDSO and compare it to how a naive implementation would work.
