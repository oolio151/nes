use super::CPU;

pub fn decode(opcode: u8) -> (fn(&mut CPU), u8) {
    match opcode {
        0xA9 => (lda_immediate, 2),
        0xAD => (lda_absolute, 4),
        
        _ => panic!("unknown opcode: {:02X}", opcode)
    }
}

fn lda_immediate(cpu: &mut CPU) {

}

fn lda_absolute(cpu: &mut CPU) {

}
