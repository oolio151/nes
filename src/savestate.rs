//! Snapshot data for the currently supported NROM emulator.
//!
//! Capture after a complete Emulator::step. Cell-backed fields are stored as
//! plain values; capture/restore must use direct field access, not bus reads.
//! PRG/CHR ROM and fixed NROM mirroring belong to the loaded cartridge and are
//! excluded. A future file wrapper must validate ROM identity and format version.
//! Pending output audio is excluded: clear APU and frontend audio queues on load.
//! Capture, restore, and file serialization are not implemented by these types.

pub struct SaveState {
    // CPU state.
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) pc: u16,
    pub(crate) s: u8,
    pub(crate) p: u8,
    pub(crate) cycle_count: u64,

    // Bus state.
    pub(crate) cpu_ram: [u8; 0x0800],
    pub(crate) dma_pending: Option<u8>,

    // Chip, cartridge, and controller state.
    pub(crate) ppu: PpuState,
    pub(crate) apu: ApuState,
    pub(crate) mapper: MapperState,
    pub(crate) controller1: ControllerSnapshot,
    pub(crate) controller2: ControllerSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
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
    pub(crate) framebuffer: [(u8, u8, u8); 61440],
    pub(crate) frame_complete_flag: bool,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

pub enum MapperState {
    Nrom { prg_ram: [u8; 0x2000] },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControllerSnapshot {
    pub(crate) button_state: u8,
    pub(crate) shift_reg: u8,
    pub(crate) strobe: bool,
}
