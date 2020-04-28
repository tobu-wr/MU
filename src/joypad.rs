use minifb::Key;

use emulator::*;

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

    pub fn read(emulator: &mut Emulator) -> u8 {
        if emulator.joypad.polling {
            emulator.window.is_key_down(Key::A) as u8
        } else {
            let state = emulator.joypad.register & 1;
            emulator.joypad.register >>= 1;
            state
        }
    }

    pub fn write(emulator: &mut Emulator, value: u8) {
        if (value & 1) == 1 {
            emulator.joypad.polling = true;
        } else if emulator.joypad.polling {
            emulator.joypad.polling = false;
            emulator.joypad.register = (emulator.window.is_key_down(Key::Q) as u8) << 0
                                     | (emulator.window.is_key_down(Key::W) as u8) << 1
                                     | (emulator.window.is_key_down(Key::Space) as u8) << 2
                                     | (emulator.window.is_key_down(Key::Enter) as u8) << 3
                                     | (emulator.window.is_key_down(Key::Up) as u8) << 4
                                     | (emulator.window.is_key_down(Key::Down) as u8) << 5
                                     | (emulator.window.is_key_down(Key::Left) as u8) << 6
                                     | (emulator.window.is_key_down(Key::Right) as u8) << 7;
        }
    }
}
