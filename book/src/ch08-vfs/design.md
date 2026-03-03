# VFS Design

The Virtual Filesystem is a layer of indirection that decouples the filesystem interface (what user programs see) from filesystem implementations (how data is stored).

## The Core Problem

Without a VFS, every program that wanted to access a file would need to know whether it's on an ext4 partition, a FAT32 USB drive, an NFS server, or a tmpfs. Clearly, this is impractical.

The VFS solves this by defining a set of operations every filesystem must implement, then routing all I/O requests through this common interface.

## The Unix Namespace

In Unix, all filesystems are mounted into a single **namespace** — a tree of directories starting at `/`. The namespace hides the boundary between different filesystems:

```
/                       ← root filesystem (e.g., ext4 on /dev/sda1)
├── bin/
├── etc/
├── home/
├── proc/               ← procfs mounted here
│   ├── 1/
│   └── cmdline
├── dev/                ← devfs mounted here
│   ├── console
│   └── null
├── tmp/                ← tmpfs mounted here
└── mnt/
    └── usb/            ← FAT32 on /dev/sdb1 mounted here
```

From a user program's perspective, all these paths look identical. The VFS handles routing each path to the correct filesystem.

## VFS Layers in Moss

```
User space calls (open, read, write, stat, ...)
              │
              ▼
         VFS Interface (src/fs/mod.rs)
         ┌─────────────────────────────────┐
         │ Mount table                     │
         │ Path resolution                 │
         │ Inode cache (future)            │
         └───────────────────────────────┬─┘
                  │                      │
         ┌────────▼──────┐      ┌────────▼──────┐
         │ Filesystem A  │      │ Filesystem B  │
         │ (ext4)        │      │ (tmpfs)       │
         └───────────────┘      └───────────────┘
```

## The `Inode` Trait

The central abstraction is the `Inode` trait, which any filesystem file must implement:

```rust
#[async_trait]
pub trait Inode: Send + Sync {
    /// Unique identifier for this inode
    fn id(&self) -> InodeId;

    /// File metadata (type, permissions, size, timestamps)
    async fn getattr(&self) -> Result<FileAttr>;

    /// Read data from this file
    async fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize>;

    /// Write data to this file
    async fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize>;

    /// List directory entries (for directories)
    async fn readdir(&self, offset: u64) -> Result<Vec<DirEntry>>;

    /// Look up a child by name (for directories)
    async fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>>;

    /// Create a new file (for directories)
    async fn create(&self, name: &str, mode: Mode) -> Result<Arc<dyn Inode>>;

    /// Create a new directory (for directories)
    async fn mkdir(&self, name: &str, mode: Mode) -> Result<Arc<dyn Inode>>;

    /// Remove a file (for directories)
    async fn unlink(&self, name: &str) -> Result<()>;

    /// Create a symbolic link (for directories)
    async fn symlink(&self, name: &str, target: &str) -> Result<Arc<dyn Inode>>;

    /// Read symlink target (for symlinks)
    async fn readlink(&self) -> Result<String>;

    /// Change permissions
    async fn chmod(&self, mode: Mode) -> Result<()>;
}
```

Every filesystem provides concrete implementations of these methods. The VFS code only calls through the `Arc<dyn Inode>` interface, without knowing which filesystem is underneath.

## The `FilesystemDriver` Trait

To mount a filesystem, it must be instantiated from a block device (or nothing, for virtual filesystems):

```rust
pub trait FilesystemDriver: Send + Sync {
    fn name(&self) -> &str;

    /// Mount this filesystem from a block device (or None for virtual FSes)
    async fn mount(&self, dev: Option<Arc<dyn BlockDevice>>) -> Result<Arc<dyn Inode>>;
    // Returns the root inode of the mounted filesystem
}
```

Moss registers drivers for: `ext4`, `tmpfs`, `devfs`, `procfs`, `fat32`.

## The Mount Table

The VFS maintains a **mount table** — a list of (path, filesystem_root) pairs. When resolving a path, the VFS checks at each component whether a filesystem is mounted there:

```rust
struct MountTable {
    entries: BTreeMap<PathBuf, Arc<dyn Inode>>,  // path → filesystem root inode
}

impl MountTable {
    fn resolve_mount(&self, path: &Path) -> Option<(Arc<dyn Inode>, &Path)> {
        // Find the longest prefix that is a mount point
        for (mount_point, root) in self.entries.iter().rev() {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return Some((root.clone(), relative));
            }
        }
        None
    }
}
```

## Exercises

1. What would happen if two filesystems were mounted at the same path? Should the second mount "shadow" the first, or should it be an error?

2. The `Inode` trait uses `async fn`. Why is this important for a filesystem that might need to read from disk?

3. What is the benefit of the VFS being an in-kernel layer rather than a user-space library? (Consider what happens when two unrelated processes access the same file.)
