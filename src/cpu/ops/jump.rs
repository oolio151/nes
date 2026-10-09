use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::absolute;

pub fn brk(cpu: &mut CPU) -> u8 {
    cpu.read_cycle(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);
    
    cpu.write(0x0100 + cpu.s as u16, (cpu.pc >> 8) as u8);
    cpu.s = cpu.s.wrapping_sub(1);
    cpu.write(0x0100 + cpu.s as u16, (cpu.pc & 0xFF) as u8);
    cpu.s = cpu.s.wrapping_sub(1);

    let pushed_flags = cpu.p | 0b0011_0000;
    cpu.write(0x0100 + cpu.s as u16, pushed_flags);
    cpu.s = cpu.s.wrapping_sub(1);

    cpu.set_flag(Flag::InterruptDisable, true);

    let lo = cpu.read_cycle(0xFFFE) as u16;
    let hi = cpu.read_cycle(0xFFFF) as u16;
    cpu.pc = (hi << 8) | lo;

    0
}

// jmp is weird af so the functions dont follow usual structure
pub fn jmp_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    cpu.pc = addr;

    0
}

// theres a cpu bug where it doesnt page cross properly
pub fn jmp_indirect(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);

    let lo = cpu.read_cycle(addr);

    let hi = if (addr & 0x00FF) == 0x00FF {
        cpu.read_cycle(addr & 0xFF00)
    } else {
        cpu.read_cycle(addr + 1)
    };

    let target: u16 = ((hi as u16) << 8) | (lo as u16);

    cpu.pc = target;

    0
}

pub fn jsr(cpu: &mut CPU) -> u8 {
    let lo = cpu.read_cycle(cpu.pc) as u16;
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.read_cycle(0x0100 + cpu.s as u16);
    let return_addr = cpu.pc;
    cpu.push((return_addr >> 8) as u8);
    cpu.push(return_addr as u8);
    let hi = cpu.read_cycle(cpu.pc) as u16;
    cpu.pc = (hi << 8) | lo;
    0
}

pub fn rti(cpu: &mut CPU) -> u8 {
    cpu.read_cycle(cpu.pc);
    cpu.read_cycle(0x0100 + cpu.s as u16);
    cpu.s = cpu.s.wrapping_add(1);
    cpu.p = (cpu.read_cycle(0x0100 + cpu.s as u16) & 0b1110_1111) | 0b0010_0000;

    cpu.s = cpu.s.wrapping_add(1);
    let lo = cpu.read_cycle(0x0100 + cpu.s as u16) as u16;

    cpu.s = cpu.s.wrapping_add(1);
    let hi = cpu.read_cycle(0x0100 + cpu.s as u16) as u16;

    cpu.pc = (hi << 8) | lo;

    0
}

pub fn rts(cpu: &mut CPU) -> u8 {
    cpu.read_cycle(cpu.pc);
    cpu.read_cycle(0x0100 + cpu.s as u16);
    cpu.s = cpu.s.wrapping_add(1);
    let lo = cpu.read_cycle(0x0100 + cpu.s as u16) as u16;

    cpu.s = cpu.s.wrapping_add(1);
    let hi = cpu.read_cycle(0x0100 + cpu.s as u16) as u16;

    cpu.pc = (hi << 8) | lo;
    cpu.read_cycle(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);

    0
}