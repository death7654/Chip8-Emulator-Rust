// Headless benchmark for the STANDARD (interpreted) CHIP-8 core.
// No SDL, no window, no draw calls — pure instruction dispatch throughput.
//
// Place this file at: <norm_project_root>/src/bin/bench.rs
// Also required: change `fn load` to `pub fn load` in src/main.rs (it's
// private right now, so this separate binary can't call it otherwise).
//
// Run with:
//   cargo run --release --bin bench -- <rom_path> <seconds>
// e.g.
//   cargo run --release --bin bench -- test_roms/test_opcode.ch8 5

use std::env;
use std::fs::File;
use std::hint::black_box;
use std::io::Read;
use std::time::{Duration, Instant};

// Pull in the real interpreter code (Emulator::new/load/cycle) as a module
// instead of reimplementing it, so we're benchmarking the exact same
// decode() match statement that ships in the windowed binary.
#[path = "../main.rs"]
mod interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("test_roms/test_opcode.ch8");
    let seconds: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);

    let mut rom = File::open(rom_path).expect("Unable to open ROM");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();

    let mut chip8 = interpreter::Emulator::new();
    chip8.load(&buffer);

    const BATCH: u64 = 10_000;
    let mut instructions: u64 = 0;

    let start = Instant::now();
    let deadline = start + Duration::from_secs(seconds);

    while Instant::now() < deadline {
        for _ in 0..BATCH {
            chip8.cycle();
        }
        instructions += BATCH;

        // Force the optimizer to treat the state as observed, so it can't
        // prove the whole loop is dead and delete it.
        black_box(chip8.get_display());
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "Standard interpreter: {} instructions in {:.3}s = {:.0} instructions/sec",
        instructions,
        elapsed,
        instructions as f64 / elapsed
    );
}