use super::consts::DMC_RATE_TABLE;
use std::cell::Cell;

pub struct DmcChannel {
    irq_enable: bool,
    loop_flag: bool,
    rate_index: u8,
    sample_addr_reg: u8,
    sample_length_reg: u8,

    timer_period: u16,
    timer_counter: u16,

    current_addr: u16,
    bytes_remaining: u16,
    sample_buffer: Option<u8>,

    shift_register: u8,
    bits_remaining: u8,
    output_level: u8,
    silence_flag: bool,

    enabled: bool,
    irq_pending: Cell<bool>,
}

impl DmcChannel {
    pub fn new() -> Self {
        Self {
            irq_enable: false,
            loop_flag: false,
            rate_index: 0,
            sample_addr_reg: 0,
            sample_length_reg: 0,

            timer_period: DMC_RATE_TABLE[0],
            timer_counter: 0,

            current_addr: 0xC000,
            bytes_remaining: 0,
            sample_buffer: None,

            shift_register: 0,
            bits_remaining: 8,
            output_level: 0,
            silence_flag: true,

            enabled: false,
            irq_pending: Cell::new(false),
        }
    }

    pub fn write_ctrl(&mut self, data: u8) {
        self.irq_enable = data & 0x80 != 0;
        self.loop_flag = data & 0x40 != 0;
        self.rate_index = data & 0x0F;
        self.timer_period = DMC_RATE_TABLE[self.rate_index as usize];
        if !self.irq_enable {
            self.irq_pending.set(false);
        }
    }

    pub fn write_direct_load(&mut self, data: u8) {
        self.output_level = data & 0x7F;
    }

    pub fn write_sample_addr(&mut self, data: u8) {
        self.sample_addr_reg = data;
    }

    pub fn write_sample_length(&mut self, data: u8) {
        self.sample_length_reg = data;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.current_addr = 0xC000 + (self.sample_addr_reg as u16) * 64;
            self.bytes_remaining = (self.sample_length_reg as u16) * 16 + 1;
        }
    }

    pub fn clear_irq(&mut self) {
        self.irq_pending.set(false);
    }

    pub fn irq_pending(&self) -> bool {
        self.irq_pending.get()
    }

    pub fn is_active(&self) -> bool {
        self.bytes_remaining > 0
    }

    pub fn tick_timer(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;
            self.clock_output_unit();
        } else {
            self.timer_counter -= 1;
        }
    }

    fn clock_output_unit(&mut self) {
        if !self.silence_flag {
            if self.shift_register & 1 != 0 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else if self.output_level >= 2 {
                self.output_level -= 2;
            }
        }
        self.shift_register >>= 1;

        if self.bits_remaining > 0 {
            self.bits_remaining -= 1;
        }
        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
            if let Some(byte) = self.sample_buffer.take() {
                self.silence_flag = false;
                self.shift_register = byte;
            } else {
                self.silence_flag = true;
            }
        }
    }

    pub fn fetch_request(&self) -> Option<u16> {
        if self.sample_buffer.is_none() && self.bytes_remaining > 0 {
            Some(self.current_addr)
        } else {
            None
        }
    }

    pub fn provide_byte(&mut self, byte: u8) {
        self.sample_buffer = Some(byte);
        self.current_addr = if self.current_addr == 0xFFFF { 0x8000 } else { self.current_addr + 1 };
        self.bytes_remaining -= 1;

        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.current_addr = 0xC000 + (self.sample_addr_reg as u16) * 64;
                self.bytes_remaining = (self.sample_length_reg as u16) * 16 + 1;
            } else if self.irq_enable {
                self.irq_pending.set(true);
            }
        }
    }

    pub fn output(&self) -> u8 {
        self.output_level
    }
}