use super::PPU;
use super::palette::NES_PALETTE;

impl PPU {
    // runs one per dot, pulling bg pixel shape and palette from shift register, eventually being written to framebuffer
    fn output_pixel(&mut self, dot: u16) {
        let bit = 15 - self.x as u16;
        let lo = (self.bg_shift_lo >> bit) & 1;
        let hi = (self.bg_shift_hi >> bit) & 1;
        let bg_pixel = ((hi << 1) | lo) as u8;

        let attr_bit = 7 - self.x;
        let attr_lo = (self.attr_shift_lo >> attr_bit) & 1;
        let attr_hi = (self.attr_shift_hi >> attr_bit) & 1;
        let bg_palette = (attr_hi << 1) | attr_lo;

        let (sprite_pixel, sprite_palette, sprite_priority, sprite_is_zero) =
            self.current_sprite_pixel(dot);

        let screen_x = (dot - 1) as u16;
        let (final_pixel, is_sprite, palette_select, hit) = self.priority_mux(
            bg_pixel, bg_palette,
            sprite_pixel, sprite_palette, sprite_priority, sprite_is_zero,
            self.bg_rendering, self.sprite_rendering,
            screen_x,
        );

        if hit {
            self.sprite0_hit.set(true);
        }

        let s_bit: u16 = if is_sprite { 1 } else { 0 };
        let palette_addr = 0x3F00 | (s_bit << 4) | ((palette_select as u16) << 2) | final_pixel as u16;
        let color_index = self.read_vram(palette_addr) & 0x3F;
        let color = NES_PALETTE[color_index as usize];

        let x = (dot - 4) as usize;
        let y = self.scanline as usize;
        self.framebuffer[y * 256 + x] = color;
    }

    // table math for output_pixel and other things
    fn priority_mux(
        &self,
        bg_pixel: u8,
        bg_palette: u8,
        sprite_pixel: u8,
        sprite_palette: u8,
        sprite_priority: u8,
        sprite_is_zero: bool,
        bg_enabled: bool,
        sprite_enabled: bool,
        screen_x: u16,
    ) -> (u8, bool, u8, bool) {
        let bg_pixel = if bg_enabled { bg_pixel } else { 0 };
        let sprite_pixel = if sprite_enabled { sprite_pixel } else { 0 };

        let (final_pixel, is_sprite, palette_select) = match (bg_pixel, sprite_pixel) {
            (0, 0) => (0u8, false, 0u8),
            (0, s) => (s, true, sprite_palette),
            (b, 0) => (b, false, bg_palette),
            (b, s) => {
                if sprite_priority == 0 { (s, true, sprite_palette) } else { (b, false, bg_palette) }
            }
        };

        let left_clip_active = screen_x < 8
            && (!self.show_bg_in_leftmost || !self.show_sprites_in_leftmost);

        let hit = bg_pixel != 0
            && sprite_pixel != 0
            && sprite_is_zero
            && bg_enabled
            && sprite_enabled
            && screen_x != 255
            && !left_clip_active;

        (final_pixel, is_sprite, palette_select, hit)
    }

    pub fn run_render_cycle(&mut self) {
        if !(self.bg_rendering || self.sprite_rendering) {
            return;
        }

        let dot = self.dot;

        // idle on cycle 0
        if dot == 0 {
            return;
        }

        // background fetch, shift, output dots 1-256 and 321-336
        if (dot >= 1 && dot <= 256) || (dot >= 321 && dot <= 336) {
            self.bg_shift_lo <<= 1;
            self.bg_shift_hi <<= 1;
            self.attr_shift_lo = (self.attr_shift_lo << 1) | self.attr_latch_lo as u8;
            self.attr_shift_hi = (self.attr_shift_hi << 1) | self.attr_latch_hi as u8;

            if dot >= 4 && dot <= 259 && self.scanline >= 0 {
                self.output_pixel(dot);
            }

            match (dot - 1) % 8 {
                0 => {
                    self.nt_latch = self.fetch_nametable_byte();
                }
                2 => {
                    self.at_latch = self.fetch_attribute_byte();
                }
                4 => {
                    self.bg_lo_latch = self.fetch_pattern_low();
                }
                6 => {
                    self.bg_hi_latch = self.fetch_pattern_high();
                }
                7 => {
                    self.bg_shift_lo = (self.bg_shift_lo & 0xFF00) | self.bg_lo_latch as u16;
                    self.bg_shift_hi = (self.bg_shift_hi & 0xFF00) | self.bg_hi_latch as u16;

                    let quadrant = self.attribute_quadrant_bits();
                    self.attr_latch_lo = quadrant & 1 != 0;
                    self.attr_latch_hi = quadrant & 2 != 0;

                    self.increment_coarse_x();
                }
                _ => {}
            }
        }

        // vertical scvroll increment on dot 256
        if dot == 256 {
            self.increment_vert_v();
        }

        // horizontal scroll reload on dot 257
        if dot == 257 {
            self.copy_horizontal_bits();
        }

        // sprite tile fetches for next scanline dots 257-320
        if dot >= 257 && dot <= 320 {
            self.sprite_fetch_cycle(dot);
        }

        // --- pre-render only, dots 280-304: vertical scroll reload ---
        if self.scanline == -1 && dot >= 280 && dot <= 304 {
            self.copy_vertical_bits();
        }

        // dots 337-340 are weird, not topuching thnat
    }
}