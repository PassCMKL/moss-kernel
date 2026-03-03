# Build Commands

All build commands are invoked through `just`. Run `just --list` to see all available recipes.

## Building the Kernel

```bash
# Build the kernel in debug mode (faster compile, slower runtime)
just build

# Build the kernel in release mode (slower compile, faster runtime)
just build-release
```

The kernel binary is placed at `target/aarch64-unknown-none-softfloat/[debug|release]/moss-kernel`.

## Creating a Root Filesystem Image

The root filesystem contains the BusyBox userspace that Moss boots into:

```bash
just create-image
```

This command:
1. Downloads or builds BusyBox for AArch64
2. Builds the usertest binary (`usertest/`)
3. Creates an ext4 disk image with BusyBox and usertest installed
4. The image is placed at a path like `rootfs.img`

The first run takes a while because it downloads BusyBox. Subsequent runs are faster.

## Build Targets

The workspace has multiple crates:

| Crate | Target | Purpose |
|---|---|---|
| `moss-kernel` (root) | `aarch64-unknown-none-softfloat` | The kernel |
| `libkernel` | `aarch64-unknown-none-softfloat` or `x86_64-unknown-linux-gnu` | Shared library |
| `moss-macros` | `x86_64-unknown-linux-gnu` (proc macro) | Macros |
| `usertest` | `aarch64-unknown-linux-musl` | Userspace test binary |

## Common Build Flags

### `--features test`

The kernel can be built with the `test` feature to enable kernel-space tests:

```bash
cargo build --target aarch64-unknown-none-softfloat --features test
```

This compiles in the `#[ktest]` test cases that run during boot.

### `RUST_LOG`

Set the log level for kernel output:

```bash
RUST_LOG=debug just run  # Verbose debug output
RUST_LOG=warn just run   # Only warnings and errors
```

## Build Output

After a successful build:

```
target/
└── aarch64-unknown-none-softfloat/
    └── debug/
        ├── moss-kernel          ← The kernel ELF binary
        └── moss-kernel.d        ← Dependency file (for incremental builds)
```

The kernel ELF can be inspected with standard tools:

```bash
# View kernel sections and sizes
aarch64-linux-gnu-size target/aarch64-unknown-none-softfloat/debug/moss-kernel

# Disassemble the boot code
aarch64-linux-gnu-objdump -d target/aarch64-unknown-none-softfloat/debug/moss-kernel \
    | grep -A 20 "<_start>"

# View symbol table
aarch64-linux-gnu-nm target/aarch64-unknown-none-softfloat/debug/moss-kernel | head -20
```

## Incremental Builds

Rust's incremental compilation means rebuilds after small changes are fast. However, some changes (especially to `libkernel`) can trigger full rebuilds.

If you encounter strange build errors, try a clean build:

```bash
cargo clean
just build
```

## Exercises

1. Run `just build` and examine the kernel ELF. What sections does it contain? How large is each section?

2. What is the difference between debug and release builds in terms of:
   - Compile time?
   - Binary size?
   - Optimization level?
   - Panic behavior?

3. The build target is `aarch64-unknown-none-softfloat`. What does each component of this target triple mean?
