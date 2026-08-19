use super::PPU;
use super::palette::NES_PALETTE;
use super::sprite::current_sprite_pixel; // will add in sprite.rs

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

        let (final_pixel, is_sprite, palette_select, hit) = self.priority_mux(
            bg_pixel, bg_palette,
            sprite_pixel, sprite_palette, sprite_priority, sprite_is_zero,
            self.bg_rendering, self.sprite_rendering,
        );

        if hit {
            self.sprite0_hit.set(true);
        }

        let s_bit: u16 = if is_sprite { 1 } else { 0 };
        let palette_addr = 0x3F00 | (s_bit << 4) | ((palette_select as u16) << 2) | final_pixel as u16;
        let color_index = self.read_vram(palette_addr) & 0x3F;
        let color = NES_PALETTE[color_index as usize];

        let x = (dot - 1) as usize;
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

        let hit = bg_pixel != 0 && sprite_pixel != 0 && sprite_is_zero && bg_enabled && sprite_enabled;

        (final_pixel, is_sprite, palette_select, hit)
    }
}