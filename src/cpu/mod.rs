pub mod opcodes;
pub mod ops;
pub mod mapper;

use crate::ppu::{PPU};
use crate::cartridge::Mirroring;
use mapper::*;

pub enum Flag {
    Carry,
    Zero,
    InterruptDisable,
    Decimal,
    B,
    Overflow,
    Negative
}

pub trait Bus {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
    fn tick_ppu(&mut self) -> bool;
    fn get_framebuffer(&self) -> &[(u8, u8, u8)];
    fn take_dma_cycles(&mut self) -> u16;
    fn frame_complete(&mut self) -> bool;
}


// Flatbus is for the tests, since the tests assume no memory mirroring
pub struct FlatBus {
    memory: [u8; 65536],
}

impl FlatBus {
    pub fn new() -> Self {
        Self { memory: [0; 65536] }
    }
}

impl Bus for FlatBus {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }

    fn tick_ppu(&mut self) -> bool { 
        false 
    }

    fn get_framebuffer(&self) -> &[(u8, u8, u8)] {
        &[]
    }

    fn take_dma_cycles(&mut self) -> u16 {
        0
    }

    fn frame_complete(&mut self) -> bool {
        false
    }
}


pub struct NesBus {
    cpu_ram: [u8; 0x0800],
    pub ppu: PPU,
    apu_io: [u8; 24],
    cartridge: Box<dyn Mapper>,
    dma_pending: Option<u8>,
}

impl Bus for NesBus {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.cpu_ram[(address & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.read_register((address & 0x0007) as u8),
            0x4000..=0x4017 => self.read_apu_io(address),
            0x4018..=0x401F => 0,
            0x4020..=0xFFFF => self.cartridge.read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.cpu_ram[(address & 0x07FF) as usize] = data,
            0x2000..=0x3FFF => self.ppu.write_register((address & 0x0007) as u8, data),
            0x4014 => self.dma_pending = Some(data),
            0x4000..=0x4017 => self.write_apu_io(address, data),
            0x4018..=0x401F => {},
            0x4020..=0xFFFF => self.cartridge.write(address, data),
        }
    }

    fn tick_ppu(&mut self) -> bool {
        self.ppu.tick();
        self.ppu.take_nmi()
    }

    fn get_framebuffer(&self) -> &[(u8, u8, u8)] {
        self.ppu.get_framebuffer()
    }

    fn take_dma_cycles(&mut self) -> u16 {
        let Some(page) = self.dma_pending.take() else {
            return 0;
        };

        let base = (page as u16) << 8;
        for i in 0..256u16 {
            let byte = self.read(base + i);
            self.ppu.write_register(4, byte); // OAMDATA
        }

        514 
    }

    fn frame_complete(&mut self) -> bool {
        self.ppu.frame_complete()
    }
}

impl NesBus {

    pub fn new(cartridge: Box<dyn Mapper>, mirroring: Mirroring, chr_rom: Vec<u8>) -> Self {
        Self {
            cpu_ram: [0; 0x0800],
            ppu: PPU::new(mirroring, chr_rom),
            apu_io: [0; 24],
            cartridge,
            dma_pending: None, 
        }
    }

    fn read_apu_io(&self, address: u16) -> u8 {
        0
    }

    fn write_apu_io(&mut self, address: u16, data: u8) {
    }
}

pub struct CPU {

    //memory: [u8; 65536],
    bus: Box<dyn Bus>,
    /*
    memory map
    0x0000 - 0x07FF 2kb of internal cpu ram
    0x0800 - 0x0FFF, 0x1000 - 0x17FF, 0x1800 - 0x1FFF all mirror cpu ram
    0x2000 - 0x2007 ppu registers
    0x2008 - 0x3FFF mirrored ppu registers, repeating every 8 bytes
    0x4000 - 0x4017 apu and io registers
    0x4018 - 0x401F api and io test stuff look at the wiki
    0x4020 - 0xFFFF unmapped, usually for cartiridge use
    - 0x6000 - 0x7FFF usually cartridge RAM when present
    - 0x8000 - 0xFFFF usually cartridge ROM and mapper registers
     */
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16, //program counter
    pub s: u8, //stack pointer
    pub p: u8, //status register
    /*status register guides
    bit 7 (high) - negative
    bit 6 - overflow
    bit 5 - always 1
    bit 4 - the b flag, read the wiki idk
    bit 3 - decimal
    bit 2 - interrupt disable
    bit 1 - zero flag
    bit 0 - carry flag
     */
    cycle_count: u64 //used to keep cycles in track with the ppu
}

impl CPU {
    pub fn new(bus: Box<dyn Bus>) -> Self {
            Self {
                bus,
                a: 0,
                x: 0, // for addressing modes
                y: 0, // for addressing modes
                pc: 0xFFFC,
                // stack pointer
                // to write to stack, write to 0x0100 + cpu.s as u16
                s: 0b1111_1101,
                p: 0b0010_0000, // status register
                cycle_count: 0

            }
            
    }

    pub fn read(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.bus.write(address, data)
    }

    pub fn reset(&mut self){
        //self.memory = [0; 65536];
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.pc = (self.read(0xFFFC) as u16) | ((self.read(0xFFFD) as u16) << 8);
        self.s = 0xFD;
        self.p = 0b0010_0000;
        self.cycle_count = 0;
    }

    pub fn tick(&mut self) -> u16 {
        let opcode: u8 = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let (instruction, base_cycles) = opcodes::decode(opcode);
        let extra_cycles = instruction(self);
        let dma_cycles = self.bus.take_dma_cycles();
        let total_cycles = base_cycles as u16 + extra_cycles as u16 + dma_cycles;
        self.cycle_count += total_cycles as u64;
        
        total_cycles
    }

    pub fn cycle(&mut self, cycles : u64) {
        self.cycle_count += cycles;
    }

    pub fn set_flag(&mut self, flag : Flag, on : bool){
        let mask: u8 = match flag {
            Flag::Carry            => 0b0000_0001,
            Flag::Zero             => 0b0000_0010,
            Flag::InterruptDisable => 0b0000_0100,
            Flag::Decimal          => 0b0000_1000,
            Flag::B                => 0b0001_0000,
            Flag::Overflow         => 0b0100_0000,
            Flag::Negative         => 0b1000_0000,
        };

        if on {
            self.p |= mask;
        } else {
            self.p &= !mask;
        }
    }

    pub fn get_flag(&self, flag : Flag) -> bool {
        let mask: u8 = match flag {
            Flag::Carry            => 0b0000_0001,
            Flag::Zero             => 0b0000_0010,
            Flag::InterruptDisable => 0b0000_0100,
            Flag::Decimal          => 0b0000_1000,
            Flag::B                => 0b0001_0000,
            Flag::Overflow         => 0b0100_0000,
            Flag::Negative         => 0b1000_0000,
        };

        (self.p & mask) != 0
    }

    // stack helpers, should prolly move these to the opcodes sometimes
    pub fn push(&mut self, value: u8) {
        self.write(0x0100 + self.s as u16, value);
        self.s = self.s.wrapping_sub(1);
    }

    pub fn pull(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read(0x0100 + self.s as u16)
    }

    fn interrupt(&mut self, vector_addr: u16, set_b: bool) {
        let pc_high = (self.pc >> 8) as u8;
        let pc_low = (self.pc & 0xFF) as u8;

        self.push(pc_high);
        self.push(pc_low);

        let mut status = self.p | 0b0010_0000; // masking for safety
        if set_b {
            status |= 0b0001_0000;
        } else {
            status &= !0b0001_0000;
        }
        self.push(status);

        self.set_flag(Flag::InterruptDisable, true);
        self.pc = (self.read(vector_addr) as u16) | ((self.read(vector_addr + 1) as u16) << 8);
    }

    pub fn nmi(&mut self) {
        self.interrupt(0xFFFA, false);
    }

    pub fn tick_ppu(&mut self) -> bool {
        self.bus.tick_ppu()
    }

    pub fn framebuffer(&self) -> &[(u8, u8, u8)] {
        self.bus.get_framebuffer()
    }

    pub fn frame_complete(&mut self) -> bool {
        self.bus.frame_complete()
    }

}