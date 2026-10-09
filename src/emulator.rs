use crate::cpu::CPU;
use crate::cpu::NesBus;
use crate::cartridge::load_rom;
use sha2::{Digest, Sha256};
pub struct Emulator {
    pub cpu: CPU,
    battery_path: Option<std::path::PathBuf>,
    pub cartridge_metadata: crate::cartridge::CartridgeMetadata,
    pub(crate) rom_filename: std::ffi::OsString,
    pub(crate) rom_fingerprint: [u8; 32],
}

impl Emulator {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let mut rom = load_rom(&bytes)?;
        let battery_path = if rom.mapper.persistent_ram().is_empty() { None } else {
            let path = crate::battery::path_for_rom(&std::fs::canonicalize(path).map_err(|e| e.to_string())?);
            if let Some(data) = crate::battery::load(&path, rom.mapper.persistent_ram().len())? {
                rom.mapper.load_persistent_ram(&data)?;
            }
            Some(path)
        };
        let rom_filename = std::path::Path::new(path).file_name()
            .ok_or("ROM path has no filename")?.to_os_string();
        let rom_fingerprint = Sha256::digest(&bytes).into();
        let bus = NesBus::new(rom.mapper);
        let mut cpu = CPU::new(Box::new(bus));
        cpu.reset();
        Ok(Self { cpu, battery_path, cartridge_metadata: rom.metadata, rom_filename, rom_fingerprint })
    }

    /// Flush raw battery-backed cartridge RAM independently of savestates.
    pub fn flush_battery(&self) -> Result<(), String> {
        if let Some(path) = &self.battery_path {
            crate::battery::save(path, self.cpu.persistent_ram())?;
        }
        Ok(())
    }

    pub fn step(&mut self) -> bool {
        self.cpu.tick();
        let mut nmi = self.cpu.take_nmi();
        let mut nmi_fired = false;
        if !nmi {
            self.cpu.service_pending_irq();
            nmi = self.cpu.take_nmi();
        }
        while nmi {
            nmi_fired = true;
            self.cpu.nmi();
            nmi = self.cpu.take_nmi();
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
