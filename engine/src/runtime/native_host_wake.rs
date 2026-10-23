//! Native host time and request-only wake delivery. No UI runtime crosses threads.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};
use std::time::{Duration, Instant};

use runenui_runtime::{MonotonicClock, MonotonicInstant, WakeTransport};
use winit::event_loop::EventLoopProxy;

/// Notifications for the single Engine native event loop, never input payloads.
#[derive(Debug, Clone, Copy)]
pub enum NativeHostEvent {
    RunenUiWake,
}

struct NativeClockEpoch {
    start: Instant,
    exhausted: AtomicBool,
}

/// Created by selected native App configuration before a slot's first mutation.
/// Clones share the exact same epoch; the runner never replaces it.
#[derive(Clone)]
pub(crate) struct NativeHostClock(Arc<NativeClockEpoch>);

impl runen_ecs::Resource for NativeHostClock {}

impl NativeHostClock {
    pub(crate) fn new() -> Self {
        Self(Arc::new(NativeClockEpoch {
            start: Instant::now(),
            exhausted: AtomicBool::new(false),
        }))
    }

    pub(crate) fn same_epoch(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn ensure_valid(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.0.exhausted.load(Ordering::Acquire),
            "native UI monotonic clock exhausted"
        );
        Ok(())
    }

    pub(crate) fn deadline(&self, deadline: MonotonicInstant) -> anyhow::Result<Instant> {
        self.ensure_valid()?;
        self.0
            .start
            .checked_add(Duration::from_nanos(deadline.as_nanos()))
            .ok_or_else(|| anyhow::anyhow!("native UI deadline is not representable"))
    }

    fn elapsed(&self, elapsed: Duration) -> MonotonicInstant {
        if !self.0.exhausted.load(Ordering::Acquire)
            && let Ok(now) = MonotonicInstant::ZERO.checked_add(elapsed)
        {
            return now;
        }
        self.0.exhausted.store(true, Ordering::Release);
        // The framework's clock callback is infallible. A sticky terminal signal
        // prevents host continuation; saturation preserves monotonicity meanwhile.
        MonotonicInstant::ZERO
            .checked_add(Duration::from_nanos(u64::MAX))
            .expect("a u64 nanosecond value is representable")
    }
}

impl MonotonicClock for NativeHostClock {
    fn now(&self) -> MonotonicInstant {
        self.elapsed(self.0.start.elapsed())
    }
}

const IDLE: u8 = 0;
const PENDING: u8 = 1;
const CLOSED: u8 = 2;

struct NativeWakeState {
    proxy: EventLoopProxy<NativeHostEvent>,
    state: AtomicU8,
    delivery_failed: AtomicBool,
}

/// The only cross-thread native UI handle. It owns no World, window or runtime.
#[derive(Clone)]
pub struct NativeWakeTransport(Arc<NativeWakeState>);

impl runen_ecs::Resource for NativeWakeTransport {}

impl NativeWakeTransport {
    pub fn new(proxy: EventLoopProxy<NativeHostEvent>) -> Self {
        Self(Arc::new(NativeWakeState {
            proxy,
            state: AtomicU8::new(IDLE),
            delivery_failed: AtomicBool::new(false),
        }))
    }

    /// Coalesces a notification until the native invoker acknowledges it.
    ///
    /// A closed gate or event-loop rejection never counts as delivered work.
    pub fn notify(&self) -> Result<(), NativeWakeClosed> {
        match self
            .0
            .state
            .compare_exchange(IDLE, PENDING, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => {
                if self
                    .0
                    .proxy
                    .send_event(NativeHostEvent::RunenUiWake)
                    .is_err()
                {
                    self.0.delivery_failed.store(true, Ordering::Release);
                    self.close();
                    return Err(NativeWakeClosed);
                }
                Ok(())
            }
            Err(PENDING) => Ok(()),
            Err(_) => Err(NativeWakeClosed),
        }
    }

    pub(crate) fn acknowledge(&self) {
        // A late callback must not reopen a terminal gate.
        let _ = self
            .0
            .state
            .compare_exchange(PENDING, IDLE, Ordering::AcqRel, Ordering::Acquire);
    }

    pub(crate) fn close(&self) {
        self.0.state.store(CLOSED, Ordering::Release);
    }

    pub(crate) fn ensure_delivery(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.0.delivery_failed.load(Ordering::Acquire),
            "native UI wake rejected by the event loop"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("native event-loop wake transport is closed")]
pub struct NativeWakeClosed;

impl WakeTransport for NativeWakeTransport {
    fn request_wake(&self) {
        if self.notify().is_err() {
            tracing::debug!("RunenUI wake rejected by closed native transport");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_clones_share_epoch_and_exhaustion_is_terminal_without_rebasing() {
        let clock = NativeHostClock::new();
        let clone = clock.clone();
        assert!(clone.same_epoch(&clock));
        assert!(!NativeHostClock::new().same_epoch(&clock));
        assert!(clock.now() <= clone.now());
        let saturated = clock.elapsed(Duration::MAX);
        assert_eq!(saturated.as_nanos(), u64::MAX);
        assert_eq!(clone.now(), saturated);
        assert!(clone.ensure_valid().is_err());
        assert!(clone.deadline(MonotonicInstant::ZERO).is_err());
    }

    /// Run in a quiet native process with RUNENWERK_NATIVE_IDLE_WAKE_SMOKE=1.
    /// RUNENWERK_NATIVE_HOST_LAUNCH_IDLE_WAKE_SMOKE=1 proves launch-selected
    /// OnDemand through the same real timer-free native wake and retirement.
    #[test]
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    fn native_idle_wake_reaches_real_thread_affine_runtime() -> anyhow::Result<()> {
        let programmatic = std::env::var_os("RUNENWERK_NATIVE_IDLE_WAKE_SMOKE").is_some();
        let launch = std::env::var_os("RUNENWERK_NATIVE_HOST_LAUNCH_IDLE_WAKE_SMOKE").is_some();
        if !programmatic && !launch {
            return Ok(());
        }
        anyhow::ensure!(
            !(programmatic && launch),
            "native idle/wake proof modes are mutually exclusive"
        );
        use crate::app::{App, AppNativeHostExt, NativeHostLaunchOverrides};
        use crate::plugins::ui::{
            AppUiExt, UiRuntimeFontConfiguration, UiRuntimeSlotsResource, UiTypedScreenId,
        };
        use crate::runtime::{
            FrameEnd, FramePacingPolicyResource, NativeWindowHook,
            NativeWindowHookRegistryResource, NativeWindowId, SystemMobilityExt, Update,
            WindowStateRegistryResource, WorldMut,
        };
        use runenui_core::{
            Effects, IntoEffects, IntoUpdateOutput, NoHostProtocol, UiApp, View, text,
        };
        use std::cell::Cell;
        use std::rc::Rc;
        use std::sync::{Mutex, mpsc};
        use std::task::{Poll, Waker};
        use std::thread::ThreadId;
        use winit::event_loop::EventLoopBuilder;
        use winit::window::Window;

        struct Work {
            ready: AtomicBool,
            waker: Mutex<Option<Waker>>,
        }
        struct State {
            done: Rc<Cell<bool>>,
            owner: ThreadId,
            work: Arc<Work>,
        }
        struct AsyncApp;
        impl UiApp for AsyncApp {
            type State = State;
            type Action = ();
            type HostProtocol = NoHostProtocol;
            fn root(_: &State) -> impl View<()> {
                text("idle native wake")
            }
            fn initial_effects(state: &State) -> impl IntoEffects<(), NoHostProtocol> {
                let work = state.work.clone();
                Effects::local_task(std::future::poll_fn(move |cx| {
                    if work.ready.load(Ordering::Acquire) {
                        Poll::Ready(Some(()))
                    } else {
                        *work.waker.lock().unwrap() = Some(cx.waker().clone());
                        Poll::Pending
                    }
                }))
            }
            fn update(state: &mut State, _: ()) -> impl IntoUpdateOutput<(), NoHostProtocol> {
                assert_eq!(state.owner, std::thread::current().id());
                state.done.set(true);
                Effects::none()
            }
        }
        struct Probe {
            release: Option<mpsc::Sender<()>>,
            idle_at: Option<Instant>,
            done: Rc<Cell<bool>>,
            frames: usize,
            motion_events: usize,
            retired: Arc<AtomicBool>,
            wake_completed: mpsc::Receiver<()>,
            wake_confirmed: bool,
        }
        impl runen_ecs::Resource for Probe {}
        struct Hook;
        impl NativeWindowHook for Hook {
            fn name(&self) -> &'static str {
                "native_idle_wake_smoke"
            }
            fn configure_event_loop(&mut self, builder: &mut EventLoopBuilder<NativeHostEvent>) {
                #[cfg(target_os = "windows")]
                {
                    use winit::platform::windows::EventLoopBuilderExtWindows;
                    builder.with_any_thread(true);
                }
                #[cfg(target_os = "linux")]
                {
                    use winit::platform::x11::EventLoopBuilderExtX11;
                    builder.with_any_thread(true).with_x11();
                }
            }
            fn attach(&mut self, window: &Window, _: &mut runen_ecs::World) -> anyhow::Result<()> {
                window.set_visible(false);
                Ok(())
            }
            fn device_event(
                &mut self,
                event: &winit::event::DeviceEvent,
                world: &mut runen_ecs::World,
            ) -> anyhow::Result<()> {
                if matches!(event, winit::event::DeviceEvent::MouseMotion { .. }) {
                    let probe = world.resource_mut::<Probe>()?;
                    probe.motion_events = probe.motion_events.saturating_add(1);
                }
                Ok(())
            }
            fn detach(&mut self, world: &mut runen_ecs::World) -> anyhow::Result<()> {
                anyhow::ensure!(
                    world.resource::<UiRuntimeSlotsResource>()?.is_empty(),
                    "idle runtime did not retire"
                );
                anyhow::ensure!(
                    world.resource::<NativeWakeTransport>()?.notify().is_err(),
                    "late idle wake remained open"
                );
                world
                    .resource::<Probe>()?
                    .retired
                    .store(true, Ordering::Release);
                Ok(())
            }
        }
        fn release_when_idle(mut world: WorldMut) -> anyhow::Result<()> {
            if world.resource::<Probe>()?.release.is_none() {
                return Ok(());
            }
            let transport = world.resource::<NativeWakeTransport>()?;
            if transport.0.state.load(Ordering::Acquire) != IDLE {
                return Ok(());
            }
            anyhow::ensure!(
                world
                    .resource::<UiRuntimeSlotsResource>()?
                    .native_readiness()?
                    == (false, None),
                "native idle test has an assisting deadline or continuation"
            );
            let probe = world.resource_mut::<Probe>()?;
            probe.idle_at = Some(Instant::now());
            probe.release.take().unwrap().send(())?;
            Ok(())
        }
        fn observe_completion(mut world: WorldMut) -> anyhow::Result<()> {
            let done = {
                let probe = world.resource_mut::<Probe>()?;
                probe.frames += 1;
                anyhow::ensure!(
                    probe.motion_events == 0,
                    "external native mouse motion contaminated the autonomous wake proof"
                );
                anyhow::ensure!(
                    probe.frames < 8,
                    "native idle probe executed duplicate/polling frames"
                );
                if probe.done.get() {
                    if !probe.wake_confirmed {
                        probe
                            .wake_completed
                            .recv_timeout(Duration::from_millis(100))?;
                        probe.wake_confirmed = true;
                    }
                    true
                } else {
                    false
                }
            };
            if done
                && world
                    .resource::<NativeWakeTransport>()?
                    .0
                    .state
                    .load(Ordering::Acquire)
                    == IDLE
            {
                anyhow::ensure!(
                    world
                        .resource::<Probe>()?
                        .idle_at
                        .is_some_and(|at| at.elapsed() < Duration::from_millis(350)),
                    "idle completion did not reach observed proxy acknowledgement promptly"
                );
                world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(NativeWindowId::primary())
                    .unwrap()
                    .request_close();
            }
            Ok(())
        }
        let done = Rc::new(Cell::new(false));
        let retired = Arc::new(AtomicBool::new(false));
        let work = Arc::new(Work {
            ready: AtomicBool::new(false),
            waker: Mutex::new(None),
        });
        let mut app = App::new();
        app.add_plugin(crate::plugins::InputFinalizePlugin);
        if launch {
            // An explicit launch request overrides the previously composed
            // continuous policy before any native window/slot can start.
            app.with_frame_pacing(FramePacingPolicyResource::continuous_uncapped());
            let request: NativeHostLaunchOverrides = "on-demand".parse()?;
            request.apply_to(&mut app)?;
            anyhow::ensure!(
                app.world().resource::<FramePacingPolicyResource>()?.mode
                    == crate::runtime::FramePacingMode::OnDemand,
                "launch-selected OnDemand failed to override the programmatic Host policy"
            );
        } else {
            app.with_frame_pacing(FramePacingPolicyResource::on_demand());
        }
        anyhow::ensure!(
            app.ui()
                .mount_app::<AsyncApp>(
                    UiTypedScreenId::new("native.idle"),
                    State {
                        done: done.clone(),
                        owner: std::thread::current().id(),
                        work: work.clone(),
                    },
                    &UiRuntimeFontConfiguration::default()
                )
                .slot_id()
                .is_some(),
            "idle app did not mount"
        );
        anyhow::ensure!(
            work.waker.lock().unwrap().is_some(),
            "initial native task did not register Pending"
        );
        let (release, gate) = mpsc::channel();
        let (wake_completed, wake_receipt) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            if gate.recv_timeout(Duration::from_secs(3)).is_err() {
                return;
            }
            std::thread::sleep(Duration::from_millis(120));
            work.ready.store(true, Ordering::Release);
            work.waker.lock().unwrap().take().unwrap().wake();
            let _ = wake_completed.send(());
        });
        app.insert_resource(Probe {
            release: Some(release),
            idle_at: None,
            done: done.clone(),
            frames: 0,
            motion_events: 0,
            retired: retired.clone(),
            wake_completed: wake_receipt,
            wake_confirmed: false,
        });
        app.init_resource::<NativeWindowHookRegistryResource>();
        app.world_mut()
            .resource_mut::<NativeWindowHookRegistryResource>()?
            .register_hook(Hook);
        app.add_systems(FrameEnd, release_when_idle.on_invoker_thread());
        app.add_systems(Update, observe_completion.on_invoker_thread());
        let result = app.run();
        worker
            .join()
            .map_err(|_| anyhow::anyhow!("idle wake worker panicked"))?;
        result?;
        anyhow::ensure!(
            done.get() && retired.load(Ordering::Acquire),
            "idle native completion/retirement missing"
        );
        if launch {
            println!("native_launch_idle_wake_smoke=pass transport_idle=true deadline_none=true");
        } else {
            println!("native_idle_wake_smoke=pass transport_idle=true deadline_none=true");
        }
        Ok(())
    }
}
