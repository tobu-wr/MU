use emulator::*;
use ppu::registers::*;

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

const MAPPER_START: u16 = 0x6000;
const MAPPER_END: u16 = 0xffff;

pub(super) fn read8(emulator: &mut Emulator, address: u16) -> u8 {
	match address {
		RAM_START ..= RAM_END => emulator.ram[(address - RAM_START) as usize % RAM_SIZE],
		PPUCTRL_ADDRESS => Ppuctrl::read(&mut emulator.ppu),
		PPUMASK_ADDRESS => Ppumask::read(&mut emulator.ppu),
		PPUSTATUS_ADDRESS => Ppustatus::read(&mut emulator.ppu),
		OAMADDR_ADDRESS => Oamaddr::read(&mut emulator.ppu),
		OAMDATA_ADDRESS => Oamdata::read(&mut emulator.ppu),
		PPUSCROLL_ADDRESS => Ppuscroll::read(&mut emulator.ppu),
		PPUADDR_ADDRESS => Ppuaddr::read(&mut emulator.ppu),
		PPUDATA_ADDRESS => Ppudata::read(&mut emulator.ppu),
		0x2008 ..= 0x3fff => read8(emulator, 0x2000 + (address - 0x2000) % 8), // mirrors of 0x2000-0x2007
		OAMDMA_ADDRESS => Oamdma::read(),
		0x4000 ..= 0x4013 | 0x4015 | 0x4017 => {
			// TODO: implement APU registers
			warn!("Read from an APU register at {:04X}", address);
			0
		},
		JOY1_ADDRESS => emulator.joypad.read(&emulator.window),
		0x4018 ..= 0x5fff => {
			// TODO: implement open bus behavior and expansion ROM
			warn!("Read from expansion ROM at {:04X}", address);
			0
		},
		MAPPER_START ..= MAPPER_END => emulator.mapper.as_ref().unwrap().read(address)
	}
}

pub(super) fn read16(emulator: &mut Emulator, address: u16) -> u16 {
	let low_byte = read8(emulator, address) as u16;
	let high_byte = read8(emulator, address.wrapping_add(1)) as u16;
	(high_byte << 8) | low_byte
}

pub(super) fn read16_zeropage(emulator: &mut Emulator, address: u8) -> u16 {
	let low_byte = read8(emulator, address as _) as u16;
	let high_byte = read8(emulator, address.wrapping_add(1) as _) as u16;
	(high_byte << 8) | low_byte
}

#[cfg(feature = "trace")]
pub(super) fn read8_debug(emulator: &Emulator, address: u16) -> u8 {
	match address {
		RAM_START ..= RAM_END => emulator.ram[(address - RAM_START) as usize % RAM_SIZE],
		PPUCTRL_ADDRESS => Ppuctrl::read_debug(&emulator.ppu),
		PPUMASK_ADDRESS => Ppumask::read_debug(&emulator.ppu),
		PPUSTATUS_ADDRESS => Ppustatus::read_debug(&emulator.ppu),
		OAMADDR_ADDRESS => Oamaddr::read_debug(&emulator.ppu),
		OAMDATA_ADDRESS => Oamdata::read_debug(&emulator.ppu),
		PPUSCROLL_ADDRESS => Ppuscroll::read_debug(&emulator.ppu),
		PPUADDR_ADDRESS => Ppuaddr::read_debug(&emulator.ppu),
		PPUDATA_ADDRESS => Ppudata::read_debug(&emulator.ppu),
		0x2008 ..= 0x3fff => read8_debug(emulator, 0x2000 + (address - 0x2000) % 8), // mirrors of 0x2000-0x2007
		OAMDMA_ADDRESS => Oamdma::read_debug(),
		JOY1_ADDRESS => emulator.joypad.read_debug(&emulator.window),
		MAPPER_START ..= MAPPER_END => emulator.mapper.as_ref().unwrap().read(address),
		_ => 0
	}
}

#[cfg(feature = "trace")]
pub(super) fn read16_debug(emulator: &Emulator, address: u16) -> u16 {
	let low_byte = read8_debug(emulator, address) as u16;
	let high_byte = read8_debug(emulator, address.wrapping_add(1)) as u16;
	(high_byte << 8) | low_byte
}

#[cfg(feature = "trace")]
pub(super) fn read16_zeropage_debug(emulator: &Emulator, address: u8) -> u16 {
	let low_byte = read8_debug(emulator, address as _) as u16;
	let high_byte = read8_debug(emulator, address.wrapping_add(1) as _) as u16;
	(high_byte << 8) | low_byte
}

pub(super) fn write(emulator: &mut Emulator, address: u16, value: u8) {
	match address {
		RAM_START ..= RAM_END => emulator.ram[(address - RAM_START) as usize % RAM_SIZE] = value,
		PPUCTRL_ADDRESS => Ppuctrl::write(&mut emulator.ppu, value),
		PPUMASK_ADDRESS => Ppumask::write(&mut emulator.ppu, value),
		PPUSTATUS_ADDRESS => Ppustatus::write(&mut emulator.ppu, value),
		OAMADDR_ADDRESS => Oamaddr::write(&mut emulator.ppu, value),
		OAMDATA_ADDRESS => Oamdata::write(&mut emulator.ppu, value),
		PPUSCROLL_ADDRESS => Ppuscroll::write(&mut emulator.ppu, value),
		PPUADDR_ADDRESS => Ppuaddr::write(&mut emulator.ppu, value),
		PPUDATA_ADDRESS => Ppudata::write(&mut emulator.ppu, value),
		0x2008 ..= 0x3fff => write(emulator, 0x2000 + (address - 0x2000) % 8, value), // mirrors of 0x2000-0x2007
		OAMDMA_ADDRESS => Oamdma::write(emulator, value),
		0x4000 ..= 0x4013 | 0x4015 | 0x4017 => {
			// TODO: implement APU registers
			warn!("Write to an APU register at {:04X}", address);
		},
		JOY1_ADDRESS => emulator.joypad.write(&emulator.window, value),
		0x4018 ..= 0x5fff => {
			// TODO: implement expansion ROM
			warn!("Write to expansion ROM at {:04X}", address);
		},
		MAPPER_START ..= MAPPER_END => emulator.mapper.as_mut().unwrap().write(address, value)
	}
}
