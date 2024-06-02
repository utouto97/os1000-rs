#![no_std]
#![no_main]
#![feature(naked_functions)]

mod shell;

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
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
