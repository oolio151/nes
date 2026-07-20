use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{immediate, zeropage, zeropagex, absolute, absolutex, absolutey, indirectx, indirecty, zeropagey};

fn lda(cpu: &mut CPU, value: u8) {
    cpu.a = value;

    cpu.set_flag(Flag::Zero, value == 0);
    cpu.set_flag(Flag::Negative, value & 0x80 != 0);
}

pub fn lda_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);

    lda(cpu, value);

    0
}

pub fn lda_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);

    lda(cpu, value);

    0
}

pub fn lda_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read(addr);

    lda(cpu, value);

    0
}

pub fn lda_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);

    lda(cpu, value);

    0
}

pub fn lda_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutex(cpu);
    let value = cpu.read(addr);

    lda(cpu, value);

    if page_crossed { 1 } else { 0 }
}

pub fn lda_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutey(cpu);
    let value = cpu.read(addr);

    lda(cpu, value);

    if page_crossed { 1 } else { 0 }
}

pub fn lda_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    let value = cpu.read(addr);

    lda(cpu, value);

    0
}

pub fn lda_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = indirecty(cpu);
    let value = cpu.read(addr);

    lda(cpu, value);

    if page_crossed { 1 } else { 0 }
}


fn ldx(cpu: &mut CPU, value: u8) {
    cpu.x = value;

    cpu.set_flag(Flag::Zero, value == 0);
    cpu.set_flag(Flag::Negative, value & 0x80 != 0);
}

pub fn ldx_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);

    ldx(cpu, value);

    0
}

pub fn ldx_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);

    ldx(cpu, value);

    0
}

pub fn ldx_zeropagey(cpu: &mut CPU) -> u8 {
    let addr = zeropagey(cpu);
    let value = cpu.read(addr);

    ldx(cpu, value);

    0
}

pub fn ldx_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);

    ldx(cpu, value);

    0
}

pub fn ldx_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutey(cpu);
    let value = cpu.read(addr);

    ldx(cpu, value);

    if page_crossed { 1 } else { 0 }
}


fn ldy(cpu: &mut CPU, value: u8) {
    cpu.y = value;

    cpu.set_flag(Flag::Zero, value == 0);
    cpu.set_flag(Flag::Negative, value & 0x80 != 0);
}

pub fn ldy_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);

    ldy(cpu, value);

    0
}

pub fn ldy_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);

    ldy(cpu, value);

    0
}

pub fn ldy_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read(addr);

    ldy(cpu, value);
    
    0
}

pub fn ldy_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);
    
    ldy(cpu, value);

    0
}

pub fn ldy_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutex(cpu);
    let value = cpu.read(addr);

    ldy(cpu, value);
    
    if page_crossed { 1 } else { 0 }
}