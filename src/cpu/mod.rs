mod memory;
mod addressing_modes;

#[cfg(test)]
mod tests;

#[cfg(feature = "trace")]
mod logger;

use emulator::*;
use self::memory::*;
use self::addressing_modes::*;

#[cfg(feature = "trace")]
use self::logger::*;

const STACK_ADDRESS: u16 = 0x100;
const NMI_VECTOR_ADDRESS: u16 = 0xfffa;
const RESET_VECTOR_ADDRESS: u16 = 0xfffc;
const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;

enum Flag {
	C = 1 << 0,
	Z = 1 << 1,
	I = 1 << 2,
	D = 1 << 3,
	B = 1 << 4,
	V = 1 << 6,
	N = 1 << 7
}

#[derive(PartialEq)]
pub enum Interrupt {
	Irq,
	Nmi
}

pub struct Cpu {
	a: u8,
	x: u8,
	y: u8,
	pc: u16,
	s: u8,
	p: u8,
	pending_interrupt: Option<Interrupt>,

	#[cfg(feature = "trace")]
	logger: Logger
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			a: 0,
			x: 0,
			y: 0,
			pc: 0,
			s: 0xfd,
			p: 0x24,
			pending_interrupt: None,

			#[cfg(feature = "trace")]
			logger: Logger::new()
		}
	}

	pub fn init_pc(emulator: &mut Emulator) {
		let value = read16(emulator, RESET_VECTOR_ADDRESS);
		emulator.cpu.set_pc(value);
	}

	fn set_pc(&mut self, value: u16) {
		info!("PC: {:04X}", value);
		self.pc = value;
	}

	fn get_flag(&self, flag: Flag) -> bool {
		(self.p & flag as u8) != 0
	}

	fn set_flag(&mut self, flag: Flag, value: bool) {
		if value {
			self.p |= flag as u8;
		} else {
			self.p &= !(flag as u8);
		}
	}

	fn set_n_flag(&mut self, value: u8) {
		self.set_flag(Flag::N, value >= 0x80);
	}

	fn set_z_flag(&mut self, value: u8) {
		self.set_flag(Flag::Z, value == 0);
	}

	fn lsr_value(&mut self, mut value: u8) -> u8 {
		self.set_flag(Flag::C, (value & 1) == 1);
		value >>= 1;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn asl_value(&mut self, mut value: u8) -> u8 {
		self.set_flag(Flag::C, (value >> 7) == 1);
		value <<= 1;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn ror_value(&mut self, mut value: u8) -> u8 {
		let c = self.get_flag(Flag::C) as u8;
		self.set_flag(Flag::C, (value & 1) == 1);
		value = (c << 7) | (value >> 1);
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn rol_value(&mut self, mut value: u8) -> u8 {
		let c = self.get_flag(Flag::C) as u8;
		self.set_flag(Flag::C, (value >> 7) == 1);
		value = (value << 1) | c;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn adc_value(&mut self, value: u8) {
		let carry = self.get_flag(Flag::C);
		let sum = self.a as u16 + value as u16 + carry as u16;
		self.set_flag(Flag::C, sum > 0xff);
		let result = sum as u8;
		self.set_flag(Flag::V, ((self.a ^ result) & (value ^ result) & 0x80) != 0);
		self.a = result;
		self.set_n_flag(self.a);
		self.set_z_flag(self.a);
	}

	pub fn request_interrupt(&mut self, interrupt: Interrupt) {
		if self.pending_interrupt != Some(Interrupt::Nmi) {
			self.pending_interrupt = Some(interrupt);
		}
	}

	pub fn execute_next_instruction(emulator: &mut Emulator) {
		match emulator.cpu.pending_interrupt {
			Some(Interrupt::Nmi) => perform_interrupt(emulator, NMI_VECTOR_ADDRESS),
			Some(Interrupt::Irq) => if !emulator.cpu.get_flag(Flag::I) {
				perform_interrupt(emulator, IRQ_VECTOR_ADDRESS);
			},
			None => {}
		}

		#[cfg(feature = "trace")]
		Logger::create_trace(emulator);
		
		let opcode = read8(emulator, emulator.cpu.pc);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
		
		match opcode {
			// NOPs
			0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xea | 0xfa => {},
			0x04 | 0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 | 0xd4 | 0xf4 => emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1),
			0x0c | 0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2),

			0xa9 => lda::<Immediate>(emulator),
			0xa5 => lda::<ZeroPage>(emulator),
			0xb5 => lda::<ZeroPageX>(emulator),
			0xad => lda::<Absolute>(emulator),
			0xbd => lda::<AbsoluteX>(emulator),
			0xb9 => lda::<AbsoluteY>(emulator),
			0xa1 => lda::<IndirectX>(emulator),
			0xb1 => lda::<IndirectY>(emulator),

			0xa2 => ldx::<Immediate>(emulator),
			0xa6 => ldx::<ZeroPage>(emulator),
			0xb6 => ldx::<ZeroPageY>(emulator),
			0xae => ldx::<Absolute>(emulator),
			0xbe => ldx::<AbsoluteY>(emulator),

			0xa0 => ldy::<Immediate>(emulator),
			0xa4 => ldy::<ZeroPage>(emulator),
			0xb4 => ldy::<ZeroPageX>(emulator),
			0xac => ldy::<Absolute>(emulator),
			0xbc => ldy::<AbsoluteX>(emulator),

			0xab => lax::<Immediate>(emulator),
			0xa7 => lax::<ZeroPage>(emulator),
			0xb7 => lax::<ZeroPageY>(emulator),
			0xaf => lax::<Absolute>(emulator),
			0xbf => lax::<AbsoluteY>(emulator),
			0xa3 => lax::<IndirectX>(emulator),
			0xb3 => lax::<IndirectY>(emulator),

			0x85 => sta::<ZeroPage>(emulator),
			0x95 => sta::<ZeroPageX>(emulator),
			0x8d => sta::<Absolute>(emulator),
			0x9d => sta::<AbsoluteX>(emulator),
			0x99 => sta::<AbsoluteY>(emulator),
			0x81 => sta::<IndirectX>(emulator),
			0x91 => sta::<IndirectY>(emulator),

			0x86 => stx::<ZeroPage>(emulator),
			0x96 => stx::<ZeroPageY>(emulator),
			0x8e => stx::<Absolute>(emulator),

			0x84 => sty::<ZeroPage>(emulator),
			0x94 => sty::<ZeroPageX>(emulator),
			0x8c => sty::<Absolute>(emulator),

			0x87 => sax::<ZeroPage>(emulator),
			0x97 => sax::<ZeroPageY>(emulator),
			0x8f => sax::<Absolute>(emulator),
			0x83 => sax::<IndirectX>(emulator),

			// SXA
			// FIXME
			0x9e => {
				let address = AbsoluteY::get_address(emulator);
				write8(emulator, address, emulator.cpu.x & emulator.cpu.a);
			},

			// SYA
			// FIXME
			0x9c => {
				let address = AbsoluteX::get_address(emulator);
				write8(emulator, address, emulator.cpu.y & emulator.cpu.a);
			},

			// TAX
			0xaa => {
				emulator.cpu.x = emulator.cpu.a;
				emulator.cpu.set_n_flag(emulator.cpu.x);
				emulator.cpu.set_z_flag(emulator.cpu.x);
			},

			// TXA
			0x8a => {
				emulator.cpu.a = emulator.cpu.x;
				emulator.cpu.set_n_flag(emulator.cpu.a);
				emulator.cpu.set_z_flag(emulator.cpu.a);
			},

			// TAY
			0xa8 => {
				emulator.cpu.y = emulator.cpu.a;
				emulator.cpu.set_n_flag(emulator.cpu.y);
				emulator.cpu.set_z_flag(emulator.cpu.y);
			},

			// TYA
			0x98 => {
				emulator.cpu.a = emulator.cpu.y;
				emulator.cpu.set_n_flag(emulator.cpu.a);
				emulator.cpu.set_z_flag(emulator.cpu.a);
			},

			// TSX
			0xba => {
				emulator.cpu.x = emulator.cpu.s;
				emulator.cpu.set_n_flag(emulator.cpu.x);
				emulator.cpu.set_z_flag(emulator.cpu.x);
			},

			// TXS
			0x9a => emulator.cpu.s = emulator.cpu.x,

			0x29 => and::<Immediate>(emulator),
			0x25 => and::<ZeroPage>(emulator),
			0x35 => and::<ZeroPageX>(emulator),
			0x2d => and::<Absolute>(emulator),
			0x3d => and::<AbsoluteX>(emulator),
			0x39 => and::<AbsoluteY>(emulator),
			0x21 => and::<IndirectX>(emulator),
			0x31 => and::<IndirectY>(emulator),

			0x0b => aac(emulator),
			0x2b => aac(emulator),

			// ASR
			0x4b => {
				and::<Immediate>(emulator);
				emulator.cpu.a = emulator.cpu.lsr_value(emulator.cpu.a);
			},

			// ARR
			0x6b => {
				emulator.cpu.a &= get_operand::<Immediate>(emulator);
				let c = emulator.cpu.get_flag(Flag::C) as u8;
				emulator.cpu.a = (c << 7) | (emulator.cpu.a >> 1);
				emulator.cpu.set_flag(Flag::C, ((emulator.cpu.a >> 6) & 1) == 1);
				emulator.cpu.set_flag(Flag::V, (((emulator.cpu.a >> 6) & 1) ^ ((emulator.cpu.a >> 5) & 1)) == 1);
				emulator.cpu.set_n_flag(emulator.cpu.a);
				emulator.cpu.set_z_flag(emulator.cpu.a);
			},

			// AXS
			0xcb => {
				let operand = get_operand::<Immediate>(emulator);
				emulator.cpu.x &= emulator.cpu.a;
				emulator.cpu.set_flag(Flag::C, emulator.cpu.x >= operand);
				emulator.cpu.x = emulator.cpu.x.wrapping_sub(operand);
				emulator.cpu.set_n_flag(emulator.cpu.x);
				emulator.cpu.set_z_flag(emulator.cpu.x);
			},

			0x09 => ora::<Immediate>(emulator),
			0x05 => ora::<ZeroPage>(emulator),
			0x15 => ora::<ZeroPageX>(emulator),
			0x0d => ora::<Absolute>(emulator),
			0x1d => ora::<AbsoluteX>(emulator),
			0x19 => ora::<AbsoluteY>(emulator),
			0x01 => ora::<IndirectX>(emulator),
			0x11 => ora::<IndirectY>(emulator),

			0x49 => eor::<Immediate>(emulator),
			0x45 => eor::<ZeroPage>(emulator),
			0x55 => eor::<ZeroPageX>(emulator),
			0x4d => eor::<Absolute>(emulator),
			0x5d => eor::<AbsoluteX>(emulator),
			0x59 => eor::<AbsoluteY>(emulator),
			0x41 => eor::<IndirectX>(emulator),
			0x51 => eor::<IndirectY>(emulator),

			0x24 => bit::<ZeroPage>(emulator),
			0x2c => bit::<Absolute>(emulator),

			0x4a => emulator.cpu.a = emulator.cpu.lsr_value(emulator.cpu.a),
			0x46 => lsr::<ZeroPage>(emulator),
			0x56 => lsr::<ZeroPageX>(emulator),
			0x4e => lsr::<Absolute>(emulator),
			0x5e => lsr::<AbsoluteX>(emulator),

			0x47 => sre::<ZeroPage>(emulator),
			0x57 => sre::<ZeroPageX>(emulator),
			0x4f => sre::<Absolute>(emulator),
			0x5f => sre::<AbsoluteX>(emulator),
			0x5b => sre::<AbsoluteY>(emulator),
			0x43 => sre::<IndirectX>(emulator),
			0x53 => sre::<IndirectY>(emulator),

			0x0a => emulator.cpu.a = emulator.cpu.asl_value(emulator.cpu.a),
			0x06 => asl::<ZeroPage>(emulator),
			0x16 => asl::<ZeroPageX>(emulator),
			0x0e => asl::<Absolute>(emulator),
			0x1e => asl::<AbsoluteX>(emulator),

			0x07 => slo::<ZeroPage>(emulator),
			0x17 => slo::<ZeroPageX>(emulator),
			0x0f => slo::<Absolute>(emulator),
			0x1f => slo::<AbsoluteX>(emulator),
			0x1b => slo::<AbsoluteY>(emulator),
			0x03 => slo::<IndirectX>(emulator),
			0x13 => slo::<IndirectY>(emulator),

			0x6a => emulator.cpu.a = emulator.cpu.ror_value(emulator.cpu.a),
			0x66 => ror::<ZeroPage>(emulator),
			0x76 => ror::<ZeroPageX>(emulator),
			0x6e => ror::<Absolute>(emulator),
			0x7e => ror::<AbsoluteX>(emulator),

			0x67 => rra::<ZeroPage>(emulator),
			0x77 => rra::<ZeroPageX>(emulator),
			0x6f => rra::<Absolute>(emulator),
			0x7f => rra::<AbsoluteX>(emulator),
			0x7b => rra::<AbsoluteY>(emulator),
			0x63 => rra::<IndirectX>(emulator),
			0x73 => rra::<IndirectY>(emulator),

			0x2a => emulator.cpu.a = emulator.cpu.rol_value(emulator.cpu.a),
			0x26 => rol::<ZeroPage>(emulator),
			0x36 => rol::<ZeroPageX>(emulator),
			0x2e => rol::<Absolute>(emulator),
			0x3e => rol::<AbsoluteX>(emulator),

			0x27 => rla::<ZeroPage>(emulator),
			0x37 => rla::<ZeroPageX>(emulator),
			0x2f => rla::<Absolute>(emulator),
			0x3f => rla::<AbsoluteX>(emulator),
			0x3b => rla::<AbsoluteY>(emulator),
			0x23 => rla::<IndirectX>(emulator),
			0x33 => rla::<IndirectY>(emulator),

			0x69 => adc::<Immediate>(emulator),
			0x65 => adc::<ZeroPage>(emulator),
			0x75 => adc::<ZeroPageX>(emulator),
			0x6d => adc::<Absolute>(emulator),
			0x7d => adc::<AbsoluteX>(emulator),
			0x79 => adc::<AbsoluteY>(emulator),
			0x61 => adc::<IndirectX>(emulator),
			0x71 => adc::<IndirectY>(emulator),

			0xe9 => sbc::<Immediate>(emulator),
			0xeb => sbc::<Immediate>(emulator),
			0xe5 => sbc::<ZeroPage>(emulator),
			0xf5 => sbc::<ZeroPageX>(emulator),
			0xed => sbc::<Absolute>(emulator),
			0xfd => sbc::<AbsoluteX>(emulator),
			0xf9 => sbc::<AbsoluteY>(emulator),
			0xe1 => sbc::<IndirectX>(emulator),
			0xf1 => sbc::<IndirectY>(emulator),
			
			// INX
			0xe8 => {
				emulator.cpu.x = emulator.cpu.x.wrapping_add(1);
				emulator.cpu.set_z_flag(emulator.cpu.x);
				emulator.cpu.set_n_flag(emulator.cpu.x);
			},

			// INY
			0xc8 => {
				emulator.cpu.y = emulator.cpu.y.wrapping_add(1);
				emulator.cpu.set_z_flag(emulator.cpu.y);
				emulator.cpu.set_n_flag(emulator.cpu.y);
			},

			0xe6 => inc::<ZeroPage>(emulator),
			0xf6 => inc::<ZeroPageX>(emulator),
			0xee => inc::<Absolute>(emulator),
			0xfe => inc::<AbsoluteX>(emulator),

			0xe7 => isb::<ZeroPage>(emulator),
			0xf7 => isb::<ZeroPageX>(emulator),
			0xef => isb::<Absolute>(emulator),
			0xff => isb::<AbsoluteX>(emulator),
			0xfb => isb::<AbsoluteY>(emulator),
			0xe3 => isb::<IndirectX>(emulator),
			0xf3 => isb::<IndirectY>(emulator),

			// DEX
			0xca => {
				emulator.cpu.x = emulator.cpu.x.wrapping_sub(1);
				emulator.cpu.set_z_flag(emulator.cpu.x);
				emulator.cpu.set_n_flag(emulator.cpu.x);
			},

			// DEY
			0x88 => {
				emulator.cpu.y = emulator.cpu.y.wrapping_sub(1);
				emulator.cpu.set_n_flag(emulator.cpu.y);
				emulator.cpu.set_z_flag(emulator.cpu.y);
			},

			0xc6 => dec::<ZeroPage>(emulator),
			0xd6 => dec::<ZeroPageX>(emulator),
			0xce => dec::<Absolute>(emulator),
			0xde => dec::<AbsoluteX>(emulator),

			0xc7 => dcp::<ZeroPage>(emulator),
			0xd7 => dcp::<ZeroPageX>(emulator),
			0xcf => dcp::<Absolute>(emulator),
			0xdf => dcp::<AbsoluteX>(emulator),
			0xdb => dcp::<AbsoluteY>(emulator),
			0xc3 => dcp::<IndirectX>(emulator),
			0xd3 => dcp::<IndirectY>(emulator),

			0xe0 => cpx::<Immediate>(emulator),
			0xe4 => cpx::<ZeroPage>(emulator),
			0xec => cpx::<Absolute>(emulator),

			0xc0 => cpy::<Immediate>(emulator),
			0xc4 => cpy::<ZeroPage>(emulator),
			0xcc => cpy::<Absolute>(emulator),

			0xc9 => cmp::<Immediate>(emulator),
			0xc5 => cmp::<ZeroPage>(emulator),
			0xd5 => cmp::<ZeroPageX>(emulator),
			0xcd => cmp::<Absolute>(emulator),
			0xdd => cmp::<AbsoluteX>(emulator),
			0xd9 => cmp::<AbsoluteY>(emulator),
			0xc1 => cmp::<IndirectX>(emulator),
			0xd1 => cmp::<IndirectY>(emulator),

			// PHA
			0x48 => push8(emulator, emulator.cpu.a),

			// PLA
			0x68 => {
				emulator.cpu.a = pull8(emulator);
				emulator.cpu.set_n_flag(emulator.cpu.a);
				emulator.cpu.set_z_flag(emulator.cpu.a);
			},

			// PHP
			0x08 => push8(emulator, emulator.cpu.p | Flag::B as u8),

			0x28 => plp(emulator),

			// CLC
			0x18 => emulator.cpu.set_flag(Flag::C, false),

			// SEC
			0x38 => emulator.cpu.set_flag(Flag::C, true),

			// CLI
			0x58 => emulator.cpu.set_flag(Flag::I, false),

			// SEI
			0x78 => emulator.cpu.set_flag(Flag::I, true),

			// CLD
			0xd8 => emulator.cpu.set_flag(Flag::D, false),

			// SED
			0xf8 => emulator.cpu.set_flag(Flag::D, true),

			// CLV
			0xb8 => emulator.cpu.set_flag(Flag::V, false),

			// JMP (absolute)
			0x4c => emulator.cpu.pc = read16(emulator, emulator.cpu.pc),

			// JMP (indirect)
			0x6c => {
				let address = read16(emulator, emulator.cpu.pc);
				let low_byte = read8(emulator, address) as u16;
				let high_byte = if (address & 0xff) == 0xff  {
					read8(emulator, address & 0xff00)
				} else {
					read8(emulator, address + 1)
				} as u16;
				emulator.cpu.pc = (high_byte << 8) | low_byte;
			},

			// BPL
			0x10 => branch(emulator, Flag::N, false),

			// BMI
			0x30 => branch(emulator, Flag::N, true),

			// BVC
			0x50 => branch(emulator, Flag::V, false),

			// BVS
			0x70 => branch(emulator, Flag::V, true),

			// BCC
			0x90 => branch(emulator, Flag::C, false),

			// BCS
			0xb0 => branch(emulator, Flag::C, true),

			// BNE
			0xd0 => branch(emulator, Flag::Z, false),

			// BEQ
			0xf0 => branch(emulator, Flag::Z, true),

			// BRK
			0x00 => {
				let address = emulator.cpu.pc.wrapping_add(1);
				push16(emulator, address);
				push8(emulator, emulator.cpu.p | Flag::B as u8);
				emulator.cpu.pc = read16(emulator, IRQ_VECTOR_ADDRESS);
				emulator.cpu.set_flag(Flag::I, true);
			},

			// JSR
			0x20 => {
				let address = emulator.cpu.pc.wrapping_add(1);
				push16(emulator, address);
				emulator.cpu.pc = read16(emulator, emulator.cpu.pc);
			},

			// RTI
			0x40 => {
				plp(emulator);
				emulator.cpu.pc = pull16(emulator);
			},

			// RTS
			0x60 => emulator.cpu.pc = pull16(emulator).wrapping_add(1),

			// KIL
			0x32 => {
				error!("CPU stopped");
				panic!();
			}

			_ => {
				error!("Unknown opcode {:02X} at {:04X}", opcode, emulator.cpu.pc.wrapping_sub(1));
				panic!();
			}
		}
	}
}

fn perform_interrupt(emulator: &mut Emulator, address: u16) {
	push16(emulator, emulator.cpu.pc);
	push8(emulator, emulator.cpu.p);
	emulator.cpu.pc = read16(emulator, address);
	emulator.cpu.set_flag(Flag::I, true);
	emulator.cpu.pending_interrupt = None;
}

fn push8(emulator: &mut Emulator, value: u8) {
	write8(emulator, STACK_ADDRESS + emulator.cpu.s as u16, value);
	emulator.cpu.s = emulator.cpu.s.wrapping_sub(1);
}

fn push16(emulator: &mut Emulator, value: u16) {
	push8(emulator, (value >> 8) as _);
	push8(emulator, value as _);
}

fn pull8(emulator: &mut Emulator) -> u8 {
	emulator.cpu.s = emulator.cpu.s.wrapping_add(1);
	read8(emulator, STACK_ADDRESS + emulator.cpu.s as u16)
}

fn pull16(emulator: &mut Emulator) -> u16 {
	let low_byte = pull8(emulator) as u16;
	let high_byte = pull8(emulator) as u16;
	(high_byte << 8) | low_byte
}

fn get_operand<T: AddressingMode>(emulator: &mut Emulator) -> u8 {
	let address = T::get_address(emulator);
	read8(emulator, address)
}

fn lda<T: AddressingMode>(emulator: &mut Emulator) {
	emulator.cpu.a = get_operand::<T>(emulator);
	emulator.cpu.set_n_flag(emulator.cpu.a);
	emulator.cpu.set_z_flag(emulator.cpu.a);
}

fn ldx<T: AddressingMode>(emulator: &mut Emulator) {
	emulator.cpu.x = get_operand::<T>(emulator);
	emulator.cpu.set_n_flag(emulator.cpu.x);
	emulator.cpu.set_z_flag(emulator.cpu.x);
}

fn ldy<T: AddressingMode>(emulator: &mut Emulator) {
	emulator.cpu.y = get_operand::<T>(emulator);
	emulator.cpu.set_n_flag(emulator.cpu.y);
	emulator.cpu.set_z_flag(emulator.cpu.y);
}

fn lax<T: AddressingMode>(emulator: &mut Emulator) {
	emulator.cpu.a = get_operand::<T>(emulator);
	emulator.cpu.x = emulator.cpu.a;
	emulator.cpu.set_n_flag(emulator.cpu.x);
	emulator.cpu.set_z_flag(emulator.cpu.x);
}

fn sta<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	write8(emulator, address, emulator.cpu.a);
}

fn stx<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	write8(emulator, address, emulator.cpu.x);
}

fn sty<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	write8(emulator, address, emulator.cpu.y);
}

fn sax<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	write8(emulator, address, emulator.cpu.a & emulator.cpu.x);
}

fn and_address(emulator: &mut Emulator, address: u16) {
	emulator.cpu.a &= read8(emulator, address);
	emulator.cpu.set_n_flag(emulator.cpu.a);
	emulator.cpu.set_z_flag(emulator.cpu.a);
}

fn and<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	and_address(emulator, address);
}

fn aac(emulator: &mut Emulator) {
	and::<Immediate>(emulator);
	let n = emulator.cpu.get_flag(Flag::N);
	emulator.cpu.set_flag(Flag::C, n);
}

fn ora_address(emulator: &mut Emulator, address: u16) {
	emulator.cpu.a |= read8(emulator, address);
	emulator.cpu.set_n_flag(emulator.cpu.a);
	emulator.cpu.set_z_flag(emulator.cpu.a);
}

fn ora<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	ora_address(emulator, address);
}

fn eor_address(emulator: &mut Emulator, address: u16) {
	emulator.cpu.a ^= read8(emulator, address);
	emulator.cpu.set_n_flag(emulator.cpu.a);
	emulator.cpu.set_z_flag(emulator.cpu.a);
}

fn eor<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	eor_address(emulator, address);
}

fn bit<T: AddressingMode>(emulator: &mut Emulator) {
	let operand = get_operand::<T>(emulator);
	emulator.cpu.set_z_flag(operand & emulator.cpu.a);
	emulator.cpu.set_n_flag(operand);
	emulator.cpu.set_flag(Flag::V, ((operand >> 6) & 1) == 1);
}

fn lsr_address(emulator: &mut Emulator, address: u16) {
	let operand = read8(emulator, address);
	let result = emulator.cpu.lsr_value(operand);
	write8(emulator, address, result);
}

fn lsr<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	lsr_address(emulator, address);
}

fn sre<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	lsr_address(emulator, address);
	eor_address(emulator, address);
}

fn asl_address(emulator: &mut Emulator, address: u16) {
	let operand = read8(emulator, address);
	let result = emulator.cpu.asl_value(operand);
	write8(emulator, address, result);
}

fn asl<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	asl_address(emulator, address);
}

fn slo<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	asl_address(emulator, address);
	ora_address(emulator, address);
}

fn ror_address(emulator: &mut Emulator, address: u16) {
	let operand = read8(emulator, address);
	let result = emulator.cpu.ror_value(operand);
	write8(emulator, address, result);
}

fn ror<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	ror_address(emulator, address);
}

fn rra<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	ror_address(emulator, address);
	adc_address(emulator, address);
}

fn rol_address(emulator: &mut Emulator, address: u16) {
	let operand = read8(emulator, address);
	let result = emulator.cpu.rol_value(operand);
	write8(emulator, address, result);
}

fn rol<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	rol_address(emulator, address);
}

fn rla<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	rol_address(emulator, address);
	and_address(emulator, address);
}

fn adc_address(emulator: &mut Emulator, address: u16) {
	let operand = read8(emulator, address);
	emulator.cpu.adc_value(operand);
}

fn adc<T: AddressingMode>(emulator: &mut Emulator) {
	let operand = get_operand::<T>(emulator);
	emulator.cpu.adc_value(operand);
}

fn sbc_address(emulator: &mut Emulator, address: u16) {
	let operand = read8(emulator, address);
	emulator.cpu.adc_value(!operand);
}

fn sbc<T: AddressingMode>(emulator: &mut Emulator) {
	let operand = get_operand::<T>(emulator);
	emulator.cpu.adc_value(!operand);
}

fn inc_address(emulator: &mut Emulator, address: u16) {
	let result = read8(emulator, address).wrapping_add(1);
	write8(emulator, address, result);
	emulator.cpu.set_z_flag(result);
	emulator.cpu.set_n_flag(result);
}

fn inc<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	inc_address(emulator, address);
}

fn isb<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	inc_address(emulator, address);
	sbc_address(emulator, address);
}

fn dec_address(emulator: &mut Emulator, address: u16) {
	let result = read8(emulator, address).wrapping_sub(1);
	write8(emulator, address, result);
	emulator.cpu.set_z_flag(result);
	emulator.cpu.set_n_flag(result);
}

fn dec<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	dec_address(emulator, address);
}

fn dcp<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	dec_address(emulator, address);
	cmp_address(emulator, address);
}

fn cpx<T: AddressingMode>(emulator: &mut Emulator) {
	let operand = get_operand::<T>(emulator);
	emulator.cpu.set_flag(Flag::C, emulator.cpu.x >= operand);
	let result = emulator.cpu.x.wrapping_sub(operand);
	emulator.cpu.set_z_flag(result);
	emulator.cpu.set_n_flag(result);
}

fn cpy<T: AddressingMode>(emulator: &mut Emulator) {
	let operand = get_operand::<T>(emulator);
	emulator.cpu.set_flag(Flag::C, emulator.cpu.y >= operand);
	let result = emulator.cpu.y.wrapping_sub(operand);
	emulator.cpu.set_z_flag(result);
	emulator.cpu.set_n_flag(result);
}

fn cmp_address(emulator: &mut Emulator, address: u16) {
	let operand = read8(emulator, address);
	emulator.cpu.set_flag(Flag::C, emulator.cpu.a >= operand);
	let result = emulator.cpu.a.wrapping_sub(operand);
	emulator.cpu.set_z_flag(result);
	emulator.cpu.set_n_flag(result);
}

fn cmp<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	cmp_address(emulator, address);
}

fn branch(emulator: &mut Emulator, flag: Flag, value: bool) {
	if emulator.cpu.get_flag(flag) == value {
		let offset = read8(emulator, emulator.cpu.pc) as i8;
		emulator.cpu.pc = if offset > 0 {
			emulator.cpu.pc.wrapping_add(offset as _)
		} else {
			emulator.cpu.pc.wrapping_sub(offset.wrapping_neg() as _)
		};
	}
	emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
}

fn plp(emulator: &mut Emulator) {
	let value = pull8(emulator);
	emulator.cpu.p = (value | 0x20) & !(Flag::B as u8);
}
