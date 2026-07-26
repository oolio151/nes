use crate::cpu::CPU;
use crate::cpu::opcodes::{immediate, zeropage, zeropagex, absolute, absolutex};

pub fn nop(_cpu: &mut CPU) -> u8 {
    // hes lonely :(, so i wrote some stuff below
    /* 
        pokemon.nonVolatileStatus = status;
        switch (status)
        {
            case NonVolatileStatus.Poison:
                return $"{pokemon.nickname} was poisoned!";
            case NonVolatileStatus.Burn:
                return $"{pokemon.nickname} was burned!";
            case NonVolatileStatus.Paralyisis:
                return $"{pokemon.nickname} was paralyzed!";
            case NonVolatileStatus.Sleep:
                return $"{pokemon.nickname} fell asleep!";
            case NonVolatileStatus.Frozen:
                return $"{pokemon.nickname} was frozen solid!";
            case NonVolatileStatus.ToxicPoison:
                return $"{pokemon.nickname} was badly poisoned!";
            case NonVolatileStatus.None:
                return $"{pokemon.nickname} was healed of all status conditions!";
            default:
                return $"An error occurred!";
        }
    */
    0
}

// START OF UNOFFICIAL OPCDES

pub fn nop_implied(cpu: &mut CPU) -> u8 {
    let _ = cpu;
    0
}

pub fn nop_immediate(cpu: &mut CPU) -> u8 {
    let _ = immediate(cpu);
    0
}

pub fn nop_zeropage(cpu: &mut CPU) -> u8 {
    let addr = zeropage(cpu);
    let _ = cpu.read(addr);
    0
}

pub fn nop_zeropagex(cpu: &mut CPU) -> u8 {
    let addr = zeropagex(cpu);
    let _ = cpu.read(addr);
    0
}

pub fn nop_absolute(cpu: &mut CPU) -> u8 {
    let addr = absolute(cpu);
    let _ = cpu.read(addr);
    0
}

pub fn nop_absolutex(cpu: &mut CPU) -> u8 {
    let (addr, page_crossed) = absolutex(cpu);
    let _ = cpu.read(addr);
    if page_crossed { 1 } else { 0 }
}

pub fn jam(cpu: &mut CPU) -> u8 {
    let _ = cpu;
    panic!("CPU jammed");
}