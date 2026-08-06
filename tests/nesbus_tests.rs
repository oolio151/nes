use oolio151_nes::cpu::{Bus, NesBus};
use oolio151_nes::cpu::mapper::Nrom;

fn make_test_bus() -> NesBus {
    // Minimal 16KB NROM cartridge, doesn't matter what's in it for RAM/PPU tests.
    let prg_rom = vec![0u8; 0x4000];
    let chr_rom = vec![0u8; 0x2000];
    NesBus::new(Box::new(Nrom::new(prg_rom, chr_rom)))
}

#[test]
fn cpu_ram_mirrors_across_all_four_ranges() {
    let mut bus = make_test_bus();

    bus.write(0x0000, 0xAB);
    assert_eq!(bus.read(0x0800), 0xAB);
    assert_eq!(bus.read(0x1000), 0xAB);
    assert_eq!(bus.read(0x1800), 0xAB);

    // Writing through a mirror should update the same underlying byte.
    bus.write(0x1800, 0xCD);
    assert_eq!(bus.read(0x0000), 0xCD);
    assert_eq!(bus.read(0x0800), 0xCD);
    assert_eq!(bus.read(0x1000), 0xCD);
}

#[test]
fn cpu_ram_high_offset_mirrors_correctly() {
    let mut bus = make_test_bus();

    bus.write(0x0123, 0x42);
    assert_eq!(bus.read(0x0923), 0x42); // +0x0800
    assert_eq!(bus.read(0x1123), 0x42); // +0x1000
    assert_eq!(bus.read(0x1923), 0x42); // +0x1800
}

#[test]
fn ppu_registers_mirror_every_eight_bytes() {
    let mut bus = make_test_bus();

    // PPU is a stub right now (read_register/write_register both no-op /
    // return 0), so this test only confirms the ADDRESS MASKING routes
    // correctly, not real PPU register behavior — that's Phase 3.
    // Once PPU has real per-register state, extend this to verify actual
    // register values propagate correctly across the mirror too.
    for base in (0x2000..0x3FFF).step_by(8) {
        assert_eq!(bus.read(base), bus.read(0x2000));
    }
}