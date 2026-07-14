use crate::cpu::CPU;
use crate::cpu::Flag;
//use crate::cpu::opcodes::{immediate, zeropage, absolute, absolutex, absolutey, indirectx, indirecty};]

pub fn clc(cpu: &mut CPU) -> u8{
    cpu.set_flag(Flag::Carry, false);

    0
}

pub fn cld(cpu: &mut CPU) -> u8 {
    cpu.set_flag(Flag::Decimal, false);

    0
}

pub fn cli(cpu: &mut CPU) -> u8 {
    cpu.set_flag(Flag::InterruptDisable, false);

    0
}

pub fn clv(cpu: &mut CPU) -> u8 {
    cpu.set_flag(Flag::Overflow, false);

    0
}