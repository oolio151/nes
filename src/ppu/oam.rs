use super::PPU;

impl PPU {
    // for secondary oam (oam2), finds sprites taht are within the y range for the next scanline, and copies them to oam2
    pub fn evaluate_sprites(&mut self) {
        let sprite_height = if self.sprites_8x16 { 16 } else { 8 };
        let target_scanline = self.scanline + 1;

        self.sprite_zero_next = false; // reset before scanning

        let mut found = 0;
        for i in 0..64 {
            let y = self.oam[i * 4] as i16;
            if target_scanline >= y && target_scanline < y + sprite_height {
                if found < 8 {
                    let base = found * 4;
                    self.oam2[base..base + 4].copy_from_slice(&self.oam[i * 4..i * 4 + 4]);

                    if i == 0 {
                        self.sprite_zero_next = true;
                    }
                } else {
                    self.sprite_overflow.set(true);
                    break;
                }
                found += 1;
            }
        }
    }

}