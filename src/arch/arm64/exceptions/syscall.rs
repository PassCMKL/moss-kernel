// ---------------------------------------------------------------------------
// Level 3 & 4 — syscall history tracking
// ---------------------------------------------------------------------------

use alloc::{
    boxed::Box,
    collections::VecDeque,
    format,
    string::String,
    vec::Vec,
};
use core::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use crate::sync::SpinLock;

/// Maximum number of bytes kept in the syscall ring buffer.
const SYSCALL_LOG_MAX_BYTES: usize = 8192;

/// Heap-backed ring buffer for formatted syscall history entries.
struct SyscallLog {
    entries: VecDeque<String>,
    total_bytes: usize,
}

impl SyscallLog {
    const fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            total_bytes: 0,
        }
    }

    fn push(&mut self, mut entry: String) {
        if entry.len() > SYSCALL_LOG_MAX_BYTES {
            entry = entry.split_off(entry.len() - SYSCALL_LOG_MAX_BYTES);
        }

        while self.total_bytes + entry.len() > SYSCALL_LOG_MAX_BYTES {
            let Some(oldest) = self.entries.pop_front() else {
                break;
            };
            self.total_bytes = self.total_bytes.saturating_sub(oldest.len());
        }

        self.total_bytes += entry.len();
        self.entries.push_back(entry);
    }

    fn snapshot_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.total_bytes);
        for entry in &self.entries {
            out.extend_from_slice(entry.as_bytes());
        }
        out
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.total_bytes = 0;
    }
}

/// Ring buffer that records the last `SYSCALL_RING_MAX` syscalls.
///
/// Backed by `VecDeque`, which allocates from the kernel heap — itself
/// serviced by the SLAB allocator.  When full, the oldest entry is evicted
/// from the front before pushing the new one onto the back.
static SYSCALL_LOG: SpinLock<SyscallLog> = SpinLock::new(SyscallLog::new());

/// Stores the number of the most recently *completed* syscall.
///
/// Updated at the end of every syscall dispatch (including the early-return
/// paths for `exit` and `sigreturn`).  The `syslog` handler (0x74) reads
/// this to report the previous syscall.
static PREV_SYSCALL: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------

use crate::{
    arch::{Arch, ArchImpl},
    clock::{
        gettime::sys_clock_gettime,
        settime::sys_clock_settime,
        timeofday::{sys_gettimeofday, sys_settimeofday},
    },
    fs::{
        dir::sys_getdents64,
        pipe::sys_pipe2,
        syscalls::{
            at::{
                access::{sys_faccessat, sys_faccessat2},
                chmod::sys_fchmodat,
                chown::sys_fchownat,
                handle::sys_name_to_handle_at,
                link::sys_linkat,
                mkdir::sys_mkdirat,
                open::sys_openat,
                readlink::sys_readlinkat,
                rename::{sys_renameat, sys_renameat2},
                stat::sys_newfstatat,
                statx::sys_statx,
                symlink::sys_symlinkat,
                unlink::sys_unlinkat,
                utime::sys_utimensat,
            },
            chdir::{sys_chdir, sys_chroot, sys_fchdir, sys_getcwd},
            chmod::sys_fchmod,
            chown::sys_fchown,
            close::{sys_close, sys_close_range},
            copy_file_range::sys_copy_file_range,
            getxattr::{sys_fgetxattr, sys_getxattr, sys_lgetxattr},
            ioctl::sys_ioctl,
            iov::{sys_preadv, sys_preadv2, sys_pwritev, sys_pwritev2, sys_readv, sys_writev},
            listxattr::{sys_flistxattr, sys_listxattr, sys_llistxattr},
            removexattr::{sys_fremovexattr, sys_lremovexattr, sys_removexattr},
            rw::{sys_pread64, sys_pwrite64, sys_read, sys_write},
            seek::sys_lseek,
            setxattr::{sys_fsetxattr, sys_lsetxattr, sys_setxattr},
            splice::sys_sendfile,
            stat::sys_fstat,
            statfs::{sys_fstatfs, sys_statfs},
            sync::{sys_fdatasync, sys_fsync, sys_sync, sys_syncfs},
            trunc::{sys_ftruncate, sys_truncate},
        },
    },
    kernel::{
        hostname::sys_sethostname, power::sys_reboot, rand::sys_getrandom, sysinfo::sys_sysinfo,
        uname::sys_uname,
    },
    memory::{
        brk::sys_brk,
        mincore::sys_mincore,
        mmap::{sys_mmap, sys_mprotect, sys_munmap},
        process_vm::sys_process_vm_readv,
        uaccess::copy_to_user_slice,
    },
    process::{
        TaskState,
        caps::{sys_capget, sys_capset},
        clone::sys_clone,
        creds::{
            sys_getegid, sys_geteuid, sys_getgid, sys_getresgid, sys_getresuid, sys_getsid,
            sys_gettid, sys_getuid, sys_setfsgid, sys_setfsuid, sys_setsid,
        },
        exec::sys_execve,
        exit::{sys_exit, sys_exit_group},
        fd_table::{
            dup::{sys_dup, sys_dup3},
            fcntl::sys_fcntl,
            select::{sys_ppoll, sys_pselect6},
        },
        prctl::sys_prctl,
        ptrace::{TracePoint, ptrace_stop, sys_ptrace},
        sleep::{sys_clock_nanosleep, sys_nanosleep},
        thread_group::{
            Pgid,
            pid::{sys_getpgid, sys_getpid, sys_getppid, sys_setpgid},
            rsrc_lim::sys_prlimit64,
            signal::{
                kill::{sys_kill, sys_tkill},
                sigaction::sys_rt_sigaction,
                sigaltstack::sys_sigaltstack,
                sigprocmask::sys_rt_sigprocmask,
            },
            umask::sys_umask,
            wait::{sys_wait4, sys_waitid},
        },
        threading::{futex::sys_futex, sys_set_robust_list, sys_set_tid_address},
    },
    sched::{current::current_task, sys_sched_yield},
};
use libkernel::{
    error::{KernelError, syscall_error::kern_err_to_syscall},
    memory::address::{TUA, UA, VA},
};

fn record_syscall(nr: u32) {
    let pid = current_task().descriptor().tgid().value() as u32;
    let timestamp = crate::drivers::timer::now()
        .map(|instant| instant.ticks())
        .unwrap_or(0);
    let mut log = SYSCALL_LOG.lock_save_irq();
    log.push(format!("syscall=0x{nr:x} pid={pid} ts={timestamp}\n"));
}

async fn sys_syslog(action: u64, buf: u64, len: u64) -> libkernel::error::Result<usize> {
    const SYSLOG_ACTION_READ_ALL: u64 = 3;
    const SYSLOG_ACTION_READ_CLEAR: u64 = 4;
    const SYSLOG_ACTION_SIZE_UNREAD: u64 = 9;
    const SYSLOG_ACTION_SIZE_BUFFER: u64 = 10;

    let prev = PREV_SYSCALL.load(AtomicOrdering::Relaxed);
    log::info!("[syslog] previous syscall: 0x{prev:x} ({prev})");

    match action {
        SYSLOG_ACTION_READ_ALL | SYSLOG_ACTION_READ_CLEAR => {
            let len = usize::try_from(len).map_err(|_| KernelError::InvalidValue)?;
            if len == 0 {
                return Ok(0);
            }

            let snapshot = {
                let log = SYSCALL_LOG.lock_save_irq();
                log.snapshot_bytes()
            };
            let copy_len = snapshot.len().min(len);

            if copy_len != 0 {
                copy_to_user_slice(&snapshot[..copy_len], UA::from_value(buf as usize)).await?;
            }

            if action == SYSLOG_ACTION_READ_CLEAR {
                SYSCALL_LOG.lock_save_irq().clear();
            }

            Ok(copy_len)
        }
        SYSLOG_ACTION_SIZE_UNREAD | SYSLOG_ACTION_SIZE_BUFFER => {
            Ok(SYSCALL_LOG.lock_save_irq().total_bytes)
        }
        _ => Ok(0),
    }
}

pub async fn handle_syscall() {
    current_task().update_accounting(None);
    current_task().in_syscall = true;
    ptrace_stop(TracePoint::SyscallEntry).await;

    let (nr, arg1, arg2, arg3, arg4, arg5, arg6) = {
        let mut task = current_task();

        let ctx = &mut task.ctx;
        let state = ctx.user();

        (
            state.x[8] as u32,
            state.x[0],
            state.x[1],
            state.x[2],
            state.x[3],
            state.x[4],
            state.x[5],
        )
    };

    record_syscall(nr);

    let res = match nr {
        0x5 => {
            sys_setxattr(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
                arg5 as _,
            )
            .await
        }
        0x6 => {
            sys_lsetxattr(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
                arg5 as _,
            )
            .await
        }
        0x7 => {
            sys_fsetxattr(
                arg1.into(),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
                arg5 as _,
            )
            .await
        }
        0x8 => {
            sys_getxattr(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0x9 => {
            sys_lgetxattr(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0xa => {
            sys_fgetxattr(
                arg1.into(),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0xb => {
            sys_listxattr(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                arg3 as _,
            )
            .await
        }
        0xc => {
            sys_llistxattr(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                arg3 as _,
            )
            .await
        }
        0xd => sys_flistxattr(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0xe => sys_removexattr(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0xf => sys_lremovexattr(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0x10 => sys_fremovexattr(arg1.into(), TUA::from_value(arg2 as _)).await,
        0x11 => sys_getcwd(TUA::from_value(arg1 as _), arg2 as _).await,
        0x17 => sys_dup(arg1.into()),
        0x18 => sys_dup3(arg1.into(), arg2.into(), arg3 as _),
        0x19 => sys_fcntl(arg1.into(), arg2 as _, arg3 as _).await,
        0x1d => sys_ioctl(arg1.into(), arg2 as _, arg3 as _).await,
        0x20 => Ok(0), // sys_flock is a noop
        0x22 => sys_mkdirat(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0x23 => sys_unlinkat(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0x24 => {
            sys_symlinkat(
                TUA::from_value(arg1 as _),
                arg2.into(),
                TUA::from_value(arg3 as _),
            )
            .await
        }
        0x25 => {
            sys_linkat(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3.into(),
                TUA::from_value(arg4 as _),
                arg5 as _,
            )
            .await
        }
        0x26 => {
            sys_renameat(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3.into(),
                TUA::from_value(arg4 as _),
            )
            .await
        }
        0x2b => sys_statfs(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0x2c => sys_fstatfs(arg1.into(), TUA::from_value(arg2 as _)).await,
        0x2d => sys_truncate(TUA::from_value(arg1 as _), arg2 as _).await,
        0x2e => sys_ftruncate(arg1.into(), arg2 as _).await,
        0x30 => sys_faccessat(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0x31 => sys_chdir(TUA::from_value(arg1 as _)).await,
        0x32 => sys_fchdir(arg1.into()).await,
        0x33 => sys_chroot(TUA::from_value(arg1 as _)).await,
        0x34 => sys_fchmod(arg1.into(), arg2 as _).await,
        0x35 => {
            sys_fchmodat(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
            )
            .await
        }
        0x36 => {
            sys_fchownat(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
                arg5 as _,
            )
            .await
        }
        0x37 => sys_fchown(arg1.into(), arg2 as _, arg3 as _).await,
        0x38 => {
            sys_openat(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
            )
            .await
        }
        0x39 => sys_close(arg1.into()).await,
        0x3b => sys_pipe2(TUA::from_value(arg1 as _), arg2 as _).await,
        0x3d => sys_getdents64(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0x3e => sys_lseek(arg1.into(), arg2 as _, arg3 as _).await,
        0x3f => sys_read(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0x40 => sys_write(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0x41 => sys_readv(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0x42 => sys_writev(arg1.into(), TUA::from_value(arg2 as _), arg3 as _).await,
        0x43 => {
            sys_pread64(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
            )
            .await
        }
        0x44 => {
            sys_pwrite64(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
            )
            .await
        }
        0x45 => {
            sys_preadv(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
            )
            .await
        }
        0x46 => {
            sys_pwritev(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
            )
            .await
        }
        0x47 => {
            sys_sendfile(
                arg1.into(),
                arg2.into(),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0x48 => {
            sys_pselect6(
                arg1 as _,
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                TUA::from_value(arg4 as _),
                TUA::from_value(arg5 as _),
                TUA::from_value(arg6 as _),
            )
            .await
        }
        0x49 => {
            sys_ppoll(
                TUA::from_value(arg1 as _),
                arg2 as _,
                TUA::from_value(arg3 as _),
                TUA::from_value(arg4 as _),
                arg5 as _,
            )
            .await
        }
        0x4e => {
            sys_readlinkat(
                arg1.into(),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0x4f => {
            sys_newfstatat(
                arg1.into(),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0x50 => sys_fstat(arg1.into(), TUA::from_value(arg2 as _)).await,
        0x51 => sys_sync().await,
        0x52 => sys_fsync(arg1.into()).await,
        0x53 => sys_fdatasync(arg1.into()).await,
        0x58 => {
            sys_utimensat(
                arg1.into(),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0x5a => sys_capget(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0x5b => sys_capset(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0x5d => {
            let _ = sys_exit(arg1 as _).await;

            debug_assert!(matches!(
                *current_task().state.lock_save_irq(),
                TaskState::Finished
            ));

            PREV_SYSCALL.store(nr, AtomicOrdering::Relaxed);
            // Don't process result on exit.
            return;
        }
        0x5e => {
            let _ = sys_exit_group(arg1 as _).await;

            debug_assert!(matches!(
                *current_task().state.lock_save_irq(),
                TaskState::Finished
            ));

            PREV_SYSCALL.store(nr, AtomicOrdering::Relaxed);
            // Don't process result on exit.
            return;
        }
        0x5f => {
            sys_waitid(
                arg1 as _,
                arg2 as _,
                TUA::from_value(arg3 as _),
                arg4 as _,
                TUA::from_value(arg5 as _),
            )
            .await
        }
        0x60 => sys_set_tid_address(TUA::from_value(arg1 as _)),
        0x62 => {
            sys_futex(
                TUA::from_value(arg1 as _),
                arg2 as _,
                arg3 as _,
                TUA::from_value(arg4 as _),
                TUA::from_value(arg5 as _),
                arg6 as _,
            )
            .await
        }
        0x63 => sys_set_robust_list(TUA::from_value(arg1 as _), arg2 as _).await,
        0x65 => sys_nanosleep(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0x70 => sys_clock_settime(arg1 as _, TUA::from_value(arg2 as _)).await,
        0x71 => sys_clock_gettime(arg1 as _, TUA::from_value(arg2 as _)).await,
        0x73 => {
            sys_clock_nanosleep(
                arg1 as _,
                arg2 as _,
                TUA::from_value(arg3 as _),
                TUA::from_value(arg4 as _),
            )
            .await
        }
        // Level 3 & 4 — syslog (klogctl) ----------------------------------------
        // syscall 0x74 is syslog/klogctl.  We use it as a kernel diagnostic:
        //   Level 3: prints the previous syscall number to the kernel log.
        //   Level 4: prints the full syscall history from the ring buffer
        //            (visible via dmesg).
        // Returns 0 to prevent a panic on unhandled syscall.
        0x74 => sys_syslog(arg1, arg2, arg3).await,
        // -------------------------------------------------------------------------
        0x75 => {
            sys_ptrace(
                arg1 as _,
                arg2 as _,
                TUA::from_value(arg3 as _),
                TUA::from_value(arg4 as _),
            )
            .await
        }
        0x7b => Err(KernelError::NotSupported),
        0x7c => sys_sched_yield(),
        0x81 => sys_kill(arg1 as _, arg2.into()),
        0x82 => sys_tkill(arg1 as _, arg2.into()),
        0x84 => sys_sigaltstack(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0x86 => {
            sys_rt_sigaction(
                arg1.into(),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0x87 => {
            sys_rt_sigprocmask(
                arg1 as _,
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                arg4 as _,
            )
            .await
        }
        0x8b => {
            // Special case for sys_rt_sigreturn
            current_task()
                .ctx
                .put_signal_work(Box::pin(ArchImpl::do_signal_return()));

            PREV_SYSCALL.store(nr, AtomicOrdering::Relaxed);
            return;
        }
        0x8e => sys_reboot(arg1 as _, arg2 as _, arg3 as _, arg4 as _).await,
        0x94 => {
            sys_getresuid(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
            )
            .await
        }
        0x96 => {
            sys_getresgid(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
            )
            .await
        }
        0x97 => sys_setfsuid(arg1 as _).map_err(|e| match e {}),
        0x98 => sys_setfsgid(arg1 as _).map_err(|e| match e {}),
        0x9a => sys_setpgid(arg1 as _, Pgid(arg2 as _)),
        0x9b => sys_getpgid(arg1 as _),
        0x9c => sys_getsid().await,
        0x9d => sys_setsid().await,
        0xa0 => sys_uname(TUA::from_value(arg1 as _)).await,
        0xa1 => sys_sethostname(TUA::from_value(arg1 as _), arg2 as _).await,
        0xa3 => Err(KernelError::InvalidValue),
        0xa6 => sys_umask(arg1 as _).map_err(|e| match e {}),
        0xa7 => sys_prctl(arg1 as _, arg2, arg3).await,
        0xa9 => sys_gettimeofday(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0xaa => sys_settimeofday(TUA::from_value(arg1 as _), TUA::from_value(arg2 as _)).await,
        0xac => sys_getpid().map_err(|e| match e {}),
        0xad => sys_getppid().map_err(|e| match e {}),
        0xae => sys_getuid().map_err(|e| match e {}),
        0xaf => sys_geteuid().map_err(|e| match e {}),
        0xb0 => sys_getgid().map_err(|e| match e {}),
        0xb1 => sys_getegid().map_err(|e| match e {}),
        0xb2 => sys_gettid().map_err(|e| match e {}),
        0xb3 => sys_sysinfo(TUA::from_value(arg1 as _)).await,
        0xc6 => Err(KernelError::NotSupported),
        0xd6 => sys_brk(VA::from_value(arg1 as _))
            .await
            .map_err(|e| match e {}),
        0xd7 => sys_munmap(VA::from_value(arg1 as usize), arg2 as _).await,
        0xdc => {
            sys_clone(
                arg1 as _,
                UA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
                TUA::from_value(arg5 as _),
                arg4 as _,
            )
            .await
        }
        0xdd => {
            sys_execve(
                TUA::from_value(arg1 as _),
                TUA::from_value(arg2 as _),
                TUA::from_value(arg3 as _),
            )
            .await
        }
        0xde => sys_mmap(arg1, arg2, arg3, arg4, arg5.into(), arg6).await,
        0xdf => Ok(0), // fadvise64_64 is a no-op
        0xe2 => sys_mprotect(VA::from_value(arg1 as _), arg2 as _, arg3 as _),
        0xe8 => sys_mincore(arg1, arg2 as _, TUA::from_value(arg3 as _)).await,
        0xe9 => Ok(0), // sys_madvise is a no-op
        0x104 => {
            sys_wait4(
                arg1.cast_signed() as _,
                TUA::from_value(arg2 as _),
                arg3 as _,
                TUA::from_value(arg4 as _),
            )
            .await
        }
        0x105 => {
            sys_prlimit64(
                arg1 as _,
                arg2 as _,
                TUA::from_value(arg3 as _),
                TUA::from_value(arg4 as _),
            )
            .await
        }
        0x108 => sys_name_to_handle_at(),
        0x109 => Err(KernelError::NotSupported),
        0x10b => sys_syncfs(arg1.into()).await,
        0x10e => {
            sys_process_vm_readv(
                arg1 as _,
                TUA::from_value(arg2 as _),
                arg3 as _,
                TUA::from_value(arg4 as _),
                arg5 as _,
                arg6 as _,
            )
            .await
        }
        0x114 => {
            sys_renameat2(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3.into(),
                TUA::from_value(arg4 as _),
                arg5 as _,
            )
            .await
        }
        0x116 => sys_getrandom(TUA::from_value(arg1 as _), arg2 as _, arg3 as _).await,
        0x11d => {
            sys_copy_file_range(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3.into(),
                TUA::from_value(arg4 as _),
                arg5 as _,
                arg6 as _,
            )
            .await
        }
        0x11e => {
            sys_preadv2(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
                arg5 as _,
            )
            .await
        }
        0x11f => {
            sys_pwritev2(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
                arg5 as _,
            )
            .await
        }
        0x123 => {
            sys_statx(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
                TUA::from_value(arg5 as _),
            )
            .await
        }
        0x125 => Err(KernelError::NotSupported),
        0x1b4 => sys_close_range(arg1.into(), arg2.into(), arg3 as _).await,
        0x1b7 => {
            sys_faccessat2(
                arg1.into(),
                TUA::from_value(arg2 as _),
                arg3 as _,
                arg4 as _,
            )
            .await
        }
        0x1b8 => Ok(0), // process_madvise is a no-op
        _ => panic!(
            "Unhandled syscall 0x{nr:x}, PC: 0x{:x}",
            current_task().ctx.user().elr_el1
        ),
    };

    let ret_val = match res {
        Ok(v) => v as isize,
        Err(e) => kern_err_to_syscall(e),
    };

    current_task().ctx.user_mut().x[0] = ret_val.cast_unsigned() as u64;
    ptrace_stop(TracePoint::SyscallExit).await;
    current_task().update_accounting(None);
    current_task().in_syscall = false;
    // Level 3: record the completed syscall number for the next syslog call.
    PREV_SYSCALL.store(nr, AtomicOrdering::Relaxed);
}

