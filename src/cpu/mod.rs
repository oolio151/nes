pub mod opcodes;
pub mod ops;
pub mod mapper;

use crate::ppu::{PPU};
use mapper::*;
use crate::input::ControllerState;
use crate::apu::APU;

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
    // just to cover my ass
    fn save_state(&self) -> Result<crate::savestate::BusState, String> {
        Err("savestates unsupported by this bus".into())
    }
    fn load_state(&mut self, _state: &crate::savestate::BusState) -> Result<(), String> {
        Err("savestates unsupported by this bus".into())
    }
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
    fn write_timed(&mut self, address: u16, data: u8, _cpu_cycle: u64) {
        self.write(address, data);
    }
    fn reset_write_timing(&mut self) {}
    fn persistent_ram(&self) -> &[u8] { &[] }
    fn tick_ppu(&mut self) -> bool;

    fn begin_cpu_cycle(&mut self) -> bool {
        let mut nmi = false;
        for _ in 0..2 { nmi |= self.tick_ppu(); }
        nmi
    }
    fn end_cpu_cycle(&mut self) -> bool {
        let nmi = self.tick_ppu();
        self.tick_apu(1);
        nmi
    }
    fn clock_cpu_cycle(&mut self) -> bool {
        let nmi = self.begin_cpu_cycle();
        self.end_cpu_cycle() | nmi
    }
    fn run_dma(&mut self, _cpu_cycle: u64) -> (u16, bool) {
        let cycles = self.take_dma_cycles();
        let mut nmi = false;
        for _ in 0..cycles { nmi |= self.clock_cpu_cycle(); }
        (cycles, nmi)
    }
    fn get_framebuffer(&self) -> &[(u8, u8, u8)];
    fn take_dma_cycles(&mut self) -> u16;
    fn frame_complete(&mut self) -> bool;
    fn set_controller1(&self, buttons: u8) { let _ = buttons; }
    fn set_controller2(&self, buttons: u8) { let _ = buttons; }
    fn irq_pending(&self) -> bool { false }
    fn tick_apu(&mut self, cycles: u32) { let _ = cycles; }
    fn reset_ppu(&mut self) { }
    fn reset_apu(&mut self) { }
    fn drain_audio_samples(&mut self) -> Vec<f32> { Vec::new() }
    fn set_audio_sample_rate(&mut self, sample_rate: u32) { let _ = sample_rate; }

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
    pub apu: APU,
    cartridge: Box<dyn Mapper>,
    dma_pending: Option<u8>,
    controller1: ControllerState,
    controller2: ControllerState
}

impl Bus for NesBus {
    fn write_timed(&mut self, address: u16, data: u8, cpu_cycle: u64) {
        // MMC1 observes consecutive CPU writes even outside its serial port.
        // Supported mappers ignore addresses below their cartridge window.
        self.cartridge.write_timed(address, data, cpu_cycle);
        if address < 0x4020 { self.write(address, data); }
    }
    fn reset_write_timing(&mut self) { self.cartridge.reset_write_timing(); }
    fn persistent_ram(&self) -> &[u8] { self.cartridge.persistent_ram() }

    fn save_state(&self) -> Result<crate::savestate::BusState, String> {
        Ok(crate::savestate::BusState {
            cpu_ram: self.cpu_ram,
            dma_pending: self.dma_pending,
            ppu: self.ppu.save_state(),
            apu: self.apu.save_state(),
            mapper: self.cartridge.save_state()?,
            controller1: self.controller1.save_state(),
            controller2: self.controller2.save_state(),
        })
    }
    fn load_state(&mut self, state: &crate::savestate::BusState) -> Result<(), String> {
        // mapper must reject unsupported states before modifying itself.
        self.cartridge.load_state(&state.mapper)?;
        self.cpu_ram = state.cpu_ram;
        self.dma_pending = state.dma_pending;
        self.ppu.load_state(&state.ppu);
        self.apu.load_state(&state.apu);
        self.controller1.load_state(&state.controller1);
        self.controller2.load_state(&state.controller2);
        Ok(())
    }
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.cpu_ram[(address & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.read_register((address & 0x0007) as u8, self.cartridge.as_ref()),
            0x4016 => self.controller1.read(),
            0x4017 => self.controller2.read(),
            0x4000..=0x4015 => self.read_apu_io(address),
            0x4018..=0x401F => 0,
            0x4020..=0xFFFF => self.cartridge.read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.cpu_ram[(address & 0x07FF) as usize] = data,
            0x2000..=0x3FFF => self.ppu.write_register((address & 0x0007) as u8, data, self.cartridge.as_mut()),
            0x4014 => self.dma_pending = Some(data),
            0x4016 => {
                self.controller1.write_strobe(data);
                self.controller2.write_strobe(data);
            }
            0x4000..=0x4017 => self.write_apu_io(address, data),
            0x4018..=0x401F => {},
            0x4020..=0xFFFF => self.cartridge.write(address, data),
        }
    }

    fn tick_ppu(&mut self) -> bool {
        self.ppu.tick(self.cartridge.as_ref());
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
            self.ppu.write_register(4, byte, self.cartridge.as_mut()); // OAMDATA
        }

        514 
    }

    fn run_dma(&mut self, cpu_cycle: u64) -> (u16, bool) {
        let Some(page) = self.dma_pending.take() else { return (0, false); };
        let alignment = 1 + (cpu_cycle & 1) as u16;
        let mut nmi = false;
        for _ in 0..alignment { nmi |= self.clock_cpu_cycle(); }
        for offset in 0..256u16 {
            nmi |= self.begin_cpu_cycle();
            let byte = self.read(((page as u16) << 8) | offset);
            nmi |= self.end_cpu_cycle();
            nmi |= self.begin_cpu_cycle();
            self.ppu.write_register(4, byte, self.cartridge.as_mut());
            nmi |= self.end_cpu_cycle();
        }
        (512 + alignment, nmi)
    }

    fn frame_complete(&mut self) -> bool {
        self.ppu.frame_complete()
    }

    fn set_controller1(&self, buttons: u8) {
        self.controller1.set_buttons(buttons);
    }

    fn set_controller2(&self, buttons: u8) {
        self.controller2.set_buttons(buttons);
    }

    fn irq_pending(&self) -> bool {
        self.apu.frame_irq_pending() || self.apu.dmc_irq_pending() || self.cartridge.irq_pending()
    }

    fn tick_apu(&mut self, cycles: u32) {
        self.apu.tick(cycles);
        if let Some(addr) = self.apu.dmc_fetch_request() {
            let byte = self.read(addr);
            self.apu.dmc_provide_byte(byte);
        }
    }

    // reset
    fn reset_ppu(&mut self) {
        self.ppu.reset();
    }
    fn reset_apu(&mut self) {
        self.apu.reset();
    }

    fn drain_audio_samples(&mut self) -> Vec<f32> {
        self.apu.drain_samples()
    }

    fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        self.apu.set_sample_rate(sample_rate);
    }

}

impl NesBus {

    pub fn new(cartridge: Box<dyn Mapper>) -> Self {
        Self {
            cpu_ram: [0; 0x0800],
            ppu: PPU::new(),
            apu: APU::new(),
            cartridge,
            dma_pending: None, 
            controller1: ControllerState::new(),
            controller2: ControllerState::new(),
        }
    }

    fn read_apu_io(&self, address: u16) -> u8 {
        self.apu.read_register(address)
    }

    fn write_apu_io(&mut self, address: u16, data: u8) {
        self.apu.write_register(address, data);
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
    cycle_count: u64, // Elapsed CPU bus cycles.
    instruction_cycles: Option<u8>, // Transient; savestates occur between instructions.
    nmi_latched: bool,
    irq_previous_cycle: bool,
    irq_poll: bool,
    irq_sampled: bool,
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
                cycle_count: 0,
                instruction_cycles: None,
                nmi_latched: false,
                irq_previous_cycle: false,
                irq_poll: false,
                irq_sampled: false,

            }
            
    }

    pub fn read(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    pub(crate) fn read_cycle(&mut self, address: u16) -> u8 {
        if self.instruction_cycles.is_some() { self.nmi_latched |= self.bus.begin_cpu_cycle(); }
        let value = self.bus.read(address);
        if self.instruction_cycles.is_some() { self.finish_cycle(); }
        value
    }

    fn clock_cycle(&mut self) {
        self.nmi_latched |= self.bus.begin_cpu_cycle();
        self.finish_cycle();
    }

    fn finish_cycle(&mut self) {
        self.nmi_latched |= self.bus.end_cpu_cycle();
        self.irq_poll = self.irq_previous_cycle;
        self.irq_previous_cycle = self.bus.irq_pending();
        self.cycle_count = self.cycle_count.wrapping_add(1);
    }

    pub fn write(&mut self, address: u16, data: u8) {
        if self.instruction_cycles.is_some() {
            self.nmi_latched |= self.bus.begin_cpu_cycle();
            self.bus.write_timed(address, data, self.cycle_count);
            self.finish_cycle();
        } else {
            self.bus.write(address, data);
        }
    }

    pub(crate) fn write_rmw(&mut self, address: u16, original: u8, modified: u8) {
        self.write(address, original);
        self.write(address, modified);
    }

    pub(crate) fn indexed_dummy_read(&mut self, base: u16, address: u16, read_cycles: u8) {
        if (base & 0xff00) != (address & 0xff00)
            || self.instruction_cycles.is_some_and(|cycles| cycles > read_cycles) {
            self.read_cycle((base & 0xff00) | (address & 0xff));
        }
    }

    pub fn take_nmi(&mut self) -> bool {
        std::mem::take(&mut self.nmi_latched)
    }

    pub fn persistent_ram(&self) -> &[u8] { self.bus.persistent_ram() }

    pub fn reset(&mut self){
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.pc = (self.read(0xFFFC) as u16) | ((self.read(0xFFFD) as u16) << 8);
        self.s = 0xFD;
        self.p = 0b0010_0100;
        self.cycle_count = 0;
        self.nmi_latched = false;
        self.irq_previous_cycle = false;
        self.irq_poll = false;
        self.irq_sampled = false;
        self.instruction_cycles = None;
        // Reset restarts the software clock; retain mapper registers/RAM.
        self.bus.reset_write_timing();
    }

    pub fn tick(&mut self) -> u16 {
        let start = self.cycle_count;
        let old_i = self.get_flag(Flag::InterruptDisable);
        self.instruction_cycles = Some(1);
        let opcode = self.read_cycle(self.pc);
        let first_cycle_irq = self.irq_previous_cycle;
        self.pc = self.pc.wrapping_add(1);
        let (instruction, base_cycles) = opcodes::decode(opcode);
        self.instruction_cycles = Some(base_cycles);
        let extra_cycles = instruction(self);
        let expected = (base_cycles as u64 + extra_cycles as u64).max(1);
        // implied instructions perform a discarded read on their final cycle.
        while self.cycle_count.wrapping_sub(start) < expected {
            self.read_cycle(self.pc);
        }
        debug_assert_eq!(self.cycle_count.wrapping_sub(start), expected,
            "incorrect bus cycle count for opcode {opcode:02x}");
        // some dirty fix
        let masked = if matches!(opcode, 0x58 | 0x78 | 0x28) { old_i }
            else { self.get_flag(Flag::InterruptDisable) };

        let polled = if opcode & 0x1f == 0x10 {
            first_cycle_irq || (extra_cycles == 2 && self.irq_poll)
        } else { self.irq_poll };
        self.irq_sampled = polled && !masked;
        self.instruction_cycles = None;
        let (dma_cycles, nmi) = self.bus.run_dma(self.cycle_count);
        self.nmi_latched |= nmi;
        if dma_cycles != 0 {
            self.irq_sampled |= self.bus.irq_pending() && !self.get_flag(Flag::InterruptDisable);
            self.irq_previous_cycle = self.bus.irq_pending();
        }
        self.cycle_count = self.cycle_count.wrapping_add(dma_cycles as u64);
        self.cycle_count.wrapping_sub(start) as u16
    }

    pub fn cycle(&mut self, cycles: u64) {
        for _ in 0..cycles { self.clock_cycle(); }
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
        self.read_cycle(0x0100 + self.s as u16)
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
        self.pc = (self.read_cycle(vector_addr) as u16) | ((self.read_cycle(vector_addr + 1) as u16) << 8);
    }

    pub fn service_pending_irq(&mut self) {
        if self.irq_sampled { self.service_interrupt(0xfffe); }
    }

    fn service_interrupt(&mut self, vector: u16) -> u16 {
        self.irq_sampled = false;
        self.instruction_cycles = Some(7);
        self.read_cycle(self.pc);
        self.read_cycle(self.pc);
        self.interrupt(vector, false);
        self.instruction_cycles = None;
        7
    }

    pub fn nmi(&mut self) -> u16 { self.service_interrupt(0xfffa) }

    pub fn irq(&mut self) -> u16 {
        if self.get_flag(Flag::InterruptDisable) { return 0; }
        self.service_interrupt(0xfffe)
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

    // passthru to the bus
    pub fn set_controller1(&self, buttons: u8) {
        self.bus.set_controller1(buttons);
    }
    pub fn set_controller2(&self, buttons: u8) {
        self.bus.set_controller2(buttons);
    }

    pub fn irq_pending(&self) -> bool {
        self.bus.irq_pending()
    }

    pub fn tick_apu(&mut self, cycles: u32) {
        self.bus.tick_apu(cycles);
    }

    // for reset
    pub fn reset_ppu(&mut self) {
        self.bus.reset_ppu();
    }
    pub fn reset_apu(&mut self) {
        self.bus.reset_apu();
    }

    pub fn drain_audio_samples(&mut self) -> Vec<f32> {
        self.bus.drain_audio_samples()
    }

    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        self.bus.set_audio_sample_rate(sample_rate);
    }

    pub fn save_state(&self) -> crate::savestate::CpuState {
        crate::savestate::CpuState {
            a: self.a,
            x: self.x,
            y: self.y,
            pc: self.pc,
            s: self.s,
            p: self.p,
            cycle_count: self.cycle_count,
            nmi_latched: self.nmi_latched,
            irq_previous_cycle: self.irq_previous_cycle,
            irq_poll: self.irq_poll,
            irq_sampled: self.irq_sampled,
        }
    }
    pub fn load_state(&mut self, state: &crate::savestate::CpuState) {
        self.a = state.a;
        self.x = state.x;
        self.y = state.y;
        self.pc = state.pc;
        self.s = state.s;
        self.p = state.p;
        self.cycle_count = state.cycle_count;
        self.nmi_latched = state.nmi_latched;
        self.irq_previous_cycle = state.irq_previous_cycle;
        self.irq_poll = state.irq_poll;
        self.irq_sampled = state.irq_sampled;
        self.instruction_cycles = None;
    }

    pub(crate) fn save_bus_state(&self) -> Result<crate::savestate::BusState, String> {
        self.bus.save_state()
    }
    pub(crate) fn load_bus_state(&mut self, state: &crate::savestate::BusState) -> Result<(), String> {
        self.bus.load_state(state)
    }
}
