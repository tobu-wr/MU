extern crate minifb;

mod emulator;
mod cpu_memory;
mod cpu;
mod ppu_memory;
mod ppu;

use emulator::*;

pub fn run() {
    let filename = std::env::args().nth(1).unwrap();
	let mut emulator = Emulator::new();
	emulator.load_rom(&filename);
	emulator.run();
}
