use minifb::Window;

pub struct Joypad {
    window: *const Window
}

impl Joypad {
    pub fn new() -> Self {
		Self {
			window: std::ptr::null()
		}
    }

    pub fn connect(&mut self, window: *const Window) {
        self.window = window;
    }

    pub fn read(&self) -> u8 {
        0
    }

    pub fn write(&mut self, value: u8) {
        // TODO
    }
}