use std::cell::Cell;

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

    // INTERNAL REGISTERS
    v: u16, // used for scroll pos while rendering, otherwise is current vram address, aka PPUADDR
    t: u16, // used for coarse-x scroll for next scanline and start y for screen, otherwise holds scroll or vram address before transferring to v
    x: u8, // fine-x pos of the current scroll, used for rendering stuff
    w: Cell<bool>, // toggles on write to PPUSCROLL || PPUADDR, indicating whether this is first or second write. clears on read of PPUSTATUS

    // PPUDATA
    read_buffer: Cell<u8>, // updates on every PPUDATA read after previous contents returned to CPU, delaying PPUDATA read by 1

    // some weird behavior of the chip
    io_latch: Cell<u8>, // writes to any register or reads to readable ones fill the latch with whatever bytes were wrote/read

    // THE OTHER CAVELIERS
    scanline: i16, // -1 indicates prerender, goes up to 260
    dot: u16, // 0 through 340
    ood_frame: bool, // weird behavior shit
    nmi_pending: bool,

}

impl PPU {
    pub fn new() -> Self {
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

            v: 0,
            t: 0,
            x: 0,
            w: Cell::new(false),

            read_buffer: Cell::new(0),

            io_latch: Cell::new(0),

            scanline: -1, // start on pre-render
            dot: 0,
            ood_frame: false,

            nmi_pending: false,
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

                if self.v & 0x3F00 == 0x3F00 {
                    ret = self.read_vram(self.v);

                    self.read_buffer.set(self.read_vram(self.v.wrapping_sub(0x1000)));
                } else {

                    ret = self.read_buffer.get();
                    self.read_buffer.set(self.read_vram(self.v));
                }

                self.v = self.v.wrapping_add(self.vram_addr_inc as u16);

                ret
            }

            _ => self.io_latch.get()
        }

        


    }

    pub fn write_register(&mut self, reg: u8, data: u8) {
    }

    pub fn tick(&mut self) {

    }
}