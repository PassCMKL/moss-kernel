# The Timer Driver

The **system timer** drives the scheduler. It fires a periodic interrupt (the "tick") that gives the scheduler a chance to preempt the current task and switch to a higher-priority one.

## AArch64 Architectural Timer

AArch64 defines a set of **architectural timers** that are standardized across all ARM implementations. Moss uses the **EL1 physical timer** (also called the kernel timer).

The timer has three key registers:

| Register | Description |
|---|---|
| `CNTPCT_EL0` | Counter: current value (64-bit, monotonically increasing) |
| `CNTFRQ_EL0` | Frequency: how many counter ticks per second |
| `CNTP_CVAL_EL0` | Compare value: fire interrupt when `CNTPCT_EL0 >= CVAL` |
| `CNTP_CTL_EL0` | Control: enable/disable the timer, check pending |

The counter increments at a fixed frequency (typically 1–50 MHz on ARM SoCs). To schedule an interrupt in `N` nanoseconds:

```rust
pub fn arm_timer_ns(ns: u64) {
    let freq = read_sysreg!(cntfrq_el0);
    let ticks = (ns * freq) / 1_000_000_000;
    let now = read_sysreg!(cntpct_el0);

    // Set compare value to current time + delay
    write_sysreg!(cntp_cval_el0, now + ticks);

    // Enable the timer
    write_sysreg!(cntp_ctl_el0, 1);
}
```

## The Timer Is Per-CPU

Each CPU has its own instance of the architectural timer. This is important for the scheduler:
- CPU 0's timer fires → CPU 0's scheduler gets to preempt its current task
- CPU 1's timer fires → CPU 1's scheduler gets to preempt its current task

No coordination between CPUs is needed for the basic scheduling tick.

## The Timer Interrupt Handler

When the timer fires, a PPI (Private Peripheral Interrupt) is delivered to the owning CPU. The interrupt handler:

```rust
pub struct ArchTimerDriver {
    tick_interval_ns: u64,
}

impl InterruptHandler for ArchTimerDriver {
    fn handle(&self, _irq: InterruptId) {
        // Acknowledge by re-arming the timer for the next tick
        arm_timer_ns(self.tick_interval_ns);

        // Tell the scheduler a tick occurred
        let elapsed = self.tick_interval_ns;
        current_cpu().sched.tick(elapsed);
    }
}
```

The default tick interval in Moss is **4 milliseconds** (250 Hz), a typical value for a desktop/server kernel. Lower values (e.g., 1 ms / 1000 Hz) give better interactive latency but more overhead. Higher values (e.g., 10 ms / 100 Hz) reduce overhead but worsen responsiveness.

## High-Resolution Timers

The architectural timer is also used for `nanosleep` and POSIX timers. Rather than setting a timer interrupt for every sleep, Moss compares sleep deadlines at each tick and fires the next earliest deadline.

For high-precision timing, Moss reads `CNTPCT_EL0` directly (no interrupt needed):

```rust
pub fn now_ns() -> u64 {
    let freq = read_sysreg!(cntfrq_el0);
    let count = read_sysreg!(cntpct_el0);
    (count * 1_000_000_000) / freq
}
```

This is how `clock_gettime(CLOCK_MONOTONIC)` is implemented in the vDSO — directly reading the counter register without a kernel entry.

## Timer Initialization

During Stage 2, each CPU initializes its timer:

```rust
pub fn init_cpu_timer(tick_interval_ns: u64) {
    // Create the timer driver and register its IRQ handler
    let driver = ArchTimerDriver::new(tick_interval_ns);
    let irq = arch_timer_irq();  // PPI for the EL1 physical timer

    interrupt_manager().register(irq, Box::new(driver));
    gic.enable(irq);

    // Arm the timer to fire its first tick
    arm_timer_ns(tick_interval_ns);
}
```

## Exercises

1. The architectural timer counter runs at a fixed frequency (e.g., 24 MHz). How many counter ticks correspond to 1 millisecond at this frequency?

2. What happens if a task takes longer than one tick to complete its syscall? Will the scheduler preempt it?

3. How does `CLOCK_MONOTONIC` differ from `CLOCK_REALTIME`? Why would a program prefer one over the other?
