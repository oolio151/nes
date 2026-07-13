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
