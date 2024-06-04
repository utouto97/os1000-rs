use crate::putchar;

#[no_mangle]
fn main() {
    let msg = "hello";
    for c in msg.bytes() {
        putchar(c);
    }

    loop {}
}
