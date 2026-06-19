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
        0x71 => (adc_indirectx, 5),

        // AND - bitwise and | A = A & memory

        
        _ => panic!("unknown opcode: {:02X}", opcode)
    }
}

fn adc_immediate(cpu: &mut CPU) -> u8 {
    
    let value = cpu.read(cpu.pc);
      
    0
}

fn adc_zeropage(cpu: &mut CPU) -> u8 {

    0
}

fn adc_zeropagex(cpu: &mut CPU) -> u8 {

    0
}

fn adc_absolute(cpu: &mut CPU) -> u8 {
    
    0
}

fn adc_absolutex(cpu: &mut CPU) -> u8 {
    
}

fn adc_absolutey(cpu: &mut CPU) -> u8 {
    
}

fn adc_indirectx(cpu: &mut CPU) -> u8 {
    
    0
}

fn adc_indirecty(cpu: &mut CPU) -> u8 {
    
}
