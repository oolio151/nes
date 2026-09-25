use crate::cartridge::Mirroring;
use crate::savestate::MapperState;

pub trait Mapper {
    fn save_state(&self) -> Result<MapperState, String> {
        Err("savestates unsupported by this mapper".into())
    }
    fn load_state(&mut self, _state: &MapperState) -> Result<(), String> {
        Err("savestates unsupported by this mapper".into())
    }
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
    fn ppu_read(&self, address: u16) -> u8;
    fn ppu_write(&mut self, address: u16, data: u8);
    fn mirroring(&self) -> Mirroring;
    fn notify_ppu_address(&mut self, _address: u16) {}
}

pub struct Nrom {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
    chr_is_ram: bool,
    prg_ram: [u8; 0x2000],
    prg_ram_enabled: bool,
    mirroring: Mirroring,
}

impl Nrom {
    pub(crate) fn set_prg_ram_enabled(&mut self, enabled: bool) {
        self.prg_ram_enabled = enabled;
    }

    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self::with_mirroring(prg_rom, chr_rom, Mirroring::Horizontal)
    }

    pub fn with_mirroring(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        assert!(matches!(prg_rom.len(), 0x4000 | 0x8000), "invalid NROM PRG size");
        assert!(matches!(chr_rom.len(), 0 | 0x2000), "invalid NROM CHR size");
        let chr_is_ram = chr_rom.is_empty();
        Self {
            prg_rom,
            chr: if chr_is_ram { vec![0; 0x2000] } else { chr_rom },
            chr_is_ram,
            prg_ram: [0; 0x2000],
            prg_ram_enabled: true,
            mirroring,
        }
    }
}

impl Mapper for Nrom {
    fn save_state(&self) -> Result<MapperState, String> {
        Ok(MapperState::Nrom {
            prg_ram: self.prg_ram,
            chr_ram: if self.chr_is_ram { self.chr.clone() } else { Vec::new() },
        })
    }
    fn load_state(&mut self, state: &MapperState) -> Result<(), String> {
        let MapperState::Nrom { prg_ram, chr_ram } = state else {
            return Err("savestate mapper mismatch".into());
        };
        if chr_ram.len() != if self.chr_is_ram { 0x2000 } else { 0 } {
            return Err("invalid NROM CHR RAM state".into());
        }
        self.prg_ram = *prg_ram;
        if self.chr_is_ram { self.chr.copy_from_slice(chr_ram); }
        Ok(())
    }
    fn read(&self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF if self.prg_ram_enabled => self.prg_ram[(address - 0x6000) as usize],
            0x8000..=0xFFFF => self.prg_rom[(address as usize - 0x8000) % self.prg_rom.len()],
            _ => 0,
        }
    }
    fn write(&mut self, address: u16, data: u8) {
        if self.prg_ram_enabled && (0x6000..=0x7fff).contains(&address) { self.prg_ram[(address - 0x6000) as usize] = data; }
    }
    fn ppu_read(&self, address: u16) -> u8 { self.chr[address as usize & 0x1fff] }
    fn ppu_write(&mut self, address: u16, data: u8) {
        if self.chr_is_ram { self.chr[address as usize & 0x1fff] = data; }
    }
    fn mirroring(&self) -> Mirroring { self.mirroring }
}

#[derive(Clone, Copy, Debug)]
pub enum BusConflicts { None, And }

pub struct Uxrom {
    prg_rom: Vec<u8>,
    chr_ram: [u8; 0x2000],
    selected_bank: u8,
    bank_mask: u8,
    mirroring: Mirroring,
    bus_conflicts: BusConflicts,
}

impl Uxrom {
    pub fn new(prg_rom: Vec<u8>, mirroring: Mirroring, bus_conflicts: BusConflicts) -> Result<Self, String> {
        let size = prg_rom.len();
        if !(0x8000..=0x40000).contains(&size) || !size.is_power_of_two() {
            return Err("UxROM requires power-of-two PRG size from 32 to 256 KiB".into());
        }
        Ok(Self {
            bank_mask: if size <= 0x20000 { 7 } else { 15 },
            prg_rom, chr_ram: [0; 0x2000], selected_bank: 0,
            mirroring, bus_conflicts,
        })
    }
}

impl Mapper for Uxrom {
    fn read(&self, address: u16) -> u8 {
        if address < 0x8000 { return 0; }
        let banks = self.prg_rom.len() / 0x4000;
        let bank = if address < 0xc000 { self.selected_bank as usize % banks } else { banks - 1 };
        self.prg_rom[bank * 0x4000 + (address as usize & 0x3fff)]
    }
    fn write(&mut self, address: u16, data: u8) {
        if address >= 0x8000 {
            let effective = match self.bus_conflicts {
                BusConflicts::None => data,
                BusConflicts::And => data & self.read(address),
            };
            self.selected_bank = effective & self.bank_mask;
        }
    }
    fn ppu_read(&self, address: u16) -> u8 { self.chr_ram[address as usize & 0x1fff] }
    fn ppu_write(&mut self, address: u16, data: u8) { self.chr_ram[address as usize & 0x1fff] = data; }
    fn mirroring(&self) -> Mirroring { self.mirroring }
    fn save_state(&self) -> Result<MapperState, String> {
        Ok(MapperState::Uxrom { selected_bank: self.selected_bank, chr_ram: self.chr_ram })
    }
    fn load_state(&mut self, state: &MapperState) -> Result<(), String> {
        let MapperState::Uxrom { selected_bank, chr_ram } = state else {
            return Err("savestate mapper mismatch".into());
        };
        if selected_bank & !self.bank_mask != 0 { return Err("invalid UxROM bank state".into()); }
        self.selected_bank = *selected_bank;
        self.chr_ram = *chr_ram;
        Ok(())
    }
}

pub struct Cnrom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    selected_chr_bank: u8,
    mirroring: Mirroring,
    bus_conflicts: BusConflicts,
}

impl Cnrom {
    pub fn new(
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
        mirroring: Mirroring,
        bus_conflicts: BusConflicts,
    ) -> Result<Self, String> {
        if !matches!(prg_rom.len(), 0x4000 | 0x8000) {
            return Err("CNROM requires 16 or 32 KiB PRG ROM".into());
        }
        if !matches!(chr_rom.len(), 0x2000 | 0x4000 | 0x8000) {
            return Err("CNROM requires 8, 16, or 32 KiB CHR ROM".into());
        }
        Ok(Self { prg_rom, chr_rom, selected_chr_bank: 0, mirroring, bus_conflicts })
    }
}

impl Mapper for Cnrom {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xffff => self.prg_rom[(address as usize - 0x8000) % self.prg_rom.len()],
            _ => 0, // no program rom
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if address >= 0x8000 {
            let effective = match self.bus_conflicts {
                BusConflicts::None => data,
                BusConflicts::And => data & self.read(address),
            };
            self.selected_chr_bank = effective & 3;
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        let bank = self.selected_chr_bank as usize % (self.chr_rom.len() / 0x2000);
        self.chr_rom[bank * 0x2000 + (address as usize & 0x1fff)]
    }

    fn ppu_write(&mut self, _address: u16, _data: u8) {}

    fn mirroring(&self) -> Mirroring { self.mirroring }

    fn save_state(&self) -> Result<MapperState, String> {
        Ok(MapperState::Cnrom { selected_chr_bank: self.selected_chr_bank })
    }

    fn load_state(&mut self, state: &MapperState) -> Result<(), String> {
        let MapperState::Cnrom { selected_chr_bank } = state else {
            return Err("savestate mapper mismatch".into());
        };
        if *selected_chr_bank > 3 {
            return Err("invalid CNROM bank state".into());
        }
        self.selected_chr_bank = *selected_chr_bank;
        Ok(())
    }
}
