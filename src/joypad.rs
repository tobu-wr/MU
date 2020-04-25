use minifb::{Key, Window};

const BUTTON_COUNT: usize = 8;

pub struct Joypad {
    window: *const Window,
    buttons: [u8; BUTTON_COUNT],
    polling: bool,
    index: usize
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            window: std::ptr::null(),
            buttons: [0; BUTTON_COUNT],
            polling: false,
            index: 0
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
        } else if self.index < BUTTON_COUNT {
            let state = self.buttons[self.index];
            self.index += 1;
            state
        } else {
            1
        }
    }

    pub fn write(&mut self, value: u8) {
        if (value & 1) == 1 {
             // start polling
            self.polling = true;
        } else if self.polling {
             // end polling
            self.polling = false;
            self.index = 0;
            self.buttons[0] = self.is_key_down(Key::A);
            self.buttons[1] = self.is_key_down(Key::B);
            self.buttons[2] = self.is_key_down(Key::Space);
            self.buttons[3] = self.is_key_down(Key::Enter);
            self.buttons[4] = self.is_key_down(Key::Up);
            self.buttons[5] = self.is_key_down(Key::Down);
            self.buttons[6] = self.is_key_down(Key::Left);
            self.buttons[7] = self.is_key_down(Key::Right);
        }
    }
}
