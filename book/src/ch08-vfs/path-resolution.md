# Path Resolution

When a user program calls `open("/etc/hosts", O_RDONLY)`, the kernel must translate the string `"/etc/hosts"` into an inode. This process is called **path resolution**.

## The Resolution Algorithm

Path resolution walks the filesystem tree one component at a time:

```
Input: "/etc/hosts"

1. Start at root inode (or cwd for relative paths)
2. Component "etc": call root.lookup("etc") → inode for /etc directory
3. Component "hosts": call etc_inode.lookup("hosts") → inode for /etc/hosts
4. Return the resulting inode
```

In Rust:

```rust
pub async fn resolve_path(path: &str) -> Result<Arc<dyn Inode>> {
    let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Start at root (absolute path) or current directory (relative path)
    let mut current = if path.starts_with('/') {
        root_inode()
    } else {
        current_task().thread_group.cwd.read().clone()
    };

    for component in &components {
        match *component {
            "."  => { /* Stay at current directory */ }
            ".." => {
                // Go to parent (filesystem must track this)
                current = current.parent_or_root().await?;
            }
            name => {
                current = current.lookup(name).await
                    .map_err(|_| ENOENT)?;

                // Follow symlinks (up to MAX_SYMLINK_DEPTH)
                if current.getattr().await?.file_type == FileType::Symlink {
                    let target = current.readlink().await?;
                    current = resolve_path(&target).await?;
                }

                // Check if this is a mount point
                if let Some(mounted_fs_root) = mount_table().get_mount(&current) {
                    current = mounted_fs_root;
                }
            }
        }
    }

    Ok(current)
}
```

## Symbolic Links

**Symbolic links** are files whose content is a path to another file. When the path resolver encounters a symlink, it follows it:

```
/etc/alternatives/python → /usr/bin/python3
```

If `/etc/alternatives/python` is a symlink to `/usr/bin/python3`, then resolving `/etc/alternatives/python` returns the inode at `/usr/bin/python3`.

### Symlink Loops

A symlink can point to another symlink, which can point back to the first:

```
/tmp/a → /tmp/b
/tmp/b → /tmp/a
```

Without a loop limit, path resolution would recurse infinitely. Moss limits symlink depth to **40 levels** (matching Linux), returning `ELOOP` if exceeded.

### Following vs. Not Following

Some syscalls follow symlinks (most of the time) and some do not:
- `stat()` follows symlinks (returns info about the target)
- `lstat()` does NOT follow symlinks (returns info about the symlink itself)
- `unlink()` does NOT follow symlinks (removes the symlink, not the target)

## Mount Points

When the path resolver reaches a directory that is a mount point, it transparently crosses into the mounted filesystem:

```
Resolving "/proc/1/maps":
  1. "/" → root inode
  2. "proc" → /proc directory inode (still on root fs)
  3. /proc is a mount point → switch to procfs root inode
  4. "1" → procfs lookup for process 1
  5. "maps" → memory map file for process 1
```

The mount point crossing is invisible to the user.

## Permissions Check

At each step, the kernel checks permissions:
- Can the current process execute (traverse) this directory?
- Does the final file have the requested permissions (read/write/execute)?

```rust
// Check execute (traverse) permission on each directory component
check_permission(&dir_inode, Access::Execute, &creds)?;
```

The root user (EUID=0) bypasses all permission checks.

## Path Resolution Modes

Different syscalls have slightly different resolution behavior:

- **`AT_FDCWD`**: Resolve relative to the current working directory
- **Dirfd**: Resolve relative to an open directory FD (used by `openat`, `mkdirat`, etc.)
- **`O_NOFOLLOW`**: Return `ELOOP` if the final component is a symlink
- **`O_PATH`**: Open a file descriptor for the path itself without opening the file

The `*at` family of syscalls (openat, mkdirat, etc.) are important for security: they allow atomically operating on a file relative to a known-open directory, preventing TOCTOU attacks involving symlink substitution.

## Exercises

1. Trace the resolution of the path `/../../etc/passwd` step by step. What does `..` do at the root directory?

2. Why are `stat()` and `lstat()` separate syscalls rather than having a single syscall that takes a flag?

3. What is a TOCTOU attack on path resolution? Give an example and explain how `openat()` prevents it.
