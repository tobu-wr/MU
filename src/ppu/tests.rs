use super::*;
use emulator::*;
use cpu::tests::*;

#[test]
fn oam_read() {
	let mut emulator = Emulator::new();
	emulator.load_file("tests/ppu/oam_read.nes");
	Cpu::init_pc(&mut emulator);
	while read8_debug(&emulator, 0x6000) != 0x80 {
		let cycles = Cpu::execute_next_instruction(&mut emulator);
		for _ in 0..cycles {
			emulator.ppu.do_cycle(&mut emulator.cpu, &mut emulator.window);
		}
	}
	while read8_debug(&emulator, 0x6000) == 0x80 {
		let cycles = Cpu::execute_next_instruction(&mut emulator);
		for _ in 0..cycles {
			emulator.ppu.do_cycle(&mut emulator.cpu, &mut emulator.window);
		}
	}
	assert_eq!(read8_debug(&emulator, 0x6000), 0);
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
