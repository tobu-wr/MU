const TABLES_SIZE: usize = 0x3000;
const TABLES_START: usize = 0x0000;
const TABLES_END: usize = TABLES_START + TABLES_SIZE - 1;

const PALETTES_SIZE: usize = 0x20;
const PALETTES_START: usize = 0x3f00;
const PALETTES_END: usize = PALETTES_START + PALETTES_SIZE - 1;

pub struct PpuMemory {
	tables: [u8; TABLES_SIZE],
	palettes: [u8; PALETTES_SIZE]
}

impl PpuMemory {
	pub fn new() -> Self {
		Self {
			tables: [0; TABLES_SIZE],
			palettes: [0; PALETTES_SIZE]
		}
	}

	pub fn load_chr_rom(&mut self, chr_rom: &[u8]) {
		self.tables[..chr_rom.len()].copy_from_slice(chr_rom);
	}

	pub fn read(&self, address: u16) -> u8 {
		match address as usize {
			TABLES_START ..= TABLES_END => self.tables[address as usize],
			PALETTES_START ..= PALETTES_END => self.palettes[(address as usize) - PALETTES_START],
			_ => {
				println!("[ERROR] [PPU] Unhandled read from {:04X}", address);
				std::process::exit(1);
			}
		}
	}

	pub fn write(&mut self, address: u16, value: u8) {
		match address as usize {
			TABLES_START ..= TABLES_END => self.tables[address as usize] = value,
			PALETTES_START ..= PALETTES_END => self.palettes[(address as usize) - PALETTES_START] = value,
			_ => {
				println!("[ERROR] [PPU] Unhandled write to {:04X}", address);
				std::process::exit(1);
			}
		}
	}
}
