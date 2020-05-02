extern crate minifb;

#[macro_use]
extern crate log;

mod emulator;
mod cpu;
mod ppu;
mod joypad;

use emulator::*;

fn main() {
	simple_logger::init().unwrap();

	let filename = std::env::args().nth(1).unwrap();
	let mut emulator = Emulator::new();
	emulator.load_file(&filename);
	emulator.run();
}
