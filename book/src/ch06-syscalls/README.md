# Chapter 6: System Calls

**System calls** (syscalls) are the interface between user programs and the kernel. A user program cannot directly access hardware or other processes' memory. Instead, it asks the kernel to perform privileged operations on its behalf via the syscall interface.

Moss implements ~105 syscalls compatible with the Linux AArch64 ABI. User programs compiled for Linux can run on Moss without modification, because they call the same syscall numbers with the same argument conventions.

## Learning Objectives

By the end of this chapter you should be able to:

- Explain the mechanism by which a user program invokes the kernel
- Trace the path from `svc #0` to the syscall handler function
- Describe how Moss's async model applies to syscalls
- List the major categories of syscalls and what they do

## Contents

- [The User–Kernel Boundary](./boundary.md)
- [Syscall Dispatch](./dispatch.md)
- [Async Syscalls](./async.md)
- [Syscall Reference](./reference.md)
