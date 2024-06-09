use common::{align_up, ascii_len, oct2int, println};

use crate::virtio::Virtio;

#[repr(C, packed)]
struct TarHeader {
    name: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    checksum: [u8; 8],
    type_: u8,
    linkname: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    uname: [u8; 32],
    gname: [u8; 32],
    devmajor: [u8; 8],
    devminor: [u8; 8],
    prefix: [u8; 155],
    pad: [u8; 12],
    data: [u8; 0],
}

#[derive(Copy, Clone)]
pub struct File {
    pub in_use: bool,
    pub name: [u8; 100],
    pub data: [u8; 1024],
    pub size: usize,
}

impl File {
    const fn new() -> Self {
        Self {
            in_use: false,
            name: [0; 100],
            data: [0; 1024],
            size: 0,
        }
    }
}

const FILES_MAX: usize = 2;
const DISK_MAX_SIZE: usize = align_up(
    core::mem::size_of::<File>() * FILES_MAX,
    Virtio::SECTOR_SIZE as usize,
);

static mut FILES: [File; FILES_MAX] = [File::new(); FILES_MAX];
static mut DISK: [u8; DISK_MAX_SIZE] = [0; DISK_MAX_SIZE];
static mut VIRTIO: *mut Virtio = core::ptr::null_mut();

pub unsafe fn fs_init(virtio: &mut Virtio) {
    let mut sector = 0;
    while sector < core::mem::size_of_val(&DISK) / Virtio::SECTOR_SIZE as usize {
        // println!(
        //     "{:?}, {}  {}",
        //     sector,
        //     core::mem::size_of_val(&DISK),
        //     Virtio::SECTOR_SIZE
        // );
        virtio.read_write_disk(
            &mut DISK[(sector * Virtio::SECTOR_SIZE as usize)..],
            sector as u64,
            false,
        );
        sector += 1;
    }

    let mut off = 0;
    for i in 0..FILES_MAX {
        let header = (&mut DISK[off] as *mut _ as *mut TarHeader)
            .as_mut()
            .unwrap();

        let name = &core::str::from_utf8(&header.name).unwrap();
        if name.is_empty() {
            break;
        }

        let magic = &core::str::from_utf8(&header.magic).unwrap()
            [0..(ascii_len(&header.magic as *const u8) - 1)];
        if magic != "ustar" {
            panic!("invalid tar header: magic={magic}");
        }

        let filesz = oct2int(
            &header.size as *const [u8] as *const u8,
            core::mem::size_of_val(&header.size),
        );

        let file = &mut FILES[i];
        file.in_use = true;
        file.name[0..name.len()].copy_from_slice(name.as_bytes());
        if filesz > 0 {
            file.data[0..filesz].copy_from_slice(core::slice::from_raw_parts(
                &header.data as *const [u8] as *const u8,
                filesz,
            ))
        }
        file.size = filesz;
        println!(
            "file: {}, size={}",
            &core::str::from_utf8(&file.name).unwrap()[0..(ascii_len(&file.name as *const u8) - 1)],
            file.size,
        );

        off += align_up(
            core::mem::size_of::<TarHeader>() + filesz,
            Virtio::SECTOR_SIZE as usize,
        );
    }
}

pub unsafe fn fs_flush(virtio: &mut Virtio) {
    let mut off: usize = 0;
    for i in 0..FILES_MAX {
        let file = &mut FILES[i];
        if !file.in_use {
            continue;
        }

        let header = (&mut DISK[off] as *mut u8 as *mut TarHeader)
            .as_mut()
            .unwrap();
        let name = &file.name;
        header.name[0..file.name.len()].copy_from_slice(name);
        let mode = b"0000644\0";
        header.mode[0..mode.len()].copy_from_slice(mode);
        let magic = b"ustar\0";
        header.magic[0..magic.len()].copy_from_slice(magic);
        let version = b"00";
        header.version[0..version.len()].copy_from_slice(version);
        header.type_ = b'0';

        let mut filesz = file.size;
        for i in 0..(header.size.len() - 1) {
            header.size[(header.size.len() - 2) - i] = (filesz % 8) as u8 + b'0';
            filesz /= 8;
        }
        header.size[header.size.len() - 1] = b'\0';

        // チェックサムを計算
        let mut checksum = b' ' as usize * core::mem::size_of_val(&header.checksum);
        checksum += header.name.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.mode.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.uid.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.gid.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.size.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.mtime.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.type_ as usize;
        checksum += header.linkname.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.magic.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.version.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.uname.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.gname.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.devmajor.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.devminor.iter().fold(0, |sum, i| sum + *i as usize);
        checksum += header.prefix.iter().fold(0, |sum, i| sum + *i as usize);

        for i in 0..6 {
            header.checksum[(header.checksum.len() - 3) - i] = (checksum % 8) as u8 + b'0';
            checksum /= 8;
        }

        let header_data =
            core::slice::from_raw_parts_mut(&mut header.data as *mut [u8] as *mut u8, file.size);
        if file.size > 0 {
            header_data.copy_from_slice(&file.data[0..file.size]);
        }

        off += align_up(
            core::mem::size_of::<TarHeader>() + file.size,
            Virtio::SECTOR_SIZE as usize,
        );
    }

    for sector in 0..(core::mem::size_of_val(&DISK) / Virtio::SECTOR_SIZE as usize) {
        virtio.read_write_disk(
            &mut DISK[(sector * Virtio::SECTOR_SIZE as usize)..],
            sector as u64,
            true,
        );
    }

    println!("wrote {} bytes to disk", core::mem::size_of_val(&DISK));
}

pub fn fs_lookup(filename: &str) -> Result<*mut File, ()> {
    for i in 0..FILES_MAX {
        let file = unsafe { &FILES[i] };
        // println!(
        //     "file: {}, size={}",
        //     &core::str::from_utf8(&file.name).unwrap()[0..(ascii_len(&file.name as *const u8) - 1)],
        //     file.size,
        // );
        let name =
            &core::str::from_utf8(&file.name).unwrap()[0..(ascii_len(&file.name as *const u8) - 1)];
        if name == filename {
            return Ok(file as *const File as *mut File);
        }
    }
    Err(())
}
