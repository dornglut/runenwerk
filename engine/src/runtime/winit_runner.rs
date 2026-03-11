use crate::app::WindowedAppState;
use crate::plugins::InputState;
use crate::plugins::render::domain::Gfx;
use crate::plugins::time::domain::Time;
use crate::runtime::fixed_time::{CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick};
use crate::runtime::platform::{PlatformEvent, apply_platform_event};
use crate::runtime::schedules::{
    FixedUpdate, FrameEnd, PreUpdate, RenderPrepare, RenderSubmit, Startup, Update,
};
use crate::runtime::window::WindowState;
use anyhow::{Context, Result};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub(crate) fn run(state: WindowedAppState) -> Result<()> {
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
            .run_schedule::<PreUpdate>(&mut self.state.world)?;
        self.run_fixed_update_schedule()?;
        self.state
            .scheduler
            .run_schedule::<Update>(&mut self.state.world)?;
        self.state
            .scheduler
            .run_schedule::<RenderPrepare>(&mut self.state.world)?;
        self.state
            .scheduler
            .run_schedule::<RenderSubmit>(&mut self.state.world)?;
        self.state
            .scheduler
            .run_schedule::<FrameEnd>(&mut self.state.world)?;
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

    fn run_fixed_update_schedule(&mut self) -> Result<()> {
        let step_seconds = self
            .state
            .world
            .resource::<FixedTimeConfig>()
            .map(|config| config.step_seconds)
            .unwrap_or(1.0 / 60.0)
            .clamp(1.0 / 240.0, 1.0 / 15.0);
        let delta_seconds = self
            .state
            .world
            .resource::<Time>()
            .map(|time| time.delta_seconds)
            .unwrap_or(step_seconds)
            .clamp(0.0, 0.25);
        let max_steps_per_frame = self
            .state
            .world
            .resource::<CatchupBudget>()
            .map(|budget| budget.max_steps_per_frame)
            .unwrap_or(4)
            .clamp(1, 16);

        {
            let mut fixed_state = self
                .state
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds = (fixed_state.accumulator_seconds + delta_seconds)
                .min(step_seconds * max_steps_per_frame as f32);
            fixed_state.steps_ran_last_frame = 0;
        }

        let mut steps = 0u32;
        loop {
            let should_step = {
                let fixed_state = self
                    .state
                    .world
                    .resource::<FixedTimeState>()
                    .expect("FixedTimeState should be installed");
                fixed_state.accumulator_seconds + f32::EPSILON >= step_seconds
                    && steps < max_steps_per_frame
            };
            if !should_step {
                break;
            }

            self.state
                .scheduler
                .run_schedule::<FixedUpdate>(&mut self.state.world)?;
            steps = steps.saturating_add(1);

            let mut fixed_state = self
                .state
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds -= step_seconds;
            fixed_state.steps_ran_last_frame = steps;
        }

        let saturated = {
            let fixed_state = self
                .state
                .world
                .resource::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds + f32::EPSILON >= step_seconds
        };
        if saturated {
            let mut fixed_state = self
                .state
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds = 0.0;
            fixed_state.saturated_frames = fixed_state.saturated_frames.saturating_add(1);
            tracing::warn!("fixed-step loop saturated, dropping accumulated time");
        }

        if steps > 0 {
            let mut tick = self
                .state
                .world
                .resource_mut::<SimulationTick>()
                .expect("SimulationTick should be installed");
            tick.0 = tick.0.saturating_add(steps as u64);
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
                tracing::error!(error = %err, "failed to create runtime window");
                event_loop.exit();
                return;
            }
        };

        if let Err(err) = self.sync_window_state(&window) {
            tracing::error!(error = %err, "failed to sync initial window state");
            event_loop.exit();
            return;
        }

        if self.state.world.resource::<Gfx>().is_err() {
            match Gfx::new(window.clone()) {
                Ok(gfx) => self.state.world.insert_resource(gfx),
                Err(err) => {
                    tracing::error!(error = %err, "failed to initialize runtime gfx");
                    event_loop.exit();
                    return;
                }
            }
        }

        if let Err(err) = self.apply_event(PlatformEvent::Resumed) {
            tracing::error!(error = %err, "failed to apply resume event");
            event_loop.exit();
            return;
        }

        if let Err(err) = self.run_startup_if_needed() {
            tracing::error!(error = %err, "runtime startup failed");
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
                    tracing::error!(error = %format!("{err:#}"), "runtime frame failed");
                    event_loop.exit();
                }
                return;
            }
            _ => Ok(()),
        };

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            tracing::error!(error = %format!("{err:#}"), "runtime window event failed");
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
            tracing::error!(error = %format!("{err:#}"), "runtime device event failed");
            event_loop.exit();
        }
    }
}
