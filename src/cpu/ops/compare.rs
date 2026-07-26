use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{immediate, zeropage, zeropagex, absolute, absolutex, absolutey, indirectx, indirecty};
use crate::cpu::ops::arithmetic::dec;


fn compare(cpu: &mut CPU, register: u8, value: u8) {
    let result = register.wrapping_sub(value);

    cpu.set_flag(Flag::Carry, register >= value);
    cpu.set_flag(Flag::Zero, register == value);
    cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);
}

pub fn cmp(cpu: &mut CPU, value: u8){
    compare(cpu, cpu.a, value);
}

pub fn cmp_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    cmp(cpu, value);

    0
}

pub fn cmp_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    cmp(cpu, cpu.read(addr));

    0
}

pub fn cmp_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    cmp(cpu, cpu.read(addr));

    0
}

pub fn cmp_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    cmp(cpu, cpu.read(addr));

    0
}

pub fn cmp_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutex(cpu);
    cmp(cpu, cpu.read(addr));

    if page_crossed {1} else {0}
}

pub fn cmp_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutey(cpu);
    cmp(cpu, cpu.read(addr));

    if page_crossed {1} else {0}
}

pub fn cmp_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    cmp(cpu, cpu.read(addr));

    0
}

pub fn cmp_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = indirecty(cpu);
    cmp(cpu, cpu.read(addr));

    if page_crossed {1} else {0}
}

pub fn cpx(cpu: &mut CPU, value: u8){
    compare(cpu, cpu.x, value);
}

pub fn cpx_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    cpx(cpu, value);

    0
}

pub fn cpx_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    cpx(cpu, cpu.read(addr));

    0
}

pub fn cpx_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    cpx(cpu, cpu.read(addr));

    0
}


pub fn cpy(cpu: &mut CPU, value: u8){
    compare(cpu, cpu.y, value);
}

pub fn cpy_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    cpy(cpu, value);

    0
}

pub fn cpy_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    cpy(cpu, cpu.read(addr));

    0
}

pub fn cpy_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    cpy(cpu, cpu.read(addr));

    0
}

// START OF UNOFFICIAL OPCODES

fn dcp(cpu: &mut CPU, addr: u16) {
    let value = cpu.read(addr);
    let result = dec(cpu, value);
    cpu.write(addr, result);

    cmp(cpu, result);
}

pub fn dcp_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    dcp(cpu, addr);
    0
}

pub fn dcp_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    dcp(cpu, addr);
    0
}

pub fn dcp_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    dcp(cpu, addr);
    0
}

pub fn dcp_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    dcp(cpu, addr);
    0
}

pub fn dcp_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutey(cpu);
    dcp(cpu, addr);
    0
}

pub fn dcp_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    dcp(cpu, addr);
    0
}

pub fn dcp_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = indirecty(cpu);
    dcp(cpu, addr);
    0
}