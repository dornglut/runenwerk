//! Real Linux/X11 proof of a Native Startup without any initial OS window.
#![cfg(target_os = "linux")]

use super::*;
use crate::app::App;
use crate::plugins::default_plugins;
use crate::runtime::native_window_requests::{
    NativeWindowRequest, NativeWindowRequestOutcome, NativeWindowRequestReadiness,
};
use crate::runtime::system::SystemConfigExt;
use crate::runtime::{Startup, Update, WorldMut};
use anyhow::{Context, Result, ensure};
use runen_ecs::Resource;
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
}

fn assert_windowless_startup(mut world: WorldMut) -> Result<()> {
    let registry = world.resource::<WindowStateRegistryResource>()?;
    ensure!(registry.primary_window_id().is_none(), "Startup fabricated a primary window");
    ensure!(registry.records().next().is_none(), "Startup already had a window record");
    ensure!(
        !world.has_resource::<PrimaryPresentationMetricsResource>(),
        "zero-window Startup fabricated primary presentation metrics"
    );
    world.resource_mut::<WindowlessCounters>()?
        .startups.fetch_add(1, Ordering::SeqCst);
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
}

impl ApplicationHandler<NativeHostEvent> for ExitAfterLateWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.runner.resumed(event_loop);
        assert!(!self.runner.window_realizer.has_primary_window());
        if !self.runner.window_realizer.has_any_window() {
            assert!(!self.runner.frame_requested, "zero-window Startup requested a frame");
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
            let windows = self
                .runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .unwrap();
            assert!(windows.primary_window_id().is_none());
            assert!(windows.record(NativeWindowId::primary()).is_none());
            assert!(windows.records().any(|record| {
                record.native_window_id != NativeWindowId::primary()
                    && record.lifecycle_state == NativeWindowLifecycleState::Created
            }));
            assert!(!self.runner.state.world.has_resource::<crate::plugins::render::Gfx>());
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
    let startups = Arc::new(AtomicUsize::new(0));
    let updates = Arc::new(AtomicUsize::new(0));
    app.insert_resource(WindowlessCounters {
        startups: Arc::clone(&startups),
        updates: Arc::clone(&updates),
    });
    app.add_systems(Startup, assert_windowless_startup.on_invoker_thread());
    app.add_systems(Update, count_update);

    let windows = app.native_window_requests()?;
    ensure!(windows.readiness()? == NativeWindowRequestReadiness::NotReady);
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
    owner.attach_wake(move || proxy.send_event(NativeHostEvent::NativeWindowRequests).is_ok())?;
    state
        .world
        .insert_resource(NativeWakeTransport::new(event_loop.create_proxy()));
    event_loop.set_control_flow(ControlFlow::Wait);

    let worker = thread::spawn(move || -> Result<NativeWindowId> {
        let deadline = Instant::now() + Duration::from_secs(20);
        while windows.readiness()? == NativeWindowRequestReadiness::NotReady {
            ensure!(Instant::now() < deadline, "windowless Winit was never request-ready");
            thread::sleep(Duration::from_millis(5));
        }
        ensure!(windows.readiness()? == NativeWindowRequestReadiness::Ready);
        let ticket = windows.request(NativeWindowRequest::new("late first window", (640, 480)))?;
        match ticket.recv_timeout(Duration::from_secs(20))? {
            NativeWindowRequestOutcome::Created { native_window_id } => {
                ensure!(native_window_id != NativeWindowId::primary());
                Ok(native_window_id)
            }
            other => anyhow::bail!("first late window was not created: {other:?}"),
        }
    });

    let mut proof = ExitAfterLateWindow {
        runner: WinitRunner::new(state),
        observed: false,
    };
    let execution = event_loop.run_app(&mut proof).map_err(anyhow::Error::from);
    let result = worker.join().expect("windowless request worker must not panic");
    execution?;
    let native_id = result.context("worker failed to observe late window")?;
    ensure!(proof.observed, "physical nonprimary window never advanced an ordinary frame");
    ensure!(startups.load(Ordering::SeqCst) == 1, "Startup did not run exactly once");
    ensure!(updates.load(Ordering::SeqCst) > 0, "secondary window never advanced ECS Update");
    ensure!(observer.readiness()? == NativeWindowRequestReadiness::Closed);
    println!(
        "native_zero_window_startup=pass physical_x11=true id={} startup=1 no_primary=true no_gpu=true after_window_update=true",
        native_id.raw()
    );
    Ok(())
}
