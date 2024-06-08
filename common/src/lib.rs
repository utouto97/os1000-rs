#![no_std]
#![no_main]

use core::fmt::Write;

extern "C" {
    fn putchar(ch: u8);
}

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.as_bytes() {
            unsafe { putchar(*c) }
        }
        core::fmt::Result::Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    let mut console = Console;
    console.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct TrapFrame {
    pub ra: u32,
    pub gp: u32,
    pub tp: u32,
    pub t0: u32,
    pub t1: u32,
    pub t2: u32,
    pub t3: u32,
    pub t4: u32,
    pub t5: u32,
    pub t6: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub a4: u32,
    pub a5: u32,
    pub a6: u32,
    pub a7: u32,
    pub s0: u32,
    pub s1: u32,
    pub s2: u32,
    pub s3: u32,
    pub s4: u32,
    pub s5: u32,
    pub s6: u32,
    pub s7: u32,
    pub s8: u32,
    pub s9: u32,
    pub s10: u32,
    pub s11: u32,
    pub sp: u32,
}

#[macro_export]
macro_rules! read_csr {
    ($csr:expr) => {
        unsafe {
            use core::arch::asm;
            let mut csrr: u32;
            asm!(concat!("csrr {r}, ", $csr), r = out(reg) csrr);
            csrr
        }
    };
}

#[macro_export]
macro_rules! write_csr {
    ($csr:expr, $value:expr) => {
        unsafe {
            use core::arch::asm;
            asm!(concat!("csrw ", $csr, ", {r}"), r = in(reg) $value);
        }
    };
}

pub type PAddr = u32;
pub type VAddr = u32;

pub const PAGE_SIZE: usize = 4096;

pub const fn align_up(value: usize, align: usize) -> usize {
    let r = value % align;
    if r == 0 {
        value
    } else {
        value + (align - r)
    }
}

pub const fn is_aligned(value: usize, align: usize) -> bool {
    value % align == 0
}

pub const SYS_PUTCHAR: u32 = 1;
pub const SYS_GETCHAR: u32 = 2;
pub const SYS_EXIT: u32 = 3;

pub const VIRTIO_BLK_PADDR: usize = 0x10001000;
