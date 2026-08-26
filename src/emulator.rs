use crate::cpu::CPU;
use crate::cpu::NesBus;
use crate::cartridge::load_rom_from_file;

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
}