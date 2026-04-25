use std::fs;
use std::io;

pub struct LoadedBinary {
    pub data: Vec<u8>,
    pub entry_point: u32,
}

fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

fn load_elf(file_data: &[u8]) -> io::Result<LoadedBinary> {
    if file_data.len() < 52 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "ELF header too short"));
    }
    if file_data[4] != 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a 32-bit ELF"));
    }
    if file_data[5] != 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not little-endian ELF"));
    }

    let entry = read_u32_le(file_data, 24);
    let phoff = read_u32_le(file_data, 28) as usize;
    let phnum = read_u16_le(file_data, 44) as usize;
    let phentsize = read_u16_le(file_data, 42) as usize;

    let mut max_addr: usize = 0;
    for i in 0..phnum {
        let ph = phoff + i * phentsize;
        let p_type = read_u32_le(file_data, ph);
        if p_type != 1 { continue; } // PT_LOAD
        let p_vaddr = read_u32_le(file_data, ph + 8) as usize;
        let p_memsz = read_u32_le(file_data, ph + 20) as usize;
        let end = p_vaddr + p_memsz;
        if end > max_addr { max_addr = end; }
    }

    let mut image = vec![0u8; max_addr];
    for i in 0..phnum {
        let ph = phoff + i * phentsize;
        let p_type = read_u32_le(file_data, ph);
        if p_type != 1 { continue; }
        let p_offset = read_u32_le(file_data, ph + 4) as usize;
        let p_vaddr = read_u32_le(file_data, ph + 8) as usize;
        let p_filesz = read_u32_le(file_data, ph + 16) as usize;
        image[p_vaddr..p_vaddr + p_filesz]
            .copy_from_slice(&file_data[p_offset..p_offset + p_filesz]);
    }

    Ok(LoadedBinary { data: image, entry_point: entry })
}

pub fn load(path: &str) -> io::Result<LoadedBinary> {
    let file_data = fs::read(path)?;
    if file_data.len() >= 4 && file_data[0..4] == ELF_MAGIC {
        load_elf(&file_data)
    } else {
        Ok(LoadedBinary { data: file_data, entry_point: 0 })
    }
}
