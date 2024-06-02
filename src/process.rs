use core::{arch::asm, ptr};

use common::VAddr;

const PROCS_MAX: usize = 8;

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    UNUSED,
    RUNNABLE,
    IDLE,
}

#[derive(Copy, Clone, Debug)]
struct Process {
    pid: u32,
    state: State,
    sp: VAddr,
    stack: [u8; 8192],
}

impl Process {
    const fn new() -> Self {
        Self {
            pid: 0,
            state: State::UNUSED,
            sp: 0,
            stack: [0; 8192],
        }
    }
}

pub struct ProcessManager {
    procs: [Process; PROCS_MAX],
    current: usize,
}

impl ProcessManager {
    pub const fn new() -> Self {
        let mut pm = Self {
            procs: [Process::new(); PROCS_MAX],
            current: 0,
        };
        pm.procs[0].state = State::IDLE;
        pm
    }

    pub fn init(&mut self) {
        let proc = &mut self.procs[0];

        unsafe {
            let stack = ptr::addr_of_mut!(proc.stack) as *mut u32;
            let sp = stack.add(proc.stack.len());
            *sp.offset(-1) = 0; // s11
            *sp.offset(-2) = 0; // s10
            *sp.offset(-3) = 0; // s9
            *sp.offset(-4) = 0; // s8
            *sp.offset(-5) = 0; // s7
            *sp.offset(-6) = 0; // s6
            *sp.offset(-7) = 0; // s5
            *sp.offset(-8) = 0; // s4
            *sp.offset(-9) = 0; // s3
            *sp.offset(-10) = 0; // s2
            *sp.offset(-11) = 0; // s1
            *sp.offset(-12) = 0; // s0
            *sp.offset(-13) = 0; // ra

            proc.pid = u32::MAX as u32;
            proc.state = State::IDLE;
            proc.sp = sp.offset(-13) as VAddr;
        }
    }

    pub fn create(&mut self, pc: u32) {
        unsafe {
            if let Some((i, proc)) = self
                .procs
                .iter_mut()
                .enumerate()
                .find(|(_, p)| p.state == State::UNUSED)
            {
                let stack = ptr::addr_of_mut!(proc.stack) as *mut u32;
                let sp = stack.add(proc.stack.len());
                *sp.offset(-1) = 0; // s11
                *sp.offset(-2) = 0; // s10
                *sp.offset(-3) = 0; // s9
                *sp.offset(-4) = 0; // s8
                *sp.offset(-5) = 0; // s7
                *sp.offset(-6) = 0; // s6
                *sp.offset(-7) = 0; // s5
                *sp.offset(-8) = 0; // s4
                *sp.offset(-9) = 0; // s3
                *sp.offset(-10) = 0; // s2
                *sp.offset(-11) = 0; // s1
                *sp.offset(-12) = 0; // s0
                *sp.offset(-13) = pc as u32; // ra

                proc.pid = i as u32;
                proc.state = State::RUNNABLE;
                proc.sp = sp.offset(-13) as VAddr;
            } else {
                panic!("no free process slots");
            }
        }
    }

    pub fn yield_(&mut self) {
        let mut next: usize = 0;
        for i in 0..PROCS_MAX {
            let idx = (self.current + i + 1) % PROCS_MAX;
            let proc = &self.procs[idx];
            if proc.state == State::RUNNABLE {
                next = idx;
                break;
            }
        }

        if next == self.current {
            return;
        }

        unsafe {
            let next_proc = &mut self.procs[next];
            let next_stack = ptr::addr_of_mut!(next_proc.stack) as *mut u32;
            let next_stack_top = next_stack.add(next_proc.stack.len());
            asm!("csrw sscratch, {0}", in(reg) next_stack_top);
        }

        let prev = self.current;
        self.current = next;
        switch_context(&mut self.procs[prev].sp, &self.procs[next].sp);
    }
}

#[naked]
#[no_mangle]
extern "C" fn switch_context(prev_sp: *mut u32, next_sp: *const u32) {
    unsafe {
        asm!(
            "addi sp, sp, -13 * 4",
            "sw ra,  0  * 4(sp)",
            "sw s0,  1  * 4(sp)",
            "sw s1,  2  * 4(sp)",
            "sw s2,  3  * 4(sp)",
            "sw s3,  4  * 4(sp)",
            "sw s4,  5  * 4(sp)",
            "sw s5,  6  * 4(sp)",
            "sw s6,  7  * 4(sp)",
            "sw s7,  8  * 4(sp)",
            "sw s8,  9  * 4(sp)",
            "sw s9,  10 * 4(sp)",
            "sw s10, 11 * 4(sp)",
            "sw s11, 12 * 4(sp)",
            "sw sp, (a0)",
            "lw sp, (a1)",
            "lw ra,  0  * 4(sp)",
            "lw s0,  1  * 4(sp)",
            "lw s1,  2  * 4(sp)",
            "lw s2,  3  * 4(sp)",
            "lw s3,  4  * 4(sp)",
            "lw s4,  5  * 4(sp)",
            "lw s5,  6  * 4(sp)",
            "lw s6,  7  * 4(sp)",
            "lw s7,  8  * 4(sp)",
            "lw s8,  9  * 4(sp)",
            "lw s9,  10 * 4(sp)",
            "lw s10, 11 * 4(sp)",
            "lw s11, 12 * 4(sp)",
            "addi sp, sp, 13 * 4",
            "ret",
            options(noreturn),
        );
    }
}
