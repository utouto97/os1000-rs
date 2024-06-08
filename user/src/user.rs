#![no_std]
#![no_main]
#![feature(naked_functions)]

mod shell;

use common::{SYS_EXIT, SYS_GETCHAR, SYS_PUTCHAR};
use core::{arch::asm, panic::PanicInfo};

extern "C" {
    static __stack_top: u32;
}

#[link_section = ".text.start"]
#[naked]
#[no_mangle]
extern "C" fn start() {
    unsafe {
        asm!(
            "la sp, {stack_top}",
            "call main",
            "call exit",
            stack_top = sym  __stack_top,
            options(noreturn)
        );
    }
}

#[no_mangle]
fn exit() {
    unsafe { syscall(SYS_EXIT, 0, 0, 0) };
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe fn syscall(sysno: u32, arg0: u32, arg1: u32, arg2: u32) -> u32 {
    let mut result: u32;

    asm!(
        "ecall",
        in("a0") arg0,
        in("a1") arg1,
        in("a2") arg2,
        in("a3") sysno,
        lateout("a0") result,
    );

    result
}

pub fn putchar(ch: u8) {
    unsafe {
        syscall(SYS_PUTCHAR, ch as u32, 0, 0);
    }
}

pub fn getchar() -> u32 {
    unsafe { syscall(SYS_GETCHAR, 0, 0, 0) }
}
