pub mod cpu_memory;

#[cfg(feature = "log")]
mod logger;

use self::cpu_memory::*;

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

	fn push8(&mut self, memory: &mut CpuMemory, value: u8) {
		memory.write8(STACK_ADDRESS + self.s as u16, value);
		self.s = self.s.wrapping_sub(1);
	}

	fn push16(&mut self, memory: &mut CpuMemory, value: u16) {
		self.push8(memory, (value >> 8) as _);
		self.push8(memory, value as _);
	}

	fn pull8(&mut self, memory: &CpuMemory) -> u8 {
		self.s = self.s.wrapping_add(1);
		memory.read8(STACK_ADDRESS + self.s as u16)
	}

	fn pull16(&mut self, memory: &CpuMemory) -> u16 {
		let low_byte = self.pull8(memory) as u16;
		let high_byte = self.pull8(memory) as u16;
		(high_byte << 8) | low_byte
	}

	fn plp(&mut self, memory: &CpuMemory) {
		let value = self.pull8(memory);
		self.p = (value | 0b0010_0000) & !(Flag::B as u8);
	}

	fn get_address(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) -> u16 {
		match addressing_mode {
			AddressingMode::Immediate => {
				let address = self.pc;
				self.pc = self.pc.wrapping_add(1);
				address
			},
			AddressingMode::ZeroPage => {
				let address = memory.read8(self.pc);
				self.pc = self.pc.wrapping_add(1);
				address as _
			},
			AddressingMode::ZeroPageX => {
				let address = memory.read8(self.pc).wrapping_add(self.x);
				self.pc = self.pc.wrapping_add(1);
				address as _
			},
			AddressingMode::ZeroPageY => {
				let address = memory.read8(self.pc).wrapping_add(self.y);
				self.pc = self.pc.wrapping_add(1);
				address as _
			},
			AddressingMode::Absolute => {
				let address = memory.read16(self.pc);
				self.pc = self.pc.wrapping_add(2);
				address
			},
			AddressingMode::AbsoluteX => {
				let address = memory.read16(self.pc).wrapping_add(self.x as _);
				self.pc = self.pc.wrapping_add(2);
				address
			},
			AddressingMode::AbsoluteY => {
				let address = memory.read16(self.pc).wrapping_add(self.y as _);
				self.pc = self.pc.wrapping_add(2);
				address
			},
			AddressingMode::IndirectX => {
				let address = memory.read8(self.pc).wrapping_add(self.x);
				self.pc = self.pc.wrapping_add(1);
				let low_byte = memory.read8(address as _) as u16;
				let high_byte = memory.read8(address.wrapping_add(1) as _) as u16;
				(high_byte << 8) | low_byte
			},
			AddressingMode::IndirectY => {
				let address = memory.read8(self.pc);
				self.pc = self.pc.wrapping_add(1);
				let low_byte = memory.read8(address as _) as u16;
				let high_byte = memory.read8(address.wrapping_add(1) as _) as u16;
				let value = (high_byte << 8) | low_byte;
				value.wrapping_add(self.y as _)
			}
		}
	}

	fn get_operand(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) -> u8 {
		let address = self.get_address(memory, addressing_mode);
		memory.read8(address)
	}

	fn lda(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		self.a = self.get_operand(memory, addressing_mode);
		self.set_n_flag(self.a);
		self.set_z_flag(self.a);
	}

	fn ldx(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		self.x = self.get_operand(memory, addressing_mode);
		self.set_n_flag(self.x);
		self.set_z_flag(self.x);
	}

	fn ldy(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		self.y = self.get_operand(memory, addressing_mode);
		self.set_n_flag(self.y);
		self.set_z_flag(self.y);
	}

	fn lax(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		self.a = self.get_operand(memory, addressing_mode);
		self.x = self.a;
		self.set_n_flag(self.x);
		self.set_z_flag(self.x);
	}

	fn sta(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		memory.write8(address, self.a);
	}

	fn stx(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		memory.write8(address, self.x);
	}

	fn sty(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		memory.write8(address, self.y);
	}

	fn sax(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		memory.write8(address, self.a & self.x);
	}

	fn and_address(&mut self, memory: &CpuMemory, address: u16) {
		self.a &= memory.read8(address);
		self.set_n_flag(self.a);
		self.set_z_flag(self.a);
	}

	fn and(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.and_address(memory, address);
	}

	/*fn aac(&mut self, memory: &CpuMemory) {
		self.and(memory, AddressingMode::Immediate);
		let n = self.get_flag(Flag::N);
		self.set_flag(Flag::C, n);
	}*/

	fn ora_address(&mut self, memory: &CpuMemory, address: u16) {
		self.a |= memory.read8(address);
		self.set_n_flag(self.a);
		self.set_z_flag(self.a);
	}

	fn ora(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.ora_address(memory, address);
	}

	fn eor_address(&mut self, memory: &CpuMemory, address: u16) {
		self.a ^= memory.read8(address);
		self.set_n_flag(self.a);
		self.set_z_flag(self.a);
	}

	fn eor(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.eor_address(memory, address);
	}

	fn bit(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let operand = self.get_operand(memory, addressing_mode);
		self.set_z_flag(operand & self.a);
		self.set_n_flag(operand);
		self.set_flag(Flag::V, ((operand >> 6) & 1) == 1);
	}

	fn lsr_value(&mut self, mut value: u8) -> u8 {
		self.set_flag(Flag::C, (value & 1) == 1);
		value >>= 1;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn lsr_address(&mut self, memory: &mut CpuMemory, address: u16) {
		let operand = memory.read8(address);
		let result = self.lsr_value(operand);
		memory.write8(address, result);
	}

	fn lsr(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.lsr_address(memory, address);
	}

	fn sre(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.lsr_address(memory, address);
		self.eor_address(memory, address);
	}

	fn asl_value(&mut self, mut value: u8) -> u8 {
		self.set_flag(Flag::C, (value >> 7) == 1);
		value <<= 1;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn asl_address(&mut self, memory: &mut CpuMemory, address: u16) {
		let operand = memory.read8(address);
		let result = self.asl_value(operand);
		memory.write8(address, result);
	}

	fn asl(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.asl_address(memory, address);
	}

	fn slo(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.asl_address(memory, address);
		self.ora_address(memory, address);
	}

	fn ror_value(&mut self, mut value: u8) -> u8 {
		let c = self.get_flag(Flag::C) as u8;
		self.set_flag(Flag::C, (value & 1) == 1);
		value = (c << 7) | (value >> 1);
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn ror_address(&mut self, memory: &mut CpuMemory, address: u16) {
		let operand = memory.read8(address);
		let result = self.ror_value(operand);
		memory.write8(address, result);
	}

	fn ror(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.ror_address(memory, address);
	}

	fn rra(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.ror_address(memory, address);
		self.adc_address(memory, address);
	}

	fn rol_value(&mut self, mut value: u8) -> u8 {
		let c = self.get_flag(Flag::C) as u8;
		self.set_flag(Flag::C, (value >> 7) == 1);
		value = (value << 1) | c;
		self.set_n_flag(value);
		self.set_z_flag(value);
		value
	}

	fn rol_address(&mut self, memory: &mut CpuMemory, address: u16) {
		let operand = memory.read8(address);
		let result = self.rol_value(operand);
		memory.write8(address, result);
	}

	fn rol(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.rol_address(memory, address);
	}

	fn rla(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.rol_address(memory, address);
		self.and_address(memory, address);
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

	fn adc_address(&mut self, memory: &CpuMemory, address: u16) {
		let operand = memory.read8(address);
		self.adc_value(operand);
	}

	fn adc(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let operand = self.get_operand(memory, addressing_mode);
		self.adc_value(operand);
	}

	fn sbc_address(&mut self, memory: &CpuMemory, address: u16) {
		let operand = memory.read8(address);
		self.adc_value(!operand);
	}

	fn sbc(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let operand = self.get_operand(memory, addressing_mode);
		self.adc_value(!operand);
	}

	fn inc_address(&mut self, memory: &mut CpuMemory, address: u16) {
		let result = memory.read8(address).wrapping_add(1);
		memory.write8(address, result);
		self.set_z_flag(result);
		self.set_n_flag(result);
	}

	fn inc(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.inc_address(memory, address);
	}

	fn isb(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.inc_address(memory, address);
		self.sbc_address(memory, address);
	}

	fn dec_address(&mut self, memory: &mut CpuMemory, address: u16) {
		let result = memory.read8(address).wrapping_sub(1);
		memory.write8(address, result);
		self.set_z_flag(result);
		self.set_n_flag(result);
	}

	fn dec(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.dec_address(memory, address);
	}

	fn dcp(&mut self, memory: &mut CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.dec_address(memory, address);
		self.cmp_address(memory, address);
	}

	fn cpx(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let operand = self.get_operand(memory, addressing_mode);
		self.set_flag(Flag::C, self.x >= operand);
		let result = self.x.wrapping_sub(operand);
		self.set_z_flag(result);
		self.set_n_flag(result);
	}

	fn cpy(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let operand = self.get_operand(memory, addressing_mode);
		self.set_flag(Flag::C, self.y >= operand);
		let result = self.y.wrapping_sub(operand);
		self.set_z_flag(result);
		self.set_n_flag(result);
	}

	fn cmp_address(&mut self, memory: &CpuMemory, address: u16) {
		let operand = memory.read8(address);
		self.set_flag(Flag::C, self.a >= operand);
		let result = self.a.wrapping_sub(operand);
		self.set_z_flag(result);
		self.set_n_flag(result);
	}

	fn cmp(&mut self, memory: &CpuMemory, addressing_mode: AddressingMode) {
		let address = self.get_address(memory, addressing_mode);
		self.cmp_address(memory, address);
	}

	fn branch(&mut self, memory: &CpuMemory, flag: Flag, value: bool) {
		if self.get_flag(flag) == value {
			let offset = memory.read8(self.pc) as i8;
			self.pc = if offset > 0 {
				self.pc.wrapping_add(offset as _)
			} else {
				self.pc.wrapping_sub(offset.wrapping_neg() as _)
			};
		}
		self.pc = self.pc.wrapping_add(1);
	}

	pub fn request_interrupt(&mut self, interrupt: Interrupt) {
		if self.pending_interrupt != Some(Interrupt::Nmi) {
			self.pending_interrupt = Some(interrupt);
		}
	}

	pub fn execute_next_instruction(&mut self, memory: &mut CpuMemory) {
		if self.pending_interrupt == Some(Interrupt::Nmi) || (self.pending_interrupt == Some(Interrupt::Irq) && !self.get_flag(Flag::I)) {
			self.push16(memory, self.pc);
			self.push8(memory, self.p);
			self.pc = memory.read16(if self.pending_interrupt == Some(Interrupt::Nmi) {
				NMI_VECTOR_ADDRESS
			} else {
				IRQ_VECTOR_ADDRESS
			});
			self.set_flag(Flag::I, true);
			self.pending_interrupt = None;
		}

		#[cfg(feature = "log")]
		self.logger.create_log(self, memory);
		
		let opcode = memory.read8(self.pc);
		self.pc = self.pc.wrapping_add(1);
		
		match opcode {
			// NOPs
			0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xea | 0xfa => {},
			0x04 | 0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 /*| 0x82 | 0x89 | 0xc2 | 0xe2*/ | 0xd4 | 0xf4 => self.pc = self.pc.wrapping_add(1),
			0x0c | 0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => self.pc = self.pc.wrapping_add(2),

			0xa9 => self.lda(memory, AddressingMode::Immediate),
			0xa5 => self.lda(memory, AddressingMode::ZeroPage),
			0xb5 => self.lda(memory, AddressingMode::ZeroPageX),
			0xad => self.lda(memory, AddressingMode::Absolute),
			0xbd => self.lda(memory, AddressingMode::AbsoluteX),
			0xb9 => self.lda(memory, AddressingMode::AbsoluteY),
			0xa1 => self.lda(memory, AddressingMode::IndirectX),
			0xb1 => self.lda(memory, AddressingMode::IndirectY),

			0xa2 => self.ldx(memory, AddressingMode::Immediate),
			0xa6 => self.ldx(memory, AddressingMode::ZeroPage),
			0xb6 => self.ldx(memory, AddressingMode::ZeroPageY),
			0xae => self.ldx(memory, AddressingMode::Absolute),
			0xbe => self.ldx(memory, AddressingMode::AbsoluteY),

			0xa0 => self.ldy(memory, AddressingMode::Immediate),
			0xa4 => self.ldy(memory, AddressingMode::ZeroPage),
			0xb4 => self.ldy(memory, AddressingMode::ZeroPageX),
			0xac => self.ldy(memory, AddressingMode::Absolute),
			0xbc => self.ldy(memory, AddressingMode::AbsoluteX),

			//0xab => self.lax(memory, AddressingMode::Immediate),
			0xa7 => self.lax(memory, AddressingMode::ZeroPage),
			0xb7 => self.lax(memory, AddressingMode::ZeroPageY),
			0xaf => self.lax(memory, AddressingMode::Absolute),
			0xbf => self.lax(memory, AddressingMode::AbsoluteY),
			0xa3 => self.lax(memory, AddressingMode::IndirectX),
			0xb3 => self.lax(memory, AddressingMode::IndirectY),

			0x85 => self.sta(memory, AddressingMode::ZeroPage),
			0x95 => self.sta(memory, AddressingMode::ZeroPageX),
			0x8d => self.sta(memory, AddressingMode::Absolute),
			0x9d => self.sta(memory, AddressingMode::AbsoluteX),
			0x99 => self.sta(memory, AddressingMode::AbsoluteY),
			0x81 => self.sta(memory, AddressingMode::IndirectX),
			0x91 => self.sta(memory, AddressingMode::IndirectY),

			0x86 => self.stx(memory, AddressingMode::ZeroPage),
			0x96 => self.stx(memory, AddressingMode::ZeroPageY),
			0x8e => self.stx(memory, AddressingMode::Absolute),

			0x84 => self.sty(memory, AddressingMode::ZeroPage),
			0x94 => self.sty(memory, AddressingMode::ZeroPageX),
			0x8c => self.sty(memory, AddressingMode::Absolute),

			0x87 => self.sax(memory, AddressingMode::ZeroPage),
			0x97 => self.sax(memory, AddressingMode::ZeroPageY),
			0x8f => self.sax(memory, AddressingMode::Absolute),
			0x83 => self.sax(memory, AddressingMode::IndirectX),

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

			// TAX
			0xaa => {
				self.x = self.a;
				self.set_n_flag(self.x);
				self.set_z_flag(self.x);
			},

			// TXA
			0x8a => {
				self.a = self.x;
				self.set_n_flag(self.a);
				self.set_z_flag(self.a);
			},

			// TAY
			0xa8 => {
				self.y = self.a;
				self.set_n_flag(self.y);
				self.set_z_flag(self.y);
			},

			// TYA
			0x98 => {
				self.a = self.y;
				self.set_n_flag(self.a);
				self.set_z_flag(self.a);
			},

			// TSX
			0xba => {
				self.x = self.s;
				self.set_n_flag(self.x);
				self.set_z_flag(self.x);
			},

			// TXS
			0x9a => self.s = self.x,

			0x29 => self.and(memory, AddressingMode::Immediate),
			0x25 => self.and(memory, AddressingMode::ZeroPage),
			0x35 => self.and(memory, AddressingMode::ZeroPageX),
			0x2d => self.and(memory, AddressingMode::Absolute),
			0x3d => self.and(memory, AddressingMode::AbsoluteX),
			0x39 => self.and(memory, AddressingMode::AbsoluteY),
			0x21 => self.and(memory, AddressingMode::IndirectX),
			0x31 => self.and(memory, AddressingMode::IndirectY),

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

			0x09 => self.ora(memory, AddressingMode::Immediate),
			0x05 => self.ora(memory, AddressingMode::ZeroPage),
			0x15 => self.ora(memory, AddressingMode::ZeroPageX),
			0x0d => self.ora(memory, AddressingMode::Absolute),
			0x1d => self.ora(memory, AddressingMode::AbsoluteX),
			0x19 => self.ora(memory, AddressingMode::AbsoluteY),
			0x01 => self.ora(memory, AddressingMode::IndirectX),
			0x11 => self.ora(memory, AddressingMode::IndirectY),

			0x49 => self.eor(memory, AddressingMode::Immediate),
			0x45 => self.eor(memory, AddressingMode::ZeroPage),
			0x55 => self.eor(memory, AddressingMode::ZeroPageX),
			0x4d => self.eor(memory, AddressingMode::Absolute),
			0x5d => self.eor(memory, AddressingMode::AbsoluteX),
			0x59 => self.eor(memory, AddressingMode::AbsoluteY),
			0x41 => self.eor(memory, AddressingMode::IndirectX),
			0x51 => self.eor(memory, AddressingMode::IndirectY),

			0x24 => self.bit(memory, AddressingMode::ZeroPage),
			0x2c => self.bit(memory, AddressingMode::Absolute),

			0x4a => self.a = self.lsr_value(self.a),
			0x46 => self.lsr(memory, AddressingMode::ZeroPage),
			0x56 => self.lsr(memory, AddressingMode::ZeroPageX),
			0x4e => self.lsr(memory, AddressingMode::Absolute),
			0x5e => self.lsr(memory, AddressingMode::AbsoluteX),

			0x47 => self.sre(memory, AddressingMode::ZeroPage),
			0x57 => self.sre(memory, AddressingMode::ZeroPageX),
			0x4f => self.sre(memory, AddressingMode::Absolute),
			0x5f => self.sre(memory, AddressingMode::AbsoluteX),
			0x5b => self.sre(memory, AddressingMode::AbsoluteY),
			0x43 => self.sre(memory, AddressingMode::IndirectX),
			0x53 => self.sre(memory, AddressingMode::IndirectY),

			0x0a => self.a = self.asl_value(self.a),
			0x06 => self.asl(memory, AddressingMode::ZeroPage),
			0x16 => self.asl(memory, AddressingMode::ZeroPageX),
			0x0e => self.asl(memory, AddressingMode::Absolute),
			0x1e => self.asl(memory, AddressingMode::AbsoluteX),

			0x07 => self.slo(memory, AddressingMode::ZeroPage),
			0x17 => self.slo(memory, AddressingMode::ZeroPageX),
			0x0f => self.slo(memory, AddressingMode::Absolute),
			0x1f => self.slo(memory, AddressingMode::AbsoluteX),
			0x1b => self.slo(memory, AddressingMode::AbsoluteY),
			0x03 => self.slo(memory, AddressingMode::IndirectX),
			0x13 => self.slo(memory, AddressingMode::IndirectY),

			0x6a => self.a = self.ror_value(self.a),
			0x66 => self.ror(memory, AddressingMode::ZeroPage),
			0x76 => self.ror(memory, AddressingMode::ZeroPageX),
			0x6e => self.ror(memory, AddressingMode::Absolute),
			0x7e => self.ror(memory, AddressingMode::AbsoluteX),

			0x67 => self.rra(memory, AddressingMode::ZeroPage),
			0x77 => self.rra(memory, AddressingMode::ZeroPageX),
			0x6f => self.rra(memory, AddressingMode::Absolute),
			0x7f => self.rra(memory, AddressingMode::AbsoluteX),
			0x7b => self.rra(memory, AddressingMode::AbsoluteY),
			0x63 => self.rra(memory, AddressingMode::IndirectX),
			0x73 => self.rra(memory, AddressingMode::IndirectY),

			0x2a => self.a = self.rol_value(self.a),
			0x26 => self.rol(memory, AddressingMode::ZeroPage),
			0x36 => self.rol(memory, AddressingMode::ZeroPageX),
			0x2e => self.rol(memory, AddressingMode::Absolute),
			0x3e => self.rol(memory, AddressingMode::AbsoluteX),

			0x27 => self.rla(memory, AddressingMode::ZeroPage),
			0x37 => self.rla(memory, AddressingMode::ZeroPageX),
			0x2f => self.rla(memory, AddressingMode::Absolute),
			0x3f => self.rla(memory, AddressingMode::AbsoluteX),
			0x3b => self.rla(memory, AddressingMode::AbsoluteY),
			0x23 => self.rla(memory, AddressingMode::IndirectX),
			0x33 => self.rla(memory, AddressingMode::IndirectY),

			0x69 => self.adc(memory, AddressingMode::Immediate),
			0x65 => self.adc(memory, AddressingMode::ZeroPage),
			0x75 => self.adc(memory, AddressingMode::ZeroPageX),
			0x6d => self.adc(memory, AddressingMode::Absolute),
			0x7d => self.adc(memory, AddressingMode::AbsoluteX),
			0x79 => self.adc(memory, AddressingMode::AbsoluteY),
			0x61 => self.adc(memory, AddressingMode::IndirectX),
			0x71 => self.adc(memory, AddressingMode::IndirectY),

			0xe9 => self.sbc(memory, AddressingMode::Immediate),
			0xeb => self.sbc(memory, AddressingMode::Immediate),
			0xe5 => self.sbc(memory, AddressingMode::ZeroPage),
			0xf5 => self.sbc(memory, AddressingMode::ZeroPageX),
			0xed => self.sbc(memory, AddressingMode::Absolute),
			0xfd => self.sbc(memory, AddressingMode::AbsoluteX),
			0xf9 => self.sbc(memory, AddressingMode::AbsoluteY),
			0xe1 => self.sbc(memory, AddressingMode::IndirectX),
			0xf1 => self.sbc(memory, AddressingMode::IndirectY),
			
			// INX
			0xe8 => {
				self.x = self.x.wrapping_add(1);
				self.set_z_flag(self.x);
				self.set_n_flag(self.x);
			},

			// INY
			0xc8 => {
				self.y = self.y.wrapping_add(1);
				self.set_z_flag(self.y);
				self.set_n_flag(self.y);
			},

			0xe6 => self.inc(memory, AddressingMode::ZeroPage),
			0xf6 => self.inc(memory, AddressingMode::ZeroPageX),
			0xee => self.inc(memory, AddressingMode::Absolute),
			0xfe => self.inc(memory, AddressingMode::AbsoluteX),

			0xe7 => self.isb(memory, AddressingMode::ZeroPage),
			0xf7 => self.isb(memory, AddressingMode::ZeroPageX),
			0xef => self.isb(memory, AddressingMode::Absolute),
			0xff => self.isb(memory, AddressingMode::AbsoluteX),
			0xfb => self.isb(memory, AddressingMode::AbsoluteY),
			0xe3 => self.isb(memory, AddressingMode::IndirectX),
			0xf3 => self.isb(memory, AddressingMode::IndirectY),

			// DEX
			0xca => {
				self.x = self.x.wrapping_sub(1);
				self.set_z_flag(self.x);
				self.set_n_flag(self.x);
			},

			// DEY
			0x88 => {
				self.y = self.y.wrapping_sub(1);
				self.set_n_flag(self.y);
				self.set_z_flag(self.y);
			},

			0xc6 => self.dec(memory, AddressingMode::ZeroPage),
			0xd6 => self.dec(memory, AddressingMode::ZeroPageX),
			0xce => self.dec(memory, AddressingMode::Absolute),
			0xde => self.dec(memory, AddressingMode::AbsoluteX),

			0xc7 => self.dcp(memory, AddressingMode::ZeroPage),
			0xd7 => self.dcp(memory, AddressingMode::ZeroPageX),
			0xcf => self.dcp(memory, AddressingMode::Absolute),
			0xdf => self.dcp(memory, AddressingMode::AbsoluteX),
			0xdb => self.dcp(memory, AddressingMode::AbsoluteY),
			0xc3 => self.dcp(memory, AddressingMode::IndirectX),
			0xd3 => self.dcp(memory, AddressingMode::IndirectY),

			0xe0 => self.cpx(memory, AddressingMode::Immediate),
			0xe4 => self.cpx(memory, AddressingMode::ZeroPage),
			0xec => self.cpx(memory, AddressingMode::Absolute),

			0xc0 => self.cpy(memory, AddressingMode::Immediate),
			0xc4 => self.cpy(memory, AddressingMode::ZeroPage),
			0xcc => self.cpy(memory, AddressingMode::Absolute),

			0xc9 => self.cmp(memory, AddressingMode::Immediate),
			0xc5 => self.cmp(memory, AddressingMode::ZeroPage),
			0xd5 => self.cmp(memory, AddressingMode::ZeroPageX),
			0xcd => self.cmp(memory, AddressingMode::Absolute),
			0xdd => self.cmp(memory, AddressingMode::AbsoluteX),
			0xd9 => self.cmp(memory, AddressingMode::AbsoluteY),
			0xc1 => self.cmp(memory, AddressingMode::IndirectX),
			0xd1 => self.cmp(memory, AddressingMode::IndirectY),

			// PHA
			0x48 => self.push8(memory, self.a),

			// PLA
			0x68 => {
				self.a = self.pull8(memory);
				self.set_n_flag(self.a);
				self.set_z_flag(self.a);
			},

			// PHP
			0x08 => self.push8(memory, self.p | Flag::B as u8),

			0x28 => self.plp(memory),

			// CLC
			0x18 => self.set_flag(Flag::C, false),

			// SEC
			0x38 => self.set_flag(Flag::C, true),

			// CLI
			//0x58 => self.set_flag(Flag::I, false),

			// SEI
			0x78 => self.set_flag(Flag::I, true),

			// CLD
			0xd8 => self.set_flag(Flag::D, false),

			// SED
			0xf8 => self.set_flag(Flag::D, true),

			// CLV
			0xb8 => self.set_flag(Flag::V, false),

			// JMP (absolute)
			0x4c => self.pc = memory.read16(self.pc),

			// JMP (indirect)
			0x6c => {
				let address = memory.read16(self.pc);
				let low_byte = memory.read8(address) as u16;
				let high_byte = if (address & 0xff) == 0xff  {
					memory.read8(address & 0xff00)
				} else {
					memory.read8(address + 1)
				} as u16;
				self.pc = (high_byte << 8) | low_byte;
			},

			// BPL
			0x10 => self.branch(memory, Flag::N, false),

			// BMI
			0x30 => self.branch(memory, Flag::N, true),

			// BVC
			0x50 => self.branch(memory, Flag::V, false),

			// BVS
			0x70 => self.branch(memory, Flag::V, true),

			// BCC
			0x90 => self.branch(memory, Flag::C, false),

			// BCS
			0xb0 => self.branch(memory, Flag::C, true),

			// BNE
			0xd0 => self.branch(memory, Flag::Z, false),

			// BEQ
			0xf0 => self.branch(memory, Flag::Z, true),

			// BRK
			/*0x00 => {
				let address = self.pc.wrapping_add(1);
				self.push16(memory, address);
				self.push8(memory, self.p | Flag::B as u8);
				self.pc = memory.read16(IRQ_VECTOR_ADDRESS);
				self.set_flag(Flag::I, true);
			},*/

			// JSR
			0x20 => {
				let address = self.pc.wrapping_add(1);
				self.push16(memory, address);
				self.pc = memory.read16(self.pc);
			},

			// RTI
			0x40 => {
				self.plp(memory);
				self.pc = self.pull16(memory);
			},

			// RTS
			0x60 => self.pc = self.pull16(memory).wrapping_add(1),

			_ => {
				println!("[ERROR] Unknown opcode {:02X} at {:04X}", opcode, self.pc.wrapping_sub(1));
				std::process::exit(1);
			}
		}
	}
}
