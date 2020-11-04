use super::*;

use std::fs::File;
use std::io::{BufReader, BufRead};

pub fn read8_debug(emulator: &Emulator, address: u16) -> u8 {
	memory::read8_debug(emulator, address)
}

#[test]
fn nestest() {
	let mut emulator = Emulator::new();
	emulator.load_file("tests/cpu/nestest/nestest.nes");
	emulator.cpu.pc = 0xc000;

	let mut cycle_counter: u16 = 7;

	let log_file = File::open("tests/cpu/nestest/nestest.log").unwrap();
	for log in BufReader::new(log_file).lines().map(|line| line.unwrap()) {
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

		let cycle = format!("{}", cycle_counter);
		let expected_cycle = &log[90..];
		assert_eq!(cycle, expected_cycle);

		cycle_counter += Cpu::execute_next_instruction(&mut emulator) as u16;
	}
}

fn run_test(filename: &str) {
	let mut emulator = Emulator::new();
	emulator.load_file(filename);
	Cpu::init_pc(&mut emulator);
	while read8_debug(&emulator, 0x6000) != 0x80 {
		Cpu::execute_next_instruction(&mut emulator);
	}
	while read8_debug(&emulator, 0x6000) == 0x80 {
		Cpu::execute_next_instruction(&mut emulator);
	}
	assert_eq!(read8_debug(&emulator, 0x6000), 0);
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

#[test]
#[ignore] // TODO: implement APU length counter
fn instr_timing() {
	run_test("tests/cpu/instr_timing/1-instr_timing.nes");
}

#[test]
#[ignore] // TODO: implement APU length counter
fn branch_timing() {
	run_test("tests/cpu/instr_timing/2-branch_timing.nes");
}

#[test]
#[ignore]
fn cli_latency() {
	run_test("tests/cpu/cpu_interrupts_v2/1-cli_latency.nes");
}

#[test]
#[ignore]
fn nmi_and_brk() {
	run_test("tests/cpu/cpu_interrupts_v2/2-nmi_and_brk.nes");
}

#[test]
#[ignore]
fn nmi_and_irq() {
	run_test("tests/cpu/cpu_interrupts_v2/3-nmi_and_irq.nes");
}

#[test]
#[ignore]
fn irq_and_dma() {
	run_test("tests/cpu/cpu_interrupts_v2/4-irq_and_dma.nes");
}

#[test]
#[ignore]
fn branch_delays_irq() {
	run_test("tests/cpu/cpu_interrupts_v2/5-branch_delays_irq.nes");
}

#[test]
#[ignore]
fn exec_space_ppuio() {
	run_test("tests/cpu/cpu_exec_space/test_cpu_exec_space_ppuio.nes");
}

#[test]
#[ignore]
fn exec_space_apu() {
	run_test("tests/cpu/cpu_exec_space/test_cpu_exec_space_apu.nes");
}
