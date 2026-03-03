# Running in QEMU

QEMU is a full-system emulator that runs Moss on your development machine as if it were running on real AArch64 hardware.

## Starting an Interactive Session

```bash
just run
```

This launches QEMU with the Moss kernel and the root filesystem image. You should see the kernel boot messages followed by a shell prompt.

The `just run` recipe is roughly equivalent to:

```bash
qemu-system-aarch64 \
    -machine virt \                      # The "virt" ARM virtual machine
    -cpu cortex-a57 \                    # Emulate a Cortex-A57 CPU
    -smp 4 \                             # 4 CPUs
    -m 512M \                            # 512 MiB RAM
    -kernel target/.../moss-kernel \     # The kernel binary
    -drive file=rootfs.img,format=raw \  # The root filesystem
    -nographic \                         # No graphical window
    -serial stdio \                      # UART connected to terminal
    -append "root=/dev/vda console=ttyAMA0"  # Kernel command line
```

## The QEMU `virt` Machine

The `virt` machine type is a QEMU-specific virtual ARM machine. It includes:
- A GICv2 or GICv3 interrupt controller (depending on QEMU version)
- An ARM PL011 UART at `0x9000000`
- VirtIO block device for the disk
- VirtIO network device (configurable)
- An architectural timer
- Up to 512 CPUs

The hardware configuration is communicated to the kernel via a Device Tree Blob that QEMU generates automatically.

## Expected Boot Output

```
[MOSS] Initializing stage 1...
[MOSS] Physical memory: 512 MiB at 0x40000000
[MOSS] Setting up kernel address space...
[MOSS] Smalloc initialized: 64 KiB
[MOSS] Stage 1 complete, switching to kernel stack

[MOSS] Initializing stage 2...
[MOSS] Frame allocator initialized: 131072 pages
[MOSS] Slab allocator initialized
[MOSS] Exception vectors installed
[MOSS] Probing devices from FDT...
[MOSS]   Found PL011 UART at 0x9000000
[MOSS]   Found GICv2 at 0x8000000
[MOSS]   Found ARM timer
[MOSS] Secondary CPUs booted: 4 total
[MOSS] VDSO initialized
[MOSS] Mounting root filesystem...
[MOSS] Spawning init (PID 1)...

Welcome to Moss!
/ #
```

## Interacting with the Shell

Once booted, you have a BusyBox shell. Try:

```bash
ls /
cat /proc/cmdline
cat /proc/meminfo
ps aux
/usertest   # Run the userspace test suite
```

## Debugging with GDB

QEMU supports connecting GDB for remote debugging:

```bash
# Start QEMU paused, waiting for GDB
just run-gdb

# In a separate terminal:
aarch64-linux-gnu-gdb target/.../moss-kernel
(gdb) target remote :1234
(gdb) continue
(gdb) break kmain
(gdb) continue
```

You can set breakpoints in kernel code, inspect registers, and trace execution.

## Adjusting QEMU Parameters

Edit the `justfile` to change QEMU configuration:

```makefile
# Increase RAM
QEMU_FLAGS := -m 1G

# Single CPU (easier debugging)
QEMU_FLAGS := -smp 1

# Enable KVM for faster emulation (Linux host only)
QEMU_FLAGS := -enable-kvm
```

## Exiting QEMU

- Type `exit` or `Ctrl+D` in the shell to exit normally
- Press `Ctrl+A` then `X` to force-quit QEMU
- The test suites automatically shut down QEMU on completion

## Common Issues

| Problem | Solution |
|---|---|
| QEMU not found | Install `qemu-system-aarch64` |
| Kernel panics on boot | Try debug build: `just build && just run` |
| No output on serial | Check `-serial stdio` flag in justfile |
| Disk image not found | Run `just create-image` first |
| Out of memory | Increase `-m` in QEMU flags |

## Exercises

1. Run `just run` and examine the kernel boot messages. How long does boot take from the first log line to the shell prompt?

2. Once booted, run `cat /proc/1/maps`. What memory regions does the init process have? Can you identify the stack, heap, and program text?

3. Run `just run-gdb` and set a breakpoint at `sys_write`. Trigger it by running a command in the shell. What arguments does it receive?
