# Chapter 2: The Boot Process

Every computer starts in an undefined state. There is no operating system, no memory management, no stack, no standard library — just raw hardware and a BIOS or bootloader. The boot process is the kernel's journey from that empty state to a running system capable of serving user processes.

Understanding the boot process teaches you:

- How hardware transitions from reset to running code
- What the kernel must set up before any "normal" Rust code can run
- How virtual memory is established
- How the kernel's C-style setup code (written in assembly) hands off to Rust

## Learning Objectives

By the end of this chapter you should be able to:

- Describe the sequence of events from power-on to the first user process
- Explain why a two-stage initialization is necessary
- Understand what the MMU is and why enabling it is a critical step
- Trace execution from `start.s` through `kmain()`

## Contents

- [From Power-On to Kernel](./power-on.md)
- [Stage 1: Early Initialization](./stage1.md)
- [Stage 2: Kernel Proper](./stage2.md)
- [Booting Secondary CPUs](./secondary-cpus.md)
