use window::*;

pub struct Joypad {
    register: u8,
    polling: bool
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            register: 0,
            polling: false
        }
    }

    pub fn read(&mut self, window: &Window) -> u8 {
        if self.polling {
            window.is_key_down(Key::Q) as u8
        } else {
            let state = self.register & 1;
            self.register >>= 1;
            state
        }
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(&self, window: &Window) -> u8 {
        if self.polling {
            window.is_key_down(Key::Q) as u8
        } else {
            self.register & 1
        }
    }

    pub fn write(&mut self, window: &Window, value: u8) {
        if (value & 1) == 1 {
            self.polling = true;
        } else if self.polling {
            self.polling = false;
            self.register = (window.is_key_down(Key::Q) as u8) << 0
                          | (window.is_key_down(Key::W) as u8) << 1
                          | (window.is_key_down(Key::Space) as u8) << 2
                          | (window.is_key_down(Key::Enter) as u8) << 3
                          | (window.is_key_down(Key::Up) as u8) << 4
                          | (window.is_key_down(Key::Down) as u8) << 5
                          | (window.is_key_down(Key::Left) as u8) << 6
                          | (window.is_key_down(Key::Right) as u8) << 7;
        }
    }
}
