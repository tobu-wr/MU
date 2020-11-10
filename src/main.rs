extern crate winit;

#[macro_use]
extern crate log;

mod emulator;
mod cpu;
mod ppu;
mod joypad;
mod mappers;
mod window;

use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder
};

use emulator::*;

fn main() {
	env_logger::Builder::new().filter_level(log::LevelFilter::max()).init();

	let filename = std::env::args().nth(1).unwrap();
	let mut emulator = Emulator::new();
	emulator.load_file(&filename);
	
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new().build(&event_loop).unwrap();

	event_loop.run(move |event, _, control_flow| {
		match event {
			Event::WindowEvent {
				ref event,
				..
			} => match event {
				WindowEvent::CloseRequested => {
					*control_flow = ControlFlow::Exit;
					return;
				},
				WindowEvent::KeyboardInput {
					ref input,
					..
				} => match input {
					KeyboardInput {
						state: ElementState::Pressed,
						ref virtual_keycode,
						..
					} => match virtual_keycode {
						Some(VirtualKeyCode::Q) => emulator.joypad.press_a_button(),
						Some(VirtualKeyCode::W) => emulator.joypad.press_b_button(),
						Some(VirtualKeyCode::Space) => emulator.joypad.press_select_button(),
						Some(VirtualKeyCode::Return) => emulator.joypad.press_start_button(),
						Some(VirtualKeyCode::Up) => emulator.joypad.press_up_button(),
						Some(VirtualKeyCode::Down) => emulator.joypad.press_down_button(),
						Some(VirtualKeyCode::Left) => emulator.joypad.press_left_button(),
						Some(VirtualKeyCode::Right) => emulator.joypad.press_right_button(),
						_ => {}
					},
					KeyboardInput {
						state: ElementState::Released,
						ref virtual_keycode,
						..
					} => match virtual_keycode {
						Some(VirtualKeyCode::Q) => emulator.joypad.release_a_button(),
						Some(VirtualKeyCode::W) => emulator.joypad.release_b_button(),
						Some(VirtualKeyCode::Space) => emulator.joypad.release_select_button(),
						Some(VirtualKeyCode::Return) => emulator.joypad.release_start_button(),
						Some(VirtualKeyCode::Up) => emulator.joypad.release_up_button(),
						Some(VirtualKeyCode::Down) => emulator.joypad.release_down_button(),
						Some(VirtualKeyCode::Left) => emulator.joypad.release_left_button(),
						Some(VirtualKeyCode::Right) => emulator.joypad.release_right_button(),
						_ => {}
					},
					_ => {}
				},
				_ => {}
			},
			Event::RedrawRequested(_) => {
				// TODO
			},
			_ => {}
		}

		*control_flow = ControlFlow::Poll;

		emulator.step();
    });
}
