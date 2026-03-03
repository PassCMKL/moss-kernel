# File Descriptor Tables

When a process opens a file, the kernel returns a small non-negative integer called a **file descriptor** (FD). The process uses this integer in subsequent system calls (`read`, `write`, `close`, etc.). Internally, the kernel maintains a per-process table mapping FD numbers to kernel file objects.

## The Three-Level Structure

Unix file management has three levels:

```
Process A         Process B
┌──────────┐      ┌──────────┐
│ FD Table │      │ FD Table │
│  0 → ●───┼──┐   │  0 → ●───┼──┐
│  1 → ●───┼──┤   │  3 → ●───┼──┤
└──────────┘  │   └──────────┘  │
              ▼                  ▼
         ┌──────────┐       ┌──────────┐
         │ OpenFile │       │ OpenFile │
         │ offset=0 │       │ offset=1024│
         │ flags=RW │       │ flags=RO  │
         └────┬─────┘       └────┬──────┘
              │                  │
              ▼                  ▼
         ┌──────────────────────────────┐
         │          Inode               │
         │  /home/user/data.txt         │
         │  (on disk, or in tmpfs, etc.)│
         └──────────────────────────────┘
```

1. **File Descriptor Table**: Per-process, maps FD integers to `OpenFile` references
2. **OpenFile**: Per-open-call, tracks the current offset and open flags. Multiple processes can have different offsets into the same inode.
3. **Inode**: The actual file object, shared across all opens of the same file

## The `FileDescriptorTable` Struct

```rust
pub struct FileDescriptorTable {
    files: SpinLock<BTreeMap<Fd, Arc<OpenFile>>>,
    next_fd: AtomicI32,
}

impl FileDescriptorTable {
    pub fn insert(&self, file: Arc<OpenFile>) -> Fd {
        let fd = self.allocate_fd();
        self.files.lock().insert(fd, file);
        fd
    }

    pub fn get(&self, fd: Fd) -> Option<Arc<OpenFile>> {
        self.files.lock().get(&fd).cloned()
    }

    pub fn close(&self, fd: Fd) -> Result<()> {
        self.files.lock().remove(&fd)
            .map(|_| ())
            .ok_or(EBADF)
    }
}
```

FDs are allocated starting from the smallest available non-negative integer. This is why standard streams are always FD 0, 1, 2 — they're allocated first when the process starts.

## The `OpenFile` Struct

```rust
pub struct OpenFile {
    inode: Arc<dyn Inode>,    // The underlying file object
    offset: AtomicU64,        // Current read/write position
    flags: OpenFlags,         // O_RDONLY, O_WRONLY, O_RDWR, O_APPEND...
}
```

Having a separate `OpenFile` per open call is important because multiple opens of the same file should have independent offsets (unless they're the same open call duplicated via `dup()`).

## Standard File Descriptors

By Unix convention, every process inherits three open FDs:

| FD | Name | Purpose |
|---|---|---|
| 0 | stdin | Standard input |
| 1 | stdout | Standard output |
| 2 | stderr | Standard error |

When the first process (`init`) is created, Moss opens `/dev/console` three times and assigns the resulting FDs to 0, 1, and 2. All subsequent processes inherit these via `fork`.

## Fork and FD Inheritance

When a process forks, the child gets a copy of the parent's FD table:

```rust
fn fork_fd_table(parent: &FileDescriptorTable) -> FileDescriptorTable {
    let new_table = FileDescriptorTable::new();
    for (fd, file) in parent.files.lock().iter() {
        if !file.flags.contains(O_CLOEXEC) {
            // Share the same OpenFile — same offset, same underlying inode
            new_table.insert_at(*fd, Arc::clone(file));
        }
        // FDs with O_CLOEXEC are NOT inherited
    }
    new_table
}
```

Parent and child share the same `Arc<OpenFile>` objects, so they share the same file offset. If the parent reads 100 bytes, the shared offset advances, and the child reading from FD 0 will continue from byte 101.

This sharing is intentional for pipelines: `ls | grep foo` works because `ls`'s stdout and `grep`'s stdin point to the same pipe `OpenFile`.

## `dup` and `dup2`: Duplicating FDs

`dup(fd)` creates a new FD in the current process that refers to the same `OpenFile`. Both FDs share the offset:

```c
int old_fd = open("file.txt", O_RDWR);
int new_fd = dup(old_fd);
// old_fd and new_fd both point to the same OpenFile
// Reading from either advances the same offset
```

`dup2(old_fd, new_fd)` additionally closes `new_fd` if it's open, then makes `new_fd` refer to the same `OpenFile` as `old_fd`. This is how shells redirect I/O:

```c
// Redirect stdout (FD 1) to file
int file_fd = open("output.txt", O_WRONLY|O_CREAT, 0644);
dup2(file_fd, 1);     // Make FD 1 point to the file
close(file_fd);        // No longer need the original FD
exec("/bin/ls", ...); // ls writes to FD 1 = the file
```

## Exercises

1. Why is having a separate `OpenFile` per `open()` call important? What would break if two opens of the same file always shared an offset?

2. What is `O_CLOEXEC`? Why is it important in secure programming? What problem would occur without it in a server that forks to handle connections?

3. Implement a simple pipe: `pipe(fds)` creates two FDs — `fds[0]` for reading and `fds[1]` for writing. Describe the data structure you would use in the kernel.
