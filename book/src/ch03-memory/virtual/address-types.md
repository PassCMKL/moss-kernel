# Address Types: VA, PA, UA

One of Rust's most useful features in a kernel context is the ability to define **distinct types for distinct concepts**. In Moss, three address types prevent a large class of bugs:

```rust
/// A kernel virtual address (in TTBR1 space, upper half)
pub struct VA(pub usize);

/// A physical address (hardware RAM address)
pub struct PA(pub usize);

/// A user virtual address (in TTBR0 space, lower half, untrusted)
pub struct UA(pub usize);
```

These are **newtype wrappers** — thin wrappers around `usize` that carry type information but have zero runtime cost.

## Why Separate Types?

Consider what happens without type separation, in C:

```c
// C: both are just void*
void* kernel_buf = kmalloc(PAGE_SIZE);
void* user_addr  = (void*)0x400000;  // from syscall argument

memcpy(user_addr, kernel_buf, 100);  // This is a kernel bug!
// We copied kernel data to a user address without validation
```

In Moss, the same operation requires explicit conversion:

```rust
fn copy_to_user(dst: UA, src: &[u8]) -> Result<()> {
    // UA is untrusted — this function validates the address
    // and handles page faults safely
    unsafe { arch::copy_to_user_raw(dst, src) }
}

// You cannot accidentally pass a VA where UA is expected:
let kernel_buf: VA = VA(0xffff_8000_1234);
let user_addr: UA = UA(0x0000_0000_4000);

copy_to_user(kernel_buf, data); // COMPILE ERROR: type mismatch
copy_to_user(user_addr, data);  // OK
```

## VA: Kernel Virtual Addresses

`VA` represents an address in kernel virtual memory (the upper half, `0xffff_...`). All kernel pointers are implicitly VAs, but making it explicit is useful for the logical map conversion:

```rust
impl VA {
    pub fn to_physical(self) -> PA {
        // Only valid for addresses in the logical map region!
        PA(self.0 - PHYSICAL_MAP_BASE)
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }
}
```

## PA: Physical Addresses

`PA` represents a raw hardware address — the address you'd see on the memory bus. The kernel cannot dereference a `PA` directly. To access a physical page, it must first convert it to a VA via the logical map:

```rust
impl PA {
    pub fn to_virtual(self) -> VA {
        VA(self.0 + PHYSICAL_MAP_BASE)
    }

    pub fn page_frame_number(self) -> usize {
        self.0 >> PAGE_SHIFT
    }
}
```

Attempting to cast a `PA` directly to a pointer and dereference it would be undefined behavior (since the kernel runs with the MMU on and the physical address is not mapped at its own address). The type system makes accidental physical-address dereferences less likely.

## UA: User Addresses

`UA` represents a virtual address in user space (the lower half, `0x0000_...`). User addresses are **untrusted** — the kernel received them from a system call argument, and the user program might:

1. Pass an invalid address (not mapped at all)
2. Pass a kernel address (trying to read kernel memory)
3. Pass an address that races with another thread (TOCTOU attacks)

The `UA` type signals to the programmer that this address must be treated with special care. Functions that accept `UA` are responsible for validating the address before use.

## Conversions

Explicit conversions between address types require unsafe code, making every such conversion a documented decision:

```rust
// Reading a user-provided buffer
pub fn sys_read(fd: i32, buf_ptr: UA, count: usize) -> Result<usize> {
    // Validate that [buf_ptr, buf_ptr+count) is a valid user mapping
    let user_slice = validate_user_slice(buf_ptr, count)?;

    let file = current_task().get_file(fd)?;
    file.read_to_user(user_slice).await
}

// This won't compile — UA is not a VA
let kernel_data: &[u8] = unsafe { &*(buf_ptr as VA as *const [u8]) };
//                                          ^^^ type error
```

## Compile-Time Address Space Safety

The benefits of typed addresses compound. Consider page table construction:

```rust
// Mapping a physical page at a virtual address
fn map_page(virt: VA, phys: PA, prot: Prot) { ... }

// Passing wrong types is caught at compile time:
map_page(phys, virt, ...);  // ERROR: expected VA, found PA
map_page(user_addr, phys, ...); // ERROR: expected VA, found UA
```

In a C kernel, all three of these would compile fine and produce incorrect behavior at runtime.

## Exercises

1. Without typed addresses, how could a kernel accidentally leak kernel data to user space? Give a specific code example.

2. Why is it important that converting `PA` to a dereferenceable pointer requires going through the logical map (`PA → VA → *T`) rather than casting directly?

3. What is a TOCTOU (Time-of-Check-to-Time-of-Use) attack in the context of user addresses? Give an example of how it could exploit the kernel.
