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
    /// CPU instruction writes carry their actual write-cycle offset.
    fn write_timed(&mut self, address: u16, data: u8, _cpu_cycle: u64) {
        self.write(address, data);
    }
    fn reset_write_timing(&mut self) {}
    fn persistent_ram(&self) -> &[u8] { &[] }
    fn load_persistent_ram(&mut self, data: &[u8]) -> Result<(), String> {
        if data.is_empty() { Ok(()) } else { Err("mapper has no persistent RAM".into()) }
    }
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
    battery_backed: bool,
    mirroring: Mirroring,
}

impl Nrom {
    pub(crate) fn set_battery_backed(&mut self, enabled: bool) { self.battery_backed = enabled; }

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
            battery_backed: false,
            mirroring,
        }
    }
}

impl Mapper for Nrom {
    fn persistent_ram(&self) -> &[u8] {
        if self.battery_backed && self.prg_ram_enabled { &self.prg_ram } else { &[] }
    }
    fn load_persistent_ram(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != self.persistent_ram().len() {
            return Err("battery save size does not match NROM PRG NVRAM".into());
        }
        if !data.is_empty() { self.prg_ram.copy_from_slice(data); }
        Ok(())
    }

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

/// Ordinary AxROM: three PRG bank bits, one nametable page bit, fixed CHR RAM.
pub struct Axrom {
    prg_rom: Vec<u8>,
    chr_ram: [u8; 0x2000],
    selected_prg_bank: u8,
    nametable_upper: bool,
    bus_conflicts: BusConflicts,
}

impl Axrom {
    pub fn new(prg_rom: Vec<u8>, bus_conflicts: BusConflicts) -> Result<Self, String> {
        let size = prg_rom.len();
        if !(0x8000..=0x40000).contains(&size) || !size.is_power_of_two() {
            return Err("AxROM requires power-of-two PRG size from 32 to 256 KiB".into());
        }
        // Deterministic startup policy; games must initialize the bank register.
        Ok(Self {
            prg_rom,
            chr_ram: [0; 0x2000],
            selected_prg_bank: 0,
            nametable_upper: false,
            bus_conflicts,
        })
    }
}

impl Mapper for Axrom {
    fn read(&self, address: u16) -> u8 {
        if address < 0x8000 { return 0; }
        // Smaller chips mirror selections for unconnected bank address lines.
        let bank = self.selected_prg_bank as usize % (self.prg_rom.len() / 0x8000);
        self.prg_rom[bank * 0x8000 + (address as usize & 0x7fff)]
    }

    fn write(&mut self, address: u16, data: u8) {
        if address >= 0x8000 {
            // Sample the old PRG mapping before changing either register field.
            let effective = match self.bus_conflicts {
                BusConflicts::None => data,
                BusConflicts::And => data & self.read(address),
            };
            self.selected_prg_bank = effective & 7;
            self.nametable_upper = effective & 0x10 != 0;
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        self.chr_ram[address as usize & 0x1fff]
    }

    fn ppu_write(&mut self, address: u16, data: u8) {
        self.chr_ram[address as usize & 0x1fff] = data;
    }

    fn mirroring(&self) -> Mirroring {
        if self.nametable_upper { Mirroring::SingleScreenUpper }
        else { Mirroring::SingleScreenLower }
    }

    fn save_state(&self) -> Result<MapperState, String> {
        Ok(MapperState::Axrom {
            selected_prg_bank: self.selected_prg_bank,
            nametable_upper: self.nametable_upper,
            chr_ram: self.chr_ram,
        })
    }

    fn load_state(&mut self, state: &MapperState) -> Result<(), String> {
        let MapperState::Axrom { selected_prg_bank, nametable_upper, chr_ram } = state else {
            return Err("savestate mapper mismatch".into());
        };
        if *selected_prg_bank > 7 {
            return Err("invalid AxROM bank state".into());
        }
        self.selected_prg_bank = *selected_prg_bank;
        self.nametable_upper = *nametable_upper;
        self.chr_ram = *chr_ram;
        Ok(())
    }
}

pub struct Gxrom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    selected_prg_bank: u8,
    selected_chr_bank: u8,
    mirroring: Mirroring,
}

impl Gxrom {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Result<Self, String> {
        if !matches!(prg_rom.len(), 0x8000 | 0x10000 | 0x20000) {
            return Err("GxROM requires 32, 64, or 128 KiB PRG ROM".into());
        }
        if !matches!(chr_rom.len(), 0x2000 | 0x4000 | 0x8000) {
            return Err("GxROM requires 8, 16, or 32 KiB CHR ROM".into());
        }
        Ok(Self { prg_rom, chr_rom, selected_prg_bank: 0, selected_chr_bank: 0, mirroring })
    }
}

impl Mapper for Gxrom {
    fn read(&self, address: u16) -> u8 {
        if address < 0x8000 { return 0; }
        let bank = self.selected_prg_bank as usize % (self.prg_rom.len() / 0x8000);
        self.prg_rom[bank * 0x8000 + (address as usize & 0x7fff)]
    }

    fn write(&mut self, address: u16, data: u8) {
        if address >= 0x8000 {
            let effective = data & self.read(address);
            self.selected_prg_bank = (effective >> 4) & 3;
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
        Ok(MapperState::Gxrom {
            selected_prg_bank: self.selected_prg_bank,
            selected_chr_bank: self.selected_chr_bank,
        })
    }

    fn load_state(&mut self, state: &MapperState) -> Result<(), String> {
        let MapperState::Gxrom { selected_prg_bank, selected_chr_bank } = state else {
            return Err("savestate mapper mismatch".into());
        };
        if *selected_prg_bank > 3 || *selected_chr_bank > 3 {
            return Err("invalid GxROM bank state".into());
        }
        self.selected_prg_bank = *selected_prg_bank;
        self.selected_chr_bank = *selected_chr_bank;
        Ok(())
    }
}

/// MMC1B core for ordinary boards with up to 256 KiB PRG, 128 KiB CHR ROM
/// (or 8 KiB CHR RAM), and at most one 8 KiB PRG RAM bank.
/// Outer banking and SNROM's additional CHR-controlled RAM gate are not modeled.
// also yeah i did kinda ai this one but dw it works, just tested battery
pub struct Mmc1 {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
    chr_is_ram: bool,
    prg_ram: Vec<u8>,
    battery_backed: bool,
    four_screen: bool,
    shift: u8,
    write_count: u8,
    control: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,
    last_write_cycle: Option<u64>,
}

impl Mmc1 {
    pub fn new(
        prg_rom: Vec<u8>, chr_rom: Vec<u8>, prg_ram_size: usize,
        battery_backed: bool, mirroring: Mirroring,
    ) -> Result<Self, String> {
        let size = prg_rom.len();
        if !(0x4000..=0x40000).contains(&size) || !size.is_power_of_two() {
            return Err("MMC1B requires power-of-two PRG ROM from 16 to 256 KiB; outer banking is unsupported".into());
        }
        let chr_size = chr_rom.len();
        if chr_size != 0 && (!(0x2000..=0x20000).contains(&chr_size) || !chr_size.is_power_of_two()) {
            return Err("MMC1B requires power-of-two CHR ROM from 8 to 128 KiB, or 8 KiB CHR RAM".into());
        }
        if !matches!(prg_ram_size, 0 | 0x2000) || (battery_backed && prg_ram_size == 0) {
            return Err("MMC1B supports no PRG RAM or one 8 KiB RAM bank".into());
        }
        let chr_is_ram = chr_rom.is_empty();
        Ok(Self {
            prg_rom,
            chr: if chr_is_ram { vec![0; 0x2000] } else { chr_rom },
            chr_is_ram,
            prg_ram: vec![0; prg_ram_size],
            battery_backed,
            four_screen: mirroring == Mirroring::FourScreen,
            // Deterministic startup: bank 0 / last bank, 8 KiB CHR, lower page.
            shift: 0, write_count: 0, control: 0x0c,
            chr_bank0: 0, chr_bank1: 0, prg_bank: 0, last_write_cycle: None,
        })
    }

    fn chr_index(&self, address: u16) -> usize {
        let address = address as usize & 0x1fff;
        let bank = if self.control & 0x10 == 0 {
            (self.chr_bank0 as usize & !1) + (address >> 12)
        } else if address < 0x1000 { self.chr_bank0 as usize }
        else { self.chr_bank1 as usize };
        (bank * 0x1000 + (address & 0xfff)) % self.chr.len()
    }

    fn register_write(&mut self, address: u16, data: u8, consecutive: bool) {
        // Bit 7 reset remains effective even on the second write of an RMW.
        if data & 0x80 != 0 {
            self.shift = 0;
            self.write_count = 0;
            self.control |= 0x0c;
            return;
        }
        if consecutive { return; }
        self.shift |= (data & 1) << self.write_count;
        self.write_count += 1;
        if self.write_count == 5 {
            match address {
                0x8000..=0x9fff => self.control = self.shift,
                0xa000..=0xbfff => self.chr_bank0 = self.shift,
                0xc000..=0xdfff => self.chr_bank1 = self.shift,
                0xe000..=0xffff => self.prg_bank = self.shift,
                _ => unreachable!(),
            }
            self.shift = 0;
            self.write_count = 0;
        }
    }

    fn write_inner(&mut self, address: u16, data: u8, consecutive: bool) {
        match address {
            0x6000..=0x7fff if !self.prg_ram.is_empty() && self.prg_bank & 0x10 == 0 => {
                self.prg_ram[address as usize - 0x6000] = data;
            }
            0x8000..=0xffff => self.register_write(address, data, consecutive),
            _ => {}
        }
    }
}

impl Mapper for Mmc1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x6000..=0x7fff if !self.prg_ram.is_empty() && self.prg_bank & 0x10 == 0 => {
                self.prg_ram[address as usize - 0x6000]
            }
            0x8000..=0xffff => {
                let banks = self.prg_rom.len() / 0x4000;
                let selected = (self.prg_bank & 0x0f) as usize;
                let upper = address >= 0xc000;
                let bank = match (self.control >> 2) & 3 {
                    0 | 1 => (selected & !1) + usize::from(upper),
                    2 => if upper { selected } else { 0 },
                    _ => if upper { banks - 1 } else { selected },
                } % banks;
                self.prg_rom[bank * 0x4000 + (address as usize & 0x3fff)]
            }
            _ => 0, // Consistent with the emulator's current unmapped-bus policy.
        }
    }

    // Untimed calls are for direct setup/debug access, not CPU instruction execution.
    fn write(&mut self, address: u16, data: u8) {
        self.last_write_cycle = None;
        self.write_inner(address, data, false);
    }

    fn write_timed(&mut self, address: u16, data: u8, cpu_cycle: u64) {
        let consecutive = self.last_write_cycle
            .is_some_and(|last| last.wrapping_add(1) == cpu_cycle);
        // Track ignored writes too, so an entire run after its first write is ignored.
        self.last_write_cycle = Some(cpu_cycle);
        self.write_inner(address, data, consecutive);
    }

    fn reset_write_timing(&mut self) { self.last_write_cycle = None; }

    fn ppu_read(&self, address: u16) -> u8 { self.chr[self.chr_index(address)] }
    fn ppu_write(&mut self, address: u16, data: u8) {
        if self.chr_is_ram {
            let index = self.chr_index(address);
            self.chr[index] = data;
        }
    }
    fn mirroring(&self) -> Mirroring {
        if self.four_screen { return Mirroring::FourScreen; }
        match self.control & 3 {
            0 => Mirroring::SingleScreenLower,
            1 => Mirroring::SingleScreenUpper,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        }
    }
    fn persistent_ram(&self) -> &[u8] {
        if self.battery_backed { &self.prg_ram } else { &[] }
    }
    fn load_persistent_ram(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != self.persistent_ram().len() {
            return Err("battery save size does not match MMC1 PRG NVRAM".into());
        }
        if self.battery_backed { self.prg_ram.copy_from_slice(data); }
        Ok(())
    }
    fn save_state(&self) -> Result<MapperState, String> {
        Ok(MapperState::Mmc1 {
            shift: self.shift, write_count: self.write_count, control: self.control,
            chr_bank0: self.chr_bank0, chr_bank1: self.chr_bank1, prg_bank: self.prg_bank,
            last_write_cycle: self.last_write_cycle,
            prg_ram: self.prg_ram.clone(),
            chr_ram: if self.chr_is_ram { self.chr.clone() } else { Vec::new() },
        })
    }
    fn load_state(&mut self, state: &MapperState) -> Result<(), String> {
        let MapperState::Mmc1 { shift, write_count, control, chr_bank0, chr_bank1,
            prg_bank, last_write_cycle, prg_ram, chr_ram } = state else {
            return Err("savestate mapper mismatch".into());
        };
        if *write_count > 4 || (*shift as u16) >= (1u16 << *write_count)
            || [*control, *chr_bank0, *chr_bank1, *prg_bank].iter().any(|v| *v > 31)
            || prg_ram.len() != self.prg_ram.len()
            || chr_ram.len() != if self.chr_is_ram { 0x2000 } else { 0 }
        {
            return Err("invalid MMC1 savestate".into());
        }
        self.shift = *shift;
        self.write_count = *write_count;
        self.control = *control;
        self.chr_bank0 = *chr_bank0;
        self.chr_bank1 = *chr_bank1;
        self.prg_bank = *prg_bank;
        self.last_write_cycle = *last_write_cycle;
        self.prg_ram.copy_from_slice(prg_ram);
        if self.chr_is_ram { self.chr.copy_from_slice(chr_ram); }
        Ok(())
    }
}
