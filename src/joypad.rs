pub struct Joypad {
    register: u8,
    strobe: bool,
    a_button_down: bool,
    b_button_down: bool,
    select_button_down: bool,
    start_button_down: bool,
    up_button_down: bool,
    down_button_down: bool,
    left_button_down: bool,
    right_button_down: bool
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            register: 0,
            strobe: false,
            a_button_down: false,
            b_button_down: false,
            select_button_down: false,
            start_button_down: false,
            up_button_down: false,
            down_button_down: false,
            left_button_down: false,
            right_button_down: false
        }
    }

    pub fn press_a_button(&mut self) {
        self.a_button_down = true;
    }

    pub fn press_b_button(&mut self) {
        self.b_button_down = true;
    }

    pub fn press_select_button(&mut self) {
        self.select_button_down = true;
    }

    pub fn press_start_button(&mut self) {
        self.start_button_down = true;
    }

    pub fn press_up_button(&mut self) {
        self.up_button_down = true;
    }

    pub fn press_down_button(&mut self) {
        self.down_button_down = true;
    }

    pub fn press_left_button(&mut self) {
        self.left_button_down = true;
    }

    pub fn press_right_button(&mut self) {
        self.right_button_down = true;
    }

    pub fn release_a_button(&mut self) {
        self.a_button_down = false;
    }

    pub fn release_b_button(&mut self) {
        self.b_button_down = false;
    }

    pub fn release_select_button(&mut self) {
        self.select_button_down = false;
    }

    pub fn release_start_button(&mut self) {
        self.start_button_down = false;
    }

    pub fn release_up_button(&mut self) {
        self.up_button_down = false;
    }

    pub fn release_down_button(&mut self) {
        self.down_button_down = false;
    }

    pub fn release_left_button(&mut self) {
        self.left_button_down = false;
    }

    pub fn release_right_button(&mut self) {
        self.right_button_down = false;
    }

    pub fn read(&mut self) -> u8 {
        if self.strobe {
            self.a_button_down as u8
        } else {
            let state = self.register & 1;
            self.register >>= 1;
            state
        }
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(&self) -> u8 {
        if self.strobe {
            self.a_button_down as u8
        } else {
            self.register & 1
        }
    }

    pub fn write(&mut self, value: u8) {
        if (value & 1) == 1 {
            self.strobe = true;
        } else if self.strobe {
            self.strobe = false;
            self.register = (self.a_button_down as u8) << 0
                          | (self.b_button_down as u8) << 1
                          | (self.select_button_down as u8) << 2
                          | (self.start_button_down as u8) << 3
                          | (self.up_button_down as u8) << 4
                          | (self.down_button_down as u8) << 5
                          | (self.left_button_down as u8) << 6
                          | (self.right_button_down as u8) << 7;
        }
    }
}
