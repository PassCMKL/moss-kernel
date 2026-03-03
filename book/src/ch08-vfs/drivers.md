# Filesystem Drivers

Moss includes several filesystem drivers, each implementing the `FilesystemDriver` trait. They live in `src/drivers/fs/`.

## ext4

**ext4** is the default filesystem for Linux. It is a journaling, block-structured filesystem with excellent performance and reliability.

### Key Features

- **Journaling**: All metadata changes are recorded in a journal before being applied. If the system crashes, the journal allows the filesystem to recover to a consistent state.
- **Extents**: Large files are stored using extents (contiguous ranges of blocks) rather than individual block pointers, reducing metadata overhead.
- **64-bit block numbers**: Supports very large filesystems.
- **`dir_index`**: B-tree indexed directories for fast lookup in large directories.

### Moss Implementation

Moss's ext4 driver currently supports:
- Reading and writing regular files
- Reading directories
- Creating, removing, and renaming files

The driver reads the superblock to discover block size, inode size, and group descriptor locations, then implements the ext2/ext3/ext4 disk layout format.

### Disk Layout

```
┌──────────────┬──────────────────┬─────────────────────────┐
│ Boot sector  │   Superblock     │   Block Groups...        │
│  (1024 B)    │   (1024 B)       │                          │
└──────────────┴──────────────────┴─────────────────────────┘

Each Block Group:
┌──────────────┬──────────────┬──────────────┬─────────────┐
│ Block bitmap │ Inode bitmap │ Inode table  │ Data blocks │
└──────────────┴──────────────┴──────────────┴─────────────┘
```

## FAT32

**FAT32** (File Allocation Table, 32-bit) is a simple, widely-supported filesystem used by SD cards, USB drives, and legacy systems.

### How FAT Works

FAT stores files as chains of clusters. The File Allocation Table is an array indexed by cluster number; each entry points to the next cluster in a file (or marks end-of-file):

```
FAT: [0:reserved][1:reserved][2:0x0FFFFFF8][3:4][4:5][5:0xFFFFFFFF]
                                 ↑ cluster 2 is end-of-chain
                                                 ↑ cluster 5 is last in chain

File "hello.txt" starts at cluster 3 → 4 → 5 → EOF
```

### Moss FAT32 Support

Moss's FAT32 driver is read-only (sufficient for reading USB drives or SD cards with preloaded data). It supports:
- Reading files and directories
- Long filename (VFAT) support
- Traversing the cluster chain

## tmpfs

**tmpfs** is a RAM-backed filesystem. All files exist only in memory — they are lost on reboot. tmpfs is used for:
- `/tmp`: Temporary files
- `/run`: Runtime data (PID files, sockets)
- Application-created memory-backed files (`memfd_create`)

### Implementation

tmpfs stores file data in memory using an in-memory tree:

```rust
pub struct TmpfsInode {
    id: InodeId,
    attr: RwLock<FileAttr>,
    kind: TmpfsKind,
}

enum TmpfsKind {
    RegularFile {
        data: RwLock<Vec<u8>>,
    },
    Directory {
        children: RwLock<BTreeMap<String, Arc<TmpfsInode>>>,
    },
    Symlink {
        target: String,
    },
}
```

Reading and writing are simple memory operations — no I/O needed. This makes tmpfs extremely fast but volatile.

### Size Limits

tmpfs uses physical memory, so it must be bounded. A large `/tmp` write can fill RAM. Moss will eventually support per-tmpfs size limits.

## Exercises

1. What is the advantage of a journaling filesystem over a non-journaling one? What workloads benefit most?

2. Why does FAT32 have poor performance for directories with many files? How does ext4's `dir_index` feature improve this?

3. When a process writes to a tmpfs file, where is the data stored? Does it appear in the page cache? Can it be swapped out?
