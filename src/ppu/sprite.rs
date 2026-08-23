use super::PPU;

impl PPU {
    // takes one of the sprites from oam2 and prepares it to be drawn on next scanline
    fn load_sprite_from_secondary_oam(&mut self, sprite_num: usize) {
        let base = sprite_num * 4;
        let y = self.oam2[base];
        let tile = self.oam2[base + 1];
        let attr = self.oam2[base + 2];
        let x = self.oam2[base + 3];

        self.sprite_x[sprite_num] = x;
        self.sprite_attr[sprite_num] = attr;

        if y == 0xFF {
            // unused slot, transparent
            self.sprite_pattern_lo[sprite_num] = 0;
            self.sprite_pattern_hi[sprite_num] = 0;
            return;
        }

        let sprite_height: u16 = if self.sprites_8x16 { 16 } else { 8 };
        let flip_v = attr & 0b1000_0000 != 0;
        let flip_h = attr & 0b0100_0000 != 0;

        let target_scanline = (self.scanline + 1) as u16;
        let mut row = target_scanline.wrapping_sub(y as u16);
        if flip_v {
            row = sprite_height - 1 - row;
        }

        let (table_base, tile_index) = if self.sprites_8x16 {
            let table = if tile & 1 != 0 { 0x1000 } else { 0x0000 };
            let base_index = tile & 0xFE;
            let index = if row >= 8 { base_index + 1 } else { base_index };
            (table, index)
        } else {
            (self.sprite_pattern_table_addr, tile)
        };

        let row_in_tile = row % 8;
        let tile_addr = table_base + (tile_index as u16 * 16);
        let mut lo = self.read_vram(tile_addr + row_in_tile);
        let mut hi = self.read_vram(tile_addr + row_in_tile + 8);

        if flip_h {
            lo = lo.reverse_bits();
            hi = hi.reverse_bits();
        }

        self.sprite_pattern_lo[sprite_num] = lo;
        self.sprite_pattern_hi[sprite_num] = hi;
    }

    // on dots 257-320, dispatches sprite tile loading, one load per 8-dot window.
    pub fn sprite_fetch_cycle(&mut self, dot: u16) {
        let offset = dot - 257;
        let sprite_num = (offset / 8) as usize;

        // Load once per sprite, at the end of its 8-dot window (matches when
        // pattern-high would be ready in the real 2-dot-per-fetch schedule).
        if offset % 8 == 7 && sprite_num < 8 {
            self.load_sprite_from_secondary_oam(sprite_num);
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