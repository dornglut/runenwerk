//! Real Linux/X11 proof of a Native Startup without any initial OS window.
#![cfg(target_os = "linux")]

use super::*;
use crate::app::{App, AppNativeHostExt};
use crate::plugins::default_plugins;
use crate::plugins::ui::{UiRuntimeFontConfiguration, UiRuntimeSlotsResource, UiTypedScreenId};
use crate::runtime::app_shutdown::{
    AppShutdownOwner, AppShutdownReadiness, AppShutdownRequestOutcome,
};
use crate::runtime::native_window_requests::{
    NativeWindowRequest, NativeWindowRequestOutcome, NativeWindowRequestReadiness,
    NativeWindowRequestTicket, NativeWindowRequests,
};
use crate::runtime::system::SystemMobilityExt;
use crate::runtime::{Startup, Update, WorldMut};
use anyhow::{Context, Result, anyhow, ensure};
use runen_ecs::Resource;
use std::sync::mpsc;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;
use winit::platform::x11::EventLoopBuilderExtX11;

#[derive(Resource)]
struct WindowlessCounters {
    startups: Arc<AtomicUsize>,
    updates: Arc<AtomicUsize>,
    requests: NativeWindowRequests,
    startup_ticket: mpsc::Sender<NativeWindowRequestTicket>,
    fail_startup: bool,
}

fn assert_windowless_startup(mut world: WorldMut) -> Result<()> {
    let registry = world.resource::<WindowStateRegistryResource>()?;
    ensure!(
        registry.primary_window_id().is_none(),
        "Startup fabricated a primary window"
    );
    ensure!(
        registry.records().next().is_none(),
        "Startup already had a window record"
    );
    ensure!(
        !world.has_resource::<PrimaryPresentationMetricsResource>(),
        "zero-window Startup fabricated primary presentation metrics"
    );
    let counters = world.resource_mut::<WindowlessCounters>()?;
    ensure!(
        counters.requests.readiness()? == NativeWindowRequestReadiness::Ready,
        "Native window command ingress is not ready inside windowless Startup"
    );
    counters.startups.fetch_add(1, Ordering::SeqCst);
    let ticket = counters.requests.request(NativeWindowRequest::new(
        "Startup first late window",
        (640, 480),
    ))?;
    counters
        .startup_ticket
        .send(ticket)
        .map_err(|_| anyhow!("windowless Startup ticket observer is unavailable"))?;
    if counters.fail_startup {
        anyhow::bail!("intentional windowless Startup failure after request acceptance");
    }
    Ok(())
}

fn count_update(counters: runen_ecs::Res<WindowlessCounters>) {
    counters.updates.fetch_add(1, Ordering::SeqCst);
}

/// Only the test wrapper owns this exit; no generic App shutdown API is
/// introduced by the Native zero-window implementation.
struct ExitAfterLateWindow {
    runner: WinitRunner,
    observed: bool,
    expected_windows: usize,
}

impl ApplicationHandler<NativeHostEvent> for ExitAfterLateWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let frames_before_resume = self.runner.ordinary_frames;
        self.runner.resumed(event_loop);
        if !event_loop.exiting() {
            // Model a repeated OS resume on this same real Winit owner.
            // Startup and earlier accepted requests must not replay.
            self.runner.resumed(event_loop);
        }
        assert!(!self.runner.window_realizer.has_primary_window());
        assert_eq!(
            self.runner.ordinary_frames, frames_before_resume,
            "Winit resume or Startup must not produce an ordinary frame"
        );
        if !self.runner.window_realizer.has_any_window() {
            assert!(
                !self.runner.frame_requested,
                "zero-window Startup requested a frame"
            );
            assert!(
                self.runner
                    .state
                    .world
                    .resource::<FramePacingRuntimeStateResource>()
                    .unwrap()
                    .next_frame_delay_ms
                    .is_none(),
                "default continuous pacing generated a no-window deadline"
            );
        }
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
        if self.runner.ordinary_frames > 0 && self.runner.window_realizer.has_any_window() {
            let created = {
                let windows = self
                    .runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap();
                assert!(windows.primary_window_id().is_none());
                assert!(windows.record(NativeWindowId::primary()).is_none());
                windows
                    .records()
                    .filter(|record| record.lifecycle_state == NativeWindowLifecycleState::Created)
                    .map(|record| record.native_window_id)
                    .collect::<Vec<_>>()
            };
            if created.len() < self.expected_windows {
                return;
            }
            assert_eq!(
                created.len(),
                self.expected_windows,
                "unexpected extra Native windows"
            );
            assert!(created.iter().all(|id| *id != NativeWindowId::primary()));
            assert!(
                !self
                    .runner
                    .state
                    .world
                    .has_resource::<crate::plugins::render::Gfx>()
            );
            // An original raw device event has no window identity.
            // It may still advance real app-owned input with only
            // nonprimary physical windows, without inventing a primary.
            self.runner.frame_requested = false;
            self.runner
                .request_redraw_for_native_window(NativeWindowId::primary());
            assert!(
                self.runner.frame_requested,
                "native nonprimary-only OnDemand device input cannot advance"
            );
            assert!(
                self.runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap()
                    .record(NativeWindowId::primary())
                    .is_none()
            );
            // The original Native owner must retire both real secondaries
            // without a primary role or a synthetic ordinary frame.
            let prior_frames = self.runner.ordinary_frames;
            for id in created {
                self.runner
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()
                    .unwrap()
                    .record_mut(id)
                    .unwrap()
                    .request_close();
            }
            self.runner
                .apply_window_effects(event_loop)
                .expect("secondary Native owner retirement must succeed");
            assert!(
                self.runner.window_realizer.realized_windows().is_empty(),
                "secondary physical windows were not retired"
            );
            assert_eq!(self.runner.ordinary_frames, prior_frames);
            self.runner
                .apply_frame_pacing(event_loop)
                .expect("zero-window owner must return to idle pacing");
            assert!(
                self.runner
                    .state
                    .world
                    .resource::<FramePacingRuntimeStateResource>()
                    .unwrap()
                    .next_frame_delay_ms
                    .is_none(),
                "retiring the final secondary must not leave a frame deadline"
            );
            self.observed = true;
            event_loop.exit();
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.exiting(event_loop);
    }
}

#[test]
fn zero_window_native_os_startup() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_ZERO_WINDOW_OS_SMOKE").is_none() {
        return Ok(());
    }

    let mut app = App::new();
    app.enable_windowing()?;
    app.add_plugins(default_plugins());
    let windows = app.native_window_requests()?;
    ensure!(windows.readiness()? == NativeWindowRequestReadiness::NotReady);
    let (startup_sender, startup_receiver) = mpsc::channel();
    let startups = Arc::new(AtomicUsize::new(0));
    let updates = Arc::new(AtomicUsize::new(0));
    app.insert_resource(WindowlessCounters {
        startups: Arc::clone(&startups),
        updates: Arc::clone(&updates),
        requests: windows.clone(),
        startup_ticket: startup_sender,
        fail_startup: false,
    });
    app.add_systems(Startup, assert_windowless_startup.on_invoker_thread());
    app.add_systems(Update, count_update);

    let observer = windows.clone();
    app.admit_composition()?;
    app.prepare_lifecycle_for_execution()?;
    app.prepare_native_ui_host()?;
    let mut state = app.into_windowed_state();
    let initial = state.initial_native_window;
    install_native_window_provider_resources(&mut state.world, initial);

    let mut builder = EventLoop::<NativeHostEvent>::with_user_event();
    builder.with_any_thread(true).with_x11();
    let event_loop = builder.build()?;
    let owner = state.world.resource::<NativeWindowRequestOwner>()?;
    let proxy = event_loop.create_proxy();
    owner.attach_wake(move || {
        proxy
            .send_event(NativeHostEvent::NativeWindowRequests)
            .is_ok()
    })?;
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));
    event_loop.set_control_flow(ControlFlow::Wait);

    let worker = thread::spawn(move || -> Result<(NativeWindowId, NativeWindowId)> {
        let deadline = Instant::now() + Duration::from_secs(20);
        while windows.readiness()? == NativeWindowRequestReadiness::NotReady {
            ensure!(
                Instant::now() < deadline,
                "windowless Winit was never request-ready"
            );
            thread::sleep(Duration::from_millis(5));
        }
        ensure!(windows.readiness()? == NativeWindowRequestReadiness::Ready);
        let ticket = startup_receiver
            .recv_timeout(Duration::from_secs(20))
            .context("Startup did not admit the first Native window request")?;
        let first = match ticket.recv_timeout(Duration::from_secs(20))? {
            NativeWindowRequestOutcome::Created { native_window_id } => native_window_id,
            other => anyhow::bail!("Startup first window was not created: {other:?}"),
        };
        ensure!(first != NativeWindowId::primary());
        let second_ticket =
            windows.request(NativeWindowRequest::new("second late window", (480, 360)))?;
        let second = match second_ticket.recv_timeout(Duration::from_secs(20))? {
            NativeWindowRequestOutcome::Created { native_window_id } => native_window_id,
            other => anyhow::bail!("second late window was not created: {other:?}"),
        };
        ensure!(second != NativeWindowId::primary());
        ensure!(
            second != first,
            "second window reused the first Native identity"
        );
        Ok((first, second))
    });

    let mut proof = ExitAfterLateWindow {
        runner: WinitRunner::new(state),
        observed: false,
        expected_windows: 2,
    };
    let execution = event_loop.run_app(&mut proof).map_err(anyhow::Error::from);
    let result = worker
        .join()
        .expect("windowless request worker must not panic");
    execution?;
    let (first, second) = result.context("worker failed to observe late windows")?;
    ensure!(
        proof.observed,
        "physical nonprimary window never advanced an ordinary frame"
    );
    ensure!(
        startups.load(Ordering::SeqCst) == 1,
        "Startup did not run exactly once"
    );
    ensure!(
        updates.load(Ordering::SeqCst) > 0,
        "secondary window never advanced ECS Update"
    );
    ensure!(observer.readiness()? == NativeWindowRequestReadiness::Closed);
    println!(
        "native_zero_window_startup=pass physical_x11=true first={} second={} startup=1 no_primary=true no_gpu=true after_window_update=true retired=true",
        first.raw(),
        second.raw()
    );
    Ok(())
}

#[test]
fn zero_window_native_os_startup_failure() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_ZERO_WINDOW_STARTUP_FAIL_OS_SMOKE").is_none() {
        return Ok(());
    }

    let mut app = App::new();
    app.enable_windowing()?;
    app.add_plugins(default_plugins());
    let windows = app.native_window_requests()?;
    ensure!(windows.readiness()? == NativeWindowRequestReadiness::NotReady);
    let (startup_sender, startup_receiver) = mpsc::channel();
    let startups = Arc::new(AtomicUsize::new(0));
    let updates = Arc::new(AtomicUsize::new(0));
    app.insert_resource(WindowlessCounters {
        startups: Arc::clone(&startups),
        updates: Arc::clone(&updates),
        requests: windows.clone(),
        startup_ticket: startup_sender,
        fail_startup: true,
    });
    app.add_systems(Startup, assert_windowless_startup.on_invoker_thread());
    app.add_systems(Update, count_update);
    app.admit_composition()?;
    app.prepare_lifecycle_for_execution()?;
    app.prepare_native_ui_host()?;
    let mut state = app.into_windowed_state();
    let initial = state.initial_native_window;
    install_native_window_provider_resources(&mut state.world, initial);

    let mut builder = EventLoop::<NativeHostEvent>::with_user_event();
    builder.with_any_thread(true).with_x11();
    let event_loop = builder.build()?;
    let owner = state.world.resource::<NativeWindowRequestOwner>()?;
    let proxy = event_loop.create_proxy();
    owner.attach_wake(move || {
        proxy
            .send_event(NativeHostEvent::NativeWindowRequests)
            .is_ok()
    })?;
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut proof = ExitAfterLateWindow {
        runner: WinitRunner::new(state),
        observed: false,
        expected_windows: 2,
    };
    event_loop.run_app(&mut proof)?;
    let terminal = startup_receiver
        .recv_timeout(Duration::from_secs(5))
        .context("failed Startup did not record its accepted ticket")?
        .recv_timeout(Duration::from_secs(5))?;
    ensure!(
        matches!(terminal, NativeWindowRequestOutcome::Cancelled),
        "failed Startup stranded or falsely realized its window: {terminal:?}"
    );
    ensure!(windows.readiness()? == NativeWindowRequestReadiness::Closed);
    ensure!(startups.load(Ordering::SeqCst) == 1);
    ensure!(updates.load(Ordering::SeqCst) == 0);
    ensure!(proof.runner.window_realizer.realized_windows().is_empty());
    ensure!(!proof.observed);
    let original_error = proof
        .runner
        .fatal_error
        .as_ref()
        .context("failed Startup did not propagate a fatal Native Host error")?;
    ensure!(
        format!("{original_error:#}")
            .contains("intentional windowless Startup failure after request acceptance"),
        "unexpected Startup failure: {original_error:#}"
    );
    let retry = proof
        .runner
        .run_startup_if_needed()
        .expect_err("failed Startup was retried");
    ensure!(format!("{retry:#}").contains("previously failed"));
    ensure!(startups.load(Ordering::SeqCst) == 1);
    println!(
        "native_zero_window_startup_failure=pass physical_x11=true accepted_cancelled=1 startup_once=true no_physical=true"
    );
    Ok(())
}

#[derive(Resource)]
struct RegistryStartupCounters {
    startups: Arc<AtomicUsize>,
    updates: Arc<AtomicUsize>,
}

fn queue_registry_window_in_startup(mut world: WorldMut) -> Result<()> {
    ensure!(
        !world.has_resource::<PrimaryPresentationMetricsResource>(),
        "registry-first Startup must not fabricate primary presentation metrics"
    );
    let registry = world.resource_mut::<WindowStateRegistryResource>()?;
    ensure!(registry.primary_window_id().is_none());
    ensure!(registry.records().next().is_none());
    let request = registry.request_window("Startup registry first window", (560, 400));
    ensure!(request.native_window_id != NativeWindowId::primary());
    ensure!(registry.pending_creation_requests().len() == 1);
    world
        .resource_mut::<RegistryStartupCounters>()?
        .startups
        .fetch_add(1, Ordering::SeqCst);
    Ok(())
}

fn count_registry_update(counters: runen_ecs::Res<RegistryStartupCounters>) {
    counters.updates.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn zero_window_native_os_registry_startup_request() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_ZERO_WINDOW_REGISTRY_OS_SMOKE").is_none() {
        return Ok(());
    }

    let mut app = App::new();
    app.enable_windowing()?;
    app.add_plugins(default_plugins());
    let startups = Arc::new(AtomicUsize::new(0));
    let updates = Arc::new(AtomicUsize::new(0));
    app.insert_resource(RegistryStartupCounters {
        startups: Arc::clone(&startups),
        updates: Arc::clone(&updates),
    });
    app.add_systems(
        Startup,
        queue_registry_window_in_startup.on_invoker_thread(),
    );
    app.add_systems(Update, count_registry_update);
    app.admit_composition()?;
    app.prepare_lifecycle_for_execution()?;
    app.prepare_native_ui_host()?;
    let mut state = app.into_windowed_state();
    let initial = state.initial_native_window;
    install_native_window_provider_resources(&mut state.world, initial);

    let mut builder = EventLoop::<NativeHostEvent>::with_user_event();
    builder.with_any_thread(true).with_x11();
    let event_loop = builder.build()?;
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut proof = ExitAfterLateWindow {
        runner: WinitRunner::new(state),
        observed: false,
        expected_windows: 1,
    };
    event_loop.run_app(&mut proof)?;

    ensure!(
        proof.observed,
        "frame-independent Startup request never realized"
    );
    ensure!(startups.load(Ordering::SeqCst) == 1);
    ensure!(updates.load(Ordering::SeqCst) > 0);
    ensure!(
        proof
            .runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()?
            .pending_creation_requests()
            .is_empty()
    );
    println!(
        "native_zero_window_registry_startup=pass physical_x11=true startup_once=true queued_first_created=true retired=true"
    );
    Ok(())
}

// Test the actual bypass around initial App composition checks: Startup can
// install a new typed UI runtime after the Native Host has been admitted.
struct StartupUiProbe;

impl runenui_core::UiApp for StartupUiProbe {
    fn initial_producer_budget(_state: &Self::State) -> runenui_core::ProducerBudget {
        runenui_core::ProducerBudget::new(1, 0).with_reconciliation(1, 0, 1)
    }

    fn action_producer_budget(
        _state: &Self::State,
        _action: &Self::Action,
    ) -> runenui_core::ProducerBudget {
        runenui_core::ProducerBudget::new(1, 0).with_reconciliation(1, 1, 1)
    }

    fn interaction_producer_budget(_state: &Self::State) -> runenui_core::ProducerBudget {
        runenui_core::ProducerBudget::new(13, 9)
            .with_reconciliation(1, 1, 1)
            .with_work_starts(runenui_core::WorkStartBudget::new(0, 0, 0, 0, 1))
            .with_work_residency(runenui_core::WorkResidencyBudget::new(0, 0, 0, 0, 0, 2))
    }

    type State = ();
    type Action = ();
    type HostProtocol = runenui_core::NoHostProtocol;

    fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
        runenui_core::button("Startup typed UI probe").on_activate(|| ())
    }

    fn update(
        _: &mut Self::State,
        _: Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
    }
}

#[derive(Resource)]
struct StartupUiCounter(Arc<AtomicUsize>);

fn mount_unsupported_ui_during_startup(mut world: WorldMut) -> Result<()> {
    ensure!(
        !world.has_resource::<UiRuntimeSlotsResource>(),
        "UI probe should be created only during Startup"
    );
    let mut slots = UiRuntimeSlotsResource::default();
    let result = slots.mount_app::<StartupUiProbe>(
        UiTypedScreenId::new("windowless.startup.ui.probe"),
        (),
        &UiRuntimeFontConfiguration::default(),
    );
    ensure!(
        result.accepted_mount(),
        "Startup typed UI mount did not actually succeed: {:?}",
        result.failure()
    );
    world.insert_resource(slots);
    world
        .resource_mut::<StartupUiCounter>()?
        .0
        .fetch_add(1, Ordering::SeqCst);
    Ok(())
}

#[test]
fn zero_window_native_os_startup_rejects_unbound_ui() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_ZERO_WINDOW_UI_OS_SMOKE").is_none() {
        return Ok(());
    }

    let mut app = App::new();
    app.enable_windowing()?;
    app.add_plugins(default_plugins());
    let count = Arc::new(AtomicUsize::new(0));
    app.insert_resource(StartupUiCounter(Arc::clone(&count)));
    app.add_systems(
        Startup,
        mount_unsupported_ui_during_startup.on_invoker_thread(),
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
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut proof = ExitAfterLateWindow {
        runner: WinitRunner::new(state),
        observed: false,
        expected_windows: 2,
    };
    event_loop.run_app(&mut proof)?;

    ensure!(count.load(Ordering::SeqCst) == 1);
    ensure!(!proof.observed);
    ensure!(proof.runner.ordinary_frames == 0);
    ensure!(proof.runner.window_realizer.realized_windows().is_empty());
    ensure!(
        proof
            .runner
            .fatal_error
            .as_ref()
            .is_some_and(|error| {
                format!("{error:#}").contains(
                    "zero-initial-window Native Startup mounted UI without a supported presentation target",
                )
            }),
        "unsupported Startup-authored UI was not reported truthfully"
    );
    println!(
        "native_zero_window_startup_ui=pass physical_x11=true no_primary=true unsupported_mount_rejected=true no_frame=true"
    );
    Ok(())
}

#[derive(Resource)]
struct IdleStartupCounter(Arc<AtomicUsize>);

fn count_idle_startup(mut world: WorldMut) -> Result<()> {
    ensure!(
        !world.has_resource::<PrimaryPresentationMetricsResource>(),
        "idle Native Startup created presentation metrics"
    );
    ensure!(
        world
            .resource::<WindowStateRegistryResource>()?
            .records()
            .next()
            .is_none(),
        "idle Native Startup unexpectedly has a window record"
    );
    world
        .resource_mut::<IdleStartupCounter>()?
        .0
        .fetch_add(1, Ordering::SeqCst);
    Ok(())
}

struct ExitAfterIdleWindowless {
    runner: WinitRunner,
    observed: bool,
}

impl ApplicationHandler<NativeHostEvent> for ExitAfterIdleWindowless {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.resumed(event_loop);
        assert!(!self.runner.window_realizer.has_any_window());
        assert_eq!(self.runner.ordinary_frames, 0);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: NativeHostEvent) {
        self.runner.user_event(event_loop, event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.runner.window_event(event_loop, id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.about_to_wait(event_loop);
        assert!(self.runner.fatal_error.is_none());
        assert!(!self.runner.window_realizer.has_any_window());
        assert!(!self.runner.frame_requested);
        assert_eq!(self.runner.ordinary_frames, 0);
        assert!(
            self.runner
                .state
                .world
                .resource::<FramePacingRuntimeStateResource>()
                .unwrap()
                .next_frame_delay_ms
                .is_none(),
            "an idle zero-window Host must not schedule a synthetic frame"
        );
        self.observed = true;
        event_loop.exit();
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.exiting(event_loop);
    }
}

#[test]
fn zero_window_native_os_idle_modes() -> Result<()> {
    let Some(mode) = std::env::var("RUNENWERK_NATIVE_ZERO_WINDOW_IDLE_OS_SMOKE").ok() else {
        return Ok(());
    };

    let mut app = App::new();
    app.enable_windowing()?;
    app.add_plugins(default_plugins());
    match mode.as_str() {
        "default" => {}
        "uncapped" => {
            app.with_frame_pacing(FramePacingPolicyResource::continuous_uncapped());
        }
        "on-demand" => {
            app.with_frame_pacing(FramePacingPolicyResource::on_demand());
        }
        _ => anyhow::bail!("unsupported windowless idle test mode: {mode}"),
    }
    let startups = Arc::new(AtomicUsize::new(0));
    app.insert_resource(IdleStartupCounter(Arc::clone(&startups)));
    app.add_systems(Startup, count_idle_startup.on_invoker_thread());
    app.admit_composition()?;
    app.prepare_lifecycle_for_execution()?;
    app.prepare_native_ui_host()?;
    let mut state = app.into_windowed_state();
    let initial = state.initial_native_window;
    install_native_window_provider_resources(&mut state.world, initial);

    let mut builder = EventLoop::<NativeHostEvent>::with_user_event();
    builder.with_any_thread(true).with_x11();
    let event_loop = builder.build()?;
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut proof = ExitAfterIdleWindowless {
        runner: WinitRunner::new(state),
        observed: false,
    };
    event_loop.run_app(&mut proof)?;
    ensure!(
        proof.observed,
        "idle Native Host did not reach a settled turn"
    );
    ensure!(startups.load(Ordering::SeqCst) == 1);
    println!(
        "native_zero_window_idle=pass mode={mode} physical_x11=true startup_once=true no_window=true no_frames=true no_deadline=true"
    );
    Ok(())
}

#[test]
fn windowless_native_device_redraw_does_not_infer_primary() -> Result<()> {
    let mut app = App::new();
    app.enable_windowing()?;
    let mut state = app.into_windowed_state();
    let initial = state.initial_native_window;
    install_native_window_provider_resources(&mut state.world, initial);
    let mut runner = WinitRunner::new(state);
    runner.request_redraw_for_native_window(NativeWindowId::primary());
    ensure!(!runner.frame_requested);
    ensure!(
        runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()?
            .primary_window_id()
            .is_none()
    );
    ensure!(
        !runner
            .state
            .world
            .has_resource::<PrimaryPresentationMetricsResource>()
    );
    Ok(())
}

#[test]
fn windowless_late_ui_without_presentation_is_not_silently_admitted() -> Result<()> {
    let mut app = App::new();
    app.enable_windowing()?;
    let state = app.into_windowed_state();
    let mut runner = WinitRunner::new(state);
    ensure!(runner.reject_unsupported_windowless_ui().is_ok());
    let mut slots = UiRuntimeSlotsResource::default();
    let result = slots.mount_app::<StartupUiProbe>(
        UiTypedScreenId::new("windowless.late.ui.probe"),
        (),
        &UiRuntimeFontConfiguration::default(),
    );
    ensure!(result.accepted_mount());
    runner.state.world.insert_resource(slots);
    let error = runner
        .reject_unsupported_windowless_ui()
        .expect_err("a later UI runtime without presentation must not silently progress");
    ensure!(format!("{error:#}").contains("without a supported presentation target"));
    Ok(())
}

#[derive(Resource)]
struct NativeShutdownStartupSignal {
    starts: Arc<AtomicUsize>,
    sender: mpsc::Sender<()>,
}

fn signal_native_shutdown_startup(mut world: WorldMut) -> Result<()> {
    ensure!(
        !world.has_resource::<PrimaryPresentationMetricsResource>(),
        "shutdown-only Startup invented primary metrics"
    );
    ensure!(
        world
            .resource::<WindowStateRegistryResource>()?
            .records()
            .next()
            .is_none(),
        "shutdown-only Startup unexpectedly realized a native window"
    );
    let signal = world.resource_mut::<NativeShutdownStartupSignal>()?;
    signal.starts.fetch_add(1, Ordering::SeqCst);
    signal
        .sender
        .send(())
        .map_err(|_| anyhow!("native shutdown Startup observer disconnected"))?;
    Ok(())
}

struct ExitThroughAppShutdown {
    runner: WinitRunner,
}

impl ApplicationHandler<NativeHostEvent> for ExitThroughAppShutdown {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.resumed(event_loop);
        assert!(!self.runner.window_realizer.has_any_window());
        assert_eq!(self.runner.ordinary_frames, 0);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: NativeHostEvent) {
        self.runner.user_event(event_loop, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.about_to_wait(event_loop);
        assert_eq!(self.runner.ordinary_frames, 0);
        assert!(!self.runner.window_realizer.has_any_window());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.runner.window_event(event_loop, id, event);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.exiting(event_loop);
    }
}

#[test]
fn native_app_shutdown_zero_window_os_proof() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_APP_SHUTDOWN_OS_SMOKE").is_none() {
        return Ok(());
    }

    let mut app = App::new();
    app.enable_windowing()?;
    app.add_plugins(default_plugins());
    let shutdown = app.shutdown_handle()?;
    let window_requests = app.native_window_requests()?;
    ensure!(shutdown.readiness()? == AppShutdownReadiness::NotReady);
    let (sender, receiver) = mpsc::channel();
    let starts = Arc::new(AtomicUsize::new(0));
    app.insert_resource(NativeShutdownStartupSignal {
        starts: Arc::clone(&starts),
        sender,
    });
    app.add_systems(Startup, signal_native_shutdown_startup.on_invoker_thread());
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
    let proxy = event_loop.create_proxy();
    state
        .world
        .resource::<AppShutdownOwner>()?
        .attach_wake(move || proxy.send_event(NativeHostEvent::AppShutdown).is_ok())?;
    let windows_proxy = event_loop.create_proxy();
    state
        .world
        .resource::<NativeWindowRequestOwner>()?
        .attach_wake(move || {
            windows_proxy
                .send_event(NativeHostEvent::NativeWindowRequests)
                .is_ok()
        })?;
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));

    let worker_shutdown = shutdown.clone();
    let worker = thread::spawn(move || -> Result<()> {
        receiver.recv_timeout(std::time::Duration::from_secs(15))?;
        ensure!(
            worker_shutdown.readiness()? == AppShutdownReadiness::Ready,
            "Native shutdown gate was not ready after Startup"
        );
        ensure!(
            worker_shutdown.request_shutdown()? == AppShutdownRequestOutcome::Accepted,
            "first shutdown request was not admitted"
        );
        ensure!(
            worker_shutdown.request_shutdown()? == AppShutdownRequestOutcome::AlreadyStopping,
            "duplicate shutdown request was not idempotent"
        );
        Ok(())
    });

    let mut proof = ExitThroughAppShutdown {
        runner: WinitRunner::new(state),
    };
    event_loop.run_app(&mut proof)?;
    worker
        .join()
        .map_err(|_| anyhow!("native shutdown worker panicked"))??;
    ensure!(proof.runner.fatal_error.is_none());
    ensure!(proof.runner.shutdown_done);
    ensure!(proof.runner.ordinary_frames == 0);
    ensure!(proof.runner.window_realizer.realized_windows().is_empty());
    ensure!(starts.load(Ordering::SeqCst) == 1);
    ensure!(shutdown.readiness()? == AppShutdownReadiness::Closed);
    ensure!(
        window_requests.readiness()? == NativeWindowRequestReadiness::Closed,
        "Native window ingress remained open after App shutdown"
    );
    println!(
        "native_app_shutdown=pass physical_x11=true startup_once=true no_window=true no_frame=true closed=true"
    );
    Ok(())
}

#[derive(Resource)]
struct StopWithPendingRegistryWindow {
    shutdown: crate::runtime::app_shutdown::AppShutdownHandle,
    starts: Arc<AtomicUsize>,
}

fn stop_with_pending_registry_window(mut world: WorldMut) -> Result<()> {
    ensure!(
        !world.has_resource::<PrimaryPresentationMetricsResource>(),
        "shutdown during Startup must not fabricate primary metrics"
    );
    let control = world.resource::<StopWithPendingRegistryWindow>()?;
    let shutdown = control.shutdown.clone();
    control.starts.fetch_add(1, Ordering::SeqCst);
    ensure!(shutdown.readiness()? == AppShutdownReadiness::Ready);
    let request = world
        .resource_mut::<WindowStateRegistryResource>()?
        .request_window("must not realize after stopping", (640, 480));
    ensure!(request.native_window_id != NativeWindowId::primary());
    ensure!(shutdown.request_shutdown()? == AppShutdownRequestOutcome::Accepted);
    Ok(())
}

#[test]
fn native_app_shutdown_during_startup_cancels_legacy_registry_request() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_APP_SHUTDOWN_STARTUP_OS_SMOKE").is_none() {
        return Ok(());
    }

    let mut app = App::new();
    app.enable_windowing()?;
    app.add_plugins(default_plugins());
    let shutdown = app.shutdown_handle()?;
    let starts = Arc::new(AtomicUsize::new(0));
    app.insert_resource(StopWithPendingRegistryWindow {
        shutdown: shutdown.clone(),
        starts: Arc::clone(&starts),
    });
    app.add_systems(
        Startup,
        stop_with_pending_registry_window.on_invoker_thread(),
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
    let proxy = event_loop.create_proxy();
    state
        .world
        .resource::<AppShutdownOwner>()?
        .attach_wake(move || proxy.send_event(NativeHostEvent::AppShutdown).is_ok())?;
    let windows_proxy = event_loop.create_proxy();
    state
        .world
        .resource::<NativeWindowRequestOwner>()?
        .attach_wake(move || {
            windows_proxy
                .send_event(NativeHostEvent::NativeWindowRequests)
                .is_ok()
        })?;
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));

    let mut proof = ExitThroughAppShutdown {
        runner: WinitRunner::new(state),
    };
    event_loop.run_app(&mut proof)?;
    ensure!(proof.runner.fatal_error.is_none());
    ensure!(proof.runner.shutdown_done);
    ensure!(starts.load(Ordering::SeqCst) == 1);
    ensure!(proof.runner.ordinary_frames == 0);
    ensure!(proof.runner.window_realizer.realized_windows().is_empty());
    ensure!(shutdown.readiness()? == AppShutdownReadiness::Closed);
    let registry = proof
        .runner
        .state
        .world
        .resource::<WindowStateRegistryResource>()?;
    ensure!(registry.pending_creation_requests().is_empty());
    ensure!(
        registry
            .records()
            .filter(|record| {
                record.lifecycle_state == NativeWindowLifecycleState::CreationFailed
                    && record
                        .failure_reason
                        .as_ref()
                        .is_some_and(|reason| reason.contains("shutdown cancelled"))
            })
            .count()
            == 1,
        "accepted ECS creation request was not terminally cancelled"
    );
    println!(
        "native_app_shutdown_startup=pass physical_x11=true cancelled_registry=true no_window=true no_frame=true"
    );
    Ok(())
}

#[derive(Resource)]
struct NativePrimaryShutdownSignal {
    shutdown: crate::runtime::app_shutdown::AppShutdownHandle,
    starts: Arc<AtomicUsize>,
    sender: mpsc::Sender<()>,
}

fn signal_native_primary_shutdown_startup(mut world: WorldMut) -> Result<()> {
    ensure!(
        world.has_resource::<PrimaryPresentationMetricsResource>(),
        "required primary metrics absent before primary Native Startup"
    );
    let control = world.resource::<NativePrimaryShutdownSignal>()?;
    ensure!(
        control.shutdown.readiness()? == AppShutdownReadiness::NotReady,
        "primary-first shutdown gate advertised Ready before mandatory Startup"
    );
    control.starts.fetch_add(1, Ordering::SeqCst);
    control
        .sender
        .send(())
        .map_err(|_| anyhow!("primary Native Startup shutdown observer disconnected"))?;
    Ok(())
}

struct ExitPrimaryThroughAppShutdown {
    runner: WinitRunner,
}

impl ApplicationHandler<NativeHostEvent> for ExitPrimaryThroughAppShutdown {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.resumed(event_loop);
        assert!(self.runner.window_realizer.has_primary_window());
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: NativeHostEvent) {
        let frames_before = self.runner.ordinary_frames;
        self.runner.user_event(event_loop, event);
        if matches!(event, NativeHostEvent::AppShutdown) {
            assert_eq!(
                self.runner.ordinary_frames, frames_before,
                "App shutdown wake executed an ordinary frame"
            );
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.about_to_wait(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.runner.window_event(event_loop, id, event);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.exiting(event_loop);
    }
}

#[test]
fn native_app_shutdown_primary_os_proof() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_APP_SHUTDOWN_PRIMARY_OS_SMOKE").is_none() {
        return Ok(());
    }

    let mut app = App::new();
    app.with_frame_pacing(FramePacingPolicyResource::on_demand());
    app.add_plugins(default_plugins());
    let shutdown = app.shutdown_handle()?;
    let (sender, receiver) = mpsc::channel();
    let starts = Arc::new(AtomicUsize::new(0));
    app.insert_resource(NativePrimaryShutdownSignal {
        shutdown: shutdown.clone(),
        starts: Arc::clone(&starts),
        sender,
    });
    app.add_systems(
        Startup,
        signal_native_primary_shutdown_startup.on_invoker_thread(),
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
    let proxy = event_loop.create_proxy();
    state
        .world
        .resource::<AppShutdownOwner>()?
        .attach_wake(move || proxy.send_event(NativeHostEvent::AppShutdown).is_ok())?;
    let windows_proxy = event_loop.create_proxy();
    state
        .world
        .resource::<NativeWindowRequestOwner>()?
        .attach_wake(move || {
            windows_proxy
                .send_event(NativeHostEvent::NativeWindowRequests)
                .is_ok()
        })?;
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));

    let worker_shutdown = shutdown.clone();
    let worker = thread::spawn(move || -> Result<()> {
        receiver.recv_timeout(std::time::Duration::from_secs(15))?;
        let until = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while worker_shutdown.readiness()? != AppShutdownReadiness::Ready {
            ensure!(
                std::time::Instant::now() < until,
                "mandatory native primary never became shutdown-ready"
            );
            thread::sleep(std::time::Duration::from_millis(5));
        }
        ensure!(worker_shutdown.request_shutdown()? == AppShutdownRequestOutcome::Accepted);
        Ok(())
    });

    let mut proof = ExitPrimaryThroughAppShutdown {
        runner: WinitRunner::new(state),
    };
    event_loop.run_app(&mut proof)?;
    worker
        .join()
        .map_err(|_| anyhow!("primary native shutdown worker panicked"))??;
    ensure!(proof.runner.fatal_error.is_none());
    ensure!(proof.runner.shutdown_done);
    ensure!(proof.runner.window_realizer.has_primary_window());
    ensure!(starts.load(Ordering::SeqCst) == 1);
    ensure!(shutdown.readiness()? == AppShutdownReadiness::Closed);
    println!(
        "native_app_shutdown_primary=pass physical_x11=true startup_once=true primary_realized=true no_shutdown_frame=true closed=true"
    );
    Ok(())
}
