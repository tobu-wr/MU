use std::io::prelude::Write;
use std::fs::File;
use std::cell::RefCell;

use emulator::*;
use super::memory::*;

const BUFFER_CAPACITY: usize = 44_000;

struct Buffer {
	logs: Vec::<String>,
	index: usize
}

impl Buffer {
	fn new() -> Self {
		Self {
			logs: Vec::<String>::with_capacity(BUFFER_CAPACITY),
			index: 0
		}
	}

	fn push(&mut self, log: String) {
		if self.logs.len() < BUFFER_CAPACITY {
			self.logs.push(log);
		} else {
			self.logs[self.index] = log;
			self.increment_index();
		}
	}

	fn increment_index(&mut self) {
		if self.index == (BUFFER_CAPACITY - 1) {
			self.index = 0;
		} else {
			self.index += 1;
		}
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		let mut file = File::create("trace.log").unwrap();
		for _ in 0..self.logs.len() {
			file.write_all(self.logs[self.index].as_bytes()).unwrap();
			self.increment_index();
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
		let instruction_string = match opcode {
			0xea => format("NOP"),
			0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => format("*NOP"),
			0x80 => format_immediate(emulator, "*NOP"),
			0x04 | 0x44 | 0x64 | 0x82 | 0x89 | 0xc2 | 0xe2 => format_zero_page(emulator, "*NOP"),
			0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 => format_zero_page_x(emulator, "*NOP"),
			0x0c => format_absolute(emulator, "*NOP"),
			0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => format_absolute_x(emulator, "*NOP"),
			
			0xa9 => format_immediate(emulator, "LDA"),
			0xa5 => format_zero_page(emulator, "LDA"),
			0xb5 => format_zero_page_x(emulator, "LDA"),
			0xad => format_absolute(emulator, "LDA"),
			0xbd => format_absolute_x(emulator, "LDA"),
			0xb9 => format_absolute_y(emulator, "LDA"),
			0xa1 => format_indirect_x(emulator, "LDA"),
			0xb1 => format_indirect_y(emulator, "LDA"),

			0xa2 => format_immediate(emulator, "LDX"),
			0xa6 => format_zero_page(emulator, "LDX"),
			0xb6 => format_zero_page_y(emulator, "LDX"),
			0xae => format_absolute(emulator, "LDX"),
			0xbe => format_absolute_y(emulator, "LDX"),

			0xa0 => format_immediate(emulator, "LDY"),
			0xa4 => format_zero_page(emulator, "LDY"),
			0xb4 => format_zero_page_x(emulator, "LDY"),
			0xac => format_absolute(emulator, "LDY"),
			0xbc => format_absolute_x(emulator, "LDY"),

			0xab => format_immediate(emulator, "LAX"),
			0xa7 => format_zero_page(emulator, "LAX"),
			0xb7 => format_zero_page_y(emulator, "LAX"),
			0xaf => format_absolute(emulator, "LAX"),
			0xbf => format_absolute_y(emulator, "LAX"),
			0xa3 => format_indirect_x(emulator, "LAX"),
			0xb3 => format_indirect_y(emulator, "LAX"),

			0x85 => format_zero_page(emulator, "STA"),
			0x95 => format_zero_page_x(emulator, "STA"),
			0x8d => format_absolute(emulator, "STA"),
			0x9d => format_absolute_x(emulator, "STA"),
			0x99 => format_absolute_y(emulator, "STA"),
			0x81 => format_indirect_x(emulator, "STA"),
			0x91 => format_indirect_y(emulator, "STA"),

			0x86 => format_zero_page(emulator, "STX"),
			0x96 => format_zero_page_y(emulator, "STX"),
			0x8e => format_absolute(emulator, "STX"),

			0x84 => format_zero_page(emulator, "STY"),
			0x94 => format_zero_page_x(emulator, "STY"),
			0x8c => format_absolute(emulator, "STY"),

			0x87 => format_zero_page(emulator, "SAX"),
			0x97 => format_zero_page_y(emulator, "SAX"),
			0x8f => format_absolute(emulator, "SAX"),
			0x83 => format_indirect_x(emulator, "SAX"),

			0x9e => format_absolute_y(emulator, "SXA"),
			0x9c => format_absolute_x(emulator, "SYA"),

			0xaa => format("TAX"),
			0x8a => format("TXA"),
			0xa8 => format("TAY"),
			0x98 => format("TYA"),
			0xba => format("TSX"),
			0x9a => format("TXS"),

			0x29 => format_immediate(emulator, "AND"),
			0x25 => format_zero_page(emulator, "AND"),
			0x35 => format_zero_page_x(emulator, "AND"),
			0x2d => format_absolute(emulator, "AND"),
			0x3d => format_absolute_x(emulator, "AND"),
			0x39 => format_absolute_y(emulator, "AND"),
			0x21 => format_indirect_x(emulator, "AND"),
			0x31 => format_indirect_y(emulator, "AND"),

			0x0b => format_immediate(emulator, "AAC"),
			0x2b => format_immediate(emulator, "AAC"),

			0x4b => format_immediate(emulator, "ASR"),
			0x6b => format_immediate(emulator, "ARR"),
			0xcb => format_immediate(emulator, "AXS"),

			0x09 => format_immediate(emulator, "ORA"),
			0x05 => format_zero_page(emulator, "ORA"),
			0x15 => format_zero_page_x(emulator, "ORA"),
			0x0d => format_absolute(emulator, "ORA"),
			0x1d => format_absolute_x(emulator, "ORA"),
			0x19 => format_absolute_y(emulator, "ORA"),
			0x01 => format_indirect_x(emulator, "ORA"),
			0x11 => format_indirect_y(emulator, "ORA"),

			0x49 => format_immediate(emulator, "EOR"),
			0x45 => format_zero_page(emulator, "EOR"),
			0x55 => format_zero_page_x(emulator, "EOR"),
			0x4d => format_absolute(emulator, "EOR"),
			0x5d => format_absolute_x(emulator, "EOR"),
			0x59 => format_absolute_y(emulator, "EOR"),
			0x41 => format_indirect_x(emulator, "EOR"),
			0x51 => format_indirect_y(emulator, "EOR"),

			0x24 => format_zero_page(emulator, "BIT"),
			0x2c => format_absolute(emulator, "BIT"),

			0x4a => format_a("LSR"),
			0x46 => format_zero_page(emulator, "LSR"),
			0x56 => format_zero_page_x(emulator, "LSR"),
			0x4e => format_absolute(emulator, "LSR"),
			0x5e => format_absolute_x(emulator, "LSR"),

			0x47 => format_zero_page(emulator, "*SRE"),
			0x57 => format_zero_page_x(emulator, "*SRE"),
			0x4f => format_absolute(emulator, "*SRE"),
			0x5f => format_absolute_x(emulator, "*SRE"),
			0x5b => format_absolute_y(emulator, "*SRE"),
			0x43 => format_indirect_x(emulator, "*SRE"),
			0x53 => format_indirect_y(emulator, "*SRE"),		

			0x0a => format_a("ASL"),
			0x06 => format_zero_page(emulator, "ASL"),
			0x16 => format_zero_page_x(emulator, "ASL"),
			0x0e => format_absolute(emulator, "ASL"),
			0x1e => format_absolute_x(emulator, "ASL"),

			0x07 => format_zero_page(emulator, "*SLO"),
			0x17 => format_zero_page_x(emulator, "*SLO"),
			0x0f => format_absolute(emulator, "*SLO"),
			0x1f => format_absolute_x(emulator, "*SLO"),
			0x1b => format_absolute_y(emulator, "*SLO"),
			0x03 => format_indirect_x(emulator, "*SLO"),
			0x13 => format_indirect_y(emulator, "*SLO"),

			0x6a => format_a("ROR"),
			0x66 => format_zero_page(emulator, "ROR"),
			0x76 => format_zero_page_x(emulator, "ROR"),
			0x6e => format_absolute(emulator, "ROR"),
			0x7e => format_absolute_x(emulator, "ROR"),

			0x67 => format_zero_page(emulator, "*RRA"),
			0x77 => format_zero_page_x(emulator, "*RRA"),
			0x6f => format_absolute(emulator, "*RRA"),
			0x7f => format_absolute_x(emulator, "*RRA"),
			0x7b => format_absolute_y(emulator, "*RRA"),
			0x63 => format_indirect_x(emulator, "*RRA"),
			0x73 => format_indirect_y(emulator, "*RRA"),

			0x2a => format_a("ROL"),
			0x26 => format_zero_page(emulator, "ROL"),
			0x36 => format_zero_page_x(emulator, "ROL"),
			0x2e => format_absolute(emulator, "ROL"),
			0x3e => format_absolute_x(emulator, "ROL"),

			0x27 => format_zero_page(emulator, "*RLA"), 
			0x37 => format_zero_page_x(emulator, "*RLA"),
			0x2f => format_absolute(emulator, "*RLA"),
			0x3f => format_absolute_x(emulator, "*RLA"),
			0x3b => format_absolute_y(emulator, "*RLA"),
			0x23 => format_indirect_x(emulator, "*RLA"),
			0x33 => format_indirect_y(emulator, "*RLA"),

			0x69 => format_immediate(emulator, "ADC"),
			0x65 => format_zero_page(emulator, "ADC"),
			0x75 => format_zero_page_x(emulator, "ADC"),
			0x6d => format_absolute(emulator, "ADC"),
			0x7d => format_absolute_x(emulator, "ADC"),
			0x79 => format_absolute_y(emulator, "ADC"),
			0x61 => format_indirect_x(emulator, "ADC"),
			0x71 => format_indirect_y(emulator, "ADC"),

			0xe9 => format_immediate(emulator, "SBC"),
			0xe5 => format_zero_page(emulator, "SBC"),
			0xf5 => format_zero_page_x(emulator, "SBC"),
			0xed => format_absolute(emulator, "SBC"),
			0xfd => format_absolute_x(emulator, "SBC"),
			0xf9 => format_absolute_y(emulator, "SBC"),
			0xe1 => format_indirect_x(emulator, "SBC"),
			0xf1 => format_indirect_y(emulator, "SBC"),
			
			0xeb => format_immediate(emulator, "*SBC"),

			0xe8 => format("INX"),
			0xc8 => format("INY"),

			0xe6 => format_zero_page(emulator, "INC"),
			0xf6 => format_zero_page_x(emulator, "INC"),
			0xee => format_absolute(emulator, "INC"),
			0xfe => format_absolute_x(emulator, "INC"),

			0xe7 => format_zero_page(emulator, "*ISB"),
			0xf7 => format_zero_page_x(emulator, "*ISB"),
			0xef => format_absolute(emulator, "*ISB"),
			0xff => format_absolute_x(emulator, "*ISB"),
			0xfb => format_absolute_y(emulator, "*ISB"),
			0xe3 => format_indirect_x(emulator, "*ISB"),
			0xf3 => format_indirect_y(emulator, "*ISB"),

			0xca => format("DEX"),
			0x88 => format("DEY"),

			0xc6 => format_zero_page(emulator, "DEC"),
			0xd6 => format_zero_page_x(emulator, "DEC"),
			0xce => format_absolute(emulator, "DEC"),
			0xde => format_absolute_x(emulator, "DEC"),

			0xc7 => format_zero_page(emulator, "*DCP"),
			0xd7 => format_zero_page_x(emulator, "*DCP"),
			0xcf => format_absolute(emulator, "*DCP"),
			0xdf => format_absolute_x(emulator, "*DCP"),
			0xdb => format_absolute_y(emulator, "*DCP"),
			0xc3 => format_indirect_x(emulator, "*DCP"),
			0xd3 => format_indirect_y(emulator, "*DCP"),

			0xe0 => format_immediate(emulator, "CPX"),
			0xe4 => format_zero_page(emulator, "CPX"),
			0xec => format_absolute(emulator, "CPX"),

			0xc0 => format_immediate(emulator, "CPY"),
			0xc4 => format_zero_page(emulator, "CPY"),
			0xcc => format_absolute(emulator, "CPY"),

			0xc9 => format_immediate(emulator, "CMP"),
			0xc5 => format_zero_page(emulator, "CMP"),
			0xd5 => format_zero_page_x(emulator, "CMP"),
			0xcd => format_absolute(emulator, "CMP"),
			0xdd => format_absolute_x(emulator, "CMP"),
			0xd9 => format_absolute_y(emulator, "CMP"),
			0xc1 => format_indirect_x(emulator, "CMP"),
			0xd1 => format_indirect_y(emulator, "CMP"),

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

			0x4c => format_jump_absolute(emulator, "JMP"),
			0x20 => format_jump_absolute(emulator, "JSR"),

			// JMP (indirect)
			0x6c => {
				let pc = emulator.cpu.pc.wrapping_add(1);
				let address = read16_debug(emulator, pc);
				let low_byte = read8_debug(emulator, address) as u16;
				let high_byte = if (address & 0xff) == 0xff  {
					read8_debug(emulator, address & 0xff00)
				} else {
					read8_debug(emulator, address + 1)
				} as u16;
				let effective_address = (high_byte << 8) | low_byte;
				format!("{:02X} {:02X}  JMP (${:04X}) = {:04X}", low_byte, high_byte, address, effective_address)
			},

			0x10 => format_jump_relative(emulator, "BPL"),
			0x30 => format_jump_relative(emulator, "BMI"),
			0x50 => format_jump_relative(emulator, "BVC"),
			0x70 => format_jump_relative(emulator, "BVS"),
			0x90 => format_jump_relative(emulator, "BCC"),
			0xb0 => format_jump_relative(emulator, "BCS"),
			0xd0 => format_jump_relative(emulator, "BNE"),
			0xf0 => format_jump_relative(emulator, "BEQ"),

			0x00 => format("BRK"),

			0x40 => format("RTI"),
			0x60 => format("RTS"),

			0x32 => format("KIL"),

			_ => {
				warn!("Unknown opcode {:02X} at {:04X}", opcode, emulator.cpu.pc);
				"# UNKNOWN OPCODE #".to_string()
			}
		};

		let log = format!("{:04X}  {:02X} {:<38} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}\n", emulator.cpu.pc, opcode, instruction_string, emulator.cpu.a, emulator.cpu.x, emulator.cpu.y, emulator.cpu.p, emulator.cpu.s);
		emulator.cpu.logger.buffer.borrow_mut().push(log);
	}
}

fn format(mnemonic: &str) -> String {
	format!("{:>10}", mnemonic)
}

fn format_a(mnemonic: &str) -> String {
	format!("{:>10} A", mnemonic)
}

fn format_immediate(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let operand = read8_debug(emulator, pc);
	format!("{:02X}{:>8} #${:02X}", operand, mnemonic, operand)
}

fn format_zero_page(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read8_debug(emulator, pc);
	let operand = read8_debug(emulator, address as _);
	format!("{:02X}{:>8} ${:02X} = {:02X}", address, mnemonic, address, operand)
}

fn format_zero_page_x(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read8_debug(emulator, pc);
	let effective_address = address.wrapping_add(emulator.cpu.x);
	let operand = read8_debug(emulator, effective_address as _);
	format!("{:02X}{:>8} ${:02X},X @ {:02X} = {:02X}", address, mnemonic, address, effective_address, operand)
}

fn format_zero_page_y(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read8_debug(emulator, pc);
	let effective_address = address.wrapping_add(emulator.cpu.y);
	let operand = read8_debug(emulator, effective_address as _);
	format!("{:02X}{:>8} ${:02X},Y @ {:02X} = {:02X}", address, mnemonic, address, effective_address, operand)
}

fn format_absolute(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read16_debug(emulator, pc);
	let low_byte = address & 0xff;
	let high_byte = address >> 8;
	let operand = read8_debug(emulator, address);
	format!("{:02X} {:02X}{:>5} ${:04X} = {:02X}", low_byte, high_byte, mnemonic, address, operand)
}

fn format_absolute_x(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read16_debug(emulator, pc);
	let low_byte = address & 0xff;
	let high_byte = address >> 8;
	let effective_address = address.wrapping_add(emulator.cpu.x as _);
	let operand = read8_debug(emulator, effective_address);
	format!("{:02X} {:02X}{:>5} ${:04X},X @ {:04X} = {:02X}", low_byte, high_byte, mnemonic, address, effective_address, operand)
}

fn format_absolute_y(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read16_debug(emulator, pc);
	let low_byte = address & 0xff;
	let high_byte = address >> 8;
	let effective_address = address.wrapping_add(emulator.cpu.y as _);
	let operand = read8_debug(emulator, effective_address);
	format!("{:02X} {:02X}{:>5} ${:04X},Y @ {:04X} = {:02X}", low_byte, high_byte, mnemonic, address, effective_address, operand)
}

fn format_indirect_x(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let immediate = read8_debug(emulator, pc);
	let address = immediate.wrapping_add(emulator.cpu.x);
	let effective_address = read16_zeropage_debug(emulator, address);
	let operand = read8_debug(emulator, effective_address);
	format!("{:02X}{:>8} (${:02X},X) @ {:02X} = {:04X} = {:02X}", immediate, mnemonic, immediate, address, effective_address, operand)
}

fn format_indirect_y(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let immediate = read8_debug(emulator, pc);
	let address = read16_zeropage_debug(emulator, address);
	let effective_address = address.wrapping_add(emulator.cpu.y as _);
	let operand = read8_debug(emulator, effective_address);
	format!("{:02X}{:>8} (${:02X}),Y = {:04X} @ {:04X} = {:02X}", immediate, mnemonic, immediate, address, effective_address, operand)
}

fn format_jump_absolute(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let address = read16_debug(emulator, pc);
	let low_byte = address & 0xff;
	let high_byte = address >> 8;
	format!("{:02X} {:02X}{:>5} ${:04X}", low_byte, high_byte, mnemonic, address)
}

fn format_jump_relative(emulator: &Emulator, mnemonic: &str) -> String {
	let pc = emulator.cpu.pc.wrapping_add(1);
	let offset = read8_debug(emulator, pc) as i8;
	let address = if offset > 0 {
		pc.wrapping_add(offset as _)
	} else {
		pc.wrapping_sub(offset.wrapping_neg() as _)
	}.wrapping_add(1);
	format!("{:02X}{:>8} ${:04X}", offset, mnemonic, address)
}
