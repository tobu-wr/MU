use std::time::{Instant, Duration};

pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;
const WINDOW_TITLE: &str = "MU 1.0.0 Alpha";

pub enum Key {
    Q,
    W,
    Space,
    Enter,
    Up,
    Down,
    Left,
    Right
}

pub struct Window {
    frame_counter: u16,
    instant: Instant
}

impl Window {
    pub fn new() -> Self {
        Self {
            frame_counter: 0,
            instant: Instant::now()
        }
    }

    pub fn is_open(&self) -> bool {
        true
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        false
    }

    pub fn update(&mut self, buffer: &[u32]) {
        self.frame_counter += 1;
        
        let elapsed = self.instant.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.instant = Instant::now();
            let fps = self.frame_counter as f64 / elapsed.as_secs_f64();
            self.frame_counter = 0;
        }
    }
}
