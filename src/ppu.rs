use minifb::Window;

use ppu_memory::*;
use cpu::*;

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
	ppumask: u8,
	ppustatus: u8,
	ppuscroll: u16,
	ppuaddr: u16,
	ppudata: u8,
	flipflop: bool,
	cpu: *mut Cpu,
	cycle_counter: u8,
	scanline_counter: u16,
	frame_buffer: [u32; FRAME_BUFFER_SIZE]
}

impl Ppu {
	pub fn new() -> Self {
		Self {
			ppuctrl: 0,
			ppumask: 0,
			ppustatus: 0,
			ppuscroll: 0,
			ppuaddr: 0,
			ppudata: 0,
			flipflop: false,
			cpu: std::ptr::null_mut(),
			cycle_counter: 0,
			scanline_counter: 0,
			frame_buffer: [0; FRAME_BUFFER_SIZE]
		}
	}

	pub fn connect(&mut self, cpu: *mut Cpu) {
		self.cpu = cpu;
	}

	fn request_nmi(&self) {
		unsafe {
			(*self.cpu).request_interrupt(Interrupt::Nmi);
		}
	}

	pub fn read(&mut self, register: Register) -> u8 {
		match register {
			Register::Ppustatus => {
				let value = self.ppustatus;
				self.ppustatus &= 0x7f;
				self.flipflop = false;
				value
			},
			_ => {
				println!("[ERROR] Unimplemented PPU register read");
				std::process::exit(1);
			}
		}
	}

	#[cfg(feature = "log")]
	pub fn read_debug(&self, register: Register) -> u8 {
		match register {
			Register::Ppuctrl => self.ppuctrl,
			Register::Ppumask => self.ppumask,
			Register::Ppustatus => self.ppustatus,
			Register::Ppuscroll => self.read16_debug(self.ppuscroll),
			Register::Ppuaddr => self.read16_debug(self.ppuaddr),
			Register::Ppudata => self.ppudata
		}
	}

	#[cfg(feature = "log")]
	fn read16_debug(&self, register: u16) -> u8 {
		(if self.flipflop {
			register
		} else {
			register >> 8
		}) as _
	}

	pub fn write(&mut self, register: Register, value: u8) {
		match register {
			Register::Ppuctrl => {
				self.ppuctrl = value;
				if (self.ppuctrl & self.ppustatus & 0x80) != 0 {
					self.request_nmi();
				}
			},
			Register::Ppumask => self.ppumask = value,
			Register::Ppustatus => {
				println!("[ERROR] Write to PPUSTATUS");
				std::process::exit(1);
			},
			Register::Ppuscroll => self.ppuscroll = self.write16(self.ppuscroll, value),
			Register::Ppuaddr => self.ppuaddr = self.write16(self.ppuaddr, value),
			Register::Ppudata => self.ppudata = value,
		}
	}

	fn write16(&mut self, register: u16, value: u8) -> u16 {
		self.flipflop = !self.flipflop;
		if self.flipflop {
			(register & 0x00ff) | ((value as u16) << 8)
		} else {
			(register & 0xff00) | (value as u16)
		}
	}

	pub fn do_cycle(&mut self, _memory: &PpuMemory, window: &mut Window) {
		self.cycle_counter += 1;
		if self.cycle_counter == 113 {
			self.cycle_counter = 0;
			self.scanline_counter += 1;
			if self.scanline_counter == 241 {
				self.ppustatus |= 0x80;
				if self.ppuctrl & 0x80 != 0 {
					self.request_nmi();
				}
				window.update_with_buffer(&self.frame_buffer, FRAME_WIDTH, FRAME_HEIGHT).unwrap();
			} else if self.scanline_counter == 262 {
				self.scanline_counter = 0;
				self.ppustatus &= 0x7f;
			}
		}
	}
}
