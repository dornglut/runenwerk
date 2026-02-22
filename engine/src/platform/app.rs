// src/app
use crate::engine::Engine;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{PhysicalKey};
use winit::keyboard::KeyCode::Escape;
use winit::window::{CursorGrabMode, Window};

pub struct App {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
}

impl<'window> App {
    pub fn new() -> Self {
        Self {
            window: None,
            engine: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attributes = Window::default_attributes().with_title("SDF Renderer");

            let window = Arc::new(
                event_loop
                    .create_window(win_attributes)
                    .expect("failed to create window"),
            );
            self.window = Some(window.clone());

            window.set_cursor_visible(false);

            // Grab the cursor (confine it to the window)
            window.set_cursor_grab(CursorGrabMode::Confined)
              .unwrap_or_else(|_| tracing::warn!("Failed to grab cursor"));

            // Init Engine
            let engine = Engine::new(window.clone()).expect("Failed to create Engine");
            self.engine = Some(engine);

            window.request_redraw();
            tracing::info!("Window created and engine initialized");
        }

        tracing::info!("Created window");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let engine = match self.engine.as_mut() {
            Some(engine) => engine,
            None => return,
        };

        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };

        engine.data.input.handle_window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window closed");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                tracing::info!("Window resized to {:?}", size);
                engine.resize(size.width, size.height);
                window.request_redraw();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, mut inner_size_writer} => {
                let window = self.window.as_ref().unwrap();

                let logical_size: LogicalSize<f64> = window.inner_size().to_logical(scale_factor);
                let physical_size = PhysicalSize::new(
                    (logical_size.width * scale_factor).round() as u32,
                    (logical_size.height * scale_factor).round() as u32,
                );

                // Apply synchronous resize
                inner_size_writer
                    .request_inner_size(physical_size)
                    .expect("Failed to set inner size");

                engine.resize(physical_size.width, physical_size.height);

                window.request_redraw();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == PhysicalKey::Code(Escape) {
                    if event.state == ElementState::Pressed {
                        tracing::info!("Escape Pressed, closing window");
                        event_loop.exit();
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                engine.update();
                window.request_redraw();
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if let Some(engine) = self.engine.as_mut() {
            engine.data.input.handle_device_event(&event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}
