pub mod registers;
mod memory;

#[cfg(test)]
mod tests;

#[cfg(feature = "nametable-viewer")]
mod nametable_viewer;

use screen::*;
use cpu::*;
use self::memory::*;

#[cfg(feature = "benchmark")]
use emulator::*;

#[cfg(feature = "nametable-viewer")]
use self::nametable_viewer::*;

const OAM_SIZE: usize = 256;

pub struct Ppu {
	ppuctrl: u8,
	ppumask: u8,
	ppustatus: u8,
	oamaddr: u8,
	scroll_x: u8,
	scroll_y: u8,
	ppuaddr: u16,
	ppudata_buffer: u8,
	flipflop: bool,
	cycle_counter: u16,
	scanline_counter: u16,
	oam: [u8; OAM_SIZE],
	memory: Memory,

	#[cfg(feature = "nametable-viewer")]
	nametable_viewer: NametableViewer
}

impl Ppu {
	pub fn new() -> Self {
		Self {
			ppuctrl: 0,
			ppumask: 0,
			ppustatus: 0,
			oamaddr: 0,
			scroll_x: 0,
			scroll_y: 0,
			ppuaddr: 0,
			ppudata_buffer: 0,
			flipflop: false,
			cycle_counter: 0,
			scanline_counter: 0,
			oam: [0; OAM_SIZE],
			memory: Memory::new(),

			#[cfg(feature = "nametable-viewer")]
			nametable_viewer: NametableViewer::new()
		}
	}

	pub fn load_chr_rom(&mut self, chr_rom: &[u8]) {
		self.memory.load_chr_rom(chr_rom);
	}

	pub fn do_cycle(&mut self, cpu: &mut Cpu, screen: &mut Screen) {
		self.cycle_counter += 1;
		if self.cycle_counter == 341 {
			self.cycle_counter = 0;
			self.scanline_counter = (self.scanline_counter + 1) % 262;
			match self.scanline_counter {
				// visible scanlines
				0 ..= 239 => {
					// render background
					if (self.ppumask & 0x08) != 0 {
						let nametable_address = 0x2000 + 0x400 * (self.ppuctrl & 0b11) as u16;
						let attribute_table_address = nametable_address + 0x3c0;
						let pattern_address = 0x1000 * ((self.ppuctrl >> 4) & 1) as u16;
						let yy = self.scanline_counter + self.scroll_y as u16;
						let tile_row = yy / 8;
						let pixel_row = yy % 8;
						let attribute_row = tile_row / 4;
						for x in 0..FRAME_WIDTH as u16 {
							let xx = x + self.scroll_x as u16;
							let tile_column = xx / 8;
							let pixel_column = xx % 8;
							let attribute_column = tile_column / 4;
							let attribute = self.memory.read(attribute_table_address + attribute_row * 8 + attribute_column);
							let palette_number = ((attribute >> (4 * ((tile_row / 2) % 2))) >> (2 * ((tile_column / 2) % 2))) & 0b11;
							let tile_number_address = nametable_address + tile_row * 32 + tile_column;
							let tile_number = self.memory.read(tile_number_address);
							let low_byte = self.memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row);
							let high_byte = self.memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row + 8);
							let low_bit = (low_byte >> (7 - pixel_column)) & 1;
							let high_bit = (high_byte >> (7 - pixel_column)) & 1;
							let color_number = (high_bit << 1) | low_bit;
							let color_address = if color_number == 0 {
								0 // backdrop color
							} else {
								4 * palette_number as u16 + color_number as u16
							} + 0x3f00;
							let color = self.memory.read(color_address);
							screen.set_pixel(x as _, self.scanline_counter as _, color);
						}
					}
					// render sprites
					if (self.ppumask & 0x10) != 0 {
						let sprite_size = (self.ppuctrl >> 6) & 1;
						if sprite_size == 1 {
							unimplemented!("8x16 sprites");
						}
						for number in (0..64).rev() {
							let sprite_y = self.oam[number * 4];
							let tile_number = self.oam[number * 4 + 1];
							let attributes = self.oam[number * 4 + 2];
							let palette_number = 4 + (attributes & 0b11);
							let horizontal_flip = (attributes & 0x40) != 0;
							let vertical_flip = (attributes & 0x80) != 0;
							let sprite_x = self.oam[number * 4 + 3];
							let pattern_address = 0x1000 * ((self.ppuctrl >> 3) & 1) as u16;
							for pixel_row in 0..8 {
								let y = (if vertical_flip {
									7 - pixel_row
								} else {
									pixel_row
								}) + sprite_y as u16;
								
								// ugly:
								if y != self.scanline_counter {
									continue;
								}
								
								let low_byte = self.memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row);
								let high_byte = self.memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row + 8);
								for pixel_column in 0..8 {
									let x = (if horizontal_flip {
										7 - pixel_column
									} else {
										pixel_column
									}) + sprite_x as u16;
									if x >= FRAME_WIDTH as u16 {
										// sprite in leftmost 8 pixels
										if (sprite_x < 8) && ((self.ppuctrl & 0x04) == 0) {
											break; // hide
										} else {
											continue; // show
										}
									}
									let low_bit = (low_byte >> (7 - pixel_column)) & 1;
									let high_bit = (high_byte >> (7 - pixel_column)) & 1;
									let color_number = (high_bit << 1) | low_bit;
									if color_number != 0 {
										if number == 0 && self.frame_buffer[(y as usize) * FRAME_WIDTH + x as usize] != 0 {
											self.ppustatus |= 0x40; // sprite 0 hit
										}
										let color = self.memory.read(0x3f00 + 4 * palette_number as u16 + color_number as u16);
										screen.set_pixel(x, y, color);
									}
								}
							}
						}
					}
				},
				// VBlank start
				241 => {
					self.ppustatus |= 0x80;
					if (self.ppuctrl & 0x80) != 0 {
						cpu.request_interrupt(Interrupt::Nmi);
					}

					#[cfg(not(test))]
					screen.request_draw();

					#[cfg(feature = "nametable-viewer")]
					NametableViewer::update(self);
				},
				// VBlank end
				261 => self.ppustatus &= 0x1f,
				_ => {}
			}
		}
	}
}
