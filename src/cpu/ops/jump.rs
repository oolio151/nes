use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{immediate, zeropage, absolute, absolutex, absolutey, indirectx, indirecty};

pub fn brk(cpu: &mut CPU) -> u8 {
    cpu.pc += 1;
    
    cpu.write(0x0100 + cpu.s as u16, (cpu.pc >> 8) as u8);
    cpu.s = cpu.s.wrapping_sub(1);
    cpu.write(0x0100 + cpu.s as u16, (cpu.pc & 0xFF) as u8);
    cpu.s = cpu.s.wrapping_sub(1);

    let pushed_flags = cpu.p | 0b0011_0000;
    cpu.write(0x0100 + cpu.s as u16, pushed_flags);
    cpu.s = cpu.s.wrapping_sub(1);

    cpu.set_flag(Flag::InterruptDisable, true);

    let lo = cpu.read(0xFFFE) as u16;
    let hi = cpu.read(0xFFFF) as u16;
    cpu.pc = (hi << 8) | lo;
    
    0
}