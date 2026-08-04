use oolio151_nes::cpu::mapper::{Mapper, Nrom};

fn make_prg_rom(size: usize) -> Vec<u8> {
    // Fill with the low byte of each index, so a read-back tells us
    // exactly which byte we got, not just "zero or not zero."
    (0..size).map(|i| (i & 0xFF) as u8).collect()
}

#[test]
fn sixteen_kb_prg_rom_mirrors_across_8000_and_c000() {
    let prg_rom = make_prg_rom(0x4000); // 16KB
    let nrom = Nrom::new(prg_rom, Vec::new());

    assert_eq!(nrom.read(0x8000), nrom.read(0xC000));
    assert_eq!(nrom.read(0x8123), nrom.read(0xC123));
    assert_eq!(nrom.read(0xBFFF), nrom.read(0xFFFF));
}

#[test]
fn thirty_two_kb_prg_rom_does_not_mirror() {
    let mut prg_rom = vec![0u8; 0x8000]; // 32KB, zeroed
    prg_rom[0x0000] = 0x11; // corresponds to $8000
    prg_rom[0x4000] = 0x22; // corresponds to $C000 — must be a DIFFERENT byte

    let nrom = Nrom::new(prg_rom, Vec::new());

    assert_eq!(nrom.read(0x8000), 0x11);
    assert_eq!(nrom.read(0xC000), 0x22);
    assert_ne!(nrom.read(0x8000), nrom.read(0xC000));
}

#[test]
fn prg_ram_read_write_roundtrip() {
    let mut nrom = Nrom::new(make_prg_rom(0x4000), Vec::new());

    nrom.write(0x6000, 0xAB);
    nrom.write(0x7FFF, 0xCD);

    assert_eq!(nrom.read(0x6000), 0xAB);
    assert_eq!(nrom.read(0x7FFF), 0xCD);
    assert_eq!(nrom.read(0x6500), 0x00); // untouched, still zero-initialized
}

#[test]
fn writes_to_prg_rom_range_are_ignored() {
    let mut nrom = Nrom::new(make_prg_rom(0x4000), Vec::new());

    let before = nrom.read(0x8000);
    nrom.write(0x8000, 0xFF); // real hardware ignores this
    let after = nrom.read(0x8000);

    assert_eq!(before, after);
}