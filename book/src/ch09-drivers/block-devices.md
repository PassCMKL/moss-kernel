# Block Devices and Ramdisk

**Block devices** provide random-access storage organized as fixed-size blocks (typically 512 bytes or 4 KiB). Unlike character devices (which stream bytes), block devices support seeking to any block and reading/writing at arbitrary positions.

## The Block Device Abstraction

```rust
#[async_trait]
pub trait BlockDevice: Send + Sync {
    /// Read `count` bytes from block offset `offset` into `buf`
    async fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize>;

    /// Write `data` to block offset `offset`
    async fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize>;

    /// Total size of the device in bytes
    fn size(&self) -> u64;

    /// Block size (minimum I/O unit)
    fn block_size(&self) -> u32;
}
```

Filesystem drivers are given a `Arc<dyn BlockDevice>` at mount time. They use this to read and write the raw storage.

## The Ramdisk

Moss's ramdisk treats a region of physical memory as a block device. This is used for loading the **initramfs** — a small filesystem image bundled with the kernel that contains early-boot utilities.

```rust
pub struct RamdiskBlkDev {
    base: VA,    // Virtual address of the ramdisk memory
    size: usize, // Size in bytes
}

#[async_trait]
impl BlockDevice for RamdiskBlkDev {
    async fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let src = self.base.0 + offset as usize;
        let len = buf.len().min(self.size - offset as usize);
        buf[..len].copy_from_slice(unsafe {
            core::slice::from_raw_parts(src as *const u8, len)
        });
        Ok(len)
    }

    async fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize> {
        let dst = self.base.0 + offset as usize;
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), dst as *mut u8, data.len())
        };
        Ok(data.len())
    }

    fn size(&self) -> u64 { self.size as u64 }
    fn block_size(&self) -> u32 { 512 }
}
```

Reading from the ramdisk is just a memory copy — no I/O latency.

## The Initramfs

The **initramfs** (initial RAM filesystem) is a small ext4 or FAT32 image embedded in the kernel binary or loaded by the bootloader. It contains:
- A minimal set of utilities (shell, mount, init)
- Device node initializers
- Scripts for setting up the real root filesystem

Moss mounts the ramdisk at `/` during early boot, runs init scripts from it, then (eventually) pivots to the real root filesystem on disk.

## Block Device Layering

Real storage systems often have multiple layers:

```
Application
    │ read("/data/file.txt")
    ▼
VFS (ext4 driver)
    │ reads blocks [1024, 1025, 1026]
    ▼
Block layer (not yet in Moss)
    │ might: merge adjacent reads, cache in page cache
    ▼
Device driver (e.g., NVMe, SATA)
    │ submits I/O request to hardware
    ▼
Hardware (SSD, HDD)
```

Moss currently has a minimal block layer — the filesystem driver calls the block device directly. A full block layer would add:
- **Page cache**: Cache recently-read blocks in RAM to avoid re-reading from disk
- **I/O scheduler**: Merge and reorder requests for better sequential I/O
- **DMA**: Use Direct Memory Access to transfer data without CPU involvement

## Exercises

1. Why do block devices have a minimum I/O unit (the block size), while character devices don't? What problems arise when writing fewer bytes than a block?

2. What is an initramfs and why is it useful? What problem would arise if the kernel tried to directly mount an ext4 root filesystem without any helper utilities?

3. The page cache caches recently read disk blocks in RAM. What consistency issues arise when a file is modified while other processes have it cached?
