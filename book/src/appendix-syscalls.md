# Appendix A: Syscall Table

This appendix provides a complete reference of syscalls implemented in Moss, organized by number.

The syscall numbers follow the Linux AArch64 ABI. Programs compiled for Linux can call these syscalls on Moss without modification.

## Status Legend

| Symbol | Meaning |
|---|---|
| ✓ | Fully implemented |
| ~ | Partially implemented (some flags or edge cases missing) |
| ✗ | Not yet implemented (returns ENOSYS) |

## Syscall Table

| Number | Name | Status | Notes |
|---|---|---|---|
| 0 | `io_setup` | ✗ | |
| 1 | `io_destroy` | ✗ | |
| 17 | `getcwd` | ✓ | |
| 23 | `dup` | ✓ | |
| 24 | `dup3` | ✓ | |
| 25 | `fcntl` | ~ | Basic flags only |
| 29 | `ioctl` | ~ | Terminal ioctls |
| 34 | `mkdirat` | ✓ | |
| 35 | `unlinkat` | ✓ | |
| 36 | `symlinkat` | ✓ | |
| 37 | `linkat` | ~ | |
| 38 | `renameat` | ✓ | |
| 39 | `umount2` | ✓ | |
| 40 | `mount` | ✓ | |
| 45 | `truncate` | ✓ | |
| 46 | `ftruncate` | ✓ | |
| 49 | `chdir` | ✓ | |
| 50 | `fchdir` | ✓ | |
| 51 | `chroot` | ✓ | |
| 52 | `fchmod` | ✓ | |
| 53 | `fchmodat` | ✓ | |
| 54 | `fchownat` | ✓ | |
| 55 | `fchown` | ✓ | |
| 56 | `openat` | ✓ | |
| 57 | `close` | ✓ | |
| 59 | `pipe2` | ✓ | |
| 62 | `lseek` | ✓ | |
| 63 | `read` | ✓ | |
| 64 | `write` | ✓ | |
| 65 | `readv` | ✓ | |
| 66 | `writev` | ✓ | |
| 67 | `pread64` | ✓ | |
| 68 | `pwrite64` | ✓ | |
| 73 | `ppoll` | ~ | |
| 78 | `readlinkat` | ✓ | |
| 79 | `newfstatat` | ✓ | |
| 80 | `fstat` | ✓ | |
| 93 | `exit` | ✓ | |
| 94 | `exit_group` | ✓ | |
| 96 | `set_tid_address` | ✓ | |
| 101 | `nanosleep` | ✓ | |
| 107 | `timer_create` | ✓ | |
| 108 | `timer_gettime` | ✓ | |
| 109 | `timer_getoverrun` | ✓ | |
| 110 | `timer_settime` | ✓ | |
| 111 | `timer_delete` | ✓ | |
| 112 | `clock_settime` | ✓ | |
| 113 | `clock_gettime` | ✓ | |
| 115 | `clock_nanosleep` | ✓ | |
| 117 | `ptrace` | ~ | Basic only |
| 124 | `sched_yield` | ✓ | |
| 129 | `kill` | ✓ | |
| 130 | `tkill` | ✓ | |
| 131 | `tgkill` | ✓ | |
| 132 | `sigaltstack` | ✓ | |
| 133 | `rt_sigsuspend` | ✓ | |
| 134 | `rt_sigaction` | ✓ | |
| 135 | `rt_sigprocmask` | ✓ | |
| 136 | `rt_sigpending` | ✓ | |
| 137 | `rt_sigtimedwait` | ✓ | |
| 139 | `rt_sigreturn` | ✓ | |
| 143 | `setregid` | ✓ | |
| 144 | `setgid` | ✓ | |
| 145 | `setreuid` | ✓ | |
| 146 | `setuid` | ✓ | |
| 147 | `setresuid` | ✓ | |
| 148 | `getresuid` | ✓ | |
| 149 | `setresgid` | ✓ | |
| 150 | `getresgid` | ✓ | |
| 154 | `setpgid` | ✓ | |
| 155 | `getpgid` | ✓ | |
| 157 | `setsid` | ✓ | |
| 158 | `getgroups` | ✓ | |
| 159 | `setgroups` | ✓ | |
| 160 | `uname` | ✓ | |
| 163 | `getrlimit` | ~ | |
| 164 | `setrlimit` | ~ | |
| 167 | `prctl` | ~ | |
| 172 | `getpid` | ✓ | Returns TGID |
| 173 | `getppid` | ✓ | |
| 174 | `getuid` | ✓ | |
| 175 | `geteuid` | ✓ | |
| 176 | `getgid` | ✓ | |
| 177 | `getegid` | ✓ | |
| 178 | `gettid` | ✓ | |
| 179 | `sysinfo` | ~ | |
| 214 | `brk` | ✓ | |
| 215 | `munmap` | ✓ | |
| 220 | `clone` | ✓ | |
| 221 | `execve` | ✓ | |
| 222 | `mmap` | ~ | Most flags |
| 226 | `mprotect` | ✓ | |
| 227 | `msync` | ~ | |
| 233 | `madvise` | ~ | |
| 260 | `wait4` | ✓ | |
| 261 | `prlimit64` | ~ | |
| 1079 | `fork` | ✓ | Alias for clone |

## Not Yet Implemented

The following categories are not yet implemented:

- **Networking**: `socket`, `bind`, `connect`, `listen`, `accept`, `send`, `recv`, etc.
- **`epoll`**: `epoll_create`, `epoll_ctl`, `epoll_wait`
- **`inotify`**: File system change notifications
- **`io_uring`**: Asynchronous I/O interface
- **`seccomp`**: System call filtering
- **`namespaces`**: Container isolation (`unshare`, `setns`)
