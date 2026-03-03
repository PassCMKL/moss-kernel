# Kernel–User Data Transfers

System calls frequently need to move data between kernel memory and user memory. This sounds simple — it's just a `memcpy`. But it requires careful handling because:

1. User addresses might be invalid (not mapped, wrong permissions)
2. User memory might be concurrently modified by another thread
3. Copying must not leak kernel data to user space or corrupt kernel memory from user-supplied pointers

## The Functions

Moss provides three primitives for crossing the kernel–user boundary:

```rust
/// Copy data FROM user space INTO a kernel buffer.
/// May sleep if pages need to be demand-faulted in.
pub async fn copy_from_user(dst: &mut [u8], src: UA) -> Result<()>;

/// Copy data FROM kernel memory TO user space.
/// May sleep if pages need to be demand-faulted in.
pub async fn copy_to_user(dst: UA, src: &[u8]) -> Result<()>;

/// Non-blocking variant of copy_from_user.
/// Returns Err if any page is not immediately available.
/// Safe to call while holding a spinlock.
pub fn try_copy_from_user(dst: &mut [u8], src: UA) -> Result<()>;
```

The key difference between `copy_from_user` and `try_copy_from_user` is that the former can trigger demand paging (sleeping until pages load), while the latter cannot sleep — important when called from a context where sleeping would cause a deadlock.

## Why Not Just Dereference the Pointer?

It's tempting to simply treat the user pointer as a kernel pointer:

```rust
// WRONG — never do this
let data = unsafe { *(user_ptr as *const u64) };
```

This is dangerous for several reasons:

1. **The address might not be mapped**: The kernel would take a page fault with no recovery mechanism.
2. **The address might be in kernel space**: A malicious user passing `0xffff_8000_1234` could read kernel memory.
3. **TOCTOU races**: The user might remap the page between when you check it and when you read it.
4. **Architecture requirements**: Some architectures require special instructions to access user memory from kernel mode (e.g., `stac`/`clac` on x86).

## Implementation

Internally, `copy_from_user` sets up a **fault recovery point** before attempting the copy:

```rust
pub async fn copy_from_user(dst: &mut [u8], src: UA) -> Result<()> {
    // First validate the address range is in user space
    if !src.is_valid_user_range(dst.len()) {
        return Err(EFAULT);
    }

    // Attempt the copy; if a page fault occurs, either:
    // - Demand fault the page in (async), then retry
    // - Return EFAULT if the address is truly invalid
    unsafe {
        arch::copy_from_user_safe(dst.as_mut_ptr(), src.0, dst.len()).await
    }
}
```

The architecture-specific implementation (`arch::copy_from_user_safe`) uses a special assembly sequence that the fault handler knows about. If a fault occurs during the copy:
- If the page can be demand-faulted in: do so and retry
- If not: the fault handler looks up the recovery table and jumps to a pre-registered error path that returns `EFAULT`

## The `UserCopyable` Trait

For structured data (rather than raw bytes), Moss uses the `UserCopyable` trait:

```rust
/// Safety: implementing this trait asserts that T has no padding bytes
/// and is safe to copy to/from user space byte-by-byte.
pub unsafe trait UserCopyable: Copy {}

// Example: copying a struct from a syscall argument
pub async fn read_user_struct<T: UserCopyable>(addr: UA) -> Result<T> {
    let mut value = MaybeUninit::<T>::uninit();
    copy_from_user(value.as_bytes_mut(), addr).await?;
    Ok(unsafe { value.assume_init() })
}
```

This trait is `unsafe` to implement because you are asserting that the type has no padding bytes that could contain uninitialized memory.

## String Copying

Null-terminated strings from user space require extra care:

```rust
pub async fn copy_string_from_user(addr: UA, max_len: usize) -> Result<String> {
    let mut buf = Vec::with_capacity(256);
    let mut offset = 0;

    loop {
        let byte = read_user_byte(UA(addr.0 + offset)).await?;
        if byte == 0 { break; }
        if offset >= max_len { return Err(ENAMETOOLONG); }
        buf.push(byte);
        offset += 1;
    }

    String::from_utf8(buf).map_err(|_| EINVAL)
}
```

This is used for `execve` (reading program name and arguments) and `open` (reading file paths).

## Security Considerations

- **Never trust user pointers** — always validate range and alignment
- **Copy before checking** — read user data once into a kernel buffer, then validate the copy. Checking user memory in-place can race with concurrent user modifications.
- **Limit string lengths** — protect against denial-of-service via infinite-length strings

## Exercises

1. Why does copying user data to a kernel buffer (rather than accessing it in-place) help prevent TOCTOU attacks?

2. What happens on an architecture that does not allow kernel code to access user memory directly? How would `copy_from_user` need to be implemented?

3. A system call receives a user pointer to an array of 1000 pointers, each pointing to a string. What is the correct strategy for safely reading all 1000 strings?
