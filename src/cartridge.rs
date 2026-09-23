use crate::cpu::mapper::{BusConflicts, Mapper, Nrom, Uxrom};
use std::fs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    SingleScreenLower,
    SingleScreenUpper,
    FourScreen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderFormat { INes, Nes2 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timing { Ntsc, Pal, MultiRegion, Dendy }

#[derive(Clone, Debug)]
pub struct CartridgeHeader {
    pub format: HeaderFormat,
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
    pub mapper_number: u16,
    pub submapper: u8,
    pub mirroring: Mirroring,
    pub has_trainer: bool,
    pub battery_backed: bool,
    pub prg_ram_size: usize,
    pub prg_nvram_size: usize,
    pub chr_ram_size: usize,
    pub chr_nvram_size: usize,
    pub timing: Timing,
    pub console_type: u8,
    pub console_detail: u8,
    pub misc_rom_count: u8,
    pub expansion_device: u8,
}

// Retain the old public name for callers of parse_header.
#[allow(non_camel_case_types)]
pub type iNESHeader = CartridgeHeader;
pub type CartridgeMetadata = CartridgeHeader;

fn rom_size(low: u8, high: u8, unit: usize) -> Result<usize, String> {
    let size = if high == 15 {
        1usize.checked_shl((low >> 2) as u32)
            .and_then(|n| n.checked_mul(((low & 3) * 2 + 1) as usize))
    } else {
        (((high as usize) << 8) | low as usize).checked_mul(unit)
    };
    size.ok_or_else(|| "ROM size exceeds host address space".into())
}

fn ram_size(shift: u8) -> usize {
    if shift == 0 { 0 } else { 64usize << shift }
}

pub fn parse_header(bytes: &[u8]) -> Result<CartridgeHeader, String> {
    if bytes.len() < 16 { return Err("file too short to contain a cartridge header".into()); }
    if &bytes[..4] != b"NES\x1a" { return Err("invalid NES signature".into()); }
    let nes2 = bytes[7] & 0x0c == 0x08;
    let mapper_number = ((bytes[6] >> 4) as u16) | ((bytes[7] & 0xf0) as u16)
        | if nes2 { ((bytes[8] & 15) as u16) << 8 } else { 0 };
    let battery_backed = bytes[6] & 2 != 0;
    let chr_rom_size = rom_size(bytes[5], if nes2 { bytes[9] >> 4 } else { 0 }, 0x2000)?;
    // Legacy iNES RAM declarations are ambiguous. Preserve the existing
    // supported-board defaults; NES 2.0 sizes are always explicit.
    let legacy_ram = if mapper_number == 0 { (bytes[8] as usize).max(1) * 0x2000 }
        else { bytes[8] as usize * 0x2000 };
    Ok(CartridgeHeader {
        format: if nes2 { HeaderFormat::Nes2 } else { HeaderFormat::INes },
        mapper_number,
        submapper: if nes2 { bytes[8] >> 4 } else { 0 },
        prg_rom_size: rom_size(bytes[4], if nes2 { bytes[9] & 15 } else { 0 }, 0x4000)?,
        chr_rom_size,
        mirroring: if bytes[6] & 8 != 0 { Mirroring::FourScreen }
            else if bytes[6] & 1 != 0 { Mirroring::Vertical } else { Mirroring::Horizontal },
        has_trainer: bytes[6] & 4 != 0,
        battery_backed,
        prg_ram_size: if nes2 { ram_size(bytes[10] & 15) } else if battery_backed { 0 } else { legacy_ram },
        prg_nvram_size: if nes2 { ram_size(bytes[10] >> 4) } else if battery_backed { legacy_ram } else { 0 },
        chr_ram_size: if nes2 { ram_size(bytes[11] & 15) } else if chr_rom_size == 0 { 0x2000 } else { 0 },
        chr_nvram_size: if nes2 { ram_size(bytes[11] >> 4) } else { 0 },
        timing: if nes2 { match bytes[12] & 3 {
            0 => Timing::Ntsc, 1 => Timing::Pal, 2 => Timing::MultiRegion, _ => Timing::Dendy,
        }} else { Timing::Ntsc },
        console_type: bytes[7] & 3,
        console_detail: if nes2 { bytes[13] } else { 0 },
        misc_rom_count: if nes2 { bytes[14] & 3 } else { 0 },
        expansion_device: if nes2 { bytes[15] & 0x3f } else { 0 },
    })
}

pub struct LoadedRom {
    pub mapper: Box<dyn Mapper>,
    pub metadata: CartridgeMetadata,
}

pub fn load_rom(bytes: &[u8]) ->  Result<LoadedRom, String> {
    let header = parse_header(bytes)?;
    if header.console_type != 0 { return Err("unsupported console type (only NES/Famicom is supported)".into()); }
    if matches!(header.timing, Timing::Pal | Timing::Dendy) {
        return Err("unsupported timing: this emulator runs NTSC".into());
    }
    if header.misc_rom_count != 0 { return Err("miscellaneous cartridge ROMs are not supported".into()); }
    if header.chr_nvram_size != 0 { return Err("CHR NVRAM is not supported".into()); }
    if header.prg_nvram_size != 0 && !header.battery_backed {
        return Err("NVRAM declared without the battery flag".into());
    }
    let prg_start = 16usize + if header.has_trainer { 512 } else { 0 };
    let prg_end = prg_start.checked_add(header.prg_rom_size).ok_or("PRG file offset overflow")?;
    let chr_end = prg_end.checked_add(header.chr_rom_size).ok_or("CHR file offset overflow")?;
    if bytes.len() < chr_end { return Err("file too short for declared ROM sizes".into()); }
    let prg_ram_size = header.prg_ram_size.checked_add(header.prg_nvram_size)
        .ok_or("PRG RAM size overflow")?;
    let expected_chr_ram = if header.chr_rom_size == 0 { 0x2000 } else { 0 };
    if header.chr_ram_size != expected_chr_ram {
        return Err("supported boards require 8 KiB CHR RAM without CHR ROM, or no CHR RAM with CHR ROM".into());
    }

    let prg_rom = bytes[prg_start..prg_end].to_vec();
    let chr_rom = bytes[prg_end..chr_end].to_vec();

    let mapper: Box<dyn Mapper> = match header.mapper_number {
        0 => {
            if !matches!(prg_rom.len(), 0x4000 | 0x8000) || !matches!(chr_rom.len(), 0 | 0x2000) {
                return Err("unsupported NROM PRG/CHR size".into());
            }
            if header.submapper != 0 { return Err(format!("unsupported NROM submapper {}", header.submapper)); }
            if !matches!(prg_ram_size, 0 | 0x2000) || (header.prg_ram_size != 0 && header.prg_nvram_size != 0) {
                return Err("NROM supports either no PRG RAM or one 8 KiB volatile/nonvolatile RAM bank".into());
            }
            if header.has_trainer && prg_ram_size == 0 { return Err("NROM trainer requires PRG RAM".into()); }
            let mut mapper = Nrom::with_mirroring(prg_rom, chr_rom, header.mirroring);
            mapper.set_prg_ram_enabled(prg_ram_size != 0);
            if header.has_trainer {
                for (offset, data) in bytes[16..528].iter().enumerate() {
                    mapper.write(0x7000 + offset as u16, *data);
                }
            }
            Box::new(mapper)
        }
        2 => {
            if !chr_rom.is_empty() || header.battery_backed || prg_ram_size != 0 || header.has_trainer {
                return Err("supported UxROM boards require CHR RAM and no PRG RAM, battery, or trainer".into());
            }
            let conflicts = match header.submapper {
                0 | 2 => BusConflicts::And,
                1 => BusConflicts::None,
                n => return Err(format!("unsupported UxROM submapper {n}")),
            };
            Box::new(Uxrom::new(prg_rom, header.mirroring, conflicts)?)
        }
        n => return Err(format!("mapper {} not yet implemented", n)),
    };

    Ok(LoadedRom {
        mapper,
        metadata: header,
    })
}

// this actually grabs from the file
pub fn load_rom_from_file(path: &str) -> Result<LoadedRom, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("failed to read {}: {}", path, e))?;

    load_rom(&bytes)
}