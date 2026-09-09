use oolio151_nes::cpu::{Bus, NesBus};
use oolio151_nes::cpu::mapper::Nrom;
use oolio151_nes::cartridge::Mirroring;

fn make_test_bus() -> NesBus {
    // Minimal 16KB NROM cartridge, doesn't matter what's in it for RAM/PPU tests.
    let prg_rom = vec![0u8; 0x4000];
    let chr_rom = vec![0u8; 0x2000];
    NesBus::new(
        Box::new(Nrom::new(prg_rom, chr_rom.clone())),
        Mirroring::Horizontal,
        chr_rom,
    )
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

    // OAMADDR/OAMDATA let us verify both writes and reads through aliases.
    for base in (0x2000..=0x3FF8).step_by(8) {
        let value = ((base - 0x2000) / 8) as u8;
        bus.write(0x2003, 0x20);
        bus.write(0x2004, !value);

        bus.write(base + 3, 0x20);
        bus.write(base + 4, value);
        bus.write(0x2003, 0x20);
        assert_eq!(bus.read(0x2004), value, "write alias at {base:#06x}");

        bus.write(0x2004, !value);
        bus.write(0x2003, 0x20);
        assert_eq!(bus.read(base + 4), !value, "read alias at {base:#06x}");
    }
}
