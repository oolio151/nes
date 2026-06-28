use crate::cpu;

use super::CPU;

// https://www.nesdev.org/wiki/Instruction_reference
pub fn decode(opcode: u8) -> (fn(&mut CPU) -> u8, u8) {
    match opcode {

        // ADC - add with carry | A = A + memory + C
        0x69 => (adc_immediate, 2),
        0x65 => (adc_zeropage, 3),
        0x75 => (adc_zeropagex, 4),
        0x6D => (adc_absolute, 4),
        0x7D => (adc_absolutex, 4),
        0x79 => (adc_absolutey, 4),
        0x61 => (adc_indirectx, 6),
        0x71 => (adc_indirecty, 5),

        // AND - bitwise and | A = A & memory
        0x29 => (and_immediate,2),
        0x25 => (and_zeropage, 3),
        0x35 => (and_zeropagex, 4),
        0x2D => (and_absolute, 4),
        0x3D => (and_absolutex, 4),
        0x39 => (and_absolutey, 4),
        0x21 => (and_indirectx, 6),
        0x31 => (and_indirecty, 5),

        // ASL - arithmetic shift left | value = value << 1
        0x0A => (asl_accumulator, 2),
        0x06 => (asl_zeropage, 5),
        0x16 => (asl_absolute, 6),
        0x0E => (asl_absolutex, 7),

        // BCC - branch if carry clear
        0x90 => (bcc_relative, 2),
        
        _ => panic!("unknown opcode: {:02X}", opcode)
    }
}

// addressing modes
fn immediate(cpu: &mut CPU) -> u8 {
    let value = cpu.read(cpu.pc);
    cpu.pc += 1;

    value
}

fn zeropage(cpu: &mut CPU) -> u16 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    byte as u16
}

fn zeropagex(cpu: &mut CPU) -> u16 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let addr: u8 = byte.wrapping_add(cpu.x);

    addr as u16
}

fn absolute(cpu: &mut CPU) -> u16{
    let low = cpu.read(cpu.pc);
    cpu.pc +=1;
    let high = cpu.read(cpu.pc);
    cpu.pc += 1;
    let addr: u16 = ((high as u16) << 8) | (low as u16);

    addr
}

fn absolutex(cpu: &mut CPU) -> (u16, bool) {
    let lo = cpu.read(cpu.pc) as u16;
    let hi = cpu.read(cpu.pc + 1) as u16;
    cpu.pc += 2;
    let base: u16 = (hi << 8) | lo;

    let addr: u16 = base.wrapping_add(cpu.x as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    (addr, page_crossed)
}

fn absolutey(cpu: &mut CPU) -> (u16, bool) {
    let lo = cpu.read(cpu.pc) as u16;
    let hi = cpu.read(cpu.pc + 1) as u16;
    cpu.pc += 2;
    let base: u16 = (hi << 8) | lo;

    let addr: u16 = base.wrapping_add(cpu.y as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    (addr, page_crossed)
}

fn indirectx(cpu: &mut CPU) -> u16 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let zp_addr = byte.wrapping_add(cpu.x);

    let lo = cpu.read(zp_addr as u16);
    let hi = cpu.read(zp_addr.wrapping_add(1) as u16);

    let addr: u16 = ((hi as u16) << 8) | (lo as u16);

    addr
}

fn indirecty(cpu: &mut CPU) -> (u16, bool) {
    let byte = cpu.read(cpu.pc);
    cpu.pc += 1;

    let lo = cpu.read(byte as u16);
    let hi = cpu.read(byte.wrapping_add(1) as u16);

    let base: u16 = ((hi as u16) << 8) | (lo as u16);

    let addr: u16 = base.wrapping_add(cpu.y as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    (addr, page_crossed)
}

fn relative(cpu: &mut CPU) -> (u16, bool) {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let signed = byte as i8;

    let base = cpu.pc;
    let target = (base as i16).wrapping_add(signed as i16) as u16;

    let page_crossed = (target & 0xFF00) != (base & 0xFF00);

    (target, page_crossed)
}

// add with carry
fn adc(cpu: &mut CPU, value : u8) {
    let carry_in = cpu.p & 0x01;

    let sum: u16 = cpu.a as u16 + value as u16 + carry_in as u16;
    let result: u8 = sum as u8;

    cpu.set_flag(cpu::Flag::Carry, sum > 0xFF);

    cpu.set_flag(cpu::Flag::Zero, result == 0);

    let overflow = (result ^ cpu.a) & (result ^ value) & 0x80;
    cpu.set_flag(cpu::Flag::Overflow, overflow != 0);

   cpu.set_flag(cpu::Flag::Negative, result & 0x80 != 0);

    cpu.a = result;
}

fn adc_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    adc(cpu, value);

    0
}

fn adc_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    adc(cpu, cpu.read(value));

    0
}

fn adc_zeropagex(cpu: &mut CPU) -> u8 {
    let value = zeropagex(cpu);
    adc(cpu, cpu.read(value));
    0
}

fn adc_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    adc(cpu, cpu.read(value));
    0
}

fn adc_absolutex(cpu: &mut CPU) -> u8 {
    let value = absolutex(cpu);
    adc(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

fn adc_absolutey(cpu: &mut CPU) -> u8 {
    let value = absolutey(cpu);
    adc(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

fn adc_indirectx(cpu: &mut CPU) -> u8 {
    let value = indirectx(cpu);
    adc(cpu, cpu.read(value));
    0
}

fn adc_indirecty(cpu: &mut CPU) -> u8 {
    let value = indirecty(cpu);
    adc(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

// bitwise and

fn and(cpu: &mut CPU, value : u8){
    let result = cpu.a & value;
    cpu.a = result;

    cpu.set_flag(cpu::Flag::Zero, result == 0);

    cpu.set_flag(cpu::Flag::Negative, result & 0x80 != 0);
}

fn and_immediate(cpu: &mut CPU) -> u8{
    let value = immediate(cpu);
    and(cpu, value);

    0
}

fn and_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    and(cpu, cpu.read(value));

    0
}

fn and_zeropagex(cpu: &mut CPU) -> u8 {
    let value = zeropagex(cpu);
    and(cpu, cpu.read(value));
    0
}

fn and_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    and(cpu, cpu.read(value));
    0
}

fn and_absolutex(cpu: &mut CPU) -> u8 {
    let value = absolutex(cpu);
    and(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

fn and_absolutey(cpu: &mut CPU) -> u8 {
    let value = absolutey(cpu);
    and(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

fn and_indirectx(cpu: &mut CPU) -> u8 {
    let value = indirectx(cpu);
    and(cpu, cpu.read(value));
    0
}

fn and_indirecty(cpu: &mut CPU) -> u8 {
    let value = indirecty(cpu);
    and(cpu, cpu.read(value.0));

     if value.1 {1} else {0}
}

//arithmetic shift left
fn asl(cpu: &mut CPU, value : u8) -> u8{

    cpu.set_flag(cpu::Flag::Carry, (value & 0x80) != 0);
    
    let result = value << 1;
    cpu.set_flag(cpu::Flag::Zero, result == 0);
    cpu.set_flag(cpu::Flag::Negative, result & 0b1000_0000 != 0);


    result
}

fn asl_accumulator(cpu: &mut CPU) -> u8 {
    let result = asl(cpu, cpu.a);
    cpu.a = result;
    0
}

fn asl_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let value = cpu.read(addr);

    let result = asl(cpu, value);

    cpu.write(addr, result);

    0
}

fn asl_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let value = cpu.read(addr);

    let result = asl(cpu, value);

    cpu.write(addr, result);

    0
}

fn asl_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let value = cpu.read(addr);

    let result = asl(cpu, value);

    cpu.write(addr, result);

    0
}

fn asl_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, _page_crossed) = absolutex(cpu);
    let value = cpu.read(addr);

    let result = asl(cpu, value);

    cpu.write(addr, result);

    0
}

fn bcc(cpu: &mut CPU) -> u8 {
    let value = relative(cpu);

    if cpu.get_flag(cpu::Flag::Carry) {
        0
    } else {
        cpu.pc = value.0;

        if value.1 {2} else {1}
    }
}