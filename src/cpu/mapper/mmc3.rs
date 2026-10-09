use super::Mapper;
use crate::cartridge::Mirroring;
use crate::savestate::MapperState;
use std::cell::Cell;

/// IRQ state uses interior mutability because CPU-facing PPU reads take &self.
/// There is still only one cartridge mapper and one IRQ counter.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Mmc3IrqState {
    pub(crate) latch: u8,
    pub(crate) counter: u8,
    pub(crate) reload: bool,
    pub(crate) enabled: bool,
    pub(crate) pending: bool,
    pub(crate) a12_high: bool,
    pub(crate) low_m2_edges: u8,
}

/// Ordinary Sharp MMC3 (mapper 4, submapper 0). No MMC6 or outer banking.
/// Register and RAM power-on values are deterministic emulator defaults.
pub struct Mmc3 {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
    chr_is_ram: bool,
    prg_ram: Vec<u8>,
    battery_backed: bool,
    four_screen: bool,
    horizontal: bool,
    bank_select: u8,
    banks: [u8; 8],
    ram_control: u8,
    irq: Cell<Mmc3IrqState>,
}

impl Mmc3 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, prg_ram_size: usize,
        battery_backed: bool, mirroring: Mirroring) -> Result<Self, String>
    {
        if !prg_rom.len().is_power_of_two() || !(0x8000..=0x80000).contains(&prg_rom.len()) {
            return Err("MMC3 requires power-of-two PRG ROM sizes from 32 to 512 KiB".into());
        }
        if !chr_rom.is_empty() && (!chr_rom.len().is_power_of_two()
            || !(0x2000..=0x40000).contains(&chr_rom.len())) {
            return Err("MMC3 requires 8–256 KiB power-of-two CHR ROM or 8 KiB CHR RAM".into());
        }
        if !matches!(prg_ram_size, 0 | 0x2000) || (battery_backed && prg_ram_size == 0) {
            return Err("MMC3 supports absent PRG RAM or one 8 KiB volatile/nonvolatile bank".into());
        }
        if !matches!(mirroring, Mirroring::Horizontal | Mirroring::Vertical | Mirroring::FourScreen) {
            return Err("unsupported MMC3 nametable wiring".into());
        }
        let chr_is_ram = chr_rom.is_empty();
        Ok(Self {
            prg_rom, chr: if chr_is_ram { vec![0; 0x2000] } else { chr_rom },
            chr_is_ram, prg_ram: vec![0; prg_ram_size], battery_backed,
            four_screen: mirroring == Mirroring::FourScreen,
            horizontal: mirroring == Mirroring::Horizontal,
            bank_select: 0, banks: [0; 8], ram_control: 0x80,
            irq: Cell::new(Mmc3IrqState::default()),
        })
    }

    fn chr_index(&self, address: u16) -> usize {
        let address = (address as usize & 0x1fff)
            ^ if self.bank_select & 0x80 != 0 { 0x1000 } else { 0 };
        let slot = address / 0x400;
        let bank = match slot {
            0..=1 => (self.banks[0] & 0xfe) as usize + slot,
            2..=3 => (self.banks[1] & 0xfe) as usize + slot - 2,
            _ => self.banks[slot - 2] as usize,
        };
        (bank * 0x400 + (address & 0x3ff)) % self.chr.len()
    }
}

impl Mapper for Mmc3 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x6000..=0x7fff if !self.prg_ram.is_empty() && self.ram_control & 0x80 != 0 =>
                self.prg_ram[address as usize & 0x1fff],
            0x8000..=0xffff => {
                let last = self.prg_rom.len() / 0x2000 - 1;
                let mode = self.bank_select & 0x40 != 0;
                let bank = match (address >> 13) & 3 {
                    0 => if mode { last - 1 } else { self.banks[6] as usize },
                    1 => self.banks[7] as usize,
                    2 => if mode { self.banks[6] as usize } else { last - 1 },
                    _ => last,
                };
                self.prg_rom[(bank * 0x2000 + (address as usize & 0x1fff)) % self.prg_rom.len()]
            }
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if (0x6000..=0x7fff).contains(&address) {
            if !self.prg_ram.is_empty() && self.ram_control & 0xc0 == 0x80 {
                self.prg_ram[address as usize & 0x1fff] = data;
            }
            return;
        }
        if address < 0x8000 { return; }
        let mut irq = self.irq.get();
        match address & 0xe001 {
            0x8000 => self.bank_select = data & 0xc7,
            0x8001 => {
                let index = (self.bank_select & 7) as usize;
                self.banks[index] = data & match index { 0 | 1 => 0xfe, 6 | 7 => 0x3f, _ => 0xff };
            }
            0xa000 => if !self.four_screen { self.horizontal = data & 1 != 0; },
            0xa001 => self.ram_control = data & 0xc0,
            0xc000 => irq.latch = data,
            0xc001 => { irq.counter = 0; irq.reload = true; }
            0xe000 => { irq.enabled = false; irq.pending = false; }
            0xe001 => irq.enabled = true,
            _ => unreachable!(),
        }
        self.irq.set(irq);
    }

    fn ppu_read(&self, address: u16) -> u8 { self.chr[self.chr_index(address)] }
    fn ppu_write(&mut self, address: u16, data: u8) {
        if self.chr_is_ram {
            let index = self.chr_index(address);
            self.chr[index] = data;
        }
    }
    fn mirroring(&self) -> Mirroring {
        if self.four_screen { Mirroring::FourScreen }
        else if self.horizontal { Mirroring::Horizontal } else { Mirroring::Vertical }
    }
    fn irq_pending(&self) -> bool { self.irq.get().pending }
    fn observe_ppu_address(&self, address: u16) {
        let mut irq = self.irq.get();
        let high = address & 0x1000 != 0;
        if high && !irq.a12_high {
            if irq.low_m2_edges >= 3 {
                if irq.counter == 0 || irq.reload { irq.counter = irq.latch; }
                else { irq.counter -= 1; }
                irq.reload = false;
                if irq.counter == 0 && irq.enabled { irq.pending = true; }
            }
            irq.low_m2_edges = 0;
        }
        irq.a12_high = high;
        self.irq.set(irq);
    }
    fn clock_m2_falling(&self) {
        let mut irq = self.irq.get();
        irq.low_m2_edges = if irq.a12_high { 0 } else { (irq.low_m2_edges + 1).min(3) };
        self.irq.set(irq);
    }
    fn persistent_ram(&self) -> &[u8] {
        if self.battery_backed { &self.prg_ram } else { &[] }
    }
    fn load_persistent_ram(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != self.persistent_ram().len() { return Err("MMC3 battery save size mismatch".into()); }
        if !data.is_empty() { self.prg_ram.copy_from_slice(data); }
        Ok(())
    }
    fn save_state(&self) -> Result<MapperState, String> {
        Ok(MapperState::Mmc3 {
            bank_select: self.bank_select, banks: self.banks, horizontal: self.horizontal,
            ram_control: self.ram_control, irq: self.irq.get(), prg_ram: self.prg_ram.clone(),
            chr_ram: if self.chr_is_ram { self.chr.clone() } else { Vec::new() },
        })
    }
    fn load_state(&mut self, state: &MapperState) -> Result<(), String> {
        let MapperState::Mmc3 { bank_select, banks, horizontal, ram_control, irq, prg_ram, chr_ram } = state
            else { return Err("savestate mapper mismatch".into()); };
        if bank_select & !0xc7 != 0 || banks[0] & 1 != 0 || banks[1] & 1 != 0
            || banks[6] > 63 || banks[7] > 63 || ram_control & !0xc0 != 0
            || irq.low_m2_edges > 3 || (irq.a12_high && irq.low_m2_edges != 0)
            || (irq.pending && !irq.enabled) || (irq.reload && irq.counter != 0)
            || prg_ram.len() != self.prg_ram.len()
            || chr_ram.len() != if self.chr_is_ram { 0x2000 } else { 0 }
            || (self.four_screen && *horizontal != self.horizontal) {
            return Err("invalid MMC3 savestate".into());
        }
        self.bank_select = *bank_select;
        self.banks = *banks;
        self.horizontal = *horizontal;
        self.ram_control = *ram_control;
        self.irq.set(*irq);
        self.prg_ram.copy_from_slice(prg_ram);
        if self.chr_is_ram { self.chr.copy_from_slice(chr_ram); }
        Ok(())
    }
}
