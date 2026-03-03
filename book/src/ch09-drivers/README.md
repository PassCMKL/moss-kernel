# Chapter 9: Device Drivers

Device drivers are the kernel's interface to hardware. They translate the generic kernel I/O operations (`read`, `write`, `ioctl`) into hardware-specific register sequences.

Moss includes drivers for:
- Serial consoles (UART)
- The architectural timer (scheduling clock)
- Interrupt controllers (GIC)
- RAM-backed block devices (ramdisk)

## Learning Objectives

By the end of this chapter you should be able to:

- Explain the role of device drivers in the kernel
- Describe how MMIO (Memory-Mapped I/O) allows software to control hardware
- Trace how a character is written to the UART console
- Understand how the system timer drives the scheduler

## Contents

- [Character Devices](./char-devices.md)
- [The Timer Driver](./timer.md)
- [Block Devices and Ramdisk](./block-devices.md)
- [Device Tree](./device-tree.md)
