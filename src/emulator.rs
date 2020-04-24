use minifb::{Window, WindowOptions};

use cpu_memory::*;
use ppu_memory::*;
use cpu::*;
use ppu::*;

pub struct Emulator {
	cpu_memory: CpuMemory,
	ppu_memory: PpuMemory,
	cpu: Cpu,
	ppu: Ppu,
	window: Window
}

impl Emulator {
	pub fn new() -> Self {
		Self {
			cpu_memory: CpuMemory::new(),
			ppu_memory: PpuMemory::new(),
			cpu: Cpu::new(),
			ppu: Ppu::new(),
			window: Window::new("RNES", FRAME_WIDTH, FRAME_HEIGHT, WindowOptions::default()).unwrap()
		}
	}

	pub fn init(&mut self) {
		self.cpu_memory.connect(&mut self.ppu);
		self.ppu.connect(&mut self.cpu, &mut self.ppu_memory);
	}

	pub fn load_file(&mut self, filename: &str) {
		let contents = std::fs::read(filename).unwrap();
		if &contents[..4] != b"NES\x1a" {
			println!("[ERROR] Wrong file format");
			std::process::exit(1);
		}

		let prg_rom_size = (contents[4] * 16) as usize;
		println!("[INFO] PRG ROM size: {}KB", prg_rom_size);
		
		let prg_rom_start = 16;
		let prg_rom_end = prg_rom_start + prg_rom_size * 1024;
		let prg_rom = &contents[prg_rom_start..prg_rom_end];
		self.cpu_memory.load_prg_rom(prg_rom); 
		
		let chr_rom_size = (contents[5] * 8) as usize;
		println!("[INFO] CHR ROM size: {}KB", chr_rom_size);

		let chr_rom_start = prg_rom_end;
		let chr_rom_end = chr_rom_start + chr_rom_size * 1024;
		let chr_rom = &contents[chr_rom_start..chr_rom_end];
		self.ppu_memory.load_chr_rom(chr_rom); 
		
		let mapper = (contents[7] & 0xf0) | (contents[6] >> 4);
		println!("[INFO] Mapper: {}", mapper);

		let value = self.cpu_memory.read16(RESET_VECTOR_ADDRESS);
		self.cpu.set_pc(value);
	}

	pub fn run(&mut self) {
		while self.window.is_open() {
			self.cpu.execute_next_instruction(&mut self.cpu_memory);
			self.ppu.do_cycle(&mut self.window);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn nestest() {
		let mut emulator = Emulator::new();
		emulator.cpu_memory.load_rom("tests/nestest.nes");
		emulator.cpu.set_pc(0xc000);

		for counter in 0..8992 {
			emulator.cpu.execute_next_instruction(&mut emulator.cpu_memory);
		}

		assert_eq!(emulator.cpu_memory.read8(0x02), 0);
		assert_eq!(emulator.cpu_memory.read8(0x03), 0);
	}
}
