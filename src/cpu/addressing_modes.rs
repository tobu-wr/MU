use emulator::*;
use super::memory::*;

pub(super) struct Immediate;
pub(super) struct ZeroPage;
pub(super) struct ZeroPageX;
pub(super) struct ZeroPageY;
pub(super) struct Absolute;
pub(super) struct AbsoluteX;
pub(super) struct AbsoluteY;
pub(super) struct IndirectX;
pub(super) struct IndirectY;

pub(super) trait AddressingMode {
	fn get_address(emulator: &mut Emulator) -> u16;
}

impl AddressingMode for Immediate {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = emulator.cpu.pc;
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
		address
	}
}

impl AddressingMode for ZeroPage {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read8(emulator, emulator.cpu.pc);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
		address as _
	}
}

impl AddressingMode for ZeroPageX {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read8(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.x);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
		address as _
	}
}

impl AddressingMode for ZeroPageY {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read8(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.y);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
		address as _
	}
}

impl AddressingMode for Absolute {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read16(emulator, emulator.cpu.pc);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2);
		address
	}
}

impl AddressingMode for AbsoluteX {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read16(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.x as _);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2);
		address
	}
}

impl AddressingMode for AbsoluteY {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read16(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.y as _);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2);
		address
	}
}

impl AddressingMode for IndirectX {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read8(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.x);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
		let low_byte = read8(emulator, address as _) as u16;
		let high_byte = read8(emulator, address.wrapping_add(1) as _) as u16;
		(high_byte << 8) | low_byte
	}
}

impl AddressingMode for IndirectY {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read8(emulator, emulator.cpu.pc);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
		let low_byte = read8(emulator, address as _) as u16;
		let high_byte = read8(emulator, address.wrapping_add(1) as _) as u16;
		let value = (high_byte << 8) | low_byte;
		value.wrapping_add(emulator.cpu.y as _)
	}
}
