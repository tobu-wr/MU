use ppu::*;
use joypad::*;

pub const STACK_ADDRESS: u16 = 0x100;
pub const NMI_VECTOR_ADDRESS: u16 = 0xfffa;
pub const RESET_VECTOR_ADDRESS: u16 = 0xfffc;
pub const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;

const RAM_SIZE: u16 = 0x800;
const RAM_START: u16 = 0;
const RAM_END: u16 = 0x1fff;

const PPUCTRL_ADDRESS: u16 = 0x2000;
const PPUMASK_ADDRESS: u16 = 0x2001;
const PPUSTATUS_ADDRESS: u16 = 0x2002;
const OAMADDR_ADDRESS: u16 = 0x2003;
const OAMDATA_ADDRESS: u16 = 0x2004;
const PPUSCROLL_ADDRESS: u16 = 0x2005;
const PPUADDR_ADDRESS: u16 = 0x2006;
const PPUDATA_ADDRESS: u16 = 0x2007;
const OAMDMA_ADDRESS: u16 = 0x4014;
const JOY1_ADDRESS: u16 = 0x4016;

const PRG_ROM_START: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;

pub struct CpuMemory {
	ram: [u8; RAM_SIZE as _],
	prg_rom: Vec<u8>,
	ppu: *mut Ppu,
	joypad: *mut Joypad
}

impl CpuMemory {
	pub fn new() -> Self {
		Self {
			ram: [0; RAM_SIZE as _],
			prg_rom: Vec::new(),
			ppu: std::ptr::null_mut(),
			joypad: std::ptr::null_mut()
		}
	}

	pub fn connect(&mut self, ppu: *mut Ppu, joypad: *mut Joypad) {
		self.ppu = ppu;
		self.joypad = joypad;
	}

	pub fn load_prg_rom(&mut self, prg_rom: &[u8]) {
		self.prg_rom = prg_rom.to_vec();
	}

	fn read_ppu(&self, register: Register) -> u8 {
		unsafe {
			(*self.ppu).read_register(register)
		}
	}

	#[cfg(feature = "log")]
	fn read_ppu_debug(&self, register: Register) -> u8 {
		unsafe {
			(*self.ppu).read_register_debug(register)
		}
	}

	fn write_ppu(&self, register: Register, value: u8) {
		unsafe {
			(*self.ppu).write_register(register, value);
		}
	}

	pub fn read8(&self, address: u16) -> u8 {
		match address {
			RAM_START ..= RAM_END => self.ram[(address % RAM_SIZE) as usize],
			PPUSTATUS_ADDRESS => self.read_ppu(Register::Ppustatus),
			PPUDATA_ADDRESS => self.read_ppu(Register::Ppudata),
			0x4000 ..= 0x4013 | 0x4015 | 0x4017 => {
				println!("[DEBUG] [CPU] Read from an APU register");
				0
			},
			JOY1_ADDRESS => unsafe {
				(*self.joypad).read()
			},
			PRG_ROM_START ..= PRG_ROM_END => self.prg_rom[((address - PRG_ROM_START) as usize) % self.prg_rom.len()],
			_ => {
				println!("[ERROR] [CPU] Unhandled read from {:04X}", address);
				std::process::exit(1);
			}
		}
	}

	#[cfg(feature = "log")]
	pub fn read8_debug(&self, address: u16) -> u8 {
		match address {
			PPUCTRL_ADDRESS => self.read_ppu_debug(Register::Ppuctrl),
			PPUMASK_ADDRESS => self.read_ppu_debug(Register::Ppumask),
			PPUSTATUS_ADDRESS => self.read_ppu_debug(Register::Ppustatus),
			PPUSCROLL_ADDRESS => self.read_ppu_debug(Register::Ppuscroll),
			PPUADDR_ADDRESS => self.read_ppu_debug(Register::Ppuaddr),
			PPUDATA_ADDRESS => self.read_ppu_debug(Register::Ppudata),
			_ => self.read8(address)
		}
	}

	pub fn read16(&self, address: u16) -> u16 {
		let low_byte = self.read8(address) as u16;
		let high_byte = self.read8(address.wrapping_add(1)) as u16;
		(high_byte << 8) | low_byte
	}

	pub fn write8(&mut self, address: u16, value: u8) {
		match address {
			RAM_START ..= RAM_END => self.ram[(address % RAM_SIZE) as usize] = value,
			PPUCTRL_ADDRESS => self.write_ppu(Register::Ppuctrl, value),
			PPUMASK_ADDRESS => self.write_ppu(Register::Ppumask, value),
			OAMADDR_ADDRESS => self.write_ppu(Register::Oamaddr, value),
			OAMDATA_ADDRESS => println!("[DEBUG] [CPU] Write to OAMDATA {:02X}", value),
			PPUSCROLL_ADDRESS => self.write_ppu(Register::Ppuscroll, value),
			PPUADDR_ADDRESS => self.write_ppu(Register::Ppuaddr, value),
			PPUDATA_ADDRESS => self.write_ppu(Register::Ppudata, value),
			0x4000 ..= 0x4013 | 0x4015 | 0x4017 => println!("[DEBUG] [CPU] Write to an APU register"),
			OAMDMA_ADDRESS => {
				let start_address = (value as usize) << 8;
				let end_address = start_address + OAM_SIZE;
				let data = &self.ram[start_address..end_address];
				unsafe {
					(*self.ppu).write_oam(data);
				}
			},
			JOY1_ADDRESS => unsafe {
				(*self.joypad).write(value);
			},
			_ => {
				println!("[ERROR] [CPU] Unhandled write to {:04X}", address);
				std::process::exit(1);
			}
		}
	}
}
