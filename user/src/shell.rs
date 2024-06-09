use crate::{exit, getchar, putchar, readfile};

#[no_mangle]
fn main() {
    loop {
        print("> ");
        let mut cmdline: [u8; 128] = [0; 128];
        let mut count = 0;
        loop {
            let ch = getchar() as u8;
            putchar(ch);
            if ch == b'\r' {
                cmdline[count] = b'\0';
                print("\n");
                break;
            } else {
                cmdline[count] = ch;
            }

            count += 1;
            if count == 128 {
                break;
            }
        }
        match core::str::from_utf8(&cmdline[..count]) {
            Ok(s) => {
                if s == "hello" {
                    print("Hello world from shell!\n");
                } else if s == "exit" {
                    exit();
                } else if s == "readfile" {
                    let mut buf: [u8; 128] = [0; 128];
                    readfile("./lorem.txt\0", &mut buf, 128);
                    match core::str::from_utf8(&buf) {
                        Ok(s) => {
                            print(s);
                        }
                        Err(_) => print("error"),
                    }
                } else {
                    print("command not found\n");
                }
            }
            Err(_) => print("command not found\n"),
        }
        print("\n");
    }
}

fn print(s: &str) {
    for c in s.bytes() {
        putchar(c);
    }
}
