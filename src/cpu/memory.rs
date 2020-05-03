use emulator::*;
use ppu::*;
use ppu::register::*;

pub(super) const STACK_ADDRESS: u16 = 0x100;
pub(super) const NMI_VECTOR_ADDRESS: u16 = 0xfffa;
pub(super) const RESET_VECTOR_ADDRESS: u16 = 0xfffc;
pub(super) const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;

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

const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7fff;

const PRG_ROM_START: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;

pub(super) fn read8(emulator: &mut Emulator, address: u16) -> u8 {
	match address {
		RAM_START ..= RAM_END => emulator.ram[(address % RAM_SIZE) as usize],
		PPUSTATUS_ADDRESS => Ppustatus::read(&mut emulator.ppu),
		PPUDATA_ADDRESS => Ppudata::read(&mut emulator.ppu),
		0x4000 ..= 0x4013 | 0x4015 | 0x4017 => {
			// TODO: implement APU registers
			warn!("Read from an APU register");
			0
		},
		JOY1_ADDRESS => emulator.joypad.read(&emulator.window),
		PRG_RAM_START ..= PRG_RAM_END => emulator.prg_ram[(address - PRG_RAM_START) as usize],
		PRG_ROM_START ..= PRG_ROM_END => emulator.prg_rom[((address - PRG_ROM_START) as usize) % emulator.prg_rom.len()],
		_ => {
			error!("Read from {:04X}", address);
			panic!();
		}
	}
}

pub(super) fn read16(emulator: &mut Emulator, address: u16) -> u16 {
	let low_byte = read8(emulator, address) as u16;
	let high_byte = read8(emulator, address.wrapping_add(1)) as u16;
	(high_byte << 8) | low_byte
}

#[cfg(feature = "trace")]
pub(super) fn read8_debug(emulator: &Emulator, address: u16) -> u8 {
	match address {
		RAM_START ..= RAM_END => emulator.ram[(address % RAM_SIZE) as usize],
		PPUCTRL_ADDRESS => Ppuctrl::read_debug(&emulator.ppu),
		PPUMASK_ADDRESS => Ppumask::read_debug(&emulator.ppu),
		PPUSTATUS_ADDRESS => Ppustatus::read_debug(&emulator.ppu),
		OAMADDR_ADDRESS => Oamaddr::read_debug(&emulator.ppu),
		OAMDATA_ADDRESS => Oamdata::read_debug(&emulator.ppu),
		PPUSCROLL_ADDRESS => Ppuscroll::read_debug(&emulator.ppu),
		PPUADDR_ADDRESS => Ppuaddr::read_debug(&emulator.ppu),
		PPUDATA_ADDRESS => Ppudata::read_debug(&emulator.ppu),
		JOY1_ADDRESS => emulator.joypad.read_debug(&emulator.window),
		PRG_RAM_START ..= PRG_RAM_END => emulator.prg_ram[(address - PRG_RAM_START) as usize],
		PRG_ROM_START ..= PRG_ROM_END => emulator.prg_rom[((address - PRG_ROM_START) as usize) % emulator.prg_rom.len()],
		_ => {
			warn!("Read from {:04X}", address);
			0
		}
	}
}

#[cfg(feature = "trace")]
pub(super) fn read16_debug(emulator: &Emulator, address: u16) -> u16 {
	let low_byte = read8_debug(emulator, address) as u16;
	let high_byte = read8_debug(emulator, address.wrapping_add(1)) as u16;
	(high_byte << 8) | low_byte
}

pub(super) fn write8(emulator: &mut Emulator, address: u16, value: u8) {
	match address {
		RAM_START ..= RAM_END => emulator.ram[(address % RAM_SIZE) as usize] = value,
		PPUCTRL_ADDRESS => Ppuctrl::write(&mut emulator.ppu, value),
		PPUMASK_ADDRESS => Ppumask::write(&mut emulator.ppu, value),
		OAMADDR_ADDRESS => Oamaddr::write(&mut emulator.ppu, value),
		OAMDATA_ADDRESS => Oamdata::write(&mut emulator.ppu, value),
		PPUSCROLL_ADDRESS => Ppuscroll::write(&mut emulator.ppu, value),
		PPUADDR_ADDRESS => Ppuaddr::write(&mut emulator.ppu, value),
		PPUDATA_ADDRESS => Ppudata::write(&mut emulator.ppu, value),
		0x4000 ..= 0x4013 | 0x4015 | 0x4017 => {
			// TODO: implement APU registers
			warn!("Write to an APU register");
		},
		OAMDMA_ADDRESS => {
				let start = (value as usize) << 8;
				let end = start + OAM_SIZE;
				emulator.ppu.write_oam(&emulator.ram[start..end]);
		},
		JOY1_ADDRESS => emulator.joypad.write(&emulator.window, value),
		PRG_RAM_START ..= PRG_RAM_END => emulator.prg_ram[(address - PRG_RAM_START) as usize] = value,
		_ => {
			error!("Write to {:04X}", address);
			panic!();
		}
	}
}
