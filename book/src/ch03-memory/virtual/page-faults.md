# Page Fault Handling

A **page fault** occurs when a virtual address is accessed but the corresponding page table entry is either absent, has insufficient permissions, or requires special handling. Page faults are the mechanism through which the kernel implements lazy allocation, demand paging, copy-on-write, and memory-mapped files.

## Types of Page Faults

AArch64 reports the fault reason in the **Fault Status Code** field of the exception syndrome register (`ESR_EL1`):

| Fault Type | Meaning |
|---|---|
| Translation fault | No valid page table entry at any level |
| Permission fault | Entry exists but access violates permissions (e.g., write to read-only) |
| Alignment fault | Misaligned access to device memory |
| Access flag fault | Page not yet accessed (used for LRU tracking) |

## The Fault Handler

When a page fault occurs, the CPU:
1. Saves the faulting virtual address in `FAR_EL1` (Fault Address Register)
2. Saves fault details in `ESR_EL1` (Exception Syndrome Register)
3. Jumps to the exception vector table (see [Chapter 7](../../ch07-interrupts/README.md))

Moss's fault handler is in `src/memory/fault.rs`. Its job is to determine what to do:

```rust
pub async fn handle_demand_fault(
    faulting_va: VA,
    access: AccessType,      // Read, Write, or Execute
) -> FaultResolution {
    let task = current_task();
    let vmas = task.address_space().vmas();

    // Find the VMA containing the faulting address
    let vma = match vmas.find(faulting_va) {
        Some(vma) => vma,
        None => return FaultResolution::Denied,  // SIGSEGV
    };

    // Check permissions
    if access == Write && !vma.is_writable() {
        // Could be a CoW fault — see copy-on-write section
        if vma.is_cow_candidate() {
            return handle_cow_fault(faulting_va, vma).await;
        }
        return FaultResolution::Denied;  // SIGSEGV
    }

    // Allocate and map the page
    match vma.backing() {
        Backing::Anonymous => {
            // Anonymous mapping — just allocate a zeroed page
            let frame = frame_alloc().expect("OOM");
            zero_page(frame);
            vma.map_page(faulting_va, frame, vma.prot());
            FaultResolution::Resolved
        }
        Backing::File(inode, offset) => {
            // File-backed mapping — load from disk
            // This may need to sleep waiting for I/O!
            let frame = read_page_from_file(inode, offset).await?;
            vma.map_page(faulting_va, frame, vma.prot());
            FaultResolution::Resolved
        }
    }
}
```

## Fault Resolution Outcomes

```rust
pub enum FaultResolution {
    Resolved,                // Page is now mapped, retry the instruction
    Denied,                  // Access violation — deliver SIGSEGV
    Deferred(Pin<Box<dyn Future<Output=()>>>),  // Async I/O needed
}
```

- **Resolved**: The page is now mapped. The CPU will retry the faulting instruction, which will now succeed.
- **Denied**: The access was illegal (no VMA, or VMA doesn't permit this access type). The kernel delivers a `SIGSEGV` signal to the process.
- **Deferred**: The page needs to be loaded from disk. The task suspends (releases the CPU) until the I/O completes, then retries.

## Demand Paging

Demand paging means pages are not loaded into memory until they are actually accessed. When a program is `exec`'d:

1. The kernel parses the ELF binary and creates VMAs for each segment (`.text`, `.data`, `.bss`)
2. No pages are allocated yet — all page table entries are invalid
3. When the program starts running and accesses its first page, a translation fault fires
4. The fault handler loads the page from the ELF file and maps it
5. The program continues, experiencing faults only for pages it actually touches

This makes `exec` extremely fast (no need to load the entire binary upfront) and conserves memory (pages only loaded when needed).

## Stack Growth

The user stack is a special VMA that grows downward on demand. When a process accesses a page just below its current stack top:

```rust
// In fault handler, if the fault is just below the stack VMA:
if faulting_va.is_just_below_stack(task) {
    // Extend the stack VMA by one page
    task.extend_stack(faulting_va);
    // Allocate and map the new page
    // ...
    return FaultResolution::Resolved;
}
```

This allows the stack to grow transparently without the program needing to explicitly allocate it.

## Kernel Fault Handling

Page faults can also occur in kernel mode — for example, when `copy_from_user` attempts to read from an invalid user address. These kernel faults are handled specially:

```rust
// copy_from_user sets up a "fault recovery" point
// If the kernel faults here, it returns EFAULT instead of panicking
let result = with_user_access(|| {
    ptr::copy_nonoverlapping(user_ptr, kernel_buf, len)
});

match result {
    Ok(()) => { /* success */ }
    Err(FaultError) => return Err(EFAULT),
}
```

Without this mechanism, a kernel page fault would be an unrecoverable error (kernel panic).

## Exercises

1. What happens when a process calls `malloc(1_000_000_000)`? Does the kernel immediately need 1 GB of RAM?

2. Trace the complete sequence of events from a program accessing an unmapped address to the page being loaded from disk and the program resuming.

3. What is the difference between a segmentation fault (SIGSEGV) and a bus error (SIGBUS)? When does each occur?
