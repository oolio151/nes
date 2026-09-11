use crate::emulator::Emulator;
use bincode::Options;
use sha2::{Digest, Sha256};
use std::{io::{Read, Write}, path::PathBuf};

pub type SaveError = String;
const MAGIC: &[u8; 8] = b"NESSTATE";
const VERSION: u32 = 2;
const MAX_FILE_SIZE: u64 = 1024 * 1024;
const HEADER_SIZE: usize = 8 + 4 + 32 + 32;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SaveState {
    pub(crate) cpu: CpuState,
    pub(crate) bus: BusState,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CpuState {
    // CPU state.
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) pc: u16,
    pub(crate) s: u8,
    pub(crate) p: u8,
    pub(crate) cycle_count: u64,

}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BusState {
    // Bus state.
    #[serde(with = "serde_big_array::BigArray")]
    pub(crate) cpu_ram: [u8; 0x0800],
    pub(crate) dma_pending: Option<u8>,

    // Chip, cartridge, and controller state.
    pub(crate) ppu: PpuState,
    pub(crate) apu: ApuState,
    pub(crate) mapper: MapperState,
    pub(crate) controller1: ControllerSnapshot,
    pub(crate) controller2: ControllerSnapshot,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PpuState {
    pub(crate) nmi_enable: bool,
    pub(crate) sprites_8x16: bool,
    pub(crate) bg_pattern_table_addr: u16,
    pub(crate) sprite_pattern_table_addr: u16,
    pub(crate) vram_addr_inc: u8,
    pub(crate) emphasize_blue: bool,
    pub(crate) emphasize_green: bool,
    pub(crate) emphasize_red: bool,
    pub(crate) sprite_rendering: bool,
    pub(crate) bg_rendering: bool,
    pub(crate) show_sprites_in_leftmost: bool,
    pub(crate) show_bg_in_leftmost: bool,
    pub(crate) grayscale: bool,
    pub(crate) vblank_flag: bool,
    pub(crate) sprite0_hit: bool,
    pub(crate) sprite_overflow: bool,
    pub(crate) oam_addr: u8,
    #[serde(with = "serde_big_array::BigArray")]
    pub(crate) oam: [u8; 256],
    pub(crate) oam2: [u8; 32],
    pub(crate) v: u16,
    pub(crate) t: u16,
    pub(crate) x: u8,
    pub(crate) w: bool,
    pub(crate) read_buffer: u8,
    pub(crate) io_latch: u8,
    pub(crate) scanline: i16,
    pub(crate) dot: u16,
    pub(crate) odd_frame: bool,
    pub(crate) nmi_pending: bool,
    #[serde(with = "serde_big_array::BigArray")]
    pub(crate) vram: [u8; 2048],
    pub(crate) palette_ram: [u8; 32],
    pub(crate) bg_shift_lo: u16,
    pub(crate) bg_shift_hi: u16,
    pub(crate) attr_shift_lo: u8,
    pub(crate) attr_shift_hi: u8,
    pub(crate) attr_latch_lo: bool,
    pub(crate) attr_latch_hi: bool,
    pub(crate) nt_latch: u8,
    pub(crate) at_latch: u8,
    pub(crate) bg_lo_latch: u8,
    pub(crate) bg_hi_latch: u8,
    pub(crate) sprite_pattern_lo: [u8; 8],
    pub(crate) sprite_pattern_hi: [u8; 8],
    pub(crate) sprite_attr: [u8; 8],
    pub(crate) sprite_x: [u8; 8],
    pub(crate) sprite_zero_next: bool,
    pub(crate) sprite_zero_current: bool,
    pub(crate) framebuffer: Vec<(u8, u8, u8)>,
    pub(crate) frame_complete_flag: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ApuState {
    pub(crate) pulse1: PulseChannelState,
    pub(crate) pulse2: PulseChannelState,
    pub(crate) cycle_parity: bool,
    pub(crate) triangle: TriangleChannelState,
    pub(crate) noise: NoiseChannelState,
    pub(crate) dmc: DmcChannelState,
    pub(crate) status: u8,
    pub(crate) frame_counter: u8,
    pub(crate) io_latch: u8,
    pub(crate) frame_cycle: u32,
    pub(crate) frame_step: u8,
    pub(crate) mode_5step: bool,
    pub(crate) irq_inhibit: bool,
    pub(crate) frame_irq_pending: bool,
    pub(crate) sample_acc: f32,
    pub(crate) cycles_per_sample: f32,
    pub(crate) filter: AudioFilterState,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PulseChannelState {
    pub(crate) duty_env: u8,
    pub(crate) sweep: u8,
    pub(crate) timer_lo: u8,
    pub(crate) length_timer_hi: u8,
    pub(crate) timer_period: u16,
    pub(crate) timer_counter: u16,
    pub(crate) sequencer_pos: u8,
    pub(crate) length_counter: u8,
    pub(crate) length_halt: bool,
    pub(crate) envelope_start: bool,
    pub(crate) envelope_decay: u8,
    pub(crate) envelope_counter: u8,
    pub(crate) constant_volume: bool,
    pub(crate) volume_or_period: u8,
    pub(crate) sweep_enabled: bool,
    pub(crate) sweep_period: u8,
    pub(crate) sweep_negate: bool,
    pub(crate) sweep_shift: u8,
    pub(crate) sweep_counter: u8,
    pub(crate) sweep_reload: bool,
    pub(crate) enabled: bool,
    pub(crate) is_channel2: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TriangleChannelState {
    pub(crate) linear_ctrl: u8,
    pub(crate) timer_lo: u8,
    pub(crate) length_timer_hi: u8,
    pub(crate) timer_period: u16,
    pub(crate) timer_counter: u16,
    pub(crate) sequencer_pos: u8,
    pub(crate) length_counter: u8,
    pub(crate) control_flag: bool,
    pub(crate) linear_counter: u8,
    pub(crate) linear_reload_value: u8,
    pub(crate) linear_reload_flag: bool,
    pub(crate) enabled: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoiseChannelState {
    pub(crate) env: u8,
    pub(crate) mode_period: u8,
    pub(crate) length: u8,
    pub(crate) timer_period: u16,
    pub(crate) timer_counter: u16,
    pub(crate) shift_reg: u16,
    pub(crate) length_counter: u8,
    pub(crate) length_halt: bool,
    pub(crate) envelope_start: bool,
    pub(crate) envelope_decay: u8,
    pub(crate) envelope_counter: u8,
    pub(crate) constant_volume: bool,
    pub(crate) volume_or_period: u8,
    pub(crate) mode_short: bool,
    pub(crate) enabled: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmcChannelState {
    pub(crate) irq_enable: bool,
    pub(crate) loop_flag: bool,
    pub(crate) rate_index: u8,
    pub(crate) sample_addr_reg: u8,
    pub(crate) sample_length_reg: u8,
    pub(crate) timer_period: u16,
    pub(crate) timer_counter: u16,
    pub(crate) current_addr: u16,
    pub(crate) bytes_remaining: u16,
    pub(crate) sample_buffer: Option<u8>,
    pub(crate) shift_register: u8,
    pub(crate) bits_remaining: u8,
    pub(crate) output_level: u8,
    pub(crate) silence_flag: bool,
    pub(crate) enabled: bool,
    pub(crate) irq_pending: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AudioFilterState {
    pub(crate) hp90_alpha: f32,
    pub(crate) hp440_alpha: f32,
    pub(crate) lp14k_alpha: f32,
    pub(crate) hp90_prev_in: f32,
    pub(crate) hp90_prev_out: f32,
    pub(crate) hp440_prev_in: f32,
    pub(crate) hp440_prev_out: f32,
    pub(crate) lp14k_prev_out: f32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MapperState {
    Nrom { #[serde(with = "serde_big_array::BigArray")] prg_ram: [u8; 0x2000] },
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ControllerSnapshot {
    pub(crate) button_state: u8,
    pub(crate) shift_reg: u8,
    pub(crate) strobe: bool,
}

pub fn capture(emu: &Emulator) -> Result<SaveState, SaveError> {
    Ok(SaveState { cpu: emu.cpu.save_state(), bus: emu.cpu.save_bus_state()? })
}

pub fn restore(emu: &mut Emulator, state: &SaveState) -> Result<(), SaveError> {
    validate(state)?;
    emu.cpu.load_bus_state(&state.bus)?;
    emu.cpu.load_state(&state.cpu);
    Ok(())
}

pub fn slot_path(emu: &Emulator) -> Result<PathBuf, SaveError> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut filename = emu.rom_filename.clone();
    filename.push(".ss0");
    Ok(executable.parent().ok_or("executable has no parent directory")?
        .join("savestates").join(filename))
}

fn encode(emu: &Emulator) -> Result<Vec<u8>, SaveError> {
    let payload = bincode::DefaultOptions::new().with_limit(MAX_FILE_SIZE)
        .serialize(&capture(emu)?).map_err(|e| e.to_string())?;
    let mut bytes = Vec::with_capacity(HEADER_SIZE + payload.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&emu.rom_fingerprint);
    bytes.extend_from_slice(&Sha256::digest(&payload));
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

fn decode(emu: &Emulator, bytes: &[u8]) -> Result<SaveState, SaveError> {
    if bytes.len() < HEADER_SIZE || bytes.len() as u64 > MAX_FILE_SIZE {
        return Err("invalid savestate size".into());
    }
    if &bytes[..8] != MAGIC || bytes[8..12] != VERSION.to_le_bytes() {
        return Err("unsupported savestate format or version".into());
    }
    if bytes[12..44] != emu.rom_fingerprint {
        return Err("savestate belongs to a different ROM".into());
    }
    let payload = &bytes[HEADER_SIZE..];
    if bytes[44..HEADER_SIZE] != Sha256::digest(payload)[..] {
        return Err("savestate checksum mismatch".into());
    }
    let state = bincode::DefaultOptions::new().with_limit(MAX_FILE_SIZE)
        .reject_trailing_bytes().deserialize(payload).map_err(|e| e.to_string())?;
    validate(&state)?;
    Ok(state)
}

pub fn save_file(emu: &Emulator) -> Result<(), SaveError> {
    let bytes = encode(emu)?;
    let path = slot_path(emu)?;
    let directory = path.parent().ok_or("savestate path has no parent")?;
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory).map_err(|e| e.to_string())?;
    temporary.write_all(&bytes).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(&path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_file(emu: &mut Emulator) -> Result<(), SaveError> {
    let file = std::fs::File::open(slot_path(emu)?).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_SIZE + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    let state = decode(emu, &bytes)?;
    restore(emu, &state)
}

fn validate(state: &SaveState) -> Result<(), SaveError> {
    let p = &state.bus.ppu;
    let a = &state.bus.apu;
    let valid = p.framebuffer.len() == 61440
        && (-1..=260).contains(&p.scanline) && p.dot <= 340
        && p.x < 8 && p.v <= 0x7fff && p.t <= 0x7fff
        && [0, 0x1000].contains(&p.bg_pattern_table_addr)
        && [0, 0x1000].contains(&p.sprite_pattern_table_addr)
        && [1, 32].contains(&p.vram_addr_inc)
        && a.pulse1.sequencer_pos < 8 && a.pulse2.sequencer_pos < 8
        && a.pulse1.sweep_shift < 8 && a.pulse2.sweep_shift < 8
        && a.triangle.sequencer_pos < 32 && a.dmc.rate_index < 16
        && a.dmc.bits_remaining <= 8 && a.dmc.output_level < 128
        && a.sample_acc.is_finite() && a.cycles_per_sample.is_finite()
        && a.cycles_per_sample > 0.0;
    if valid { Ok(()) } else { Err("invalid savestate component data".into()) }
}
