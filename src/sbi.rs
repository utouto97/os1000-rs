use core::arch::asm;

struct SbiRet {
    _error: i32,
    _value: i32,
}

unsafe fn sbi_call(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
    fid: i32,
    eid: i32,
) -> SbiRet {
    let mut error;
    let mut value;
    asm!(
        "ecall",
        inout("a0") arg0 => error, inout("a1") arg1 => value,
        in("a2") arg2, in("a3") arg3, in("a4") arg4, in("a5") arg5,
        in("a6") fid, in("a7") eid
    );
    SbiRet {
        _error: error,
        _value: value,
    }
}

#[no_mangle]
pub fn putchar(ch: u8) {
    unsafe {
        sbi_call(ch as i32, 0, 0, 0, 0, 0, 0, 1);
    }
}

#[no_mangle]
pub fn getchar() -> i32 {
    unsafe {
        let ret = sbi_call(0, 0, 0, 0, 0, 0, 0, 2);
        return ret._error;
    }
}
