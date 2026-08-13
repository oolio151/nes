use std::cell::Cell;
use crate::cartridge::Mirroring;
use palette::NES_PALETTE;

pub mod palette;

// ntsc because pal is for chuds
// https://www.nesdev.org/wiki/PPU_programmer_reference
pub enum PPURegister {
    PPUCTRL, // W
    PPUMASK, // W
    PPUSTATUS, // R
    OAMADDR, // W
    OAMDATA, // RW
    PPUSCROLL, // Wx2
    PPUADDR, // Wx2
    PPUDATA, // RW
}


pub struct PPU {

    // ===========================
    // PPU REGISTER SHIT GOES HERE
    // ===========================

    // PPUCTRL
    nmi_enable: bool,
    // ppu master slave is unused
    sprites_8x16: bool, // 8x8 if false, 8x16 if true
    bg_pattern_table_addr: u16, // can either be 0x0000 or 0x1000
    sprite_pattern_table_addr: u16, // also can be 0x0000 or 0x1000, ignored in 8x16 mode
    vram_addr_inc: u8, // vram address increment per CPU r/w of PPUDATA, either 1 or 32

    // PPUMASK
    emphasize_blue: bool,
    emphasize_green: bool,
    emphasize_red: bool,
    sprite_rendering: bool,
    bg_rendering: bool,
    show_sprites_in_leftmost: bool, // show spritesi nthe leftmost 8 pixels of the screen if true
    show_bg_in_leftmost: bool, // same, but for the background
    grayscale: bool, // take a guess bro

    // PPUSTATUS
    vblank_flag: Cell<bool>, // set at start of vblank, reading or waiting until dot 1 of prerender scanline will clear
    sprite0_hit: Cell<bool>, // checks collision btwn sprite 0 and background, read wiki'
    sprite_overflow: Cell<bool>, // true if there are >8 sprites on a scanline, this is inaccurate, read the wiki

    // OAM STUFF
    oam_addr: u8,
    oam: [u8; 256],
    oam2: [u8; 32], // secondary oam, holds 8 sprites max

    // INTERNAL REGISTERS
    v: Cell<u16>, // used for scroll pos while rendering, otherwise is current vram address, aka PPUADDR
    t: u16, // used for coarse-x scroll for next scanline and start y for screen, otherwise holds scroll or vram address before transferring to v
    x: u8, // fine-x pos of the current scroll, used for rendering stuff
    w: Cell<bool>, // toggles on write to PPUSCROLL || PPUADDR, indicating whether this is first or second write. clears on read of PPUSTATUS

    // PPUDATA
    read_buffer: Cell<u8>, // updates on every PPUDATA read after previous contents returned to CPU, delaying PPUDATA read by 1

    // some weird behavior of the chip
    io_latch: Cell<u8>, // writes to any register or reads to readable ones fill the latch with whatever bytes were wrote/read

    // THE OTHER CAVELIERS
    scanline: i16, // -1 indicates prerender, goes up to 260
    /*
    scanline ranges
    -1       pre-render scanline
    0-239    visible scanlines
    240      post-render scanline, aka idle ppu
    241-260  vblank scanlines
    
     */
    dot: u16, // 0 through 340
    odd_frame: bool, // weird behavior shit
    nmi_pending: bool,
    vram: [u8; 2048],
    palette_ram: [u8; 32],
    mirroring: Mirroring,
    chr_rom: Vec<u8>,

}

impl PPU {
    pub fn new(mirroring: Mirroring, chr_rom: Vec<u8>) -> Self {
        Self {
            nmi_enable: false,
            sprites_8x16: false,
            bg_pattern_table_addr: 0,
            sprite_pattern_table_addr: 0,
            vram_addr_inc: 1,

            emphasize_blue: false,
            emphasize_green: false,
            emphasize_red: false,
            sprite_rendering: false,
            bg_rendering: false,
            show_sprites_in_leftmost: false,
            show_bg_in_leftmost: false,
            grayscale: false,

            vblank_flag: Cell::new(false),
            sprite0_hit: Cell::new(false),
            sprite_overflow: Cell::new(false),

            oam_addr: 0,
            oam: [0; 256],
            oam2: [0; 32],

            v: Cell::new(0),
            t: 0,
            x: 0,
            w: Cell::new(false),

            read_buffer: Cell::new(0),

            io_latch: Cell::new(0),

            scanline: -1, // start on pre-render
            dot: 0,
            odd_frame: false,

            nmi_pending: false,

            vram: [0; 2048],
            palette_ram: [0; 32],
            mirroring, 
            chr_rom,
            
        }
    }
    
    fn index_to_register(i: u8) -> PPURegister {
        match i & 0x07 {
            0 => PPURegister::PPUCTRL,
            1 => PPURegister::PPUMASK,
            2 => PPURegister::PPUSTATUS,
            3 => PPURegister::OAMADDR,
            4 => PPURegister::OAMDATA,
            5 => PPURegister::PPUSCROLL,
            6 => PPURegister::PPUADDR,
            7 => PPURegister::PPUDATA,
            _ => unreachable!(),
        }
    }

    pub fn read_register(&self, register: u8) -> u8 {
        let r = Self::index_to_register(register);

        match r {
            PPURegister::PPUSTATUS => {
                let status_bytes = ((self.vblank_flag.get() as u8) << 7)
            | ((self.sprite0_hit.get() as u8) << 6)
            | ((self.sprite_overflow.get() as u8) << 5)
            | (self.io_latch.get() & 0x1F);

                self.vblank_flag.set(false);
                self.w.set(false);
                self.io_latch.set(status_bytes);

                status_bytes
            }

            PPURegister::OAMDATA => {
                let oam_read = self.oam[self.oam_addr as usize];

                self.io_latch.set(oam_read);

                oam_read
            }

            PPURegister::PPUDATA => {
                let ret;

                if self.v.get() & 0x3F00 == 0x3F00 {
                    ret = self.read_vram(self.v.get());
                    self.read_buffer.set(self.read_vram(self.v.get().wrapping_sub(0x1000)));
                } else {
                    ret = self.read_buffer.get();
                    self.read_buffer.set(self.read_vram(self.v.get()));
                }
                self.v.set(self.v.get().wrapping_add(self.vram_addr_inc as u16));

                ret
            }

            _ => self.io_latch.get()
        }

        


    }

    pub fn write_register(&mut self, reg: u8, data: u8) {
        let r = Self::index_to_register(reg);

        match r {
            PPURegister::PPUCTRL => {
                self.nmi_enable = data & 0b1000_0000 != 0;
                self.sprites_8x16 = data & 0b0010_0000 != 0;
                self.bg_pattern_table_addr = if data & 0b0001_0000 != 0 { 0x1000 } else { 0x0000 };
                self.sprite_pattern_table_addr = if data & 0b0000_1000 != 0 { 0x1000 } else { 0x0000 };
                self.vram_addr_inc = if data & 0b0000_0100 != 0 { 32 } else { 1 };
                self.t = (self.t & 0b0111_0011_1111_1111) | (((data as u16) & 0b0000_0011) << 10);

                self.io_latch.set(data);
            }

            PPURegister::PPUMASK => {
                self.grayscale = data & 0b0000_0001 != 0;
                self.show_bg_in_leftmost = data & 0b0000_0010 != 0;
                self.show_sprites_in_leftmost = data & 0b0000_0100 != 0;
                self.bg_rendering = data & 0b0000_1000 != 0;
                self.sprite_rendering = data & 0b0001_0000 != 0;
                self.emphasize_red = data & 0b0010_0000 != 0;
                self.emphasize_green = data & 0b0100_0000 != 0;
                self.emphasize_blue = data & 0b1000_0000 != 0;

                self.io_latch.set(data);
            }

            PPURegister::PPUSTATUS => {
                self.io_latch.set(data);
            }

            PPURegister::OAMADDR => {
                self.oam_addr = data;
                self.io_latch.set(data);
            }

            PPURegister::OAMDATA => {
                self.oam[self.oam_addr as usize] = data;
                self.oam_addr = self.oam_addr.wrapping_add(1);
                self.io_latch.set(data);
            }

            PPURegister::PPUSCROLL => {
                if !self.w.get() {
                    self.x = data & 0x07;
                    self.t = (self.t & 0b1111_1111_1110_0000) | ((data as u16) >> 3);
                    self.w.set(true);
                } else {
                    let coarse_y = (data as u16 >> 3) & 0x1F;
                    let fine_y = (data as u16 & 0x07) << 12;
                    self.t = (self.t & 0b0000_1100_0001_1111) | (coarse_y << 5) | fine_y;
                    self.w.set(false);
                }

                self.io_latch.set(data);
            }

            PPURegister::PPUADDR => {
                if !self.w.get() {
                    self.t = (self.t & 0x00FF) | (((data as u16) & 0x3F) << 8);
                    self.w.set(true);
                } else {
                    self.t = (self.t & 0xFF00) | (data as u16);
                    self.v.set(self.t);
                    self.w.set(false);
                }

                self.io_latch.set(data);
            }

            PPURegister::PPUDATA => {
                self.write_vram(self.v.get(), data);
                self.v.set(self.v.get().wrapping_add(self.vram_addr_inc as u16));

                self.io_latch.set(data);
            }
        }
    }

    pub fn tick(&mut self) {
        if self.scanline == 241 && self.dot == 1 {
            self.vblank_flag.set(true);
            self.nmi_pending = self.nmi_enable;
        }

        if self.scanline == -1 && self.dot == 1 {
            self.vblank_flag.set(false);
            self.sprite0_hit.set(false);
            self.sprite_overflow.set(false);
        }

        if self.odd_frame && self.scanline == -1 && self.dot == 339 {
            self.dot = 0;
            self.scanline = 0;
            self.odd_frame = false;
            return;
        }

        if self.scanline >= -1 && self.scanline <= 239 {
            if self.dot == 1 {
                self.oam2 = [0xFF; 32];
            }
            if self.dot == 65 {
                self.evaluate_sprites();
            }
        }

        self.dot += 1;
        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;
            if self.scanline > 260 {
                self.scanline = -1;
                self.odd_frame = !self.odd_frame;
            }
        }
    }

    // for secondary oam (oam2), finds sprites taht are within the y range for the next scanline, and copies them to oam2
    fn evaluate_sprites(&mut self) {
        let sprite_height = if self.sprites_8x16 { 16 } else { 8 };
        let target_scanline = self.scanline + 1;

        let mut found = 0;
        for i in 0..64 {
            let y = self.oam[i * 4] as i16;
            if target_scanline >= y && target_scanline < y + sprite_height {
                if found < 8 {
                    let base = found * 4;
                    self.oam2[base..base + 4].copy_from_slice(&self.oam[i * 4..i * 4 + 4]);
                } else {
                    self.sprite_overflow.set(true);
                    break;
                }
                found += 1;
            }
        }
    }

    pub fn take_nmi(&mut self) -> bool {
        let pending = self.nmi_pending;
        self.nmi_pending = false;
        pending
    }

    fn read_vram(&self, addr: u16) -> u8 {
    let addr = addr & 0x3FFF; // PPU address space is 14-bit
    match addr {
            0x0000..=0x1FFF => self.chr_rom[addr as usize], 
            0x2000..=0x3EFF => self.vram[self.mirror_nametable(addr)],
            0x3F00..=0x3FFF => self.palette_ram[self.mirror_palette(addr)],
            _ => unreachable!(),
        }
    }

    fn write_vram(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => { /* used for chr ram, replace this once you go past nrom */ }
            0x2000..=0x3EFF => self.vram[self.mirror_nametable(addr)] = data,
            0x3F00..=0x3FFF => self.palette_ram[self.mirror_palette(addr)] = data,
            _ => unreachable!(),
        }
    }

    fn mirror_nametable(&self, addr: u16) -> usize {
        let addr = (addr - 0x2000) % 0x1000;
        let table = addr / 0x0400;
        let offset = addr % 0x0400;

        let physical_table = match self.mirroring {
            Mirroring::Horizontal => table / 2,
            Mirroring::Vertical   => table % 2, 
        };

        (physical_table as usize * 0x0400) + offset as usize
    }   

    fn mirror_palette(&self, addr: u16) -> usize {
        let mut index = (addr - 0x3F00) % 0x20;
        if index >= 0x10 && index % 4 == 0 {
            index -= 0x10;
        }
        index as usize
    }

    // does the mini 4 color pallette shit
    fn tile_pixel(&self, table_base: u16, tile_index: u8, row: u8, col: u8) -> u8 {
        let tile_addr = table_base + (tile_index as u16 * 16);

        let plane0 = self.read_vram(tile_addr + row as u16);
        let plane1 = self.read_vram(tile_addr + row as u16 + 8);

        let bit = 7 - col;
        let lo = (plane0 >> bit) & 1;
        let hi = (plane1 >> bit) & 1;

        (hi << 1) | lo
    }

    fn background_pixel_color(&self) -> (u8, u8, u8) {
        let v = self.v.get();

        let tile_addr = 0x2000 | (v & 0x0FFF);
        let tile_index = self.read_vram(tile_addr);

        let attr_addr = 0x23C0
            | (v & 0x0C00)
            | ((v >> 4) & 0x38)
            | ((v >> 2) & 0x07);
        let attr_byte = self.read_vram(attr_addr);

        let shift = ((v >> 4) & 0b100) | (v & 0b010);
        let palette_select = (attr_byte >> shift) & 0b11;

        let fine_y = ((v >> 12) & 0x07) as u8;
        let col = self.x;

        let pattern_value = self.tile_pixel(
            self.bg_pattern_table_addr,
            tile_index,
            fine_y,
            col,
        );

        let palette_addr = 0x3F00
            | ((palette_select as u16) << 2)
            | (pattern_value as u16);

        let color_index = self.read_vram(palette_addr) & 0x3F;
        NES_PALETTE[color_index as usize]
    }
}