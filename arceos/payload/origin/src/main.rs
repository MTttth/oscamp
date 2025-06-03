#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
#[unsafe(naked)]
unsafe extern "C" fn _start() -> ! {
    #[cfg(target_arch = "riscv64")]
    core::arch::naked_asm!(
        "addi sp, sp, -4",
        "sw a0, (sp)",
        "li a7, 93",
        "ecall",
    );
    #[cfg(target_arch = "x86_64")]
    core::arch::naked_asm!(
        "sub rsp, 64",
        "mov    qword ptr [rsp], 93",
        "mov rax, 93",
        "xor rdi, rdi",
        "syscall",
    );
    #[cfg(target_arch = "aarch64")]
    core::arch::naked_asm!(
        "sub sp, sp, #4",      // AArch64 栈操作
        "mov sp, x0",           // 设置返回值为 0
        "mov x8, #93",         // AArch64 系统调用号存放在 x8
        "svc #0",              // 执行系统调用
    );
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
