#[no_mangle]
fn main() {
    unsafe {
        let addr = 0x80200000 as *mut u32;
        *addr = 0x1234;
    }

    loop {}
}
