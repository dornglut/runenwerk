//! N2-B0: real secondary Vulkan+RunenUI display after physical primary close
//! with selected secondary M13 recipient and native callback-route proof.
//! This does NOT replace true X11-generated keyboard/IME/pointer qualification
//! or selected original-primary UI cancellation required by full N2-B.
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
    RenderFlow, RenderFlowId, RenderFrameProducerId, RunenUiPaintSubmissionRegistryResource,
};
use crate::plugins::ui::{
    AppUiExt, NativeInputRecipient, UiNativeInputRecipientsResource, UiRuntimeFontConfiguration,
    UiRuntimeNativeMapping, UiRuntimePresentationAssociationsResource,
    UiRuntimePresentationBinding, UiRuntimeSlotId, UiRuntimeSlotsResource, UiTypedScreenId,
    bind_native_input_recipient,
};
use crate::plugins::{RenderPlugin, UiPlugin, default_plugins};
use crate::runtime::system::SystemMobilityExt;
use crate::runtime::{Startup, Update, WorldMut};
use anyhow::{Result, anyhow, ensure};
use runenui_core::{NoHostProtocol, ProducerBudget, StyleEnvironment, UiApp, View, button};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};
use winit::dpi::PhysicalPosition;
use winit::platform::x11::EventLoopBuilderExtX11;

struct RetainedSecondaryUi;

impl UiApp for RetainedSecondaryUi {
    type State = usize;
    type Action = ();
    type HostProtocol = NoHostProtocol;

    fn initial_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(1, 0).with_reconciliation(1, 0, 1)
    }

    fn action_producer_budget(_: &Self::State, _: &Self::Action) -> ProducerBudget {
        ProducerBudget::new(1, 0).with_reconciliation(1, 1, 1)
    }

    fn interaction_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(13, 9)
            .with_reconciliation(1, 1, 1)
            .with_work_starts(runenui_core::WorkStartBudget::new(0, 0, 0, 0, 1))
            .with_work_residency(runenui_core::WorkResidencyBudget::new(0, 0, 0, 0, 0, 2))
    }

    fn root(_: &Self::State) -> impl View<Self::Action> {
        button("retained secondary input").on_activate(|| ())
    }

    fn update(
        state: &mut Self::State,
        _: (),
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        *state += 1;
    }
}

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
                "native.n2b.postclose.secondary",
                flow,
                "main",
            )],
            [],
        )?;
    published.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

struct NativeSecondaryUiProof {
    runner: WinitRunner,
    flow: RenderFlowId,
    published: Arc<AtomicUsize>,
    slot: UiRuntimeSlotId,
    stage: u8,
    secondary: Option<NativeWindowId>,
    secondary_surface: Option<RenderSurfaceId>,
    frames_at_close: u64,
    ui_redraw_attempts: usize,
    ui_last_requested_frame: u64,
    terminal_ids: std::collections::BTreeSet<u64>,
    observed: bool,
    failure: Option<anyhow::Error>,
    deadline: Instant,
}

impl NativeSecondaryUiProof {
    fn inspect(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        ensure!(
            Instant::now() < self.deadline,
            "N2-B0 watchdog: stage={} primary_realized={} secondary={:?} secondary_surface={:?} postclose_publications={} terminal_g5_receipts={} ordinary_frames={}",
            self.stage,
            self.runner.window_realizer.has_primary_window(),
            self.secondary,
            self.secondary_surface,
            self.published.load(Ordering::SeqCst),
            self.terminal_ids.len(),
            self.runner.ordinary_frames,
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

                // Select the actual independently mounted secondary M13 scope.
                // The World transaction is the only legitimate recipient writer;
                // never mutate the registry directly or silently rebind primary.
                ensure!(
                    bind_native_input_recipient(
                        &mut self.runner.state.world,
                        secondary,
                        NativeInputRecipient::RunenUiSlot(self.slot),
                    )?,
                    "secondary M13 input recipient was not freshly bound"
                );
                self.runner
                    .state
                    .world
                    .resource_mut::<UiRuntimePresentationAssociationsResource>()?
                    .bind(UiRuntimePresentationBinding::new(
                        self.slot,
                        RenderFrameProducerId::try_from_raw(1510)
                            .expect("N2-B0 UI producer identity"),
                        secondary_surface,
                        StyleEnvironment::default(),
                    ))?;

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
                let secondary = self
                    .secondary
                    .ok_or_else(|| anyhow!("missing secondary Native identity"))?;
                let window = self
                    .runner
                    .window_realizer
                    .realized_windows()
                    .into_iter()
                    .find(|(_, native, _)| *native == secondary)
                    .map(|(_, _, window)| window)
                    .ok_or_else(|| anyhow!("secondary physical Winit window disappeared"))?;
                let size = window.inner_size();
                let mapping = UiRuntimeNativeMapping::new(
                    secondary,
                    surface,
                    (size.width, size.height),
                    window.scale_factor(),
                )?;
                let associations = self
                    .runner
                    .state
                    .world
                    .resource::<UiRuntimePresentationAssociationsResource>()?;
                if associations
                    .displayed_for_mapping(self.slot, mapping)
                    .is_none()
                {
                    // This is a real OnDemand App: the first post-close GPU
                    // Present does not settle RunenUI automatically. Explicitly
                    // request a bounded number of ordinary native redraws,
                    // without manually running ECS or fabricating G5 receipts.
                    let binding = associations.binding(self.slot).is_some();
                    let pending = associations.pending(self.slot).is_some();
                    let in_flight = associations.in_flight_count();
                    let any_display = associations
                        .displayed_for_native_recipient(self.slot, secondary)
                        .is_some();
                    let paint = self
                        .runner
                        .state
                        .world
                        .resource::<RunenUiPaintSubmissionRegistryResource>()?
                        .ordered_submissions_for_surface(surface)
                        .len();
                    ensure!(
                        self.ui_redraw_attempts < 8,
                        "secondary RunenUI did not terminally display after {} real post-close redraws: binding={} pending={} in_flight={} displayed_for_native={} paint_submissions={} native_frames={} terminal_g5={}",
                        self.ui_redraw_attempts,
                        binding,
                        pending,
                        in_flight,
                        any_display,
                        paint,
                        self.runner.ordinary_frames,
                        self.terminal_ids.len(),
                    );
                    if self.runner.ordinary_frames > self.ui_last_requested_frame {
                        self.ui_last_requested_frame = self.runner.ordinary_frames;
                        self.ui_redraw_attempts += 1;
                        self.runner.request_redraw_for_native_window(secondary);
                        self.runner.pace_or_exit(event_loop);
                    }
                    return Ok(());
                }
                let recipients = self
                    .runner
                    .state
                    .world
                    .resource::<UiNativeInputRecipientsResource>()?;
                ensure!(
                    recipients.binding_epoch_for_window(secondary).is_some(),
                    "secondary displayed UI lacks an exact live input binding epoch"
                );
                let next = {
                    let gfx = self.runner.state.world.resource::<Gfx>()?;
                    let (gpu, handle) = gfx
                        .physical_surface_proof_handles(surface)
                        .ok_or_else(|| anyhow!("surviving GPU surface handle missing"))?;
                    gpu.acquire_surface_image(handle)?
                };
                next.abandon();

                // Test the existing Winit-to-selected-M13 callback path on the
                // actual secondary identity. These are admitted callbacks,
                // not proof of OS-generated X11 pointer/IME ingress.
                let device = winit::event::DeviceId::dummy();
                let context = self
                    .runner
                    .native_input_mut()
                    .window_context(secondary, device)?;
                let (recipient, epoch) = self.runner.native_input_recipient(secondary)?;
                ensure!(
                    recipient == NativeInputRecipient::RunenUiSlot(self.slot),
                    "secondary native input no longer targets the selected UI slot"
                );
                let epoch = epoch.ok_or_else(|| anyhow!("missing selected secondary M13 epoch"))?;
                let point = PhysicalPosition::new(12.0, 12.0);
                self.runner.apply_semantic_native_input(
                    secondary,
                    PlatformEvent::CursorMoved {
                        context,
                        position: runen_input::Point2::new(
                            point.x as f32,
                            point.y as f32,
                            runen_input::CoordinateSpace::WindowPhysicalPixels,
                        ),
                    },
                )?;
                self.runner
                    .route_ui_cursor_moved(secondary, self.slot, epoch, device, context, point)?;
                for state in [ElementState::Pressed, ElementState::Released] {
                    let (input, disposition) = self.runner.admit_native_mouse_button(
                        secondary,
                        device,
                        state,
                        MouseButton::Left,
                    )?;
                    ensure!(
                        disposition == NativeMouseButtonDisposition::Original,
                        "secondary Winit callback was not admitted as its original native input"
                    );
                    self.runner.route_ui_mouse_button(
                        secondary,
                        self.slot,
                        epoch,
                        input,
                        state,
                        MouseButton::Left,
                    )?;
                }
                self.stage = 3;
                self.runner.request_redraw_for_native_window(secondary);
                self.runner.pace_or_exit(event_loop);
            }
            3 => {
                let activations = self
                    .runner
                    .state
                    .world
                    .resource::<UiRuntimeSlotsResource>()?
                    .observe_app::<RetainedSecondaryUi, _>(self.slot, |app| *app.state())
                    .map_err(|error| {
                        anyhow!("selected secondary M13 observation failed: {error:?}")
                    })?;
                if activations == 0 {
                    let secondary = self
                        .secondary
                        .ok_or_else(|| anyhow!("secondary disappeared"))?;
                    self.runner.request_redraw_for_native_window(secondary);
                    return Ok(());
                }
                ensure!(
                    !self.runner.window_realizer.has_primary_window(),
                    "secondary UI action must not resurrect original primary"
                );
                self.observed = true;
                event_loop.exit();
            }
            _ => {}
        }
        Ok(())
    }
}

impl ApplicationHandler<NativeHostEvent> for NativeSecondaryUiProof {
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
            if !event_loop.exiting() {
                // A diagnostic OS wake, not a synthetic App/Render/Present.
                // Without a bounded Winit opportunity, a failed M13/display
                // producer can sleep forever in ControlFlow::Wait before
                // the proof's own deadline is observed.
                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + Duration::from_millis(20),
                ));
            }
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.exiting(event_loop);
    }
}

#[test]
fn native_keep_alive_ui_os_proof() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_KEEP_ALIVE_UI_OS_SMOKE").is_none() {
        return Ok(());
    }
    let mut app = App::new();
    app.keep_alive()?;
    app.with_frame_pacing(FramePacingPolicyResource::on_demand());
    app.with_primary_close_policy(NativePrimaryClosePolicy::Explicit)?;
    app.add_plugins(default_plugins());
    app.add_plugin(RenderPlugin);
    app.add_plugin(UiPlugin);
    let slot = app
        .ui()
        .mount_app::<RetainedSecondaryUi>(
            UiTypedScreenId::new("native.n2b.secondary.ui"),
            0,
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .ok_or_else(|| anyhow!("secondary typed RunenUI slot failed to mount"))?;

    let flow = RenderFlow::new("native.n2b.postclose.secondary")
        .explicit_invocations_only()
        .with_surface_color()?
        .fullscreen_pass("native.n2.clear")
        .main_surface_only()
        .clear_color([0.125, 0.25, 0.5, 1.0])
        .write_surface_color()?
        .finish()
        .builtin_ui_composite_pass("native.n2b.ui")?
        .main_surface_only()
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

    let mut proof = NativeSecondaryUiProof {
        runner: WinitRunner::new(state),
        flow: flow_id,
        published,
        slot,
        stage: 0,
        secondary: None,
        secondary_surface: None,
        frames_at_close: 0,
        ui_redraw_attempts: 0,
        ui_last_requested_frame: 0,
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
        "same-App post-close secondary RunenUI input and G5 Present were not proven"
    );
    ensure!(
        proof.published.load(Ordering::SeqCst) == 1,
        "secondary producer did not publish exactly once after primary close"
    );
    println!(
        "native_keep_alive_ui=pass physical_x11=true vulkan=true primary_detached=true secondary_ui_displayed=true secondary_m13_callback_activated=true secondary_g5_completed=true"
    );
    Ok(())
}
