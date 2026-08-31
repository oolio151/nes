use crate::cpu::CPU;
use crate::cpu::NesBus;
use crate::cartridge::load_rom_from_file;
use crate::apu::APU;

pub struct Emulator {
    pub cpu: CPU,
}

impl Emulator {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let rom = load_rom_from_file(path)?;
        let bus = NesBus::new(rom.mapper, rom.mirroring, rom.chr_rom);
        let mut cpu = CPU::new(Box::new(bus));
        cpu.reset();
        Ok(Self { cpu })
    }

    pub fn step(&mut self) -> bool {
        let cpu_cycles = self.cpu.tick();
        let mut nmi_fired = false;
        
        for _ in 0..(cpu_cycles as u32 * 3) {
            if self.cpu.tick_ppu() {
                self.cpu.nmi();
                nmi_fired = true;
            }
        }

        self.cpu.tick_apu(cpu_cycles as u32);

        if self.cpu.irq_pending() {
            self.cpu.irq();
        }

        nmi_fired
    }
 
    pub fn run_one_frame(&mut self) {
        loop {
            self.step();
            if self.cpu.frame_complete() {
                break;
            }
        }
    }

    // passthru from cpu from bus
    pub fn set_controller1(&self, buttons: u8) {
        self.cpu.set_controller1(buttons);
    }
    pub fn set_controller2(&self, buttons: u8) {
        self.cpu.set_controller2(buttons);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.reset_ppu();
        self.cpu.reset_apu();
    }
}