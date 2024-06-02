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
