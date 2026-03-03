# The Interrupt Controller (GIC)

On AArch64 systems, hardware interrupts are managed by the **GIC** (Generic Interrupt Controller). The GIC is a separate hardware block that aggregates interrupt signals from all devices and delivers them to CPU cores in a controlled manner.

## Why an Interrupt Controller?

A modern SoC (System-on-Chip) might have hundreds of interrupt sources: timers, UARTs, storage controllers, network interfaces, USB controllers, and more. Without an interrupt controller:
- Each CPU would need hundreds of interrupt input pins
- Distributing interrupts to specific CPUs would be impossible
- Masking or prioritizing individual interrupts would be very difficult

The GIC solves all of these by acting as an intermediary: devices signal the GIC, and the GIC decides which CPU gets which interrupt at what priority.

## GIC Architecture

```
Hardware Devices                GIC                    CPUs
─────────────────          ──────────────          ──────────
UART (SPI #33)    ───IRQ──►│              │──IRQ──► CPU 0
Timer (PPI #14)   ───IRQ──►│  Distributor │──IRQ──► CPU 1
Network (SPI #42) ───IRQ──►│  (global)    │──IRQ──► CPU 2
USB (SPI #55)     ───IRQ──►│              │──IRQ──► CPU 3
                           └──────────────┘
                                  │
                           ┌──────▼──────┐
                           │  CPU I/F    │  (per CPU)
                           │ (prioritize,│
                           │  acknowledge│
                           └─────────────┘
```

The GIC has two main components:
1. **Distributor**: Global. Receives all interrupt signals, determines enabled/disabled state and priority.
2. **CPU Interface**: Per-CPU. Filters interrupts by priority and delivers the highest-priority interrupt to the CPU.

## Interrupt Types

| Type | Abbreviation | Source | Delivery |
|---|---|---|---|
| Software-Generated Interrupt | SGI | Software (`gicd_sgir` write) | Specific CPUs (used as IPI) |
| Private Peripheral Interrupt | PPI | Per-CPU device (timer, PMU) | Only to the "owner" CPU |
| Shared Peripheral Interrupt | SPI | External device (UART, network) | Any CPU |

SGIs use interrupt IDs 0–15, PPIs use 16–31, and SPIs use 32+.

## Interrupt Lifecycle

1. **Device asserts** an interrupt line
2. **Distributor** checks if the interrupt is enabled; if so, marks it as pending
3. **CPU Interface** selects the highest-priority pending interrupt targeting this CPU
4. **CPU** takes an IRQ exception, jumps to the vector table
5. **Kernel handler** reads `IAR` (Interrupt Acknowledge Register) to get the interrupt ID, which also **claims** the interrupt (prevents re-delivery)
6. **Kernel** dispatches to the appropriate device driver handler
7. **Device driver handler** clears the interrupt condition at the device
8. **Kernel** writes to `EOIR` (End Of Interrupt Register) to signal completion to the GIC

## Moss's GIC Driver

Moss supports both GICv2 and GICv3, the two most common versions. The driver is split into:

```
src/drivers/interrupts/
├── mod.rs             — InterruptController trait (hardware abstraction)
├── arm_gic_v2.rs     — GICv2 implementation
└── arm_gic_v3.rs     — GICv3 implementation
```

The `InterruptController` trait abstracts the differences:

```rust
pub trait InterruptController {
    /// Claim the next pending interrupt on this CPU
    fn acknowledge(&self) -> Option<InterruptId>;

    /// Signal completion of interrupt handling
    fn end_of_interrupt(&self, id: InterruptId);

    /// Enable a specific interrupt
    fn enable(&self, id: InterruptId);

    /// Disable a specific interrupt
    fn disable(&self, id: InterruptId);

    /// Set the priority of an interrupt (lower = higher priority)
    fn set_priority(&self, id: InterruptId, priority: u8);

    /// Configure which CPUs receive a given SPI
    fn set_affinity(&self, id: InterruptId, cpu_mask: u64);

    /// Send an IPI (SGI) to specific CPUs
    fn send_ipi(&self, target_cpus: CpuMask, ipi_id: u8);
}
```

## Interrupt Handling in Moss

The IRQ handler calls the GIC to discover which interrupt fired, then dispatches to the registered handler:

```rust
pub fn handle_irq() {
    let gic = get_interrupt_controller();

    // Acknowledge and get interrupt ID (atomically claims the interrupt)
    while let Some(id) = gic.acknowledge() {
        // Dispatch to registered handler
        if let Some(handler) = INTERRUPT_HANDLERS.get(id) {
            handler.handle(id);
        } else {
            warn!("Spurious interrupt: {:?}", id);
        }

        // Signal completion
        gic.end_of_interrupt(id);
    }
}
```

Device drivers register their handlers during initialization:

```rust
// In UART driver init:
interrupt_manager().register(uart_irq_id, Box::new(UartHandler::new(uart)));
gic.enable(uart_irq_id);
```

## Priority and Preemption

The GIC supports priority-based interrupt preemption. A higher-priority interrupt (lower priority number) can preempt the handling of a lower-priority interrupt. The CPU Interface has a **priority mask register** that filters out interrupts below a certain priority level.

Moss uses this for:
- Timer interrupts (moderate priority — needed for scheduling)
- IPIs (high priority — needed for TLB shootdowns)
- Device interrupts (lower priority — can be slightly delayed)

## Exercises

1. What is the difference between masking an interrupt at the GIC level versus masking it at the CPU level (the `DAIF` bits)?

2. Why does acknowledging an interrupt (reading `IAR`) need to happen before the handler runs, rather than after?

3. What would happen if a device driver forgot to clear the interrupt condition at the device but still wrote to `EOIR`? What would the GIC do?
