use crate::cpu::CPU;
use crate::cpu::Flag;
use crate::cpu::opcodes::{immediate, zeropage, zeropagex, absolute, absolutex, absolutey, indirectx, indirecty};

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