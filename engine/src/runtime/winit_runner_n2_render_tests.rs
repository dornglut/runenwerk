//! Same-App native primary retirement followed by a genuine secondary RenderFlow
//! Vulkan submission and terminal G5 Present. The older Native-only N1 and
//! Render-only R0 physical proofs cannot substitute for this combined case.
#![cfg(target_os = "linux")]

use super::*;
use crate::app::{App, AppNativeHostExt, NativePrimaryClosePolicy};
use crate::plugins::render::host::{RenderSurfaceId, RenderSurfaceRegistryResource};
use crate::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationKey, RenderFrameObservationPolicyResource,
};
use crate::plugins::render::renderer::RenderGpuFrameTerminalDisposition;
use crate::plugins::render::{
    AppRenderExt, Gfx, PreparedFlowInvocationRequest, PreparedRenderFrameRequestResource,
    RenderFlow, RenderFlowId, RenderFrameProducerId,
};
use crate::plugins::{RenderPlugin, default_plugins};
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
struct SecondaryRenderIntent {
    flow: RenderFlowId,
    published: Arc<AtomicUsize>,
}

fn request_render_secondary(mut world: WorldMut) -> Result<()> {
    world
        .resource_mut::<WindowStateRegistryResource>()?
        .request_window("N2 surviving Render secondary", (480, 320));
    Ok(())
}

/// App-owned Update, not the legacy hook tied to the original primary Winit
/// window. Publish only after that exact primary has physically disappeared.
fn publish_secondary_render_after_close(mut world: WorldMut) -> Result<()> {
    let intent = world.resource::<SecondaryRenderIntent>()?;
    let flow = intent.flow;
    let published = Arc::clone(&intent.published);
    if published.load(Ordering::SeqCst) != 0 {
        return Ok(());
    }
    let secondary = {
        let windows = world.resource::<WindowStateRegistryResource>()?;
        if windows.record(NativeWindowId::primary()).is_some() {
            return Ok(());
        }
        windows
            .records()
            .find(|record| record.lifecycle_state == NativeWindowLifecycleState::Created)
            .map(|record| record.native_window_id)
    };
    let secondary = secondary.ok_or_else(|| anyhow!("no created secondary after primary close"))?;
    let surface = world
        .resource::<RenderSurfaceRegistryResource>()?
        .surface_for_native_window(secondary)
        .ok_or_else(|| anyhow!("surviving secondary has no Render-owned surface"))?;
    ensure!(
        surface != RenderSurfaceId::primary(),
        "secondary must not inherit original primary Render identity"
    );
    world
        .resource_mut::<PreparedRenderFrameRequestResource>()?
        .replace_surface_contribution_with_automatic_main_replacements(
            RenderFrameProducerId::try_from_raw(1506).expect("fixed proof producer is nonzero"),
            surface,
            [],
            [PreparedFlowInvocationRequest::new(
                "native.n2.postclose.secondary",
                flow,
                "main",
            )],
            [],
        )?;
    published.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

struct NativeSecondaryRenderProof {
    runner: WinitRunner,
    flow: RenderFlowId,
    published: Arc<AtomicUsize>,
    stage: u8,
    secondary: Option<NativeWindowId>,
    secondary_surface: Option<RenderSurfaceId>,
    frames_at_close: u64,
    terminal_ids: std::collections::BTreeSet<u64>,
    observed: bool,
    failure: Option<anyhow::Error>,
    deadline: Instant,
}

impl NativeSecondaryRenderProof {
    fn inspect(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        ensure!(
            Instant::now() < self.deadline,
            "N2 real secondary RenderFlow proof exceeded deadline"
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
                    .find(|native| *native != primary)
                    .ok_or_else(|| anyhow!("secondary was not physically realized"))?;
                let secondary_surface = self
                    .runner
                    .state
                    .world
                    .resource::<RenderSurfaceRegistryResource>()?
                    .surface_for_native_window(secondary)
                    .ok_or_else(|| anyhow!("secondary Render surface was never attached"))?;
                ensure!(
                    secondary_surface != RenderSurfaceId::primary(),
                    "the physical secondary reused the primary Render surface"
                );
                let gfx = self.runner.state.world.resource::<Gfx>()?;
                ensure!(
                    gfx.has_surface(RenderSurfaceId::primary())
                        && gfx.has_surface(secondary_surface)
                        && gfx.adapter_facts().backend() == runen_gpu::GpuBackendFamily::Vulkan,
                    "two physical Vulkan surfaces must share the actual Native App Gfx"
                );

                // Pending and vetoed close cannot retire a physical G7 owner.
                let original = self
                    .runner
                    .native_input_mut()
                    .window_context(primary, winit::event::DeviceId::dummy())?;
                self.runner
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(primary)
                    .ok_or_else(|| anyhow!("primary record missing"))?
                    .receive_close_intent();
                self.runner.apply_window_effects(event_loop)?;
                ensure!(
                    self.runner.window_realizer.has_primary_window()
                        && self
                            .runner
                            .native_input()
                            .window_source_context(primary)
                            .is_some_and(|context| context.source == original.source),
                    "pending primary close changed original source"
                );
                self.runner
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(primary)
                    .ok_or_else(|| anyhow!("primary missing on close veto"))?
                    .veto_close();
                self.runner.apply_window_effects(event_loop)?;
                ensure!(
                    self.runner.window_realizer.has_primary_window(),
                    "vetoed close incorrectly detached the physical primary"
                );

                // A real acquired primary image must make approved retirement
                // defer, without consuming the lease or touching secondary.
                let acquired_primary = {
                    let gfx = self.runner.state.world.resource::<Gfx>()?;
                    let (gpu, handle) = gfx
                        .physical_surface_proof_handles(RenderSurfaceId::primary())
                        .ok_or_else(|| anyhow!("real primary GPU surface handle missing"))?;
                    gpu.acquire_surface_image(handle)?
                };
                self.runner
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(primary)
                    .ok_or_else(|| anyhow!("primary missing before approved close"))?
                    .request_close();
                self.runner.apply_window_effects(event_loop)?;
                ensure!(
                    !event_loop.exiting()
                        && self.runner.window_realizer.has_primary_window()
                        && self
                            .runner
                            .state
                            .world
                            .resource::<Gfx>()?
                            .has_surface(RenderSurfaceId::primary()),
                    "active G7 primary image lease did not block physical close"
                );
                acquired_primary.abandon();

                self.secondary = Some(secondary);
                self.secondary_surface = Some(secondary_surface);
                self.stage = 1;
                self.runner.apply_window_effects(event_loop)?;
            }
            1 => {
                if self.runner.window_realizer.has_primary_window() {
                    // Real GPU progress and existing Native owner retry logic
                    // remain responsible for clearing any deferred G5 work.
                    self.runner.apply_window_effects(event_loop)?;
                    return Ok(());
                }
                let secondary = self.secondary.ok_or_else(|| anyhow!("missing secondary"))?;
                let surface = self
                    .secondary_surface
                    .ok_or_else(|| anyhow!("missing secondary surface"))?;
                let windows = self
                    .runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()?;
                ensure!(
                    windows.record(NativeWindowId::primary()).is_none()
                        && windows.record(secondary).is_some(),
                    "physical close failed to preserve only the secondary Native record"
                );
                let surfaces = self
                    .runner
                    .state
                    .world
                    .resource::<RenderSurfaceRegistryResource>()?;
                ensure!(
                    surfaces
                        .surface_for_native_window(NativeWindowId::primary())
                        .is_none()
                        && surfaces.surface_for_native_window(secondary) == Some(surface),
                    "primary G7 retirement damaged the secondary Render identity"
                );
                let gfx = self.runner.state.world.resource_mut::<Gfx>()?;
                ensure!(
                    !gfx.has_surface(RenderSurfaceId::primary()) && gfx.has_surface(surface),
                    "surviving App lost its secondary physical Vulkan surface"
                );
                gfx.set_gpu_frame_terminal_delivery(true);

                // Real OS resume must never revive original fixed primary ID 1.
                self.runner.resumed(event_loop);
                ensure!(
                    !self.runner.window_realizer.has_primary_window() && !event_loop.exiting(),
                    "resumed resurrected the original primary or stopped the App"
                );
                self.frames_at_close = self.runner.ordinary_frames;
                self.stage = 2;
                self.runner.request_redraw_for_native_window(secondary);
                self.runner.apply_window_effects(event_loop)?;
                self.runner.pace_or_exit(event_loop);
            }
            2 => {
                if self.published.load(Ordering::SeqCst) == 0 {
                    return Ok(());
                }
                let surface = self
                    .secondary_surface
                    .ok_or_else(|| anyhow!("missing Render secondary"))?;
                let batch = self
                    .runner
                    .state
                    .world
                    .resource_mut::<Gfx>()?
                    .take_gpu_frame_terminal_outcomes();
                ensure!(
                    batch.delivery_enabled() && !batch.has_delivery_gap(),
                    "secondary G5 terminal evidence has a delivery gap"
                );
                let history = self
                    .runner
                    .state
                    .world
                    .resource::<RenderFrameHistoryState>()?;
                for outcome in batch.outcomes().iter().filter(|outcome| {
                    outcome.surface() == surface.raw()
                        && outcome
                            .invocations()
                            .iter()
                            .any(|invocation| invocation.flow_id() == self.flow)
                }) {
                    ensure!(
                        matches!(
                            outcome.disposition(),
                            RenderGpuFrameTerminalDisposition::Completed
                        ) && outcome
                            .invocations()
                            .iter()
                            .filter(|invocation| invocation.flow_id() == self.flow)
                            .count()
                            == 1,
                        "post-close secondary flow must be one successfully completed invocation per frame"
                    );
                    ensure!(
                        history
                            .observation(RenderFrameObservationKey::new(
                                outcome.frame(),
                                surface.raw(),
                            ))
                            .is_some_and(|observation| {
                                observation.presentation.present_operation_submitted
                            }),
                        "exact completed secondary G5 frame has no correlated Present record"
                    );
                    ensure!(
                        self.terminal_ids.insert(outcome.submission_id().get()),
                        "duplicate secondary G5 terminal receipt"
                    );
                }
                // Another independently admitted frame may legitimately still
                // be in flight on this secondary. Qualification requires an
                // exact Completed receipt for our flow and a fresh, real G7
                // image acquisition; global submission quiescence would turn
                // an otherwise valid continuing App into a false timeout.
                if self.terminal_ids.is_empty() {
                    return Ok(());
                }
                ensure!(
                    self.runner.ordinary_frames > self.frames_at_close,
                    "terminal secondary flow did not follow a genuine post-close App Update"
                );
                let next = {
                    let gfx = self.runner.state.world.resource::<Gfx>()?;
                    let (gpu, handle) = gfx
                        .physical_surface_proof_handles(surface)
                        .ok_or_else(|| anyhow!("surviving GPU surface handle missing"))?;
                    gpu.acquire_surface_image(handle)?
                };
                next.abandon();
                self.observed = true;
                event_loop.exit();
            }
            _ => {}
        }
        Ok(())
    }
}

impl ApplicationHandler<NativeHostEvent> for NativeSecondaryRenderProof {
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
        if !event_loop.exiting()
            && let Err(error) = self.inspect(event_loop)
        {
            self.failure = Some(error);
            event_loop.exit();
            return;
        }
        if !event_loop.exiting() {
            self.runner.about_to_wait(event_loop);
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.exiting(event_loop);
    }
}

#[test]
fn native_keep_alive_render_os_proof() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_KEEP_ALIVE_RENDER_OS_SMOKE").is_none() {
        return Ok(());
    }
    let mut app = App::new();
    app.keep_alive()?;
    app.with_frame_pacing(FramePacingPolicyResource::on_demand());
    app.with_primary_close_policy(NativePrimaryClosePolicy::Explicit)?;
    app.add_plugins(default_plugins());
    app.add_plugin(RenderPlugin);

    let flow = RenderFlow::new("native.n2.postclose.secondary")
        .explicit_invocations_only()
        .with_surface_color()?
        .fullscreen_pass("native.n2.clear")
        .main_surface_only()
        .clear_color([0.125, 0.25, 0.5, 1.0])
        .write_surface_color()?
        .finish()
        .present_pass("native.n2.present")?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()?;
    let flow_id = flow.id();
    app.add_render_flow(flow);
    app.init_resource::<PreparedRenderFrameRequestResource>();
    app.insert_resource(RenderFrameObservationPolicyResource::enabled(64));
    let published = Arc::new(AtomicUsize::new(0));
    app.insert_resource(SecondaryRenderIntent {
        flow: flow_id,
        published: Arc::clone(&published),
    });
    app.add_systems(Startup, request_render_secondary.on_invoker_thread());
    app.add_systems(
        Update,
        publish_secondary_render_after_close.on_invoker_thread(),
    );
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

    let mut proof = NativeSecondaryRenderProof {
        runner: WinitRunner::new(state),
        flow: flow_id,
        published,
        stage: 0,
        secondary: None,
        secondary_surface: None,
        frames_at_close: 0,
        terminal_ids: std::collections::BTreeSet::new(),
        observed: false,
        failure: None,
        deadline: Instant::now() + Duration::from_secs(75),
    };
    event_loop.run_app(&mut proof)?;
    if let Some(error) = proof.failure {
        return Err(error);
    }
    ensure!(proof.runner.fatal_error.is_none(), "Native Host failed");
    ensure!(
        proof.observed,
        "same-App post-close secondary G5 Present was not proven"
    );
    ensure!(
        proof.published.load(Ordering::SeqCst) == 1,
        "secondary producer did not publish exactly once after primary close"
    );
    println!(
        "native_keep_alive_render=pass physical_x11=true vulkan=true active_primary_lease_deferred=true primary_detached=true secondary_g5_present_completed=true secondary_reacquired=true"
    );
    Ok(())
}
