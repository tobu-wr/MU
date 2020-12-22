extern crate winit;
extern crate wgpu;
extern crate futures;

#[macro_use]
extern crate log;

mod emulator;
mod renderer;
mod cpu;
mod ppu;
mod joypad;
mod mappers;
mod screen;

use std::time::{Instant, Duration};

use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder
};

use emulator::*;
use renderer::*;
use screen::*;

fn main() {
	env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();

	let filename = std::env::args().nth(1).unwrap();
	let mut emulator = Emulator::new();
	emulator.load_file(&filename);
	
	let event_loop = EventLoop::new();

	const EMULATOR_NAME: &str = "MU";
	let window = WindowBuilder::new().with_title(EMULATOR_NAME).build(&event_loop).unwrap();
	
	let mut renderer = Renderer::new(&window, FRAME_WIDTH as _, FRAME_HEIGHT as _);

	let mut frame_counter = 0u16;
	let mut frame_counting_instant = Instant::now();
	let mut last_frame_instant = Instant::now();

	event_loop.run(move |event, _, control_flow| {
		match event {
			Event::WindowEvent {
				ref event,
				..
			} => match event {
				WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
				WindowEvent::KeyboardInput {
					ref input,
					..
				} => match input {
					KeyboardInput {
						state: ElementState::Pressed,
						ref virtual_keycode,
						..
					} => match virtual_keycode {
						Some(VirtualKeyCode::A) => emulator.joypad.press_a_button(),
						Some(VirtualKeyCode::Z) => emulator.joypad.press_b_button(),
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
						Some(VirtualKeyCode::A) => emulator.joypad.release_a_button(),
						Some(VirtualKeyCode::Z) => emulator.joypad.release_b_button(),
						Some(VirtualKeyCode::Space) => emulator.joypad.release_select_button(),
						Some(VirtualKeyCode::Return) => emulator.joypad.release_start_button(),
						Some(VirtualKeyCode::Up) => emulator.joypad.release_up_button(),
						Some(VirtualKeyCode::Down) => emulator.joypad.release_down_button(),
						Some(VirtualKeyCode::Left) => emulator.joypad.release_left_button(),
						Some(VirtualKeyCode::Right) => emulator.joypad.release_right_button(),
						_ => {}
					}
				},
				WindowEvent::Resized(size) => renderer.resize(*size),
				WindowEvent::ScaleFactorChanged {
					ref new_inner_size,
					..
				} => renderer.resize(**new_inner_size),
				_ => {}
			},
			Event::MainEventsCleared => {
				// draw frame
				while !emulator.screen.is_draw_requested() {
					emulator.step();
				}
				renderer.draw(emulator.screen.get_frame_buffer());
				emulator.screen.finish_draw();

				// regulate frame rate
				const FRAME_DURATION: Duration = Duration::from_millis(17);
				let elapsed = last_frame_instant.elapsed();
				if elapsed < FRAME_DURATION {
					std::thread::sleep(FRAME_DURATION - elapsed);
				}
				last_frame_instant = Instant::now();

				// compute and display frame rate
				frame_counter += 1;
				let elapsed = frame_counting_instant.elapsed();
				if elapsed >= Duration::from_secs(1) {
					frame_counting_instant = Instant::now();
					let fps = (frame_counter as f64 / elapsed.as_secs_f64()).round();
					frame_counter = 0;
					let speed = (fps / 0.6).round();
					let title = format!("{} - FPS: {} - SPEED: {}%", EMULATOR_NAME, fps, speed);
					window.set_title(&title);
				}
			},
			_ => {}
		}
    });
}
