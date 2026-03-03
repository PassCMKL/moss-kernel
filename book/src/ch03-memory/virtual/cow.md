# Copy-on-Write

Copy-on-write (CoW) is an optimization that allows multiple processes to share the same physical pages as long as none of them writes to those pages. A copy is only made when a process actually modifies the data.

## The Fork Problem

Without CoW, `fork()` would need to copy every page of the parent's address space to create the child. A process with 512 MiB of data would need to copy 512 MiB on every `fork()`. This is prohibitively slow.

With CoW, `fork()` is nearly instantaneous:
1. Create a new page table structure for the child
2. Copy all page table entries from the parent to the child
3. Mark **all writable pages** as **read-only** in both parent and child
4. Done — no actual page copying

When either process tries to write to a shared page, a **permission fault** fires. The fault handler then makes a real copy for the writing process:

```
Before write:          After parent writes:
Parent: [page A]  →    Parent: [page A copy]
Child:  [page A]  →    Child:  [page A original]
         ↑ shared             ↑ now separate
```

## CoW in Moss

Moss implements CoW using a reference count on physical pages:

```rust
pub struct PhysPage {
    ref_count: AtomicUsize,   // How many mappings point to this page
    // ... other metadata
}
```

When `fork()` is called:

```rust
fn fork_address_space(parent: &AddressSpace) -> AddressSpace {
    let child = AddressSpace::new();

    for vma in parent.vmas() {
        if vma.is_writable() {
            // Mark parent's pages read-only
            vma.set_permissions(Read);
            // Increment ref count for all pages in this VMA
            for page in vma.pages() {
                page.inc_ref_count();
            }
        }
        // Share the same physical pages in the child
        child.clone_vma_shared(vma);
    }

    child
}
```

When a write fault occurs on a CoW page:

```rust
fn handle_cow_fault(va: VA, vma: &Vma) -> FaultResolution {
    let old_page = vma.page_at(va);

    if old_page.ref_count() == 1 {
        // Only one user — no need to copy, just restore write permission
        vma.set_page_permissions(va, vma.original_prot());
    } else {
        // Multiple users — make a private copy
        let new_page = frame_alloc().expect("OOM");
        copy_page(old_page, new_page);
        old_page.dec_ref_count();

        // Install the new private page with write permission
        vma.replace_page(va, new_page, vma.original_prot());
    }

    FaultResolution::Resolved
}
```

## CoW in Practice: `exec` After `fork`

A common Unix pattern is `fork()` followed immediately by `exec()`:

```c
pid_t pid = fork();
if (pid == 0) {
    // Child: replace image with new program
    exec("/bin/ls", ...);
}
// Parent: continues
```

Without CoW, `fork` would copy all the parent's memory — wasteful, since `exec` throws it all away immediately. With CoW:
- `fork` is nearly free (just page table operations)
- `exec` discards the (still-unmodified) shared pages
- Zero actual copying occurs

This is why `fork+exec` is efficient in Unix systems.

## Lazy CoW: `mmap(MAP_PRIVATE)`

CoW is not just for `fork`. When a file is opened with `mmap(MAP_PRIVATE)`, Moss uses CoW semantics for the entire file mapping:

- Multiple processes mapping the same file share the same physical pages
- Any write by one process triggers a CoW fault, creating a private copy of just the modified page
- Other processes continue to see the original file content

This allows efficient implementation of process-private data segments in dynamically linked executables: the `.data` segment starts as a CoW copy of the on-disk ELF file.

## Exercises

1. Consider a parent process with 256 MiB of CoW-shared memory and a child that modifies 1 MiB of it. What is the total physical memory used?

2. What happens if a CoW fault occurs during a `write()` system call to a read-only `mmap`'d file? Should the CoW copy be made?

3. Linux uses "eager CoW" for page tables but "lazy CoW" for actual pages. What does this mean, and why is the distinction important for performance?
