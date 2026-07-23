use crate::cpu::CPU;
use crate::cpu::Flag;

pub fn tax(cpu: &mut CPU) -> u8 {
    cpu.x = cpu.a;

    cpu.set_flag(Flag::Zero, cpu.x == 0);
    cpu.set_flag(Flag::Negative, cpu.x & 0x80 != 0);

    0
}

pub fn tay(cpu: &mut CPU) -> u8 {
    cpu.y = cpu.a;

    cpu.set_flag(Flag::Zero, cpu.y == 0);
    cpu.set_flag(Flag::Negative, cpu.y & 0x80 != 0);

    0
}

pub fn txa(cpu: &mut CPU) -> u8 {
    cpu.a = cpu.x;

    cpu.set_flag(Flag::Zero, cpu.a == 0);
    cpu.set_flag(Flag::Negative, cpu.a & 0x80 != 0);

    0
}

pub fn tya(cpu: &mut CPU) -> u8 {
    cpu.a = cpu.y;

    cpu.set_flag(Flag::Zero, cpu.a == 0);
    cpu.set_flag(Flag::Negative, cpu.a & 0x80 != 0);

    0
}

pub fn tsx(cpu: &mut CPU) -> u8 {
    cpu.x = cpu.s;

    cpu.set_flag(Flag::Zero, cpu.x == 0);
    cpu.set_flag(Flag::Negative, cpu.x & 0x80 != 0);

    0
}

pub fn txs(cpu: &mut CPU) -> u8 {
    cpu.s = cpu.x;

    0
}