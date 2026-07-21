#![allow(dead_code)] // all the yellow squiggles are a bitch, no more

//use crate::cpu;
use crate::cpu::ops::arithmetic::*;
use crate::cpu::ops::bitwise::*;
use crate::cpu::ops::shift::*;
use crate::cpu::ops::branch::*;
use crate::cpu::ops::jump::*;
use crate::cpu::ops::flags::*;
use crate::cpu::ops::compare::*;
use crate::cpu::ops::access::*;
use crate::cpu::ops::other::*;
use crate::cpu::ops::stack::*;

use super::CPU;

// https://www.nesdev.org/wiki/Instruction_reference
pub fn decode(opcode: u8) -> (fn(&mut CPU) -> u8, u8) {
    match opcode {

        // add with carry | A = A + memory + C
        0x69 => (adc_immediate, 2),
        0x65 => (adc_zeropage, 3),
        0x75 => (adc_zeropagex, 4),
        0x6D => (adc_absolute, 4),
        0x7D => (adc_absolutex, 4),
        0x79 => (adc_absolutey, 4),
        0x61 => (adc_indirectx, 6),
        0x71 => (adc_indirecty, 5),

        // bitwise and | A = A & memory
        0x29 => (and_immediate,2),
        0x25 => (and_zeropage, 3),
        0x35 => (and_zeropagex, 4),
        0x2D => (and_absolute, 4),
        0x3D => (and_absolutex, 4),
        0x39 => (and_absolutey, 4),
        0x21 => (and_indirectx, 6),
        0x31 => (and_indirecty, 5),

        // arithmetic shift left | value = value << 1
        0x0A => (asl_accumulator, 2),
        0x06 => (asl_zeropage, 5),
        0x16 => (asl_zeropagex, 6),
        0x0E => (asl_absolute, 6),
        0x1E => (asl_absolutex, 7),

        // branch if carry clear
        0x90 => (bcc, 2),

        // branch if carry set
        0xB0 => (bcs, 2),

        // branch if equal
        0xF0 => (beq, 2),

        // bit test
        0x24 => (bit_zeropage, 3),
        0x2C => (bit_absolute, 4),

        // branch if minus
        0x30 => (bmi, 2),

        // branch if not equal
        0xD0 => (bne, 2),

        // branch if plus
        0x10 => (bpl, 2),

        // break (software iq) <- dunno what ts means
        0x00 => (brk, 0),

        // branch if carry clear
        0x50 => (bvc, 2),

        // branch if carry set
        0x70 => (bvs, 2),

        // clear carry, decimal, interruptdisable, and overflow
        0x18 => (clc, 2),
        0xD8 => (cld, 2),
        0x58 => (cli, 2),
        0xB8 => (clv, 2),

        // compare a
        0xC9 => (cmp_immediate, 2),
        0xC5 => (cmp_zeropage, 3),
        0xD5 => (cmp_zeropagex, 4),
        0xCD => (cmp_absolute, 4),
        0xDD => (cmp_absolutex, 4),
        0xD9 => (cmp_absolutey, 4),
        0xC1 => (cmp_indirectx, 6),
        0xD1 => (cmp_indirecty, 5),

        // compare x
        0xE0 => (cpx_immediate, 2),
        0xE4 => (cpx_zeropage, 3),
        0xEC => (cpx_absolute, 4),
        
        // compare y
        0xC0 => (cpy_immediate, 2),
        0xC4 => (cpy_zeropage, 3),
        0xCC => (cpy_absolute, 4),

        // decrement memory
        0xC6 => (dec_zeropage, 5),
        0xD6 => (dec_zeropagex, 6),
        0xCE => (dec_absolute, 6),
        0xDE => (dec_absolutex, 7),

        // decrement x register
        0xCA => (dex, 2),

        // decrement y register
        0x88 => (dey, 2),

        // xor
        0x49 => (eor_immediate, 2),
        0x45 => (eor_zeropage, 3),
        0x55 => (eor_zeropagex, 4),
        0x4D => (eor_absolute, 4),
        0x5D => (eor_absolutey, 4),
        0x41 => (eor_indirectx, 6),
        0x51 => (eor_indirecty, 5),

        // increment memory
        0xE6 => (inc_zeropage, 5),
        0xF6 => (inc_zeropagex, 6),
        0xEE => (inc_absolute, 6),
        0xFE => (inc_absolutex, 7),

        // increment x register
        0xE8 => (inx, 2),

        // increment y register
        0xC8 => (iny, 2),

        // jump
        0x4C => (jmp_absolute, 3),
        0x6C => (jmp_indirect, 5),

        // jump to subroutine
        0x20 => (jsr, 6),

        // load into accumulator
        0xA9 => (lda_immediate, 2),
        0xA5 => (lda_zeropage, 3),
        0xB5 => (lda_zeropagex, 4),
        0xAD => (lda_absolute, 4),
        0xBD => (lda_absolutex, 4),
        0xB9 => (lda_absolutey, 4),
        0xA1 => (lda_indirectx, 6),
        0xB1 => (lda_indirecty, 5),

        // load into x register
        0xA2 => (ldx_immediate, 2),
        0xA6 => (ldx_zeropage, 3),
        0xB6 => (ldx_zeropagey, 4),
        0xAE => (ldx_absolute, 4),
        0xBE => (ldx_absolutey, 4),

        // load into y register
        0xA0 => (ldy_immediate, 2),
        0xA4 => (ldy_zeropage, 3),
        0xB4 => (ldy_zeropagex, 4),
        0xAC => (ldy_absolute, 4),
        0xBC => (ldy_absolutex, 4),

        // logical shift right
        0x4A => (lsr_accumulator, 2),
        0x46 => (lsr_zeropage, 5),
        0x56 => (lsr_zeropagex, 6),
        0x4E => (lsr_absolute, 6),
        0x5E => (lsr_absolutex, 7),

        // nothing
        0xEA => (nop, 2),

        // bitwise or
        0x09 => (ora_immediate, 2),
        0x05 => (ora_zeropage, 3),
        0x15 => (ora_zeropagex, 4),
        0x0D => (ora_absolute, 4),
        0x1D => (ora_absolutex, 4),
        0x19 => (ora_absolutey, 4),
        0x01 => (ora_indirectx, 6),
        0x11 => (ora_indirecty, 5),

        // push accumulator to stack
        0x48 => (pha, 3),

        // push status register to stack
        0x08 => (php, 3),

        // pop from stack into accumulator
        0x68 => (pla, 4),

        // pop from stack into status register
        0x28 => (plp, 4),

        // rotate memory value left
        0x2A => (rol_accumulator, 2),
        0x26 => (rol_zeropage, 5),
        0x36 => (rol_zeropagex, 6),
        0x2E => (rol_absolute, 6),
        0x3E => (rol_absolutex, 7),

        // rotate memory value right
        0x6A => (ror_accumulator, 2),
        0x66 => (ror_zeropage, 5),
        0x76 => (ror_zeropagex, 6),
        0x6E => (ror_absolute, 6),
        0x7E => (ror_absolutex, 7),

        _ => panic!("unknown opcode: {:02X}", opcode)
    }
}

// addressing modes
pub fn immediate(cpu: &mut CPU) -> u8 {
    let value = cpu.read(cpu.pc);
    cpu.pc += 1;

    value
}

pub fn zeropage(cpu: &mut CPU) -> u16 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    byte as u16
}

pub fn zeropagex(cpu: &mut CPU) -> u16 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let addr: u8 = byte.wrapping_add(cpu.x);

    addr as u16
}

pub fn zeropagey(cpu: &mut CPU) -> u16 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let addr: u8 = byte.wrapping_add(cpu.y);

    addr as u16
}

pub fn absolute(cpu: &mut CPU) -> u16{
    let low = cpu.read(cpu.pc);
    cpu.pc +=1;
    let high = cpu.read(cpu.pc);
    cpu.pc += 1;
    let addr: u16 = ((high as u16) << 8) | (low as u16);

    addr
}

pub fn absolutex(cpu: &mut CPU) -> (u16, bool) {
    let lo = cpu.read(cpu.pc) as u16;
    let hi = cpu.read(cpu.pc + 1) as u16;
    cpu.pc += 2;
    let base: u16 = (hi << 8) | lo;

    let addr: u16 = base.wrapping_add(cpu.x as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    (addr, page_crossed)
}

pub fn absolutey(cpu: &mut CPU) -> (u16, bool) {
    let lo = cpu.read(cpu.pc) as u16;
    let hi = cpu.read(cpu.pc + 1) as u16;
    cpu.pc += 2;
    let base: u16 = (hi << 8) | lo;

    let addr: u16 = base.wrapping_add(cpu.y as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    (addr, page_crossed)
}

pub fn indirectx(cpu: &mut CPU) -> u16 {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let zp_addr = byte.wrapping_add(cpu.x);

    let lo = cpu.read(zp_addr as u16);
    let hi = cpu.read(zp_addr.wrapping_add(1) as u16);

    let addr: u16 = ((hi as u16) << 8) | (lo as u16);

    addr
}

pub fn indirecty(cpu: &mut CPU) -> (u16, bool) {
    let byte = cpu.read(cpu.pc);
    cpu.pc += 1;

    let lo = cpu.read(byte as u16);
    let hi = cpu.read(byte.wrapping_add(1) as u16);

    let base: u16 = ((hi as u16) << 8) | (lo as u16);

    let addr: u16 = base.wrapping_add(cpu.y as u16);

    let page_crossed = (addr & 0xFF00) != (base & 0xFF00);

    (addr, page_crossed)
}

pub fn relative(cpu: &mut CPU) -> (u16, bool) {
    let byte: u8 = cpu.read(cpu.pc);
    cpu.pc += 1;

    let signed = byte as i8;

    let base = cpu.pc;
    let target = (base as i16).wrapping_add(signed as i16) as u16;

    let page_crossed = (target & 0xFF00) != (base & 0xFF00);

    (target, page_crossed)
}