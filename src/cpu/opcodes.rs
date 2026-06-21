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
        
        _ => panic!("unknown opcode: {:02X}", opcode)
    }
}

// addressing modes
fn immediate(cpu: &mut CPU) -> u8 {
    let value = cpu.read(cpu.pc);
    cpu.pc += 1;

    value
}

fn zeropage(cpu: &mut CPU) -> u8 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    cpu.read(byte as u16)
}

fn zeropagex(cpu: &mut CPU) -> u8 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let addr: u8 = byte.wrapping_add(cpu.x);

    cpu.read(addr as u16)
}

fn absolute(cpu: &mut CPU) -> u8{
    let low = cpu.read(cpu.pc);
    cpu.pc +=1;
    let high = cpu.read(cpu.pc);
    cpu.pc += 1;
    let addr: u16 = ((high as u16) << 8) | (low as u16);

    cpu.read(addr)
}

fn absolutex(cpu: &mut CPU) -> (u8, bool) {
    let lo = cpu.read(cpu.pc) as u16;
    let hi = cpu.read(cpu.pc + 1) as u16;
    cpu.pc += 2;
    let base: u16 = (hi << 8) | lo;

    let addr: u16 = base.wrapping_add(cpu.x as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    let value = cpu.read(addr);
    (value, page_crossed)
}

fn absolutey(cpu: &mut CPU) -> (u8, bool) {
    let lo = cpu.read(cpu.pc) as u16;
    let hi = cpu.read(cpu.pc + 1) as u16;
    cpu.pc += 2;
    let base: u16 = (hi << 8) | lo;

    let addr: u16 = base.wrapping_add(cpu.y as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    let value = cpu.read(addr);
    (value, page_crossed)
}

fn indirectx(cpu: &mut CPU) -> u8 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let zp_addr = byte.wrapping_add(cpu.x);

    let lo = cpu.read(zp_addr as u16);
    let hi = cpu.read(zp_addr.wrapping_add(1) as u16);

    let addr: u16 = ((hi as u16) << 8) | (lo as u16);

    cpu.read(addr)
}

fn indirecty(cpu: &mut CPU) -> (u8, bool) {
    let byte = cpu.read(cpu.pc);
    cpu.pc += 1;

    let lo = cpu.read(byte as u16);
    let hi = cpu.read(byte.wrapping_add(1) as u16);

    let base: u16 = ((hi as u16) << 8) | (lo as u16);

    let addr: u16 = base.wrapping_add(cpu.y as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    let value = cpu.read(addr);
    (value, page_crossed)
}

// add with carry
fn adc(cpu: &mut CPU, value : u8) {
    let carry_in = cpu.p & 0x01;

    let sum: u16 = cpu.a as u16 + value as u16 + carry_in as u16;
    let result: u8 = sum as u8;

    if sum > 0xFF {
        cpu.p |= 0x01;
    } else {
        cpu.p &= !0x01;
    }

    if result == 0 {
        cpu.p |= 0x02;
    } else {
        cpu.p &= !0x02;
    }

    let overflow = (result ^ cpu.a) & (result ^ value) & 0x80;
    if overflow != 0 {
        cpu.p |= 0x40;
    } else {
        cpu.p &= !0x40;
    }

    if result & 0x80 != 0 {
        cpu.p |= 0x80;
    } else {
        cpu.p &= !0x80;
    }

    cpu.a = result;
}

fn adc_immediate(cpu: &mut CPU) -> u8 {
    let value = immediate(cpu);
    adc(cpu, value);

    0
}

fn adc_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    adc(cpu, value);

    0
}

fn adc_zeropagex(cpu: &mut CPU) -> u8 {
    let value = zeropagex(cpu);
    adc(cpu, value);
    0
}

fn adc_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    adc(cpu, value);
    0
}

fn adc_absolutex(cpu: &mut CPU) -> u8 {
    let value = absolutex(cpu);
    adc(cpu, value.0);

     if value.1 {1} else {0}
}

fn adc_absolutey(cpu: &mut CPU) -> u8 {
    let value = absolutey(cpu);
    adc(cpu, value.0);

     if value.1 {1} else {0}
}

fn adc_indirectx(cpu: &mut CPU) -> u8 {
    let value = indirectx(cpu);
    adc(cpu, value);
    0
}

fn adc_indirecty(cpu: &mut CPU) -> u8 {
    let value = indirecty(cpu);
    adc(cpu, value.0);

     if value.1 {1} else {0}
}

// bitwise and

fn and(cpu: &mut CPU, value : u8){
    let result = cpu.a & value;
    cpu.a = result;

    if result == 0 {cpu.p |= 0x02} else {cpu.p &= !0x02}

    if result & 0x80 != 0 {cpu.p |= 0x80} else {cpu.p &= !0x08} 
}

fn and_immediate(cpu: &mut CPU) -> u8{
    let value = immediate(cpu);
    and(cpu, value);

    0
}

fn and_zeropage(cpu: &mut CPU) -> u8 {
    let value = zeropage(cpu);
    and(cpu, value);

    0
}

fn and_zeropagex(cpu: &mut CPU) -> u8 {
    let value = zeropagex(cpu);
    and(cpu, value);
    0
}

fn and_absolute(cpu: &mut CPU) -> u8 {
    let value = absolute(cpu);
    and(cpu, value);
    0
}

fn and_absolutex(cpu: &mut CPU) -> u8 {
    let value = absolutex(cpu);
    and(cpu, value.0);

     if value.1 {1} else {0}
}

fn and_absolutey(cpu: &mut CPU) -> u8 {
    let value = absolutey(cpu);
    and(cpu, value.0);

     if value.1 {1} else {0}
}

fn and_indirectx(cpu: &mut CPU) -> u8 {
    let value = indirectx(cpu);
    and(cpu, value);
    0
}

fn and_indirecty(cpu: &mut CPU) -> u8 {
    let value = indirecty(cpu);
    and(cpu, value.0);

     if value.1 {1} else {0}
}

//arithmetic shift left
fn asl(cpu: &mut CPU, value : u8) {

    if (value & 0x80) != 0 {cpu.p |= 0x01} else {cpu.p &= !0x01};
    

}