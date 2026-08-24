// tests/nmi_smoke_test.rs
use oolio151_nes::emulator::Emulator;

#[test]
#[ignore] // requires a real .nes ROM file on disk — run manually
fn nmi_fires_during_boot() {
    let mut emu = Emulator::from_file("tests/roms/SMB.nes")
        .expect("failed to load ROM");

    let mut nmi_count = 0;
    let mut last_nmi_pc = 0u16;

    // Run long enough to get several frames in, not just the first NMI.
    for _ in 0..2_000_000 {
        if emu.step() {
            nmi_count += 1;
            last_nmi_pc = emu.cpu.pc;
        }
    }

    assert!(nmi_count > 0, "NMI never fired within 2,000,000 CPU instructions");

    let vector_lo = emu.cpu.read(0xFFFA);
    let vector_hi = emu.cpu.read(0xFFFB);
    let expected_pc = (vector_hi as u16) << 8 | vector_lo as u16;

    assert_eq!(
        last_nmi_pc, expected_pc,
        "pc after NMI ({:#06x}) doesn't match the ROM's NMI vector ({:#06x})",
        last_nmi_pc, expected_pc
    );

    println!("NMI fired {} times", nmi_count);

    // Basic sanity check: the framebuffer shouldn't be a single solid color
    // by this point — SMB draws its title screen well within a couple
    // hundred frames of boot.
    let fb = emu.cpu.framebuffer();
    let first_pixel = fb[0];
    let all_same = fb.iter().all(|&p| p == first_pixel);
    assert!(!all_same, "framebuffer is a single solid color — nothing appears to be rendering");

    // Dump the framebuffer to a plain PPM file so you can actually look at it.
    // PPM is trivial to write by hand and any image viewer / VS Code extension can open it.
    let mut ppm = String::from("P3\n256 240\n255\n");
    for &(r, g, b) in fb {
        ppm.push_str(&format!("{} {} {}\n", r, g, b));
    }
    std::fs::write("target/framebuffer_dump.ppm", ppm)
        .expect("failed to write framebuffer dump");
}