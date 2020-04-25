extern crate minifb;

mod emulator;
mod cpu_memory;
mod cpu;
mod ppu_memory;
mod ppu;
mod joypad;

use emulator::*;

fn main() {
	let filename = std::env::args().nth(1).unwrap();
	let mut emulator = Emulator::new();
	emulator.init();
	emulator.load_file(&filename);
	emulator.run();
}
