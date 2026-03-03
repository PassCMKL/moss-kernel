# Creating Processes: fork and exec

New processes in Unix are created through two complementary system calls: `fork` (which duplicates the current process) and `exec` (which replaces the current process image with a new program). Together they form the foundation of Unix process management.

## fork: Duplicating a Process

`fork()` creates an exact copy of the calling process. After `fork()` returns:
- There are two processes: the **parent** (original) and the **child** (copy)
- Both processes resume at the exact same instruction (right after the `fork()` call)
- The only observable difference is the return value of `fork()`: the parent gets the child's PID, the child gets 0

```c
pid_t pid = fork();
if (pid == 0) {
    // We are the child
    printf("I am the child, PID = %d\n", getpid());
} else {
    // We are the parent
    printf("I spawned child with PID = %d\n", pid);
}
```

### What fork Copies

Moss's `fork()` is implemented via `sys_clone()` with appropriate flags. It creates:

1. **A new `Task`** with a new TID and its own scheduler state
2. **A new `ThreadGroup`** with a new TGID (= new TID)
3. **A copy of the address space** — using copy-on-write (see [CoW](../ch03-memory/virtual/cow.md))
4. **A copy of the file descriptor table** — file descriptors are shared references, so both parent and child have open descriptors to the same files
5. **Copied credentials** — UID, GID, groups
6. **Copied signal handlers** — same handlers for the same signals

The new `Task` starts in the `Runnable` state and is placed in the scheduler queue.

### The Architecture-Specific Part: Fork Return

After fork, both parent and child must return "from" the `fork()` system call. The parent naturally continues because it was already running. The child needs the kernel to fake a return:

```rust
fn fork_arch_state(parent_state: &ExceptionState) -> ArchTaskState {
    let mut child_state = parent_state.clone();
    // Set X0 to 0 — child's "return value" from fork
    child_state.x[0] = 0;
    // ELR_EL1 (return address) is copied from parent, so child resumes
    // at the same user-space instruction
    child_state
}
```

When the child is first scheduled, it returns to user space with X0=0, making it look like `fork()` returned 0 to the child.

## exec: Replacing the Process Image

`execve(path, argv, envp)` replaces the current process with a new program loaded from `path`. Everything about the current process changes:
- The address space is discarded and rebuilt for the new program
- Open file descriptors may be closed (if `O_CLOEXEC` is set)
- The program counter is reset to the new program's entry point
- The stack is set up with `argv` and `envp`

But the PID and TID stay the same — it's still the same process, just running a new program.

### ELF Loading

Moss supports **ELF** (Executable and Linkable Format) binaries, the standard format on Linux/Unix. An ELF file contains:
- A header describing the binary (architecture, entry point, etc.)
- **Program headers** (segments): `.text` (code), `.data` (initialized data), `.bss` (zero-initialized data)
- **Section headers**: debug info, symbol tables, etc.

Loading an ELF:

```rust
pub async fn load_elf(path: &str, argv: Vec<String>, envp: Vec<String>) -> Result<()> {
    let file = open(path)?;
    let elf = parse_elf_header(&file).await?;

    // Set up a new address space
    let new_as = AddressSpace::new();

    // Map each loadable segment into the new address space
    for segment in elf.load_segments() {
        new_as.map_segment(segment, &file).await?;
    }

    // Map the dynamic linker if needed
    if elf.is_dynamic() {
        load_dynamic_linker(&new_as, &elf).await?;
    }

    // Atomically replace the current address space
    current_task().thread_group.replace_address_space(new_as);

    // Set up the initial stack with argv, envp, and the auxiliary vector
    setup_user_stack(argv, envp, &elf)?;

    // Jump to the entry point
    return_to_user(elf.entry_point());
}
```

### Setting Up the User Stack

Before the new program's `main()` can run, the kernel must set up the stack in a format that the C runtime (`libc`) expects:

```
High addresses
┌─────────────────────┐
│  environment strings│ "HOME=/root\0PATH=/bin\0..."
│  argument strings   │ "/bin/ls\0-la\0/tmp\0"
├─────────────────────┤
│  NULL               │ (end of envp)
│  envp[n-1]          │ pointer to "PATH=/bin"
│  ...                │
│  envp[0]            │ pointer to "HOME=/root"
├─────────────────────┤
│  NULL               │ (end of argv)
│  argv[argc-1]       │ pointer to "/tmp"
│  argv[1]            │ pointer to "-la"
│  argv[0]            │ pointer to "/bin/ls"
├─────────────────────┤
│  argc               │ = 3
├─────────────────────┤
│  Auxiliary Vector   │ AT_ENTRY, AT_PHDR, AT_PHNUM, AT_PAGESZ...
│  (AT_NULL)          │
└─────────────────────┘
Low addresses (stack pointer here)
```

The **auxiliary vector** provides the dynamic linker with information it needs to relocate the program and set up the C runtime.

### Dynamic Linking Bias

For dynamically linked programs, Moss uses a fixed load bias:
- Main program mapped at: `0x5000_0000_0000`
- Dynamic linker (`ld.so`) mapped at: `0x7000_0000_0000`

These biases are similar to what Linux uses, ensuring that programs compiled for Linux work correctly on Moss.

## fork + exec Together

The classic pattern is:

```c
pid_t pid = fork();
if (pid == 0) {
    // Child: close unwanted FDs, then exec
    close(unused_fd);
    execve("/bin/ls", argv, envp);
    // If exec returns, something went wrong
    _exit(1);
}
// Parent: wait for child if desired
waitpid(pid, &status, 0);
```

Thanks to copy-on-write, the `fork()` is cheap even for a large process, and the child's memory is immediately discarded by `exec()`.

## Exercises

1. What is the difference between `fork()` and `vfork()`? Why was `vfork()` historically necessary, and why is it less important today?

2. When `exec()` is called in a multi-threaded process, what happens to the other threads?

3. The auxiliary vector includes `AT_RANDOM` — 16 random bytes. What is this used for?
