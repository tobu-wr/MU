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
	fn get_address(&mut Emulator) -> u16;
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
		let address = read_next16(emulator);
		let effective_address = address.wrapping_add(emulator.cpu.x as _);
		emulator.cpu.check_page_crossing(address, effective_address);
		effective_address
	}
}

impl AddressingMode for AbsoluteY {
	fn get_address(emulator: &mut Emulator) -> u16 {
		let address = read_next16(emulator);
		let effective_address = address.wrapping_add(emulator.cpu.y as _);
		emulator.cpu.check_page_crossing(address, effective_address);
		effective_address
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
		let immediate = read_next8(emulator);
		let address = read16_zeropage(emulator, immediate);
		let effective_address = address.wrapping_add(emulator.cpu.y as _);
		emulator.cpu.check_page_crossing(address, effective_address);
		effective_address
	}
}
