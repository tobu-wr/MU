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
		let mut ppu = Ppu::new();
		Self {
			cpu_memory: CpuMemory::new(&mut ppu),
			cpu: Cpu::new(),
			ppu_memory: PpuMemory::new(),
			ppu,
			window: Window::new("RNES", FRAME_WIDTH, FRAME_HEIGHT, WindowOptions::default()).unwrap()
		}
	}

	pub fn load_rom(&mut self, filename: &String) {
		self.cpu_memory.load_rom(filename);

		// TODO: move it to a test module, so we can get rid of '#[cfg(not(feature = "nestest"))]' below
		#[cfg(feature = "nestest")]
		self.cpu.set_pc(NESTEST_START_ADDRESS);
		
		#[cfg(not(feature = "nestest"))]
		{
			let value = self.cpu_memory.read16(RESET_VECTOR_ADDRESS);
			self.cpu.set_pc(value);
		}
	}

	pub fn run(&mut self) {
		while self.window.is_open() {
			self.cpu.execute_next_instruction(&mut self.cpu_memory);
			self.ppu.do_cycle(&self.ppu_memory, &mut self.window);
		}
	}
}
