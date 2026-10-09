use super::PPU;
use crate::cpu::mapper::Mapper;

impl PPU {
    fn sprite_pattern_address(&self, sprite_num: usize) -> u16 {
        let base = sprite_num * 4;
        let y = self.oam2[base];
        let tile = self.oam2[base + 1];
        let attr = self.oam2[base + 2];
        let height = if self.sprites_8x16 { 16 } else { 8 };
        // Empty slots still fetch tile $FF. Mask the row before flipping.
        let mut row = ((self.scanline + 1) as u16).wrapping_sub(y as u16) & (height - 1);
        if attr & 0x80 != 0 { row ^= height - 1; }
        let (table, tile) = if self.sprites_8x16 {
            (((tile as u16 & 1) << 12), (tile & 0xfe) as u16 + row / 8)
        } else { (self.sprite_pattern_table_addr, tile as u16) };
        table + tile * 16 + (row & 7)
    }

    pub fn sprite_fetch_cycle(&mut self, dot: u16, mapper: &dyn Mapper) {
        let offset = dot - 257;
        let slot = (offset / 8) as usize;
        match offset % 8 {
            0 => {
                self.sprite_x[slot] = self.oam2[slot * 4 + 3];
                self.sprite_attr[slot] = self.oam2[slot * 4 + 2];
                self.begin_nametable_fetch(mapper);
            }
            2 => self.begin_nametable_fetch(mapper),
            1 | 3 => { let _ = self.sample_bus(mapper); }
            4 => self.drive_bus(self.sprite_pattern_address(slot), mapper),
            6 => self.drive_bus(self.sprite_pattern_address(slot) + 8, mapper),
            phase => {
                let mut byte = self.sample_bus(mapper);
                if self.sprite_attr[slot] & 0x40 != 0 { byte = byte.reverse_bits(); }
                if self.oam2[slot * 4] == 0xff { byte = 0; }
                if phase == 5 { self.sprite_pattern_lo[slot] = byte; }
                else { self.sprite_pattern_hi[slot] = byte; }
            }
        }
    }

    // finds highest priority sprite covering current and returns data about it
    pub fn current_sprite_pixel(&mut self, dot: u16) -> (u8, u8, u8, bool) {
        let screen_x = (dot - 1) as u16;

        for i in 0..8 {
            let sx = self.sprite_x[i] as u16;
            if screen_x >= sx && screen_x < sx + 8 {
                let col = (screen_x - sx) as u8;
                let bit = 7 - col;

                let lo = (self.sprite_pattern_lo[i] >> bit) & 1;
                let hi = (self.sprite_pattern_hi[i] >> bit) & 1;
                let pixel = (hi << 1) | lo;

                if pixel != 0 {
                    let attr = self.sprite_attr[i];
                    let palette = attr & 0b11;
                    let priority = (attr >> 5) & 1;
                    let is_zero = i == 0 && self.sprite_zero_current;
                    return (pixel, palette, priority, is_zero);
                }
            }
        }

        (0, 0, 0, false)
    }
}