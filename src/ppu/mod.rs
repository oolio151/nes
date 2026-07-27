pub struct PPU {

}

impl PPU {
    pub fn new() -> Self {
        PPU {}
    }

    pub fn read_register(&mut self, reg: u16) -> u8 {
        0
    }

    pub fn write_register(&mut self, reg: u16, data: u8) {
    }
}