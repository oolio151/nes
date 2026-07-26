use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{zeropage, zeropagex, absolute, absolutex, indirectx, indirecty, absolutey};
use crate::cpu::ops::bitwise::{ora, and, eor};

//arithmetic shift left
fn asl(cpu: &mut CPU, value : u8) -> u8{

    cpu.set_flag(Flag::Carry, (value & 0x80) != 0);
    
    let result = value << 1;
    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);


    result
}

pub fn asl_accumulator(cpu: &mut CPU) -> u8 {
    let result = asl(cpu, cpu.a);
    cpu.a = result;
    0
}

pub fn asl_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);

    let result = asl(cpu, value);

    cpu.write(addr, result);

    0
}

pub fn asl_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read(addr);

    let result = asl(cpu, value);

    cpu.write(addr, result);

    0
}

pub fn asl_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);

    let result = asl(cpu, value);

    cpu.write(addr, result);

    0
}

pub fn asl_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    let value = cpu.read(addr);

    let result = asl(cpu, value);

    cpu.write(addr, result);

    0
}

fn lsr(cpu: &mut CPU, value: u8) -> u8{
    cpu.set_flag(Flag::Carry, value & 0b0000_0001 == 0b0000_0001);

    let result = value >> 1;

    cpu.set_flag(Flag::Negative, false);
    cpu.set_flag(Flag::Zero, result == 0);


    result
}

pub fn lsr_accumulator(cpu: &mut CPU) -> u8 {
    let value = cpu.a;
    let result = lsr(cpu, value);
    cpu.a = result;
    0
}

pub fn lsr_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);
    let result = lsr(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn lsr_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read(addr);
    let result = lsr(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn lsr_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);
    let result = lsr(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn lsr_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    let value = cpu.read(addr);
    let result = lsr(cpu, value);
    cpu.write(addr, result);
    0
}

fn rol(cpu: &mut CPU, value: u8) -> u8 {
    let old_carry = cpu.get_flag(Flag::Carry);

    cpu.set_flag(Flag::Carry, value & 0b1000_0000 != 0);

    let mut result = value << 1;
    if old_carry {
        result |= 0b0000_0001;
    }

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);

    result
}

pub fn rol_accumulator(cpu: &mut CPU) -> u8 {
    let value = cpu.a;
    let result = rol(cpu, value);
    cpu.a = result;
    0
}

pub fn rol_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);
    let result = rol(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn rol_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read(addr);
    let result = rol(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn rol_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);
    let result = rol(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn rol_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    let value = cpu.read(addr);
    let result = rol(cpu, value);
    cpu.write(addr, result);
    0
}

pub(crate) fn ror(cpu: &mut CPU, value: u8) -> u8 {
    let old_carry = cpu.get_flag(Flag::Carry);

    cpu.set_flag(Flag::Carry, value & 0b0000_0001 != 0);

    let mut result = value >> 1;
    if old_carry {
        result |= 0b1000_0000;
    }

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);

    result
}

pub fn ror_accumulator(cpu: &mut CPU) -> u8 {
    let value = cpu.a;
    let result = ror(cpu, value);
    cpu.a = result;
    0
}

pub fn ror_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);
    let result = ror(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn ror_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read(addr);
    let result = ror(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn ror_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);
    let result = ror(cpu, value);
    cpu.write(addr, result);
    0
}

pub fn ror_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    let value = cpu.read(addr);
    let result = ror(cpu, value);
    cpu.write(addr, result);
    0
}

// START OF UNOFFICIAL OPCODES

fn slo(cpu: &mut CPU, addr: u16) {
    let value = cpu.read(addr);
    let result = asl(cpu, value);
    cpu.write(addr, result);

    ora(cpu, result);
}

pub fn slo_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    slo(cpu, addr);
    0
}

pub fn slo_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    slo(cpu, addr);
    0
}

pub fn slo_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    slo(cpu, addr);
    0
}

pub fn slo_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    slo(cpu, addr);
    0
}

pub fn slo_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutey(cpu);
    slo(cpu, addr);
    0
}

pub fn slo_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    slo(cpu, addr);
    0
}

pub fn slo_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = indirecty(cpu);
    slo(cpu, addr);
    0
}

fn rla(cpu: &mut CPU, addr: u16) {
    let value = cpu.read(addr);
    let result = rol(cpu, value);
    cpu.write(addr, result);

    and(cpu, result);
}

pub fn rla_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    rla(cpu, addr);
    0
}

pub fn rla_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    rla(cpu, addr);
    0
}

pub fn rla_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    rla(cpu, addr);
    0
}

pub fn rla_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    rla(cpu, addr);
    0
}

pub fn rla_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutey(cpu);
    rla(cpu, addr);
    0
}

pub fn rla_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    rla(cpu, addr);
    0
}

pub fn rla_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = indirecty(cpu);
    rla(cpu, addr);
    0
}


fn sre(cpu: &mut CPU, addr: u16) {
    let value = cpu.read(addr);
    let result = lsr(cpu, value);
    cpu.write(addr, result);

    eor(cpu, result);
}

pub fn sre_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    sre(cpu, addr);
    0
}

pub fn sre_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    sre(cpu, addr);
    0
}

pub fn sre_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    sre(cpu, addr);
    0
}

pub fn sre_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    sre(cpu, addr);
    0
}

pub fn sre_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutey(cpu);
    sre(cpu, addr);
    0
}

pub fn sre_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    sre(cpu, addr);
    0
}

pub fn sre_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = indirecty(cpu);
    sre(cpu, addr);
    0
}