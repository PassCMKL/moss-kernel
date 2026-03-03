# Appendix C: Glossary

**Address Space**: The range of virtual addresses that a process can potentially use. On AArch64, each process has a 48-bit user address space (lower half) and shares the kernel address space (upper half).

**AArch64**: The 64-bit ARM instruction set architecture (also called ARM64). Moss targets AArch64 running on the QEMU `virt` virtual machine.

**ASLR** (Address Space Layout Randomization): A security technique that randomizes the base addresses of the stack, heap, and loaded libraries to make exploitation harder.

**Async/Await**: A Rust language feature for writing asynchronous code. Functions marked `async fn` return a `Future` instead of a direct value. The `.await` operator suspends execution until the `Future` completes, allowing other work to proceed.

**Buddy Allocator**: A physical memory allocator that manages memory in power-of-two-sized blocks. Free blocks can be merged with their "buddy" to form larger blocks.

**Context Switch**: The act of saving one task's register state and restoring another's, allowing the CPU to run a different task. Includes switching page tables when moving between processes.

**CoW** (Copy-on-Write): An optimization where multiple processes share the same physical pages until one of them writes to a page, at which point a private copy is made.

**DAIF**: Debug, Abort, IRQ, FIQ — the four interrupt mask bits on AArch64. When a bit is set, the corresponding type of interrupt is masked.

**Demand Paging**: Loading pages from storage into RAM only when they are first accessed (causing a page fault), rather than eagerly at process creation.

**DTB** (Device Tree Blob): A binary description of a system's hardware that the bootloader passes to the kernel. Used on ARM systems to describe memory, CPUs, and peripherals.

**EEVDF** (Earliest Eligible Virtual Deadline First): The scheduling algorithm used in Moss (and Linux 6.6+). It selects the eligible task with the earliest virtual deadline to run next.

**EL** (Exception Level): ARM's hardware privilege levels. EL0 = user space, EL1 = kernel, EL2 = hypervisor, EL3 = secure monitor.

**ELF** (Executable and Linkable Format): The standard binary format on Linux/Unix. Contains code, data, and metadata needed to load and run a program.

**`eret`**: AArch64 instruction that atomically returns from an exception, restoring the program counter from `ELR_EL1` and the processor state from `SPSR_EL1`.

**ESR_EL1**: Exception Syndrome Register at EL1. Contains information about why an exception occurred (exception class, fault type, etc.).

**FAR_EL1**: Fault Address Register at EL1. Contains the virtual address that caused a page fault.

**Frame Allocator**: The physical memory allocator that manages page frames (physical pages of RAM).

**GIC** (Generic Interrupt Controller): The standard interrupt controller on ARM systems. Manages routing of hardware interrupts from devices to CPUs.

**IPI** (Inter-Processor Interrupt): An interrupt sent by one CPU core to another. Used for TLB shootdowns, wakeups, and kernel panic broadcasts.

**Inode**: A data structure representing a file's metadata and data, independent of any directory entry or file name. Multiple directory entries (hard links) can point to the same inode.

**MMIO** (Memory-Mapped I/O): A technique where hardware device registers are accessible at specific physical addresses. Reading/writing these addresses controls the device.

**MMU** (Memory Management Unit): Hardware that translates virtual addresses to physical addresses using page tables.

**Page**: The unit of virtual memory management, typically 4 KiB. The smallest unit the MMU can independently map.

**Page Fault**: An exception that occurs when a virtual address is accessed but the corresponding page table entry is absent, has wrong permissions, or requires special handling.

**Page Table**: A hierarchical data structure that maps virtual addresses to physical addresses. AArch64 uses a 4-level page table.

**PGID** (Process Group ID): An identifier for a group of related processes (typically a shell pipeline). Signals can be sent to entire process groups.

**PID** (Process ID): A unique identifier for a process. In Moss (following Linux), this is the TGID (Thread Group ID).

**Preemption**: Forcibly removing a running task from the CPU to run a different task. Triggered by timer interrupts or by a higher-priority task becoming runnable.

**procfs**: A virtual filesystem mounted at `/proc` that exposes kernel and process information through the file interface.

**SIGSEGV**: The signal delivered to a process when it accesses an invalid memory address. Default action: terminate with a core dump.

**Slab Allocator**: A kernel memory allocator that maintains caches of pre-allocated objects of fixed sizes, reducing fragmentation and allocation overhead.

**SMP** (Symmetric Multi-Processing): A system with multiple CPUs that share memory and can each run kernel code.

**Syscall** (System Call): A mechanism by which user-space code requests a service from the kernel. On AArch64, executed via the `svc #0` instruction.

**TGID** (Thread Group ID): The PID of the first thread in a process. All threads in a process share the same TGID.

**TID** (Thread ID): A unique identifier for a specific thread.

**TLB** (Translation Lookaside Buffer): A hardware cache of recent virtual-to-physical address translations. Must be flushed when page tables change.

**TLB Shootdown**: The process of invalidating TLB entries on multiple CPUs after a page table change. Requires IPIs to notify all CPUs.

**TTBR0/TTBR1**: Translation Table Base Registers. TTBR0 points to the user-space page table (changes on context switch). TTBR1 points to the kernel page table (constant).

**UA** (User Address): A Moss type for virtual addresses in user space. Distinct from `VA` (kernel virtual address) and `PA` (physical address) for type safety.

**VFS** (Virtual Filesystem): The kernel layer that provides a uniform interface to all filesystems and devices.

**vDSO** (Virtual Dynamically Shared Object): A small shared library that the kernel maps into every process's address space, providing fast paths for some syscalls without kernel entry overhead.

**VMA** (Virtual Memory Area): A contiguous range of virtual addresses with uniform properties (permissions, backing store). Process address spaces are composed of VMAs.

**Volatile**: In Rust (and C), a qualifier that prevents compiler optimization of memory accesses. Essential for MMIO register access.

**Work Stealing**: A scheduling technique where idle CPUs take tasks from the run queues of busy CPUs to maintain load balance.

**Zombie Process**: A process that has exited but whose entry has not yet been reaped by its parent via `wait4()`. Holds the exit status until the parent reads it.
