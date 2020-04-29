pub mod memory;

pub use self::memory::*;

use emulator::*;
use cpu::*;

pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;
pub const OAM_SIZE: usize = 256;

const FRAME_BUFFER_SIZE: usize = FRAME_WIDTH * FRAME_HEIGHT;

pub enum Register {
	Ppuctrl,
	Ppumask,
	Ppustatus,
	Oamaddr,
	Ppuscroll,
	Ppuaddr,
	Ppudata
}

pub struct Ppu {
	ppuctrl: u8,
	ppumask: u8,
	ppustatus: u8,
	oamaddr: u8,
	ppuscroll: u16,
	ppuaddr: u16,
	flipflop: bool,
	cycle_counter: u8,
	scanline_counter: u16,
	frame_buffer: [u32; FRAME_BUFFER_SIZE],
	oam: [u8; OAM_SIZE]
}

impl Ppu {
	pub fn new() -> Self {
		Self {
			ppuctrl: 0,
			ppumask: 0,
			ppustatus: 0,
			oamaddr: 0,
			ppuscroll: 0,
			ppuaddr: 0,
			flipflop: false,
			cycle_counter: 0,
			scanline_counter: 0,
			frame_buffer: [0; FRAME_BUFFER_SIZE],
			oam: [0; OAM_SIZE]
		}
	}

	pub fn read(&mut self, memory: &Memory, register: Register) -> u8 {
		match register {
			Register::Ppustatus => {
				let value = self.ppustatus;
				self.ppustatus &= 0x7f;
				self.flipflop = false;
				value
			},
			Register::Ppudata => {
				let value = memory.read(self.ppuaddr);
				self.increment_ppuaddr();
				value
			},
			_ => {
				println!("[ERROR] [PPU] Unhandled PPU register read");
				std::process::exit(1);
			}
		}
	}

	#[cfg(feature = "log")]
	pub fn read_debug(&self, memory: &Memory, register: Register) -> u8 {
		match register {
			Register::Ppuctrl => self.ppuctrl,
			Register::Ppumask => self.ppumask,
			Register::Ppustatus => self.ppustatus,
			Register::Ppuscroll => self.read16_debug(self.ppuscroll),
			Register::Ppuaddr => self.read16_debug(self.ppuaddr),
			Register::Ppudata => memory.read(self.ppuaddr)
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

	pub fn write(&mut self, memory: &mut Memory, register: Register, value: u8) {
		match register {
			Register::Ppuctrl => {
				self.ppuctrl = value;
				//	TODO: check for NMI
			},
			Register::Ppumask => self.ppumask = value,
			Register::Ppustatus => {
				println!("[ERROR] [PPU] Write to PPUSTATUS");
				std::process::exit(1);
			},
			Register::Oamaddr => self.oamaddr = value,
			Register::Ppuscroll => self.ppuscroll = self.write16(self.ppuscroll, value),
			Register::Ppuaddr => self.ppuaddr = self.write16(self.ppuaddr, value),
			Register::Ppudata => {
				memory.write(self.ppuaddr, value);
				self.increment_ppuaddr();
			}
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

	fn increment_ppuaddr(&mut self) {
		self.ppuaddr += if (self.ppuctrl & 0x04) == 0 {
			1
		} else {
			32
		};
	}

	pub fn write_oam(&mut self, data: &[u8]) {
		for value in data {
			self.oam[self.oamaddr as usize] = *value;
			self.oamaddr = self.oamaddr.wrapping_add(1);
		}
	}

	fn set_pixel(&mut self, x: u16, y: u16, color: u8) {
		const COLORS: [u32; 0x40] = [0x00545454, 0x00001e74, 0x00081090, 0x00300088, 0x00440064, 0x005c0030, 0x00540400, 0x003c1800,
		0x00202a00, 0x00083a00, 0x00004000, 0x00003c00, 0x0000323c, 0x00000000, 0x00000000, 0x00000000, 0x00989698, 0x00084cc4,
		0x003032ec, 0x005c1ee4, 0x008814b0, 0x00a01464, 0x00982220, 0x00783c00, 0x00545a00, 0x00287200, 0x00087c00, 0x00007628,
		0x00006678, 0x00000000, 0x00000000, 0x00000000, 0x00eceeec, 0x004c9aec, 0x00787cec, 0x00b062ec, 0x00e454ec, 0x00ec58b4,
		0x00ec6a64, 0x00d48820, 0x00a0aa00, 0x0074c400, 0x004cd020, 0x0038cc6c, 0x0038b4cc, 0x003c3c3c, 0x00000000, 0x00000000,
		0x00eceeec, 0x00a8ccec, 0x00bcbcec, 0x00d4b2ec, 0x00ecaeec, 0x00ecaed4, 0x00ecb4b0, 0x00e4c490, 0x00ccd278, 0x00b4de78,
		0x00a8e290, 0x0098e2b4, 0x00a0d6e4, 0x00a0a2a0, 0x00000000, 0x00000000];
		self.frame_buffer[(y as usize) * FRAME_WIDTH + x as usize] = COLORS[color as usize];
	}

	pub fn do_cycle(emulator: &mut Emulator) {
		emulator.ppu.cycle_counter += 1;
		if emulator.ppu.cycle_counter == 113 {
			emulator.ppu.cycle_counter = 0;
			emulator.ppu.scanline_counter += 1;
			if emulator.ppu.scanline_counter == 241 {
				// start VBlank
				emulator.ppu.ppustatus |= 0x80;
				if (emulator.ppu.ppuctrl & 0x80) != 0 {
					emulator.cpu.request_interrupt(Interrupt::Nmi);
				}
				// render background
				if (emulator.ppu.ppumask & 0x08) != 0 {
					let nametable_address = 0x2000 + 0x400 * (emulator.ppu.ppuctrl & 0b11) as u16;
					let attribute_table_address = nametable_address + 0x3c0;
					let pattern_address = 0x1000 * ((emulator.ppu.ppuctrl >> 4) & 1) as u16;
					for tile_row in 0..30 {
						for tile_column in 0..32 {
							let tile_number_address = nametable_address + tile_row * 32 + tile_column;
							let tile_number = emulator.ppu_memory.read(tile_number_address);
							let attribute_row = tile_row / 4;
							let attribute_column = tile_column / 4;
							let attribute = emulator.ppu_memory.read(attribute_table_address + attribute_row * 8 + attribute_column);
							let color_set = ((attribute >> (4 * ((tile_row / 2) % 2))) >> (2 * ((tile_column / 2) % 2))) & 0b11;
							let palette_number = 4 * color_set;
							for pixel_row in 0..8 {
								let y = tile_row * 8 + pixel_row;
								let low_byte = emulator.ppu_memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row);
								let high_byte = emulator.ppu_memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row + 8);
								for pixel_column in 0..8 {
									let low_bit = (low_byte >> (7 - pixel_column)) & 1;
									let high_bit = (high_byte >> (7 - pixel_column)) & 1;
									let color_number = (high_bit << 1) | low_bit;
									let color = emulator.ppu_memory.read(0x3f00 + palette_number as u16 + color_number as u16);
									let x = tile_column * 8 + pixel_column;
									emulator.ppu.set_pixel(x, y, color);
								}
							}
						}
					}
				} 
				// render sprites
				if (emulator.ppu.ppumask & 0x10) != 0 {
					for number in 0..64 {
						let sprite_y = emulator.ppu.oam[number * 4];
						let tile_number = emulator.ppu.oam[number * 4 + 1];
						let attributes = emulator.ppu.oam[number * 4 + 2];
						let palette_number = (4 + (attributes & 0b11)) * 4;
						let horizontal_flip = (attributes & 0x40) != 0;
						let sprite_x = emulator.ppu.oam[number * 4 + 3];
						let pattern_address = 0x1000 * ((emulator.ppu.ppuctrl >> 3) & 1) as u16;
						for pixel_row in 0..8 {
							let y = pixel_row + sprite_y as u16;
							if y >= FRAME_HEIGHT as u16 {
								break;
							}
							let low_byte = emulator.ppu_memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row);
							let high_byte = emulator.ppu_memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row + 8);
							for pixel_column in 0..8 {
								let x = pixel_column + sprite_x as u16;
								if x >= FRAME_WIDTH as u16 {
									break;
								}
								let low_bit = (low_byte >> if horizontal_flip {
									pixel_column
								} else {
									7 - pixel_column
								}) & 1;
								let high_bit = (high_byte >>  if horizontal_flip {
									pixel_column
								} else {
									7 - pixel_column
								}) & 1;
								let color_number = (high_bit << 1) | low_bit;
								if color_number != 0 {
									let color = emulator.ppu_memory.read(0x3f00 + palette_number as u16 + color_number as u16);
									emulator.ppu.set_pixel(x, y, color);
								}
							}
						}
					}
				}
				// update window
				emulator.window.update_with_buffer(&emulator.ppu.frame_buffer, FRAME_WIDTH, FRAME_HEIGHT).unwrap();
			} else if emulator.ppu.scanline_counter == 262 {
				// end VBlank
				emulator.ppu.scanline_counter = 0;
				emulator.ppu.ppustatus &= 0x7f;
			}
		}
	}
}
