use crate::cpu::mapper::{Mapper, Nrom};
use std::fs;

pub enum Mirroring {
    Vertical,
    Horizontal
}

#[allow(non_camel_case_types)]
pub struct iNESHeader{
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
    pub mapper_number: u8,
    pub mirroring: Mirroring,
    pub has_trainer: bool,
    pub battery_backed: bool,
}

// gonna only do the iNES header for now, ill do the other one later
pub fn parse_header(bytes: &[u8]) -> Result<iNESHeader, String> {
    if bytes.len() < 16 {
        return Err("file too short to contain a valid iNES header".to_string());
    }
    if &bytes[0..4] != b"NES\x1A" {
        return Err(format!("invalid start of header, expected \"NES\" and 0x1A: {:02X?}", &bytes[0..4]));
    }

    let prg_rom_size = bytes[4] as usize * 16384;
    let chr_rom_size = bytes[5] as usize * 8192;

    let flags6 = bytes[6];
    let flags7 = bytes[7];

    let mirroring = if flags6 & 0b0000_0001 != 0 {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    };
    let battery_backed = flags6 & 0b0000_0010 != 0;
    let has_trainer = flags6 & 0b0000_0100 != 0;

    let mapper_low = (flags6 & 0b1111_0000) >> 4;
    let mapper_high = flags7 & 0b1111_0000;
    let mapper_number = mapper_high | mapper_low;

    Ok(iNESHeader {
        prg_rom_size,
        chr_rom_size,
        mapper_number,
        mirroring,
        has_trainer,
        battery_backed,
    })
}

pub fn load_rom(bytes: &[u8]) -> Result<Box<dyn Mapper>, String> {
    let header = parse_header(bytes)?;
    let prg_start = 16 + if header.has_trainer { 512 } else { 0 };
    let prg_end = prg_start + header.prg_rom_size;
    let chr_end = prg_end + header.chr_rom_size;

    if bytes.len() < chr_end {
        return Err(format!("file too short",));
    }

    let prg_rom = bytes[prg_start..prg_end].to_vec();
    let chr_rom = bytes[prg_end..chr_end].to_vec();

    match header.mapper_number {
        0 => Ok(Box::new(Nrom::new(prg_rom, chr_rom))),
        n => Err(format!("mapper {} not yet implemented", n)),
    }
}

// this actually grabs from the file
pub fn load_rom_from_file(path: &str) -> Result<Box<dyn Mapper>, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("failed to read {}: {}", path, e))?;

    load_rom(&bytes)
}