use std::io::prelude::Write;
use std::fs::File;
use std::cell::RefCell;

use cpu::*;
use cpu_memory::*;

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

	fn format_immediate(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let operand = memory.read8(cpu.pc.wrapping_add(1));
		format!("{:02X}{:>8} #${:02X}", operand, mnemonic, operand)
	}

	fn format_zero_page(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let address = memory.read8(cpu.pc.wrapping_add(1));
		let operand = memory.read8(address as _);
		format!("{:02X}{:>8} ${:02X} = {:02X}", address, mnemonic, address, operand)
	}

	fn format_zero_page_x(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let immediate = memory.read8(cpu.pc.wrapping_add(1));
		let address = immediate.wrapping_add(cpu.x);
		let operand = memory.read8(address as _);
		format!("{:02X}{:>8} ${:02X},X @ {:02X} = {:02X}", immediate, mnemonic, immediate, address, operand)
	}

	fn format_zero_page_y(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let immediate = memory.read8(cpu.pc.wrapping_add(1));
		let address = immediate.wrapping_add(cpu.y);
		let operand = memory.read8(address as _);
		format!("{:02X}{:>8} ${:02X},Y @ {:02X} = {:02X}", immediate, mnemonic, immediate, address, operand)
	}

	fn format_absolute(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let address = memory.read16(cpu.pc.wrapping_add(1));
		let low_byte = address & 0xff;
		let high_byte = address >> 8;
		let operand = memory.read8(address);
		format!("{:02X} {:02X}{:>5} ${:04X} = {:02X}", low_byte, high_byte, mnemonic, address, operand)
	}

	fn format_absolute_x(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let intermediate_address = memory.read16(cpu.pc.wrapping_add(1));
		let low_byte = intermediate_address & 0xff;
		let high_byte = intermediate_address >> 8;
		let address = intermediate_address.wrapping_add(cpu.x as _);
		let operand = memory.read8(address);
		format!("{:02X} {:02X}{:>5} ${:04X},X @ {:04X} = {:02X}", low_byte, high_byte, mnemonic, intermediate_address, address, operand)
	}

	fn format_absolute_y(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let intermediate_address = memory.read16(cpu.pc.wrapping_add(1));
		let low_byte = intermediate_address & 0xff;
		let high_byte = intermediate_address >> 8;
		let address = intermediate_address.wrapping_add(cpu.y as _);
		let operand = memory.read8(address);
		format!("{:02X} {:02X}{:>5} ${:04X},Y @ {:04X} = {:02X}", low_byte, high_byte, mnemonic, intermediate_address, address, operand)
	}

	fn format_indirect_x(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let immediate = memory.read8(cpu.pc.wrapping_add(1));
		let intermediate_address = immediate.wrapping_add(cpu.x);
		let low_byte = memory.read8(intermediate_address as _) as u16;
		let high_byte = memory.read8(intermediate_address.wrapping_add(1) as _) as u16;
		let address = (high_byte << 8) | low_byte;
		let operand = memory.read8(address);
		format!("{:02X}{:>8} (${:02X},X) @ {:02X} = {:04X} = {:02X}", immediate, mnemonic, immediate, intermediate_address, address, operand)
	}

	fn format_indirect_y(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let immediate = memory.read8(cpu.pc.wrapping_add(1));
		let low_byte = memory.read8(immediate as _) as u16;
		let high_byte = memory.read8(immediate.wrapping_add(1) as _) as u16;
		let intermediate_address = (high_byte << 8) | low_byte;
		let address = intermediate_address.wrapping_add(cpu.y as _);
		let operand = memory.read8(address);
		format!("{:02X}{:>8} (${:02X}),Y = {:04X} @ {:04X} = {:02X}", immediate, mnemonic, immediate, intermediate_address, address, operand)
	}

	fn format_jump_absolute(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let address = memory.read16(cpu.pc.wrapping_add(1));
		let low_byte = address & 0xff;
		let high_byte = address >> 8;
		format!("{:02X} {:02X}{:>5} ${:04X}", low_byte, high_byte, mnemonic, address)
	}

	fn format_jump_relative(cpu: &Cpu, memory: &CpuMemory, mnemonic: &str) -> String {
		let offset = memory.read8(cpu.pc.wrapping_add(1)) as i8;
		let address = if offset > 0 {
			cpu.pc.wrapping_add(offset as _)
		} else {
			cpu.pc.wrapping_sub(offset.wrapping_neg() as _)
		}.wrapping_add(2);
		format!("{:02X}{:>8} ${:04X}", offset, mnemonic, address)
	}

	pub(super) fn create_log(&self, cpu: &Cpu, memory: &CpuMemory) {
		let opcode = memory.read8(cpu.pc);
		let instruction_string = match opcode {
			0xea => Self::format("NOP"),
			0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => Self::format("*NOP"),
			0x80 => Self::format_immediate(cpu, memory, "*NOP"),
			0x04 | 0x44 | 0x64 /* 0x82 | 0x89 | 0xc2 | 0xe2 */ => Self::format_zero_page(cpu, memory, "*NOP"),
			0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 => Self::format_zero_page_x(cpu, memory, "*NOP"),
			0x0c => Self::format_absolute(cpu, memory, "*NOP"),
			0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => Self::format_absolute_x(cpu, memory, "*NOP"),
			
			0xa9 => Self::format_immediate(cpu, memory, "LDA"),
			0xa5 => Self::format_zero_page(cpu, memory, "LDA"),
			0xb5 => Self::format_zero_page_x(cpu, memory, "LDA"),
			0xad => Self::format_absolute(cpu, memory, "LDA"),
			0xbd => Self::format_absolute_x(cpu, memory, "LDA"),
			0xb9 => Self::format_absolute_y(cpu, memory, "LDA"),
			0xa1 => Self::format_indirect_x(cpu, memory, "LDA"),
			0xb1 => Self::format_indirect_y(cpu, memory, "LDA"),

			0xa2 => Self::format_immediate(cpu, memory, "LDX"),
			0xa6 => Self::format_zero_page(cpu, memory, "LDX"),
			0xb6 => Self::format_zero_page_y(cpu, memory, "LDX"),
			0xae => Self::format_absolute(cpu, memory, "LDX"),
			0xbe => Self::format_absolute_y(cpu, memory, "LDX"),

			0xa0 => Self::format_immediate(cpu, memory, "LDY"),
			0xa4 => Self::format_zero_page(cpu, memory, "LDY"),
			0xb4 => Self::format_zero_page_x(cpu, memory, "LDY"),
			0xac => Self::format_absolute(cpu, memory, "LDY"),
			0xbc => Self::format_absolute_x(cpu, memory, "LDY"),

			//0xab => Self::format_immediate(cpu, memory, "LAX"),
			0xa7 => Self::format_zero_page(cpu, memory, "LAX"),
			0xb7 => Self::format_zero_page_y(cpu, memory, "LAX"),
			0xaf => Self::format_absolute(cpu, memory, "LAX"),
			0xbf => Self::format_absolute_y(cpu, memory, "LAX"),
			0xa3 => Self::format_indirect_x(cpu, memory, "LAX"),
			0xb3 => Self::format_indirect_y(cpu, memory, "LAX"),

			0x85 => Self::format_zero_page(cpu, memory, "STA"),
			0x95 => Self::format_zero_page_x(cpu, memory, "STA"),
			0x8d => Self::format_absolute(cpu, memory, "STA"),
			0x9d => Self::format_absolute_x(cpu, memory, "STA"),
			0x99 => Self::format_absolute_y(cpu, memory, "STA"),
			0x81 => Self::format_indirect_x(cpu, memory, "STA"),
			0x91 => Self::format_indirect_y(cpu, memory, "STA"),

			0x86 => Self::format_zero_page(cpu, memory, "STX"),
			0x96 => Self::format_zero_page_y(cpu, memory, "STX"),
			0x8e => Self::format_absolute(cpu, memory, "STX"),

			0x84 => Self::format_zero_page(cpu, memory, "STY"),
			0x94 => Self::format_zero_page_x(cpu, memory, "STY"),
			0x8c => Self::format_absolute(cpu, memory, "STY"),

			0x87 => Self::format_zero_page(cpu, memory, "SAX"),
			0x97 => Self::format_zero_page_y(cpu, memory, "SAX"),
			0x8f => Self::format_absolute(cpu, memory, "SAX"),
			0x83 => Self::format_indirect_x(cpu, memory, "SAX"),

			// SXA
			/*0x9e => {
				let address = self.get_address(memory, AddressingMode::AbsoluteY);
				memory.write8(address, self.x & self.a);
			},

			// SYA
			0x9c => {
				let address = self.get_address(memory, AddressingMode::AbsoluteX);
				memory.write8(address, self.y & self.a);
			},*/

			0xaa => Self::format("TAX"),
			0x8a => Self::format("TXA"),
			0xa8 => Self::format("TAY"),
			0x98 => Self::format("TYA"),
			0xba => Self::format("TSX"),
			0x9a => Self::format("TXS"),

			0x29 => Self::format_immediate(cpu, memory, "AND"),
			0x25 => Self::format_zero_page(cpu, memory, "AND"),
			0x35 => Self::format_zero_page_x(cpu, memory, "AND"),
			0x2d => Self::format_absolute(cpu, memory, "AND"),
			0x3d => Self::format_absolute_x(cpu, memory, "AND"),
			0x39 => Self::format_absolute_y(cpu, memory, "AND"),
			0x21 => Self::format_indirect_x(cpu, memory, "AND"),
			0x31 => Self::format_indirect_y(cpu, memory, "AND"),

			/*0x0b => self.aac(memory),
			0x2b => self.aac(memory),

			// ASR
			0x4b => {
				self.and(memory, AddressingMode::Immediate);
				self.a = self.lsr_value(self.a);
			},

			// ARR
			0x6b => {
				self.and(memory, AddressingMode::Immediate);
				self.a = self.ror_value(self.a);
			},

			// AXS
			0xcb => {
				let operand = memory.read8(self.pc);
				self.pc = self.pc.wrapping_add(1);
				self.x &= self.a;
				self.set_flag(Flag::C, self.x >= operand);
				self.x = self.x.wrapping_sub(operand);
				self.set_n_flag(self.x);
				self.set_z_flag(self.x);
			},*/

			0x09 => Self::format_immediate(cpu, memory, "ORA"),
			0x05 => Self::format_zero_page(cpu, memory, "ORA"),
			0x15 => Self::format_zero_page_x(cpu, memory, "ORA"),
			0x0d => Self::format_absolute(cpu, memory, "ORA"),
			0x1d => Self::format_absolute_x(cpu, memory, "ORA"),
			0x19 => Self::format_absolute_y(cpu, memory, "ORA"),
			0x01 => Self::format_indirect_x(cpu, memory, "ORA"),
			0x11 => Self::format_indirect_y(cpu, memory, "ORA"),

			0x49 => Self::format_immediate(cpu, memory, "EOR"),
			0x45 => Self::format_zero_page(cpu, memory, "EOR"),
			0x55 => Self::format_zero_page_x(cpu, memory, "EOR"),
			0x4d => Self::format_absolute(cpu, memory, "EOR"),
			0x5d => Self::format_absolute_x(cpu, memory, "EOR"),
			0x59 => Self::format_absolute_y(cpu, memory, "EOR"),
			0x41 => Self::format_indirect_x(cpu, memory, "EOR"),
			0x51 => Self::format_indirect_y(cpu, memory, "EOR"),

			0x24 => Self::format_zero_page(cpu, memory, "BIT"),
			0x2c => Self::format_absolute(cpu, memory, "BIT"),

			0x4a => Self::format_a("LSR"),
			0x46 => Self::format_zero_page(cpu, memory, "LSR"),
			0x56 => Self::format_zero_page_x(cpu, memory, "LSR"),
			0x4e => Self::format_absolute(cpu, memory, "LSR"),
			0x5e => Self::format_absolute_x(cpu, memory, "LSR"),

			0x47 => Self::format_zero_page(cpu, memory, "*SRE"),
			0x57 => Self::format_zero_page_x(cpu, memory, "*SRE"),
			0x4f => Self::format_absolute(cpu, memory, "*SRE"),
			0x5f => Self::format_absolute_x(cpu, memory, "*SRE"),
			0x5b => Self::format_absolute_y(cpu, memory, "*SRE"),
			0x43 => Self::format_indirect_x(cpu, memory, "*SRE"),
			0x53 => Self::format_indirect_y(cpu, memory, "*SRE"),		

			0x0a => Self::format_a("ASL"),
			0x06 => Self::format_zero_page(cpu, memory, "ASL"),
			0x16 => Self::format_zero_page_x(cpu, memory, "ASL"),
			0x0e => Self::format_absolute(cpu, memory, "ASL"),
			0x1e => Self::format_absolute_x(cpu, memory, "ASL"),

			0x07 => Self::format_zero_page(cpu, memory, "*SLO"),
			0x17 => Self::format_zero_page_x(cpu, memory, "*SLO"),
			0x0f => Self::format_absolute(cpu, memory, "*SLO"),
			0x1f => Self::format_absolute_x(cpu, memory, "*SLO"),
			0x1b => Self::format_absolute_y(cpu, memory, "*SLO"),
			0x03 => Self::format_indirect_x(cpu, memory, "*SLO"),
			0x13 => Self::format_indirect_y(cpu, memory, "*SLO"),

			0x6a => Self::format_a("ROR"),
			0x66 => Self::format_zero_page(cpu, memory, "ROR"),
			0x76 => Self::format_zero_page_x(cpu, memory, "ROR"),
			0x6e => Self::format_absolute(cpu, memory, "ROR"),
			0x7e => Self::format_absolute_x(cpu, memory, "ROR"),

			0x67 => Self::format_zero_page(cpu, memory, "*RRA"),
			0x77 => Self::format_zero_page_x(cpu, memory, "*RRA"),
			0x6f => Self::format_absolute(cpu, memory, "*RRA"),
			0x7f => Self::format_absolute_x(cpu, memory, "*RRA"),
			0x7b => Self::format_absolute_y(cpu, memory, "*RRA"),
			0x63 => Self::format_indirect_x(cpu, memory, "*RRA"),
			0x73 => Self::format_indirect_y(cpu, memory, "*RRA"),

			0x2a => Self::format_a("ROL"),
			0x26 => Self::format_zero_page(cpu, memory, "ROL"),
			0x36 => Self::format_zero_page_x(cpu, memory, "ROL"),
			0x2e => Self::format_absolute(cpu, memory, "ROL"),
			0x3e => Self::format_absolute_x(cpu, memory, "ROL"),

			0x27 => Self::format_zero_page(cpu, memory, "*RLA"), 
			0x37 => Self::format_zero_page_x(cpu, memory, "*RLA"),
			0x2f => Self::format_absolute(cpu, memory, "*RLA"),
			0x3f => Self::format_absolute_x(cpu, memory, "*RLA"),
			0x3b => Self::format_absolute_y(cpu, memory, "*RLA"),
			0x23 => Self::format_indirect_x(cpu, memory, "*RLA"),
			0x33 => Self::format_indirect_y(cpu, memory, "*RLA"),

			0x69 => Self::format_immediate(cpu, memory, "ADC"),
			0x65 => Self::format_zero_page(cpu, memory, "ADC"),
			0x75 => Self::format_zero_page_x(cpu, memory, "ADC"),
			0x6d => Self::format_absolute(cpu, memory, "ADC"),
			0x7d => Self::format_absolute_x(cpu, memory, "ADC"),
			0x79 => Self::format_absolute_y(cpu, memory, "ADC"),
			0x61 => Self::format_indirect_x(cpu, memory, "ADC"),
			0x71 => Self::format_indirect_y(cpu, memory, "ADC"),

			0xe9 => Self::format_immediate(cpu, memory, "SBC"),
			0xe5 => Self::format_zero_page(cpu, memory, "SBC"),
			0xf5 => Self::format_zero_page_x(cpu, memory, "SBC"),
			0xed => Self::format_absolute(cpu, memory, "SBC"),
			0xfd => Self::format_absolute_x(cpu, memory, "SBC"),
			0xf9 => Self::format_absolute_y(cpu, memory, "SBC"),
			0xe1 => Self::format_indirect_x(cpu, memory, "SBC"),
			0xf1 => Self::format_indirect_y(cpu, memory, "SBC"),
			
			0xeb => Self::format_immediate(cpu, memory, "*SBC"),

			0xe8 => Self::format("INX"),
			0xc8 => Self::format("INY"),

			0xe6 => Self::format_zero_page(cpu, memory, "INC"),
			0xf6 => Self::format_zero_page_x(cpu, memory, "INC"),
			0xee => Self::format_absolute(cpu, memory, "INC"),
			0xfe => Self::format_absolute_x(cpu, memory, "INC"),

			0xe7 => Self::format_zero_page(cpu, memory, "*ISB"),
			0xf7 => Self::format_zero_page_x(cpu, memory, "*ISB"),
			0xef => Self::format_absolute(cpu, memory, "*ISB"),
			0xff => Self::format_absolute_x(cpu, memory, "*ISB"),
			0xfb => Self::format_absolute_y(cpu, memory, "*ISB"),
			0xe3 => Self::format_indirect_x(cpu, memory, "*ISB"),
			0xf3 => Self::format_indirect_y(cpu, memory, "*ISB"),

			0xca => Self::format("DEX"),
			0x88 => Self::format("DEY"),

			0xc6 => Self::format_zero_page(cpu, memory, "DEC"),
			0xd6 => Self::format_zero_page_x(cpu, memory, "DEC"),
			0xce => Self::format_absolute(cpu, memory, "DEC"),
			0xde => Self::format_absolute_x(cpu, memory, "DEC"),

			0xc7 => Self::format_zero_page(cpu, memory, "*DCP"),
			0xd7 => Self::format_zero_page_x(cpu, memory, "*DCP"),
			0xcf => Self::format_absolute(cpu, memory, "*DCP"),
			0xdf => Self::format_absolute_x(cpu, memory, "*DCP"),
			0xdb => Self::format_absolute_y(cpu, memory, "*DCP"),
			0xc3 => Self::format_indirect_x(cpu, memory, "*DCP"),
			0xd3 => Self::format_indirect_y(cpu, memory, "*DCP"),

			0xe0 => Self::format_immediate(cpu, memory, "CPX"),
			0xe4 => Self::format_zero_page(cpu, memory, "CPX"),
			0xec => Self::format_absolute(cpu, memory, "CPX"),

			0xc0 => Self::format_immediate(cpu, memory, "CPY"),
			0xc4 => Self::format_zero_page(cpu, memory, "CPY"),
			0xcc => Self::format_absolute(cpu, memory, "CPY"),

			0xc9 => Self::format_immediate(cpu, memory, "CMP"),
			0xc5 => Self::format_zero_page(cpu, memory, "CMP"),
			0xd5 => Self::format_zero_page_x(cpu, memory, "CMP"),
			0xcd => Self::format_absolute(cpu, memory, "CMP"),
			0xdd => Self::format_absolute_x(cpu, memory, "CMP"),
			0xd9 => Self::format_absolute_y(cpu, memory, "CMP"),
			0xc1 => Self::format_indirect_x(cpu, memory, "CMP"),
			0xd1 => Self::format_indirect_y(cpu, memory, "CMP"),

			0x48 => Self::format("PHA"),
			0x68 => Self::format("PLA"),
			0x08 => Self::format("PHP"),
			0x28 => Self::format("PLP"),
			0x18 => Self::format("CLC"),
			0x38 => Self::format("SEC"),
		//	0x58 => Self::format("CLI"),
			0x78 => Self::format("SEI"),
			0xd8 => Self::format("CLD"),
			0xf8 => Self::format("SED"),
			0xb8 => Self::format("CLV"),

			0x4c => Self::format_jump_absolute(cpu, memory, "JMP"),
			0x20 => Self::format_jump_absolute(cpu, memory, "JSR"),

			// JMP (indirect)
			0x6c => {
				let intermediate_address = memory.read16(cpu.pc.wrapping_add(1));
				let low_byte = memory.read8(intermediate_address) as u16;
				let high_byte = if (intermediate_address & 0xff) == 0xff  {
					memory.read8(intermediate_address & 0xff00)
				} else {
					memory.read8(intermediate_address + 1)
				} as u16;
				let address = (high_byte << 8) | low_byte;
				format!("{:02X} {:02X}  JMP (${:04X}) = {:04X}", low_byte, high_byte, intermediate_address, address)
			},

			0x10 => Self::format_jump_relative(cpu, memory, "BPL"),
			0x30 => Self::format_jump_relative(cpu, memory, "BMI"),
			0x50 => Self::format_jump_relative(cpu, memory, "BVC"),
			0x70 => Self::format_jump_relative(cpu, memory, "BVS"),
			0x90 => Self::format_jump_relative(cpu, memory, "BCC"),
			0xb0 => Self::format_jump_relative(cpu, memory, "BCS"),
			0xd0 => Self::format_jump_relative(cpu, memory, "BNE"),
			0xf0 => Self::format_jump_relative(cpu, memory, "BEQ"),

			// BRK
			/*0x00 => {
				let address = self.pc.wrapping_add(1);
				self.push(memory, (address >> 8) as _);
				self.push(memory, address as _);
				self.push(memory, self.p | 0b0001_0000);
				self.pc = memory.read16(IRQ_VECTOR_ADDRESS);
				self.set_flag(Flag::I, true);
			},*/

			0x40 => Self::format("RTI"),
			0x60 => Self::format("RTS"),

			_ => {
				println!("[ERROR] Unknown opcode {:02X} at {:04X}", opcode, cpu.pc);
				std::process::exit(1);
			}
		};
		write!(self.file.borrow_mut(), "{:04X}  {:02X} {:<38} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}\n", cpu.pc, opcode, instruction_string, cpu.a, cpu.x, cpu.y, cpu.p, cpu.s).unwrap();
	}
}
