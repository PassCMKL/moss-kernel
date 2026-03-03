# Inodes and Open Files

The VFS distinguishes between the **file object** (an inode) and a **handle to an open file** (an `OpenFile`). This distinction is fundamental to correct behavior.

## Inodes

An **inode** (index node) represents a file's metadata and data, independent of any particular access to it. One inode exists per file, regardless of how many processes have that file open.

### Inode Contents

- **File type**: regular file, directory, symbolic link, character device, block device, FIFO, socket
- **Permissions**: owner UID/GID, mode bits (rwxrwxrwx)
- **Timestamps**: creation time, modification time, access time
- **Size**: current file size in bytes
- **Data**: the file's actual content (stored differently by each filesystem)

### Inodes are Not File Names

A common misconception: an inode does not contain a file name. File names are stored in **directory entries** (dirents), which map names to inode numbers. This allows **hard links** — multiple directory entries pointing to the same inode:

```bash
# Two names, one inode
ln /etc/hosts /tmp/hosts_copy
# /etc/hosts and /tmp/hosts_copy are both names for the same inode
ls -i /etc/hosts /tmp/hosts_copy  # Same inode number!
```

The inode has a **link count** that tracks how many directory entries point to it. When the link count reaches 0 and no processes have it open, the inode is deleted.

### Inode IDs

Each filesystem assigns unique numeric IDs to its inodes. On ext4, these are 32-bit integers. In Moss, the `InodeId` type combines a filesystem ID and an inode number to ensure global uniqueness:

```rust
pub struct InodeId {
    pub fs_id: FsId,    // Which filesystem
    pub ino:   u64,     // Inode number within that filesystem
}
```

## Open Files

An `OpenFile` represents a specific open instance of an inode — a connection between a process and a file:

```rust
pub struct OpenFile {
    pub inode: Arc<dyn Inode>,
    pub offset: AtomicU64,      // Current read/write position
    pub flags: OpenFlags,       // O_RDONLY, O_WRONLY, O_RDWR, O_APPEND...
}
```

Multiple `OpenFile` instances can refer to the same inode. Each has its own:
- **Current offset**: `read()` and `write()` advance this independently
- **Open flags**: one process might have the file open read-only, another read-write

### O_APPEND

When a file is opened with `O_APPEND`, every `write()` first atomically seeks to the end of the file:

```rust
pub async fn write(&self, data: &[u8]) -> Result<usize> {
    if self.flags.contains(O_APPEND) {
        // Atomically set offset to current file size
        let size = self.inode.getattr().await?.size;
        self.offset.store(size, Ordering::SeqCst);
    }
    let offset = self.offset.fetch_add(data.len() as u64, Ordering::SeqCst);
    self.inode.write_at(offset, data).await
}
```

This is important for log files written by multiple processes — without `O_APPEND`, concurrent writers would overwrite each other.

## File Metadata: `FileAttr`

```rust
pub struct FileAttr {
    pub ino: InodeId,
    pub file_type: FileType,  // Regular, Directory, Symlink, ...
    pub size: u64,
    pub blocks: u64,          // Disk blocks used
    pub atime: Timespec,      // Last access time
    pub mtime: Timespec,      // Last modification time
    pub ctime: Timespec,      // Last status change time
    pub uid: Uid,
    pub gid: Gid,
    pub mode: Mode,           // Permission bits
    pub nlink: u32,           // Number of hard links
}
```

This is what `stat()` and `fstat()` return to user space.

## The `stat` Syscall

```rust
pub async fn sys_fstat(fd: i32, stat_ptr: UA) -> Result<()> {
    let file = current_task().get_file(fd)?;
    let attr = file.inode.getattr().await?;

    // Convert to the kernel stat struct
    let stat = to_kernel_stat(&attr);

    // Copy to user space
    copy_to_user(stat_ptr, &stat).await?;

    Ok(())
}
```

## Directory Entries

Directories are special files whose content is a list of `(name, inode_id)` pairs:

```rust
pub struct DirEntry {
    pub ino: u64,
    pub name: String,
    pub file_type: FileType,
}
```

The `readdir()` method on a directory inode returns these entries. User space accesses them via `getdents64()` (or the `opendir`/`readdir` libc wrappers).

## Exercises

1. Why does the inode not contain the file name? Give a concrete example of a situation where this design matters.

2. What happens to an inode when its last hard link is removed but a process still has the file open? When is the inode actually freed?

3. Implement the `getdents64` syscall for a simple in-memory directory: given a directory inode, read a batch of directory entries and copy them to user space.
