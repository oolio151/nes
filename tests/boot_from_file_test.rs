use oolio151_nes::cpu::{CPU, NesBus};
use oolio151_nes::cartridge::load_rom_from_file;



#[test]
#[ignore] // requires a real .nes file on disk — run manually with `cargo test -- --ignored`
fn boot_from_file_test() {
    let mapper = load_rom_from_file("tests/roms/SMB.nes").unwrap();
    let bus = NesBus::new(mapper);
    let mut cpu = CPU::new(Box::new(bus));
    cpu.reset();

    for _ in 0..20 {
        cpu.tick();
        println!("pc: {:#06x} a: {:#04x} x: {:#04x} y: {:#04x}", cpu.pc, cpu.a, cpu.x, cpu.y);
    }
}