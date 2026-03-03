# Chapter 5: Scheduling

The **scheduler** decides which task runs next on each CPU. When there are more runnable tasks than CPUs (the common case), the scheduler must choose fairly, efficiently, and with low latency.

Scheduling is a deeply studied problem with many algorithms: round-robin, priority queues, completely fair scheduling, earliest deadline first, and more. Moss implements **EEVDF** (Earliest Eligible Virtual Deadline First) — the same algorithm adopted by Linux in version 6.6.

## Learning Objectives

By the end of this chapter you should be able to:

- Explain the fundamental trade-offs in scheduling (throughput vs. latency vs. fairness)
- Describe how a virtual clock enables proportional-share scheduling
- Trace how EEVDF selects the next task to run
- Understand how SMP scheduling works with work stealing

## Contents

- [Scheduling Concepts](./concepts.md)
- [The EEVDF Algorithm](./eevdf.md)
- [Per-CPU State](./per-cpu.md)
- [Work Stealing (SMP)](./work-stealing.md)
- [The Idle Task](./idle.md)
