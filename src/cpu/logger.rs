use std::io::prelude::Write;
use std::fs::File;
use std::cell::RefCell;

use emulator::*;
use super::memory::*;

pub(super) struct Logger {
	file: RefCell<File>
}

impl Logger {
	pub(super) fn new() -> Self {
		Self {
			file: RefCell::new(File::create("rnes.log").unwrap())
		}
	}

	fn format(mnemonic: &str) -> String {
		format!("{:>10}", mnemonic)
	}

	fn format_a(mnemonic: &str) -> String {
		format!("{:>10} A", mnemonic)
	}

	fn format_immediate(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let operand = read8_debug(emulator, pc);
		format!("{:02X}{:>8} #${:02X}", operand, mnemonic, operand)
	}

	fn format_zero_page(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let address = read8_debug(emulator, pc);
		let operand = read8_debug(emulator, address as _);
		format!("{:02X}{:>8} ${:02X} = {:02X}", address, mnemonic, address, operand)
	}

	fn format_zero_page_x(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let address = read8_debug(emulator, pc);
		let effective_address = address.wrapping_add(emulator.cpu.x);
		let operand = read8_debug(emulator, effective_address as _);
		format!("{:02X}{:>8} ${:02X},X @ {:02X} = {:02X}", address, mnemonic, address, effective_address, operand)
	}

	fn format_zero_page_y(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let address = read8_debug(emulator, pc);
		let effective_address = address.wrapping_add(emulator.cpu.y);
		let operand = read8_debug(emulator, effective_address as _);
		format!("{:02X}{:>8} ${:02X},Y @ {:02X} = {:02X}", address, mnemonic, address, effective_address, operand)
	}

	fn format_absolute(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let address = read16(emulator, pc);
		let low_byte = address & 0xff;
		let high_byte = address >> 8;
		let operand = read8_debug(emulator, address);
		format!("{:02X} {:02X}{:>5} ${:04X} = {:02X}", low_byte, high_byte, mnemonic, address, operand)
	}

	fn format_absolute_x(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let address = read16(emulator, pc);
		let low_byte = address & 0xff;
		let high_byte = address >> 8;
		let effective_address = address.wrapping_add(emulator.cpu.x as _);
		let operand = read8_debug(emulator, effective_address);
		format!("{:02X} {:02X}{:>5} ${:04X},X @ {:04X} = {:02X}", low_byte, high_byte, mnemonic, address, effective_address, operand)
	}

	fn format_absolute_y(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let address = read16(emulator, pc);
		let low_byte = address & 0xff;
		let high_byte = address >> 8;
		let effective_address = address.wrapping_add(emulator.cpu.y as _);
		let operand = read8_debug(emulator, effective_address);
		format!("{:02X} {:02X}{:>5} ${:04X},Y @ {:04X} = {:02X}", low_byte, high_byte, mnemonic, address, effective_address, operand)
	}

	fn format_indirect_x(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let immediate = read8_debug(emulator, pc);
		let address = immediate.wrapping_add(emulator.cpu.x);
		let low_byte = read8_debug(emulator, address as _) as u16;
		let high_byte = read8_debug(emulator, address.wrapping_add(1) as _) as u16;
		let effective_address = (high_byte << 8) | low_byte;
		let operand = read8_debug(emulator, effective_address);
		format!("{:02X}{:>8} (${:02X},X) @ {:02X} = {:04X} = {:02X}", immediate, mnemonic, immediate, address, effective_address, operand)
	}

	fn format_indirect_y(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let immediate = read8_debug(emulator, pc);
		let low_byte = read8_debug(emulator, immediate as _) as u16;
		let high_byte = read8_debug(emulator, immediate.wrapping_add(1) as _) as u16;
		let address = (high_byte << 8) | low_byte;
		let effective_address = address.wrapping_add(emulator.cpu.y as _);
		let operand = read8_debug(emulator, address);
		format!("{:02X}{:>8} (${:02X}),Y = {:04X} @ {:04X} = {:02X}", immediate, mnemonic, immediate, address, effective_address, operand)
	}

	fn format_jump_absolute(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let address = read16(emulator, pc);
		let low_byte = address & 0xff;
		let high_byte = address >> 8;
		format!("{:02X} {:02X}{:>5} ${:04X}", low_byte, high_byte, mnemonic, address)
	}

	fn format_jump_relative(emulator: &mut Emulator, mnemonic: &str) -> String {
		let pc = emulator.cpu.pc.wrapping_add(1);
		let offset = read8_debug(emulator, pc) as i8;
		let address = if offset > 0 {
			pc.wrapping_add(offset as _)
		} else {
			pc.wrapping_sub(offset.wrapping_neg() as _)
		}.wrapping_add(1);
		format!("{:02X}{:>8} ${:04X}", offset, mnemonic, address)
	}

	pub(super) fn create_log(emulator: &mut Emulator) {
		let opcode = read8_debug(emulator, emulator.cpu.pc);
		let instruction_string = match opcode {
			0xea => Self::format("NOP"),
			0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => Self::format("*NOP"),
			0x80 => Self::format_immediate(emulator, "*NOP"),
			0x04 | 0x44 | 0x64 | 0x82 | 0x89 | 0xc2 | 0xe2 => Self::format_zero_page(emulator, "*NOP"),
			0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 => Self::format_zero_page_x(emulator, "*NOP"),
			0x0c => Self::format_absolute(emulator, "*NOP"),
			0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => Self::format_absolute_x(emulator, "*NOP"),
			
			0xa9 => Self::format_immediate(emulator, "LDA"),
			0xa5 => Self::format_zero_page(emulator, "LDA"),
			0xb5 => Self::format_zero_page_x(emulator, "LDA"),
			0xad => Self::format_absolute(emulator, "LDA"),
			0xbd => Self::format_absolute_x(emulator, "LDA"),
			0xb9 => Self::format_absolute_y(emulator, "LDA"),
			0xa1 => Self::format_indirect_x(emulator, "LDA"),
			0xb1 => Self::format_indirect_y(emulator, "LDA"),

			0xa2 => Self::format_immediate(emulator, "LDX"),
			0xa6 => Self::format_zero_page(emulator, "LDX"),
			0xb6 => Self::format_zero_page_y(emulator, "LDX"),
			0xae => Self::format_absolute(emulator, "LDX"),
			0xbe => Self::format_absolute_y(emulator, "LDX"),

			0xa0 => Self::format_immediate(emulator, "LDY"),
			0xa4 => Self::format_zero_page(emulator, "LDY"),
			0xb4 => Self::format_zero_page_x(emulator, "LDY"),
			0xac => Self::format_absolute(emulator, "LDY"),
			0xbc => Self::format_absolute_x(emulator, "LDY"),

			0xab => Self::format_immediate(emulator, "LAX"),
			0xa7 => Self::format_zero_page(emulator, "LAX"),
			0xb7 => Self::format_zero_page_y(emulator, "LAX"),
			0xaf => Self::format_absolute(emulator, "LAX"),
			0xbf => Self::format_absolute_y(emulator, "LAX"),
			0xa3 => Self::format_indirect_x(emulator, "LAX"),
			0xb3 => Self::format_indirect_y(emulator, "LAX"),

			0x85 => Self::format_zero_page(emulator, "STA"),
			0x95 => Self::format_zero_page_x(emulator, "STA"),
			0x8d => Self::format_absolute(emulator, "STA"),
			0x9d => Self::format_absolute_x(emulator, "STA"),
			0x99 => Self::format_absolute_y(emulator, "STA"),
			0x81 => Self::format_indirect_x(emulator, "STA"),
			0x91 => Self::format_indirect_y(emulator, "STA"),

			0x86 => Self::format_zero_page(emulator, "STX"),
			0x96 => Self::format_zero_page_y(emulator, "STX"),
			0x8e => Self::format_absolute(emulator, "STX"),

			0x84 => Self::format_zero_page(emulator, "STY"),
			0x94 => Self::format_zero_page_x(emulator, "STY"),
			0x8c => Self::format_absolute(emulator, "STY"),

			0x87 => Self::format_zero_page(emulator, "SAX"),
			0x97 => Self::format_zero_page_y(emulator, "SAX"),
			0x8f => Self::format_absolute(emulator, "SAX"),
			0x83 => Self::format_indirect_x(emulator, "SAX"),

			0x9e => Self::format_absolute_y(emulator, "SXA"),
			0x9c => Self::format_absolute_x(emulator, "SYA"),

			0xaa => Self::format("TAX"),
			0x8a => Self::format("TXA"),
			0xa8 => Self::format("TAY"),
			0x98 => Self::format("TYA"),
			0xba => Self::format("TSX"),
			0x9a => Self::format("TXS"),

			0x29 => Self::format_immediate(emulator, "AND"),
			0x25 => Self::format_zero_page(emulator, "AND"),
			0x35 => Self::format_zero_page_x(emulator, "AND"),
			0x2d => Self::format_absolute(emulator, "AND"),
			0x3d => Self::format_absolute_x(emulator, "AND"),
			0x39 => Self::format_absolute_y(emulator, "AND"),
			0x21 => Self::format_indirect_x(emulator, "AND"),
			0x31 => Self::format_indirect_y(emulator, "AND"),

			0x0b => Self::format_immediate(emulator, "AAC"),
			0x2b => Self::format_immediate(emulator, "AAC"),

			0x4b => Self::format_immediate(emulator, "ASR"),
			0x6b => Self::format_immediate(emulator, "ARR"),
			0xcb => Self::format_immediate(emulator, "AXS"),

			0x09 => Self::format_immediate(emulator, "ORA"),
			0x05 => Self::format_zero_page(emulator, "ORA"),
			0x15 => Self::format_zero_page_x(emulator, "ORA"),
			0x0d => Self::format_absolute(emulator, "ORA"),
			0x1d => Self::format_absolute_x(emulator, "ORA"),
			0x19 => Self::format_absolute_y(emulator, "ORA"),
			0x01 => Self::format_indirect_x(emulator, "ORA"),
			0x11 => Self::format_indirect_y(emulator, "ORA"),

			0x49 => Self::format_immediate(emulator, "EOR"),
			0x45 => Self::format_zero_page(emulator, "EOR"),
			0x55 => Self::format_zero_page_x(emulator, "EOR"),
			0x4d => Self::format_absolute(emulator, "EOR"),
			0x5d => Self::format_absolute_x(emulator, "EOR"),
			0x59 => Self::format_absolute_y(emulator, "EOR"),
			0x41 => Self::format_indirect_x(emulator, "EOR"),
			0x51 => Self::format_indirect_y(emulator, "EOR"),

			0x24 => Self::format_zero_page(emulator, "BIT"),
			0x2c => Self::format_absolute(emulator, "BIT"),

			0x4a => Self::format_a("LSR"),
			0x46 => Self::format_zero_page(emulator, "LSR"),
			0x56 => Self::format_zero_page_x(emulator, "LSR"),
			0x4e => Self::format_absolute(emulator, "LSR"),
			0x5e => Self::format_absolute_x(emulator, "LSR"),

			0x47 => Self::format_zero_page(emulator, "*SRE"),
			0x57 => Self::format_zero_page_x(emulator, "*SRE"),
			0x4f => Self::format_absolute(emulator, "*SRE"),
			0x5f => Self::format_absolute_x(emulator, "*SRE"),
			0x5b => Self::format_absolute_y(emulator, "*SRE"),
			0x43 => Self::format_indirect_x(emulator, "*SRE"),
			0x53 => Self::format_indirect_y(emulator, "*SRE"),		

			0x0a => Self::format_a("ASL"),
			0x06 => Self::format_zero_page(emulator, "ASL"),
			0x16 => Self::format_zero_page_x(emulator, "ASL"),
			0x0e => Self::format_absolute(emulator, "ASL"),
			0x1e => Self::format_absolute_x(emulator, "ASL"),

			0x07 => Self::format_zero_page(emulator, "*SLO"),
			0x17 => Self::format_zero_page_x(emulator, "*SLO"),
			0x0f => Self::format_absolute(emulator, "*SLO"),
			0x1f => Self::format_absolute_x(emulator, "*SLO"),
			0x1b => Self::format_absolute_y(emulator, "*SLO"),
			0x03 => Self::format_indirect_x(emulator, "*SLO"),
			0x13 => Self::format_indirect_y(emulator, "*SLO"),

			0x6a => Self::format_a("ROR"),
			0x66 => Self::format_zero_page(emulator, "ROR"),
			0x76 => Self::format_zero_page_x(emulator, "ROR"),
			0x6e => Self::format_absolute(emulator, "ROR"),
			0x7e => Self::format_absolute_x(emulator, "ROR"),

			0x67 => Self::format_zero_page(emulator, "*RRA"),
			0x77 => Self::format_zero_page_x(emulator, "*RRA"),
			0x6f => Self::format_absolute(emulator, "*RRA"),
			0x7f => Self::format_absolute_x(emulator, "*RRA"),
			0x7b => Self::format_absolute_y(emulator, "*RRA"),
			0x63 => Self::format_indirect_x(emulator, "*RRA"),
			0x73 => Self::format_indirect_y(emulator, "*RRA"),

			0x2a => Self::format_a("ROL"),
			0x26 => Self::format_zero_page(emulator, "ROL"),
			0x36 => Self::format_zero_page_x(emulator, "ROL"),
			0x2e => Self::format_absolute(emulator, "ROL"),
			0x3e => Self::format_absolute_x(emulator, "ROL"),

			0x27 => Self::format_zero_page(emulator, "*RLA"), 
			0x37 => Self::format_zero_page_x(emulator, "*RLA"),
			0x2f => Self::format_absolute(emulator, "*RLA"),
			0x3f => Self::format_absolute_x(emulator, "*RLA"),
			0x3b => Self::format_absolute_y(emulator, "*RLA"),
			0x23 => Self::format_indirect_x(emulator, "*RLA"),
			0x33 => Self::format_indirect_y(emulator, "*RLA"),

			0x69 => Self::format_immediate(emulator, "ADC"),
			0x65 => Self::format_zero_page(emulator, "ADC"),
			0x75 => Self::format_zero_page_x(emulator, "ADC"),
			0x6d => Self::format_absolute(emulator, "ADC"),
			0x7d => Self::format_absolute_x(emulator, "ADC"),
			0x79 => Self::format_absolute_y(emulator, "ADC"),
			0x61 => Self::format_indirect_x(emulator, "ADC"),
			0x71 => Self::format_indirect_y(emulator, "ADC"),

			0xe9 => Self::format_immediate(emulator, "SBC"),
			0xe5 => Self::format_zero_page(emulator, "SBC"),
			0xf5 => Self::format_zero_page_x(emulator, "SBC"),
			0xed => Self::format_absolute(emulator, "SBC"),
			0xfd => Self::format_absolute_x(emulator, "SBC"),
			0xf9 => Self::format_absolute_y(emulator, "SBC"),
			0xe1 => Self::format_indirect_x(emulator, "SBC"),
			0xf1 => Self::format_indirect_y(emulator, "SBC"),
			
			0xeb => Self::format_immediate(emulator, "*SBC"),

			0xe8 => Self::format("INX"),
			0xc8 => Self::format("INY"),

			0xe6 => Self::format_zero_page(emulator, "INC"),
			0xf6 => Self::format_zero_page_x(emulator, "INC"),
			0xee => Self::format_absolute(emulator, "INC"),
			0xfe => Self::format_absolute_x(emulator, "INC"),

			0xe7 => Self::format_zero_page(emulator, "*ISB"),
			0xf7 => Self::format_zero_page_x(emulator, "*ISB"),
			0xef => Self::format_absolute(emulator, "*ISB"),
			0xff => Self::format_absolute_x(emulator, "*ISB"),
			0xfb => Self::format_absolute_y(emulator, "*ISB"),
			0xe3 => Self::format_indirect_x(emulator, "*ISB"),
			0xf3 => Self::format_indirect_y(emulator, "*ISB"),

			0xca => Self::format("DEX"),
			0x88 => Self::format("DEY"),

			0xc6 => Self::format_zero_page(emulator, "DEC"),
			0xd6 => Self::format_zero_page_x(emulator, "DEC"),
			0xce => Self::format_absolute(emulator, "DEC"),
			0xde => Self::format_absolute_x(emulator, "DEC"),

			0xc7 => Self::format_zero_page(emulator, "*DCP"),
			0xd7 => Self::format_zero_page_x(emulator, "*DCP"),
			0xcf => Self::format_absolute(emulator, "*DCP"),
			0xdf => Self::format_absolute_x(emulator, "*DCP"),
			0xdb => Self::format_absolute_y(emulator, "*DCP"),
			0xc3 => Self::format_indirect_x(emulator, "*DCP"),
			0xd3 => Self::format_indirect_y(emulator, "*DCP"),

			0xe0 => Self::format_immediate(emulator, "CPX"),
			0xe4 => Self::format_zero_page(emulator, "CPX"),
			0xec => Self::format_absolute(emulator, "CPX"),

			0xc0 => Self::format_immediate(emulator, "CPY"),
			0xc4 => Self::format_zero_page(emulator, "CPY"),
			0xcc => Self::format_absolute(emulator, "CPY"),

			0xc9 => Self::format_immediate(emulator, "CMP"),
			0xc5 => Self::format_zero_page(emulator, "CMP"),
			0xd5 => Self::format_zero_page_x(emulator, "CMP"),
			0xcd => Self::format_absolute(emulator, "CMP"),
			0xdd => Self::format_absolute_x(emulator, "CMP"),
			0xd9 => Self::format_absolute_y(emulator, "CMP"),
			0xc1 => Self::format_indirect_x(emulator, "CMP"),
			0xd1 => Self::format_indirect_y(emulator, "CMP"),

			0x48 => Self::format("PHA"),
			0x68 => Self::format("PLA"),
			0x08 => Self::format("PHP"),
			0x28 => Self::format("PLP"),
			0x18 => Self::format("CLC"),
			0x38 => Self::format("SEC"),
			0x58 => Self::format("CLI"),
			0x78 => Self::format("SEI"),
			0xd8 => Self::format("CLD"),
			0xf8 => Self::format("SED"),
			0xb8 => Self::format("CLV"),

			0x4c => Self::format_jump_absolute(emulator, "JMP"),
			0x20 => Self::format_jump_absolute(emulator, "JSR"),

			// JMP (indirect)
			0x6c => {
				let pc = emulator.cpu.pc.wrapping_add(1);
				let address = read16(emulator, pc);
				let low_byte = read8_debug(emulator, address) as u16;
				let high_byte = if (address & 0xff) == 0xff  {
					read8_debug(emulator, address & 0xff00)
				} else {
					read8_debug(emulator, address + 1)
				} as u16;
				let effective_address = (high_byte << 8) | low_byte;
				format!("{:02X} {:02X}  JMP (${:04X}) = {:04X}", low_byte, high_byte, address, effective_address)
			},

			0x10 => Self::format_jump_relative(emulator, "BPL"),
			0x30 => Self::format_jump_relative(emulator, "BMI"),
			0x50 => Self::format_jump_relative(emulator, "BVC"),
			0x70 => Self::format_jump_relative(emulator, "BVS"),
			0x90 => Self::format_jump_relative(emulator, "BCC"),
			0xb0 => Self::format_jump_relative(emulator, "BCS"),
			0xd0 => Self::format_jump_relative(emulator, "BNE"),
			0xf0 => Self::format_jump_relative(emulator, "BEQ"),

			0x00 => Self::format("BRK"),

			0x40 => Self::format("RTI"),
			0x60 => Self::format("RTS"),

			_ => {
				println!("[ERROR] Unknown opcode {:02X} at {:04X}", opcode, emulator.cpu.pc);
				std::process::exit(1);
			}
		};
	//	write!(emulator.cpu.logger.file.borrow_mut(), "{:04X}  {:02X} {:<38} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}\n", emulator.cpu.pc, opcode, instruction_string, emulator.cpu.a, emulator.cpu.x, emulator.cpu.y, emulator.cpu.p, emulator.cpu.s).unwrap();
	}
}
