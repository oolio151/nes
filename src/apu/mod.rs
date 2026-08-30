use std::cell::Cell;
/*
APU register reference (in CPU address space)
0x4000-0x4003 pulse (square) 1 timer, length counter, envelope, sweep
0x4004-0x4007 pulse 2 timer, lc, envl, swp
0x4008-0x400B triangle timer, lc, envl, swp
0x400C-0x400F noise timer, length counter, envelope, linear feedback shift reg
0x4010-0x4013 DMC timer, memory reader, sample bugger, output unit
0x4015 channel enable and length counter status
0x4017 frame counter

https://www.nesdev.org/wiki/APU_registers
*/
pub struct APU {
    // register stuff
    // pulse 1 channel
    pulse1_duty_env: u8, // 00
    pulse1_sweep: u8, // 01
    pulse1_timer_lo: u8, // 02
    pulse1_length_timer_hi: u8, // 03

    // pulse 2 channel
    pulse2_duty_env: u8, // 04
    pulse2_sweep: u8, // 05
    pulse2_timer_lo: u8, // 06
    pulse2_length_timer_hi: u8, // 07

    // triangle
    triangle_linear_ctrl: u8, // 08
    triangle_timer_lo: u8, // 0A
    triangle_length_timer_hi: u8, //0B

    // noise channel
    noise_env: u8,
    noise_mode_period: u8,
    noise_length: u8,

    // dmc channel
    dmc_ctrl: u8,
    dmc_direct_load: u8,
    dmc_sample_addr: u8,
    dmc_sample_len: u8,

    // misc registers
    status: u8,
    frame_counter: u8,

    // open-bus latch
    io_latch: Cell<u8>,

    frame_cycle: u32,
    frame_step: u8,
    mode_5step: bool,
    irq_inhibit: bool,
    frame_irq_pending: Cell<bool>,
}

impl APU {

    pub fn new() -> Self {
        Self {
            pulse1_duty_env: 0,
            pulse1_sweep: 0,
            pulse1_timer_lo: 0,
            pulse1_length_timer_hi: 0,

            pulse2_duty_env: 0,
            pulse2_sweep: 0,
            pulse2_timer_lo: 0,
            pulse2_length_timer_hi: 0,

            triangle_linear_ctrl: 0,
            triangle_timer_lo: 0,
            triangle_length_timer_hi: 0,

            noise_env: 0,
            noise_mode_period: 0,
            noise_length: 0,

            dmc_ctrl: 0,
            dmc_direct_load: 0,
            dmc_sample_addr: 0,
            dmc_sample_len: 0,

            status: 0,
            frame_counter: 0,

            io_latch: Cell::new(0),
            
            frame_cycle: 0,
            frame_step: 0,
            mode_5step: false,
            irq_inhibit: false,
            frame_irq_pending: Cell::new(false),
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 => self.pulse1_duty_env = data,
            0x4001 => self.pulse1_sweep = data,
            0x4002 => self.pulse1_timer_lo = data,
            0x4003 => self.pulse1_length_timer_hi = data,

            0x4004 => self.pulse2_duty_env = data,
            0x4005 => self.pulse2_sweep = data,
            0x4006 => self.pulse2_timer_lo = data,
            0x4007 => self.pulse2_length_timer_hi = data,

            0x4008 => self.triangle_linear_ctrl = data,
            0x4009 => {}
            0x400A => self.triangle_timer_lo = data,
            0x400B => self.triangle_length_timer_hi = data,

            0x400C => self.noise_env = data,
            0x400D => {}
            0x400E => self.noise_mode_period = data,
            0x400F => self.noise_length = data,

            0x4010 => self.dmc_ctrl = data,
            0x4011 => self.dmc_direct_load = data,
            0x4012 => self.dmc_sample_addr = data,
            0x4013 => self.dmc_sample_len = data,

            0x4015 => self.status = data,
            0x4017 => {
                self.frame_counter = data;
                self.mode_5step = data & 0x80 != 0;
                self.irq_inhibit = data & 0x40 != 0;
                self.frame_cycle = 0;
                self.frame_step = 0;
                if self.irq_inhibit {
                    self.frame_irq_pending.set(false);
                }
            }

            _ => {}
        }

        self.io_latch.set(data);
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                let was_pending = self.frame_irq_pending.get();
                self.frame_irq_pending.set(false);
                let status_bytes = (was_pending as u8) << 6;
                self.io_latch.set(status_bytes);
                status_bytes
            }

            _ => self.io_latch.get(),
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        self.frame_cycle += cycles;

        loop {
            let step_count = if self.mode_5step { 5 } else { 4 };
            let boundaries: &[u32] = if self.mode_5step {
                &[7457, 14913, 22371, 29829, 37281]
            } else {
                &[7457, 14913, 22371, 29829]
            };

            let boundary = boundaries[self.frame_step as usize];
            if self.frame_cycle < boundary {
                break;
            }

            self.fire_step(self.frame_step);

            self.frame_step += 1;
            if self.frame_step >= step_count {
                self.frame_step = 0;
                self.frame_cycle -= boundary;
            }
        }
    }

    fn fire_step(&mut self, step: u8) {
        let is_last_step = if self.mode_5step { step == 4 } else { step == 3 };

        let clocks_envelope = !(self.mode_5step && step == 3);
        if clocks_envelope {
            // TODO: clock envelope + linear counter on channels once they exist
        }

        let clocks_length_sweep = if self.mode_5step {
            step == 1 || step == 4
        } else {
            step == 1 || step == 3
        };
        if clocks_length_sweep {
            // TODO: clock length counters + sweep units on channels once they exist
        }

        if !self.mode_5step && is_last_step && !self.irq_inhibit {
            self.frame_irq_pending.set(true);
        }
    }

    pub fn frame_irq_pending(&self) -> bool {
        self.frame_irq_pending.get()
    }
}