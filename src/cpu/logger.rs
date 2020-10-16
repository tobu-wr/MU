use std::io::prelude::Write;
use std::fs::File;
use std::cell::RefCell;

use emulator::*;
use super::memory::*;

const MAX_LOGS: usize = 44_000;

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
			data: Vec::with_capacity(MAX_LOGS),
			index: 0
		}
	}

	fn push(&mut self, data: Data) {
		if self.data.len() < MAX_LOGS {
			self.data.push(data);
		} else {
			self.data[self.index] = data;
			self.index = (self.index + 1) % MAX_LOGS;
		}
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		let mut file = File::create("trace.log").unwrap();
		for i in 0..self.data.len() {
			let index = (self.index + i) % MAX_LOGS;
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

	pub(super) fn get_trace_function(opcode: u8) -> fn(&Emulator) {
		match opcode {
			// NOPs
			0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xea | 0xfa => trace_function,
			0x80 => trace_function_immediate,
			0x04 | 0x44 | 0x64 | 0x82 | 0x89 | 0xc2 | 0xe2 => trace_function_zero_page,
			0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 => trace_function_zero_page_x,
			0x0c => trace_function_absolute,
			0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => trace_function_absolute_x,
	
			// LDA
			0xa9 => trace_function_immediate,
			0xa5 => trace_function_zero_page,
			0xb5 => trace_function_zero_page_x,
			0xad => trace_function_absolute,
			0xbd => trace_function_absolute_x,
			0xb9 => trace_function_absolute_y,
			0xa1 => trace_function_indirect_x,
			0xb1 => trace_function_indirect_y,
	
			// LDX
			0xa2 => trace_function_immediate,
			0xa6 => trace_function_zero_page,
			0xb6 => trace_function_zero_page_y,
			0xae => trace_function_absolute,
			0xbe => trace_function_absolute_y,
	
			// LDY
			0xa0 => trace_function_immediate,
			0xa4 => trace_function_zero_page,
			0xb4 => trace_function_zero_page_x,
			0xac => trace_function_absolute,
			0xbc => trace_function_absolute_x,
	
			// LAX
			0xab => trace_function_immediate,
			0xa7 => trace_function_zero_page,
			0xb7 => trace_function_zero_page_y,
			0xaf => trace_function_absolute,
			0xbf => trace_function_absolute_y,
			0xa3 => trace_function_indirect_x,
			0xb3 => trace_function_indirect_y,
	
			// STA
			0x85 => trace_function_zero_page,
			0x95 => trace_function_zero_page_x,
			0x8d => trace_function_absolute,
			0x9d => trace_function_absolute_x,
			0x99 => trace_function_absolute_y,
			0x81 => trace_function_indirect_x,
			0x91 => trace_function_indirect_y,
	
			// STX
			0x86 => trace_function_zero_page,
			0x96 => trace_function_zero_page_y,
			0x8e => trace_function_absolute,
	
			// STY
			0x84 => trace_function_zero_page,
			0x94 => trace_function_zero_page_x,
			0x8c => trace_function_absolute,
	
			// SAX
			0x87 => trace_function_zero_page,
			0x97 => trace_function_zero_page_y,
			0x8f => trace_function_absolute,
			0x83 => trace_function_indirect_x,
	
			// SXA
			0x9e => trace_function_absolute_y,
	
			// SYA
			0x9c => trace_function_absolute_x,
	
			// TAX
			0xaa => trace_function,
			
			// TXA
			0x8a => trace_function,
	
			// TAY
			0xa8 => trace_function,
	
			// TYA
			0x98 => trace_function,
	
			// TSX
			0xba => trace_function,
	
			// TXS
			0x9a => trace_function,
	
			// AND
			0x29 => trace_function_immediate,
			0x25 => trace_function_zero_page,
			0x35 => trace_function_zero_page_x,
			0x2d => trace_function_absolute,
			0x3d => trace_function_absolute_x,
			0x39 => trace_function_absolute_y,
			0x21 => trace_function_indirect_x,
			0x31 => trace_function_indirect_y,
	
			// AAC
			0x0b | 0x2b => trace_function_immediate,
	
			// ASR
			0x4b => trace_function_immediate,
	
			// ARR
			0x6b => trace_function_immediate,
	
			// AXS
			0xcb => trace_function_immediate,
	
			// ORA
			0x09 => trace_function_immediate,
			0x05 => trace_function_zero_page,
			0x15 => trace_function_zero_page_x,
			0x0d => trace_function_absolute,
			0x1d => trace_function_absolute_x,
			0x19 => trace_function_absolute_y,
			0x01 => trace_function_indirect_x,
			0x11 => trace_function_indirect_y,
	
			// EOR
			0x49 => trace_function_immediate,
			0x45 => trace_function_zero_page,
			0x55 => trace_function_zero_page_x,
			0x4d => trace_function_absolute,
			0x5d => trace_function_absolute_x,
			0x59 => trace_function_absolute_y,
			0x41 => trace_function_indirect_x,
			0x51 => trace_function_indirect_y,
	
			// BIT
			0x24 => trace_function_zero_page,
			0x2c => trace_function_absolute,
	
			// LSR
			0x4a => trace_function,
			0x46 => trace_function_zero_page,
			0x56 => trace_function_zero_page_x,
			0x4e => trace_function_absolute,
			0x5e => trace_function_absolute_x,
	
			// SRE
			0x47 => trace_function_zero_page,
			0x57 => trace_function_zero_page_x,
			0x4f => trace_function_absolute,
			0x5f => trace_function_absolute_x,
			0x5b => trace_function_absolute_y,
			0x43 => trace_function_indirect_x,
			0x53 => trace_function_indirect_y,		
	
			// ASL
			0x0a => trace_function,
			0x06 => trace_function_zero_page,
			0x16 => trace_function_zero_page_x,
			0x0e => trace_function_absolute,
			0x1e => trace_function_absolute_x,
	
			// SLO
			0x07 => trace_function_zero_page,
			0x17 => trace_function_zero_page_x,
			0x0f => trace_function_absolute,
			0x1f => trace_function_absolute_x,
			0x1b => trace_function_absolute_y,
			0x03 => trace_function_indirect_x,
			0x13 => trace_function_indirect_y,
	
			// ROR
			0x6a => trace_function,
			0x66 => trace_function_zero_page,
			0x76 => trace_function_zero_page_x,
			0x6e => trace_function_absolute,
			0x7e => trace_function_absolute_x,
	
			// RRA
			0x67 => trace_function_zero_page,
			0x77 => trace_function_zero_page_x,
			0x6f => trace_function_absolute,
			0x7f => trace_function_absolute_x,
			0x7b => trace_function_absolute_y,
			0x63 => trace_function_indirect_x,
			0x73 => trace_function_indirect_y,
	
			// ROL
			0x2a => trace_function,
			0x26 => trace_function_zero_page,
			0x36 => trace_function_zero_page_x,
			0x2e => trace_function_absolute,
			0x3e => trace_function_absolute_x,
	
			// RLA
			0x27 => trace_function_zero_page, 
			0x37 => trace_function_zero_page_x,
			0x2f => trace_function_absolute,
			0x3f => trace_function_absolute_x,
			0x3b => trace_function_absolute_y,
			0x23 => trace_function_indirect_x,
			0x33 => trace_function_indirect_y,
	
			// ADC
			0x69 => trace_function_immediate,
			0x65 => trace_function_zero_page,
			0x75 => trace_function_zero_page_x,
			0x6d => trace_function_absolute,
			0x7d => trace_function_absolute_x,
			0x79 => trace_function_absolute_y,
			0x61 => trace_function_indirect_x,
			0x71 => trace_function_indirect_y,
	
			// SBC
			0xe9 | 0xeb => trace_function_immediate,
			0xe5 => trace_function_zero_page,
			0xf5 => trace_function_zero_page_x,
			0xed => trace_function_absolute,
			0xfd => trace_function_absolute_x,
			0xf9 => trace_function_absolute_y,
			0xe1 => trace_function_indirect_x,
			0xf1 => trace_function_indirect_y,
	
			// INX
			0xe8 => trace_function,
	
			// INY
			0xc8 => trace_function,
	
			// INC
			0xe6 => trace_function_zero_page,
			0xf6 => trace_function_zero_page_x,
			0xee => trace_function_absolute,
			0xfe => trace_function_absolute_x,
	
			// ISB
			0xe7 => trace_function_zero_page,
			0xf7 => trace_function_zero_page_x,
			0xef => trace_function_absolute,
			0xff => trace_function_absolute_x,
			0xfb => trace_function_absolute_y,
			0xe3 => trace_function_indirect_x,
			0xf3 => trace_function_indirect_y,
	
			// DEX
			0xca => trace_function,
	
			// DEY
			0x88 => trace_function,
	
			// DEC
			0xc6 => trace_function_zero_page,
			0xd6 => trace_function_zero_page_x,
			0xce => trace_function_absolute,
			0xde => trace_function_absolute_x,
	
			// DCP
			0xc7 => trace_function_zero_page,
			0xd7 => trace_function_zero_page_x,
			0xcf => trace_function_absolute,
			0xdf => trace_function_absolute_x,
			0xdb => trace_function_absolute_y,
			0xc3 => trace_function_indirect_x,
			0xd3 => trace_function_indirect_y,
	
			// CPX
			0xe0 => trace_function_immediate,
			0xe4 => trace_function_zero_page,
			0xec => trace_function_absolute,
	
			// CPY
			0xc0 => trace_function_immediate,
			0xc4 => trace_function_zero_page,
			0xcc => trace_function_absolute,
	
			// CMP
			0xc9 => trace_function_immediate,
			0xc5 => trace_function_zero_page,
			0xd5 => trace_function_zero_page_x,
			0xcd => trace_function_absolute,
			0xdd => trace_function_absolute_x,
			0xd9 => trace_function_absolute_y,
			0xc1 => trace_function_indirect_x,
			0xd1 => trace_function_indirect_y,
	
			// PHA
			0x48 => trace_function,
	
			// PLA
			0x68 => trace_function,
	
			// PHP
			0x08 => trace_function,
	
			// PLP
			0x28 => trace_function,
	
			// CLC
			0x18 => trace_function,
	
			// SEC
			0x38 => trace_function,
	
			// CLI
			0x58 => trace_function,
	
			// SEI
			0x78 => trace_function,
	
			// CLD
			0xd8 => trace_function,
	
			// SED
			0xf8 => trace_function,
	
			// CLV
			0xb8 => trace_function,
	
			// JMP
			0x4c => trace_function_jump_absolute,
	
			// JSR
			0x20 => trace_function_jump_absolute,
	
			// JMP (indirect)
			0x6c => {
				let pc = emulator.cpu.pc.wrapping_add(1);
				let address = read16_debug(emulator, pc);
				let low_byte = read8_debug(emulator, address);
				let high_byte = read8_debug(emulator, (address & 0xff00) | (address.wrapping_add(1) & 0x00ff));
				opcode_data.push(address);
				opcode_data.push(low_byte as _);
				opcode_data.push(high_byte as _);


				let data = Data {
					pc: emulator.cpu.pc,
					opcode: read8_debug(emulator, emulator.cpu.pc),
					opcode_data,
					a: emulator.cpu.a,
					x: emulator.cpu.x,
					y: emulator.cpu.y,
					p: emulator.cpu.p,
					s: emulator.cpu.s
				};
				emulator.cpu.logger.buffer.borrow_mut().push(data);
			},
	
			// BPL
			0x10 => trace_function_jump_relative,
	
			// BMI
			0x30 => trace_function_jump_relative,
	
			// BVC
			0x50 => trace_function_jump_relative,
	
			// BVS
			0x70 => trace_function_jump_relative,
	
			// BCC
			0x90 => trace_function_jump_relative,
	
			// BCS
			0xb0 => trace_function_jump_relative,
	
			// BNE
			0xd0 => trace_function_jump_relative,
	
			// BEQ
			0xf0 => trace_function_jump_relative,
	
			// BRK
			0x00 => trace_function,
	
			// RTI
			0x40 => trace_function,
	
			// RTS
			0x60 => trace_function,
	
			// KIL
			0x32 => trace_function,
	
			_ => |emulator| {
				let opcode = read8_debug(emulator, emulator.cpu.pc);
				warn!("Unknown opcode {:02X} at {:04X}", opcode, emulator.cpu.pc);
				trace_function(emulator);
			}
		}
	}
}

fn trace_function(emulator: &Emulator) {
	let opcode = read8_debug(emulator, emulator.cpu.pc);
	let data = Data {
		pc: emulator.cpu.pc,
		opcode,
		opcode_data: Vec::new(),
		a: emulator.cpu.a,
		x: emulator.cpu.x,
		y: emulator.cpu.y,
		p: emulator.cpu.p,
		s: emulator.cpu.s
	};
	emulator.cpu.logger.buffer.borrow_mut().push(data);
}

fn trace_function_immediate(emulator: &Emulator) {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let operand = read8_debug(emulator, pc);
	let mut opcode_data = Vec::<u16>::new();
	opcode_data.push(operand as _);
	
	let data = Data {
		pc: emulator.cpu.pc,
		opcode: read8_debug(emulator, emulator.cpu.pc),
		opcode_data,
		a: emulator.cpu.a,
		x: emulator.cpu.x,
		y: emulator.cpu.y,
		p: emulator.cpu.p,
		s: emulator.cpu.s
	};
	emulator.cpu.logger.buffer.borrow_mut().push(data);
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
	opcode_data.push(offset as _);
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

		0xab => format_immediate(data, "*LAX"),
		0xa7 => format_zero_page(data, "*LAX"),
		0xb7 => format_zero_page_y(data, "*LAX"),
		0xaf => format_absolute(data, "*LAX"),
		0xbf => format_absolute_y(data, "*LAX"),
		0xa3 => format_indirect_x(data, "*LAX"),
		0xb3 => format_indirect_y(data, "*LAX"),

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

		0x87 => format_zero_page(data, "*SAX"),
		0x97 => format_zero_page_y(data, "*SAX"),
		0x8f => format_absolute(data, "*SAX"),
		0x83 => format_indirect_x(data, "*SAX"),

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
			let address_low = address & 0xff;
			let address_high = address >> 8;
			let effective_address_low = data.opcode_data[1];
			let effective_address_high = data.opcode_data[2];
			let effective_address = (effective_address_high << 8) | effective_address_low;
			format!("{:02X} {:02X}  JMP (${:04X}) = {:04X}", address_low, address_high, address, effective_address)
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
	let address = data.pc.wrapping_add(offset as i8 as _).wrapping_add(2);
	format!("{:02X}{:>8} ${:04X}", offset, mnemonic, address)
}
