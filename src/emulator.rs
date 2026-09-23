use crate::cpu::CPU;
use crate::cpu::NesBus;
use crate::cartridge::load_rom;
use sha2::{Digest, Sha256};
pub struct Emulator {
    pub cpu: CPU,
    pub cartridge_metadata: crate::cartridge::CartridgeMetadata,
    pub(crate) rom_filename: std::ffi::OsString,
    pub(crate) rom_fingerprint: [u8; 32],
}

impl Emulator {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let rom = load_rom(&bytes)?;
        let rom_filename = std::path::Path::new(path).file_name()
            .ok_or("ROM path has no filename")?.to_os_string();
        let rom_fingerprint = Sha256::digest(&bytes).into();
        let bus = NesBus::new(rom.mapper);
        let mut cpu = CPU::new(Box::new(bus));
        cpu.reset();
        Ok(Self { cpu, cartridge_metadata: rom.metadata, rom_filename, rom_fingerprint })
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

    pub fn drain_audio_samples(&mut self) -> Vec<f32> {
        self.cpu.drain_audio_samples()
    }

    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        self.cpu.set_audio_sample_rate(sample_rate);
    }
}
