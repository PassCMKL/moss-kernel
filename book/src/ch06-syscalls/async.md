# Async Syscalls

One of Moss's most distinctive design choices is implementing system calls as `async fn`. This section explains what that means, why it matters, and how it compares to the traditional approach.

## The Problem with Blocking Syscalls

In a traditional kernel (like Linux), a syscall that needs to wait (for disk I/O, a mutex, etc.) blocks the calling thread entirely:

```
Thread A calls read():
  ┌─ save state
  ├─ find file
  ├─ request page from disk
  ├─ BLOCK: wait in wait queue (thread pinned here, uses a kernel stack)
  │   ... disk I/O in progress (possibly 10ms) ...
  ├─ wake up, retry
  └─ return to user space
```

During this time, Thread A occupies a kernel stack (typically 8–16 KiB) and cannot run anything else. On a heavily loaded system with thousands of threads all blocking on I/O, this can consume significant memory.

## The Async Solution

In Moss, syscall handlers are `async fn`. When they need to wait, they use `.await`, which:
1. Saves the current execution state as a **Future** (a heap-allocated state machine)
2. Returns the CPU to the scheduler
3. When the event occurs, the Future is scheduled to resume

```rust
// Moss's async read syscall
pub async fn sys_read(fd: i32, buf: UA, count: usize) -> i64 {
    let task = current_task();
    let file = match task.get_file(fd) {
        Ok(f) => f,
        Err(e) => return -e as i64,
    };

    // .await here: if data isn't ready, this task suspends.
    // The CPU can run other tasks while waiting.
    let result = file.read_to_user(buf, count).await;

    match result {
        Ok(n) => n as i64,
        Err(e) => -(e as i64),
    }
}
```

The key difference: while waiting for disk I/O, no kernel stack is held. The waiting state is stored in a much smaller `Future` object on the heap.

## The Interruptible Future

Syscalls often need to be interrupted by signals. If a process is sleeping in `read()` and receives `SIGINT`, the `read()` should return `-EINTR` immediately.

Moss provides an `.interruptible()` combinator:

```rust
pub async fn sys_nanosleep(duration: Duration) -> i64 {
    let sleep_future = timer::sleep(duration);

    // Wait for either the sleep to complete OR a signal to arrive
    match sleep_future.interruptible().await {
        Ok(()) => 0,        // Sleep completed normally
        Err(EINTR) => -EINTR as i64,  // Signal arrived
    }
}
```

The `.interruptible()` combinator races the future against the task's signal queue. Whichever completes first wins.

## Why Async Prevents Spinlock Deadlocks

Consider the following buggy code in a C kernel:

```c
spinlock_lock(&cache_lock);
// ...
read_page_from_disk();  // This blocks! We still hold cache_lock!
// If the disk completion tries to acquire cache_lock → DEADLOCK
spinlock_unlock(&cache_lock);
```

This is a class of bugs that is notoriously hard to prevent in C kernels — a lock accidentally held across a blocking call.

In Moss, the Rust compiler prevents this:

```rust
{
    let guard = cache_lock.lock(); // SpinLockGuard acquired

    // COMPILE ERROR: SpinLockGuard is not Send,
    // and .await requires the future to be Send.
    // You cannot hold a SpinLock across an .await point.
    read_page_from_disk().await;

    drop(guard);
}
```

The `SpinLockGuard` type in Moss deliberately implements `!Send` (cannot be sent across threads). Since async tasks may resume on different CPUs, holding a spinlock across `.await` is a compile error. The borrow checker eliminates an entire class of kernel deadlocks.

## The Kernel Work Executor

Async tasks in Moss are run by a simple single-threaded executor per CPU:

```rust
pub fn run_kernel_work() {
    while let Some(future) = kernel_work_queue().pop() {
        match future.poll() {
            Poll::Ready(()) => {
                // Work item complete
            }
            Poll::Pending => {
                // Task is waiting for an event
                // It registered a waker — it will be re-queued when ready
            }
        }
    }
}
```

The executor is called:
- After returning from user space (to drain any pending work)
- After an interrupt (to process any newly-ready work)

## Comparison with Linux

Linux handles sleeping syscalls using "wait queues" — lists of tasks waiting for a specific event. When the event occurs, the kernel iterates the wait queue and wakes each task. The task then re-checks its condition and either continues or blocks again.

This is functionally similar to Moss's waker mechanism but without the compiler-enforced safety properties. In Linux, nothing stops a developer from holding a spinlock across a `wait_event_interruptible()` call — the bug would only manifest at runtime.

## Exercises

1. What is the difference between a thread and an async task in terms of memory usage while waiting for I/O?

2. Look at Rust's `Future` trait. What does `Poll::Ready(T)` vs `Poll::Pending` mean? Who calls `poll()`, and what does it provide via the `Context` parameter?

3. Could you implement `.interruptible()` without async/await? What would the equivalent code look like in C?
