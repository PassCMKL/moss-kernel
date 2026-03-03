# What Are Signals?

Signals are small integer notifications sent to a process. Unlike regular inter-process communication (pipes, shared memory), signals:

- Carry no data (just a signal number)
- Are delivered asynchronously
- Can interrupt a sleeping process
- Have default actions if no handler is installed

## Standard Signals

Moss supports the standard POSIX signals:

| Signal | Number | Default Action | Description |
|---|---|---|---|
| `SIGHUP` | 1 | Terminate | Hangup detected on controlling terminal |
| `SIGINT` | 2 | Terminate | Interrupt (Ctrl+C) |
| `SIGQUIT` | 3 | Core dump | Quit (Ctrl+\\) |
| `SIGILL` | 4 | Core dump | Illegal instruction |
| `SIGTRAP` | 5 | Core dump | Trace/breakpoint trap |
| `SIGABRT` | 6 | Core dump | Abort signal (`abort()`) |
| `SIGFPE` | 8 | Core dump | Floating-point exception |
| `SIGKILL` | 9 | Terminate | Kill (cannot be caught or ignored) |
| `SIGSEGV` | 11 | Core dump | Segmentation fault |
| `SIGPIPE` | 13 | Terminate | Broken pipe (write to pipe with no readers) |
| `SIGALRM` | 14 | Terminate | Timer alarm |
| `SIGTERM` | 15 | Terminate | Termination signal (graceful) |
| `SIGCHLD` | 17 | Ignore | Child stopped or exited |
| `SIGCONT` | 18 | Continue | Continue if stopped |
| `SIGSTOP` | 19 | Stop | Stop process (cannot be caught or ignored) |
| `SIGTSTP` | 20 | Stop | Stop typed at terminal (Ctrl+Z) |
| `SIGUSR1` | 10 | Terminate | User-defined signal 1 |
| `SIGUSR2` | 12 | Terminate | User-defined signal 2 |

## Real-Time Signals

In addition to the standard signals, POSIX defines real-time signals (`SIGRTMIN` to `SIGRTMAX`, signals 34–64). Unlike standard signals:
- Multiple instances of the same real-time signal can be pending simultaneously
- They are delivered in order (lowest number first)
- They can carry data (via `sigqueue`)

Moss supports real-time signals.

## The Signal Bitmask

Moss represents the set of pending or masked signals as a 64-bit bitmask:

```rust
pub struct SigSet(u64);

impl SigSet {
    pub fn contains(&self, sig: Signal) -> bool {
        self.0 & (1 << (sig as u32 - 1)) != 0
    }

    pub fn add(&mut self, sig: Signal) {
        self.0 |= 1 << (sig as u32 - 1);
    }

    pub fn remove(&mut self, sig: Signal) {
        self.0 &= !(1 << (sig as u32 - 1));
    }
}
```

This allows checking whether any of a set of signals is pending with a single AND operation.

## Default Actions

If a process hasn't installed a signal handler, the kernel performs the **default action**:

- **Terminate**: Kill the process
- **Core dump**: Kill the process and write a core file (memory snapshot for debugging)
- **Ignore**: Silently discard the signal
- **Stop**: Pause the process (SIGSTOP, SIGTSTP)
- **Continue**: Resume a stopped process (SIGCONT)

## Non-Catchable Signals

**SIGKILL** and **SIGSTOP** cannot be caught, blocked, or ignored. This ensures:
- A process can always be killed by root
- A process can always be stopped by the terminal

If these signals could be caught, a malicious program could make itself unkillable.

## Exercises

1. Why do `SIGKILL` and `SIGSTOP` not allow signal handlers? What security property would be violated if they did?

2. What is the difference between `SIGTERM` and `SIGKILL`? When should you send each?

3. A process receives `SIGSEGV` due to a null pointer dereference. What happens by default? What would happen if the process installed a `SIGSEGV` handler?
