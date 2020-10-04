use super::*;

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
		read_next8(emulator) as _
	}
}

impl AddressingMode for ZeroPageX {
	fn get_address(emulator: &mut Emulator) -> u16 {
		read_next8(emulator).wrapping_add(emulator.cpu.x) as _
	}
}

impl AddressingMode for ZeroPageY {
	fn get_address(emulator: &mut Emulator) -> u16 {
		read_next8(emulator).wrapping_add(emulator.cpu.y) as _
	}
}

impl AddressingMode for Absolute {
	fn get_address(emulator: &mut Emulator) -> u16 {
		read_next16(emulator)
	}
}

impl AddressingMode for AbsoluteX {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let result = read_next16(emulator).wrapping_add(emulator.cpu.x as _);
		emulator.cpu.page_crossing = (result & 0xff00) != result.wrapping_sub(emulator.cpu.x as _) & 0xff00;
		result
	}
}

impl AddressingMode for AbsoluteY {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let result = read_next16(emulator).wrapping_add(emulator.cpu.y as _);
		emulator.cpu.page_crossing = (result & 0xff00) != result.wrapping_sub(emulator.cpu.y as _) & 0xff00;
		result
	}
}

impl AddressingMode for IndirectX {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read_next8(emulator).wrapping_add(emulator.cpu.x);
		read16_zeropage(emulator, address)
	}
}

impl AddressingMode for IndirectY {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read_next8(emulator);
		let result = read16_zeropage(emulator, address).wrapping_add(emulator.cpu.y as _);
		emulator.cpu.page_crossing = (result & 0xff00) != result.wrapping_sub(emulator.cpu.y as _) & 0xff00;
		result
	}
}
