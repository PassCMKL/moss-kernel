# Special Filesystems

Beyond regular disk filesystems, Moss provides several **virtual filesystems** that expose kernel state and devices through the familiar file interface.

## procfs: The Process Information Filesystem

**procfs** is mounted at `/proc` and exposes information about running processes and the kernel itself. Unlike other filesystems, its "files" are generated on-the-fly when read — there is no persistent storage.

### Key Files

| Path | Content |
|---|---|
| `/proc/cmdline` | Kernel command line (from bootloader) |
| `/proc/stat` | CPU usage statistics |
| `/proc/meminfo` | Memory usage (total, free, cached) |
| `/proc/[pid]/` | Per-process directory |
| `/proc/[pid]/maps` | Virtual memory areas (VMAs) |
| `/proc/[pid]/fd/` | Per-process open file descriptors |
| `/proc/[pid]/status` | Process status and identity |
| `/proc/[pid]/comm` | Process name |

### Implementation

procfs implements the `Inode` trait with virtual files that generate their content from kernel data structures:

```rust
pub struct ProcMapsInode {
    pid: Pid,
}

#[async_trait]
impl Inode for ProcMapsInode {
    async fn read_at(&self, _offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Find the process
        let process = find_process(self.pid).ok_or(ESRCH)?;

        // Generate the maps content from the VMA tree
        let mut output = String::new();
        for vma in process.address_space().vmas() {
            writeln!(output,
                "{:012x}-{:012x} {} {:08x} {:02x}:{:02x} {} {}",
                vma.start().0, vma.end().0,
                vma.prot_str(),      // rwxp
                vma.file_offset(),
                0u32, 0u32,          // device major:minor
                vma.inode_id().unwrap_or(0),
                vma.name().unwrap_or("")
            ).unwrap();
        }

        let bytes = output.as_bytes();
        let len = bytes.len().min(buf.len());
        buf[..len].copy_from_slice(&bytes[..len]);
        Ok(len)
    }
}
```

### Security Considerations

procfs exposes sensitive information (memory maps, file descriptors, etc.) that should not be accessible to untrusted users. Moss checks whether the requesting process has permission to read another process's information (same UID, or root).

## devfs: The Device Filesystem

**devfs** is mounted at `/dev` and provides access to device drivers through file-like interfaces.

### Device Types

- **Character devices** (`/dev/console`, `/dev/null`, `/dev/zero`, `/dev/urandom`): Support `read()` and `write()` operations; data flows as a stream of bytes.
- **Block devices** (`/dev/sda`, `/dev/mmcblk0`): Support reading/writing at arbitrary offsets; data is organized in fixed-size blocks.

### Key Devices

| Device | Purpose |
|---|---|
| `/dev/console` | Primary console (UART output) |
| `/dev/null` | Bit bucket — discards all writes, returns EOF on read |
| `/dev/zero` | Returns infinite zero bytes on read |
| `/dev/urandom` | Returns random bytes (from kernel entropy pool) |
| `/dev/tty` | Current controlling terminal |

### `/dev/null` Implementation

```rust
pub struct NullDevice;

#[async_trait]
impl Inode for NullDevice {
    async fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize> {
        Ok(0)  // EOF immediately
    }

    async fn write_at(&self, _offset: u64, data: &[u8]) -> Result<usize> {
        Ok(data.len())  // Silently discard all data
    }
}
```

### `/dev/urandom` and Entropy

`/dev/urandom` is crucial for security: it provides cryptographically secure random bytes to user programs. These are used for session tokens, encryption keys, ASLR offsets, and more.

The kernel collects entropy from hardware sources (timer jitter, hardware RNG if available) and feeds it into a cryptographic PRNG (Pseudo-Random Number Generator).

## sysfs

**sysfs** is a virtual filesystem (mounted at `/sys`) that exposes kernel and driver objects as a hierarchical tree of directories and files. In Linux, it is used by udev to discover hardware and configure device nodes.

Moss includes a sysfs stub but does not yet implement the full hierarchy.

## Exercises

1. Why is `/proc/[pid]/maps` useful for debugging? What information does it provide that a debugger needs?

2. What is the difference between `/dev/random` and `/dev/urandom` on Linux? Why does Moss only implement `/dev/urandom`?

3. Implement a simple `/proc/uptime` file: it should return the number of seconds since boot as a string. What kernel data structure would you need to read?
