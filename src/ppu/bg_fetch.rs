use super::PPU;

impl PPU {
    // which tile should be drawn
    fn fetch_nametable_byte(&self) -> u8 {
        let addr = 0x2000 | (self.v.get() & 0x0FFF);
        self.read_vram(addr)
    }

    // which of the 4 background palletes should be used
    fn fetch_attribute_byte(&self) -> u8 {
        let v = self.v.get();
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        self.read_vram(addr)
    }

    // given attribute byte, what 2 bits apply to tile
    fn attribute_quadrant_bits(&self) -> u8 {
        let v = self.v.get();
        let shift = ((v >> 4) & 0b100) | (v & 0b010);
        (self.at_latch >> shift) & 0b11
    }

    // finding the pixel shapes for a tiles current row
    fn fetch_pattern_low(&self) -> u8 {
        let fine_y = (self.v.get() >> 12) & 0x07;
        let addr = self.bg_pattern_table_addr + (self.nt_latch as u16 * 16) + fine_y;
        self.read_vram(addr)
    }

    fn fetch_pattern_high(&self) -> u8 {
        let fine_y = (self.v.get() >> 12) & 0x07;
        let addr = self.bg_pattern_table_addr + (self.nt_latch as u16 * 16) + fine_y + 8;
        self.read_vram(addr)
    }
}