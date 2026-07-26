use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::absolutey;

pub fn pha(cpu: &mut CPU) -> u8 {
    cpu.write(0x0100 + cpu.s as u16, cpu.a);
    cpu.s = cpu.s.wrapping_sub(1);

    0
}

pub fn php(cpu: &mut CPU) -> u8 {
    let pushed = cpu.p | 0b0011_0000;
    cpu.write(0x0100 + cpu.s as u16, pushed);
    cpu.s = cpu.s.wrapping_sub(1);

    0
}

pub fn pla(cpu: &mut CPU) -> u8 {
    cpu.s = cpu.s.wrapping_add(1);
    let value = cpu.read(0x0100 + cpu.s as u16);

    cpu.set_flag(Flag::Zero, value == 0);
    cpu.set_flag(Flag::Negative, value & 0b1000_0000 != 0);

    cpu.a = value;

    0
}

pub fn plp(cpu: &mut CPU) -> u8 {
    cpu.s = cpu.s.wrapping_add(1);
    let pulled = cpu.read(0x0100 + cpu.s as u16);

    cpu.p = (pulled & 0b1110_1111) | 0b0010_0000;

    0
}

// START OF UNOFFICIAL OPCODES
pub fn tas_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutey(cpu);

    cpu.s = cpu.a & cpu.x;

    let hi = (addr >> 8) as u8;
    let value = cpu.s & hi.wrapping_add(1);
    cpu.write(addr, value);

    0
}