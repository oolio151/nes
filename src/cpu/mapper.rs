pub trait Mapper {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);

    // used for certain mappers like mmc5 and 3
    fn notify_ppu_address(&mut self, _address: u16) {}
}

#[allow(dead_code)]
pub struct Nrom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: [u8; 0x2000],
}

impl Nrom {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_ram: [0; 0x2000],
        }
    }
}

impl Mapper for Nrom {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => self.prg_ram[(address - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let mask = if self.prg_rom.len() == 0x4000 { 0x3FFF } else { 0x7FFF };
                self.prg_rom[(address as usize - 0x8000) & mask]
            }
            _ => 0,
        }
    }
    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x6000..=0x7FFF => self.prg_ram[(address - 0x6000) as usize] = data,
            0x8000..=0xFFFF => { /* ROM — real hardware ignores writes here */ }
            _ => {}
        }
    }


    
}