pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;

const PIXEL_SIZE: usize = 4;
const FRAME_BUFFER_SIZE: usize = FRAME_WIDTH * FRAME_HEIGHT * PIXEL_SIZE;

pub struct Screen {
    frame_buffer: Vec<u8>,
    draw_requested: bool
}

impl Screen {
    pub fn new() -> Self {
        Self {
            frame_buffer: vec![0; FRAME_BUFFER_SIZE],
            draw_requested: false
        }
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }

    pub fn set_pixel(&mut self, row: usize, column: usize, color: usize) {
        const RGBA: [u32; 0x40] = [0x545454ff, 0x001e74ff, 0x081090ff, 0x300088ff, 0x440064ff, 0x5c0030ff, 0x540400ff, 0x3c1800ff,
                                   0x202a00ff, 0x083a00ff, 0x004000ff, 0x003c00ff, 0x00323cff, 0x000000ff, 0x000000ff, 0x000000ff,
                                   0x989698ff, 0x084cc4ff, 0x3032ecff, 0x5c1ee4ff, 0x8814b0ff, 0xa01464ff, 0x982220ff, 0x783c00ff,
                                   0x545a00ff, 0x287200ff, 0x087c00ff, 0x007628ff, 0x006678ff, 0x000000ff, 0x000000ff, 0x000000ff,
                                   0xeceeecff, 0x4c9aecff, 0x787cecff, 0xb062ecff, 0xe454ecff, 0xec58b4ff, 0xec6a64ff, 0xd48820ff,
                                   0xa0aa00ff, 0x74c400ff, 0x4cd020ff, 0x38cc6cff, 0x38b4ccff, 0x3c3c3cff, 0x000000ff, 0x000000ff,
                                   0xeceeecff, 0xa8ccecff, 0xbcbcecff, 0xd4b2ecff, 0xecaeecff, 0xecaed4ff, 0xecb4b0ff, 0xe4c490ff,
                                   0xccd278ff, 0xb4de78ff, 0xa8e290ff, 0x98e2b4ff, 0xa0d6e4ff, 0xa0a2a0ff, 0x000000ff, 0x000000ff];
        
        let offset = (row * FRAME_WIDTH + column) * PIXEL_SIZE;
        self.frame_buffer[offset..offset + PIXEL_SIZE].copy_from_slice(&RGBA[color].to_be_bytes());
    }

    pub fn is_draw_requested(&self) -> bool {
        self.draw_requested
    }

    pub fn request_draw(&mut self) {
        self.draw_requested = true;
    }

    pub fn finish_draw(&mut self) {
        self.draw_requested = false;
    }
}
