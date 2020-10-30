mod memory;
mod addressing_modes;

#[cfg(test)]
pub mod tests;

#[cfg(feature = "trace")]
mod logger;

use std::mem::{self, MaybeUninit};
use emulator::*;
use self::memory::*;
use self::addressing_modes::*;

#[cfg(feature = "trace")]
use self::logger::*;

const STACK_ADDRESS: u16 = 0x100;
const NMI_VECTOR_ADDRESS: u16 = 0xfffa;
const RESET_VECTOR_ADDRESS: u16 = 0xfffc;
const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;

const OPCODE_COUNT: usize = 0x100;

                                        // 0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
const OPCODE_CYCLES: [u8; OPCODE_COUNT] = [7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,  // 00
                                           2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // 10
                                           6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,  // 20
                                           2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // 30
                                           6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,  // 40
                                           2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // 50
                                           6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,  // 60
                                           2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // 70
                                           2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,  // 80
                                           2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,  // 90
                                           2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,  // a0
                                           2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,  // b0
                                           2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,  // c0
                                           2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // d0
                                           2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,  // e0
                                           2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7]; // f0

                                                      // 0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
const PAGE_CROSSING_OPCODE_CYCLES: [u8; OPCODE_COUNT] = [7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,  // 00
                                                         3, 6, 2, 8, 4, 4, 6, 6, 2, 5, 2, 7, 5, 5, 7, 7,  // 10
                                                         6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,  // 20
                                                         3, 6, 2, 8, 4, 4, 6, 6, 2, 5, 2, 7, 5, 5, 7, 7,  // 30
                                                         6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,  // 40
                                                         3, 6, 2, 8, 4, 4, 6, 6, 2, 5, 2, 7, 5, 5, 7, 7,  // 50
                                                         6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,  // 60
                                                         3, 6, 2, 8, 4, 4, 6, 6, 2, 5, 2, 7, 5, 5, 7, 7,  // 70
                                                         2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,  // 80
                                                         3, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,  // 90
                                                         2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,  // a0
                                                         3, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,  // b0
                                                         2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,  // c0
                                                         3, 6, 2, 8, 4, 4, 6, 6, 2, 5, 2, 7, 5, 5, 7, 7,  // d0
                                                         2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,  // e0
                                                         3, 6, 2, 8, 4, 4, 6, 6, 2, 5, 2, 7, 5, 5, 7, 7]; // f0

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

#[derive(Copy, Clone)]
struct LookupTableEntry {
	instruction: fn(&mut Emulator),
	cycles: u8,
	page_crossing_cycles: u8,

	#[cfg(feature = "trace")]
	trace_function: fn(&Emulator, u8)
}

pub struct Cpu {
	a: u8,
	x: u8,
	y: u8,
	pc: u16,
	s: u8,
	p: u8,
	pending_interrupt: Option<Interrupt>,
	lookup_table: [LookupTableEntry; OPCODE_COUNT],
	page_crossed: bool,
	branch_taken: bool,

	#[cfg(feature = "trace")]
	logger: Logger
}

impl Cpu {
	pub fn new() -> Self {
		let lookup_table = {
			let mut lookup_table: [MaybeUninit<LookupTableEntry>; OPCODE_COUNT] = unsafe {
				MaybeUninit::uninit().assume_init()
			};

			for (opcode, entry) in lookup_table.iter_mut().enumerate() {
				*entry = MaybeUninit::new(LookupTableEntry {
					instruction: get_instruction(opcode as _),
					cycles: OPCODE_CYCLES[opcode],
					page_crossing_cycles: PAGE_CROSSING_OPCODE_CYCLES[opcode],

					#[cfg(feature = "trace")]
					trace_function: Logger::get_trace_function(opcode as _)
				});
			}

			unsafe { mem::transmute::<_, [LookupTableEntry; OPCODE_COUNT]>(lookup_table) }
		};

		Self {
			a: 0,
			x: 0,
			y: 0,
			pc: 0,
			s: 0xfd,
			p: 0x24,
			pending_interrupt: None,
			lookup_table,
			page_crossed: false,
			branch_taken: false,

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

	fn check_page_crossing(&mut self, address_a: u16, address_b: u16) {
		self.page_crossed = (address_a & 0xff00) != (address_b & 0xff00);
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

	fn set_nz_flags(&mut self, value: u8) {
		self.set_n_flag(value);
		self.set_z_flag(value);
	}

	fn lsr_value(&mut self, mut value: u8) -> u8 {
		self.set_flag(Flag::C, (value & 1) == 1);
		value >>= 1;
		self.set_nz_flags(value);
		value
	}

	fn asl_value(&mut self, mut value: u8) -> u8 {
		self.set_flag(Flag::C, (value >> 7) == 1);
		value <<= 1;
		self.set_nz_flags(value);
		value
	}

	fn ror_value(&mut self, mut value: u8) -> u8 {
		let c = self.get_flag(Flag::C) as u8;
		self.set_flag(Flag::C, (value & 1) == 1);
		value = (c << 7) | (value >> 1);
		self.set_nz_flags(value);
		value
	}

	fn rol_value(&mut self, mut value: u8) -> u8 {
		let c = self.get_flag(Flag::C) as u8;
		self.set_flag(Flag::C, (value >> 7) == 1);
		value = (value << 1) | c;
		self.set_nz_flags(value);
		value
	}

	fn adc_value(&mut self, value: u8) {
		let carry = self.get_flag(Flag::C);
		let sum = self.a as u16 + value as u16 + carry as u16;
		self.set_flag(Flag::C, sum > 0xff);
		let result = sum as u8;
		self.set_flag(Flag::V, ((self.a ^ result) & (value ^ result) & 0x80) != 0);
		self.a = result;
		self.set_nz_flags(self.a);
	}

	fn inc_value(&mut self, mut value: u8) -> u8 {
		value = value.wrapping_add(1);
		self.set_nz_flags(value);
		value
	}

	fn dec_value(&mut self, mut value: u8) -> u8 {
		value = value.wrapping_sub(1);
		self.set_nz_flags(value);
		value
	}

	pub fn request_interrupt(&mut self, interrupt: Interrupt) {
		if self.pending_interrupt != Some(Interrupt::Nmi) {
			self.pending_interrupt = Some(interrupt);
		}
	}

	pub fn execute_next_instruction(emulator: &mut Emulator) -> u8 {
		match emulator.cpu.pending_interrupt {
			Some(Interrupt::Nmi) => perform_interrupt(emulator, NMI_VECTOR_ADDRESS),
			Some(Interrupt::Irq) => if !emulator.cpu.get_flag(Flag::I) {
				perform_interrupt(emulator, IRQ_VECTOR_ADDRESS);
			},
			None => {}
		}
		
		let opcode = read_next8(emulator);
		let entry = emulator.cpu.lookup_table[opcode as usize];

		#[cfg(feature = "trace")]
		(entry.trace_function)(emulator, opcode);

		emulator.cpu.branch_taken = false;
		emulator.cpu.page_crossed = false;
		(entry.instruction)(emulator);
		
		(if emulator.cpu.page_crossed {
			entry.page_crossing_cycles
		} else {
			entry.cycles
		}) + emulator.cpu.branch_taken as u8
	}
}

fn get_instruction(opcode: u8) -> fn(&mut Emulator) {
	match opcode {
		// NOPs
		0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xea | 0xfa => |_| {},
		0x04 | 0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 | 0xd4 | 0xf4 => |emulator| emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1),
		0x0c => |emulator| emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2),
		0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => |emulator| {
			AbsoluteX::get_address(emulator);
		},

		0xa9 => lda::<Immediate>,
		0xa5 => lda::<ZeroPage>,
		0xb5 => lda::<ZeroPageX>,
		0xad => lda::<Absolute>,
		0xbd => lda::<AbsoluteX>,
		0xb9 => lda::<AbsoluteY>,
		0xa1 => lda::<IndirectX>,
		0xb1 => lda::<IndirectY>,

		0xa2 => ldx::<Immediate>,
		0xa6 => ldx::<ZeroPage>,
		0xb6 => ldx::<ZeroPageY>,
		0xae => ldx::<Absolute>,
		0xbe => ldx::<AbsoluteY>,

		0xa0 => ldy::<Immediate>,
		0xa4 => ldy::<ZeroPage>,
		0xb4 => ldy::<ZeroPageX>,
		0xac => ldy::<Absolute>,
		0xbc => ldy::<AbsoluteX>,

		0xab => lax::<Immediate>,
		0xa7 => lax::<ZeroPage>,
		0xb7 => lax::<ZeroPageY>,
		0xaf => lax::<Absolute>,
		0xbf => lax::<AbsoluteY>,
		0xa3 => lax::<IndirectX>,
		0xb3 => lax::<IndirectY>,

		0x85 => sta::<ZeroPage>,
		0x95 => sta::<ZeroPageX>,
		0x8d => sta::<Absolute>,
		0x9d => sta::<AbsoluteX>,
		0x99 => sta::<AbsoluteY>,
		0x81 => sta::<IndirectX>,
		0x91 => sta::<IndirectY>,

		0x86 => stx::<ZeroPage>,
		0x96 => stx::<ZeroPageY>,
		0x8e => stx::<Absolute>,

		0x84 => sty::<ZeroPage>,
		0x94 => sty::<ZeroPageX>,
		0x8c => sty::<Absolute>,

		0x87 => sax::<ZeroPage>,
		0x97 => sax::<ZeroPageY>,
		0x8f => sax::<Absolute>,
		0x83 => sax::<IndirectX>,

		// SXA
		0x9e => |emulator| {
			let mut address = AbsoluteY::get_address(emulator);
			let high_byte = (address >> 8) as u8;
			if emulator.cpu.page_crossed {
				address &= (emulator.cpu.x as u16) << 8;
			}
			write(emulator, address, emulator.cpu.x & high_byte.wrapping_add(1));
		},

		// SYA
		0x9c => |emulator| {
			let mut address = AbsoluteX::get_address(emulator);
			let high_byte = (address >> 8) as u8;
			if emulator.cpu.page_crossed {
				address &= (emulator.cpu.y as u16) << 8;
			}
			write(emulator, address, emulator.cpu.y & high_byte.wrapping_add(1));
		},

		// TAX
		0xaa => |emulator| {
			emulator.cpu.x = emulator.cpu.a;
			emulator.cpu.set_nz_flags(emulator.cpu.x);
		},

		// TXA
		0x8a => |emulator| {
			emulator.cpu.a = emulator.cpu.x;
			emulator.cpu.set_nz_flags(emulator.cpu.a);
		},

		// TAY
		0xa8 => |emulator| {
			emulator.cpu.y = emulator.cpu.a;
			emulator.cpu.set_nz_flags(emulator.cpu.y);
		},

		// TYA
		0x98 => |emulator| {
			emulator.cpu.a = emulator.cpu.y;
			emulator.cpu.set_nz_flags(emulator.cpu.a);
		},

		// TSX
		0xba => |emulator| {
			emulator.cpu.x = emulator.cpu.s;
			emulator.cpu.set_nz_flags(emulator.cpu.x);
		},

		// TXS
		0x9a => |emulator| emulator.cpu.s = emulator.cpu.x,

		0x29 => and::<Immediate>,
		0x25 => and::<ZeroPage>,
		0x35 => and::<ZeroPageX>,
		0x2d => and::<Absolute>,
		0x3d => and::<AbsoluteX>,
		0x39 => and::<AbsoluteY>,
		0x21 => and::<IndirectX>,
		0x31 => and::<IndirectY>,

		// AAC
		0x0b | 0x2b => |emulator| {
			and::<Immediate>(emulator);
			let n = emulator.cpu.get_flag(Flag::N);
			emulator.cpu.set_flag(Flag::C, n);
		},

		// ASR
		0x4b => |emulator| {
			and::<Immediate>(emulator);
			emulator.cpu.a = emulator.cpu.lsr_value(emulator.cpu.a);
		},

		// ARR
		0x6b => |emulator| {
			emulator.cpu.a &= get_operand::<Immediate>(emulator);
			let c = emulator.cpu.get_flag(Flag::C) as u8;
			emulator.cpu.a = (c << 7) | (emulator.cpu.a >> 1);
			emulator.cpu.set_flag(Flag::C, ((emulator.cpu.a >> 6) & 1) == 1);
			emulator.cpu.set_flag(Flag::V, (((emulator.cpu.a >> 6) & 1) ^ ((emulator.cpu.a >> 5) & 1)) == 1);
			emulator.cpu.set_nz_flags(emulator.cpu.a);
		},

		// AXS
		0xcb => |emulator| {
			let operand = get_operand::<Immediate>(emulator);
			emulator.cpu.x &= emulator.cpu.a;
			emulator.cpu.set_flag(Flag::C, emulator.cpu.x >= operand);
			emulator.cpu.x = emulator.cpu.x.wrapping_sub(operand);
			emulator.cpu.set_nz_flags(emulator.cpu.x);
		},

		0x09 => ora::<Immediate>,
		0x05 => ora::<ZeroPage>,
		0x15 => ora::<ZeroPageX>,
		0x0d => ora::<Absolute>,
		0x1d => ora::<AbsoluteX>,
		0x19 => ora::<AbsoluteY>,
		0x01 => ora::<IndirectX>,
		0x11 => ora::<IndirectY>,

		0x49 => eor::<Immediate>,
		0x45 => eor::<ZeroPage>,
		0x55 => eor::<ZeroPageX>,
		0x4d => eor::<Absolute>,
		0x5d => eor::<AbsoluteX>,
		0x59 => eor::<AbsoluteY>,
		0x41 => eor::<IndirectX>,
		0x51 => eor::<IndirectY>,

		0x24 => bit::<ZeroPage>,
		0x2c => bit::<Absolute>,

		0x4a => |emulator| emulator.cpu.a = emulator.cpu.lsr_value(emulator.cpu.a),
		0x46 => lsr::<ZeroPage>,
		0x56 => lsr::<ZeroPageX>,
		0x4e => lsr::<Absolute>,
		0x5e => lsr::<AbsoluteX>,

		0x47 => sre::<ZeroPage>,
		0x57 => sre::<ZeroPageX>,
		0x4f => sre::<Absolute>,
		0x5f => sre::<AbsoluteX>,
		0x5b => sre::<AbsoluteY>,
		0x43 => sre::<IndirectX>,
		0x53 => sre::<IndirectY>,

		0x0a => |emulator| emulator.cpu.a = emulator.cpu.asl_value(emulator.cpu.a),
		0x06 => asl::<ZeroPage>,
		0x16 => asl::<ZeroPageX>,
		0x0e => asl::<Absolute>,
		0x1e => asl::<AbsoluteX>,

		0x07 => slo::<ZeroPage>,
		0x17 => slo::<ZeroPageX>,
		0x0f => slo::<Absolute>,
		0x1f => slo::<AbsoluteX>,
		0x1b => slo::<AbsoluteY>,
		0x03 => slo::<IndirectX>,
		0x13 => slo::<IndirectY>,

		0x6a => |emulator| emulator.cpu.a = emulator.cpu.ror_value(emulator.cpu.a),
		0x66 => ror::<ZeroPage>,
		0x76 => ror::<ZeroPageX>,
		0x6e => ror::<Absolute>,
		0x7e => ror::<AbsoluteX>,

		0x67 => rra::<ZeroPage>,
		0x77 => rra::<ZeroPageX>,
		0x6f => rra::<Absolute>,
		0x7f => rra::<AbsoluteX>,
		0x7b => rra::<AbsoluteY>,
		0x63 => rra::<IndirectX>,
		0x73 => rra::<IndirectY>,

		0x2a => |emulator| emulator.cpu.a = emulator.cpu.rol_value(emulator.cpu.a),
		0x26 => rol::<ZeroPage>,
		0x36 => rol::<ZeroPageX>,
		0x2e => rol::<Absolute>,
		0x3e => rol::<AbsoluteX>,

		0x27 => rla::<ZeroPage>,
		0x37 => rla::<ZeroPageX>,
		0x2f => rla::<Absolute>,
		0x3f => rla::<AbsoluteX>,
		0x3b => rla::<AbsoluteY>,
		0x23 => rla::<IndirectX>,
		0x33 => rla::<IndirectY>,

		0x69 => adc::<Immediate>,
		0x65 => adc::<ZeroPage>,
		0x75 => adc::<ZeroPageX>,
		0x6d => adc::<Absolute>,
		0x7d => adc::<AbsoluteX>,
		0x79 => adc::<AbsoluteY>,
		0x61 => adc::<IndirectX>,
		0x71 => adc::<IndirectY>,

		0xe9 | 0xeb => sbc::<Immediate>,
		0xe5 => sbc::<ZeroPage>,
		0xf5 => sbc::<ZeroPageX>,
		0xed => sbc::<Absolute>,
		0xfd => sbc::<AbsoluteX>,
		0xf9 => sbc::<AbsoluteY>,
		0xe1 => sbc::<IndirectX>,
		0xf1 => sbc::<IndirectY>,
		
		// INX
		0xe8 => |emulator| emulator.cpu.x = emulator.cpu.inc_value(emulator.cpu.x),

		// INY
		0xc8 => |emulator| emulator.cpu.y = emulator.cpu.inc_value(emulator.cpu.y),

		0xe6 => inc::<ZeroPage>,
		0xf6 => inc::<ZeroPageX>,
		0xee => inc::<Absolute>,
		0xfe => inc::<AbsoluteX>,

		0xe7 => isb::<ZeroPage>,
		0xf7 => isb::<ZeroPageX>,
		0xef => isb::<Absolute>,
		0xff => isb::<AbsoluteX>,
		0xfb => isb::<AbsoluteY>,
		0xe3 => isb::<IndirectX>,
		0xf3 => isb::<IndirectY>,

		// DEX
		0xca => |emulator| emulator.cpu.x = emulator.cpu.dec_value(emulator.cpu.x),

		// DEY
		0x88 => |emulator| emulator.cpu.y = emulator.cpu.dec_value(emulator.cpu.y),

		0xc6 => dec::<ZeroPage>,
		0xd6 => dec::<ZeroPageX>,
		0xce => dec::<Absolute>,
		0xde => dec::<AbsoluteX>,

		0xc7 => dcp::<ZeroPage>,
		0xd7 => dcp::<ZeroPageX>,
		0xcf => dcp::<Absolute>,
		0xdf => dcp::<AbsoluteX>,
		0xdb => dcp::<AbsoluteY>,
		0xc3 => dcp::<IndirectX>,
		0xd3 => dcp::<IndirectY>,

		0xe0 => cpx::<Immediate>,
		0xe4 => cpx::<ZeroPage>,
		0xec => cpx::<Absolute>,

		0xc0 => cpy::<Immediate>,
		0xc4 => cpy::<ZeroPage>,
		0xcc => cpy::<Absolute>,

		0xc9 => cmp::<Immediate>,
		0xc5 => cmp::<ZeroPage>,
		0xd5 => cmp::<ZeroPageX>,
		0xcd => cmp::<Absolute>,
		0xdd => cmp::<AbsoluteX>,
		0xd9 => cmp::<AbsoluteY>,
		0xc1 => cmp::<IndirectX>,
		0xd1 => cmp::<IndirectY>,

		// PHA
		0x48 => |emulator| push8(emulator, emulator.cpu.a),

		// PLA
		0x68 => |emulator| {
			emulator.cpu.a = pull8(emulator);
			emulator.cpu.set_nz_flags(emulator.cpu.a);
		},

		// PHP
		0x08 => |emulator| push8(emulator, emulator.cpu.p | Flag::B as u8),

		0x28 => plp,

		// CLC
		0x18 => |emulator| emulator.cpu.set_flag(Flag::C, false),

		// SEC
		0x38 => |emulator| emulator.cpu.set_flag(Flag::C, true),

		// CLI
		0x58 => |emulator| emulator.cpu.set_flag(Flag::I, false),

		// SEI
		0x78 => |emulator| emulator.cpu.set_flag(Flag::I, true),

		// CLD
		0xd8 => |emulator| emulator.cpu.set_flag(Flag::D, false),

		// SED
		0xf8 => |emulator| emulator.cpu.set_flag(Flag::D, true),

		// CLV
		0xb8 => |emulator| emulator.cpu.set_flag(Flag::V, false),

		// JMP (absolute)
		0x4c => |emulator| emulator.cpu.pc = read16(emulator, emulator.cpu.pc),

		// JMP (indirect)
		0x6c => |emulator| {
			let address = read16(emulator, emulator.cpu.pc);
			let low_byte = read8(emulator, address) as u16;
			let high_byte = read8(emulator, (address & 0xff00) | (address.wrapping_add(1) & 0x00ff)) as u16;
			emulator.cpu.pc = (high_byte << 8) | low_byte;
		},

		// BPL
		0x10 => |emulator| branch(emulator, Flag::N, false),

		// BMI
		0x30 => |emulator| branch(emulator, Flag::N, true),

		// BVC
		0x50 => |emulator| branch(emulator, Flag::V, false),

		// BVS
		0x70 => |emulator| branch(emulator, Flag::V, true),

		// BCC
		0x90 => |emulator| branch(emulator, Flag::C, false),

		// BCS
		0xb0 => |emulator| branch(emulator, Flag::C, true),

		// BNE
		0xd0 => |emulator| branch(emulator, Flag::Z, false),

		// BEQ
		0xf0 => |emulator| branch(emulator, Flag::Z, true),

		// BRK
		0x00 => |emulator| {
			let address = emulator.cpu.pc.wrapping_add(1);
			push16(emulator, address);
			push8(emulator, emulator.cpu.p | Flag::B as u8);
			emulator.cpu.pc = read16(emulator, IRQ_VECTOR_ADDRESS);
			emulator.cpu.set_flag(Flag::I, true);
		},

		// JSR
		0x20 => |emulator| {
			let address = emulator.cpu.pc.wrapping_add(1);
			push16(emulator, address);
			emulator.cpu.pc = read16(emulator, emulator.cpu.pc);
		},

		// RTI
		0x40 => |emulator| {
			plp(emulator);
			emulator.cpu.pc = pull16(emulator);
		},

		// RTS
		0x60 => |emulator| emulator.cpu.pc = pull16(emulator).wrapping_add(1),

		// KIL
		0x02 | 0x32 => |_| panic!("CPU stopped"),

		_ => |emulator| {
			let pc = emulator.cpu.pc.wrapping_sub(1);
			let opcode = read8(emulator, pc);
			panic!("Unknown opcode {:02X} at {:04X}", opcode, pc);
		}
	}
}

fn read_next8(emulator: &mut Emulator) -> u8 {
	let value = read8(emulator, emulator.cpu.pc);
	emulator.cpu.pc = emulator.cpu.pc.wrapping_add(1);
	value
}

fn read_next16(emulator: &mut Emulator) -> u16 {
	let value = read16(emulator, emulator.cpu.pc);
	emulator.cpu.pc = emulator.cpu.pc.wrapping_add(2);
	value
}

fn perform_interrupt(emulator: &mut Emulator, address: u16) {
	push16(emulator, emulator.cpu.pc);
	push8(emulator, emulator.cpu.p);
	emulator.cpu.pc = read16(emulator, address);
	emulator.cpu.set_flag(Flag::I, true);
	emulator.cpu.pending_interrupt = None;
}

fn push8(emulator: &mut Emulator, value: u8) {
	write(emulator, STACK_ADDRESS + emulator.cpu.s as u16, value);
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
	emulator.cpu.set_nz_flags(emulator.cpu.a);
}

fn ldx<T: AddressingMode>(emulator: &mut Emulator) {
	emulator.cpu.x = get_operand::<T>(emulator);
	emulator.cpu.set_nz_flags(emulator.cpu.x);
}

fn ldy<T: AddressingMode>(emulator: &mut Emulator) {
	emulator.cpu.y = get_operand::<T>(emulator);
	emulator.cpu.set_nz_flags(emulator.cpu.y);
}

fn lax<T: AddressingMode>(emulator: &mut Emulator) {
	emulator.cpu.a = get_operand::<T>(emulator);
	emulator.cpu.x = emulator.cpu.a;
	emulator.cpu.set_nz_flags(emulator.cpu.x);
}

fn sta<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	write(emulator, address, emulator.cpu.a);
}

fn stx<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	write(emulator, address, emulator.cpu.x);
}

fn sty<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	write(emulator, address, emulator.cpu.y);
}

fn sax<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	write(emulator, address, emulator.cpu.a & emulator.cpu.x);
}

fn and_address(emulator: &mut Emulator, address: u16) {
	emulator.cpu.a &= read8(emulator, address);
	emulator.cpu.set_nz_flags(emulator.cpu.a);
}

fn and<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	and_address(emulator, address);
}

fn ora_address(emulator: &mut Emulator, address: u16) {
	emulator.cpu.a |= read8(emulator, address);
	emulator.cpu.set_nz_flags(emulator.cpu.a);
}

fn ora<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	ora_address(emulator, address);
}

fn eor_address(emulator: &mut Emulator, address: u16) {
	emulator.cpu.a ^= read8(emulator, address);
	emulator.cpu.set_nz_flags(emulator.cpu.a);
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
	write(emulator, address, result);
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
	write(emulator, address, result);
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
	write(emulator, address, result);
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
	write(emulator, address, result);
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
	let operand = read8(emulator, address);
	let result = emulator.cpu.inc_value(operand);
	write(emulator, address, result);
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
	let operand = read8(emulator, address);
	let result = emulator.cpu.dec_value(operand);
	write(emulator, address, result);
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
	emulator.cpu.set_nz_flags(result);
}

fn cpy<T: AddressingMode>(emulator: &mut Emulator) {
	let operand = get_operand::<T>(emulator);
	emulator.cpu.set_flag(Flag::C, emulator.cpu.y >= operand);
	let result = emulator.cpu.y.wrapping_sub(operand);
	emulator.cpu.set_nz_flags(result);
}

fn cmp_address(emulator: &mut Emulator, address: u16) {
	let operand = read8(emulator, address);
	emulator.cpu.set_flag(Flag::C, emulator.cpu.a >= operand);
	let result = emulator.cpu.a.wrapping_sub(operand);
	emulator.cpu.set_nz_flags(result);
}

fn cmp<T: AddressingMode>(emulator: &mut Emulator) {
	let address = T::get_address(emulator);
	cmp_address(emulator, address);
}

fn branch(emulator: &mut Emulator, flag: Flag, value: bool) {
	let offset = read_next8(emulator);
	if emulator.cpu.get_flag(flag) == value {
		let pc = emulator.cpu.pc;
		emulator.cpu.pc = pc.wrapping_add(offset as i8 as _);
		emulator.cpu.check_page_crossing(pc, emulator.cpu.pc);
		emulator.cpu.branch_taken = true;
	}
}

fn plp(emulator: &mut Emulator) {
	let value = pull8(emulator);
	emulator.cpu.p = (value | 0x20) & !(Flag::B as u8);
}
