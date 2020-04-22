pub struct PpuMemory {
	data: [u8; 0x4000]
}

impl PpuMemory {
	pub fn new() -> Self {
		Self {
			data: [0; 0x4000]
		}
	}

	pub fn write(&mut self, address: u16, value: u8) {
		self.data[address as usize] = value;
	}
}
