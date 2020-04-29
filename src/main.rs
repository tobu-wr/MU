extern crate minifb;

mod emulator;
mod cpu;
mod ppu;
mod joypad;

use emulator::*;

fn main() {
	let filename = std::env::args().nth(1).unwrap();
	let mut emulator = Emulator::new();
	emulator.load_file(&filename);
	emulator.run();
}
