use minifb::{Window, WindowOptions};
use mappers::*;
use cpu::*;
use ppu::*;
use joypad::*;

pub const EMULATOR_NAME: &str = "KirbyNES";
pub const RAM_SIZE: u16 = 0x800;

pub struct Emulator {
	pub ram: [u8; RAM_SIZE as _],
	pub mapper: Option<Box<dyn Mapper>>,
	pub cpu: Cpu,
	pub ppu: Ppu,
	pub joypad: Joypad,
	pub window: Window
}

impl Emulator {
	pub fn new() -> Self {
		let options = WindowOptions{ resize: true, ..WindowOptions::default() };
		let mut window = Window::new(EMULATOR_NAME, FRAME_WIDTH, FRAME_HEIGHT, options).unwrap();

		#[cfg(not(feature = "benchmark"))]
		window.limit_update_rate(Some(std::time::Duration::from_nanos(1_000_000_000 / 60)));

		Self {
			ram: [0; RAM_SIZE as _],
			mapper: None,
			cpu: Cpu::new(),
			ppu: Ppu::new(),
			joypad: Joypad::new(),
			window
		}
	}

	pub fn load_file(&mut self, filename: &str) {
		let contents = std::fs::read(filename).unwrap();
		if &contents[..4] != b"NES\x1a" {
			panic!("Wrong file format");
		}

		let prg_rom_size = (contents[4] * 16) as usize;
		info!("PRG ROM size: {}KB", prg_rom_size);
		
		let prg_rom_start = 16;
		let prg_rom_end = prg_rom_start + prg_rom_size * 1024;
		let prg_rom = &contents[prg_rom_start..prg_rom_end];
		
		let chr_rom_size = (contents[5] * 8) as usize;
		info!("CHR ROM size: {}KB", chr_rom_size);

		let chr_rom_start = prg_rom_end;
		let chr_rom_end = chr_rom_start + chr_rom_size * 1024;
		let chr_rom = &contents[chr_rom_start..chr_rom_end];
		self.ppu.load_chr_rom(chr_rom);
		
		let mapper_number = (contents[7] & 0xf0) | (contents[6] >> 4);
		info!("Cartridge mapper: {}", mapper_number);
		self.mapper = Some(create_mapper(mapper_number, prg_rom));
	}

	pub fn run(&mut self) {
		Cpu::init_pc(self);
		while self.window.is_open() {
			let cycles = Cpu::execute_next_instruction(self);
			for _ in 0..cycles {
				self.ppu.do_cycle(&mut self.cpu, &mut self.window);
			}
		}
	}
}
