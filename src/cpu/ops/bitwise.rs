use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{immediate, zeropage, zeropagex, absolute, absolutex, absolutey, indirectx, indirecty};

// bitwise and

pub(crate) fn and(cpu: &mut CPU, value : u8){
    let result = cpu.a & value;
    cpu.a = result;

    cpu.set_flag(Flag::Zero, result == 0);

    cpu.set_flag(Flag::Negative, result & 0x80 != 0);
}

pub fn and_immediate(cpu: &mut CPU) -> u8{
    let value = immediate(cpu);
    and(cpu, value);

    0
}

pub fn and_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    let value = cpu.read_cycle(value);
    and(cpu, value);

    0
}

pub fn and_zeropagex(cpu: &mut CPU) -> u8 {
    let value = zeropagex(cpu);
    let value = cpu.read_cycle(value);
    and(cpu, value);
    0
}

pub fn and_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    let value = cpu.read_cycle(value);
    and(cpu, value);
    0
}

pub fn and_absolutex(cpu: &mut CPU) -> u8 {
    let (address, page_crossed) = absolutex(cpu);
    let operand = cpu.read_cycle(address);
    and(cpu, operand);

    if page_crossed { 1 } else { 0 }
}

pub fn and_absolutey(cpu: &mut CPU) -> u8 {
    let (address, page_crossed) = absolutey(cpu);
    let operand = cpu.read_cycle(address);
    and(cpu, operand);

    if page_crossed { 1 } else { 0 }
}

pub fn and_indirectx(cpu: &mut CPU) -> u8 {
    let value = indirectx(cpu);
    let value = cpu.read_cycle(value);
    and(cpu, value);
    0
}

pub fn and_indirecty(cpu: &mut CPU) -> u8 {
    let (address, page_crossed) = indirecty(cpu);
    let operand = cpu.read_cycle(address);
    and(cpu, operand);

    if page_crossed { 1 } else { 0 }
}

// bit test
pub fn bit(cpu: &mut CPU, value : u8) {
    let result = cpu.a & value;

    cpu.set_flag(Flag::Zero, result == 0);

    cpu.set_flag(Flag::Overflow, value & 0b01000000 != 0);

    cpu.set_flag(Flag::Negative, value & 0b10000000 != 0);
    
}

pub fn bit_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    let value = cpu.read_cycle(value);
    bit(cpu, value);

    0
}

pub fn bit_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    let value = cpu.read_cycle(value);
    bit(cpu, value);

    0
}

pub(crate) fn eor(cpu: &mut CPU, value: u8) {
    let result = cpu.a ^ value;

    cpu.a = result;

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0b1000_0000 != 0);


}

pub fn eor_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    eor(cpu, value);

    0
}

pub fn eor_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    let value = cpu.read_cycle(value);
    eor(cpu, value);

    0
}

pub fn eor_zeropagex(cpu: &mut CPU) -> u8 {
    let value = zeropagex(cpu);
    let value = cpu.read_cycle(value);
    eor(cpu, value);

    0
}

pub fn eor_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    let value = cpu.read_cycle(value);
    eor(cpu, value);

    0
}

pub fn eor_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutex(cpu);
    let value = cpu.read_cycle(addr);
    eor(cpu, value);

    if page_crossed {1} else {0}
}

pub fn eor_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutey(cpu);
    let value = cpu.read_cycle(addr);
    eor(cpu, value);

    if page_crossed {1} else {0}
}

pub fn eor_indirectx(cpu: &mut CPU) -> u8 {
    let value = indirectx(cpu);
    let value = cpu.read_cycle(value);
    eor(cpu, value);

    0
}

pub fn eor_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = indirecty(cpu);
    let value = cpu.read_cycle(addr);
    eor(cpu, value);

    if page_crossed {1} else {0}
}

pub(crate) fn ora(cpu: &mut CPU, value : u8){
    let result = cpu.a | value;
    cpu.a = result;

    cpu.set_flag(Flag::Zero, result == 0);

    cpu.set_flag(Flag::Negative, result & 0x80 != 0);
}

pub fn ora_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    ora(cpu, value);

    0
}

pub fn ora_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read_cycle(addr);
    ora(cpu, value);

    0
}

pub fn ora_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read_cycle(addr);
    ora(cpu, value);

    0
}

pub fn ora_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read_cycle(addr);
    ora(cpu, value);

    0
}

pub fn ora_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutex(cpu);
    let value = cpu.read_cycle(addr);
    ora(cpu, value);

    if page_crossed {1} else {0}
}

pub fn ora_absolutey(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutey(cpu);
    let value = cpu.read_cycle(addr);
    ora(cpu, value);

    if page_crossed {1} else {0}
}

pub fn ora_indirectx(cpu: &mut CPU) -> u8 {
    let addr = indirectx(cpu);
    let value = cpu.read_cycle(addr);
    ora(cpu, value);

    0
}

pub fn ora_indirecty(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = indirecty(cpu);
    let value = cpu.read_cycle(addr);
    ora(cpu, value);

    if page_crossed {1} else {0}
}

// START OF UNOFFICIAL OPCODES

fn anc(cpu: &mut CPU, value: u8) -> u8 {
    let result = cpu.a & value;
    cpu.a = result;

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0x80 != 0);
    cpu.set_flag(Flag::Carry, result & 0x80 != 0);

    0
}

pub fn anc_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    anc(cpu, value)
}

pub fn anc_immediate_dup(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    anc(cpu, value)
}

pub fn alr_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    let anded = cpu.a & value;

    cpu.set_flag(Flag::Carry, anded & 0x01 != 0);

    let result = anded >> 1;
    cpu.a = result;

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, false);

    0
}

pub fn xaa_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    let result = (cpu.a | 0xEE) & cpu.x & value;

    cpu.a = result;

    cpu.set_flag(Flag::Zero, result == 0);
    cpu.set_flag(Flag::Negative, result & 0x80 != 0);

    0
}
