use super::*;

use std::io::{BufReader, BufRead};

pub fn read8(emulator: &mut Emulator, address: u16) -> u8 {
	memory::read8(emulator, address)
}

#[test]
fn nestest() {
	let mut emulator = Emulator::new();
	emulator.load_file("tests/cpu/nestest/nestest.nes");
	emulator.cpu.set_pc(0xc000);

	let log_file = std::fs::File::open("tests/cpu/nestest/nestest.log").unwrap();
	for line in BufReader::new(log_file).lines() {
		// TODO: compare logger output with nestest logs
		let log = line.unwrap();
		
		let pc = format!("{:04X}", emulator.cpu.pc);
		let expected_pc = &log[..4];
		assert_eq!(pc, expected_pc);

		let a = format!("{:02X}", emulator.cpu.a);
		let expected_a = &log[50..52];
		assert_eq!(a, expected_a);

		let x = format!("{:02X}", emulator.cpu.x);
		let expected_x = &log[55..57];
		assert_eq!(x, expected_x);

		let y = format!("{:02X}", emulator.cpu.y);
		let expected_y = &log[60..62];
		assert_eq!(y, expected_y);

		let p = format!("{:02X}", emulator.cpu.p);
		let expected_p = &log[65..67];
		assert_eq!(p, expected_p);

		let s = format!("{:02X}", emulator.cpu.s);
		let expected_s = &log[71..73];
		assert_eq!(s, expected_s);

		Cpu::execute_next_instruction(&mut emulator);
	}
}

fn run_test(filename: &str) {
	let mut emulator = Emulator::new();
	emulator.load_file(filename);
	Cpu::init_pc(&mut emulator);
	while read8(&mut emulator, 0x6000) != 0x80 {
		Cpu::execute_next_instruction(&mut emulator);
	}
	while read8(&mut emulator, 0x6000) == 0x80 {
		Cpu::execute_next_instruction(&mut emulator);
	}
	assert_eq!(read8(&mut emulator, 0x6000), 0);
}

#[test]
fn basics() {
	run_test("tests/cpu/instr_test_v5/01-basics.nes");
}

#[test]
fn implied() {
	run_test("tests/cpu/instr_test_v5/02-implied.nes");
}

#[test]
fn immediate() {
	run_test("tests/cpu/instr_test_v5/03-immediate.nes");
}

#[test]
fn zero_page() {
	run_test("tests/cpu/instr_test_v5/04-zero_page.nes");
}

#[test]
fn zp_xy() {
	run_test("tests/cpu/instr_test_v5/05-zp_xy.nes");
}

#[test]
fn absolute() {
	run_test("tests/cpu/instr_test_v5/06-absolute.nes");
}

#[test]
fn abs_xy() {
	run_test("tests/cpu/instr_test_v5/07-abs_xy.nes");
}

#[test]
fn ind_x() {
	run_test("tests/cpu/instr_test_v5/08-ind_x.nes");
}

#[test]
fn ind_y() {
	run_test("tests/cpu/instr_test_v5/09-ind_y.nes");
}

#[test]
fn branches() {
	run_test("tests/cpu/instr_test_v5/10-branches.nes");
}

#[test]
fn stack() {
	run_test("tests/cpu/instr_test_v5/11-stack.nes");
}

#[test]
fn jmp_jsr() {
	run_test("tests/cpu/instr_test_v5/12-jmp_jsr.nes");
}

#[test]
fn rts() {
	run_test("tests/cpu/instr_test_v5/13-rts.nes");
}

#[test]
fn rti() {
	run_test("tests/cpu/instr_test_v5/14-rti.nes");
}

#[test]
fn brk() {
	run_test("tests/cpu/instr_test_v5/15-brk.nes");
}

#[test]
fn special() {
	run_test("tests/cpu/instr_test_v5/16-special.nes");
}

#[test]
fn abs_x_wrap() {
	run_test("tests/cpu/instr_misc/01-abs_x_wrap.nes");
}

#[test]
fn branch_wrap() {
	run_test("tests/cpu/instr_misc/02-branch_wrap.nes");
}
