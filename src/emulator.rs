use mappers::*;
use cpu::*;
use ppu::*;
use apu::*;
use joypad::*;
use screen::*;

pub const RAM_SIZE: usize = 0x800;

pub struct Emulator {
	pub ram: [u8; RAM_SIZE],
	pub mapper: Option<Box<dyn Mapper>>,
	pub cpu: Cpu,
	pub ppu: Ppu,
	pub apu: Apu,
	pub joypad: Joypad,
	pub screen: Screen
}

impl Emulator {
	pub fn new() -> Self {
		Self {
			ram: [0; RAM_SIZE],
			mapper: None,
			cpu: Cpu::new(),
			ppu: Ppu::new(),
			apu: Apu::new(),
			joypad: Joypad::new(),
			screen: Screen::new()
		}
	}

	pub fn load_file(&mut self, filename: &str) {
		let contents = std::fs::read(filename).unwrap();
		if &contents[..4] != b"NES\x1a" {
			panic!("Wrong file format");
		}

		let prg_rom_size = contents[4] as usize * 16;
		info!("PRG ROM size: {}KB", prg_rom_size);
		
		let prg_rom_start = 16;
		let prg_rom_end = prg_rom_start + prg_rom_size * 1024;
		let prg_rom = &contents[prg_rom_start..prg_rom_end];
		
		let chr_rom_size = contents[5] as usize * 8;
		info!("CHR ROM size: {}KB", chr_rom_size);

		let chr_rom_start = prg_rom_end;
		let chr_rom_end = chr_rom_start + chr_rom_size * 1024;
		let chr_rom = &contents[chr_rom_start..chr_rom_end];
		self.ppu.load_chr_rom(chr_rom);
		
		let mapper_number = (contents[7] & 0xf0) | (contents[6] >> 4);
		info!("Cartridge mapper: {}", mapper_number);
		self.mapper = Some(create_mapper(mapper_number, prg_rom));

		Cpu::init_pc(self);
	}

	pub fn step(&mut self) {
		let cycles = 3 * self.cpu.execute_next_instruction();
		for _ in 0..cycles {
			self.ppu.do_cycle(&mut self.cpu, &mut self.screen);
		}
	}
}
