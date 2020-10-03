use minifb::WindowOptions;
use super::*;

const WIDTH: usize = FRAME_WIDTH * 2;
const HEIGHT: usize = FRAME_HEIGHT * 2;

pub(super) struct NametableViewer {
	window: Window,
	buffer: Vec<u32>
}

impl NametableViewer {
    pub(super) fn new() -> Self {
        let options = WindowOptions{ resize: true, ..WindowOptions::default() };
		Self {
			window: Window::new("Nametable Viewer", WIDTH, HEIGHT, options).unwrap(),
			buffer: vec![0; WIDTH * HEIGHT]
		}
	}

	pub(super) fn update(ppu: &mut Ppu) {
		let nametable_address: u16 = 0x2000;
		let attribute_table_address = nametable_address + 0x3c0;
		let pattern_address = 0x1000 * ((ppu.ppuctrl >> 4) & 1) as u16;
		for y in 0..HEIGHT as u16 {
			let tile_row = y / 8;
			let pixel_row = y % 8;
			let attribute_row = tile_row / 4;
			for x in 0..WIDTH as u16 {
				let tile_column = x / 8;
				let pixel_column = x % 8;
				let attribute_column = tile_column / 4;
				let attribute = ppu.memory.read(attribute_table_address + attribute_row * 8 + attribute_column);
				let palette_number = ((attribute >> (4 * ((tile_row / 2) % 2))) >> (2 * ((tile_column / 2) % 2))) & 0b11;
				let tile_number_address = nametable_address + tile_row * 32 + tile_column;
				let tile_number = ppu.memory.read(tile_number_address);
				let low_byte = ppu.memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row);
				let high_byte = ppu.memory.read(pattern_address + (tile_number as u16) * 16 + pixel_row + 8);
				let low_bit = (low_byte >> (7 - pixel_column)) & 1;
				let high_bit = (high_byte >> (7 - pixel_column)) & 1;
				let color_number = (high_bit << 1) | low_bit;
				let color_address = if color_number == 0 {
					0 // backdrop color
				} else {
					4 * palette_number as u16 + color_number as u16
				} + 0x3f00;
				let color = ppu.memory.read(color_address);			

				const COLORS: [u32; 0x40] = [0x00545454, 0x00001e74, 0x00081090, 0x00300088, 0x00440064, 0x005c0030, 0x00540400, 0x003c1800,
									 		 0x00202a00, 0x00083a00, 0x00004000, 0x00003c00, 0x0000323c, 0x00000000, 0x00000000, 0x00000000,
											 0x00989698, 0x00084cc4, 0x003032ec, 0x005c1ee4, 0x008814b0, 0x00a01464, 0x00982220, 0x00783c00,
											 0x00545a00, 0x00287200, 0x00087c00, 0x00007628, 0x00006678, 0x00000000, 0x00000000, 0x00000000,
											 0x00eceeec, 0x004c9aec, 0x00787cec, 0x00b062ec, 0x00e454ec, 0x00ec58b4, 0x00ec6a64, 0x00d48820,
											 0x00a0aa00, 0x0074c400, 0x004cd020, 0x0038cc6c, 0x0038b4cc, 0x003c3c3c, 0x00000000, 0x00000000,
											 0x00eceeec, 0x00a8ccec, 0x00bcbcec, 0x00d4b2ec, 0x00ecaeec, 0x00ecaed4, 0x00ecb4b0, 0x00e4c490,
											 0x00ccd278, 0x00b4de78, 0x00a8e290, 0x0098e2b4, 0x00a0d6e4, 0x00a0a2a0, 0x00000000, 0x00000000];
				ppu.nametable_viewer.buffer[(y as usize) * WIDTH + (x as usize)] = COLORS[color as usize];
			}
		}
		ppu.nametable_viewer.window.update_with_buffer(&ppu.nametable_viewer.buffer, WIDTH, HEIGHT).unwrap();
	}
}
