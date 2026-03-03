# Job Control

**Job control** is the ability to manage multiple commands in a terminal session: starting them in the background, bringing them to the foreground, suspending and resuming them. It is implemented through process groups and signals.

## Process Groups

Every process belongs to a **process group** (identified by its PGID — Process Group ID). The process group was originally created to support the concept of a "job" in a shell.

```
Shell (PID=100, PGID=100)
├── "ls | grep foo" — a pipeline (one job, one process group)
│    ├── ls (PID=101, PGID=101)
│    └── grep (PID=102, PGID=101)  ← same PGID as ls
└── "vim" (PID=103, PGID=103)     ← different job, different PGID
```

When you press `Ctrl+C`, the terminal sends `SIGINT` to the **entire foreground process group** — not just one process. This kills the entire pipeline atomically.

## Foreground and Background Jobs

A shell implements job control by:
1. Creating each pipeline in a new process group
2. Assigning one process group as the **foreground** (connected to the terminal)
3. All other process groups are **background** (cannot read from/write to the terminal)

```bash
vim &           # Start vim in background
ls | grep foo   # Run in foreground (blocks the terminal)
```

If a background process tries to read from the terminal, it receives `SIGTTIN` (which stops it by default).

## The `setpgid` Syscall

When a shell creates a new job:

```c
pid_t pid = fork();
if (pid == 0) {
    // Child: move to new process group
    setpgid(0, 0);  // PGID = my own PID (create new group)
    exec(...);
}
// Parent: also set the child's PGID (race-free)
setpgid(pid, pid);
```

Both parent and child call `setpgid` to avoid a race condition: either might run first.

## Stopping and Continuing

`SIGSTOP` and `SIGTSTP` suspend a process group:

```
Process state: Running → Stopped
```

`SIGCONT` resumes a stopped process group:

```
Process state: Stopped → Running
```

The shell tracks the state of each job and uses `waitpid(pid, WUNTRACED)` to detect when a foreground job is stopped (e.g., when the user presses `Ctrl+Z`).

## Implementation in Moss

Moss tracks the PGID in the `ThreadGroup`:

```rust
pub struct ThreadGroup {
    pub pgid: AtomicPgid,
    // ...
}
```

When `kill(-pgid, sig)` is called (negative PID = send to process group):

```rust
pub fn send_signal_to_group(pgid: Pgid, sig: Signal) {
    for process in all_processes() {
        if process.pgid == pgid {
            send_signal_to_thread_group(&process, sig);
        }
    }
}
```

For `SIGCONT` specifically, all threads in stopped processes must be woken:

```rust
fn handle_sigcont(process: &ThreadGroup) {
    if process.state == ThreadGroupState::Stopped {
        process.state = ThreadGroupState::Running;

        // Wake all tasks in the group
        for task in process.tasks.lock().values() {
            if let Some(task) = task.upgrade() {
                task.state.store(TaskState::Runnable);
                scheduler().enqueue(task);
            }
        }
    }
}
```

## Sessions and Controlling Terminals

Beyond process groups, processes also belong to **sessions** (SID). A session typically corresponds to a login: one terminal connection or SSH session.

The **controlling terminal** is the terminal associated with a session. When the terminal is closed (user logs out), `SIGHUP` is sent to all processes in the foreground process group.

Sessions are created with `setsid()` — this is how daemon processes detach from their controlling terminal.

## Exercises

1. What happens to background jobs when you close a terminal? How can you prevent them from being killed?

2. The shell command `Ctrl+Z` pauses a job and returns control to the shell. Trace the sequence of events: key press → signal → process state change → shell prompt.

3. What is a "daemon" process? What steps does a program typically take to "daemonize" itself (become a background service detached from any terminal)?
