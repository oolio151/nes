// scroll functions
use super::PPU;

impl PPU{
    pub fn increment_coarse_x(&mut self) {
        let mut v = self.v.get();
        if v & 0x001F == 31 {
            v &= !0x001F;
            v ^= 0x0400;
        } else {
            v += 1;
        }
        self.v.set(v);
    }

    pub fn increment_vert_v(&mut self) {
        let mut v = self.v.get();
        if v & 0x7000 != 0x7000 {
            v += 0x1000;
        } else {
            v &= !0x7000;
            let mut coarse_y = (v & 0x03E0) >> 5;
            if coarse_y == 29 {
                coarse_y = 0;
                v ^= 0x0800;
            } else if coarse_y == 31 {
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }
            v = (v & !0x03E0) | (coarse_y << 5);
        }
        self.v.set(v);
    }

    pub fn copy_horizontal_bits(&mut self) {
        let v = self.v.get();
        let t = self.t;
        self.v.set((v & !0x041F) | (t & 0x041F));
    }

    pub fn copy_vertical_bits(&mut self) {
        let v = self.v.get();
        let t = self.t;
        self.v.set((v & !0x7BE0) | (t & 0x7BE0));
    }
}