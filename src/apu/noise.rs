use super::consts::{LENGTH_TABLE, NOISE_PERIOD_TABLE};

// wasnt feeling well, claude mostly did this and triangle
pub struct NoiseChannel {
    env: u8,
    mode_period: u8,
    length: u8,

    timer_period: u16,
    timer_counter: u16,
    shift_reg: u16,

    length_counter: u8,
    length_halt: bool,

    envelope_start: bool,
    envelope_decay: u8,
    envelope_counter: u8,
    constant_volume: bool,
    volume_or_period: u8,

    mode_short: bool,
    enabled: bool,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            env: 0,
            mode_period: 0,
            length: 0,

            timer_period: NOISE_PERIOD_TABLE[0],
            timer_counter: 0,
            shift_reg: 1, // cant be zero for some reason

            length_counter: 0,
            length_halt: false,

            envelope_start: false,
            envelope_decay: 0,
            envelope_counter: 0,
            constant_volume: false,
            volume_or_period: 0,

            mode_short: false,
            enabled: false,
        }
    }

    pub fn write_env(&mut self, data: u8) {
        self.env = data;
        self.length_halt = data & 0x20 != 0;
        self.constant_volume = data & 0x10 != 0;
        self.volume_or_period = data & 0x0F;
    }

    pub fn write_mode_period(&mut self, data: u8) {
        self.mode_period = data;
        self.mode_short = data & 0x80 != 0;
        self.timer_period = NOISE_PERIOD_TABLE[(data & 0x0F) as usize];
    }

    pub fn write_length(&mut self, data: u8) {
        self.length = data;
        if self.enabled {
            self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        }
        self.envelope_start = true;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    pub fn tick_timer(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;

            let feedback_bit = if self.mode_short { 6 } else { 1 };
            let feedback = (self.shift_reg & 1) ^ ((self.shift_reg >> feedback_bit) & 1);
            self.shift_reg >>= 1;
            self.shift_reg |= feedback << 14;
        } else {
            self.timer_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_decay = 15;
            self.envelope_counter = self.volume_or_period;
        } else if self.envelope_counter > 0 {
            self.envelope_counter -= 1;
        } else {
            self.envelope_counter = self.volume_or_period;
            if self.envelope_decay > 0 {
                self.envelope_decay -= 1;
            } else if self.length_halt {
                self.envelope_decay = 15;
            }
        }
    }

    pub fn clock_length_counter(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn current_volume(&self) -> u8 {
        if self.constant_volume { self.volume_or_period } else { self.envelope_decay }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || self.length_counter == 0 {
            return 0;
        }
        if self.shift_reg & 1 != 0 {
            return 0;
        }
        self.current_volume()
    }

    pub fn length_counter_active(&self) -> bool {
        self.length_counter > 0
    }

    pub fn save_state(&self) -> crate::savestate::NoiseChannelState {
        crate::savestate::NoiseChannelState {
            env: self.env,
            mode_period: self.mode_period,
            length: self.length,
            timer_period: self.timer_period,
            timer_counter: self.timer_counter,
            shift_reg: self.shift_reg,
            length_counter: self.length_counter,
            length_halt: self.length_halt,
            envelope_start: self.envelope_start,
            envelope_decay: self.envelope_decay,
            envelope_counter: self.envelope_counter,
            constant_volume: self.constant_volume,
            volume_or_period: self.volume_or_period,
            mode_short: self.mode_short,
            enabled: self.enabled,
        }
    }
    pub fn load_state(&mut self, state: &crate::savestate::NoiseChannelState) {
        self.env = state.env;
        self.mode_period = state.mode_period;
        self.length = state.length;
        self.timer_period = state.timer_period;
        self.timer_counter = state.timer_counter;
        self.shift_reg = state.shift_reg;
        self.length_counter = state.length_counter;
        self.length_halt = state.length_halt;
        self.envelope_start = state.envelope_start;
        self.envelope_decay = state.envelope_decay;
        self.envelope_counter = state.envelope_counter;
        self.constant_volume = state.constant_volume;
        self.volume_or_period = state.volume_or_period;
        self.mode_short = state.mode_short;
        self.enabled = state.enabled;
    }
}
