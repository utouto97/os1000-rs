use core::ptr;

use common::{is_aligned, PAddr, VAddr, PAGE_SIZE};

extern "C" {
    static mut __free_ram: u8;
    static mut __free_ram_end: u8;
}

pub const SATP_SV32: u32 = 1 << 31;
pub const PAGE_V: u32 = 1 << 0;
pub const PAGE_R: u32 = 1 << 1;
pub const PAGE_W: u32 = 1 << 2;
pub const PAGE_X: u32 = 1 << 3;
pub const PAGE_U: u32 = 1 << 4;

static mut NEXT_PADDR: *mut u8 = unsafe { ptr::addr_of_mut!(__free_ram) };

pub fn alloc_pages(n: usize) -> PAddr {
    unsafe {
        let paddr = NEXT_PADDR as PAddr;
        NEXT_PADDR = NEXT_PADDR.add(n * PAGE_SIZE);

        if NEXT_PADDR > ptr::addr_of_mut!(__free_ram_end) {
            panic!("out of memory");
        }

        ptr::write_bytes(paddr as *mut u8, 0, (n * PAGE_SIZE) as usize);
        paddr
    }
}

pub fn map_page(table1: *mut u32, vaddr: VAddr, paddr: PAddr, flags: u32) {
    if !is_aligned(vaddr as usize, PAGE_SIZE) {
        panic!("unaligned vaddr {vaddr}");
    }
    if !is_aligned(paddr as usize, PAGE_SIZE) {
        panic!("unaligned paddr {paddr}");
    }

    let table1 = table1 as *mut u32;
    let vpn1 = ((vaddr >> 22) & 0x3ff) as isize;
    unsafe {
        if (*table1.offset(vpn1) & PAGE_V) == 0 {
            let pt_paddr = alloc_pages(1);
            *table1.offset(vpn1) = ((pt_paddr / PAGE_SIZE as u32) << 10) | PAGE_V;
        }

        let vpn0 = ((vaddr >> 12) & 0x3ff) as isize;
        let table0 = ((*table1.offset(vpn1) >> 10) * PAGE_SIZE as u32) as *mut u32;
        *(table0.offset(vpn0)) = ((paddr / PAGE_SIZE as u32) << 10) | flags | PAGE_V;
    }
}
