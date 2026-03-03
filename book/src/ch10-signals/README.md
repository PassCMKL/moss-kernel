# Chapter 10: Signals

**Signals** are a simple inter-process communication mechanism in Unix. They allow one process (or the kernel itself) to notify another process about an event. Signals are asynchronous — they can arrive at any time, interrupting normal execution.

Common examples:
- `Ctrl+C` in a terminal sends `SIGINT` to the foreground process
- A process accessing an unmapped address receives `SIGSEGV` from the kernel
- A parent receives `SIGCHLD` when a child process exits
- `kill -9 PID` sends `SIGKILL` to forcibly terminate a process

## Learning Objectives

By the end of this chapter you should be able to:

- Explain the signal delivery mechanism
- Describe how user-space signal handlers work
- Explain signal masking and pending signals
- Understand job control signals and process groups

## Contents

- [What Are Signals?](./what-are-signals.md)
- [Sending and Receiving Signals](./delivery.md)
- [Signal Handlers](./handlers.md)
- [Job Control](./job-control.md)
