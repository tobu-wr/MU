mod memory;

#[cfg(feature = "log")]
mod logger;

use emulator::*;
use self::memory::*;

#[cfg(feature = "log")]
use self::logger::*;

enum Flag {
	C = 1 << 0,
	Z = 1 << 1,
	I = 1 << 2,
	D = 1 << 3,
	B = 1 << 4,
	V = 1 << 6,
	N = 1 << 7
}

#[derive(Clone, Copy)]
enum AddressingMode {
	Immediate,
	ZeroPage,
	ZeroPageX,
	ZeroPageY,
	Absolute,
	AbsoluteX,
	AbsoluteY,
	IndirectX,
	IndirectY
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

	#[cfg(feature = "log")]
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

			#[cfg(feature = "log")]
			logger: Logger::new()
		}
	}

	pub fn init_pc(emulator: &mut Emulator) {
		let value = read16(emulator, RESET_VECTOR_ADDRESS);
		emulator.cpu.set_pc(value);
	}

	pub fn set_pc(&mut self, value: u16) {
		println!("[INFO] PC: {:04X}", value);
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

	fn push8(emulator: &mut Emulator, value: u8) {
		write8(emulator, STACK_ADDRESS + emulator.cpu.s as u16, value);
		emulator.cpu.s = emulator.cpu.s.wrapping_sub(1);
	}

	fn push16(emulator: &mut Emulator, value: u16) {
		Self::push8(emulator, (value >> 8) as _);
		Self::push8(emulator, value as _);
	}

	fn pull8(emulator: &mut Emulator) -> u8 {
		emulator.cpu.s = emulator.cpu.s.wrapping_add(1);
		read8(emulator, STACK_ADDRESS + emulator.cpu.s as u16)
	}

	fn pull16(emulator: &mut Emulator) -> u16 {
		let low_byte = Self::pull8(emulator) as u16;
		let high_byte = Self::pull8(emulator) as u16;
		(high_byte << 8) | low_byte
	}

	fn plp(emulator: &mut Emulator) {
		let value = Self::pull8(emulator);
		emulator.cpu.p = (value | 0b0010_0000) & !(Flag::B as u8);
	}

	fn get_address(emulator: &mut Emulator, addressing_mode: AddressingMode) -> u16 {
		match addressing_mode {
			AddressingMode::Immediate => {
				let address = emulator.cpu.pc;
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
				address
			},
			AddressingMode::ZeroPage => {
				let address = read8(emulator, emulator.cpu.pc);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
				address as _
			},
			AddressingMode::ZeroPageX => {
				let address = read8(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.x);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
				address as _
			},
			AddressingMode::ZeroPageY => {
				let address = read8(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.y);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
				address as _
			},
			AddressingMode::Absolute => {
				let address = read16(emulator, emulator.cpu.pc);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2);
				address
			},
			AddressingMode::AbsoluteX => {
				let address = read16(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.x as _);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2);
				address
			},
			AddressingMode::AbsoluteY => {
				let address = read16(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.y as _);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2);
				address
			},
			AddressingMode::IndirectX => {
				let address = read8(emulator, emulator.cpu.pc).wrapping_add(emulator.cpu.x);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
				let low_byte = read8(emulator, address as _) as u16;
				let high_byte = read8(emulator, address.wrapping_add(1) as _) as u16;
				(high_byte << 8) | low_byte
			},
			AddressingMode::IndirectY => {
				let address = read8(emulator, emulator.cpu.pc);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
				let low_byte = read8(emulator, address as _) as u16;
				let high_byte = read8(emulator, address.wrapping_add(1) as _) as u16;
				let value = (high_byte << 8) | low_byte;
				value.wrapping_add(emulator.cpu.y as _)
			}
		}
	}

	fn get_operand(emulator: &mut Emulator, addressing_mode: AddressingMode) -> u8 {
		let address = Self::get_address(emulator, addressing_mode);
		read8(emulator, address)
	}

	fn lda(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		emulator.cpu.a = Self::get_operand(emulator, addressing_mode);
		emulator.cpu.set_n_flag(emulator.cpu.a);
		emulator.cpu.set_z_flag(emulator.cpu.a);
	}

	fn ldx(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		emulator.cpu.x = Self::get_operand(emulator, addressing_mode);
		emulator.cpu.set_n_flag(emulator.cpu.x);
		emulator.cpu.set_z_flag(emulator.cpu.x);
	}

	fn ldy(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		emulator.cpu.y = Self::get_operand(emulator, addressing_mode);
		emulator.cpu.set_n_flag(emulator.cpu.y);
		emulator.cpu.set_z_flag(emulator.cpu.y);
	}

	fn lax(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		emulator.cpu.a = Self::get_operand(emulator, addressing_mode);
		emulator.cpu.x = emulator.cpu.a;
		emulator.cpu.set_n_flag(emulator.cpu.x);
		emulator.cpu.set_z_flag(emulator.cpu.x);
	}

	fn sta(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		write8(emulator, address, emulator.cpu.a);
	}

	fn stx(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		write8(emulator, address, emulator.cpu.x);
	}

	fn sty(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		write8(emulator, address, emulator.cpu.y);
	}

	fn sax(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		write8(emulator, address, emulator.cpu.a & emulator.cpu.x);
	}

	fn and_address(emulator: &mut Emulator, address: u16) {
		emulator.cpu.a &= read8(emulator, address);
		emulator.cpu.set_n_flag(emulator.cpu.a);
		emulator.cpu.set_z_flag(emulator.cpu.a);
	}

	fn and(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::and_address(emulator, address);
	}

	/*fn aac(emulator: &mut Emulator) {
		Self::and(emulator, AddressingMode::Immediate);
		let n = emulator.cpu.get_flag(Flag::N);
		emulator.cpu.set_flag(Flag::C, n);
	}*/

	fn ora_address(emulator: &mut Emulator, address: u16) {
		emulator.cpu.a |= read8(emulator, address);
		emulator.cpu.set_n_flag(emulator.cpu.a);
		emulator.cpu.set_z_flag(emulator.cpu.a);
	}

	fn ora(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::ora_address(emulator, address);
	}

	fn eor_address(emulator: &mut Emulator, address: u16) {
		emulator.cpu.a ^= read8(emulator, address);
		emulator.cpu.set_n_flag(emulator.cpu.a);
		emulator.cpu.set_z_flag(emulator.cpu.a);
	}

	fn eor(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::eor_address(emulator, address);
	}

	fn bit(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let operand = Self::get_operand(emulator, addressing_mode);
		emulator.cpu.set_z_flag(operand & emulator.cpu.a);
		emulator.cpu.set_n_flag(operand);
		emulator.cpu.set_flag(Flag::V, ((operand >> 6) & 1) == 1);
	}

	fn lsr_value(&mut self, mut value: u8) -> u8 {
		self.set_flag(Flag::C, (value & 1) == 1);
		value >>= 1;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn lsr_address(emulator: &mut Emulator, address: u16) {
		let operand = read8(emulator, address);
		let result = emulator.cpu.lsr_value(operand);
		write8(emulator, address, result);
	}

	fn lsr(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::lsr_address(emulator, address);
	}

	fn sre(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::lsr_address(emulator, address);
		Self::eor_address(emulator, address);
	}

	fn asl_value(&mut self, mut value: u8) -> u8 {
		self.set_flag(Flag::C, (value >> 7) == 1);
		value <<= 1;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn asl_address(emulator: &mut Emulator, address: u16) {
		let operand = read8(emulator, address);
		let result = emulator.cpu.asl_value(operand);
		write8(emulator, address, result);
	}

	fn asl(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::asl_address(emulator, address);
	}

	fn slo(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::asl_address(emulator, address);
		Self::ora_address(emulator, address);
	}

	fn ror_value(&mut self, mut value: u8) -> u8 {
		let c = self.get_flag(Flag::C) as u8;
		self.set_flag(Flag::C, (value & 1) == 1);
		value = (c << 7) | (value >> 1);
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn ror_address(emulator: &mut Emulator, address: u16) {
		let operand = read8(emulator, address);
		let result = emulator.cpu.ror_value(operand);
		write8(emulator, address, result);
	}

	fn ror(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::ror_address(emulator, address);
	}

	fn rra(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::ror_address(emulator, address);
		Self::adc_address(emulator, address);
	}

	fn rol_value(&mut self, mut value: u8) -> u8 {
		let c = self.get_flag(Flag::C) as u8;
		self.set_flag(Flag::C, (value >> 7) == 1);
		value = (value << 1) | c;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn rol_address(emulator: &mut Emulator, address: u16) {
		let operand = read8(emulator, address);
		let result = emulator.cpu.rol_value(operand);
		write8(emulator, address, result);
	}

	fn rol(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::rol_address(emulator, address);
	}

	fn rla(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::rol_address(emulator, address);
		Self::and_address(emulator, address);
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

	fn adc_address(emulator: &mut Emulator, address: u16) {
		let operand = read8(emulator, address);
		emulator.cpu.adc_value(operand);
	}

	fn adc(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let operand = Self::get_operand(emulator, addressing_mode);
		emulator.cpu.adc_value(operand);
	}

	fn sbc_address(emulator: &mut Emulator, address: u16) {
		let operand = read8(emulator, address);
		emulator.cpu.adc_value(!operand);
	}

	fn sbc(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let operand = Self::get_operand(emulator, addressing_mode);
		emulator.cpu.adc_value(!operand);
	}

	fn inc_address(emulator: &mut Emulator, address: u16) {
		let result = read8(emulator, address).wrapping_add(1);
		write8(emulator, address, result);
		emulator.cpu.set_z_flag(result);
		emulator.cpu.set_n_flag(result);
	}

	fn inc(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::inc_address(emulator, address);
	}

	fn isb(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::inc_address(emulator, address);
		Self::sbc_address(emulator, address);
	}

	fn dec_address(emulator: &mut Emulator, address: u16) {
		let result = read8(emulator, address).wrapping_sub(1);
		write8(emulator, address, result);
		emulator.cpu.set_z_flag(result);
		emulator.cpu.set_n_flag(result);
	}

	fn dec(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::dec_address(emulator, address);
	}

	fn dcp(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::dec_address(emulator, address);
		Self::cmp_address(emulator, address);
	}

	fn cpx(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let operand = Self::get_operand(emulator, addressing_mode);
		emulator.cpu.set_flag(Flag::C, emulator.cpu.x >= operand);
		let result = emulator.cpu.x.wrapping_sub(operand);
		emulator.cpu.set_z_flag(result);
		emulator.cpu.set_n_flag(result);
	}

	fn cpy(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let operand = Self::get_operand(emulator, addressing_mode);
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

	fn cmp(emulator: &mut Emulator, addressing_mode: AddressingMode) {
		let address = Self::get_address(emulator, addressing_mode);
		Self::cmp_address(emulator, address);
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

	pub fn request_interrupt(&mut self, interrupt: Interrupt) {
		if self.pending_interrupt != Some(Interrupt::Nmi) {
			self.pending_interrupt = Some(interrupt);
		}
	}

	pub fn execute_next_instruction(emulator: &mut Emulator) {
		if emulator.cpu.pending_interrupt == Some(Interrupt::Nmi) || (emulator.cpu.pending_interrupt == Some(Interrupt::Irq) && !emulator.cpu.get_flag(Flag::I)) {
			Self::push16(emulator, emulator.cpu.pc);
			Self::push8(emulator, emulator.cpu.p);
			let address = if emulator.cpu.pending_interrupt == Some(Interrupt::Nmi) {
				NMI_VECTOR_ADDRESS
			} else {
				IRQ_VECTOR_ADDRESS
			};
			emulator.cpu.pc = read16(emulator, address);
			emulator.cpu.set_flag(Flag::I, true);
			emulator.cpu.pending_interrupt = None;
		}

		#[cfg(feature = "log")]
		emulator.cpu.logger.create_log(emulator.cpu, emulator.memory);
		
		let opcode = read8(emulator, emulator.cpu.pc);
		emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
		
		match opcode {
			// NOPs
			0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xea | 0xfa => {},
			0x04 | 0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 /*| 0x82 | 0x89 | 0xc2 | 0xe2*/ | 0xd4 | 0xf4 => emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1),
			0x0c | 0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2),

			0xa9 => Self::lda(emulator, AddressingMode::Immediate),
			0xa5 => Self::lda(emulator, AddressingMode::ZeroPage),
			0xb5 => Self::lda(emulator, AddressingMode::ZeroPageX),
			0xad => Self::lda(emulator, AddressingMode::Absolute),
			0xbd => Self::lda(emulator, AddressingMode::AbsoluteX),
			0xb9 => Self::lda(emulator, AddressingMode::AbsoluteY),
			0xa1 => Self::lda(emulator, AddressingMode::IndirectX),
			0xb1 => Self::lda(emulator, AddressingMode::IndirectY),

			0xa2 => Self::ldx(emulator, AddressingMode::Immediate),
			0xa6 => Self::ldx(emulator, AddressingMode::ZeroPage),
			0xb6 => Self::ldx(emulator, AddressingMode::ZeroPageY),
			0xae => Self::ldx(emulator, AddressingMode::Absolute),
			0xbe => Self::ldx(emulator, AddressingMode::AbsoluteY),

			0xa0 => Self::ldy(emulator, AddressingMode::Immediate),
			0xa4 => Self::ldy(emulator, AddressingMode::ZeroPage),
			0xb4 => Self::ldy(emulator, AddressingMode::ZeroPageX),
			0xac => Self::ldy(emulator, AddressingMode::Absolute),
			0xbc => Self::ldy(emulator, AddressingMode::AbsoluteX),

			//0xab => Self::lax(emulator, AddressingMode::Immediate),
			0xa7 => Self::lax(emulator, AddressingMode::ZeroPage),
			0xb7 => Self::lax(emulator, AddressingMode::ZeroPageY),
			0xaf => Self::lax(emulator, AddressingMode::Absolute),
			0xbf => Self::lax(emulator, AddressingMode::AbsoluteY),
			0xa3 => Self::lax(emulator, AddressingMode::IndirectX),
			0xb3 => Self::lax(emulator, AddressingMode::IndirectY),

			0x85 => Self::sta(emulator, AddressingMode::ZeroPage),
			0x95 => Self::sta(emulator, AddressingMode::ZeroPageX),
			0x8d => Self::sta(emulator, AddressingMode::Absolute),
			0x9d => Self::sta(emulator, AddressingMode::AbsoluteX),
			0x99 => Self::sta(emulator, AddressingMode::AbsoluteY),
			0x81 => Self::sta(emulator, AddressingMode::IndirectX),
			0x91 => Self::sta(emulator, AddressingMode::IndirectY),

			0x86 => Self::stx(emulator, AddressingMode::ZeroPage),
			0x96 => Self::stx(emulator, AddressingMode::ZeroPageY),
			0x8e => Self::stx(emulator, AddressingMode::Absolute),

			0x84 => Self::sty(emulator, AddressingMode::ZeroPage),
			0x94 => Self::sty(emulator, AddressingMode::ZeroPageX),
			0x8c => Self::sty(emulator, AddressingMode::Absolute),

			0x87 => Self::sax(emulator, AddressingMode::ZeroPage),
			0x97 => Self::sax(emulator, AddressingMode::ZeroPageY),
			0x8f => Self::sax(emulator, AddressingMode::Absolute),
			0x83 => Self::sax(emulator, AddressingMode::IndirectX),

			// SXA
			/*0x9e => {
				let address = Self::get_address(emulator, AddressingMode::AbsoluteY);
				write8(emulator, address, emulator.cpu.x & emulator.cpu.a);
			},

			// SYA
			0x9c => {
				let address = Self::get_address(emulator, AddressingMode::AbsoluteX);
				write8(emulator, address, emulator.cpu.y & emulator.cpu.a);
			},*/

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

			0x29 => Self::and(emulator, AddressingMode::Immediate),
			0x25 => Self::and(emulator, AddressingMode::ZeroPage),
			0x35 => Self::and(emulator, AddressingMode::ZeroPageX),
			0x2d => Self::and(emulator, AddressingMode::Absolute),
			0x3d => Self::and(emulator, AddressingMode::AbsoluteX),
			0x39 => Self::and(emulator, AddressingMode::AbsoluteY),
			0x21 => Self::and(emulator, AddressingMode::IndirectX),
			0x31 => Self::and(emulator, AddressingMode::IndirectY),

			/*0x0b => Self::aac(emulator),
			0x2b => Self::aac(emulator),

			// ASR
			0x4b => {
				Self::and(emulator, AddressingMode::Immediate);
				emulator.cpu.a = emulator.cpu.lsr_value(emulator.cpu.a);
			},

			// ARR
			0x6b => {
				Self::and(emulator, AddressingMode::Immediate);
				emulator.cpu.a = emulator.cpu.ror_value(emulator.cpu.a);
			},

			// AXS
			0xcb => {
				let operand = read8(emulator, emulator.cpu.pc);
				emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
				emulator.cpu.x &= emulator.cpu.a;
				emulator.cpu.set_flag(Flag::C, emulator.cpu.x >= operand);
				emulator.cpu.x = emulator.cpu.x.wrapping_sub(operand);
				emulator.cpu.set_n_flag(emulator.cpu.x);
				emulator.cpu.set_z_flag(emulator.cpu.x);
			},*/

			0x09 => Self::ora(emulator, AddressingMode::Immediate),
			0x05 => Self::ora(emulator, AddressingMode::ZeroPage),
			0x15 => Self::ora(emulator, AddressingMode::ZeroPageX),
			0x0d => Self::ora(emulator, AddressingMode::Absolute),
			0x1d => Self::ora(emulator, AddressingMode::AbsoluteX),
			0x19 => Self::ora(emulator, AddressingMode::AbsoluteY),
			0x01 => Self::ora(emulator, AddressingMode::IndirectX),
			0x11 => Self::ora(emulator, AddressingMode::IndirectY),

			0x49 => Self::eor(emulator, AddressingMode::Immediate),
			0x45 => Self::eor(emulator, AddressingMode::ZeroPage),
			0x55 => Self::eor(emulator, AddressingMode::ZeroPageX),
			0x4d => Self::eor(emulator, AddressingMode::Absolute),
			0x5d => Self::eor(emulator, AddressingMode::AbsoluteX),
			0x59 => Self::eor(emulator, AddressingMode::AbsoluteY),
			0x41 => Self::eor(emulator, AddressingMode::IndirectX),
			0x51 => Self::eor(emulator, AddressingMode::IndirectY),

			0x24 => Self::bit(emulator, AddressingMode::ZeroPage),
			0x2c => Self::bit(emulator, AddressingMode::Absolute),

			0x4a => emulator.cpu.a = emulator.cpu.lsr_value(emulator.cpu.a),
			0x46 => Self::lsr(emulator, AddressingMode::ZeroPage),
			0x56 => Self::lsr(emulator, AddressingMode::ZeroPageX),
			0x4e => Self::lsr(emulator, AddressingMode::Absolute),
			0x5e => Self::lsr(emulator, AddressingMode::AbsoluteX),

			0x47 => Self::sre(emulator, AddressingMode::ZeroPage),
			0x57 => Self::sre(emulator, AddressingMode::ZeroPageX),
			0x4f => Self::sre(emulator, AddressingMode::Absolute),
			0x5f => Self::sre(emulator, AddressingMode::AbsoluteX),
			0x5b => Self::sre(emulator, AddressingMode::AbsoluteY),
			0x43 => Self::sre(emulator, AddressingMode::IndirectX),
			0x53 => Self::sre(emulator, AddressingMode::IndirectY),

			0x0a => emulator.cpu.a = emulator.cpu.asl_value(emulator.cpu.a),
			0x06 => Self::asl(emulator, AddressingMode::ZeroPage),
			0x16 => Self::asl(emulator, AddressingMode::ZeroPageX),
			0x0e => Self::asl(emulator, AddressingMode::Absolute),
			0x1e => Self::asl(emulator, AddressingMode::AbsoluteX),

			0x07 => Self::slo(emulator, AddressingMode::ZeroPage),
			0x17 => Self::slo(emulator, AddressingMode::ZeroPageX),
			0x0f => Self::slo(emulator, AddressingMode::Absolute),
			0x1f => Self::slo(emulator, AddressingMode::AbsoluteX),
			0x1b => Self::slo(emulator, AddressingMode::AbsoluteY),
			0x03 => Self::slo(emulator, AddressingMode::IndirectX),
			0x13 => Self::slo(emulator, AddressingMode::IndirectY),

			0x6a => emulator.cpu.a = emulator.cpu.ror_value(emulator.cpu.a),
			0x66 => Self::ror(emulator, AddressingMode::ZeroPage),
			0x76 => Self::ror(emulator, AddressingMode::ZeroPageX),
			0x6e => Self::ror(emulator, AddressingMode::Absolute),
			0x7e => Self::ror(emulator, AddressingMode::AbsoluteX),

			0x67 => Self::rra(emulator, AddressingMode::ZeroPage),
			0x77 => Self::rra(emulator, AddressingMode::ZeroPageX),
			0x6f => Self::rra(emulator, AddressingMode::Absolute),
			0x7f => Self::rra(emulator, AddressingMode::AbsoluteX),
			0x7b => Self::rra(emulator, AddressingMode::AbsoluteY),
			0x63 => Self::rra(emulator, AddressingMode::IndirectX),
			0x73 => Self::rra(emulator, AddressingMode::IndirectY),

			0x2a => emulator.cpu.a = emulator.cpu.rol_value(emulator.cpu.a),
			0x26 => Self::rol(emulator, AddressingMode::ZeroPage),
			0x36 => Self::rol(emulator, AddressingMode::ZeroPageX),
			0x2e => Self::rol(emulator, AddressingMode::Absolute),
			0x3e => Self::rol(emulator, AddressingMode::AbsoluteX),

			0x27 => Self::rla(emulator, AddressingMode::ZeroPage),
			0x37 => Self::rla(emulator, AddressingMode::ZeroPageX),
			0x2f => Self::rla(emulator, AddressingMode::Absolute),
			0x3f => Self::rla(emulator, AddressingMode::AbsoluteX),
			0x3b => Self::rla(emulator, AddressingMode::AbsoluteY),
			0x23 => Self::rla(emulator, AddressingMode::IndirectX),
			0x33 => Self::rla(emulator, AddressingMode::IndirectY),

			0x69 => Self::adc(emulator, AddressingMode::Immediate),
			0x65 => Self::adc(emulator, AddressingMode::ZeroPage),
			0x75 => Self::adc(emulator, AddressingMode::ZeroPageX),
			0x6d => Self::adc(emulator, AddressingMode::Absolute),
			0x7d => Self::adc(emulator, AddressingMode::AbsoluteX),
			0x79 => Self::adc(emulator, AddressingMode::AbsoluteY),
			0x61 => Self::adc(emulator, AddressingMode::IndirectX),
			0x71 => Self::adc(emulator, AddressingMode::IndirectY),

			0xe9 => Self::sbc(emulator, AddressingMode::Immediate),
			0xeb => Self::sbc(emulator, AddressingMode::Immediate),
			0xe5 => Self::sbc(emulator, AddressingMode::ZeroPage),
			0xf5 => Self::sbc(emulator, AddressingMode::ZeroPageX),
			0xed => Self::sbc(emulator, AddressingMode::Absolute),
			0xfd => Self::sbc(emulator, AddressingMode::AbsoluteX),
			0xf9 => Self::sbc(emulator, AddressingMode::AbsoluteY),
			0xe1 => Self::sbc(emulator, AddressingMode::IndirectX),
			0xf1 => Self::sbc(emulator, AddressingMode::IndirectY),
			
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

			0xe6 => Self::inc(emulator, AddressingMode::ZeroPage),
			0xf6 => Self::inc(emulator, AddressingMode::ZeroPageX),
			0xee => Self::inc(emulator, AddressingMode::Absolute),
			0xfe => Self::inc(emulator, AddressingMode::AbsoluteX),

			0xe7 => Self::isb(emulator, AddressingMode::ZeroPage),
			0xf7 => Self::isb(emulator, AddressingMode::ZeroPageX),
			0xef => Self::isb(emulator, AddressingMode::Absolute),
			0xff => Self::isb(emulator, AddressingMode::AbsoluteX),
			0xfb => Self::isb(emulator, AddressingMode::AbsoluteY),
			0xe3 => Self::isb(emulator, AddressingMode::IndirectX),
			0xf3 => Self::isb(emulator, AddressingMode::IndirectY),

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

			0xc6 => Self::dec(emulator, AddressingMode::ZeroPage),
			0xd6 => Self::dec(emulator, AddressingMode::ZeroPageX),
			0xce => Self::dec(emulator, AddressingMode::Absolute),
			0xde => Self::dec(emulator, AddressingMode::AbsoluteX),

			0xc7 => Self::dcp(emulator, AddressingMode::ZeroPage),
			0xd7 => Self::dcp(emulator, AddressingMode::ZeroPageX),
			0xcf => Self::dcp(emulator, AddressingMode::Absolute),
			0xdf => Self::dcp(emulator, AddressingMode::AbsoluteX),
			0xdb => Self::dcp(emulator, AddressingMode::AbsoluteY),
			0xc3 => Self::dcp(emulator, AddressingMode::IndirectX),
			0xd3 => Self::dcp(emulator, AddressingMode::IndirectY),

			0xe0 => Self::cpx(emulator, AddressingMode::Immediate),
			0xe4 => Self::cpx(emulator, AddressingMode::ZeroPage),
			0xec => Self::cpx(emulator, AddressingMode::Absolute),

			0xc0 => Self::cpy(emulator, AddressingMode::Immediate),
			0xc4 => Self::cpy(emulator, AddressingMode::ZeroPage),
			0xcc => Self::cpy(emulator, AddressingMode::Absolute),

			0xc9 => Self::cmp(emulator, AddressingMode::Immediate),
			0xc5 => Self::cmp(emulator, AddressingMode::ZeroPage),
			0xd5 => Self::cmp(emulator, AddressingMode::ZeroPageX),
			0xcd => Self::cmp(emulator, AddressingMode::Absolute),
			0xdd => Self::cmp(emulator, AddressingMode::AbsoluteX),
			0xd9 => Self::cmp(emulator, AddressingMode::AbsoluteY),
			0xc1 => Self::cmp(emulator, AddressingMode::IndirectX),
			0xd1 => Self::cmp(emulator, AddressingMode::IndirectY),

			// PHA
			0x48 => Self::push8(emulator, emulator.cpu.a),

			// PLA
			0x68 => {
				emulator.cpu.a = Self::pull8(emulator);
				emulator.cpu.set_n_flag(emulator.cpu.a);
				emulator.cpu.set_z_flag(emulator.cpu.a);
			},

			// PHP
			0x08 => Self::push8(emulator, emulator.cpu.p | Flag::B as u8),

			0x28 => Self::plp(emulator),

			// CLC
			0x18 => emulator.cpu.set_flag(Flag::C, false),

			// SEC
			0x38 => emulator.cpu.set_flag(Flag::C, true),

			// CLI
			//0x58 => emulator.cpu.set_flag(Flag::I, false),

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
			0x10 => Self::branch(emulator, Flag::N, false),

			// BMI
			0x30 => Self::branch(emulator, Flag::N, true),

			// BVC
			0x50 => Self::branch(emulator, Flag::V, false),

			// BVS
			0x70 => Self::branch(emulator, Flag::V, true),

			// BCC
			0x90 => Self::branch(emulator, Flag::C, false),

			// BCS
			0xb0 => Self::branch(emulator, Flag::C, true),

			// BNE
			0xd0 => Self::branch(emulator, Flag::Z, false),

			// BEQ
			0xf0 => Self::branch(emulator, Flag::Z, true),

			// BRK
			/*0x00 => {
				let address = emulator.cpu.pc.wrapping_add(1);
				Self::push16(emulator, address);
				Self::push8(emulator, emulator.cpu.p | Flag::B as u8);
				emulator.cpu.pc = read16(emulator, IRQ_VECTOR_ADDRESS);
				emulator.cpu.set_flag(Flag::I, true);
			},*/

			// JSR
			0x20 => {
				let address = emulator.cpu.pc.wrapping_add(1);
				Self::push16(emulator, address);
				emulator.cpu.pc = read16(emulator, emulator.cpu.pc);
			},

			// RTI
			0x40 => {
				Self::plp(emulator);
				emulator.cpu.pc = Self::pull16(emulator);
			},

			// RTS
			0x60 => emulator.cpu.pc = Self::pull16(emulator).wrapping_add(1),

			_ => {
				println!("[ERROR] Unknown opcode {:02X} at {:04X}", opcode, emulator.cpu.pc.wrapping_sub(1));
				std::process::exit(1);
			}
		}
	}
}
