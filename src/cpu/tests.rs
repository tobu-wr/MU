use super::*;

#[test]
fn nestest() {
	let mut emulator = Emulator::new();
	emulator.load_file("tests/nestest.nes");
	emulator.cpu.set_pc(0xc000);
	for _ in 0..8991 {
		Cpu::execute_next_instruction(&mut emulator);
	}
	assert_eq!(read8(&mut emulator, 0x02), 0);
	assert_eq!(read8(&mut emulator, 0x03), 0);
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
	run_test("tests/01-basics.nes");
}

#[test]
fn implied() {
	run_test("tests/02-implied.nes");
}

#[test]
fn immediate() {
	run_test("tests/03-immediate.nes");
}

#[test]
fn zero_page() {
	run_test("tests/04-zero_page.nes");
}

#[test]
fn zp_xy() {
	run_test("tests/05-zp_xy.nes");
}

#[test]
fn absolute() {
	run_test("tests/06-absolute.nes");
}

#[test]
#[ignore] // FIXME
fn abs_xy() {
	run_test("tests/07-abs_xy.nes");
}

#[test]
fn ind_x() {
	run_test("tests/08-ind_x.nes");
}

#[test]
fn ind_y() {
	run_test("tests/09-ind_y.nes");
}

#[test]
fn branches() {
	run_test("tests/10-branches.nes");
}

#[test]
fn stack() {
	run_test("tests/11-stack.nes");
}

#[test]
fn jmp_jsr() {
	run_test("tests/12-jmp_jsr.nes");
}

#[test]
fn rts() {
	run_test("tests/13-rts.nes");
}

#[test]
fn rti() {
	run_test("tests/14-rti.nes");
}

#[test]
fn brk() {
	run_test("tests/15-brk.nes");
}

#[test]
fn special() {
	run_test("tests/16-special.nes");
}
