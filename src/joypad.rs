use minifb::{Key, Window};

pub struct Joypad {
    window: *const Window,
    register: u8,
    polling: bool
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            window: std::ptr::null(),
            register: 0,
            polling: false
        }
    }

    pub fn connect(&mut self, window: *const Window) {
        self.window = window;
    }

    fn is_key_down(&self, key: Key) -> u8 {
        unsafe {
            (*self.window).is_key_down(key) as u8
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.polling {
            self.is_key_down(Key::A)
        } else {
            let state = self.register & 1;
            self.register >>= 1;
            state
        }
    }

    pub fn write(&mut self, value: u8) {
        if (value & 1) == 1 {
            self.polling = true;
        } else if self.polling {
            self.polling = false;
            self.register = self.is_key_down(Key::Q) << 0
                          | self.is_key_down(Key::W) << 1
                          | self.is_key_down(Key::Space) << 2
                          | self.is_key_down(Key::Enter) << 3
                          | self.is_key_down(Key::Up) << 4
                          | self.is_key_down(Key::Down) << 5
                          | self.is_key_down(Key::Left) << 6
                          | self.is_key_down(Key::Right) << 7;
        }
    }
}
