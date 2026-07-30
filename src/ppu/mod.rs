use std::cell::Cell;

pub struct PPU {
    status: Cell<u8>,
    // ts is just all temporary
}

impl PPU {
    pub fn new() -> Self {
        PPU { status: Cell::new(0) }
    }

    pub fn read_register(&self, reg: u16) -> u8 {
        match reg {
            0x0002 => {
                let value = self.status.get();
                self.status.set(value & 0b0111_1111);
                value
            }
            _ => 0,
        }
    }

    pub fn write_register(&mut self, reg: u16, data: u8) {
    }
}