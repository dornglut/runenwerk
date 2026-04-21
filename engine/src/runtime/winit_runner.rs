use crate::app::WindowedAppState;
use crate::plugins::InputState;
use crate::plugins::render::renderer::Gfx;
use crate::runtime::frame_lifecycle::{
    prepare_world_for_run, run_frame as run_runtime_frame, run_startup_if_needed,
};
use crate::runtime::platform::{PlatformEvent, apply_platform_event};
use crate::runtime::window::WindowState;
use anyhow::{Context, Result, anyhow};
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
        fatal_error: None,
    };
    event_loop
        .run_app(&mut runner)
        .map_err(anyhow::Error::from)?;
    if let Some(err) = runner.fatal_error {
        Err(err)
    } else {
        Ok(())
    }
}

struct WinitRunner {
    state: WindowedAppState,
    window: Option<Arc<Window>>,
    fatal_error: Option<anyhow::Error>,
}

impl WinitRunner {
    fn sync_window_state(&mut self, window: &Window) -> Result<()> {
        let size = window.inner_size();
        let window_state = self
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
        // Windowed flow uses the same startup contract as headless.
        prepare_world_for_run(&mut self.state.world, &self.state.title, false);
        run_startup_if_needed(
            &mut self.state.world,
            &mut self.state.scheduler,
            &mut self.state.startup_ran,
        )
    }

    fn run_frame(&mut self) -> Result<()> {
        // Windowed flow uses the same per-frame schedule order as headless.
        run_runtime_frame(&mut self.state.world, &mut self.state.scheduler)
    }

    fn apply_window_effects(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let Some(window) = self.window.as_ref() else {
            return Ok(());
        };

        let window_state = self
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

    fn exit_with_error(&mut self, event_loop: &ActiveEventLoop, err: anyhow::Error) {
        tracing::error!(error = %format!("{err:#}"), "runtime windowed execution failed");
        self.fatal_error = Some(err);
        event_loop.exit();
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
                self.exit_with_error(
                    event_loop,
                    anyhow!("failed to create runtime window: {err}"),
                );
                return;
            }
        };

        if let Err(err) = self.sync_window_state(&window) {
            self.exit_with_error(
                event_loop,
                anyhow!("failed to sync initial window state: {err:#}"),
            );
            return;
        }

        if self.state.world.resource::<Gfx>().is_err() {
            match Gfx::new(window.clone()) {
                Ok(gfx) => self.state.world.insert_resource(gfx),
                Err(err) => {
                    self.exit_with_error(
                        event_loop,
                        anyhow!("failed to initialize runtime gfx: {err:#}"),
                    );
                    return;
                }
            }
        }

        if let Err(err) = self.apply_event(PlatformEvent::Resumed) {
            self.exit_with_error(event_loop, anyhow!("failed to apply resume event: {err:#}"));
            return;
        }

        if let Err(err) = self.run_startup_if_needed() {
            self.exit_with_error(event_loop, anyhow!("runtime startup failed: {err:#}"));
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
                    self.exit_with_error(event_loop, anyhow!("runtime frame failed: {err:#}"));
                }
                return;
            }
            _ => Ok(()),
        };

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            self.exit_with_error(event_loop, anyhow!("runtime window event failed: {err:#}"));
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
            self.exit_with_error(event_loop, anyhow!("runtime device event failed: {err:#}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::runtime::fixed_time::{
        CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick,
    };
    use crate::runtime::schedules::{FixedUpdate, PreUpdate};
    use crate::runtime::{Res, ResMut};

    #[derive(Debug, Default, ecs::Component, ecs::Resource)]
    struct FixedTickLog(Vec<u64>);

    fn configure_probe(app: &mut App) {
        app.init_resource::<FixedTickLog>();
        app.insert_resource(FixedTimeConfig {
            step_seconds: 1.0 / 60.0,
        });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.add_systems(PreUpdate, set_frame_delta);
        app.add_systems(FixedUpdate, log_tick);
    }

    fn set_frame_delta(mut time: ResMut<crate::plugins::time::domain::Time>) {
        (*time).delta_seconds = 0.05;
    }

    fn log_tick(tick: Res<SimulationTick>, mut log: ResMut<FixedTickLog>) {
        (*log).0.push(tick.0);
    }

    #[test]
    fn headless_and_windowed_paths_share_fixed_step_semantics() {
        let mut headless = App::headless();
        configure_probe(&mut headless);
        headless
            .prepare_for_run(true)
            .expect("headless startup should run");
        headless.run_frame().expect("headless frame should run");

        let headless_log = headless
            .world()
            .resource::<FixedTickLog>()
            .expect("headless log resource should exist")
            .0
            .clone();
        let headless_tick = headless
            .world()
            .resource::<SimulationTick>()
            .expect("headless tick should exist")
            .0;
        let headless_fixed = *headless
            .world()
            .resource::<FixedTimeState>()
            .expect("headless fixed state should exist");

        let mut windowed = App::new();
        configure_probe(&mut windowed);
        let mut runner = WinitRunner {
            state: windowed.into_windowed_state(),
            window: None,
            fatal_error: None,
        };
        runner
            .run_startup_if_needed()
            .expect("windowed startup should run");
        runner.run_frame().expect("windowed frame should run");

        let windowed_log = runner
            .state
            .world
            .resource::<FixedTickLog>()
            .expect("windowed log resource should exist")
            .0
            .clone();
        let windowed_tick = runner
            .state
            .world
            .resource::<SimulationTick>()
            .expect("windowed tick should exist")
            .0;
        let windowed_fixed = *runner
            .state
            .world
            .resource::<FixedTimeState>()
            .expect("windowed fixed state should exist");

        assert_eq!(headless_log, vec![1, 2, 3]);
        assert_eq!(windowed_log, vec![1, 2, 3]);
        assert_eq!(headless_log, windowed_log);
        assert_eq!(headless_tick, windowed_tick);
        assert_eq!(headless_fixed.steps_ran_last_frame, 3);
        assert_eq!(windowed_fixed.steps_ran_last_frame, 3);
        assert_eq!(headless_fixed.saturated_frames, 0);
        assert_eq!(windowed_fixed.saturated_frames, 0);
    }
}
