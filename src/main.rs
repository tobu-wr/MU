extern crate minifb;

#[macro_use]
extern crate log;

mod emulator;
mod cpu;
mod ppu;
mod joypad;
mod mappers;

use emulator::*;

fn main() {
	env_logger::Builder::new().filter_level(log::LevelFilter::max()).init();

	let filename = std::env::args().nth(1).unwrap();
	let mut emulator = Emulator::new();
	emulator.load_file(&filename);
	emulator.run();
}
