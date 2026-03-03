# Syscall Reference

Moss implements approximately 105 Linux-compatible system calls. This chapter provides an organized overview of the major categories.

## Process Management

| Syscall | Number | Description |
|---|---|---|
| `fork` | 1079 | Create a child process (copy current) |
| `clone` | 220 | Create a thread or process with shared resources |
| `execve` | 221 | Replace process image with new program |
| `exit` | 93 | Terminate current thread |
| `exit_group` | 94 | Terminate all threads in the process |
| `wait4` | 260 | Wait for a child process to change state |
| `waitpid` | — | Wait for specific child (legacy) |
| `getpid` | 172 | Get process ID (TGID) |
| `getppid` | 173 | Get parent process ID |
| `gettid` | 178 | Get thread ID |
| `sched_yield` | 124 | Voluntarily yield CPU to other tasks |
| `prctl` | 167 | Process control operations |

## File Descriptor Operations

| Syscall | Number | Description |
|---|---|---|
| `read` | 63 | Read from file descriptor |
| `write` | 64 | Write to file descriptor |
| `close` | 57 | Close file descriptor |
| `openat` | 56 | Open file (relative to directory FD) |
| `lseek` | 62 | Seek to position in file |
| `dup` | 23 | Duplicate file descriptor |
| `dup3` | 24 | Duplicate FD to specified number |
| `pipe2` | 59 | Create pipe |
| `fcntl` | 25 | File descriptor control operations |
| `ioctl` | 29 | Device-specific control operations |
| `poll` | 73 | Wait for events on multiple FDs |
| `ppoll` | 73 | `poll` with signal mask |
| `select` | — | Wait for events (legacy) |

## Filesystem Operations

| Syscall | Number | Description |
|---|---|---|
| `fstat` | 80 | Get file metadata by FD |
| `newfstatat` | 79 | Get file metadata by path |
| `mkdirat` | 34 | Create directory |
| `unlinkat` | 35 | Remove file or directory |
| `renameat` | 38 | Rename file |
| `symlinkat` | 36 | Create symbolic link |
| `readlinkat` | 78 | Read symbolic link |
| `fchmodat` | 53 | Change file permissions |
| `fchownat` | 54 | Change file owner |
| `truncate` | 45 | Truncate file to length |
| `ftruncate` | 46 | Truncate file by FD |
| `getcwd` | 17 | Get current working directory |
| `chdir` | 49 | Change current directory |
| `fchdir` | 50 | Change directory by FD |
| `chroot` | 51 | Change root directory |
| `mount` | 40 | Mount filesystem |
| `umount2` | 39 | Unmount filesystem |

## Memory Management

| Syscall | Number | Description |
|---|---|---|
| `mmap` | 222 | Map memory or files into address space |
| `munmap` | 215 | Unmap memory region |
| `mprotect` | 226 | Change memory protection |
| `brk` | 214 | Adjust program break (heap end) |
| `msync` | 227 | Synchronize memory-mapped file |
| `madvise` | 233 | Memory usage hints |

## Credentials

| Syscall | Number | Description |
|---|---|---|
| `getuid` | 174 | Get real user ID |
| `getgid` | 176 | Get real group ID |
| `geteuid` | 175 | Get effective user ID |
| `getegid` | 177 | Get effective group ID |
| `setuid` | 146 | Set user ID |
| `setgid` | 144 | Set group ID |
| `setreuid` | 145 | Set real and effective user IDs |
| `setregid` | 143 | Set real and effective group IDs |
| `setresuid` | 147 | Set real, effective, and saved UIDs |
| `setresgid` | 149 | Set real, effective, and saved GIDs |
| `setgroups` | 159 | Set supplementary group list |
| `getgroups` | 158 | Get supplementary group list |
| `setpgid` | 154 | Set process group ID |
| `getpgid` | 155 | Get process group ID |
| `setsid` | 157 | Create new session |

## Signals

| Syscall | Number | Description |
|---|---|---|
| `kill` | 129 | Send signal to process or group |
| `tkill` | 130 | Send signal to specific thread |
| `rt_sigaction` | 134 | Set signal handler |
| `rt_sigprocmask` | 135 | Get/set blocked signal mask |
| `rt_sigpending` | 136 | Get pending signals |
| `rt_sigsuspend` | 133 | Wait for signal |
| `rt_sigreturn` | 139 | Return from signal handler |
| `sigaltstack` | 132 | Set alternate signal stack |

## Time

| Syscall | Number | Description |
|---|---|---|
| `clock_gettime` | 113 | Get time from clock |
| `clock_settime` | 112 | Set time on clock |
| `clock_nanosleep` | 115 | High-resolution sleep |
| `nanosleep` | 101 | Sleep for a duration |
| `gettimeofday` | — | Get current time (legacy) |
| `timer_create` | 107 | Create POSIX timer |
| `timer_settime` | 110 | Arm POSIX timer |
| `timer_delete` | 111 | Delete POSIX timer |

## System Information

| Syscall | Number | Description |
|---|---|---|
| `uname` | 160 | Get system information |
| `sysinfo` | 179 | Get system statistics |
| `getrlimit` | 163 | Get resource limit |
| `setrlimit` | 164 | Set resource limit |
| `prlimit64` | 261 | Get/set resource limit |
| `ptrace` | 117 | Process tracing (debugging) |

## Unimplemented / Stub Syscalls

Some syscalls are known to be needed but not yet implemented. Moss returns `ENOSYS` for unknown syscall numbers and `ENOTSUP` for stub syscalls.

Common stubs: `socket`, `bind`, `connect` (networking not yet implemented), `epoll_*` (partial implementation), `inotify_*`.

## Exercises

1. The `clone` syscall creates both threads and processes. What flags control whether the address space, file descriptor table, and signal handlers are shared or copied?

2. Why does `execve` close file descriptors with the `O_CLOEXEC` flag but not others?

3. `mmap` with `MAP_ANONYMOUS|MAP_PRIVATE` is commonly used for `malloc`. Trace the life of a `malloc(1024)` call: what syscalls are made, and what page faults occur?
