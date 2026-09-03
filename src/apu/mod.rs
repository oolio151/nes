/*
bugs im deferring for later because i am lazy
- dmc stall cycles on the cpu
- some sub cycle stuff on write to 0x4017


*/

use std::cell::Cell;

pub mod consts;
pub mod pulse;
pub mod triangle;
pub mod noise;
pub mod dmc;

use crate::apu::pulse::PulseChannel;
use crate::apu::noise::NoiseChannel;
use crate::apu::triangle::TriangleChannel;
use crate::apu::dmc::DmcChannel;
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
    // pulse 1 channel
    pulse1: PulseChannel,

    // pulse 2 channel
    pulse2: PulseChannel,
    cycle_parity: bool,

    // triangle
    triangle: TriangleChannel,

    // noise channel
    noise: NoiseChannel,

    // dmc channel
    dmc: DmcChannel,

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

    // used for proper timing of audio
    sample_acc: f32,
    cycles_per_sample: f32,
    sample_buffer: Vec<f32>,
}

impl APU {

    pub fn new() -> Self {
        Self {
            pulse1: PulseChannel::new(false),
            pulse2: PulseChannel::new(true),
            cycle_parity: false,

            triangle: TriangleChannel::new(),

            noise: NoiseChannel::new(),

            dmc: DmcChannel::new(),

            status: 0,
            frame_counter: 0,

            io_latch: Cell::new(0),
            
            frame_cycle: 0,
            frame_step: 0,
            mode_5step: false,
            irq_inhibit: false,
            frame_irq_pending: Cell::new(false),

            sample_acc: 0.0,
            cycles_per_sample: 1789773.0 / 44100.0,
            sample_buffer: Vec::new(),
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 => self.pulse1.write_duty_env(data),
            0x4001 => self.pulse1.write_sweep(data),
            0x4002 => self.pulse1.write_timer_lo(data),
            0x4003 => self.pulse1.write_length_timer_hi(data),

            0x4004 => self.pulse2.write_duty_env(data),
            0x4005 => self.pulse2.write_sweep(data),
            0x4006 => self.pulse2.write_timer_lo(data),
            0x4007 => self.pulse2.write_length_timer_hi(data),

            0x4008 => self.triangle.write_linear_ctrl(data),
            0x4009 => {}
            0x400A => self.triangle.write_timer_lo(data),
            0x400B => self.triangle.write_length_timer_hi(data),

            0x400C => self.noise.write_env(data),
            0x400D => {}
            0x400E => self.noise.write_mode_period(data),
            0x400F => self.noise.write_length(data),

            0x4010 => self.dmc.write_ctrl(data),
            0x4011 => self.dmc.write_direct_load(data),
            0x4012 => self.dmc.write_sample_addr(data),
            0x4013 => self.dmc.write_sample_length(data),

            0x4015 => {
                self.status = data;
                self.pulse1.set_enabled(data & 0x01 != 0);
                self.pulse2.set_enabled(data & 0x02 != 0);
                self.triangle.set_enabled(data & 0x04 != 0);
                self.noise.set_enabled(data & 0x08 != 0);
                self.dmc.set_enabled(data & 0x10 != 0);
                self.dmc.clear_irq();
            },

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

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                let mut status_bytes = 0u8;

                if self.pulse1.length_counter_active() {
                    status_bytes |= 0x01;
                }
                
                if self.pulse2.length_counter_active() {
                    status_bytes |= 0x02;
                }

                if self.triangle.length_counter_active() {
                    status_bytes |= 0x04;
                }

                if self.noise.length_counter_active() {
                    status_bytes |= 0x08;
                }

                if self.dmc.is_active() {
                    status_bytes |= 0x10;
                }

                if self.frame_irq_pending.get() {
                    status_bytes |= 0x40;
                }

                if self.dmc.irq_pending() {
                    status_bytes |= 0x80;
                }

                self.frame_irq_pending.set(false);

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

        for _ in 0..cycles {
            self.cycle_parity = !self.cycle_parity;
            if self.cycle_parity {
                self.pulse1.tick_timer();
                self.pulse2.tick_timer();
                self.noise.tick_timer();
                self.dmc.tick_timer();
            }
        }

        for _ in 0..cycles {
            self.triangle.tick_timer();
        }

        self.sample_acc += cycles as f32;
        while self.sample_acc >= self.cycles_per_sample {
            self.sample_acc -= self.cycles_per_sample;
            let s = self.sample();
            self.sample_buffer.push(s);
        }
    }

    fn fire_step(&mut self, step: u8) {
        let is_last_step = if self.mode_5step { step == 4 } else { step == 3 };

        let clocks_envelope = !(self.mode_5step && step == 3);
        if clocks_envelope {
            self.pulse1.clock_envelope();
            self.pulse2.clock_envelope();
            self.noise.clock_envelope();
            self.triangle.clock_linear_counter();
        }

        let clocks_length_sweep = if self.mode_5step {
            step == 1 || step == 4
        } else {
            step == 1 || step == 3
        };

        if clocks_length_sweep {
            self.pulse1.clock_length_counter();
            self.pulse1.clock_sweep();
            self.pulse2.clock_length_counter();
            self.pulse2.clock_sweep();
            self.triangle.clock_length_counter();
            self.noise.clock_length_counter();
        }

        if !self.mode_5step && is_last_step && !self.irq_inhibit {
            self.frame_irq_pending.set(true);
        }
    }

    pub fn frame_irq_pending(&self) -> bool {
        self.frame_irq_pending.get()
    }

    pub fn reset(&mut self) {
        self.status = 0;

        self.frame_cycle = 0;
        self.frame_step = 0;

        self.pulse1.set_enabled(false);
        self.pulse2.set_enabled(false);

        self.triangle.set_enabled(false);
        self.noise.set_enabled(false);
        
        self.dmc.set_enabled(false);
    }

    pub fn dmc_fetch_request(&self) -> Option<u16> {
        self.dmc.fetch_request()
    }

    pub fn dmc_provide_byte(&mut self, byte: u8) {
        self.dmc.provide_byte(byte);
    }

    pub fn dmc_irq_pending(&self) -> bool {
        self.dmc.irq_pending()
    }
    
    pub fn sample(&self) -> f32 {
        let pulse1 = self.pulse1.output() as f32;
        let pulse2 = self.pulse2.output() as f32;
        let triangle = self.triangle.output() as f32;
        let noise = self.noise.output() as f32;
        let dmc = self.dmc.output() as f32;

        let pulse_out = if pulse1 + pulse2 == 0.0 {
            0.0
        } else {
            95.88 / ((8128.0 / (pulse1 + pulse2)) + 100.0)
        };

        let tnd_sum = triangle / 8227.0 + noise / 12241.0 + dmc / 22638.0;
        let tnd_out = if tnd_sum == 0.0 {
            0.0
        } else {
            159.79 / ((1.0 / tnd_sum) + 100.0)
        };

        pulse_out + tnd_out
    }

    pub fn drain_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.sample_buffer)
    }
}