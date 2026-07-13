use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{immediate, zeropage, zeropagex, absolute, absolutex, absolutey, indirectx, indirecty, relative};

pub fn bcc(cpu: &mut CPU) -> u8 {
    let value = relative(cpu);

    if cpu.get_flag(Flag::Carry) {
        0
    } else {
        cpu.pc = value.0;

        if value.1 {2} else {1}
    }
}

pub fn bcs(cpu: &mut CPU) -> u8 {
    let value = relative(cpu);

    if !cpu.get_flag(Flag::Carry) {
        0
    } else {
        cpu.pc = value.0;

        if value.1 {2} else {1}
    }
}

pub fn beq(cpu: &mut CPU) -> u8 {
    let value = relative(cpu);

    if !cpu.get_flag(Flag::Zero) {
        0
    } else {
        cpu.pc = value.0;

        if value.1 {2} else {1}
    }
}



pub fn bmi(cpu: &mut CPU) -> u8 {
    0
}

pub fn bne(cpu: &mut CPU) -> u8 {
    0
}

pub fn bpl(cpu: &mut CPU) -> u8 {
    0
}