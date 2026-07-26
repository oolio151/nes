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

pub fn sta_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    cpu.write(addr, cpu.a);

    0
}

pub fn sta_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    cpu.write(addr, cpu.a);

    0
}

pub fn sta_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    cpu.write(addr, cpu.a);

    0
}

pub fn sta_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    cpu.write(addr, cpu.a);

    0
}

pub fn sta_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutey(cpu);
    cpu.write(addr, cpu.a);

    0
}

pub fn sta_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    cpu.write(addr, cpu.a);

    0
}

pub fn sta_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = indirecty(cpu);
    cpu.write(addr, cpu.a);

    0
}

pub fn stx_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    cpu.write(addr, cpu.x);

    0
}

pub fn stx_zeropagey(cpu: &mut CPU) -> u8 {
    let addr = zeropagey(cpu);
    cpu.write(addr, cpu.x);

    0
}

pub fn stx_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    cpu.write(addr, cpu.x);

    0
}

pub fn sty_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    cpu.write(addr, cpu.y);

    0
}

pub fn sty_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    cpu.write(addr, cpu.y);

    0
}

pub fn sty_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    cpu.write(addr, cpu.y);

    0
}

// START OF UNOFFICIAL OPCODES

fn lax(cpu: &mut CPU, value: u8) {
    cpu.a = value;
    cpu.x = value;

    cpu.set_flag(Flag::Zero, value == 0);
    cpu.set_flag(Flag::Negative, value & 0x80 != 0);
}

pub fn lax_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);

    lax(cpu, value);

    0
}

pub fn lax_zeropagey(cpu: &mut CPU) -> u8 {
    let addr = zeropagey(cpu);
    let value = cpu.read(addr);

    lax(cpu, value);

    0
}

pub fn lax_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);

    lax(cpu, value);

    0
}

pub fn lax_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutey(cpu);
    let value = cpu.read(addr);

    lax(cpu, value);

    if page_crossed { 1 } else { 0 }
}

pub fn lax_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    let value = cpu.read(addr);

    lax(cpu, value);

    0
}

pub fn lax_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = indirecty(cpu);
    let value = cpu.read(addr);

    lax(cpu, value);

    if page_crossed { 1 } else { 0 }
}

pub fn sax_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    cpu.write(addr, cpu.a & cpu.x);

    0
}

pub fn sax_zeropagey(cpu: &mut CPU) -> u8 {
    let addr = zeropagey(cpu);
    cpu.write(addr, cpu.a & cpu.x);

    0
}

pub fn sax_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    cpu.write(addr, cpu.a & cpu.x);

    0
}

pub fn sax_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    cpu.write(addr, cpu.a & cpu.x);

    0
}

pub fn shy_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    let hi = (addr >> 8) as u8;
    let value = cpu.y & hi.wrapping_add(1);
    cpu.write(addr, value);

    0
}

pub fn shx_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutey(cpu);
    let hi = (addr >> 8) as u8;
    let value = cpu.x & hi.wrapping_add(1);
    cpu.write(addr, value);

    0
}

pub fn ahx_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = indirecty(cpu);
    let hi = (addr >> 8) as u8;
    let value = cpu.a & cpu.x & hi.wrapping_add(1);
    cpu.write(addr, value);

    0
}

pub fn ahx_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutey(cpu);
    let hi = (addr >> 8) as u8;
    let value = cpu.a & cpu.x & hi.wrapping_add(1);
    cpu.write(addr, value);

    0
}

pub fn las_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutey(cpu);
    let value = cpu.read(addr) & cpu.s;

    cpu.a = value;
    cpu.x = value;
    cpu.s = value;

    cpu.set_flag(Flag::Zero, value == 0);
    cpu.set_flag(Flag::Negative, value & 0x80 != 0);

    if page_crossed { 1 } else { 0 }
}

pub fn lxa_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    let result = (cpu.a | 0xEE) & value;

    cpu.a = result;
    cpu.x = result;

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0x80 != 0);

    0
}