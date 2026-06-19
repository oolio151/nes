mod opcodes;

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
    a: u8,
    x: u8,
    y: u8,
    pc: u16, //program counter
    s: u8, //stack pointer
    p: u8, //status register
    cycle_count: u64 //used to keep cycles in track with the ppu
}

impl CPU {
    pub fn new() -> Self {
            Self {
                memory: [0; 65536],
                a: 0,
                x: 0,
                y: 0,
                pc: 0xFFFC,
                s: 0xFD,
                p: 0,
                cycle_count: 0

            }
            
    }

    pub fn read(&self, address : u16) -> u8{
        return self.memory[address as usize];
    }

    pub fn write(&mut self, address : u16, data : u8) {
        if address <= 0x1FFF {
        self.memory[(address & 0x07FF) as usize] = data;

        } else if address >= 0x2000 && address <= 0x3FFF {
            self.memory[(address & 0x0007) as usize] = data;
        }

        else {self.memory[address as usize] = data;}
    }

    pub fn reset(&mut self){
        self.memory = [0; 65536];
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.pc = (self.read(0xFFFC) as u16) | ((self.read(0xFFFD) as u16) << 8);
        self.s = 0xFD;
        self.p = 0;
        self.cycle_count = 0;
    }

    pub fn tick(&mut self) -> u8 {
        let opcode: u8 = self.read(self.pc);
        self.pc += 1;
        let (instruction, base_cycles) = opcodes::decode(opcode);
        let extra_cycles = instruction(self);
        let total_cycles = base_cycles + extra_cycles;
        self.cycle_count += total_cycles as u64;
        
        total_cycles
    }

    pub fn cycle(&mut self, cycles : u64) {
        self.cycle_count += cycles;
    }
}