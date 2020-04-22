const SIZE: usize = 0x4000;

pub struct PpuMemory {
	data: [u8; SIZE]
}

impl PpuMemory {
	pub fn new() -> Self {
		Self {
			data: [0; SIZE]
		}
	}

	pub fn read(&self, address: u16) -> u8 {
		self.data[address as usize]
	}

	pub fn write(&mut self, address: u16, value: u8) {
		self.data[address as usize] = value;
	}
}
