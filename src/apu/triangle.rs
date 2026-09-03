// read the note in noise
use super::consts::{LENGTH_TABLE, TRIANGLE_TABLE};

pub struct TriangleChannel {
    linear_ctrl: u8,
    timer_lo: u8,
    length_timer_hi: u8,

    timer_period: u16,
    timer_counter: u16,
    sequencer_pos: u8,

    length_counter: u8,
    control_flag: bool,

    linear_counter: u8,
    linear_reload_value: u8,
    linear_reload_flag: bool,

    enabled: bool,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self {
            linear_ctrl: 0,
            timer_lo: 0,
            length_timer_hi: 0,

            timer_period: 0,
            timer_counter: 0,
            sequencer_pos: 0,

            length_counter: 0,
            control_flag: false,

            linear_counter: 0,
            linear_reload_value: 0,
            linear_reload_flag: false,

            enabled: false,
        }
    }

    pub fn write_linear_ctrl(&mut self, data: u8) {
        self.linear_ctrl = data;
        self.control_flag = data & 0x80 != 0;
        self.linear_reload_value = data & 0x7F;
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
        self.linear_reload_flag = true;
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
            if self.length_counter > 0 && self.linear_counter > 0 {
                self.sequencer_pos = (self.sequencer_pos + 1) & 31;
            }
        } else {
            self.timer_counter -= 1;
        }
    }

    pub fn clock_linear_counter(&mut self) {
        if self.linear_reload_flag {
            self.linear_counter = self.linear_reload_value;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        if !self.control_flag {
            self.linear_reload_flag = false;
        }
    }

    pub fn clock_length_counter(&mut self) {
        if !self.control_flag && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled {
            return 0;
        }
        TRIANGLE_TABLE[self.sequencer_pos as usize]
    }

    pub fn length_counter_active(&self) -> bool {
        self.length_counter > 0
    }
}