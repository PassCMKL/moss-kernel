# Prerequisites and Toolchain

## Required Tools

### Rust Toolchain

Moss uses a pinned Rust nightly toolchain (specified in `rust-toolchain.toml`). Install Rust with rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The `rust-toolchain.toml` file in the repository root automatically selects the correct nightly version when you run any `cargo` command inside the repository.

You also need the AArch64 target for bare-metal compilation:

```bash
rustup target add aarch64-unknown-none-softfloat
```

And for cross-compiling userspace:

```bash
rustup target add aarch64-unknown-linux-musl
```

### just

Moss uses `just` as its build command runner:

```bash
# On Ubuntu/Debian
sudo apt install just

# With cargo
cargo install just

# On macOS with Homebrew
brew install just
```

### QEMU

QEMU is the emulator used to run Moss during development:

```bash
# Ubuntu/Debian
sudo apt install qemu-system-aarch64

# Fedora
sudo dnf install qemu-system-aarch64

# macOS with Homebrew
brew install qemu
```

Verify the installation:

```bash
qemu-system-aarch64 --version
# Should show QEMU version 7.0 or later
```

### Cross-Compilation Tools (Optional)

For compiling userspace binaries, you may need AArch64 binutils:

```bash
# Ubuntu/Debian
sudo apt install binutils-aarch64-linux-gnu gcc-aarch64-linux-gnu
```

### Image Creation Tools

To create the disk image:

```bash
# Ubuntu/Debian
sudo apt install e2fsprogs   # mkfs.ext4
sudo apt install dosfstools  # mkfs.fat (for FAT images)
```

## Verifying the Setup

After installing all prerequisites, verify everything works:

```bash
cd moss-kernel

# Check the toolchain
cargo --version
rustup show active-toolchain

# Check QEMU
qemu-system-aarch64 --version

# Check just
just --version

# Run unit tests (no QEMU needed)
just test-unit
```

If unit tests pass, your environment is set up correctly.

## Platform Notes

### Linux (Recommended)

The development environment is best supported on Linux. All tools are easily installed via system package managers.

### macOS

Development on macOS works but requires Homebrew for QEMU and some tools. Disk image creation may require different commands.

### Windows (WSL2)

Development on Windows requires WSL2 (Windows Subsystem for Linux). Install Ubuntu in WSL2 and follow the Linux instructions. QEMU can run inside WSL2 or natively on Windows, but bridging the two can be complex.

## Disk Space Requirements

- Rust toolchain: ~2 GiB
- QEMU: ~100 MiB
- Repository + build artifacts: ~500 MiB
- Root filesystem image: ~500 MiB

Total: approximately 3–4 GiB of free space required.
