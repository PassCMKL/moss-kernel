# Chapter 4: Processes and Threads

A **process** is a running program — a program image loaded into memory, combined with a set of resources (open files, network connections, allocated memory) and one or more threads of execution. **Threads** are the units of CPU scheduling: multiple threads within the same process share memory and resources but run concurrently.

In Unix, these concepts are unified: a process is just a thread group where all threads share the same address space and file descriptor table. A single-threaded program is a thread group with one thread.

## Learning Objectives

By the end of this chapter you should be able to:

- Describe the difference between a task, a thread group, and a process
- Trace the lifecycle of a task from creation to destruction
- Explain how `fork` and `exec` create new processes
- Describe how file descriptors are managed per-process

## Contents

- [Tasks and Thread Groups](./tasks.md)
- [Task Lifecycle](./lifecycle.md)
- [Creating Processes: fork and exec](./fork-exec.md)
- [File Descriptor Tables](./file-descriptors.md)
- [Credentials](./credentials.md)
