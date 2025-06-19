#![allow(dead_code)]

use core::ffi::c_void;
use axhal::arch::TrapFrame;
use axhal::trap::{register_trap_handler, SYSCALL};
use axerrno::LinuxError;
use axtask::current;
use axtask::TaskExtRef;
use arceos_posix_api as api;

#[cfg(target_arch = "riscv64")]
pub mod syscall_num {
    // 以下数字来源于 QEMU 平台的反馈，和实际或有出入

    /// ioctl 在 RISC-V Linux 下的 syscall 编号
    pub const SYS_IOCTL: usize = 29;

    /// set_tid_address 在 RISC-V Linux 下的 syscall 编号
    pub const SYS_SET_TID_ADDRESS: usize = 96;

    /// writev 在 RISC-V Linux 下的 syscall 编号
    pub const SYS_WRITEV: usize = 66;

    /// exit 在 RISC-V Linux 下的 syscall 编号
    pub const SYS_EXIT: usize = 93;

    /// exit_group 在 RISC-V Linux 下的 syscall 编号
    pub const SYS_EXIT_GROUP: usize = 94;
}

#[cfg(target_arch = "x86_64")]
pub mod syscall_num {
    // 以下数字来源于 QEMU 平台的反馈，和实际或有出入。
    // 这里只列出我们关心的那几条：

    /// ioctl 在 x86_64 Linux 下的 syscall 编号
    pub const SYS_IOCTL: usize = 218;

    /// set_tid_address 在 x86_64 Linux 下的 syscall 编号
    pub const SYS_SET_TID_ADDRESS: usize = 158;

    /// writev 在 x86_64 Linux 下的 syscall 编号
    pub const SYS_WRITEV: usize = 20;

    /// exit 在 x86_64 Linux 下的 syscall 编号
    pub const SYS_EXIT: usize = 60;

    /// exit_group 在 x86_64 Linux 下的 syscall 编号
    pub const SYS_EXIT_GROUP: usize = 231;
}

#[register_trap_handler(SYSCALL)]
fn handle_syscall(tf: &TrapFrame, syscall_num: usize) -> isize {
    ax_println!("handle_syscall [{}] ...", syscall_num);
    let ret = match syscall_num {
        syscall_num::SYS_IOCTL => sys_ioctl(tf.arg0() as _, tf.arg1() as _, tf.arg2() as _) as _,
        syscall_num::SYS_SET_TID_ADDRESS => sys_set_tid_address(tf.arg0() as _),
        syscall_num::SYS_WRITEV => sys_writev(tf.arg0() as _, tf.arg1() as _, tf.arg2() as _),
        syscall_num::SYS_EXIT_GROUP => {
            ax_println!("[SYS_EXIT_GROUP]: system is exiting ..");
            axtask::exit(tf.arg0() as _)
        },
        syscall_num::SYS_EXIT => {
            ax_println!("[SYS_EXIT]: system is exiting ..");
            axtask::exit(tf.arg0() as _)
        },
        _ => {
            ax_println!("Unimplemented syscall: {}", syscall_num);
            -LinuxError::ENOSYS.code() as _
        }
    };
    ret
}

fn sys_writev(fd: i32, iov: *const api::ctypes::iovec, iocnt: i32) -> isize {
    unsafe { api::sys_writev(fd, iov, iocnt) }
}

pub(crate) fn sys_set_tid_address(tid_ptd: *const i32) -> isize {
    let curr = current();
    curr.task_ext().set_clear_child_tid(tid_ptd as _);
    curr.id().as_u64() as isize
}
/// 0x0000000000401ef8
fn sys_ioctl(_fd: i32, _op: usize, _argp: *mut c_void) -> i32 {
    ax_println!("Unimplemented syscall: SYS_IOCTL");
    0
}
