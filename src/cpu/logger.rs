use std::io::prelude::Write;
use std::fs::File;
use std::cell::RefCell;

use emulator::*;
use super::memory::*;

const BUFFER_CAPACITY: usize = 44_000;

struct Data {
	pc: u16,
	opcode: u8,
	opcode_data: Vec<u16>,
	a: u8,
	x: u8,
	y: u8,
	p: u8,
	s: u8
}

struct Buffer {
	data: Vec<Data>,
	index: usize
}

impl Buffer {
	fn new() -> Self {
		Self {
			data: Vec::with_capacity(BUFFER_CAPACITY),
			index: 0
		}
	}

	fn push(&mut self, data: Data) {
		if self.data.len() < BUFFER_CAPACITY {
			self.data.push(data);
		} else {
			self.data[self.index] = data;
			self.index = (self.index + 1) % BUFFER_CAPACITY;
		}
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		let mut file = File::create("trace.log").unwrap();
		for i in 0..self.data.len() {
			let index = (self.index + i) % BUFFER_CAPACITY;
			let data = &self.data[index];
			let instruction = format_instruction(data);
			let log = format!("{:04X}  {:02X} {:<38} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}\n", data.pc, data.opcode, instruction, data.a, data.x, data.y, data.p, data.s);
			file.write_all(log.as_bytes()).unwrap();
		}
		file.flush().unwrap();
	}
}

pub(super) struct Logger {
	buffer: RefCell<Buffer>
}

impl Logger {
	pub(super) fn new() -> Self {
		Self {
			buffer: RefCell::new(Buffer::new())
		}
	}

	pub(super) fn create_trace(emulator: &Emulator) {
		let opcode = read8_debug(emulator, emulator.cpu.pc);
		let data = Data {
			pc: emulator.cpu.pc,
			opcode,
			opcode_data: get_opcode_data(opcode, emulator),
			a: emulator.cpu.a,
			x: emulator.cpu.x,
			y: emulator.cpu.y,
			p: emulator.cpu.p,
			s: emulator.cpu.s
		};
		emulator.cpu.logger.buffer.borrow_mut().push(data);
	}
}

fn get_opcode_data(opcode: u8, emulator: &Emulator) -> Vec<u16> {
	let mut opcode_data = Vec::<u16>::new();
	match opcode {
		// NOPs
		0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xea | 0xfa => {},
		0x80 => get_opcode_data_immediate(emulator, &mut opcode_data),
		0x04 | 0x44 | 0x64 | 0x82 | 0x89 | 0xc2 | 0xe2 => get_opcode_data_zero_page(emulator, &mut opcode_data),
		0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 => get_opcode_data_zero_page_x(emulator, &mut opcode_data),
		0x0c => get_opcode_data_absolute(emulator, &mut opcode_data),
		0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => get_opcode_data_absolute_x(emulator, &mut opcode_data),

		// LDA
		0xa9 => get_opcode_data_immediate(emulator, &mut opcode_data),
		0xa5 => get_opcode_data_zero_page(emulator, &mut opcode_data),
		0xb5 => get_opcode_data_zero_page_x(emulator, &mut opcode_data),
		0xad => get_opcode_data_absolute(emulator, &mut opcode_data),
		0xbd => get_opcode_data_absolute_x(emulator, &mut opcode_data),
		0xb9 => get_opcode_data_absolute_y(emulator, &mut opcode_data),
		0xa1 => get_opcode_data_indirect_x(emulator, &mut opcode_data),
		0xb1 => get_opcode_data_indirect_y(emulator, &mut opcode_data),

		// LDX
		0xa2 => get_opcode_data_immediate(emulator, opcode_data),
		0xa6 => get_opcode_data_zero_page(emulator, opcode_data),
		0xb6 => get_opcode_data_zero_page_y(emulator, opcode_data),
		0xae => get_opcode_data_absolute(emulator, opcode_data),
		0xbe => get_opcode_data_absolute_y(emulator, opcode_data),

		// LDY
		0xa0 => get_opcode_data_immediate(emulator, opcode_data),
		0xa4 => get_opcode_data_zero_page(emulator, opcode_data),
		0xb4 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0xac => get_opcode_data_absolute(emulator, opcode_data),
		0xbc => get_opcode_data_absolute_x(emulator, opcode_data),

		// LAX
		0xab => get_opcode_data_immediate(emulator, opcode_data),
		0xa7 => get_opcode_data_zero_page(emulator, opcode_data),
		0xb7 => get_opcode_data_zero_page_y(emulator, opcode_data),
		0xaf => get_opcode_data_absolute(emulator, opcode_data),
		0xbf => get_opcode_data_absolute_y(emulator, opcode_data),
		0xa3 => get_opcode_data_indirect_x(emulator, opcode_data),
		0xb3 => get_opcode_data_indirect_y(emulator, opcode_data),

		// STA
		0x85 => get_opcode_data_zero_page(emulator, opcode_data),
		0x95 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x8d => get_opcode_data_absolute(emulator, opcode_data),
		0x9d => get_opcode_data_absolute_x(emulator, opcode_data),
		0x99 => get_opcode_data_absolute_y(emulator, opcode_data),
		0x81 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x91 => get_opcode_data_indirect_y(emulator, opcode_data),

		// STX
		0x86 => get_opcode_data_zero_page(emulator, opcode_data),
		0x96 => get_opcode_data_zero_page_y(emulator, opcode_data),
		0x8e => get_opcode_data_absolute(emulator, opcode_data),

		// STY
		0x84 => get_opcode_data_zero_page(emulator, opcode_data),
		0x94 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x8c => get_opcode_data_absolute(emulator, opcode_data),

		// SAX
		0x87 => get_opcode_data_zero_page(emulator, opcode_data),
		0x97 => get_opcode_data_zero_page_y(emulator, opcode_data),
		0x8f => get_opcode_data_absolute(emulator, opcode_data),
		0x83 => get_opcode_data_indirect_x(emulator, opcode_data),

		// SXA
		0x9e => get_opcode_data_absolute_y(emulator, opcode_data),

		// SYA
		0x9c => get_opcode_data_absolute_x(emulator, opcode_data),

		// TAX
		0xaa => {},
		
		// TXA
		0x8a => {},

		// TAY
		0xa8 => {},

		// TYA
		0x98 => {},

		// TSX
		0xba => {},

		// TXS
		0x9a => {},

		// AND
		0x29 => get_opcode_data_immediate(emulator, opcode_data),
		0x25 => get_opcode_data_zero_page(emulator, opcode_data),
		0x35 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x2d => get_opcode_data_absolute(emulator, opcode_data),
		0x3d => get_opcode_data_absolute_x(emulator, opcode_data),
		0x39 => get_opcode_data_absolute_y(emulator, opcode_data),
		0x21 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x31 => get_opcode_data_indirect_y(emulator, opcode_data),

		// AAC
		0x0b | 0x2b => get_opcode_data_immediate(emulator, opcode_data),

		// ASR
		0x4b => get_opcode_data_immediate(emulator, opcode_data),

		// ARR
		0x6b => get_opcode_data_immediate(emulator, opcode_data),

		// AXS
		0xcb => get_opcode_data_immediate(emulator, opcode_data),

		// ORA
		0x09 => get_opcode_data_immediate(emulator, opcode_data),
		0x05 => get_opcode_data_zero_page(emulator, opcode_data),
		0x15 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x0d => get_opcode_data_absolute(emulator, opcode_data),
		0x1d => get_opcode_data_absolute_x(emulator, opcode_data),
		0x19 => get_opcode_data_absolute_y(emulator, opcode_data),
		0x01 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x11 => get_opcode_data_indirect_y(emulator, opcode_data),

		// EOR
		0x49 => get_opcode_data_immediate(emulator, opcode_data),
		0x45 => get_opcode_data_zero_page(emulator, opcode_data),
		0x55 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x4d => get_opcode_data_absolute(emulator, opcode_data),
		0x5d => get_opcode_data_absolute_x(emulator, opcode_data),
		0x59 => get_opcode_data_absolute_y(emulator, opcode_data),
		0x41 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x51 => get_opcode_data_indirect_y(emulator, opcode_data),

		// BIT
		0x24 => get_opcode_data_zero_page(emulator, opcode_data),
		0x2c => get_opcode_data_absolute(emulator, opcode_data),

		// LSR
		0x4a => {},
		0x46 => get_opcode_data_zero_page(emulator, opcode_data),
		0x56 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x4e => get_opcode_data_absolute(emulator, opcode_data),
		0x5e => get_opcode_data_absolute_x(emulator, opcode_data),

		// SRE
		0x47 => get_opcode_data_zero_page(emulator, opcode_data),
		0x57 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x4f => get_opcode_data_absolute(emulator, opcode_data),
		0x5f => get_opcode_data_absolute_x(emulator, opcode_data),
		0x5b => get_opcode_data_absolute_y(emulator, opcode_data),
		0x43 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x53 => get_opcode_data_indirect_y(emulator, opcode_data),		

		// ASL
		0x0a => {},
		0x06 => get_opcode_data_zero_page(emulator, opcode_data),
		0x16 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x0e => get_opcode_data_absolute(emulator, opcode_data),
		0x1e => get_opcode_data_absolute_x(emulator, opcode_data),

		// SLO
		0x07 => get_opcode_data_zero_page(emulator, opcode_data),
		0x17 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x0f => get_opcode_data_absolute(emulator, opcode_data),
		0x1f => get_opcode_data_absolute_x(emulator, opcode_data),
		0x1b => get_opcode_data_absolute_y(emulator, opcode_data),
		0x03 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x13 => get_opcode_data_indirect_y(emulator, opcode_data),

		// ROR
		0x6a => {},
		0x66 => get_opcode_data_zero_page(emulator, opcode_data),
		0x76 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x6e => get_opcode_data_absolute(emulator, opcode_data),
		0x7e => get_opcode_data_absolute_x(emulator, opcode_data),

		// RRA
		0x67 => get_opcode_data_zero_page(emulator, opcode_data),
		0x77 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x6f => get_opcode_data_absolute(emulator, opcode_data),
		0x7f => get_opcode_data_absolute_x(emulator, opcode_data),
		0x7b => get_opcode_data_absolute_y(emulator, opcode_data),
		0x63 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x73 => get_opcode_data_indirect_y(emulator, opcode_data),

		// ROL
		0x2a => {},
		0x26 => get_opcode_data_zero_page(emulator, opcode_data),
		0x36 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x2e => get_opcode_data_absolute(emulator, opcode_data),
		0x3e => get_opcode_data_absolute_x(emulator, opcode_data),

		// RLA
		0x27 => get_opcode_data_zero_page(emulator, opcode_data), 
		0x37 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x2f => get_opcode_data_absolute(emulator, opcode_data),
		0x3f => get_opcode_data_absolute_x(emulator, opcode_data),
		0x3b => get_opcode_data_absolute_y(emulator, opcode_data),
		0x23 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x33 => get_opcode_data_indirect_y(emulator, opcode_data),

		// ADC
		0x69 => get_opcode_data_immediate(emulator, opcode_data),
		0x65 => get_opcode_data_zero_page(emulator, opcode_data),
		0x75 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0x6d => get_opcode_data_absolute(emulator, opcode_data),
		0x7d => get_opcode_data_absolute_x(emulator, opcode_data),
		0x79 => get_opcode_data_absolute_y(emulator, opcode_data),
		0x61 => get_opcode_data_indirect_x(emulator, opcode_data),
		0x71 => get_opcode_data_indirect_y(emulator, opcode_data),

		// SBC
		0xe9 | 0xeb => get_opcode_data_immediate(emulator, opcode_data),
		0xe5 => get_opcode_data_zero_page(emulator, opcode_data),
		0xf5 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0xed => get_opcode_data_absolute(emulator, opcode_data),
		0xfd => get_opcode_data_absolute_x(emulator, opcode_data),
		0xf9 => get_opcode_data_absolute_y(emulator, opcode_data),
		0xe1 => get_opcode_data_indirect_x(emulator, opcode_data),
		0xf1 => get_opcode_data_indirect_y(emulator, opcode_data),

		// INX
		0xe8 => {},

		// INY
		0xc8 => {},

		// INC
		0xe6 => get_opcode_data_zero_page(emulator, opcode_data),
		0xf6 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0xee => get_opcode_data_absolute(emulator, opcode_data),
		0xfe => get_opcode_data_absolute_x(emulator, opcode_data),

		// ISB
		0xe7 => get_opcode_data_zero_page(emulator, opcode_data),
		0xf7 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0xef => get_opcode_data_absolute(emulator, opcode_data),
		0xff => get_opcode_data_absolute_x(emulator, opcode_data),
		0xfb => get_opcode_data_absolute_y(emulator, opcode_data),
		0xe3 => get_opcode_data_indirect_x(emulator, opcode_data),
		0xf3 => get_opcode_data_indirect_y(emulator, opcode_data),

		// DEX
		0xca => {},

		// DEY
		0x88 => {},

		// DEC
		0xc6 => get_opcode_data_zero_page(emulator, opcode_data),
		0xd6 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0xce => get_opcode_data_absolute(emulator, opcode_data),
		0xde => get_opcode_data_absolute_x(emulator, opcode_data),

		// DCP
		0xc7 => get_opcode_data_zero_page(emulator, opcode_data),
		0xd7 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0xcf => get_opcode_data_absolute(emulator, opcode_data),
		0xdf => get_opcode_data_absolute_x(emulator, opcode_data),
		0xdb => get_opcode_data_absolute_y(emulator, opcode_data),
		0xc3 => get_opcode_data_indirect_x(emulator, opcode_data),
		0xd3 => get_opcode_data_indirect_y(emulator, opcode_data),

		// CPX
		0xe0 => get_opcode_data_immediate(emulator, opcode_data),
		0xe4 => get_opcode_data_zero_page(emulator, opcode_data),
		0xec => get_opcode_data_absolute(emulator, opcode_data),

		// CPY
		0xc0 => get_opcode_data_immediate(emulator, opcode_data),
		0xc4 => get_opcode_data_zero_page(emulator, opcode_data),
		0xcc => get_opcode_data_absolute(emulator, opcode_data),

		// CMP
		0xc9 => get_opcode_data_immediate(emulator, opcode_data),
		0xc5 => get_opcode_data_zero_page(emulator, opcode_data),
		0xd5 => get_opcode_data_zero_page_x(emulator, opcode_data),
		0xcd => get_opcode_data_absolute(emulator, opcode_data),
		0xdd => get_opcode_data_absolute_x(emulator, opcode_data),
		0xd9 => get_opcode_data_absolute_y(emulator, opcode_data),
		0xc1 => get_opcode_data_indirect_x(emulator, opcode_data),
		0xd1 => get_opcode_data_indirect_y(emulator, opcode_data),

		// PHA
		0x48 => {},

		// PLA
		0x68 => {},

		// PHP
		0x08 => {},

		// PLP
		0x28 => {},

		// CLC
		0x18 => {},

		// SEC
		0x38 => {},

		// CLI
		0x58 => {},

		// SEI
		0x78 => {},

		// CLD
		0xd8 => {},

		// SED
		0xf8 => {},

		// CLV
		0xb8 => {},

		// JMP
		0x4c => get_opcode_data_jump_absolute(emulator, opcode_data),

		// JSR
		0x20 => get_opcode_data_jump_absolute(emulator, opcode_data),

		// JMP (indirect)
		0x6c => {
			let pc = emulator.cpu.pc.wrapping_add(1);
			let address = read16_debug(emulator, pc);
			let low_byte = read8_debug(emulator, address);
			let high_byte = read8_debug(emulator, (address & 0xff00) | (address.wrapping_add(1) & 0x00ff));
			opcode_data.push(address);
			opcode_data.push(low_byte as _);
			opcode_data.push(high_byte as _);
		},

		// BPL
		0x10 => get_opcode_data_jump_relative(emulator, opcode_data),

		// BMI
		0x30 => get_opcode_data_jump_relative(emulator, opcode_data),

		// BVC
		0x50 => get_opcode_data_jump_relative(emulator, opcode_data),

		// BVS
		0x70 => get_opcode_data_jump_relative(emulator, opcode_data),

		// BCC
		0x90 => get_opcode_data_jump_relative(emulator, opcode_data),

		// BCS
		0xb0 => get_opcode_data_jump_relative(emulator, opcode_data),

		// BNE
		0xd0 => get_opcode_data_jump_relative(emulator, opcode_data),

		// BEQ
		0xf0 => get_opcode_data_jump_relative(emulator, opcode_data),

		// BRK
		0x00 => {},

		// RTI
		0x40 => {},

		// RTS
		0x60 => {},

		// KIL
		0x32 => {},

		_ => {}
	};
	opcode_data
}

fn get_opcode_data_immediate(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let operand = read8_debug(emulator, pc);
	opcode_data.push(operand as _);
}

fn get_opcode_data_zero_page(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read8_debug(emulator, pc) as u16;
	let operand = read8_debug(emulator, address);
	opcode_data.push(address);
	opcode_data.push(operand as _);
}

fn get_opcode_data_zero_page_x(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read8_debug(emulator, pc);
	let effective_address = address.wrapping_add(emulator.cpu.x) as u16;
	let operand = read8_debug(emulator, effective_address);
	opcode_data.push(address as _);
	opcode_data.push(effective_address);
	opcode_data.push(operand as _);
}

fn get_opcode_data_zero_page_y(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read8_debug(emulator, pc);
	let effective_address = address.wrapping_add(emulator.cpu.y) as u16;
	let operand = read8_debug(emulator, effective_address);
	opcode_data.push(address as _);
	opcode_data.push(effective_address);
	opcode_data.push(operand as _);
}

fn get_opcode_data_absolute(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read16_debug(emulator, pc);
	let operand = read8_debug(emulator, address);
	opcode_data.push(address);
	opcode_data.push(operand as _);
}

fn get_opcode_data_absolute_x(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read16_debug(emulator, pc);
	let effective_address = address.wrapping_add(emulator.cpu.x as _);
	let operand = read8_debug(emulator, effective_address);
	opcode_data.push(address);
	opcode_data.push(effective_address);
	opcode_data.push(operand as _);
}

fn get_opcode_data_absolute_y(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read16_debug(emulator, pc);
	let effective_address = address.wrapping_add(emulator.cpu.y as _);
	let operand = read8_debug(emulator, effective_address);
	opcode_data.push(address);
	opcode_data.push(effective_address);
	opcode_data.push(operand as _);
}

fn get_opcode_data_indirect_x(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let immediate = read8_debug(emulator, pc);
	let address = immediate.wrapping_add(emulator.cpu.x);
	let effective_address = read16_zeropage_debug(emulator, address);
	let operand = read8_debug(emulator, effective_address);
	opcode_data.push(immediate as _);
	opcode_data.push(address as _);
	opcode_data.push(effective_address);
	opcode_data.push(operand as _);
}

fn get_opcode_data_indirect_y(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let immediate = read8_debug(emulator, pc);
	let address = read16_zeropage_debug(emulator, immediate);
	let effective_address = address.wrapping_add(emulator.cpu.y as _);
	let operand = read8_debug(emulator, effective_address);
	opcode_data.push(immediate as _);
	opcode_data.push(address);
	opcode_data.push(effective_address);
	opcode_data.push(operand as _);
}

fn get_opcode_data_jump_absolute(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read16_debug(emulator, pc);
	opcode_data.push(address);
}

fn get_opcode_data_jump_relative(emulator: &Emulator, opcode_data: &mut Vec<u16>) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let offset = read8_debug(emulator, pc);
	opcode_data.push(offset as i8 as _);
}

fn format_instruction(data: &Data) -> String {
	match data.opcode {
		0xea => format("NOP"),
		0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => format("*NOP"),
		0x80 => format_immediate(data, "*NOP"),
		0x04 | 0x44 | 0x64 | 0x82 | 0x89 | 0xc2 | 0xe2 => format_zero_page(data, "*NOP"),
		0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 => format_zero_page_x(data, "*NOP"),
		0x0c => format_absolute(data, "*NOP"),
		0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => format_absolute_x(data, "*NOP"),
			
		0xa9 => format_immediate(data, "LDA"),
		0xa5 => format_zero_page(data, "LDA"),
		0xb5 => format_zero_page_x(data, "LDA"),
		0xad => format_absolute(data, "LDA"),
		0xbd => format_absolute_x(data, "LDA"),
		0xb9 => format_absolute_y(data, "LDA"),
		0xa1 => format_indirect_x(data, "LDA"),
		0xb1 => format_indirect_y(data, "LDA"),

		0xa2 => format_immediate(data, "LDX"),
		0xa6 => format_zero_page(data, "LDX"),
		0xb6 => format_zero_page_y(data, "LDX"),
		0xae => format_absolute(data, "LDX"),
		0xbe => format_absolute_y(data, "LDX"),

		0xa0 => format_immediate(data, "LDY"),
		0xa4 => format_zero_page(data, "LDY"),
		0xb4 => format_zero_page_x(data, "LDY"),
		0xac => format_absolute(data, "LDY"),
		0xbc => format_absolute_x(data, "LDY"),

		0xab => format_immediate(data, "LAX"),
		0xa7 => format_zero_page(data, "LAX"),
		0xb7 => format_zero_page_y(data, "LAX"),
		0xaf => format_absolute(data, "LAX"),
		0xbf => format_absolute_y(data, "LAX"),
		0xa3 => format_indirect_x(data, "LAX"),
		0xb3 => format_indirect_y(data, "LAX"),

		0x85 => format_zero_page(data, "STA"),
		0x95 => format_zero_page_x(data, "STA"),
		0x8d => format_absolute(data, "STA"),
		0x9d => format_absolute_x(data, "STA"),
		0x99 => format_absolute_y(data, "STA"),
		0x81 => format_indirect_x(data, "STA"),
		0x91 => format_indirect_y(data, "STA"),

		0x86 => format_zero_page(data, "STX"),
		0x96 => format_zero_page_y(data, "STX"),
		0x8e => format_absolute(data, "STX"),

		0x84 => format_zero_page(data, "STY"),
		0x94 => format_zero_page_x(data, "STY"),
		0x8c => format_absolute(data, "STY"),

		0x87 => format_zero_page(data, "SAX"),
		0x97 => format_zero_page_y(data, "SAX"),
		0x8f => format_absolute(data, "SAX"),
		0x83 => format_indirect_x(data, "SAX"),

		0x9e => format_absolute_y(data, "SXA"),
		0x9c => format_absolute_x(data, "SYA"),

		0xaa => format("TAX"),
		0x8a => format("TXA"),
		0xa8 => format("TAY"),
		0x98 => format("TYA"),
		0xba => format("TSX"),
		0x9a => format("TXS"),

		0x29 => format_immediate(data, "AND"),
		0x25 => format_zero_page(data, "AND"),
		0x35 => format_zero_page_x(data, "AND"),
		0x2d => format_absolute(data, "AND"),
		0x3d => format_absolute_x(data, "AND"),
		0x39 => format_absolute_y(data, "AND"),
		0x21 => format_indirect_x(data, "AND"),
		0x31 => format_indirect_y(data, "AND"),

		0x0b | 0x2b => format_immediate(data, "AAC"),
		0x4b => format_immediate(data, "ASR"),
		0x6b => format_immediate(data, "ARR"),
		0xcb => format_immediate(data, "AXS"),

		0x09 => format_immediate(data, "ORA"),
		0x05 => format_zero_page(data, "ORA"),
		0x15 => format_zero_page_x(data, "ORA"),
		0x0d => format_absolute(data, "ORA"),
		0x1d => format_absolute_x(data, "ORA"),
		0x19 => format_absolute_y(data, "ORA"),
		0x01 => format_indirect_x(data, "ORA"),
		0x11 => format_indirect_y(data, "ORA"),

		0x49 => format_immediate(data, "EOR"),
		0x45 => format_zero_page(data, "EOR"),
		0x55 => format_zero_page_x(data, "EOR"),
		0x4d => format_absolute(data, "EOR"),
		0x5d => format_absolute_x(data, "EOR"),
		0x59 => format_absolute_y(data, "EOR"),
		0x41 => format_indirect_x(data, "EOR"),
		0x51 => format_indirect_y(data, "EOR"),

		0x24 => format_zero_page(data, "BIT"),
		0x2c => format_absolute(data, "BIT"),

		0x4a => format_a("LSR"),
		0x46 => format_zero_page(data, "LSR"),
		0x56 => format_zero_page_x(data, "LSR"),
		0x4e => format_absolute(data, "LSR"),
		0x5e => format_absolute_x(data, "LSR"),

		0x47 => format_zero_page(data, "*SRE"),
		0x57 => format_zero_page_x(data, "*SRE"),
		0x4f => format_absolute(data, "*SRE"),
		0x5f => format_absolute_x(data, "*SRE"),
		0x5b => format_absolute_y(data, "*SRE"),
		0x43 => format_indirect_x(data, "*SRE"),
		0x53 => format_indirect_y(data, "*SRE"),		

		0x0a => format_a("ASL"),
		0x06 => format_zero_page(data, "ASL"),
		0x16 => format_zero_page_x(data, "ASL"),
		0x0e => format_absolute(data, "ASL"),
		0x1e => format_absolute_x(data, "ASL"),

		0x07 => format_zero_page(data, "*SLO"),
		0x17 => format_zero_page_x(data, "*SLO"),
		0x0f => format_absolute(data, "*SLO"),
		0x1f => format_absolute_x(data, "*SLO"),
		0x1b => format_absolute_y(data, "*SLO"),
		0x03 => format_indirect_x(data, "*SLO"),
		0x13 => format_indirect_y(data, "*SLO"),

		0x6a => format_a("ROR"),
		0x66 => format_zero_page(data, "ROR"),
		0x76 => format_zero_page_x(data, "ROR"),
		0x6e => format_absolute(data, "ROR"),
		0x7e => format_absolute_x(data, "ROR"),

		0x67 => format_zero_page(data, "*RRA"),
		0x77 => format_zero_page_x(data, "*RRA"),
		0x6f => format_absolute(data, "*RRA"),
		0x7f => format_absolute_x(data, "*RRA"),
		0x7b => format_absolute_y(data, "*RRA"),
		0x63 => format_indirect_x(data, "*RRA"),
		0x73 => format_indirect_y(data, "*RRA"),

		0x2a => format_a("ROL"),
		0x26 => format_zero_page(data, "ROL"),
		0x36 => format_zero_page_x(data, "ROL"),
		0x2e => format_absolute(data, "ROL"),
		0x3e => format_absolute_x(data, "ROL"),

		0x27 => format_zero_page(data, "*RLA"), 
		0x37 => format_zero_page_x(data, "*RLA"),
		0x2f => format_absolute(data, "*RLA"),
		0x3f => format_absolute_x(data, "*RLA"),
		0x3b => format_absolute_y(data, "*RLA"),
		0x23 => format_indirect_x(data, "*RLA"),
		0x33 => format_indirect_y(data, "*RLA"),

		0x69 => format_immediate(data, "ADC"),
		0x65 => format_zero_page(data, "ADC"),
		0x75 => format_zero_page_x(data, "ADC"),
		0x6d => format_absolute(data, "ADC"),
		0x7d => format_absolute_x(data, "ADC"),
		0x79 => format_absolute_y(data, "ADC"),
		0x61 => format_indirect_x(data, "ADC"),
		0x71 => format_indirect_y(data, "ADC"),

		0xe9 => format_immediate(data, "SBC"),
		0xe5 => format_zero_page(data, "SBC"),
		0xf5 => format_zero_page_x(data, "SBC"),
		0xed => format_absolute(data, "SBC"),
		0xfd => format_absolute_x(data, "SBC"),
		0xf9 => format_absolute_y(data, "SBC"),
		0xe1 => format_indirect_x(data, "SBC"),
		0xf1 => format_indirect_y(data, "SBC"),
			
		0xeb => format_immediate(data, "*SBC"),

		0xe8 => format("INX"),
		0xc8 => format("INY"),

		0xe6 => format_zero_page(data, "INC"),
		0xf6 => format_zero_page_x(data, "INC"),
		0xee => format_absolute(data, "INC"),
		0xfe => format_absolute_x(data, "INC"),

		0xe7 => format_zero_page(data, "*ISB"),
		0xf7 => format_zero_page_x(data, "*ISB"),
		0xef => format_absolute(data, "*ISB"),
		0xff => format_absolute_x(data, "*ISB"),
		0xfb => format_absolute_y(data, "*ISB"),
		0xe3 => format_indirect_x(data, "*ISB"),
		0xf3 => format_indirect_y(data, "*ISB"),

		0xca => format("DEX"),
		0x88 => format("DEY"),

		0xc6 => format_zero_page(data, "DEC"),
		0xd6 => format_zero_page_x(data, "DEC"),
		0xce => format_absolute(data, "DEC"),
		0xde => format_absolute_x(data, "DEC"),

		0xc7 => format_zero_page(data, "*DCP"),
		0xd7 => format_zero_page_x(data, "*DCP"),
		0xcf => format_absolute(data, "*DCP"),
		0xdf => format_absolute_x(data, "*DCP"),
		0xdb => format_absolute_y(data, "*DCP"),
		0xc3 => format_indirect_x(data, "*DCP"),
		0xd3 => format_indirect_y(data, "*DCP"),

		0xe0 => format_immediate(data, "CPX"),
		0xe4 => format_zero_page(data, "CPX"),
		0xec => format_absolute(data, "CPX"),

		0xc0 => format_immediate(data, "CPY"),
		0xc4 => format_zero_page(data, "CPY"),
		0xcc => format_absolute(data, "CPY"),

		0xc9 => format_immediate(data, "CMP"),
		0xc5 => format_zero_page(data, "CMP"),
		0xd5 => format_zero_page_x(data, "CMP"),
		0xcd => format_absolute(data, "CMP"),
		0xdd => format_absolute_x(data, "CMP"),
		0xd9 => format_absolute_y(data, "CMP"),
		0xc1 => format_indirect_x(data, "CMP"),
		0xd1 => format_indirect_y(data, "CMP"),

		0x48 => format("PHA"),
		0x68 => format("PLA"),
		0x08 => format("PHP"),
		0x28 => format("PLP"),
		0x18 => format("CLC"),
		0x38 => format("SEC"),
		0x58 => format("CLI"),
		0x78 => format("SEI"),
		0xd8 => format("CLD"),
		0xf8 => format("SED"),
		0xb8 => format("CLV"),

		0x4c => format_jump_absolute(data, "JMP"),
		0x20 => format_jump_absolute(data, "JSR"),

		// JMP (indirect)
		0x6c => {
			let address = data.opcode_data[0];
			let low_byte = data.opcode_data[1];
			let high_byte = data.opcode_data[2];
			let effective_address = (high_byte << 8) | low_byte;
			format!("{:02X} {:02X}  JMP (${:04X}) = {:04X}", low_byte, high_byte, address, effective_address)
		},

		0x10 => format_jump_relative(data, "BPL"),
		0x30 => format_jump_relative(data, "BMI"),
		0x50 => format_jump_relative(data, "BVC"),
		0x70 => format_jump_relative(data, "BVS"),
		0x90 => format_jump_relative(data, "BCC"),
		0xb0 => format_jump_relative(data, "BCS"),
		0xd0 => format_jump_relative(data, "BNE"),
		0xf0 => format_jump_relative(data, "BEQ"),

		0x00 => format("BRK"),

		0x40 => format("RTI"),
		0x60 => format("RTS"),

		0x32 => format("KIL"),

		_ => "# UNKNOWN OPCODE #".to_string()
	}
}

fn format(mnemonic: &str) -> String {
	format!("{:>10}", mnemonic)
}

fn format_a(mnemonic: &str) -> String {
	format!("{:>10} A", mnemonic)
}

fn format_immediate(data: &Data, mnemonic: &str) -> String {
	let operand = data.opcode_data[0];
	format!("{:02X}{:>8} #${:02X}", operand, mnemonic, operand)
}

fn format_zero_page(data: &Data, mnemonic: &str) -> String {
	let address = data.opcode_data[0];
	let operand = data.opcode_data[1];
	format!("{:02X}{:>8} ${:02X} = {:02X}", address, mnemonic, address, operand)
}

fn format_zero_page_x(data: &Data, mnemonic: &str) -> String {
	let address = data.opcode_data[0];
	let effective_address = data.opcode_data[1];
	let operand = data.opcode_data[2];
	format!("{:02X}{:>8} ${:02X},X @ {:02X} = {:02X}", address, mnemonic, address, effective_address, operand)
}

fn format_zero_page_y(data: &Data, mnemonic: &str) -> String {
	let address = data.opcode_data[0];
	let effective_address = data.opcode_data[1];
	let operand = data.opcode_data[2];
	format!("{:02X}{:>8} ${:02X},Y @ {:02X} = {:02X}", address, mnemonic, address, effective_address, operand)
}

fn format_absolute(data: &Data, mnemonic: &str) -> String {
	let address = data.opcode_data[0];
	let low_byte = address & 0xff;
	let high_byte = address >> 8;
	let operand = data.opcode_data[1];
	format!("{:02X} {:02X}{:>5} ${:04X} = {:02X}", low_byte, high_byte, mnemonic, address, operand)
}

fn format_absolute_x(data: &Data, mnemonic: &str) -> String {
	let address = data.opcode_data[0];
	let low_byte = address & 0xff;
	let high_byte = address >> 8;
	let effective_address = data.opcode_data[1];
	let operand = data.opcode_data[2];
	format!("{:02X} {:02X}{:>5} ${:04X},X @ {:04X} = {:02X}", low_byte, high_byte, mnemonic, address, effective_address, operand)
}

fn format_absolute_y(data: &Data, mnemonic: &str) -> String {
	let address = data.opcode_data[0];
	let low_byte = address & 0xff;
	let high_byte = address >> 8;
	let effective_address = data.opcode_data[1];
	let operand = data.opcode_data[2];
	format!("{:02X} {:02X}{:>5} ${:04X},Y @ {:04X} = {:02X}", low_byte, high_byte, mnemonic, address, effective_address, operand)
}

fn format_indirect_x(data: &Data, mnemonic: &str) -> String {
	let immediate = data.opcode_data[0];
	let address = data.opcode_data[1];
	let effective_address = data.opcode_data[2];
	let operand = data.opcode_data[3];
	format!("{:02X}{:>8} (${:02X},X) @ {:02X} = {:04X} = {:02X}", immediate, mnemonic, immediate, address, effective_address, operand)
}

fn format_indirect_y(data: &Data, mnemonic: &str) -> String {
	let immediate = data.opcode_data[0];
	let address = data.opcode_data[1];
	let effective_address = data.opcode_data[2];
	let operand = data.opcode_data[3];
	format!("{:02X}{:>8} (${:02X}),Y = {:04X} @ {:04X} = {:02X}", immediate, mnemonic, immediate, address, effective_address, operand)
}

fn format_jump_absolute(data: &Data, mnemonic: &str) -> String {
	let address = data.opcode_data[0];
	let low_byte = address & 0xff;
	let high_byte = address >> 8;
	format!("{:02X} {:02X}{:>5} ${:04X}", low_byte, high_byte, mnemonic, address)
}

fn format_jump_relative(data: &Data, mnemonic: &str) -> String {
	let offset = data.opcode_data[0];
	let address = data.pc.wrapping_add(offset).wrapping_add(2);
	format!("{:02X}{:>8} ${:04X}", offset, mnemonic, address)
}
