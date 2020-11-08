use std::time::{Instant, Duration};
use minifb::WindowOptions;

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
    window: minifb::Window,    
    frame_counter: u16,
    instant: Instant
}

impl Window {
    pub fn new() -> Self {
        let options = WindowOptions{ resize: true, ..WindowOptions::default() };
        let mut window = minifb::Window::new(WINDOW_TITLE, FRAME_WIDTH, FRAME_HEIGHT, options).unwrap();
    
        #[cfg(not(feature = "fullspeed"))]
        window.limit_update_rate(Some(std::time::Duration::from_secs_f64(1.0 / 60.0)));
    
        Self {
            window,
            frame_counter: 0,
            instant: Instant::now()
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        match key {
            Key::Q => self.window.is_key_down(minifb::Key::Q),
            Key::W => self.window.is_key_down(minifb::Key::W),
            Key::Space => self.window.is_key_down(minifb::Key::Space),
            Key::Enter => self.window.is_key_down(minifb::Key::Enter),
            Key::Up => self.window.is_key_down(minifb::Key::Up),
            Key::Down => self.window.is_key_down(minifb::Key::Down),
            Key::Left => self.window.is_key_down(minifb::Key::Left),
            Key::Right => self.window.is_key_down(minifb::Key::Right)
        }
    }

    pub fn update(&mut self, buffer: &[u32]) {
        self.window.update_with_buffer(buffer, FRAME_WIDTH, FRAME_HEIGHT).unwrap();
        self.frame_counter += 1;
        
        let elapsed = self.instant.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.instant = Instant::now();
            let fps = self.frame_counter as f64 / elapsed.as_secs_f64();
            self.frame_counter = 0;
            let title = format!("{} - FPS: {}", WINDOW_TITLE, fps.round() as u16);
            self.window.set_title(&title);
        }
    }
}
