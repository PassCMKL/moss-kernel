# Exploring the Source Tree

Before diving into subsystems, let's orient ourselves in the Moss repository. Understanding where things live will make it much easier to follow code references throughout the book.

## Top-Level Layout

```
moss-kernel/
├── src/                  # The kernel itself
├── libkernel/            # Architecture-agnostic kernel library
├── moss-macros/          # Procedural macros (e.g., #[ktest])
├── usertest/             # Userspace test suite
├── book/                 # This book (mdBook source)
├── scripts/              # Build and test helper scripts
├── Cargo.toml            # Workspace manifest
├── Cargo.lock            # Pinned dependency versions
└── justfile              # Build recipes (using `just`)
```

The project is organized as a **Cargo workspace** — multiple Rust crates that are built together. The main crate (`src/`) produces the kernel binary. `libkernel/` contains code that is shared between the kernel and host-side tests (allowing certain algorithms like page table management to be unit-tested on your development machine without running QEMU).

## Inside `src/`

```
src/
├── main.rs               # Kernel entry point (kmain)
├── arch/                 # Architecture-specific code
│   └── arm64/            # AArch64 implementation
│       ├── boot/         # Two-stage boot (start.s, mod.rs)
│       ├── exceptions/   # Exception vector table, syscall dispatch
│       └── memory/       # Page tables, MMU setup, address space
├── memory/               # Architecture-independent memory logic
│   ├── fault.rs          # Page fault handler
│   └── ...
├── process/              # Task and thread group structs
├── sched/                # EEVDF scheduler
├── fs/                   # Virtual filesystem (VFS)
├── drivers/              # Device drivers
│   ├── uart/             # Serial console drivers
│   ├── timer/            # Architectural timer
│   ├── interrupts/       # GICv2/GICv3 interrupt controllers
│   └── fs/               # Filesystem drivers (ext4, tmpfs, etc.)
├── interrupts/           # Interrupt management layer
├── signals/              # POSIX signal delivery
├── console/              # Kernel logging and console output
├── kernel/               # Core kernel utilities (linked lists, etc.)
└── testing/              # In-kernel test harness
```

## Inside `libkernel/`

```
libkernel/
└── src/
    ├── memory/
    │   ├── proc_vm/      # Process virtual memory (VMAs, address space)
    │   ├── allocators/   # Slab allocator
    │   └── ...
    └── ...
```

Code in `libkernel` compiles for both `aarch64-unknown-none-softfloat` (the kernel target) and `x86_64-unknown-linux-gnu` (the test host). This is why memory management algorithms can be unit-tested without booting a VM.

## Key Files to Bookmark

| File | What it contains |
|---|---|
| `src/main.rs` | `kmain()` — first Rust code to run after boot setup |
| `src/arch/arm64/boot/mod.rs` | Two-stage boot initialization |
| `src/arch/arm64/boot/start.s` | AArch64 assembly entry point |
| `src/arch/arm64/exceptions/mod.rs` | Exception/interrupt handlers |
| `src/process/mod.rs` | `Task` and `ThreadGroup` structs |
| `src/sched/mod.rs` | EEVDF scheduler |
| `src/memory/fault.rs` | Page fault handler |
| `src/fs/mod.rs` | VFS interface |
| `src/arch/arm64/memory/address_space.rs` | Page table implementation |
| `src/arch/mod.rs` | `Arch` trait (HAL definition) |

## The `justfile` — Your Build Interface

Moss uses [`just`](https://github.com/casey/just), a command runner similar to `make`. The key recipes are:

```bash
just run              # Build and run in QEMU (interactive shell)
just test-unit        # Run unit tests on the host (fast)
just test-kunit       # Run kernel-space tests in QEMU
just test-userspace   # Run userspace syscall tests in QEMU
just create-image     # Build a rootfs disk image
```

Chapter 11 covers building and testing in detail.

## Cargo Features and Build Targets

Moss uses custom Cargo build targets:

| Target | Used for |
|---|---|
| `aarch64-unknown-none-softfloat` | The kernel binary (bare metal, no FP) |
| `x86_64-unknown-linux-gnu` | Host-side unit tests |
| `aarch64-unknown-linux-musl` | Cross-compiled userspace binaries |

The `softfloat` suffix means floating-point operations are emulated in software rather than using hardware FPU registers. This is common in kernels because saving/restoring FPU state on every context switch is expensive.

## Exercises

1. Clone the Moss repository and run `just test-unit`. How many tests pass?

2. How many lines of Rust code are in the `src/` directory? (Hint: use `find src -name '*.rs' | xargs wc -l`.)

3. Why might it be useful to put memory management code in `libkernel/` rather than directly in `src/`? What constraint does this place on the code in `libkernel/`?
