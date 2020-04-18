use ppu::*;

pub const RESET_VECTOR_ADDRESS: u16 = 0xfffc;
pub const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;

pub const STACK_ADDRESS: u16 = 0x100;

const RAM_START: u16 = 0;
const RAM_END: u16 = 0x1fff;
const RAM_SIZE: u16 = 0x800;

const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7fff;

const PRG_ROM_START: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;

const APU_STATUS_ADDRESS: u16 = 0x4015;

const PPUCTRL_ADDRESS: u16 = 0x2000;
const PPUMASK_ADDRESS: u16 = 0x2001;
const PPUSTATUS_ADDRESS: u16 = 0x2002;
const PPUSCROLL_ADDRESS: u16 = 0x2005;
const PPUADDR_ADDRESS: u16 = 0x2006;
const PPUDATA_ADDRESS: u16 = 0x2007;

pub struct CpuMemory {
	ram: [u8; RAM_SIZE as _],
	//prg_ram: 
	prg_rom: Vec<u8>,
	ppu: *mut Ppu
}

impl CpuMemory {
	pub fn new() -> Self {
		Self {
			ram: [0; RAM_SIZE as _],
			//prg_ram: 
			prg_rom: Vec::new(),
			ppu: std::ptr::null_mut()
		}
	}

	pub fn connect(&mut self, ppu: *mut Ppu) {
		self.ppu = ppu;
	}

	pub fn load_rom(&mut self, filename: &str) {
		let contents = std::fs::read(filename).unwrap();

		if &contents[..4] != [0x4e, 0x45, 0x53, 0x1a] {
			println!("[ERROR] Wrong file format");
			std::process::exit(1);
		}

		let prg_rom_size = (contents[4] as usize) * 16;
		println!("[INFO] PRG ROM size: {}KB", prg_rom_size);

		const HEADER_SIZE: usize = 16;
		self.prg_rom = contents[HEADER_SIZE..(HEADER_SIZE + prg_rom_size * 1024)].to_vec();

		let mapper = (contents[7] & 0xf0) | (contents[6] >> 4);
		println!("[INFO] Mapper: {}", mapper);
	}

	fn read_ppu(&self, register: Register) -> u8 {
		unsafe {
			(*self.ppu).read(register)
		}
	}

	pub fn read8(&self, address: u16) -> u8 {
		match address {
			RAM_START ..= RAM_END => {
				let effective_address = address as usize % RAM_SIZE as usize;
				self.ram[effective_address]
			},
			PPUSTATUS_ADDRESS => self.read_ppu(Register::Ppustatus),
			0x4000 | 0x4001 | 0x4002 | 0x4003 | 0x4004 | 0x4005 | 0x4006 | 0x4007 | 0x4008 | 0x4009 | 0x400a | 0x400b | 0x400c | 0x400d | 0x400e | 0x400f | 0x4010 | 0x4011 | 0x4012 | 0x4013 | 0x4017 | APU_STATUS_ADDRESS => {
				println!("[DEBUG] Read from an APU register");
				0
			},
			PRG_ROM_START ..= PRG_ROM_END => {
				let mut effective_address = (address - PRG_ROM_START) as usize;
				if effective_address >= self.prg_rom.len() {
					effective_address -= self.prg_rom.len();
				}

				self.prg_rom[effective_address]
			},
			_ => {
				println!("[ERROR] Unhandled read8 from {:04X}", address);
				std::process::exit(1);
			}
		}
	}

	#[cfg(feature = "log")]
	fn read_ppu_debug(&self, register: Register) -> u8 {
		unsafe {
			(*self.ppu).read_debug(register)
		}
	}

	#[cfg(feature = "log")]
	pub fn read8_debug(&self, address: u16) -> u8 {
		match address {
			PPUCTRL_ADDRESS => self.read_ppu_debug(Register::Ppuctrl),
			PPUSTATUS_ADDRESS => self.read_ppu_debug(Register::Ppustatus),
			_ => self.read8(address)
		}
	}

	pub fn read16(&self, address: u16) -> u16 {
		match address {
			RAM_START ..= RAM_END => {
				let effective_address = address as usize % RAM_SIZE as usize;
				let low_byte = self.ram[effective_address] as u16;
				let high_byte = self.ram[effective_address.wrapping_add(1)] as u16;
				(high_byte << 8) | low_byte
			},
			PRG_ROM_START ..= PRG_ROM_END => {
				let mut effective_address = (address - PRG_ROM_START) as usize;
				if effective_address >= self.prg_rom.len() {
					effective_address -= self.prg_rom.len();
				}

				let low_byte = self.prg_rom[effective_address] as u16;
				let high_byte = self.prg_rom[effective_address.wrapping_add(1)] as u16;
				(high_byte << 8) | low_byte
			},
			_ => {
				println!("[ERROR] Unhandled read16 from {:04X}", address);
				std::process::exit(1);
			}
		}
	}

	fn write_ppu(&self, register: Register, value: u8) {
		unsafe {
			(*self.ppu).write(register, value);
		}
	}

	pub fn write8(&mut self, address: u16, value: u8) {
		match address {
			RAM_START ..= RAM_END => {
				let effective_address = address as usize % RAM_SIZE as usize;
				self.ram[effective_address] = value;
			},
			PRG_RAM_START ..= PRG_RAM_END => {
				println!("[DEBUG] Write to PRG RAM {:02X}", value);
			},
			PPUCTRL_ADDRESS => self.write_ppu(Register::Ppuctrl, value),
			PPUMASK_ADDRESS => self.write_ppu(Register::Ppumask, value),
			0x2003 => {
				println!("[DEBUG] Write to OAMADDR {:02X}", value);
			},
			0x2004 => {
				println!("[DEBUG] Write to OAMDATA {:02X}", value);
			},
			PPUSCROLL_ADDRESS => self.write_ppu(Register::Ppuscroll, value),
			PPUADDR_ADDRESS => self.write_ppu(Register::Ppuaddr, value),
			PPUDATA_ADDRESS => self.write_ppu(Register::Ppudata, value),
			0x4000 | 0x4001 | 0x4002 | 0x4003 | 0x4004 | 0x4005 | 0x4006 | 0x4007 | 0x4008 | 0x4009 | 0x400a | 0x400b | 0x400c | 0x400d | 0x400e | 0x400f | 0x4010 | 0x4011 | 0x4012 | 0x4013 | 0x4017 | APU_STATUS_ADDRESS => {
				println!("[DEBUG] Write to an APU register");
			},
			_ => {
				println!("[ERROR] Unhandled write to {:04X}", address);
				std::process::exit(1);
			}
		}
	}

	/*pub fn write16(&mut self, address: u16, value: u16) {
		match address {
			RAM_START ..= RAM_END => {
				let effective_address = address as usize % RAM_SIZE as usize;
				self.ram[effective_address] = value as u8;
				self.ram[effective_address + 1] = (value >> 8) as u8;

				if cfg!(feature = "nestest") && (effective_address == 0x02 || effective_address == 0x03) && self.ram[effective_address] != 0 {
					println!("[ERROR] Failure code: {:x}", value);
					std::process::exit(1);
				}

				if cfg!(feature = "nestest") && ((effective_address + 1) == 0x02 || (effective_address + 1) == 0x03) && self.ram[effective_address + 1] != 0 {
					println!("[ERROR] Failure code: {:x}", value);
					std::process::exit(1);
				}
			},
			_ => {
				println!("[ERROR] Unhandled write at 0x{:x}", address);
				std::process::exit(1);
			}
		}
	}*/
}
