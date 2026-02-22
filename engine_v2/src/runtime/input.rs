use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Default, Debug)]
pub struct InputState {
    keys_down: HashSet<KeyCode>,
    pub typed_text: String,
    pub submitted: bool,
    pub backspace: bool,
    pub page_up: bool,
    pub page_down: bool,
    pub mouse_delta: (f32, f32),
    pub mouse_position: (f32, f32),
    pub scroll_delta: f32,
    mouse_buttons_down: HashSet<MouseButton>,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.keys_down.insert(code);
                            if code == KeyCode::Enter {
                                self.submitted = true;
                            }
                            if code == KeyCode::Backspace {
                                self.backspace = true;
                            }
                            if code == KeyCode::PageUp {
                                self.page_up = true;
                            }
                            if code == KeyCode::PageDown {
                                self.page_down = true;
                            }
                        }
                        ElementState::Released => {
                            self.keys_down.remove(&code);
                        }
                    }
                }

                if let Some(text) = &event.text {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            self.typed_text.push(ch);
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => self.scroll_delta += *y,
                MouseScrollDelta::PixelDelta(p) => self.scroll_delta += p.y as f32,
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.mouse_buttons_down.insert(*button);
                    if *button == MouseButton::Left {
                        self.left_mouse_pressed = true;
                    }
                }
                ElementState::Released => {
                    self.mouse_buttons_down.remove(button);
                    if *button == MouseButton::Left {
                        self.left_mouse_released = true;
                    }
                }
            },
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.mouse_delta.0 += delta.0 as f32;
            self.mouse_delta.1 += delta.1 as f32;
        }
    }

    pub fn clear_frame(&mut self) {
        self.typed_text.clear();
        self.submitted = false;
        self.backspace = false;
        self.page_up = false;
        self.page_down = false;
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
        self.left_mouse_pressed = false;
        self.left_mouse_released = false;
    }

    pub fn left_mouse_down(&self) -> bool {
        self.mouse_buttons_down.contains(&MouseButton::Left)
    }

    pub fn left_mouse_pressed(&self) -> bool {
        self.left_mouse_pressed
    }

    pub fn left_mouse_released(&self) -> bool {
        self.left_mouse_released
    }
}
