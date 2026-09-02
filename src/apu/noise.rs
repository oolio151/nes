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
}