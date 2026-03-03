# Inter-Processor Interrupts

**Inter-Processor Interrupts (IPIs)** are a mechanism for one CPU to send a message to another CPU. They are implemented using the GIC's Software-Generated Interrupt (SGI) mechanism.

## Why IPIs Are Needed

On a multi-CPU system, some operations must be performed on a specific CPU or on all CPUs simultaneously:

| Operation | Why it needs IPIs |
|---|---|
| **TLB shootdown** | After modifying a page table entry, all CPUs must flush their TLBs |
| **New task notification** | When a task is placed on a remote CPU's run queue, wake that CPU |
| **Kernel panic** | Halt all CPUs when one panics |
| **CPU hotplug** | Request a CPU to take itself offline |

Without IPIs, a CPU would have to constantly poll shared memory to detect these conditions — inefficient and high-latency.

## IPI Mechanism

On AArch64 with a GIC, IPIs use SGI interrupt IDs 0–15:

```rust
// Moss's IPI IDs
pub const IPI_RESCHEDULE: u8 = 0;  // "Check your run queue"
pub const IPI_FLUSH_TLB:  u8 = 1;  // "Flush your TLB"
pub const IPI_HALT:       u8 = 2;  // "Stop and halt (panic)"
pub const IPI_CALL_FUNC:  u8 = 3;  // "Run this function"
```

Sending an IPI:
```rust
pub fn send_ipi(target: CpuId, ipi_type: u8) {
    let gic = get_interrupt_controller();
    let target_mask = 1u64 << target.0;
    gic.send_ipi(target_mask, ipi_type);
}

pub fn broadcast_ipi(ipi_type: u8) {
    let gic = get_interrupt_controller();
    let all_cpus = !0u64;  // All CPUs
    gic.send_ipi(all_cpus, ipi_type);
}
```

The receiving CPU takes an IRQ exception. The IRQ handler checks if it's an IPI and dispatches accordingly:

```rust
fn handle_irq() {
    while let Some(id) = gic.acknowledge() {
        if id < 16 {
            // It's an IPI
            handle_ipi(id as u8);
        } else {
            // Regular device interrupt
            dispatch_device_irq(id);
        }
        gic.end_of_interrupt(id);
    }
}

fn handle_ipi(ipi_type: u8) {
    match ipi_type {
        IPI_RESCHEDULE => {
            // A new task was added — reschedule
            set_need_resched();
        }
        IPI_FLUSH_TLB => {
            arch::flush_tlb_all();
        }
        IPI_HALT => {
            // Kernel panic on another CPU — stop here too
            arch::halt();
        }
        _ => warn!("Unknown IPI: {}", ipi_type),
    }
}
```

## TLB Shootdown in Detail

TLB shootdowns are one of the most common uses of IPIs. When a page is unmapped or its permissions change, all CPUs that might have cached that translation must invalidate their TLBs.

The sequence:

```
CPU 0 (unmapping page):
  1. Remove page table entry
  2. Write memory barrier (dsb ishst)
  3. Send IPI_FLUSH_TLB to all other CPUs
  4. Wait for all CPUs to acknowledge
  5. Issue local TLB invalidation
  6. Continue

CPU 1, 2, 3 (receiving IPI):
  1. Receive IPI_FLUSH_TLB
  2. Issue TLB invalidation
  3. Acknowledge (set a per-CPU flag)
  4. Return from interrupt
```

The "wait for acknowledgment" in step 4 on CPU 0 ensures that no CPU will use the stale TLB entry after the page is freed/reallocated.

On AArch64, the TLB invalidation instruction `tlbi vmalle1is` invalidates all user-space TLB entries across the inner shareable domain (all CPUs sharing the same L3 cache), so Moss can sometimes avoid explicit IPIs for TLB maintenance.

## The Cost of IPIs

IPIs are not free. Each IPI causes:
- An interrupt exception on the target CPU
- An exception entry/exit overhead (~50-200 ns)
- Possible pipeline flush and cache effects

For operations like `munmap` that touch many pages, a single "flush everything" IPI is used rather than one IPI per page.

## Exercises

1. What would happen if a CPU's TLB were not flushed after a page table entry was changed to remove write permission? What attack could this enable?

2. Implement a simple "CPU call" IPI: CPU 0 wants to run a function on CPU 1. What data structure would you use to pass the function pointer and arguments?

3. If all 8 CPUs in a system are handling IPIs simultaneously and each sends an IPI back to all others, how many total IPI deliveries occur?
