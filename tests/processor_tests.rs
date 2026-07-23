// thanks claude for ts

use oolio151_nes::cpu::CPU;
use oolio151_nes::cpu::FlatBus;

use serde::Deserialize;
use std::fs;
use std::panic;
use std::path::Path;

const TEST_DIR: &str = "tests/nes6502/v1";

#[derive(Deserialize, Debug)]
struct TestCase {
    name: String,
    initial: CpuState,
    #[serde(rename = "final")]
    expected: CpuState,
    // cycles: not checked yet — see note at bottom of file.
}

#[derive(Deserialize, Debug)]
struct CpuState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

fn build_cpu(state: &CpuState) -> CPU {
    let mut cpu = CPU::new(Box::new(FlatBus::new()));
    cpu.pc = state.pc;
    cpu.s = state.s;
    cpu.a = state.a;
    cpu.x = state.x;
    cpu.y = state.y;
    cpu.p = state.p;

    for &(addr, value) in &state.ram {
        cpu.write(addr, value);
    }

    cpu
}

/// Runs one test case. Returns Ok(()) on pass, Err(message) on failure.
fn run_case(case: &TestCase) -> Result<(), String> {
    let mut cpu = build_cpu(&case.initial);

    // Run exactly one instruction.
    cpu.tick();

    let mut mismatches = Vec::new();

    if cpu.pc != case.expected.pc {
        mismatches.push(format!("pc: got {:#06x}, want {:#06x}", cpu.pc, case.expected.pc));
    }
    if cpu.s != case.expected.s {
        mismatches.push(format!("s: got {:#04x}, want {:#04x}", cpu.s, case.expected.s));
    }
    if cpu.a != case.expected.a {
        mismatches.push(format!("a: got {:#04x}, want {:#04x}", cpu.a, case.expected.a));
    }
    if cpu.x != case.expected.x {
        mismatches.push(format!("x: got {:#04x}, want {:#04x}", cpu.x, case.expected.x));
    }
    if cpu.y != case.expected.y {
        mismatches.push(format!("y: got {:#04x}, want {:#04x}", cpu.y, case.expected.y));
    }
    if cpu.p != case.expected.p {
        mismatches.push(format!(
            "p: got {:#010b}, want {:#010b}",
            cpu.p, case.expected.p
        ));
    }

    for &(addr, expected_value) in &case.expected.ram {
        let actual_value = cpu.read(addr);
        if actual_value != expected_value {
            mismatches.push(format!(
                "ram[{:#06x}]: got {:#04x}, want {:#04x}",
                addr, actual_value, expected_value
            ));
        }
    }

    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!("case \"{}\" failed:\n    {}", case.name, mismatches.join("\n    ")))
    }
}

/// Runs every case in one opcode's JSON file. Returns (passed, failed, first_failures).
fn run_file(path: &Path) -> (usize, usize, Vec<String>) {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));

    let cases: Vec<TestCase> = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e));

    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for case in &cases {
        // Catch panics so one case (e.g. an unimplemented opcode hitting
        // decode()'s panic!, or an arithmetic overflow bug) doesn't kill
        // the whole run — we want a full report, not a single stack trace.
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| run_case(case)));

        match result {
            Ok(Ok(())) => passed += 1,
            Ok(Err(msg)) => {
                failed += 1;
                if failures.len() < 5 {
                    failures.push(msg);
                }
            }
            Err(panic_payload) => {
                failed += 1;
                let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                if failures.len() < 5 {
                    failures.push(format!("case \"{}\" panicked: {}", case.name, msg));
                }
            }
        }
    }

    (passed, failed, failures)
}

#[test]
fn run_all_processor_tests() {
    // Silence panic output during the run — we're catching and reporting
    // panics ourselves, no need for the default handler to also print
    // every single one.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let dir = Path::new(TEST_DIR);
    assert!(
        dir.is_dir(),
        "test directory not found: {} (see setup instructions at top of this file)",
        TEST_DIR
    );

    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("failed to read test directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
        .collect();
    entries.sort();

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut files_with_failures = Vec::new();

    for path in &entries {
        let (passed, failed, failures) = run_file(path);
        total_passed += passed;
        total_failed += failed;

        let filename = path.file_name().unwrap().to_string_lossy();

        if failed == 0 {
            println!("{:>10}  PASS  ({} cases)", filename, passed);
        } else {
            println!("{:>10}  FAIL  {} / {} cases failed", filename, failed, passed + failed);
            files_with_failures.push((filename.to_string(), failures));
        }
    }

    panic::set_hook(default_hook);

    println!("\n==================== SUMMARY ====================");
    println!("total: {} passed, {} failed", total_passed, total_failed);

    if !files_with_failures.is_empty() {
        println!("\n---------------- sample failures ----------------");
        for (filename, failures) in &files_with_failures {
            println!("\n{}:", filename);
            for f in failures {
                println!("  {}", f);
            }
        }
    }

    assert_eq!(total_failed, 0, "{} test case(s) failed — see output above", total_failed);
}

// NOTE ON CYCLE CHECKING:
// This harness only checks final register/flag/RAM state, not the
// cycles[] array (which lists every individual bus read/write with its
// address, value, and direction — the ground truth for exact cycle
// timing). Since your CPU currently executes a whole instruction in one
// tick() call rather than cycle-by-cycle, there's nothing to compare the
// cycle log against yet. Once/if you move to cycle-stepped execution,
// this same file can be extended to also assert on `case.cycles`.