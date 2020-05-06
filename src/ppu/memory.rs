const TABLE_SIZE: u16 = 0x3000;
const TABLE_START: u16 = 0x0000;
const TABLE_END: u16 = TABLE_START + TABLE_SIZE - 1;

const TABLE_MIRROR_OFFSET: u16 = 0x1000;
const TABLE_MIRROR_START: u16 = 0x3000;
const TABLE_MIRROR_END: u16 = 0x3eff;

const PALETTE_SIZE: u16 = 0x20;
const PALETTE_START: u16 = 0x3f00;
const PALETTE_END: u16 = 0x3fff;

const BACKGROUND_PALETTE_SIZE: u16 = 0x10;
const BACKGROUND_PALETTE_START: u16 = 0x3f00;
const BACKGROUND_PALETTE_END: u16 = BACKGROUND_PALETTE_START + BACKGROUND_PALETTE_SIZE - 1;

const BACKGROUND_PALETTE_MIRROR_ADDRESS_0: u16 = 0x3f10;
const BACKGROUND_PALETTE_MIRROR_ADDRESS_1: u16 = 0x3f14;
const BACKGROUND_PALETTE_MIRROR_ADDRESS_2: u16 = 0x3f18;
const BACKGROUND_PALETTE_MIRROR_ADDRESS_3: u16 = 0x3f1c;

const SPRITE_PALETTE_SIZE: usize = 12;

const SPRITE_PALETTE_0_START: u16 = 0x3f11;
const SPRITE_PALETTE_0_END: u16 = 0x3f13;

const SPRITE_PALETTE_1_START: u16 = 0x3f15;
const SPRITE_PALETTE_1_END: u16 = 0x3f17;

const SPRITE_PALETTE_2_START: u16 = 0x3f19;
const SPRITE_PALETTE_2_END: u16 = 0x3f1b;

const SPRITE_PALETTE_3_START: u16 = 0x3f1d;
const SPRITE_PALETTE_3_END: u16 = 0x3f1f;

pub(super) struct Memory {
	table: [u8; TABLE_SIZE as _],
	background_palette: [u8; BACKGROUND_PALETTE_SIZE as _],
	sprite_palette: [u8; SPRITE_PALETTE_SIZE]
}

impl Memory {
	pub(super) fn new() -> Self {
		Self {
			table: [0; TABLE_SIZE as _],
			background_palette: [0; BACKGROUND_PALETTE_SIZE as _],
			sprite_palette: [0; SPRITE_PALETTE_SIZE]
		}
	}

	pub(super) fn load_chr_rom(&mut self, chr_rom: &[u8]) {
		self.table[..chr_rom.len()].copy_from_slice(chr_rom);
	}

	pub(super) fn read(&self, address: u16) -> u8 {
		match address {
			TABLE_START ..= TABLE_END => self.table[(address - TABLE_START) as usize],
			TABLE_MIRROR_START ..= TABLE_MIRROR_END => self.table[(address - TABLE_MIRROR_OFFSET) as usize],
			PALETTE_START ..= PALETTE_END => match PALETTE_START + address % PALETTE_SIZE {
				BACKGROUND_PALETTE_START ..= BACKGROUND_PALETTE_END => self.background_palette[(address - BACKGROUND_PALETTE_START) as usize],
				BACKGROUND_PALETTE_MIRROR_ADDRESS_0 => self.background_palette[0],
				BACKGROUND_PALETTE_MIRROR_ADDRESS_1 => self.background_palette[4],
				BACKGROUND_PALETTE_MIRROR_ADDRESS_2 => self.background_palette[8],
				BACKGROUND_PALETTE_MIRROR_ADDRESS_3 => self.background_palette[12],
				SPRITE_PALETTE_0_START ..= SPRITE_PALETTE_0_END |
				SPRITE_PALETTE_1_START ..= SPRITE_PALETTE_1_END |
				SPRITE_PALETTE_2_START ..= SPRITE_PALETTE_2_END |
				SPRITE_PALETTE_3_START ..= SPRITE_PALETTE_3_END => {
					let palette = (address >> 2) & 0b11;
					let index = address & 0b11;
					self.sprite_palette[(palette * 3 + index - 1) as usize]
				},
				_ => {
					error!("Read from {:04X}", address);
					panic!();
				}
			},
			_ => {
				error!("Read from {:04X}", address);
				panic!();
			}
		}
	}

	pub(super) fn write(&mut self, address: u16, value: u8) {
		match address {
			TABLE_START ..= TABLE_END => self.table[(address - TABLE_START) as usize] = value,
			TABLE_MIRROR_START ..= TABLE_MIRROR_END => self.table[(address - TABLE_MIRROR_OFFSET) as usize] = value,
			PALETTE_START ..= PALETTE_END => match PALETTE_START + address % PALETTE_SIZE {
				BACKGROUND_PALETTE_START ..= BACKGROUND_PALETTE_END => self.background_palette[(address - BACKGROUND_PALETTE_START) as usize] = value,
				BACKGROUND_PALETTE_MIRROR_ADDRESS_0 => self.background_palette[0] = value,
				BACKGROUND_PALETTE_MIRROR_ADDRESS_1 => self.background_palette[4] = value,
				BACKGROUND_PALETTE_MIRROR_ADDRESS_2 => self.background_palette[8] = value,
				BACKGROUND_PALETTE_MIRROR_ADDRESS_3 => self.background_palette[12] = value,
				SPRITE_PALETTE_0_START ..= SPRITE_PALETTE_0_END |
				SPRITE_PALETTE_1_START ..= SPRITE_PALETTE_1_END |
				SPRITE_PALETTE_2_START ..= SPRITE_PALETTE_2_END |
				SPRITE_PALETTE_3_START ..= SPRITE_PALETTE_3_END => {
					let palette = (address >> 2) & 0b11;
					let index = address & 0b11;
					self.sprite_palette[(palette * 3 + index - 1) as usize]  = value;
				},
				_ => {
					error!("Write to {:04X}", address);
					panic!();
				}
			},
			_ => {
				error!("Write to {:04X}", address);
				panic!();
			}
		}
	}
}
