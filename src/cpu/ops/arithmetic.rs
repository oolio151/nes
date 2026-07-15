use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{immediate, zeropage, zeropagex, absolute, absolutex, absolutey, indirectx, indirecty};

// add with carry
fn adc(cpu: &mut CPU, value : u8) {
    let carry_in = cpu.p & 0x01;

    let sum: u16 = cpu.a as u16 + value as u16 + carry_in as u16;
    let result: u8 = sum as u8;

    cpu.set_flag(Flag::Carry, sum > 0xFF);

    cpu.set_flag(Flag::Zero, result == 0);

    let overflow = (result ^ cpu.a) & (result ^ value) & 0x80;
    cpu.set_flag(Flag::Overflow, overflow != 0);

   cpu.set_flag(Flag::Negative, result & 0x80 != 0);

    cpu.a = result;
}

pub fn adc_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    adc(cpu, value);

    0
}

pub fn adc_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    adc(cpu, cpu.read(value));

    0
}

pub fn adc_zeropagex(cpu: &mut CPU) -> u8 {
    let value = zeropagex(cpu);
    adc(cpu, cpu.read(value));
    0
}

pub fn adc_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    adc(cpu, cpu.read(value));
    0
}

pub fn adc_absolutex(cpu: &mut CPU) -> u8 {
    let value = absolutex(cpu);
    adc(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

pub fn adc_absolutey(cpu: &mut CPU) -> u8 {
    let value = absolutey(cpu);
    adc(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

pub fn adc_indirectx(cpu: &mut CPU) -> u8 {
    let value = indirectx(cpu);
    adc(cpu, cpu.read(value));
    0
}

pub fn adc_indirecty(cpu: &mut CPU) -> u8 {
    let value = indirecty(cpu);
    adc(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

fn dec(cpu: &mut CPU, value: u8) -> u8 {
    let result = value.wrapping_sub(1);

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);

    result
}

pub fn dec_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);

    let result = dec(cpu, value);

    cpu.write(addr, result);

    0
}


pub fn dec_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read(addr);

    let result = dec(cpu, value);

    cpu.write(addr, result);

    0
}


pub fn dec_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);

    let result = dec(cpu, value);

    cpu.write(addr, result);

    0
}

pub fn dec_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    let value = cpu.read(addr);

    let result = dec(cpu, value);

    cpu.write(addr, result);

    0
}

pub fn dex(cpu: &mut CPU) -> u8 {

    let result = cpu.x.wrapping_sub(1);
    cpu.x = result;

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);

    0
}

pub fn dey(cpu: &mut CPU) -> u8 {

    let result = cpu.y.wrapping_sub(1);
    cpu.y = result;

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);

    0
}

    fn inc(cpu: &mut CPU, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        cpu.set_flag(Flag::Zero, result == 0);
        cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);

        result
    }

    pub fn inc_zeropage(cpu: &mut CPU) -> u8 {
        let addr = zeropage(cpu);
        let value = cpu.read(addr);

        let result = inc(cpu, value);

        cpu.write(addr, result);

        0
    }


    pub fn inc_zeropagex(cpu: &mut CPU) -> u8 {
        let addr = zeropagex(cpu);
        let value = cpu.read(addr);

        let result = inc(cpu, value);

        cpu.write(addr, result);

        0
    }


    pub fn inc_absolute(cpu: &mut CPU) -> u8 {
        let addr = absolute(cpu);
        let value = cpu.read(addr);

        let result = inc(cpu, value);

        cpu.write(addr, result);

        0
    }

    pub fn inc_absolutex(cpu: &mut CPU) -> u8 {
        let (addr, _page_crossed) = absolutex(cpu);
        let value = cpu.read(addr);

        let result = inc(cpu, value);

        cpu.write(addr, result);

        0
    }

    pub fn inx(cpu: &mut CPU) -> u8 {

        let result = cpu.x.wrapping_add(1);
        cpu.x = result;

        cpu.set_flag(Flag::Zero, result == 0);
        cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);

        0
    }

    pub fn iny(cpu: &mut CPU) -> u8 {

        let result = cpu.y.wrapping_add(1);
        cpu.y = result;

        cpu.set_flag(Flag::Zero, result == 0);
        cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);

        0
    }