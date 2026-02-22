use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Default, Debug)]
pub struct InputState {
	pub(crate) keys_down: HashSet<KeyCode>,
	mouse_buttons_down: HashSet<MouseButton>,

	pub mouse_delta: (f32, f32),
	pub scroll_delta: f32,
}

impl InputState {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn handle_event<T>(&mut self, event: &Event<T>) {
		match event {
			Event::WindowEvent { event, .. } => {
				self.handle_window_event(event);
			}
			Event::DeviceEvent { event, .. } => {
				self.handle_device_event(event);
			}
			_ => {}
		}
	}

	pub fn handle_window_event(&mut self, event: &WindowEvent) {
		match event {
			WindowEvent::KeyboardInput { event, .. } => {
				if let PhysicalKey::Code(code) = event.physical_key {
					match event.state {
						ElementState::Pressed => {
							self.keys_down.insert(code);
						}
						ElementState::Released => {
							self.keys_down.remove(&code);
						}
					}
				}
			}
			WindowEvent::MouseWheel { delta, .. } => {
				match delta {
					MouseScrollDelta::LineDelta(_, y) => {
						self.scroll_delta += *y;
					}
					MouseScrollDelta::PixelDelta(pos) => {
						self.scroll_delta += pos.y as f32;
					}
				}
			}
			WindowEvent::MouseInput { state, button, .. } => {
				match state {
					ElementState::Pressed => {
						self.mouse_buttons_down.insert(*button);
					}
					ElementState::Released => {
						self.mouse_buttons_down.remove(button);
					}
				}
			},
			_ => {}
		}
	}

	pub fn handle_device_event(&mut self, event: &DeviceEvent) {
		match event {
			DeviceEvent::MouseMotion { delta } => {
				self.mouse_delta.0 += delta.0 as f32;
				self.mouse_delta.1 += delta.1 as f32;
			},
			_ => { }
		}
	}

	pub fn key(&self, key: KeyCode) -> bool {
		self.keys_down.contains(&key)
	}

	pub fn mouse_button(&self, button: MouseButton) -> bool {
		self.mouse_buttons_down.contains(&button)
	}

	pub fn clear_frame(&mut self) {
		self.mouse_delta = (0.0, 0.0);
		self.scroll_delta = 0.0;
	}
}