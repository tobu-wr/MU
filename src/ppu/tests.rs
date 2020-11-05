use super::*;
use emulator::*;
use cpu::tests::*;

fn run_test(filename: &str) {
	let mut emulator = Emulator::new();
	emulator.load_file(filename);
	Cpu::init_pc(&mut emulator);
	while read8_debug(&emulator, 0x6000) != 0x80 {
		let cycles = 3 * Cpu::execute_next_instruction(&mut emulator);
		for _ in 0..cycles {
			emulator.ppu.do_cycle(&mut emulator.cpu, &mut emulator.window);
		}
	}
	while read8_debug(&emulator, 0x6000) == 0x80 {
		let cycles = 3 * Cpu::execute_next_instruction(&mut emulator);
		for _ in 0..cycles {
			emulator.ppu.do_cycle(&mut emulator.cpu, &mut emulator.window);
		}
	}
	assert_eq!(read8_debug(&emulator, 0x6000), 0);
}

#[test]
#[ignore]
fn vbl_basics() {
	run_test("tests/ppu/ppu_vbl_nmi/01-vbl_basics.nes");
}

#[test]
#[ignore]
fn vbl_set_time() {
	run_test("tests/ppu/ppu_vbl_nmi/02-vbl_set_time.nes");
}

#[test]
fn vbl_clear_time() {
	run_test("tests/ppu/ppu_vbl_nmi/03-vbl_clear_time.nes");
}

#[test]
#[ignore]
fn nmi_control() {
	run_test("tests/ppu/ppu_vbl_nmi/04-nmi_control.nes");
}

#[test]
#[ignore]
fn nmi_timing() {
	run_test("tests/ppu/ppu_vbl_nmi/05-nmi_timing.nes");
}

#[test]
#[ignore]
fn suppression() {
	run_test("tests/ppu/ppu_vbl_nmi/06-suppression.nes");
}

#[test]
#[ignore]
fn nmi_on_timing() {
	run_test("tests/ppu/ppu_vbl_nmi/07-nmi_on_timing.nes");
}

#[test]
#[ignore]
fn nmi_off_timing() {
	run_test("tests/ppu/ppu_vbl_nmi/08-nmi_off_timing.nes");
}

#[test]
#[ignore]
fn even_odd_frames() {
	run_test("tests/ppu/ppu_vbl_nmi/09-even_odd_frames.nes");
}

#[test]
#[ignore]
fn even_odd_timing() {
	run_test("tests/ppu/ppu_vbl_nmi/10-even_odd_timing.nes");
}

#[test]
#[ignore]
fn basics() {
	run_test("tests/ppu/ppu_sprite_hit/01-basics.nes");
}

#[test]
#[ignore]
fn alignment() {
	run_test("tests/ppu/ppu_sprite_hit/02-alignment.nes");
}

#[test]
#[ignore]
fn corner() {
	run_test("tests/ppu/ppu_sprite_hit/03-corner.nes");
}

#[test]
#[ignore]
fn flip() {
	run_test("tests/ppu/ppu_sprite_hit/04-flip.nes");
}

#[test]
#[ignore]
fn left_clip() {
	run_test("tests/ppu/ppu_sprite_hit/05-left_clip.nes");
}

#[test]
#[ignore]
fn right_edge() {
	run_test("tests/ppu/ppu_sprite_hit/06-right_edge.nes");
}

#[test]
#[ignore]
fn screen_bottom() {
	run_test("tests/ppu/ppu_sprite_hit/07-screen_bottom.nes");
}

#[test]
#[ignore]
fn double_height() {
	run_test("tests/ppu/ppu_sprite_hit/08-double_height.nes");
}

#[test]
#[ignore]
fn timing() {
	run_test("tests/ppu/ppu_sprite_hit/09-timing.nes");
}

#[test]
#[ignore]
fn timing_order() {
	run_test("tests/ppu/ppu_sprite_hit/10-timing_order.nes");
}

#[test]
fn oam_read() {
	run_test("tests/ppu/oam_read.nes");
}

#[test]
#[ignore]
fn palette_ram() {
	let mut emulator = Emulator::new();
	emulator.load_file("tests/ppu/blargg_ppu_tests/palette_ram.nes");

	// TODO
	/*while  {
		Cpu::execute_next_instruction(&mut emulator);
		emulator.ppu.do_cycle(&mut emulator.cpu, &mut emulator.window);
	}*/
}

#[test]
#[ignore]
fn sprite_ram() {
	let mut emulator = Emulator::new();
	emulator.load_file("tests/ppu/blargg_ppu_tests/sprite_ram.nes");

	// TODO
	/*while  {
		Cpu::execute_next_instruction(&mut emulator);
		emulator.ppu.do_cycle(&mut emulator.cpu, &mut emulator.window);
	}*/
}
