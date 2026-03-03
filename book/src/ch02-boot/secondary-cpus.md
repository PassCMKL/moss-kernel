# Booting Secondary CPUs

Modern systems have multiple CPU cores. The CPU that runs the boot sequence is called the **primary CPU** (or BSP — Bootstrap Processor). The others are **secondary CPUs** (or APs — Application Processors). They start in a halted state and must be explicitly woken up.

## The SMP Boot Sequence

After `kmain()` is running on the primary CPU, Moss wakes secondary CPUs. Each secondary CPU goes through a simplified initialization called `arch_init_secondary()`.

### What the Primary CPU Does

The primary CPU:
1. Allocates a stack and per-CPU data structure for each secondary CPU
2. Writes the secondary CPU's entry point address into a memory location the secondary CPU will read
3. Sends a CPU-wakeup signal (in QEMU/ARM, this is done via a memory-mapped "release" register or the PSCI firmware interface)

### What Secondary CPUs Do

When a secondary CPU wakes up, it jumps to `arch_init_secondary()`:

```
Secondary CPU wakes at physical entry point
  ├─ Disable ID map (same as primary's Stage 2)
  ├─ Load TTBR1 (same kernel address space as primary)
  ├─ Set up own per-CPU data (own stack, own scheduler state)
  ├─ Install exception vector table
  ├─ Enable interrupts
  ├─ Arm the per-CPU system timer
  └─ Call sched_start() — join the scheduler pool
```

Notice that secondary CPUs skip most of Stage 2. They don't re-initialize the frame allocator (already done by the primary), don't re-probe devices, and don't launch `init`. They just join the existing kernel infrastructure.

## Per-CPU Isolation

A key design principle is that certain data structures are **per-CPU** — each CPU has its own private copy. This eliminates lock contention for frequently accessed data:

| Per-CPU resource | Why it's per-CPU |
|---|---|
| Kernel stack | Stack pointer must be unique per CPU |
| Scheduler run queue | Avoids global lock on task selection |
| Slab allocator cache | Hot-path allocation without contention |
| Timer state | Each CPU has its own hardware timer |

Data that must be shared between CPUs (e.g., the process table, the filesystem cache) is protected by locks or atomic operations.

## Inter-Processor Interrupts (IPIs)

Once secondary CPUs are running, the primary needs a way to communicate with them — for example, to tell CPU 2 to flush its TLB after a page table change.

Moss uses **Inter-Processor Interrupts (IPIs)** for this. The GIC (interrupt controller) supports sending a software-generated interrupt to specific CPUs:

```rust
// Tell all CPUs to flush their TLBs
for cpu in all_cpus() {
    send_ipi(cpu, IpiMessage::FlushTlb);
}
```

On the receiving CPU, the IPI fires an interrupt, the handler reads the message, and executes the requested action.

## Work Stealing

When a new task is created, it needs to be assigned to a CPU. Moss places new tasks on the least-loaded CPU using an atomic "CPU info" word that tracks the current task count per CPU.

If one CPU falls far behind (say, it gets stuck on a slow I/O operation), the scheduler can steal tasks from its run queue. This **work stealing** ensures all CPUs stay busy and minimizes task latency.

## Exercises

1. Why do secondary CPUs need their own per-CPU kernel stacks? What would happen if two CPUs shared a kernel stack?

2. What is a TLB shootdown? Why must all CPUs flush their TLBs when a page table entry is changed?

3. Design a simple work-stealing scheduler: when should a CPU attempt to steal work, and from which CPU should it steal?
