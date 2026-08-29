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
    io_latch: u8,
}

impl APU {
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
            0x4017 => self.frame_counter = data,

            _ => {}
        }

        self.io_latch = data;
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                // TODO: build this from real length-counter/IRQ state once channels exist (IF-D NT21)
                let status_bytes = 0; // replace with real bit assembly later
                self.io_latch = status_bytes;
                status_bytes
            }

            _ => self.io_latch,
        }
    }
}