# Tasks and Thread Groups

Moss uses the same two-level model as Linux: **tasks** (threads) and **thread groups** (processes).

## The `Task` Struct

A `Task` represents a single thread of execution. It is the fundamental scheduling unit — the scheduler works with tasks, not with processes. The `Task` struct lives in `src/process/mod.rs`:

```rust
pub struct Task {
    // Identity
    pub tid: Tid,                         // Unique thread ID
    pub comm: SpinLock<ArrayString<16>>,  // Thread name (e.g., "bash")

    // Group membership
    pub thread_group: Arc<ThreadGroup>,   // The process this thread belongs to

    // CPU scheduling state
    pub sched_state: SchedTaskState,      // Scheduler priority, virtual time, etc.
    pub state: AtomicTaskState,           // Running/Runnable/Sleeping/Stopped/Finished

    // Architecture state
    pub arch_state: ArchTaskState,        // Saved registers (when not running)

    // CPU accounting
    pub utime: AtomicU64,                 // Time spent in user mode
    pub stime: AtomicU64,                 // Time spent in kernel mode

    // Signal delivery
    pub sig_mask: SpinLock<SigSet>,       // Blocked signals
    pub pending_signals: AtomicSigSet,    // Signals awaiting delivery
}
```

The `Task` struct deliberately does **not** own the address space, file descriptor table, or credentials — those belong to the `ThreadGroup`. This is what allows threads to share them.

## The `ThreadGroup` Struct

A `ThreadGroup` represents a process — a collection of threads that share:
- A virtual address space
- File descriptor table
- Credentials (UID, GID, groups)
- Signal handlers
- Working directory

```rust
pub struct ThreadGroup {
    // Identity
    pub tgid: Pid,                    // Thread Group ID (= PID in Unix)
    pub pgid: AtomicPgid,             // Process Group ID (for job control)

    // Shared resources
    pub address_space: Arc<ProcessAddressSpace>,
    pub file_table: Arc<FileDescriptorTable>,
    pub creds: Arc<RwLock<Credentials>>,
    pub sig_actions: Arc<SpinLock<SignalActionState>>,
    pub cwd: Arc<RwLock<Inode>>,      // Current working directory

    // Thread membership
    pub tasks: SpinLock<BTreeMap<Tid, Weak<Task>>>,  // All threads in group

    // Process hierarchy
    pub parent: Weak<ThreadGroup>,    // Parent process
    pub children: SpinLock<BTreeMap<Pid, Arc<ThreadGroup>>>,

    // Exit information
    pub exit_code: AtomicI32,
    pub state: ThreadGroupState,      // Running, Stopped, Zombie
}
```

## The Relationship Between Task and ThreadGroup

```
ThreadGroup (process/PID 42)
    ├── address_space: VMA tree, page tables
    ├── file_table: FD → OpenFile map
    ├── creds: uid=1000, gid=1000
    └── tasks:
          ├── Task (TID 42) — main thread
          ├── Task (TID 43) — worker thread
          └── Task (TID 44) — I/O thread
```

In a single-threaded process, TID == PID (the only task's TID equals the thread group's TGID). In a multi-threaded process, there are multiple tasks with different TIDs, all sharing the same TGID.

## Unique IDs

| ID | What it identifies |
|---|---|
| TID | A specific thread (unique system-wide) |
| PID / TGID | A process (= TID of the first thread) |
| PGID | A process group (for job control) |
| SID | A session (for terminal control) |

When a user-space program calls `getpid()`, it actually gets the TGID. When it calls `gettid()`, it gets the actual TID.

## Arc and Weak References

Moss uses Rust's reference counting (`Arc<T>`) for most kernel objects. The `Arc<Task>` held by the scheduler keeps a task alive. When the task exits and the scheduler drops its reference, the task is cleaned up.

Circular references (e.g., parent ↔ child) use `Weak<T>` — a non-owning reference that returns `None` if the target has been freed. This prevents memory leaks from reference cycles.

## Exercises

1. Why doesn't `Task` own the address space directly? What would happen if it did in a multi-threaded program?

2. When a thread exits but other threads in the same process are still running, what should happen to the `ThreadGroup`? When should the `ThreadGroup` be freed?

3. In Linux, calling `clone()` with different flags creates either a new thread or a new process. What flags control which resources are shared versus copied?
