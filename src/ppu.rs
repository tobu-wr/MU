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
	flipflop: bool,
	cycle_counter: u8,
	scanline_counter: u16,
	frame_buffer: [u32; FRAME_BUFFER_SIZE],
	memory: *mut PpuMemory,
	cpu: *mut Cpu
}

impl Ppu {
	pub fn new() -> Self {
		Self {
			ppuctrl: 0,
			ppumask: 0,
			ppustatus: 0,
			ppuscroll: 0,
			ppuaddr: 0,
			flipflop: false,
			cycle_counter: 0,
			scanline_counter: 0,
			frame_buffer: [0; FRAME_BUFFER_SIZE],
			memory: std::ptr::null_mut(),
			cpu: std::ptr::null_mut()
		}
	}

	pub fn connect(&mut self, cpu: *mut Cpu, memory: *mut PpuMemory) {
		self.cpu = cpu;
		self.memory = memory;
	}

	fn check_for_nmi(&self) {
		if (self.ppuctrl & self.ppustatus & 0x80) != 0 {
			unsafe {
				(*self.cpu).request_interrupt(Interrupt::Nmi);
			}
		}
	}

	fn read_memory(&self, address: u16) -> u8 {
		unsafe {
			(*self.memory).read(address)
		}
	}

	pub fn read_register(&mut self, register: Register) -> u8 {
		match register {
			Register::Ppustatus => {
				let value = self.ppustatus;
				self.ppustatus &= 0x7f;
				self.flipflop = false;
				value
			},
			_ => {
				println!("[ERROR] [PPU] Unhandled PPU register read");
				std::process::exit(1);
			}
		}
	}

	#[cfg(feature = "log")]
	pub fn read_register_debug(&self, register: Register) -> u8 {
		match register {
			Register::Ppuctrl => self.ppuctrl,
			Register::Ppumask => self.ppumask,
			Register::Ppustatus => self.ppustatus,
			Register::Ppuscroll => self.read16_register_debug(self.ppuscroll),
			Register::Ppuaddr => self.read16_register_debug(self.ppuaddr),
			Register::Ppudata => self.memory.read(self.ppuaddr)
		}
	}

	#[cfg(feature = "log")]
	fn read16_register_debug(&self, register: u16) -> u8 {
		(if self.flipflop {
			register
		} else {
			register >> 8
		}) as _
	}

	pub fn write_register(&mut self, register: Register, value: u8) {
		match register {
			Register::Ppuctrl => {
				self.ppuctrl = value;
				self.check_for_nmi();
			},
			Register::Ppumask => self.ppumask = value,
			Register::Ppustatus => {
				println!("[ERROR] [PPU] Write to PPUSTATUS");
				std::process::exit(1);
			},
			Register::Ppuscroll => self.ppuscroll = self.write16_register(self.ppuscroll, value),
			Register::Ppuaddr => self.ppuaddr = self.write16_register(self.ppuaddr, value),
			Register::Ppudata => {
				unsafe {
					(*self.memory).write(self.ppuaddr, value);
				}
				self.ppuaddr += if (self.ppuctrl & 0x04) == 0 {
					1
				} else {
					32
				};
			}
		}
	}

	fn write16_register(&mut self, register: u16, value: u8) -> u16 {
		self.flipflop = !self.flipflop;
		if self.flipflop {
			(register & 0x00ff) | ((value as u16) << 8)
		} else {
			(register & 0xff00) | (value as u16)
		}
	}

	pub fn do_cycle(&mut self, window: &mut Window) {
		self.cycle_counter += 1;
		if self.cycle_counter == 113 {
			self.cycle_counter = 0;
			self.scanline_counter += 1;
			if self.scanline_counter == 241 {
				self.ppustatus |= 0x80;
				self.check_for_nmi();
				if (self.ppumask & 0x08) != 0 {
					for tile_row in 0..30 {
						for tile_column in 0..32 {
							let tile_number_address = 0x2000 + tile_row * 32 + tile_column;
							let tile_number = self.read_memory(tile_number_address);
							let attribute_row = tile_row / 4;
							let attribute_column = tile_column / 4;
							let attribute = self.read_memory(0x23c0 + attribute_row * 8 + attribute_column);
							let color_set = ((attribute >> (4 * (tile_row % 2))) >> (2 * (tile_column))) & 0b11;
							let palette_number = 4 * color_set;
							for pixel_row in 0..8 {
								let low_byte = self.read_memory(0x0000 + (tile_number as u16) * 16 + pixel_row);
								let high_byte = self.read_memory(0x0000 + (tile_number as u16) * 16 + pixel_row + 8);
								for pixel_column in 0..8 {
									let low_bit = (low_byte >> (7 - pixel_column)) & 1;
									let high_bit = (high_byte >> (7 - pixel_column)) & 1;
									let color_number = (high_bit << 1) | low_bit;
									let color = self.read_memory(0x3f00 + palette_number as u16 + color_number as u16);
									self.frame_buffer[(tile_row * 256 * 8 + pixel_row * 256 + tile_column * 8 + pixel_column) as usize] = match color {
										0x0f => 0x00_00_00_00,
										0x33 => 0x00_d4_b2_ec,
										_ => {
											println!("[ERROR] [PPU] Unhandled color {:02X}", color);
											std::process::exit(1);
										}
									};
								}
							}
						}
					}
					window.update_with_buffer(&self.frame_buffer, FRAME_WIDTH, FRAME_HEIGHT).unwrap();
				} else {
					window.update();
				}
			} else if self.scanline_counter == 262 {
				self.scanline_counter = 0;
				self.ppustatus &= 0x7f;
			}
		}
	}
}
