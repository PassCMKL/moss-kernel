# Userspace Tests

Userspace tests run as regular user programs inside Moss. They test the system call interface from the user's perspective, verifying Linux ABI compatibility.

## Running Userspace Tests

```bash
just test-userspace
```

This builds the kernel and the `usertest` binary, creates a disk image, boots QEMU, runs `/usertest`, and reports results.

Expected output:

```
[usertest] Running syscall tests...
[usertest] test_write ... ok
[usertest] test_read_write_file ... ok
[usertest] test_fork_returns_pid ... ok
[usertest] test_execve_ls ... ok
[usertest] test_signal_sigint ... ok
[usertest] test_mmap_anonymous ... ok
[usertest] test_mmap_file ... ok
[usertest] test_pipe ... ok
[usertest] test_waitpid ... ok
...
[usertest] 48/48 tests passed
```

## The `usertest` Binary

The `usertest` binary (`usertest/` crate) is a Rust program compiled for AArch64 Linux (not for bare metal). It uses the musl libc target for static linking, so it has no dynamic library dependencies.

The tests use a simple framework:

```rust
// usertest/src/main.rs
fn main() {
    let tests: &[(&str, fn() -> Result<()>)] = &[
        ("test_write", test_write),
        ("test_read_write_file", test_read_write_file),
        ("test_fork_returns_pid", test_fork_returns_pid),
        // ...
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (name, test_fn) in tests {
        print!("[usertest] {} ... ", name);
        match test_fn() {
            Ok(()) => {
                println!("ok");
                passed += 1;
            }
            Err(e) => {
                println!("FAILED: {}", e);
                failed += 1;
            }
        }
    }

    println!("[usertest] {}/{} tests passed", passed, passed + failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
```

## Example Test Cases

### Testing `fork`

```rust
fn test_fork_returns_pid() -> Result<()> {
    let pid = unsafe { libc::fork() };

    if pid == 0 {
        // Child: verify our PID is a new PID
        assert_ne!(unsafe { libc::getpid() }, unsafe { libc::getppid() });
        unsafe { libc::_exit(0) };
    } else {
        // Parent: verify child PID is valid
        assert!(pid > 0, "fork returned negative");
        let mut status = 0;
        let waited_pid = unsafe { libc::waitpid(pid, &mut status, 0) };
        assert_eq!(waited_pid, pid);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
    }

    Ok(())
}
```

### Testing `mmap`

```rust
fn test_mmap_anonymous() -> Result<()> {
    let size = 4096;
    let addr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    assert_ne!(addr, libc::MAP_FAILED, "mmap failed");

    // Write and read back
    let ptr = addr as *mut u8;
    unsafe { *ptr = 42 };
    assert_eq!(unsafe { *ptr }, 42);

    // Unmap
    let ret = unsafe { libc::munmap(addr, size) };
    assert_eq!(ret, 0, "munmap failed");

    Ok(())
}
```

### Testing Signals

```rust
fn test_signal_sigalrm() -> Result<()> {
    static GOT_ALARM: AtomicBool = AtomicBool::new(false);

    extern "C" fn handler(_: libc::c_int) {
        GOT_ALARM.store(true, Ordering::SeqCst);
    }

    unsafe {
        libc::signal(libc::SIGALRM, handler as libc::sighandler_t);
        libc::alarm(1);  // Send SIGALRM after 1 second
        libc::pause();   // Wait for a signal
    }

    assert!(GOT_ALARM.load(Ordering::SeqCst), "SIGALRM was not delivered");
    Ok(())
}
```

## What Userspace Tests Validate

Userspace tests are the final check that Moss is truly Linux-compatible:
- Correct system call return values and error codes
- Correct signal delivery timing and semantics
- Memory management (mmap, brk, page faults)
- File operations (open, read, write, seek, stat)
- Process management (fork, exec, wait, credentials)
- Pipes and inter-process communication

## Adding a New Test

To add a test for a new syscall:

1. Add a test function to `usertest/src/` (create a new file or add to an existing one)
2. Register it in the test table in `main.rs`
3. Run `just test-userspace` to verify it passes

## Exercises

1. Run `just test-userspace`. Which tests pass? Are there any failures? What do failing tests tell you about unimplemented features?

2. Write a userspace test for `pipe`: create a pipe, fork, have the parent write to the write end, and have the child read from the read end.

3. Write a test that verifies copy-on-write semantics: fork, have the child write to a variable, and verify the parent's copy is unchanged.
