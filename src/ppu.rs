use minifb::Window;

use ppu_memory::*;
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
	oam: [u8; OAM_SIZE],
	memory: *mut PpuMemory,
	cpu: *mut Cpu
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
			oam: [0; OAM_SIZE],
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
			Register::Ppudata => {
				let value = self.read_memory(self.ppuaddr);
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
	pub fn read_register_debug(&self, register: Register) -> u8 {
		match register {
			Register::Ppuctrl => self.ppuctrl,
			Register::Ppumask => self.ppumask,
			Register::Ppustatus => self.ppustatus,
			Register::Ppuscroll => self.read16_register_debug(self.ppuscroll),
			Register::Ppuaddr => self.read16_register_debug(self.ppuaddr),
			Register::Ppudata => self.read_memory(self.ppuaddr)
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

	pub fn write_oam(&mut self, data: &[u8]) {
		for value in data {
			self.oam[self.oamaddr as usize] = *value;
			self.oamaddr = self.oamaddr.wrapping_add(1);
		}
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
			Register::Oamaddr => self.oamaddr = value,
			Register::Ppuscroll => self.ppuscroll = self.write16_register(self.ppuscroll, value),
			Register::Ppuaddr => self.ppuaddr = self.write16_register(self.ppuaddr, value),
			Register::Ppudata => {
				unsafe {
					(*self.memory).write(self.ppuaddr, value);
				}
				self.increment_ppuaddr();
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

	fn increment_ppuaddr(&mut self) {
		self.ppuaddr += if (self.ppuctrl & 0x04) == 0 {
			1
		} else {
			32
		};
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
					let nametable_address = 0x2000 + 0x400 * (self.ppuctrl & 0b11) as u16;
					let attribute_table_address = nametable_address + 0x3c0;
					let pattern_address = 0x1000 * ((self.ppuctrl >> 4) & 1) as u16;
					for tile_row in 0..30 {
						for tile_column in 0..32 {
							let tile_number_address = nametable_address + tile_row * 32 + tile_column;
							let tile_number = self.read_memory(tile_number_address);
							let attribute_row = tile_row / 4;
							let attribute_column = tile_column / 4;
							let attribute = self.read_memory(attribute_table_address + attribute_row * 8 + attribute_column);
							let color_set = ((attribute >> (4 * ((tile_row / 2) % 2))) >> (2 * ((tile_column / 2) % 2))) & 0b11;
							let palette_number = 4 * color_set;
							for pixel_row in 0..8 {
								let low_byte = self.read_memory(pattern_address + (tile_number as u16) * 16 + pixel_row);
								let high_byte = self.read_memory(pattern_address + (tile_number as u16) * 16 + pixel_row + 8);
								for pixel_column in 0..8 {
									let low_bit = (low_byte >> (7 - pixel_column)) & 1;
									let high_bit = (high_byte >> (7 - pixel_column)) & 1;
									let color_number = (high_bit << 1) | low_bit;
									let color = self.read_memory(0x3f00 + palette_number as u16 + color_number as u16);
									self.frame_buffer[(tile_row * 256 * 8 + pixel_row * 256 + tile_column * 8 + pixel_column) as usize] = match color {
										0x00 => 0x00_54_54_54,
										0x01 => 0x00_00_1e_74,
										0x02 => 0x00_08_10_90,
										0x03 => 0x00_30_00_88,
										0x04 => 0x00_44_00_64,
										0x05 => 0x00_5c_00_30,
										0x06 => 0x00_54_04_00,
										0x07 => 0x00_3c_18_00,
										0x08 => 0x00_20_2a_00,
										0x09 => 0x00_08_3a_00,
										0x0a => 0x00_00_40_00,
										0x0b => 0x00_00_3c_00,
										0x0c => 0x00_00_32_3c,
										0x0d => 0x00_00_00_00,
										0x0e => 0x00_00_00_00,
										0x0f => 0x00_00_00_00,
										0x10 => 0x00_98_96_98,
										0x11 => 0x00_08_4c_c4,
										0x12 => 0x00_30_32_ec,
										0x13 => 0x00_5c_1e_e4,
										0x14 => 0x00_88_14_b0,
										0x15 => 0x00_a0_14_64,
										0x16 => 0x00_98_22_20,
										0x17 => 0x00_78_3c_00,
										0x18 => 0x00_54_5a_00,
										0x19 => 0x00_28_72_00,
										0x1a => 0x00_08_7c_00,
										0x1b => 0x00_00_76_28,
										0x1c => 0x00_00_66_78,
										0x1d => 0x00_00_00_00,
										0x1e => 0x00_00_00_00,
										0x1f => 0x00_00_00_00,
										0x20 => 0x00_ec_ee_ec,
										0x21 => 0x00_4c_9a_ec,
										0x22 => 0x00_78_7c_ec,
										0x23 => 0x00_b0_62_ec,
										0x24 => 0x00_e4_54_ec,
										0x25 => 0x00_ec_58_b4,
										0x26 => 0x00_ec_6a_64,
										0x27 => 0x00_d4_88_20,
										0x28 => 0x00_a0_aa_00,
										0x29 => 0x00_74_c4_00,
										0x2a => 0x00_4c_d0_20,
										0x2b => 0x00_38_cc_6c,
										0x2c => 0x00_38_b4_cc,
										0x2d => 0x00_3c_3c_3c,
										0x2e => 0x00_00_00_00,
										0x2f => 0x00_00_00_00,
										0x30 => 0x00_ec_ee_ec,
										0x31 => 0x00_a8_cc_ec,
										0x32 => 0x00_bc_bc_ec,
										0x33 => 0x00_d4_b2_ec,
										0x34 => 0x00_ec_ae_ec,
										0x35 => 0x00_ec_ae_d4,
										0x36 => 0x00_ec_b4_b0,
										0x37 => 0x00_e4_c4_90,
										0x38 => 0x00_cc_d2_78,
										0x39 => 0x00_b4_de_78,
										0x3a => 0x00_a8_e2_90,
										0x3b => 0x00_98_e2_b4,
										0x3c => 0x00_a0_d6_e4,
										0x3d => 0x00_a0_a2_a0,
										0x3e => 0x00_00_00_00,
										0x3f => 0x00_00_00_00,
										_ => {
											println!("[ERROR] [PPU] Wrong color {:02X}", color);
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
