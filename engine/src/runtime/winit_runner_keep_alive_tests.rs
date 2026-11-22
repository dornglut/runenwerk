//! Real Native Host lifetime after original primary closes with a surviving
//! secondary. Render R0 separately proves G7 and Vulkan Present. This proof
//! exercises the actual Winit close transaction and App continuation without
//! silently fabricating primary GPU or RunenUI capabilities.
#![cfg(target_os = "linux")]

use super::*;
use crate::app::{App, AppNativeHostExt, NativePrimaryClosePolicy};
use crate::plugins::default_plugins;
use crate::runtime::system::SystemMobilityExt;
use crate::runtime::{Startup, Update, WorldMut};
use anyhow::{Result, anyhow, ensure};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};
use winit::platform::x11::EventLoopBuilderExtX11;

#[derive(runen_ecs::Resource)]
struct PrimaryKeepAliveCounters {
    startup: Arc<AtomicUsize>,
    updates: Arc<AtomicUsize>,
}

fn request_surviving_secondary(mut world: WorldMut) -> Result<()> {
    world
        .resource_mut::<WindowStateRegistryResource>()?
        .request_window("surviving secondary", (480, 320));
    world
        .resource::<PrimaryKeepAliveCounters>()?
        .startup
        .fetch_add(1, Ordering::SeqCst);
    Ok(())
}

fn count_primary_keep_alive_update(counters: runen_ecs::Res<PrimaryKeepAliveCounters>) {
    counters.updates.fetch_add(1, Ordering::SeqCst);
}

struct NativeKeepAliveProof {
    runner: WinitRunner,
    stage: u8,
    secondary: Option<NativeWindowId>,
    frames_at_close: u64,
    observed: bool,
    failure: Option<anyhow::Error>,
    deadline: Instant,
}

impl NativeKeepAliveProof {
    fn inspect(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        ensure!(
            Instant::now() < self.deadline,
            "primary close proof exceeded its deadline"
        );
        match self.stage {
            0 => {
                let realized = self.runner.window_realizer.realized_windows();
                if realized.len() < 2 {
                    return Ok(());
                }
                let primary = NativeWindowId::primary();
                let secondary = realized
                    .iter()
                    .map(|(_, native, _)| *native)
                    .find(|id| *id != primary)
                    .ok_or_else(|| anyhow!("Winit never realized the requested secondary"))?;
                ensure!(
                    self.runner.window_realizer.has_primary_window(),
                    "original primary must still be real before its close"
                );

                // A close intent is not retirement. The original source,
                // physical primary and the secondary must survive a pending
                // confirmation and a product veto.
                let original = self
                    .runner
                    .native_input_mut()
                    .window_context(primary, winit::event::DeviceId::dummy())?;
                self.runner
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(primary)
                    .ok_or_else(|| anyhow!("original primary record missing"))?
                    .receive_close_intent();
                self.runner.apply_window_effects(event_loop)?;
                ensure!(
                    self.runner.window_realizer.has_primary_window()
                        && self
                            .runner
                            .native_input()
                            .window_source_context(primary)
                            .is_some_and(|context| context.source == original.source),
                    "pending close must not retire the original physical/input source"
                );
                self.runner
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(primary)
                    .ok_or_else(|| anyhow!("primary disappeared during close veto"))?
                    .veto_close();
                self.runner.apply_window_effects(event_loop)?;
                ensure!(
                    self.runner.window_realizer.has_primary_window(),
                    "vetoed close must not retire original Winit window"
                );

                // This is the product's explicit close authorization, not an
                // implicit Native close policy or an unapproved OS shortcut.
                self.runner
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(primary)
                    .ok_or_else(|| anyhow!("primary missing before approved close"))?
                    .request_close();
                self.runner.apply_window_effects(event_loop)?;
                ensure!(
                    !event_loop.exiting(),
                    "approved primary close must not terminate opted-in Native App"
                );
                ensure!(
                    !self.runner.window_realizer.has_primary_window()
                        && self.runner.window_realizer.has_any_window(),
                    "original primary must retire while secondary stays physical"
                );
                ensure!(
                    self.runner
                        .native_input()
                        .window_source_context(primary)
                        .is_none(),
                    "physically destroyed primary must not retain an original source"
                );
                let registry = self
                    .runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()?;
                ensure!(
                    registry.primary_window_id().is_none()
                        && registry.record(primary).is_none()
                        && registry.record(secondary).is_some(),
                    "Native registry must retain exactly the surviving secondary"
                );

                // The real OS resume callback may never recreate fixed ID 1.
                self.runner.resumed(event_loop);
                ensure!(
                    !self.runner.window_realizer.has_primary_window() && !event_loop.exiting(),
                    "resume must preserve the retired original identity and App"
                );
                self.secondary = Some(secondary);
                self.frames_at_close = self.runner.ordinary_frames;
                self.stage = 1;
                self.runner.request_redraw_for_native_window(secondary);
                self.runner.apply_window_effects(event_loop)?;
                self.runner.pace_or_exit(event_loop);
            }
            1 => {
                if self.runner.ordinary_frames <= self.frames_at_close {
                    return Ok(());
                }
                let secondary = self.secondary.ok_or_else(|| anyhow!("missing secondary"))?;
                ensure!(
                    !event_loop.exiting()
                        && !self.runner.window_realizer.has_primary_window()
                        && self
                            .runner
                            .window_realizer
                            .realized_windows()
                            .iter()
                            .any(|(_, id, _)| *id == secondary),
                    "surviving secondary must execute a real ordinary App frame"
                );
                self.observed = true;
                self.stage = 2;
                event_loop.exit();
            }
            _ => {}
        }
        Ok(())
    }
}

impl ApplicationHandler<NativeHostEvent> for NativeKeepAliveProof {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.resumed(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: NativeHostEvent) {
        self.runner.user_event(event_loop, event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.runner.window_event(event_loop, id, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        self.runner.device_event(event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.about_to_wait(event_loop);
        if !event_loop.exiting()
            && let Err(error) = self.inspect(event_loop)
        {
            self.failure = Some(error);
            event_loop.exit();
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.exiting(event_loop);
    }
}

#[test]
fn native_keep_alive_primary_close_os_proof() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_KEEP_ALIVE_PRIMARY_OS_SMOKE").is_none() {
        return Ok(());
    }
    let mut app = App::new();
    app.keep_alive()?;
    app.with_frame_pacing(FramePacingPolicyResource::on_demand());
    app.with_primary_close_policy(NativePrimaryClosePolicy::Explicit)?;
    app.add_plugins(default_plugins());
    let startup = Arc::new(AtomicUsize::new(0));
    let updates = Arc::new(AtomicUsize::new(0));
    app.insert_resource(PrimaryKeepAliveCounters {
        startup: Arc::clone(&startup),
        updates: Arc::clone(&updates),
    });
    app.add_systems(Startup, request_surviving_secondary.on_invoker_thread());
    app.add_systems(Update, count_primary_keep_alive_update);
    app.admit_composition()?;
    app.prepare_lifecycle_for_execution()?;
    app.prepare_native_ui_host()?;
    let mut state = app.into_windowed_state();
    let initial = state.initial_native_window;
    install_native_window_provider_resources(&mut state.world, initial);

    let mut builder = EventLoop::<NativeHostEvent>::with_user_event();
    builder.with_any_thread(true).with_x11();
    let event_loop = builder.build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));

    let mut proof = NativeKeepAliveProof {
        runner: WinitRunner::new(state),
        stage: 0,
        secondary: None,
        frames_at_close: 0,
        observed: false,
        failure: None,
        deadline: Instant::now() + Duration::from_secs(30),
    };
    event_loop.run_app(&mut proof)?;
    if let Some(error) = proof.failure {
        return Err(error);
    }
    ensure!(
        proof.observed,
        "physical primary close never preserved secondary App execution"
    );
    ensure!(
        proof.runner.fatal_error.is_none(),
        "Native Host reported a fatal error"
    );
    ensure!(
        startup.load(Ordering::SeqCst) == 1,
        "Startup must run exactly once"
    );
    ensure!(
        updates.load(Ordering::SeqCst) >= 2,
        "no ordinary secondary App update"
    );
    println!(
        "native_keep_alive_primary=pass physical_x11=true first_primary_retired=true secondary_survived=true resumed_without_recreation=true app_updates_after_retirement=true"
    );
    Ok(())
}
