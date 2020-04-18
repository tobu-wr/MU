use minifb::Window;

use ppu_memory::*;

pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;

const FRAME_BUFFER_SIZE: usize = FRAME_WIDTH * FRAME_HEIGHT;

pub enum Register {
	Ppuctrl,
	Ppumask,
	Ppustatus,
	Ppuscroll,
	Ppuaddr,
	Ppudata
}

pub struct Ppu {
	ppuctrl: u8,
	ppustatus: u8,
	cycle_counter: u8,
	scanline_counter: u16,
	frame_buffer: [u32; FRAME_BUFFER_SIZE]
}

impl Ppu {
	pub fn new() -> Self {
		Self {
			ppuctrl: 0,
			ppustatus: 0,
			cycle_counter: 0,
			scanline_counter: 0,
			frame_buffer: [0; FRAME_BUFFER_SIZE]
		}
	}

	pub fn read(&mut self, register: Register) -> u8 {
		match register {
			Register::Ppustatus => {
				let value = self.ppustatus;
				self.ppustatus &= 0x7f;
				value
			},
			_ => {
				println!("[ERROR] Unhandled PPU register read");
				std::process::exit(1);
			}
		}
	}

	#[cfg(feature = "log")]
	pub fn read_debug(&self, register: Register) -> u8 {
		match register {
			Register::Ppuctrl => self.ppuctrl,
			Register::Ppustatus => self.ppustatus,
			_ => {
				println!("[ERROR] Unhandled PPU register read");
				std::process::exit(1);
			}
		}
	}

	pub fn write(&self, register: Register, _value: u8) {
		match register {
		/*	Register::PPUCTRL => {

			},
			Register::PPUMASK => {

			}
			Register::PPUSCROLL => {

			},
			Register::PPUADDR => {

			},
			Register::PPUDATA => {

			},*/
			_ =>{
				println!("[ERROR] Unhandled PPU register write");
				std::process::exit(1);
			}
		}
	}

	pub fn do_cycle(&mut self, _memory: &PpuMemory, window: &mut Window) {
		self.cycle_counter += 1;
		if self.cycle_counter == 113 {
			self.cycle_counter = 0;
			self.scanline_counter += 1;
			if self.scanline_counter == 241 {
				self.ppustatus |= 0x80;
				window.update_with_buffer(&self.frame_buffer, FRAME_WIDTH, FRAME_HEIGHT).unwrap();
			} else if self.scanline_counter == 262 {
				self.scanline_counter = 0;
				self.ppustatus &= 0x7f;
			}
		}
	}
}
