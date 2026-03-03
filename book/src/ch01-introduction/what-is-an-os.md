# What Is an Operating System?

An operating system (OS) is software that manages hardware resources and provides services to application programs. It occupies the boundary between hardware and user-space software, acting simultaneously as:

1. **A resource manager** — The OS decides which process gets CPU time, how much memory each process can use, and which process can access a particular file at a given moment.

2. **An abstraction layer** — Rather than exposing raw hardware (memory-mapped registers, sector numbers, interrupt lines), the OS provides clean, portable abstractions: processes, files, sockets, and signals.

3. **A protection boundary** — User programs are isolated from one another and from the kernel. A bug in one program cannot corrupt another program's memory, and a user program cannot directly manipulate hardware.

## The Kernel vs. the Operating System

People often use "operating system" and "kernel" interchangeably, but they are distinct:

- The **kernel** is the core software that runs with full hardware privilege. It manages memory, schedules processes, handles interrupts, and implements system calls.
- The **operating system** includes the kernel plus user-space tools: a shell, libraries (like libc), system daemons, package managers, and so on.

Moss is a **kernel**. It is designed to boot, run a userspace (currently a BusyBox-based Arch Linux root filesystem), and expose a Linux-compatible system call interface to those user programs.

## Kernel Architectures

Different kernels make different trade-offs about how much code runs with full hardware privilege.

### Monolithic Kernels

In a monolithic kernel, virtually all OS services — scheduling, memory management, filesystems, device drivers, networking — run in a single privileged address space. Communication between subsystems is a direct function call.

**Pros:** Fast (no context switches between subsystems), simple to implement initially.
**Cons:** A bug anywhere in kernel space can crash or compromise the entire system; adding new drivers means adding privileged code.

Linux and FreeBSD are monolithic kernels.

### Microkernels

In a microkernel, only the absolute minimum runs in privileged mode: address space management and message passing. Everything else — filesystems, drivers, even parts of the scheduler — runs as unprivileged user-space servers.

**Pros:** Faults are isolated (a crashing driver cannot take down the kernel); smaller trusted computing base.
**Cons:** Inter-process communication overhead; more complex to build.

seL4, MINIX 3, and QNX are microkernel examples.

### Moss's Position

Moss is architecturally a **monolithic kernel** with clean internal abstractions. Filesystems, drivers, and the scheduler all run in privileged kernel space. However, Moss uses Rust's type system and async/await model to enforce safety properties that traditionally required microkernel isolation. More on this in the [Why Rust?](./why-rust.md) section.

## The Hardware Privilege Model

Modern CPUs enforce hardware privilege levels. On AArch64 (the architecture Moss targets), these are called **Exception Levels**:

```
EL3  ──  Secure Monitor (firmware, typically not used by OS)
EL2  ──  Hypervisor
EL1  ──  Kernel (Moss runs here)
EL0  ──  User programs
```

Code running at EL1 can execute privileged instructions (e.g., modifying page tables, enabling/disabling interrupts) that are illegal at EL0. When a user program needs a kernel service, it executes a special instruction (`svc #0` on AArch64) that causes the CPU to switch to EL1 and jump to a handler registered by the kernel. This is the **system call** mechanism, which we explore in depth in [Chapter 6](../ch06-syscalls/README.md).

## Key Concepts to Keep in Mind

Throughout this book, several recurring themes will appear:

| Concept | Meaning |
|---|---|
| **Process isolation** | Each process believes it owns the entire machine; the kernel enforces this illusion |
| **Privilege separation** | User code cannot directly access hardware or other processes' memory |
| **Abstraction** | Hardware details are hidden behind clean interfaces (files, sockets, etc.) |
| **Concurrency** | Multiple processes and interrupts can all demand service simultaneously |
| **Safety** | Bugs in one component should not corrupt other components |

Moss addresses each of these, and each chapter will show you how.

## Exercises

1. What is the difference between a kernel and an operating system? Give two examples of components that are part of a Linux-based OS but not part of the Linux kernel.

2. In a monolithic kernel, if a device driver has a buffer overflow bug, what is the worst-case impact? How does this differ in a microkernel?

3. Look up the concept of a "unikernel." How does it differ from both a monolithic kernel and a microkernel? What workloads might it be suited for?
