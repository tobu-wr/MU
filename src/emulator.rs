use minifb::{Window, WindowOptions};

use cpu_memory::*;
use cpu::*;
use ppu_memory::*;
use ppu::*;

pub struct Emulator {
	cpu_memory: CpuMemory,
	cpu: Cpu,
	ppu_memory: PpuMemory,
	ppu: Ppu,
	window: Window
}

impl Emulator {
	pub fn new() -> Self {
		Self {
			cpu_memory: CpuMemory::new(),
			cpu: Cpu::new(),
			ppu_memory: PpuMemory::new(),
			ppu: Ppu::new(),
			window: Window::new("RNES", FRAME_WIDTH, FRAME_HEIGHT, WindowOptions::default()).unwrap()
		}
	}

	pub fn init(&mut self) {
		self.cpu_memory.connect(&mut self.ppu);
	}

	pub fn load_rom(&mut self, filename: &str) {
		self.cpu_memory.load_rom(filename);

		let value = self.cpu_memory.read16(RESET_VECTOR_ADDRESS);
		self.cpu.set_pc(value);
	}

	pub fn run(&mut self) {
		while self.window.is_open() {
			self.cpu.execute_next_instruction(&mut self.cpu_memory);
			self.ppu.do_cycle(&self.ppu_memory, &mut self.window);
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

		let mut counter = 0;
		while counter < 8991 {
			emulator.cpu.execute_next_instruction(&mut emulator.cpu_memory);
			counter += 1;
		}

		assert_eq!(emulator.cpu_memory.read8(0x02), 0);
		assert_eq!(emulator.cpu_memory.read8(0x03), 0);
	}
}
