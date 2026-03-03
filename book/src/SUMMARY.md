# Summary

[Preface](./preface.md)

---

# Part I: Foundations

- [Introduction](./ch01-introduction/README.md)
  - [What Is an Operating System?](./ch01-introduction/what-is-an-os.md)
  - [Why Rust?](./ch01-introduction/why-rust.md)
  - [Exploring the Source Tree](./ch01-introduction/source-tree.md)

- [The Boot Process](./ch02-boot/README.md)
  - [From Power-On to Kernel](./ch02-boot/power-on.md)
  - [Stage 1: Early Initialization](./ch02-boot/stage1.md)
  - [Stage 2: Kernel Proper](./ch02-boot/stage2.md)
  - [Booting Secondary CPUs](./ch02-boot/secondary-cpus.md)

---

# Part II: Memory Management

- [Physical Memory](./ch03-memory/README.md)
  - [The Memory Layout](./ch03-memory/layout.md)
  - [The Early Allocator (Smalloc)](./ch03-memory/smalloc.md)
  - [The Buddy Allocator](./ch03-memory/buddy.md)
  - [The Slab Allocator](./ch03-memory/slab.md)

- [Virtual Memory](./ch03-memory/virtual/README.md)
  - [AArch64 Paging](./ch03-memory/virtual/paging.md)
  - [Page Tables in Moss](./ch03-memory/virtual/page-tables.md)
  - [Address Types: VA, PA, UA](./ch03-memory/virtual/address-types.md)
  - [Page Fault Handling](./ch03-memory/virtual/page-faults.md)
  - [Copy-on-Write](./ch03-memory/virtual/cow.md)
  - [Kernel–User Data Transfers](./ch03-memory/virtual/copy-user.md)

---

# Part III: Processes and Scheduling

- [Processes and Threads](./ch04-processes/README.md)
  - [Tasks and Thread Groups](./ch04-processes/tasks.md)
  - [Task Lifecycle](./ch04-processes/lifecycle.md)
  - [Creating Processes: fork and exec](./ch04-processes/fork-exec.md)
  - [File Descriptor Tables](./ch04-processes/file-descriptors.md)
  - [Credentials](./ch04-processes/credentials.md)

- [Scheduling](./ch05-scheduling/README.md)
  - [Scheduling Concepts](./ch05-scheduling/concepts.md)
  - [The EEVDF Algorithm](./ch05-scheduling/eevdf.md)
  - [Per-CPU State](./ch05-scheduling/per-cpu.md)
  - [Work Stealing (SMP)](./ch05-scheduling/work-stealing.md)
  - [The Idle Task](./ch05-scheduling/idle.md)

---

# Part IV: Kernel–Hardware Interface

- [System Calls](./ch06-syscalls/README.md)
  - [The User–Kernel Boundary](./ch06-syscalls/boundary.md)
  - [Syscall Dispatch](./ch06-syscalls/dispatch.md)
  - [Async Syscalls](./ch06-syscalls/async.md)
  - [Syscall Reference](./ch06-syscalls/reference.md)

- [Interrupts and Exceptions](./ch07-interrupts/README.md)
  - [Exception Levels on AArch64](./ch07-interrupts/exception-levels.md)
  - [The Exception Vector Table](./ch07-interrupts/vector-table.md)
  - [Saving and Restoring State](./ch07-interrupts/exception-state.md)
  - [The Interrupt Controller (GIC)](./ch07-interrupts/gic.md)
  - [Inter-Processor Interrupts](./ch07-interrupts/ipi.md)

---

# Part V: I/O and Storage

- [The Virtual Filesystem](./ch08-vfs/README.md)
  - [VFS Design](./ch08-vfs/design.md)
  - [Inodes and Open Files](./ch08-vfs/inodes.md)
  - [Path Resolution](./ch08-vfs/path-resolution.md)
  - [Filesystem Drivers](./ch08-vfs/drivers.md)
  - [Special Filesystems](./ch08-vfs/special-fs.md)

- [Device Drivers](./ch09-drivers/README.md)
  - [Character Devices](./ch09-drivers/char-devices.md)
  - [The Timer Driver](./ch09-drivers/timer.md)
  - [Block Devices and Ramdisk](./ch09-drivers/block-devices.md)
  - [Device Tree](./ch09-drivers/device-tree.md)

---

# Part VI: Process Coordination

- [Signals](./ch10-signals/README.md)
  - [What Are Signals?](./ch10-signals/what-are-signals.md)
  - [Sending and Receiving Signals](./ch10-signals/delivery.md)
  - [Signal Handlers](./ch10-signals/handlers.md)
  - [Job Control](./ch10-signals/job-control.md)

---

# Part VII: Working with Moss

- [Building and Testing](./ch11-building/README.md)
  - [Prerequisites and Toolchain](./ch11-building/prerequisites.md)
  - [Build Commands](./ch11-building/build-commands.md)
  - [Running in QEMU](./ch11-building/qemu.md)
  - [Unit Tests](./ch11-building/unit-tests.md)
  - [Kernel Tests](./ch11-building/kernel-tests.md)
  - [Userspace Tests](./ch11-building/userspace-tests.md)

---

[Appendix A: Syscall Table](./appendix-syscalls.md)
[Appendix B: Memory Map Reference](./appendix-memory-map.md)
[Appendix C: Glossary](./appendix-glossary.md)
[Appendix D: Further Reading](./appendix-reading.md)
