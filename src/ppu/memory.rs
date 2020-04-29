const TABLES_SIZE: u16 = 0x3000;
const TABLES_START: u16 = 0x0000;
const TABLES_END: u16 = TABLES_START + TABLES_SIZE - 1;

const TABLES_MIRRORS_OFFSET: u16 = 0x1000;
const TABLES_MIRRORS_START: u16 = 0x3000;
const TABLES_MIRRORS_END: u16 = 0x3eff;

const PALETTES_SIZE: u16 = 0x20;
const PALETTES_START: u16 = 0x3f00;
const PALETTES_END: u16 = 0x3fff;

pub struct Memory {
	tables: [u8; TABLES_SIZE as _],
	palettes: [u8; PALETTES_SIZE as _]
}

impl Memory {
	pub fn new() -> Self {
		Self {
			tables: [0; TABLES_SIZE as _],
			palettes: [0; PALETTES_SIZE as _]
		}
	}

	pub fn load_chr_rom(&mut self, chr_rom: &[u8]) {
		self.tables[..chr_rom.len()].copy_from_slice(chr_rom);
	}

	pub(super) fn read(&self, address: u16) -> u8 {
		match address {
			TABLES_START ..= TABLES_END => self.tables[address as usize],
			TABLES_MIRRORS_START ..= TABLES_MIRRORS_END => self.tables[(address - TABLES_MIRRORS_OFFSET) as usize],
			PALETTES_START ..= PALETTES_END => self.palettes[(address % PALETTES_SIZE) as usize],
			_ => {
				println!("[ERROR] [PPU] Read from {:04X}", address);
				std::process::exit(1);
			}
		}
	}

	pub(super) fn write(&mut self, address: u16, value: u8) {
		match address {
			TABLES_START ..= TABLES_END => self.tables[address as usize] = value,
			TABLES_MIRRORS_START ..= TABLES_MIRRORS_END => self.tables[(address - TABLES_MIRRORS_OFFSET) as usize] = value,
			PALETTES_START ..= PALETTES_END => self.palettes[(address % PALETTES_SIZE) as usize] = value,
			_ => {
				println!("[ERROR] [PPU] Write to {:04X}", address);
				std::process::exit(1);
			}
		}
	}
}
