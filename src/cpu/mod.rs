pub mod opcodes;
pub mod ops;

pub enum Flag {
    Carry,
    Zero,
    InterruptDisable,
    Decimal,
    B,
    Overflow,
    Negative
}
pub struct CPU {

    memory: [u8; 65536],
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
    pub fn new() -> Self {
            Self {
                memory: [0; 65536],
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

    pub fn read(&self, address : u16) -> u8 {
        if address <= 0x1FFF {
            self.memory[(address & 0x07FF) as usize]
        } else if address >= 0x2000 && address <= 0x3FFF {
            self.memory[(0x2000 + (address & 0x0007)) as usize]
        } else {
            self.memory[address as usize]
        }
    }

    pub fn write(&mut self, address : u16, data : u8) {
        if address <= 0x1FFF {
        self.memory[(address & 0x07FF) as usize] = data;

        } else if address >= 0x2000 && address <= 0x3FFF {
            self.memory[(0x2000 + (address & 0x0007)) as usize] = data;
        }

        else {self.memory[address as usize] = data;}
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

    pub fn tick(&mut self) -> u8 {
        let opcode: u8 = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let (instruction, base_cycles) = opcodes::decode(opcode);
        let extra_cycles = instruction(self);
        let total_cycles = base_cycles + extra_cycles;
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

}