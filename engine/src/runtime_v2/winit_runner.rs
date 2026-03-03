use crate::app::WindowedAppState;
use crate::plugins::input::domain::InputState;
use crate::runtime_v2::platform::{PlatformEvent, apply_platform_event};
use crate::runtime_v2::schedules::{RenderPrepare, RenderSubmit, Startup, Update};
use crate::runtime_v2::window::WindowState;
use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub fn run(state: WindowedAppState) -> Result<()> {
    ensure_build_ready(&state.build_errors)?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(state.control_flow);
    let mut runner = WinitRunner {
        state,
        window: None,
    };
    event_loop.run_app(&mut runner).map_err(Into::into)
}

struct WinitRunner {
    state: WindowedAppState,
    window: Option<Arc<Window>>,
}

impl WinitRunner {
    fn sync_window_state(&mut self, window: &Window) -> Result<()> {
        let size = window.inner_size();
        let mut window_state = self
            .state
            .world
            .resource_mut::<WindowState>()
            .context("missing WindowState resource")?;
        window_state.set_headless(false);
        window_state.size_px = (size.width, size.height);
        window_state.scale_factor = window.scale_factor();
        window_state.title = window.title().to_string();
        Ok(())
    }

    fn apply_event(&mut self, event: PlatformEvent) -> Result<()> {
        match &event {
            PlatformEvent::Resumed
            | PlatformEvent::CloseRequested
            | PlatformEvent::Resized { .. }
            | PlatformEvent::ScaleFactorChanged { .. }
            | PlatformEvent::RedrawRequested => {
                let mut window_state = self
                    .state
                    .world
                    .resource_mut::<WindowState>()
                    .context("missing WindowState resource")?;
                let mut input = InputState::new();
                apply_platform_event(&mut window_state, &mut input, &event);
            }
            PlatformEvent::KeyboardInput { .. }
            | PlatformEvent::MouseWheel { .. }
            | PlatformEvent::CursorMoved { .. }
            | PlatformEvent::MouseInput { .. }
            | PlatformEvent::MouseMotion { .. } => {
                let mut window_state = WindowState::headless("");
                let mut input = self
                    .state
                    .world
                    .resource_mut::<InputState>()
                    .context("missing InputState resource")?;
                apply_platform_event(&mut window_state, &mut input, &event);
            }
        }
        Ok(())
    }

    fn run_startup_if_needed(&mut self) -> Result<()> {
        if self.state.startup_ran {
            return Ok(());
        }
        self.state
            .scheduler
            .run_schedule::<Startup>(&mut self.state.world)?;
        self.state.startup_ran = true;
        Ok(())
    }

    fn run_frame(&mut self) -> Result<()> {
        self.state
            .scheduler
            .run_schedule::<Update>(&mut self.state.world)?;
        self.state
            .scheduler
            .run_schedule::<RenderPrepare>(&mut self.state.world)?;
        self.state
            .scheduler
            .run_schedule::<RenderSubmit>(&mut self.state.world)?;
        Ok(())
    }

    fn apply_window_effects(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let Some(window) = self.window.as_ref() else {
            return Ok(());
        };

        let mut window_state = self
            .state
            .world
            .resource_mut::<WindowState>()
            .context("missing WindowState resource")?;

        if window.title() != window_state.title {
            window.set_title(&window_state.title);
        }

        if window_state.close_requested {
            event_loop.exit();
            return Ok(());
        }

        if window_state.redraw_requested {
            window.request_redraw();
            window_state.redraw_requested = false;
        } else {
            window.request_redraw();
        }

        Ok(())
    }
}

impl ApplicationHandler for WinitRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs: WindowAttributes =
            Window::default_attributes().with_title(self.state.title.clone());
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                tracing::error!(error = %err, "failed to create typed runtime window");
                event_loop.exit();
                return;
            }
        };

        if let Err(err) = self.sync_window_state(&window) {
            tracing::error!(error = %err, "failed to sync initial window state");
            event_loop.exit();
            return;
        }

        if let Err(err) = self.apply_event(PlatformEvent::Resumed) {
            tracing::error!(error = %err, "failed to apply resume event");
            event_loop.exit();
            return;
        }

        if let Err(err) = self.run_startup_if_needed() {
            tracing::error!(error = %err, "typed runtime startup failed");
            event_loop.exit();
            return;
        }

        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let result = match event {
            WindowEvent::CloseRequested => self.apply_event(PlatformEvent::CloseRequested),
            WindowEvent::Resized(size) => self.apply_event(PlatformEvent::Resized {
                width: size.width,
                height: size.height,
            }),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let Some(window) = self.window.as_ref() else {
                    return;
                };
                let size = window.inner_size();
                self.apply_event(PlatformEvent::ScaleFactorChanged {
                    scale_factor,
                    width: size.width,
                    height: size.height,
                })
            }
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                winit::keyboard::PhysicalKey::Code(code) => {
                    self.apply_event(PlatformEvent::KeyboardInput {
                        key: code,
                        state: event.state,
                        text: event.text.as_deref().map(str::to_string),
                    })
                }
                _ => Ok(()),
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32,
                };
                self.apply_event(PlatformEvent::MouseWheel { delta })
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.apply_event(PlatformEvent::CursorMoved {
                    x: position.x as f32,
                    y: position.y as f32,
                })
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.apply_event(PlatformEvent::MouseInput { state, button })
            }
            WindowEvent::RedrawRequested => {
                let frame_result = self
                    .apply_event(PlatformEvent::RedrawRequested)
                    .and_then(|_| self.run_frame())
                    .and_then(|_| self.apply_window_effects(event_loop));
                if let Err(err) = frame_result {
                    tracing::error!(error = %err, "typed runtime frame failed");
                    event_loop.exit();
                }
                return;
            }
            _ => Ok(()),
        };

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            tracing::error!(error = %err, "typed runtime window event failed");
            event_loop.exit();
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let result = match event {
            DeviceEvent::MouseMotion { delta } => self.apply_event(PlatformEvent::MouseMotion {
                delta_x: delta.0 as f32,
                delta_y: delta.1 as f32,
            }),
            _ => Ok(()),
        };

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            tracing::error!(error = %err, "typed runtime device event failed");
            event_loop.exit();
        }
    }
}

fn ensure_build_ready(build_errors: &[anyhow::Error]) -> Result<()> {
    if build_errors.is_empty() {
        return Ok(());
    }
    let messages: Vec<_> = build_errors.iter().map(ToString::to_string).collect();
    Err(anyhow!("app setup failed:\n{}", messages.join("\n")))
}
