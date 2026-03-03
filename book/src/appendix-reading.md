# Appendix D: Further Reading

This appendix lists resources for going deeper on the topics covered in this book.

## Operating Systems Textbooks

### Classic Texts

**"Operating Systems: Three Easy Pieces"** (Remzi and Andrea Arpaci-Dusseau)
- Free online: https://pages.cs.wisc.edu/~remzi/OSTEP/
- An excellent, accessible introduction to OS concepts
- Covers virtualization, concurrency, and persistence in depth
- Great companion to this book

**"Modern Operating Systems"** (Andrew Tanenbaum)
- The classic OS textbook, now in its 4th edition
- Covers microkernel design in depth (Tanenbaum designed MINIX)
- Thorough treatment of distributed systems

**"Operating System Concepts"** (Silberschatz, Galvin, Gagne) — the "Dinosaur Book"
- Comprehensive coverage of all major OS topics
- Used in many university courses

### Advanced Topics

**"Understanding the Linux Kernel"** (Bovet and Cesati)
- Deep dive into Linux internals
- Excellent companion for understanding how Moss's Linux compatibility works

**"Linux Device Drivers"** (Corbet, Rubini, Kroah-Hartman)
- Free online: https://lwn.net/Kernel/LDD3/
- How to write Linux kernel modules and drivers

## Scheduling

**"Earliest Eligible Virtual Deadline First"** (Stoica et al., 1995)
- The original EEVDF paper: https://citeseerx.ist.psu.edu/doc/10.1.1.51.3440

**"Completely Fair Scheduler" — Linux CFS**
- LWN article: https://lwn.net/Articles/230574/

**"EEVDF Scheduler for Linux"**
- LWN article on EEVDF adoption in Linux 6.6: https://lwn.net/Articles/925371/

## Memory Management

**"What Every Programmer Should Know About Memory"** (Ulrich Drepper)
- Free PDF: https://people.freebsd.org/~lstewart/articles/cpumemory.pdf
- Deep dive into caches, NUMA, and memory access patterns

**"The Art of Writing Efficient Programs"** (Fedor Pikus)
- Chapter on memory-friendly data structures

## Rust for Systems Programming

**"The Rust Programming Language"** (Klabnik and Nichols)
- https://doc.rust-lang.org/book/
- The official Rust book, free online

**"The Rustonomicon"**
- https://doc.rust-lang.org/nomicon/
- The dark arts of unsafe Rust — essential for kernel programming

**"Writing an OS in Rust"** (Philipp Oppermann)
- https://os.phil-opp.com/
- A tutorial series on building an x86_64 kernel in Rust
- Excellent companion to Moss

**"Rust for Linux" Project**
- https://rust-for-linux.com/
- The initiative to add Rust support to the Linux kernel
- Discusses practical challenges of Rust in kernel contexts

## AArch64 Architecture

**"Arm Architecture Reference Manual for A-profile architecture"**
- https://developer.arm.com/documentation/ddi0487/
- The authoritative reference for AArch64
- Free PDF from ARM (requires registration)

**"Learn the Architecture: AArch64 Instruction Set Architecture"**
- https://developer.arm.com/documentation/102374/
- A more approachable introduction than the full reference manual

**"Learn the Architecture: Memory Management"**
- https://developer.arm.com/documentation/101811/
- AArch64 page tables, TLB, memory attributes

## QEMU

**QEMU Documentation**
- https://www.qemu.org/docs/master/
- Covers the `virt` machine type and device model

## Research Papers

**"seL4: Formal Verification of an OS Kernel"** (Klein et al., SOSP 2009)
- How to formally verify a microkernel
- Discusses what properties matter for a correct kernel

**"The Design and Implementation of the MINIX 3 Operating System"** (Herder et al.)
- The modern MINIX microkernel approach

**"Unikernels: Library Operating Systems for the Cloud"** (Madhavapeddy et al., ASPLOS 2013)
- An alternative to traditional OS design

## Online Resources

**LWN.net** — https://lwn.net
- Weekly articles on Linux kernel development
- Deep technical coverage of new features and kernel architecture

**OSDev Wiki** — https://wiki.osdev.org
- Community resource for OS developers
- Detailed articles on specific hardware and techniques

**The Linux Kernel documentation** — https://www.kernel.org/doc/html/latest/
- Official documentation for Linux kernel subsystems

**Rust OS Development community** — https://github.com/rust-osdev
- A collection of OS development crates for Rust

## Academic Courses

Many universities offer OS courses with freely available materials:

- **MIT 6.S081 / 6.828**: Uses xv6, a simple teaching OS
  - https://pdos.csail.mit.edu/6.828/
- **Stanford CS140e**: Builds an OS for Raspberry Pi in Rust
  - Historical materials available
- **CMU 15-410**: Operating System Design and Implementation
  - Legendary course with detailed notes online
