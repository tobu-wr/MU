use emulator::*;
use ppu::*;
use joypad::*;

pub const STACK_ADDRESS: u16 = 0x100;
pub const NMI_VECTOR_ADDRESS: u16 = 0xfffa;
pub const RESET_VECTOR_ADDRESS: u16 = 0xfffc;
pub const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;

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

pub fn read8(emulator: &mut Emulator, address: u16) -> u8 {
	match address {
		RAM_START ..= RAM_END => emulator.ram[(address % RAM_SIZE) as usize],
		PPUSTATUS_ADDRESS => emulator.ppu.read_register(Register::Ppustatus),
		PPUDATA_ADDRESS => emulator.ppu.read_register(Register::Ppudata),
		0x4000 ..= 0x4013 | 0x4015 | 0x4017 => {
			println!("[DEBUG] [CPU] Read from an APU register");
			0
		},
		JOY1_ADDRESS => Joypad::read(emulator),
		PRG_ROM_START ..= PRG_ROM_END => emulator.prg_rom[((address - PRG_ROM_START) as usize) % emulator.prg_rom.len()],
		_ => {
			println!("[ERROR] [CPU] Unhandled read from {:04X}", address);
			std::process::exit(1);
		}
	}
}

#[cfg(feature = "log")]
pub fn read8_debug(emulator: &mut Emulator, address: u16) -> u8 {
	match address {
		PPUCTRL_ADDRESS => emulator.ppu.read_register_debug(Register::Ppuctrl),
		PPUMASK_ADDRESS => emulator.ppu.read_register_debug(Register::Ppumask),
		PPUSTATUS_ADDRESS => emulator.ppu.read_register_debug(Register::Ppustatus),
		PPUSCROLL_ADDRESS => emulator.ppu.read_register_debug(Register::Ppuscroll),
		PPUADDR_ADDRESS => emulator.ppu.read_register_debug(Register::Ppuaddr),
		PPUDATA_ADDRESS => emulator.ppu.read_register_debug(Register::Ppudata),
		_ => read8(emulator, address)
	}
}

pub fn read16(emulator: &mut Emulator, address: u16) -> u16 {
	let low_byte = read8(emulator, address) as u16;
	let high_byte = read8(emulator, address.wrapping_add(1)) as u16;
	(high_byte << 8) | low_byte
}

pub fn write8(emulator: &mut Emulator, address: u16, value: u8) {
	match address {
		RAM_START ..= RAM_END => emulator.ram[(address % RAM_SIZE) as usize] = value,
		PPUCTRL_ADDRESS => emulator.ppu.write_register(Register::Ppuctrl, value),
		PPUMASK_ADDRESS => emulator.ppu.write_register(Register::Ppumask, value),
		OAMADDR_ADDRESS => emulator.ppu.write_register(Register::Oamaddr, value),
		OAMDATA_ADDRESS => println!("[DEBUG] [CPU] Write to OAMDATA {:02X}", value),
		PPUSCROLL_ADDRESS => emulator.ppu.write_register(Register::Ppuscroll, value),
		PPUADDR_ADDRESS => emulator.ppu.write_register(Register::Ppuaddr, value),
		PPUDATA_ADDRESS => emulator.ppu.write_register(Register::Ppudata, value),
		0x4000 ..= 0x4013 | 0x4015 | 0x4017 => println!("[DEBUG] [CPU] Write to an APU register"),
		OAMDMA_ADDRESS => {
				let start = (value as usize) << 8;
				let end = start + OAM_SIZE;
				emulator.ppu.write_oam(&emulator.ram[start..end]);
		},
		JOY1_ADDRESS => Joypad::write(emulator, value),
		_ => {
			println!("[ERROR] [CPU] Unhandled write to {:04X}", address);
			std::process::exit(1);
		}
	}
}
