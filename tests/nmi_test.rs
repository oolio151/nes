// tests/nmi_smoke_test.rs
use oolio151_nes::emulator::Emulator;

#[test]
#[ignore] // requires a real .nes ROM file on disk at a path only you have — run manually
fn nmi_fires_during_boot() {
    let mut emu = Emulator::from_file("tests/roms/SMB.nes")
        .expect("failed to load ROM");

    let mut nmi_fired = false;
    let mut nmi_pc = 0u16;

    for _ in 0..200_000 {
        if emu.step() {
            nmi_fired = true;
            nmi_pc = emu.cpu.pc;
            break;
        }
    }

    assert!(nmi_fired, "NMI never fired within 200,000 CPU instructions");

    // Confirm pc actually landed on the ROM's real NMI vector, not just "changed."
    let vector_lo = emu.cpu.read(0xFFFA);
    let vector_hi = emu.cpu.read(0xFFFB);
    let expected_pc = (vector_hi as u16) << 8 | vector_lo as u16;

    assert_eq!(
        nmi_pc, expected_pc,
        "pc after NMI ({:#06x}) doesn't match the ROM's NMI vector ({:#06x})",
        nmi_pc, expected_pc
    );
}