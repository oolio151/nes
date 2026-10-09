use super::PPU;
use crate::cpu::mapper::Mapper;

impl PPU {
    // which tile should be drawn
    pub fn begin_nametable_fetch(&self, mapper: &dyn Mapper) {
        let addr = 0x2000 | (self.v.get() & 0x0FFF);
        self.drive_bus(addr, mapper);
    }

    // which of the 4 background palletes should be used
    pub fn begin_attribute_fetch(&self, mapper: &dyn Mapper) {
        let v = self.v.get();
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        self.drive_bus(addr, mapper);
    }

    // given attribute byte, what 2 bits apply to tile
    pub fn attribute_quadrant_bits(&self) -> u8 {
        let v = self.v.get();
        let shift = ((v >> 4) & 0b100) | (v & 0b010);
        (self.at_latch >> shift) & 0b11
    }

    // finding the pixel shapes for a tiles current row
    pub fn begin_pattern_low_fetch(&self, mapper: &dyn Mapper) {
        let fine_y = (self.v.get() >> 12) & 0x07;
        let addr = self.bg_pattern_table_addr + (self.nt_latch as u16 * 16) + fine_y;
        self.drive_bus(addr, mapper);
    }

    pub fn begin_pattern_high_fetch(&self, mapper: &dyn Mapper) {
        let fine_y = (self.v.get() >> 12) & 0x07;
        let addr = self.bg_pattern_table_addr + (self.nt_latch as u16 * 16) + fine_y + 8;
        self.drive_bus(addr, mapper);
    }
}