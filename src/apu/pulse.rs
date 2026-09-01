use super::consts::{LENGTH_TABLE, DUTY_TABLE};
pub struct PulseChannel {
    duty_env: u8,
    sweep: u8,
    timer_lo: u8,
    length_timer_hi: u8,

    timer_period: u16,
    timer_counter: u16,
    sequencer_pos: u8,

    length_counter: u8,
    length_halt: bool,

    envelope_start: bool,
    envelope_decay: u8,
    envelope_counter: u8,
    constant_volume: bool,
    volume_or_period: u8,

    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_counter: u8,
    sweep_reload: bool,

    enabled: bool,

    is_channel2: bool,
}

impl PulseChannel {
    pub fn new(is_channel2: bool) -> Self {
        Self {
            duty_env: 0,
            sweep: 0,
            timer_lo: 0,
            length_timer_hi: 0,

            timer_period: 0,
            timer_counter: 0,
            sequencer_pos: 0,

            length_counter: 0,
            length_halt: false,

            envelope_start: false,
            envelope_decay: 0,
            envelope_counter: 0,
            constant_volume: false,
            volume_or_period: 0,

            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_counter: 0,
            sweep_reload: false,

            enabled: false,
            is_channel2,
        }
    }

    pub fn write_duty_env(&mut self, data: u8) {
        self.duty_env = data;
        self.length_halt = data & 0x20 != 0;
        self.constant_volume = data & 0x10 != 0;
        self.volume_or_period = data & 0x0F;
    }

    pub fn write_sweep(&mut self, data: u8) {
        self.sweep = data;
        self.sweep_enabled = data & 0x80 != 0;
        self.sweep_period = (data >> 4) & 0x07;
        self.sweep_negate = data & 0x08 != 0;
        self.sweep_shift = data & 0x07;
        self.sweep_reload = true;
    }

    pub fn write_timer_lo(&mut self, data: u8) {
        self.timer_lo = data;
        self.timer_period = (self.timer_period & 0xFF00) | data as u16;
    }

    pub fn write_length_timer_hi(&mut self, data: u8) {
        self.length_timer_hi = data;
        self.timer_period = (self.timer_period & 0x00FF) | (((data & 0x07) as u16) << 8);

        if self.enabled {
            self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        }

        self.sequencer_pos = 0;
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
            self.sequencer_pos = (self.sequencer_pos + 1) & 7;
        } else {
            self.timer_counter -= 1;
        }
    }

    pub fn current_duty_output(&self) -> u8 {
        let duty = (self.duty_env >> 6) & 0x03;
        DUTY_TABLE[duty as usize][self.sequencer_pos as usize]
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

    pub fn current_volume(&self) -> u8 {
        if self.constant_volume {
            self.volume_or_period
        } else {
            self.envelope_decay
        }
    }

    pub fn clock_length_counter(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    // highkey claude wrote ts
    pub fn clock_sweep(&mut self) {
        let change_amount = self.timer_period >> self.sweep_shift;

        let target_period = if self.sweep_negate {
            if self.is_channel2 {
                self.timer_period.wrapping_sub(change_amount)
            } else {
                self.timer_period.wrapping_sub(change_amount).wrapping_sub(1)
            }
        } else {
            self.timer_period.wrapping_add(change_amount)
        };

        let muting = self.timer_period < 8 || target_period > 0x7FF;

        if self.sweep_counter == 0 && self.sweep_enabled && self.sweep_shift > 0 && !muting {
            self.timer_period = target_period;
        }

        if self.sweep_counter == 0 || self.sweep_reload {
            self.sweep_counter = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_counter -= 1;
        }
    }
    }