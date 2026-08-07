use std::cell::Cell;

// ntsc because pal is for chuds
pub enum PPURegister {
    PPUCTRL, // W
    PPUMASK, // W
    PPUSTATUS, // R
    OAMADDR, // W
    OAMDATA, // RW
    PPUSCROLL, // Wx2
    PPUADDR, // Wx2
    PPUDATA, // RW
    OAMDMA, // W
}


pub struct PPU {
    registers: [u8; 8],
    scanline: i16, // -1 indicates prerender, goes up to 260
    dot: u16, // 0 through 340
    ood_frame: bool, // weird behavior shit

}

impl PPU {
    pub fn new() -> Self {
        
    }

    pub fn read_register(&self, reg: u16) -> u8 {

    }

    pub fn write_register(&mut self, reg: u16, data: u8) {
    }

    pub fn tick(&mut self) {

    }
}