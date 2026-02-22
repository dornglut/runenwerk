use crate::runtime::Engine;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct App {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            engine: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs: WindowAttributes =
            Window::default_attributes().with_title("Grotto Quest - Engine v2");
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );

        let mut engine = Engine::new(window.clone()).expect("failed to create engine");
        // Keep UI sizing aligned with the display scale from the first frame.
        engine.set_ui_scale_from_window_factor(window.scale_factor());
        window.request_redraw();

        self.window = Some(window);
        self.engine = Some(engine);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };
        let Some(window) = self.window.as_ref() else {
            return;
        };

        engine.data.input.handle_window_event(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                engine.resize(size.width, size.height);
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                let logical: LogicalSize<f64> = window.inner_size().to_logical(scale_factor);
                let physical = PhysicalSize::new(
                    (logical.width * scale_factor).round() as u32,
                    (logical.height * scale_factor).round() as u32,
                );

                let _ = inner_size_writer.request_inner_size(physical);
                engine.set_ui_scale_from_window_factor(scale_factor);
                engine.resize(physical.width, physical.height);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                {
                    if engine.data.scene.overlay_runtime.ui.editor.enabled {
                        engine.data.scene.overlay_runtime.ui.editor.enabled = false;
                        engine.data.scene.overlay_runtime.ui.editor.dragging = false;
                        engine.data.scene.overlay_runtime.ui.editor.status =
                            "editor: off (F1 to toggle)".to_string();
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

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(engine) = self.engine.as_mut() {
            engine.data.input.handle_device_event(&event);
        }
    }
}
