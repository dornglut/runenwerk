//! B0: real X11/Winit secondary RunenUI display and OS-delivered input after
//! a different physical primary safely retires. No original-primary UI mount:
//! B1 qualifies the independent original-M13/UI teardown boundary.
#![cfg(target_os = "linux")]

use super::*;
use crate::app::{App, AppNativeHostExt, NativePrimaryClosePolicy};
use crate::plugins::render::host::{RenderSurfaceId, RenderSurfaceRegistryResource};
use crate::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationPolicyResource,
};
use crate::plugins::render::{
    AppRenderExt, Gfx, PreparedFlowInvocationRequest, PreparedRenderFrameRequestResource,
    RenderFlow, RenderFlowId, RenderFrameProducerId, SurfaceFrameSubmissionOrder,
};
use crate::plugins::ui::{
    AppUiExt, NativeInputRecipient, UiNativeInputRecipientsResource, UiRuntimeFontConfiguration,
    UiRuntimeNativeMapping, UiRuntimePresentationAssociationsResource,
    UiRuntimePresentationBinding, UiRuntimeSlotId, UiTypedScreenId, bind_native_input_recipient,
};
use crate::plugins::{RenderPlugin, UiPlugin, default_plugins};
use crate::runtime::system::SystemMobilityExt;
use crate::runtime::{Startup, Update, WorldMut};
use anyhow::{Result, anyhow, ensure};
use runenui_core::{
    Effects, FontFamilyName, GenericFontFamily, LayoutBound, LayoutDimension, LayoutStyle,
    LogicalLength, NoHostProtocol, ProducerBudget, StyleEnvironment, UiApp, View, button,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, InputFocus};
use x11rb::protocol::xtest::ConnectionExt as _;

const FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/fonts/JetBrainsMono-Regular.ttf"
));

struct SurvivingSecondaryUi;

#[derive(Clone, Copy)]
enum SecondaryAction {
    Click,
}

impl UiApp for SurvivingSecondaryUi {
    type State = Arc<AtomicUsize>;
    type Action = SecondaryAction;
    type HostProtocol = NoHostProtocol;

    fn initial_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(3, 0).with_reconciliation(2, 0, 2)
    }

    fn action_producer_budget(_: &Self::State, _: &Self::Action) -> ProducerBudget {
        ProducerBudget::new(4, 1).with_reconciliation(2, 2, 2)
    }

    fn interaction_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(13, 9)
            .with_reconciliation(2, 2, 2)
            .with_work_starts(runenui_core::WorkStartBudget::new(0, 0, 0, 0, 1))
            .with_work_residency(runenui_core::WorkResidencyBudget::new(0, 0, 0, 0, 0, 2))
    }

    fn root(_: &Self::State) -> impl View<Self::Action> {
        button("ACTIVATE SURVIVING SECONDARY")
            .id("native.n2.b0.secondary.button")
            .key("native.n2.b0.secondary.button")
            .on_activate(|| SecondaryAction::Click)
            .with_layout(
                LayoutStyle::default()
                    .with_width(LayoutDimension::Fill)
                    .with_min_height(LayoutBound::length(LogicalLength::from(240_u16))),
            )
    }

    fn update(
        clicks: &mut Self::State,
        action: Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        match action {
            SecondaryAction::Click => {
                clicks.fetch_add(1, Ordering::SeqCst);
            }
        }
        Effects::none()
    }
}

#[derive(runen_ecs::Resource)]
struct SecondaryUiIntent {
    slot: UiRuntimeSlotId,
    flow: RenderFlowId,
    bound: Arc<AtomicBool>,
}

fn request_secondary_ui_window(mut world: WorldMut) -> Result<()> {
    world
        .resource_mut::<WindowStateRegistryResource>()?
        .request_window("N2 B0 surviving UI secondary", (480, 320));
    Ok(())
}

/// The existing exclusive World owner performs both selected-M13 binding and
/// exact Render target selection. No native-primary convenience or raw
/// recipient-registry mutation is permissible for this secondary.
fn bind_and_publish_secondary_ui(mut world: WorldMut) -> Result<()> {
    let intent = world.resource::<SecondaryUiIntent>()?;
    let slot = intent.slot;
    let flow = intent.flow;
    let bound = Arc::clone(&intent.bound);
    if bound.load(Ordering::SeqCst) {
        return Ok(());
    }
    let native = {
        let windows = world.resource::<WindowStateRegistryResource>()?;
        if windows.record(NativeWindowId::primary()).is_some() {
            return Ok(());
        }
        windows
            .records()
            .find(|record| record.lifecycle_state == NativeWindowLifecycleState::Created)
            .map(|record| record.native_window_id)
    };
    let native = native.ok_or_else(|| anyhow!("no actual Created secondary after close"))?;
    let surface = world
        .resource::<RenderSurfaceRegistryResource>()?
        .surface_for_native_window(native)
        .ok_or_else(|| anyhow!("selected secondary Render attachment missing"))?;
    ensure!(
        surface != RenderSurfaceId::primary(),
        "primary Render ID was reused"
    );

    bind_native_input_recipient(&mut world, native, NativeInputRecipient::RunenUiSlot(slot))?;
    let selected = world
        .resource::<UiNativeInputRecipientsResource>()?
        .selected_slot_binding(slot);
    ensure!(
        selected.is_some_and(|(window, _, _)| window == native),
        "actual M13 secondary input binding was not established"
    );
    world
        .resource_mut::<UiRuntimePresentationAssociationsResource>()?
        .bind(
            UiRuntimePresentationBinding::new(
                slot,
                RenderFrameProducerId::try_from_raw(1510).expect("test producer is nonzero"),
                surface,
                StyleEnvironment::default(),
            )
            .with_order(SurfaceFrameSubmissionOrder::new(20, 0)),
        )?;
    world
        .resource_mut::<PreparedRenderFrameRequestResource>()?
        .replace_surface_contribution_with_automatic_main_replacements(
            RenderFrameProducerId::try_from_raw(1511).expect("test RenderFlow producer is nonzero"),
            surface,
            [],
            [PreparedFlowInvocationRequest::new(
                "native.n2.b0.secondary.ui",
                flow,
                "main",
            )],
            [],
        )?;
    bound.store(true, Ordering::SeqCst);
    Ok(())
}

/// Publish actual X server events from a distinct X11 client. Calling the
/// runner's window_event manually would not qualify physical OS input.
fn send_real_secondary_pointer_click(window: &winit::window::Window) -> Result<()> {
    let raw = window.window_handle()?.as_raw();
    let xid = match raw {
        RawWindowHandle::Xlib(handle) => u32::try_from(handle.window)
            .map_err(|_| anyhow!("Xlib window ID exceeded an X11 XID"))?,
        RawWindowHandle::Xcb(handle) => handle.window.get(),
        other => anyhow::bail!("expected a real X11 window, got {other:?}"),
    };
    let (conn, screen_idx) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_idx];
    conn.xtest_get_version(2, 2)?.reply()?;
    let geometry = conn.get_geometry(xid)?.reply()?;
    ensure!(
        geometry.width >= 240 && geometry.height >= 200,
        "secondary physical window extent is insufficient for the button"
    );
    let translated = conn
        .translate_coordinates(xid, screen.root, 0, 0)?
        .reply()?;
    let x = translated
        .dst_x
        .checked_add(90)
        .ok_or_else(|| anyhow!("secondary X11 pointer X overflow"))?;
    let y = translated
        .dst_y
        .checked_add(90)
        .ok_or_else(|| anyhow!("secondary X11 pointer Y overflow"))?;
    conn.set_input_focus(InputFocus::PARENT, xid, x11rb::CURRENT_TIME)?
        .check()?;
    // Core X11 event codes: MotionNotify = 6, ButtonPress = 4,
    // ButtonRelease = 5; detail 1 is the primary pointer button.
    conn.xtest_fake_input(6, 0, 0, screen.root, x, y, 0)?
        .check()?;
    conn.xtest_fake_input(4, 1, 0, screen.root, x, y, 0)?
        .check()?;
    conn.xtest_fake_input(5, 1, 0, screen.root, x, y, 0)?
        .check()?;
    conn.flush()?;
    Ok(())
}

struct NativeSecondaryUiOsProof {
    runner: WinitRunner,
    slot: UiRuntimeSlotId,
    clicks: Arc<AtomicUsize>,
    bound: Arc<AtomicBool>,
    stage: u8,
    secondary: Option<NativeWindowId>,
    surface: Option<RenderSurfaceId>,
    frames_at_close: u64,
    frames_at_input: u64,
    followup_requested: bool,
    observed: bool,
    failure: Option<anyhow::Error>,
    deadline: Instant,
}

impl NativeSecondaryUiOsProof {
    fn secondary_mapping(&self) -> Result<UiRuntimeNativeMapping> {
        let native = self.secondary.ok_or_else(|| anyhow!("missing secondary"))?;
        let surface = self
            .surface
            .ok_or_else(|| anyhow!("missing secondary surface"))?;
        let record = self
            .runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()?
            .record(native)
            .ok_or_else(|| anyhow!("surviving secondary Native record disappeared"))?;
        Ok(UiRuntimeNativeMapping::new(
            native,
            surface,
            record.size_px,
            record.scale_factor,
        )?)
    }

    fn inspect(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        ensure!(
            Instant::now() < self.deadline,
            "N2-B0 physical UI/XTEST proof timed out"
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
                    .ok_or_else(|| anyhow!("secondary was not physically realized"))?;
                let surface = self
                    .runner
                    .state
                    .world
                    .resource::<RenderSurfaceRegistryResource>()?
                    .surface_for_native_window(secondary)
                    .ok_or_else(|| anyhow!("secondary physical Render attachment missing"))?;
                ensure!(
                    surface != RenderSurfaceId::primary(),
                    "secondary reused primary"
                );
                ensure!(
                    self.runner
                        .state
                        .world
                        .resource::<Gfx>()?
                        .adapter_facts()
                        .backend()
                        == runen_gpu::GpuBackendFamily::Vulkan,
                    "B0 requires actual Vulkan on X11"
                );
                // B0 selects no original-primary M13/UI binding by design.
                ensure!(
                    self.runner
                        .state
                        .world
                        .resource::<UiRuntimePresentationAssociationsResource>()?
                        .bindings()
                        .all(|binding| binding.render_surface_id != RenderSurfaceId::primary()),
                    "B0 must not accidentally select original-primary UI"
                );
                self.runner
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(primary)
                    .ok_or_else(|| anyhow!("original primary disappeared before approval"))?
                    .request_close();
                self.runner.apply_window_effects(event_loop)?;
                self.secondary = Some(secondary);
                self.surface = Some(surface);
                self.stage = 1;
            }
            1 => {
                if self.runner.window_realizer.has_primary_window() {
                    self.runner.apply_window_effects(event_loop)?;
                    return Ok(());
                }
                let secondary = self.secondary.ok_or_else(|| anyhow!("missing secondary"))?;
                let surface = self
                    .surface
                    .ok_or_else(|| anyhow!("missing Render target"))?;
                ensure!(
                    self.runner
                        .state
                        .world
                        .resource::<Gfx>()?
                        .has_surface(surface),
                    "physical secondary Gfx was detached with the primary"
                );
                self.runner
                    .state
                    .world
                    .resource_mut::<Gfx>()?
                    .set_gpu_frame_terminal_delivery(true);
                self.frames_at_close = self.runner.ordinary_frames;
                self.stage = 2;
                self.runner.resumed(event_loop);
                self.runner.request_redraw_for_native_window(secondary);
                self.runner.apply_window_effects(event_loop)?;
                self.runner.pace_or_exit(event_loop);
            }
            2 => {
                if !self.bound.load(Ordering::SeqCst) {
                    return Ok(());
                }
                let secondary = self.secondary.ok_or_else(|| anyhow!("missing secondary"))?;
                let surface = self
                    .surface
                    .ok_or_else(|| anyhow!("missing Render surface"))?;
                if !self.followup_requested {
                    self.followup_requested = true;
                    self.runner.request_redraw_for_native_window(secondary);
                    self.runner
                        .request_ordinary_frame("selected secondary UI publication");
                }
                let mapping = self.secondary_mapping()?;
                let displayed = self
                    .runner
                    .state
                    .world
                    .resource::<UiRuntimePresentationAssociationsResource>()?
                    .displayed_for_mapping(self.slot, mapping)
                    .is_some();
                if !displayed {
                    return Ok(());
                }
                ensure!(
                    self.runner.ordinary_frames > self.frames_at_close,
                    "secondary UI displayed without a real post-close App update"
                );
                ensure!(
                    self.runner
                        .state
                        .world
                        .resource::<RenderFrameHistoryState>()?
                        .observations()
                        .any(|frame| {
                            frame.key.render_surface_id == surface.raw()
                                && frame.presentation.present_operation_submitted
                        }),
                    "selected secondary UI has no actual surface Present observation"
                );
                let window = self
                    .runner
                    .window_realizer
                    .realized_windows()
                    .into_iter()
                    .find(|(_, id, _)| *id == secondary)
                    .map(|(_, _, window)| window)
                    .ok_or_else(|| anyhow!("real secondary Winit window disappeared"))?;
                send_real_secondary_pointer_click(&window)?;
                self.frames_at_input = self.runner.ordinary_frames;
                self.stage = 3;
            }
            3 => {
                let clicks = self.clicks.load(Ordering::SeqCst);
                ensure!(
                    clicks <= 1,
                    "one actual XTEST click activated UI more than once"
                );
                if clicks == 0 {
                    return Ok(());
                }
                if self.runner.ordinary_frames <= self.frames_at_input {
                    return Ok(());
                }
                let mapping = self.secondary_mapping()?;
                if self
                    .runner
                    .state
                    .world
                    .resource::<UiRuntimePresentationAssociationsResource>()?
                    .displayed_for_mapping(self.slot, mapping)
                    .is_none()
                {
                    return Ok(());
                }
                let secondary = self.secondary.ok_or_else(|| anyhow!("missing secondary"))?;
                let surface = self
                    .surface
                    .ok_or_else(|| anyhow!("missing Render surface"))?;
                ensure!(
                    !self.runner.window_realizer.has_primary_window()
                        && self
                            .runner
                            .state
                            .world
                            .resource::<RenderSurfaceRegistryResource>()?
                            .surface_for_native_window(secondary)
                            == Some(surface),
                    "typed UI/input continuation revived primary or lost secondary"
                );
                self.observed = true;
                event_loop.exit();
            }
            _ => {}
        }
        Ok(())
    }
}

impl ApplicationHandler<NativeHostEvent> for NativeSecondaryUiOsProof {
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
fn native_secondary_ui_input_os_proof() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_SECONDARY_UI_INPUT_OS_SMOKE").is_none() {
        return Ok(());
    }
    let mut app = App::new();
    app.keep_alive()?;
    app.with_frame_pacing(FramePacingPolicyResource::on_demand());
    app.with_primary_close_policy(NativePrimaryClosePolicy::Explicit)?;
    app.add_plugins(default_plugins());
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);

    let flow = RenderFlow::new("native.n2.b0.secondary.ui")
        .explicit_invocations_only()
        .with_surface_color()?
        .fullscreen_pass("native.n2.b0.clear")
        .main_surface_only()
        .clear_color([0.06, 0.07, 0.09, 1.0])
        .write_surface_color()?
        .finish()
        .builtin_ui_composite_pass("native.n2.b0.ui")?
        .main_surface_only()
        .finish()
        .present_pass("native.n2.b0.present")?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()?;
    let flow_id = flow.id();
    app.add_render_flow(flow);
    app.init_resource::<PreparedRenderFrameRequestResource>();
    app.insert_resource(RenderFrameObservationPolicyResource::enabled(96));

    let clicks = Arc::new(AtomicUsize::new(0));
    let fonts = UiRuntimeFontConfiguration::new(vec![FONT.to_vec()]).with_generic_mapping(
        GenericFontFamily::SansSerif,
        vec![FontFamilyName::new("JetBrains Mono")?],
    );
    let slot = app
        .ui()
        .mount_app::<SurvivingSecondaryUi>(
            UiTypedScreenId::new("native.n2.b0.secondary"),
            Arc::clone(&clicks),
            &fonts,
        )
        .slot_id()
        .ok_or_else(|| anyhow!("typed secondary UI mount was rejected"))?;
    let bound = Arc::new(AtomicBool::new(false));
    app.insert_resource(SecondaryUiIntent {
        slot,
        flow: flow_id,
        bound: Arc::clone(&bound),
    });
    app.add_systems(Startup, request_secondary_ui_window.on_invoker_thread());
    app.add_systems(Update, bind_and_publish_secondary_ui.on_invoker_thread());

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

    let mut proof = NativeSecondaryUiOsProof {
        runner: WinitRunner::new(state),
        slot,
        clicks,
        bound,
        stage: 0,
        secondary: None,
        surface: None,
        frames_at_close: 0,
        frames_at_input: 0,
        followup_requested: false,
        observed: false,
        failure: None,
        deadline: Instant::now() + Duration::from_secs(90),
    };
    event_loop.run_app(&mut proof)?;
    if let Some(error) = proof.failure {
        return Err(error);
    }
    ensure!(
        proof.runner.fatal_error.is_none(),
        "Native Host reported a fatal error"
    );
    ensure!(
        proof.observed && proof.clicks.load(Ordering::SeqCst) == 1,
        "no real post-close secondary UI display and XTEST activation"
    );
    println!(
        "native_secondary_ui_input=pass physical_x11=true vulkan=true primary_retired=true secondary_ui_displayed=true secondary_x11_click_activated=true"
    );
    Ok(())
}
