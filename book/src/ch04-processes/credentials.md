# Credentials

Every process in a Unix system has a set of **credentials** that determine what it is allowed to do. Credentials consist of user IDs and group IDs, and the kernel checks them on every security-sensitive operation.

## Credential Types

```rust
pub struct Credentials {
    pub uid: Uid,    // Real User ID — who launched the process
    pub euid: Euid,  // Effective User ID — used for permission checks
    pub suid: Suid,  // Saved User ID — for setuid programs

    pub gid: Gid,    // Real Group ID
    pub egid: Egid,  // Effective Group ID
    pub sgid: Sgid,  // Saved Group ID

    pub groups: Vec<Gid>,  // Supplementary group list
}
```

The distinction between **real** and **effective** IDs allows programs to temporarily drop or gain privileges.

## Why Multiple IDs?

### The `setuid` Mechanism

Consider `/usr/bin/passwd` — the program that changes your password. It needs to write to `/etc/shadow`, which is owned by root. But a normal user should be able to run it.

Unix solves this with the **setuid bit** on the executable. When a setuid executable is run:
- Real UID = the user who launched it (e.g., UID 1000)
- Effective UID = the file owner (e.g., UID 0 = root)

The program can temporarily drop root privileges (set EUID = real UID), do unprivileged work, then raise them again (set EUID = saved UID) when it needs to write the password file.

```c
// In a setuid program (initial EUID = 0):
seteuid(getuid());    // Drop to real UID for safety
// ... do unprivileged work ...
seteuid(0);           // Restore root for privileged operation
write_password_file();
seteuid(getuid());    // Drop again
```

### The Saved User ID

The **saved UID** remembers the original EUID so a setuid program can regain privileges after temporarily dropping them. Without it, once you called `setuid(real_uid)`, you'd have no way to go back to root.

## Credentials in Moss

Credentials are stored in the `ThreadGroup` and shared among all threads:

```rust
pub struct ThreadGroup {
    pub creds: Arc<RwLock<Credentials>>,
    // ...
}
```

The `RwLock` allows multiple threads to read credentials simultaneously but requires exclusive access for modifications.

### Syscall Implementations

```rust
pub fn sys_getuid() -> Uid {
    current_task().thread_group.creds.read().uid
}

pub fn sys_setuid(uid: Uid) -> Result<()> {
    let mut creds = current_task().thread_group.creds.write();

    if creds.euid == 0 {
        // Root can set uid/euid/suid to anything
        creds.uid = uid;
        creds.euid = uid;
        creds.suid = uid;
    } else if uid == creds.uid || uid == creds.suid {
        // Non-root can only set to real or saved UID
        creds.euid = uid;
    } else {
        return Err(EPERM);
    }

    Ok(())
}
```

## Permission Checks

When a process tries to open a file, the kernel checks:

1. Is the process UID 0 (root)? If so, allow everything.
2. Is the file's owner UID equal to the process's EUID? If so, apply owner permissions (rwx------).
3. Is the file's group GID in the process's group list? If so, apply group permissions (---rwx---).
4. Otherwise, apply other permissions (------rwx).

```rust
pub fn check_permission(inode: &Inode, access: Access, creds: &Credentials) -> bool {
    if creds.euid == 0 {
        return true;  // Root can do anything
    }

    let mode = inode.mode();
    let shift = if inode.uid() == creds.euid {
        6  // Owner bits
    } else if creds.groups.contains(&inode.gid()) {
        3  // Group bits
    } else {
        0  // Other bits
    };

    (mode >> shift) & access.bits() == access.bits()
}
```

## The nobody User

Many system daemons run as the `nobody` user (UID 65534) — a user with no special privileges and no file ownership. This limits the damage from a compromised daemon.

Moss supports this pattern through its credentials system: a daemon can `setuid(65534)` after initialization to drop root privileges.

## Exercises

1. Why are there separate real and effective UIDs? What security property does this provide?

2. A setuid-root program has a buffer overflow vulnerability. Why is this more dangerous than a buffer overflow in a regular program?

3. What is `sudo`? How does it differ from `su`? Which uses setuid and which uses a setuid binary plus credential checks?
