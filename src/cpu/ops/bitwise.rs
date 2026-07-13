use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{immediate, zeropage, zeropagex, absolute, absolutex, absolutey, indirectx, indirecty};

// bitwise and

fn and(cpu: &mut CPU, value : u8){
    let result = cpu.a & value;
    cpu.a = result;

    cpu.set_flag(Flag::Zero, result == 0);

    cpu.set_flag(Flag::Negative, result & 0x80 != 0);
}

pub fn and_immediate(cpu: &mut CPU) -> u8{
    let value = immediate(cpu);
    and(cpu, value);

    0
}

pub fn and_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    and(cpu, cpu.read(value));

    0
}

pub fn and_zeropagex(cpu: &mut CPU) -> u8 {
    let value = zeropagex(cpu);
    and(cpu, cpu.read(value));
    0
}

pub fn and_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    and(cpu, cpu.read(value));
    0
}

pub fn and_absolutex(cpu: &mut CPU) -> u8 {
    let value = absolutex(cpu);
    and(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

pub fn and_absolutey(cpu: &mut CPU) -> u8 {
    let value = absolutey(cpu);
    and(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

pub fn and_indirectx(cpu: &mut CPU) -> u8 {
    let value = indirectx(cpu);
    and(cpu, cpu.read(value));
    0
}

pub fn and_indirecty(cpu: &mut CPU) -> u8 {
    let value = indirecty(cpu);
    and(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

// bit test
pub fn bit(cpu: &mut CPU, value : u8) {
    let result = cpu.a & value;

    if result == 0 {
        cpu.set_flag(Flag::Zero, true);
    } 

    cpu.set_flag(Flag::Overflow, value & 0b01000000 != 0);

    cpu.set_flag(Flag::Negative, value & 0b10000000 != 0);
    
}

pub fn bit_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    bit(cpu, cpu.read(value));

    0
}

pub fn bit_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    bit(cpu, cpu.read(value));

    0
}