use common::{align_up, PAGE_SIZE, VIRTIO_BLK_PADDR};

use crate::{memory::alloc_pages, println};
use core::{
    arch::asm,
    mem,
    ptr::{self, read_volatile, write_volatile},
};

const VIRTQ_ENTRY_NUM: usize = 16;
const VIRTIO_DEVICE_BLK: u32 = 2;
const VIRTIO_REG_MAGIC: usize = 0x00;
const VIRTIO_REG_VERSION: usize = 0x04;
const VIRTIO_REG_DEVICE_ID: usize = 0x08;
const VIRTIO_REG_QUEUE_SEL: usize = 0x30;
// const VIRTIO_REG_QUEUE_NUM_MAX: usize = 0x34;
const VIRTIO_REG_QUEUE_NUM: usize = 0x38;
const VIRTIO_REG_QUEUE_ALIGN: usize = 0x3c;
const VIRTIO_REG_QUEUE_PFN: usize = 0x40;
// const VIRTIO_REG_QUEUE_READY: u32 = 0x44;
const VIRTIO_REG_QUEUE_NOTIFY: u32 = 0x50;
const VIRTIO_REG_DEVICE_STATUS: usize = 0x70;
const VIRTIO_REG_DEVICE_CONFIG: usize = 0x100;
const VIRTIO_STATUS_ACK: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;
const VIRTIO_STATUS_FEAT_OK: u32 = 8;
const VIRTQ_DESC_F_NEXT: u32 = 1;
const VIRTQ_DESC_F_WRITE: u32 = 2;
// const VIRTQ_AVAIL_F_NO_INTERRUPT: u32 = 1;
const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;

#[repr(C, packed)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C, packed)]
struct VirtqAvail {
    flags: u16,
    index: u16,
    ring: [u16; VIRTQ_ENTRY_NUM],
}

#[repr(C, packed)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C, packed)]
struct VirtqUsed {
    flags: u16,
    index: u16,
    ring: [VirtqUsedElem; VIRTQ_ENTRY_NUM],
}

const SIZE_OF_U16: usize = core::mem::size_of::<u16>();
const SIZE_OF_U32: usize = core::mem::size_of::<u32>();
const SIZE_OF_U64: usize = core::mem::size_of::<u64>();

const SIZE: usize = (SIZE_OF_U64 + SIZE_OF_U16 + SIZE_OF_U32 + SIZE_OF_U16) * VIRTQ_ENTRY_NUM
    + (SIZE_OF_U16 + SIZE_OF_U16 + SIZE_OF_U16 * VIRTQ_ENTRY_NUM as usize);

#[repr(C, packed)]
struct VirtioVirtq {
    descs: [VirtqDesc; VIRTQ_ENTRY_NUM],
    avail: VirtqAvail,
    pad: [u8; (PAGE_SIZE - (SIZE % PAGE_SIZE)) / mem::size_of::<u8>()],
    used: VirtqUsed,

    queue_index: u32,
    used_index: *mut u16,
    last_used_index: u16,
}

#[repr(C, packed)]
struct VirtioBlkReq {
    type_: u32,
    reserved: u32,
    sector: u64,
    data: [u8; 512],
    status: u8,
}

fn virtio_reg_read32(offset: usize) -> u32 {
    unsafe { read_volatile((VIRTIO_BLK_PADDR + offset) as *const u32) }
}

fn virtio_reg_read64(offset: usize) -> u64 {
    unsafe { read_volatile((VIRTIO_BLK_PADDR + offset) as *const u64) }
}

fn virtio_reg_write32(offset: usize, value: u32) {
    unsafe { write_volatile((VIRTIO_BLK_PADDR + offset) as *mut u32, value) }
}

fn virtio_reg_fetch_and_or32(offset: usize, value: u32) {
    let current_value = virtio_reg_read32(offset);
    virtio_reg_write32(offset, current_value | value);
}

pub struct Virtio<'a> {
    blk_request_vq: &'a mut VirtioVirtq,
    blk_req: &'a mut VirtioBlkReq,
    blk_req_paddr: u32,
    blk_capacity: u64,
}

impl<'a> Virtio<'a> {
    pub const SECTOR_SIZE: u64 = 512;

    pub fn new() -> Self {
        unsafe {
            if virtio_reg_read32(VIRTIO_REG_MAGIC) != 0x74726976 {
                panic!("virtio: invalid magic value");
            }
            if virtio_reg_read32(VIRTIO_REG_VERSION) != 1 {
                panic!("virtio: invalid version");
            }
            if virtio_reg_read32(VIRTIO_REG_DEVICE_ID) != VIRTIO_DEVICE_BLK {
                panic!("virtio: invalid device id");
            }

            virtio_reg_write32(VIRTIO_REG_DEVICE_STATUS, 0);
            virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_ACK);
            virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER);
            virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_FEAT_OK);
            let blk_request_vq = Self::virtq_init(0);
            virtio_reg_write32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER_OK);

            let blk_capacity = virtio_reg_read64(VIRTIO_REG_DEVICE_CONFIG) * Self::SECTOR_SIZE;
            println!("virtio-blk: capacity is {} bytes\n", blk_capacity);

            let blk_req_size = align_up(core::mem::size_of::<VirtioBlkReq>(), PAGE_SIZE);
            let blk_req_paddr = alloc_pages(blk_req_size / PAGE_SIZE);

            Self {
                blk_request_vq: blk_request_vq.as_mut().unwrap(),
                blk_req: (blk_req_paddr as *mut VirtioBlkReq).as_mut().unwrap(),
                blk_req_paddr,
                blk_capacity,
            }
        }
    }

    unsafe fn virtq_init(index: u32) -> *mut VirtioVirtq {
        let virtq_size = align_up(core::mem::size_of::<VirtioVirtq>(), PAGE_SIZE);
        let virtq_paddr = alloc_pages(virtq_size / PAGE_SIZE);
        let vq = (virtq_paddr as *mut VirtioVirtq).as_mut().unwrap();

        vq.queue_index = index;
        let used_index = (&mut (vq.used) as *const VirtqUsed as *const u8)
            .offset(core::mem::offset_of!(VirtqUsed, index) as isize);
        vq.used_index = used_index as *mut u16;

        virtio_reg_write32(VIRTIO_REG_QUEUE_SEL, index);
        virtio_reg_write32(VIRTIO_REG_QUEUE_NUM, VIRTQ_ENTRY_NUM as u32);
        virtio_reg_write32(VIRTIO_REG_QUEUE_ALIGN, 0);
        virtio_reg_write32(VIRTIO_REG_QUEUE_PFN, virtq_paddr);

        vq
    }

    fn virtq_kick(vq: &mut VirtioVirtq, desc_index: u32) {
        vq.avail.ring[vq.avail.index as usize % VIRTQ_ENTRY_NUM] = desc_index as u16;
        vq.avail.index += 1;
        unsafe { asm!("fence") }
        virtio_reg_write32(VIRTIO_REG_QUEUE_NOTIFY as usize, vq.queue_index);
        vq.last_used_index += 1;
    }

    fn virtq_is_busy(vq: &mut VirtioVirtq) -> bool {
        unsafe { vq.last_used_index != ptr::read_volatile(vq.used_index) }
    }

    pub fn read_write_disk(&mut self, buf: &mut [u8], sector: u64, is_write: bool) {
        unsafe {
            if sector >= self.blk_capacity / Self::SECTOR_SIZE as u64 {
                println!(
                    "virtio: tried to read/write sector={}, but capacity is {}",
                    sector,
                    self.blk_capacity / Self::SECTOR_SIZE as u64
                );
                return;
            }

            self.blk_req.sector = sector;
            self.blk_req.type_ = if is_write {
                VIRTIO_BLK_T_OUT
            } else {
                VIRTIO_BLK_T_IN
            };
            if is_write {
                ptr::copy(
                    buf as *mut _ as *mut u8,
                    &mut self.blk_req.data as *mut [u8] as *mut u8,
                    Self::SECTOR_SIZE as usize,
                );
            }

            self.blk_request_vq.descs[0].addr = self.blk_req_paddr as u64;
            self.blk_request_vq.descs[0].len =
                (mem::size_of::<u32>() * 2 + mem::size_of::<u64>()) as u32;
            self.blk_request_vq.descs[0].flags = VIRTQ_DESC_F_NEXT as u16;
            self.blk_request_vq.descs[0].next = 1;

            self.blk_request_vq.descs[1].addr =
                self.blk_req_paddr as u64 + mem::offset_of!(VirtioBlkReq, data) as u64;
            self.blk_request_vq.descs[1].len = Self::SECTOR_SIZE as u32;
            self.blk_request_vq.descs[1].flags =
                (VIRTQ_DESC_F_NEXT | if is_write { 0 } else { VIRTQ_DESC_F_WRITE }) as u16;
            self.blk_request_vq.descs[1].next = 2;

            self.blk_request_vq.descs[2].addr =
                self.blk_req_paddr as u64 + mem::offset_of!(VirtioBlkReq, status) as u64;
            self.blk_request_vq.descs[2].len = mem::size_of::<u8>() as u32;
            self.blk_request_vq.descs[2].flags = VIRTQ_DESC_F_WRITE as u16;

            Self::virtq_kick(self.blk_request_vq, 0);

            while Self::virtq_is_busy(self.blk_request_vq) {}

            if self.blk_req.status != 0 {
                println!(
                    "virtio: warn: failed to read/write sector={} status={}",
                    sector, self.blk_req.status,
                );
                return;
            }

            if !is_write {
                ptr::copy(
                    &self.blk_req.data as *const [u8] as *const u8,
                    buf as *mut _ as *mut u8,
                    Self::SECTOR_SIZE as usize,
                );
            }
        }
    }
}
