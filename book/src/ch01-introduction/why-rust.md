# Why Rust?

Moss is written almost entirely in Rust (with a small amount of AArch64 assembly for the lowest-level boot and exception handling). This is not an accident — the choice of Rust is a deliberate architectural decision that directly improves the correctness and safety of the kernel.

## The Traditional Problem: C and Memory Safety

Almost every major production kernel — Linux, Windows, macOS, FreeBSD — is written in C. C is fast, portable, and has direct access to hardware. But it also gives programmers enormous freedom to shoot themselves in the foot:

- **Buffer overflows**: Writing past the end of an array silently corrupts adjacent memory.
- **Use-after-free**: Accessing memory after it has been freed produces undefined behavior.
- **Data races**: Two threads accessing the same data without synchronization leads to corrupt state.
- **Null pointer dereferences**: Accessing through a null pointer crashes (or worse, corrupts) the kernel.

Studies of CVEs (Common Vulnerabilities and Exposures) in the Linux kernel find that **~70% of security vulnerabilities** are caused by memory safety bugs. These bugs are not the result of careless programmers — they are the result of C's inability to express and enforce the invariants that safe code requires.

## What Rust Provides

Rust is a systems programming language designed to be as fast as C while making memory safety bugs impossible to compile (with some caveats for `unsafe` code). The key mechanisms are:

### Ownership and Borrowing

Every value in Rust has exactly one **owner**. When the owner goes out of scope, the value is freed. This eliminates use-after-free by construction:

```rust
let page = allocate_page();       // page is owned here
process_page(&page);              // borrow — page is still valid
// page is freed here automatically
drop(page);
// Using page here would be a *compile error*, not a runtime crash
```

### The Borrow Checker

Rust's borrow checker enforces that you cannot have both a mutable reference and any other reference to the same data at the same time. This eliminates data races at compile time:

```rust
let mut scheduler = Scheduler::new();
let task_ref = &scheduler.current_task;  // immutable borrow
scheduler.add_task(new_task);            // ERROR: cannot borrow mutably
                                         // while immutable borrow exists
```

In kernel terms, this means the compiler prevents many classes of concurrency bugs before the code even runs.

### No Null Pointers

Rust has no null pointers. Optional values are represented as `Option<T>`, which must be explicitly handled:

```rust
fn find_task(tid: Tid) -> Option<Arc<Task>> {
    // Returns Some(task) or None — never a null pointer
}

// The caller MUST handle both cases
match find_task(42) {
    Some(task) => schedule(task),
    None => return Err(ESRCH),
}
```

### `unsafe` Blocks and Audited Boundaries

Rust does not pretend that a kernel can avoid all unsafe operations. Interacting with hardware registers, setting up page tables, and switching CPU contexts inherently require operations the type system cannot verify. Rust handles this with `unsafe` blocks:

```rust
unsafe {
    // Write to a hardware register — the programmer vouches
    // that this is correct
    core::ptr::write_volatile(UART_DR as *mut u32, byte as u32);
}
```

The key benefit is that `unsafe` is explicit and auditable. In a C kernel, every line of code is implicitly unsafe. In a Rust kernel, you can grep for `unsafe` and review every location where safety invariants depend on programmer discipline rather than the type system.

## How Rust Shapes Moss's Design

### Async/Await for System Calls

One of the most innovative aspects of Moss is its use of Rust's `async/await` for system calls and kernel work. Traditional kernels block a thread when it needs to wait (e.g., for disk I/O). Moss instead models waiting as a `Future` — a value that will eventually produce a result.

```rust
// A simplified view of an async syscall
pub async fn sys_read(fd: i32, buf: UserBuffer) -> Result<usize> {
    let file = current_task().get_file(fd)?;
    // This .await point suspends the task if data isn't ready,
    // freeing the CPU to run other tasks
    let n = file.read(buf).await?;
    Ok(n)
}
```

The critical benefit: the Rust compiler prevents you from holding a spinlock across an `.await` point. If you try, you get a compile error. This eliminates an entire class of kernel deadlocks that plague C kernels.

### Type-Safe Address Spaces

Moss defines distinct types for virtual addresses, physical addresses, and user addresses:

```rust
pub struct VA(usize);   // Virtual Address (kernel)
pub struct PA(usize);   // Physical Address
pub struct UA(usize);   // User Address (untrusted)
```

These types are not interchangeable. Accidentally using a physical address where a virtual address is expected is a **compile error**, not a silent bug that corrupts memory.

### Trait-Based Hardware Abstraction

Moss's hardware abstraction layer is defined as Rust traits:

```rust
pub trait Arch {
    fn current_cpu() -> CpuId;
    fn enable_interrupts();
    fn disable_interrupts();
    fn flush_tlb_all();
    // ... more architecture operations
}
```

The AArch64 implementation provides concrete implementations. Adding support for a new architecture means implementing this trait — all the architecture-independent kernel code continues to work unchanged.

## The Cost of Rust

Rust is not without trade-offs:

- **Learning curve**: The borrow checker rejects programs that would be valid in C, and understanding why takes time.
- **Compile times**: Rust compiles slower than C, though this is improving.
- **Ecosystem**: Fewer OS-specific libraries exist for Rust than for C (though this is also improving rapidly).
- **`no_std` environment**: A bare-metal kernel cannot use Rust's standard library, requiring careful selection of `no_std`-compatible crates.

For Moss's purposes as an educational kernel, these costs are outweighed by the clarity and safety that Rust provides.

## Exercises

1. Find three CVEs in the Linux kernel from the past five years that were caused by memory safety bugs. What type of bug was each (use-after-free, buffer overflow, etc.)?

2. In Rust, what is the difference between `&T` and `&mut T`? What invariant does the borrow checker enforce about these two kinds of references?

3. Search the Moss source tree for `unsafe` blocks using `grep -rn "unsafe" src/`. List three uses and explain why each one cannot be made safe without `unsafe`.
