use std::collections::BTreeMap;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use engine::plugins::render::host::{
    RenderSurfaceId, RenderSurfaceLifecycleState, RenderSurfaceRegistryResource,
};
use engine::plugins::render::{
    Gfx, GpuParams, GpuStorage, GpuUniform, RenderFixedStepIterationUniform, RenderFlow,
    RenderFlowId, RenderFrameProducerId, RenderGpuFixedStepInput, RenderGpuFlowStepStreams,
    SurfaceFrameSubmissionOrder,
};
use runen_gpu::{GpuBindingKey, GpuSubmissionStatus};
use engine::plugins::ui::{
    AppUiExt, UiRuntimeFontConfiguration, UiRuntimeNativeMapping,
    UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding, UiRuntimeSlotId,
    UiRuntimeSlotsResource, UiScreen, UiTypedActionDescriptor, UiTypedScreenId, UiTypedSource,
};
use engine::plugins::{RenderPlugin, UiPlugin, default_plugins};
use engine::prelude::{App, AppRenderExt, Res, Startup, Update};
use engine::runtime::{
    NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId, NativeWindowLifecycleState,
    WindowStateRegistryResource,
};
use runenui_core::{FontFamilyName, GenericFontFamily, StyleEnvironment};
use ui_controls::BUTTON_CONTROL_KIND_ID;
use ui_definition::{
    AuthoredBindingRef, AuthoredControlAccessibilityDefinition, AuthoredControlKindId,
    AuthoredControlValue, AuthoredId, AuthoredRouteId, UiNodeDefinition, UiValueBinding,
};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion, UiProgramSourceId};
use ui_schema::UiSchemaRef;
use winit::window::Window;

const NO_RENDER_ENV: &str = "RUNENWERK_NATIVE_NO_RENDER_SMOKE";
const RENDER_HOST_ENV: &str = "RUNENWERK_NATIVE_RENDER_HOST_SMOKE";
const RENDER_UI_HOST_ENV: &str = "RUNENWERK_NATIVE_RENDER_UI_HOST_SMOKE";
const HISTORICAL_GPU_ENV: &str = "RUNENWERK_NATIVE_HISTORICAL_GPU_SMOKE";

const COUNTER_FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/fonts/JetBrainsMono-Regular.ttf"
));

/// Same genuinely authored Counter root/control as the headless direct U5
/// integration proof; native execution additionally requires terminal Present.
#[derive(Debug, Clone, Copy)]
struct NativeCounterScreen;

impl UiScreen for NativeCounterScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new("counter.screen")
    }

    fn build_source(&self) -> UiTypedSource {
        let mut properties = BTreeMap::new();
        properties.insert(
            "label".to_owned(),
            AuthoredControlValue::String("Counter output".to_owned()),
        );
        let mut bindings = BTreeMap::new();
        bindings.insert(
            "selected".to_owned(),
            AuthoredBindingRef::new("counter.output.selected"),
        );
        UiTypedSource::new(
            self.screen_id(),
            UiProgramSourceId::new("counter.screen.source"),
            UiNodeDefinition::Column {
                id: AuthoredId::new("counter.root"),
                children: vec![
                    UiNodeDefinition::Label {
                        id: AuthoredId::new("counter.title"),
                        label: UiValueBinding::static_text("Counter"),
                        availability: None,
                    },
                    UiNodeDefinition::Control {
                        id: AuthoredId::new("counter.output"),
                        kind: AuthoredControlKindId::new(BUTTON_CONTROL_KIND_ID),
                        properties,
                        bindings,
                        route: Some(AuthoredRouteId::new("counter.increment")),
                        accessibility: Some(AuthoredControlAccessibilityDefinition {
                            role: "button".to_owned(),
                            label: Some("Counter output".to_owned()),
                        }),
                        children: Vec::new(),
                    },
                ],
            },
        )
        .with_action_descriptor(UiTypedActionDescriptor::new(
            engine::plugins::ui::UiTypedActionId::new("counter.increment.action"),
            RouteId::new("counter.increment"),
            RouteSchemaVersion::new(1),
            UiSchemaRef::new("runenwerk.ui.controls.button.event", 1),
            RouteCapability::new("counter.action.increment"),
        ))
    }
}

fn native_counter_fonts() -> UiRuntimeFontConfiguration {
    UiRuntimeFontConfiguration::new(vec![COUNTER_FONT.to_vec()]).with_generic_mapping(
        GenericFontFamily::SansSerif,
        vec![FontFamilyName::new("JetBrains Mono").expect("controlled font family")],
    )
}

fn main() {
    if let Ok(mode) = std::env::var("RUNENWERK_NATIVE_UI_READINESS_SMOKE") {
        assert!(
            [NO_RENDER_ENV, RENDER_HOST_ENV, RENDER_UI_HOST_ENV, HISTORICAL_GPU_ENV]
                .iter()
                .all(|flag| std::env::var_os(flag).is_none()),
            "native Host smoke modes are mutually exclusive"
        );
        native_readiness::run(&mode).expect("native UI readiness smoke must succeed");
        return;
    }
    match (
        std::env::var_os(NO_RENDER_ENV).is_some(),
        std::env::var_os(RENDER_HOST_ENV).is_some(),
        std::env::var_os(RENDER_UI_HOST_ENV).is_some(),
        std::env::var_os(HISTORICAL_GPU_ENV).is_some(),
    ) {
        (false, false, false, false) => {}
        (true, false, false, false) => {
            native_no_render_host_smoke().expect("native no-Render Host smoke should succeed")
        }
        (false, true, false, false) => {
            native_render_host_smoke(false).expect("bare native Render Host smoke should succeed");
            run_native_render_ui_child_smoke()
                .expect("native Render + UI Host child smoke should succeed");
        }
        (false, false, true, false) => {
            native_render_host_smoke(true).expect("native Render + UI Host smoke should succeed")
        }
        (false, false, false, true) => {
            native_historical_gpu_smoke().expect("native historical GPU RenderFlow must succeed")
        }
        _ => panic!("native Host smoke modes are mutually exclusive"),
    }
}

/// Actual App-configured runtimes and native callback execution, with no UI/game
/// arbitration or synthetic scheduler/deadline implementation in the fixture.
mod native_readiness {
    use super::*;
    use engine::app::AppNativeHostExt;
    use engine::runtime::{
        FrameEnd, FramePacingPolicyResource, NativeHostEvent, NativeWakeTransport,
        SystemMobilityExt, WorldMut,
    };
    use runenui_core::{
        Effects, IntoEffects, IntoUpdateOutput, NoHostProtocol, TimerEffect, UiApp, View,
        WorkFamily, WorkKey, text,
    };
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::{Mutex, mpsc};
    use std::task::{Poll, Waker};
    use std::thread::ThreadId;
    use std::time::{Duration, Instant};
    use winit::event::WindowEvent;
    use winit::event_loop::EventLoop;

    struct External {
        ready: AtomicBool,
        waker: Mutex<Option<Waker>>,
        registered: Mutex<Option<mpsc::Sender<()>>>,
    }

    struct State {
        owner: ThreadId,
        events: Rc<RefCell<Vec<&'static str>>>,
        external: Option<Arc<External>>,
    }

    #[derive(Clone, Copy)]
    enum Action {
        Timer,
        Async,
        Cancel,
        Replace,
        Secondary,
        ArmWatchdog,
    }

    struct TimedApp;
    impl UiApp for TimedApp {
        type State = State;
        type Action = Action;
        type HostProtocol = NoHostProtocol;
        fn root(_: &State) -> impl View<Action> {
            text("native timed application")
        }
        fn initial_effects(_: &State) -> impl IntoEffects<Action, NoHostProtocol> {
            Effects::timer(
                TimerEffect::once(Duration::from_secs(5), || Action::Timer)
                    .keyed(WorkKey::new("native-timer").unwrap()),
            )
        }
        fn update(
            state: &mut State,
            action: Action,
        ) -> impl IntoUpdateOutput<Action, NoHostProtocol> {
            assert_eq!(state.owner, std::thread::current().id());
            match action {
                Action::Timer => {
                    state.events.borrow_mut().push("timer");
                    Effects::none()
                }
                Action::Secondary => {
                    state.events.borrow_mut().push("secondary");
                    Effects::none()
                }
                Action::Cancel => {
                    Effects::cancel(WorkFamily::Timer, WorkKey::new("native-timer").unwrap())
                }
                Action::Replace => Effects::timer(
                    TimerEffect::once(Duration::from_millis(50), || Action::Timer)
                        .keyed(WorkKey::new("native-timer").unwrap()),
                ),
                Action::Async | Action::ArmWatchdog => {
                    unreachable!("TimedApp has no asynchronous effect")
                }
            }
        }
    }

    struct AsyncApp;
    impl UiApp for AsyncApp {
        type State = State;
        type Action = Action;
        type HostProtocol = NoHostProtocol;
        fn root(_: &State) -> impl View<Action> {
            text("native asynchronous application")
        }
        fn initial_effects(state: &State) -> impl IntoEffects<Action, NoHostProtocol> {
            let external = state.external.as_ref().unwrap().clone();
            Effects::local_task(std::future::poll_fn(move |cx| {
                if external.ready.load(Ordering::Acquire) {
                    Poll::Ready(Some(Action::Async))
                } else {
                    *external.waker.lock().unwrap() = Some(cx.waker().clone());
                    if let Some(sender) = external.registered.lock().unwrap().take() {
                        sender.send(()).unwrap();
                    }
                    Poll::Pending
                }
            }))
        }
        fn update(
            state: &mut State,
            action: Action,
        ) -> impl IntoUpdateOutput<Action, NoHostProtocol> {
            assert_eq!(state.owner, std::thread::current().id());
            if matches!(action, Action::ArmWatchdog) {
                return Effects::timer(TimerEffect::once(Duration::from_millis(600), || {
                    Action::Timer
                }));
            }
            state.events.borrow_mut().push(match action {
                Action::Async => "async",
                Action::Timer => "second-timer",
                _ => unreachable!("AsyncApp only handles its own effect completions"),
            });
            Effects::none()
        }
    }

    struct Probe {
        timed: UiRuntimeSlotId,
        asynchronous: UiRuntimeSlotId,
        events: Rc<RefCell<Vec<&'static str>>>,
        other: Rc<RefCell<Vec<&'static str>>>,
        frames: usize,
        motion_events: usize,
        observed_motion_events: usize,
        unexplained_frames: usize,
        continuous: bool,
        last_frame: Option<Instant>,
        hook_frames: usize,
        replaced: bool,
        cross: bool,
        fail: bool,
        start: Instant,
        transport: Option<NativeWakeTransport>,
        retired: Arc<AtomicBool>,
        worker_release: Option<mpsc::Sender<()>>,
        armed_at: Option<Instant>,
        idle_started: Option<Instant>,
        async_validated: bool,
    }
    impl runen_ecs::Resource for Probe {}

    struct Hook {
        visible: bool,
        primary_window: Option<winit::window::WindowId>,
    }
    impl NativeWindowHook for Hook {
        fn name(&self) -> &'static str {
            "native_ui_readiness_smoke"
        }
        fn attach(&mut self, window: &Window, world: &mut runen_ecs::World) -> anyhow::Result<()> {
            // Winit's X11 title getter returns an empty string. Identify the
            // first (primary) attachment by native identity, never its title.
            let primary = *self.primary_window.get_or_insert(window.id());
            let secondary = window.id() != primary;
            window.set_visible(self.visible || secondary);
            let transport = world.resource::<NativeWakeTransport>()?.clone();
            world.resource_mut::<Probe>()?.transport = Some(transport);
            if secondary {
                let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(641, 481));
            }
            Ok(())
        }
        fn window_event(
            &mut self,
            window: &Window,
            event: &WindowEvent,
            world: &mut runen_ecs::World,
        ) -> anyhow::Result<()> {
            if Some(window.id()) != self.primary_window && matches!(event, WindowEvent::Resized(_))
            {
                let slot = world.resource::<Probe>()?.timed;
                world
                    .resource_mut::<UiRuntimeSlotsResource>()?
                    .submit_app_action::<TimedApp>(slot, Action::Secondary)
                    .map_err(|error| {
                        anyhow::anyhow!("secondary callback action failed: {error:?}")
                    })?;
            }
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
        fn frame(&mut self, _window: &Window, world: &mut runen_ecs::World) -> anyhow::Result<()> {
            world.resource_mut::<Probe>()?.hook_frames += 1;
            Ok(())
        }
        fn detach(&mut self, world: &mut runen_ecs::World) -> anyhow::Result<()> {
            anyhow::ensure!(
                world.resource::<UiRuntimeSlotsResource>()?.is_empty(),
                "native slot retirement was not completed"
            );
            let probe = world.resource::<Probe>()?;
            anyhow::ensure!(
                probe.transport.as_ref().unwrap().notify().is_err(),
                "late wake was accepted after terminal cleanup"
            );
            probe.retired.store(true, Ordering::Release);
            Ok(())
        }
    }

    fn update(mut world: WorldMut) -> anyhow::Result<()> {
        let (slot, replace, cross, finish) = {
            let probe = world.resource_mut::<Probe>()?;
            let now = Instant::now();
            if probe.continuous
                && let Some(last) = probe.last_frame
            {
                anyhow::ensure!(
                    now.duration_since(last) >= Duration::from_millis(30),
                    "continuous native frames exceeded the 30 FPS cap"
                );
            }
            probe.last_frame = Some(now);
            probe.frames += 1;
            // Raw motion legitimately wakes OnDemand on an interactive desktop.
            // Credit only this frame when new motion arrived; discard coalesced
            // surplus so it cannot mask later self-driven polling frames.
            if probe.motion_events == probe.observed_motion_events {
                probe.unexplained_frames += 1;
            }
            probe.observed_motion_events = probe.motion_events;
            anyhow::ensure!(
                probe.hook_frames == probe.frames,
                "window callbacks advanced duplicate Engine frames"
            );
            if probe.fail {
                anyhow::bail!("injected native readiness frame failure");
            }
            anyhow::ensure!(
                probe.unexplained_frames < 80 && probe.start.elapsed() < Duration::from_secs(3),
                "native readiness failed to make bounded progress: frames={}, unexplained={}, motions={}, elapsed={:?}, timed={:?}, asynchronous={:?}, idle_started={:?}",
                probe.frames,
                probe.unexplained_frames,
                probe.motion_events,
                probe.start.elapsed(),
                probe.events.borrow(),
                probe.other.borrow(),
                probe.idle_started.map(|start| start.elapsed())
            );
            let async_seen = probe.other.borrow().contains(&"async");
            let secondary = probe.events.borrow().contains(&"secondary");
            if async_seen && !probe.async_validated {
                anyhow::ensure!(
                    probe
                        .idle_started
                        .is_some_and(|start| start.elapsed() < Duration::from_millis(350)),
                    "async completion relied on a due timer"
                );
                probe.async_validated = true;
            }
            let replace = async_seen && secondary && !probe.replaced;
            if replace {
                probe.replaced = true;
            }
            let finish = probe.replaced
                && probe.events.borrow().contains(&"timer")
                && probe.other.borrow().contains(&"second-timer");
            (probe.timed, replace, probe.cross, finish)
        };
        if replace {
            let slots = world.resource_mut::<UiRuntimeSlotsResource>()?;
            slots
                .submit_app_action::<TimedApp>(slot, Action::Cancel)
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            slots
                .submit_app_action::<TimedApp>(slot, Action::Replace)
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            if cross {
                std::thread::sleep(Duration::from_millis(75));
            }
        }
        if finish {
            world
                .resource_mut::<WindowStateRegistryResource>()?
                .record_mut(NativeWindowId::primary())
                .unwrap()
                .request_close();
        }
        Ok(())
    }

    fn release_idle_worker(mut world: WorldMut) -> anyhow::Result<()> {
        if world.resource::<Probe>()?.armed_at.is_none() {
            let slot = world.resource::<Probe>()?.asynchronous;
            world
                .resource_mut::<UiRuntimeSlotsResource>()?
                .submit_app_action::<AsyncApp>(slot, Action::ArmWatchdog)
                .map_err(|error| anyhow::anyhow!("idle watchdog admission failed: {error:?}"))?;
            world.resource_mut::<Probe>()?.armed_at = Some(Instant::now());
            // Arming posts a native notification. Drain that turn before
            // allowing external completion to make the application ready.
            return Ok(());
        }
        let sender = world.resource_mut::<Probe>()?.worker_release.take();
        if let Some(sender) = sender {
            anyhow::ensure!(
                world.resource::<Probe>()?.armed_at.unwrap().elapsed() < Duration::from_millis(100),
                "watchdog is not safely later than the idle receipt interval"
            );
            anyhow::ensure!(
                !world
                    .resource::<Probe>()?
                    .other
                    .borrow()
                    .contains(&"second-timer"),
                "watchdog already fired before idle release"
            );
            world.resource_mut::<Probe>()?.idle_started = Some(Instant::now());
            // The async effect was Pending at configuration and stays unready
            // through the complete arming turn and its subsequent native frame.
            // The worker alone supplies its later completion notification.
            sender.send(()).unwrap();
        }
        Ok(())
    }

    pub(super) fn run(mode: &str) -> anyhow::Result<()> {
        if mode == "late-manual-registry" {
            struct RetirementProbe(Arc<AtomicBool>);
            impl NativeWindowHook for RetirementProbe {
                fn name(&self) -> &'static str {
                    "late_manual_registry_smoke"
                }
                fn attach(
                    &mut self,
                    window: &Window,
                    _: &mut runen_ecs::World,
                ) -> anyhow::Result<()> {
                    window.set_visible(false);
                    Ok(())
                }
                fn detach(&mut self, world: &mut runen_ecs::World) -> anyhow::Result<()> {
                    anyhow::ensure!(
                        world.resource::<UiRuntimeSlotsResource>()?.is_empty(),
                        "rejected late registry did not retire"
                    );
                    anyhow::ensure!(
                        world.resource::<NativeWakeTransport>()?.notify().is_err(),
                        "late rejected registry left wake gate open"
                    );
                    self.0.store(true, Ordering::Release);
                    Ok(())
                }
            }
            fn install_manual_registry(mut world: WorldMut) -> anyhow::Result<()> {
                let mut slots = UiRuntimeSlotsResource::default();
                anyhow::ensure!(
                    slots
                        .mount_app::<TimedApp>(
                            UiTypedScreenId::new("late.manual"),
                            State {
                                owner: std::thread::current().id(),
                                events: Rc::new(RefCell::new(Vec::new())),
                                external: None,
                            },
                            &UiRuntimeFontConfiguration::default()
                        )
                        .slot_id()
                        .is_some(),
                    "manual mount failed"
                );
                world.insert_resource(slots);
                Ok(())
            }
            let retired = Arc::new(AtomicBool::new(false));
            let mut app = App::new();
            app.add_plugin(engine::plugins::InputFinalizePlugin);
            app.with_frame_pacing(FramePacingPolicyResource::on_demand());
            app.init_resource::<NativeWindowHookRegistryResource>();
            app.world_mut()
                .resource_mut::<NativeWindowHookRegistryResource>()?
                .register_hook(RetirementProbe(retired.clone()));
            app.add_systems(Startup, install_manual_registry.on_invoker_thread());
            anyhow::ensure!(app.run().is_err_and(|error| format!("{error:#}").contains("native clock qualification")),
                "late manual runtime silently entered native execution");
            anyhow::ensure!(
                retired.load(Ordering::Acquire),
                "late registry retirement was not observed"
            );
            println!("native_ui_readiness_smoke=pass mode=late-manual-registry");
            return Ok(());
        }
        if mode == "closed-proxy" {
            let event_loop = EventLoop::<NativeHostEvent>::with_user_event().build()?;
            let transport = NativeWakeTransport::new(event_loop.create_proxy());
            drop(event_loop);
            anyhow::ensure!(transport.notify().is_err(), "destroyed loop accepted wake");
            anyhow::ensure!(
                transport.notify().is_err(),
                "rejected wake became falsely coalesced success"
            );
            println!("native_ui_readiness_smoke=pass mode=closed-proxy");
            return Ok(());
        }
        anyhow::ensure!(
            matches!(
                mode,
                "hidden" | "visible" | "crossed-deadline" | "failed-frame" | "continuous"
            ),
            "unknown readiness mode"
        );
        let (registered, registration) = mpsc::channel();
        let external = Arc::new(External {
            ready: AtomicBool::new(false),
            waker: Mutex::new(None),
            registered: Mutex::new(Some(registered)),
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let other = Rc::new(RefCell::new(Vec::new()));
        let retired = Arc::new(AtomicBool::new(false));
        let owner = std::thread::current().id();
        let start = Instant::now();
        let mut app = App::new();
        app.add_plugin(engine::plugins::InputFinalizePlugin);
        app.with_frame_pacing(if mode == "continuous" {
            FramePacingPolicyResource::continuous_capped(30)
        } else {
            FramePacingPolicyResource::on_demand()
        });
        let timed = app
            .ui()
            .mount_app::<TimedApp>(
                UiTypedScreenId::new("native.timed"),
                State {
                    owner,
                    events: events.clone(),
                    external: None,
                },
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .ok_or_else(|| anyhow::anyhow!("native timer mount rejected"))?;
        let asynchronous = app
            .ui()
            .mount_app::<AsyncApp>(
                UiTypedScreenId::new("native.async"),
                State {
                    owner,
                    events: other.clone(),
                    external: Some(external.clone()),
                },
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .ok_or_else(|| anyhow::anyhow!("native async mount rejected"))?;
        // First Pending poll must occur during App configuration before proxy attachment.
        registration.recv_timeout(Duration::from_secs(1))?;
        let (worker_release, idle_gate) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            match idle_gate.recv_timeout(Duration::from_secs(3)) {
                Ok(()) => {}
                // A failed frame never reaches FrameEnd. World retirement
                // drops its sender, so the worker exits without UI execution.
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    panic!("native idle gate did not complete")
                }
            }
            std::thread::sleep(Duration::from_millis(120));
            external.ready.store(true, Ordering::Release);
            external.waker.lock().unwrap().take().unwrap().wake();
        });
        app.init_resource::<WindowStateRegistryResource>();
        app.world_mut()
            .resource_mut::<WindowStateRegistryResource>()?
            .request_window("Native readiness secondary", (640, 480));
        app.insert_resource(Probe {
            timed,
            asynchronous,
            events,
            other,
            frames: 0,
            motion_events: 0,
            observed_motion_events: 0,
            unexplained_frames: 0,
            continuous: mode == "continuous",
            last_frame: None,
            hook_frames: 0,
            replaced: false,
            cross: mode == "crossed-deadline",
            fail: mode == "failed-frame",
            start,
            transport: None,
            retired: retired.clone(),
            worker_release: Some(worker_release),
            armed_at: None,
            idle_started: None,
            async_validated: false,
        });
        app.add_systems(Update, update.on_invoker_thread());
        app.add_systems(FrameEnd, release_idle_worker.on_invoker_thread());
        app.init_resource::<NativeWindowHookRegistryResource>();
        app.world_mut()
            .resource_mut::<NativeWindowHookRegistryResource>()?
            .register_hook(Hook {
                visible: mode == "visible",
                primary_window: None,
            });
        let result = app.run();
        worker
            .join()
            .map_err(|_| anyhow::anyhow!("native wake worker panicked"))?;
        anyhow::ensure!(
            retired.load(Ordering::Acquire),
            "native shutdown was not observed"
        );
        if mode == "failed-frame" {
            anyhow::ensure!(
                result.is_err_and(|error| format!("{error:#}")
                    .contains("injected native readiness frame failure")),
                "expected native frame failure was not returned"
            );
        } else {
            result?;
        }
        println!("native_ui_readiness_smoke=pass mode={mode}");
        Ok(())
    }
}

fn run_native_render_ui_child_smoke() -> anyhow::Result<()> {
    let executable = std::env::current_exe()?;
    let status = Command::new(executable)
        .env_remove(NO_RENDER_ENV)
        .env_remove(RENDER_HOST_ENV)
        .env(RENDER_UI_HOST_ENV, "1")
        .status()?;
    anyhow::ensure!(
        status.success(),
        "native Render + UI Host child smoke exited with {status}"
    );
    Ok(())
}

#[derive(Clone)]
struct LifecycleProbe {
    startup_ran: Arc<AtomicBool>,
    update_ran: Arc<AtomicBool>,
}

impl runen_ecs::Resource for LifecycleProbe {}

fn observe_startup(probe: Res<LifecycleProbe>) {
    probe.startup_ran.store(true, Ordering::SeqCst);
}

fn observe_update(probe: Res<LifecycleProbe>) {
    probe.update_ran.store(true, Ordering::SeqCst);
}

fn observe_render_startup(
    gfx: Res<Gfx>,
    surfaces: Res<RenderSurfaceRegistryResource>,
    probe: Res<LifecycleProbe>,
) -> anyhow::Result<()> {
    let primary_surface = surfaces
        .surface_for_native_window(NativeWindowId::primary())
        .ok_or_else(|| anyhow::anyhow!("selected Render did not correlate the primary window"))?;
    anyhow::ensure!(
        primary_surface == RenderSurfaceId::primary(),
        "selected Render correlated the primary window to the wrong surface"
    );
    anyhow::ensure!(
        surfaces
            .record(primary_surface)
            .map(|record| record.lifecycle_state)
            == Some(RenderSurfaceLifecycleState::Attached),
        "selected Render primary surface was not Attached before Startup"
    );
    anyhow::ensure!(
        gfx.has_surface(primary_surface),
        "selected Render Gfx did not own the primary surface before Startup"
    );
    probe.startup_ran.store(true, Ordering::SeqCst);
    Ok(())
}

struct NativeNoRenderSmokeHook {
    frame_seen: Arc<AtomicBool>,
    secondary_created: Arc<AtomicBool>,
    unexpected_render_state: Arc<AtomicBool>,
    secondary_requested: bool,
}

impl NativeWindowHook for NativeNoRenderSmokeHook {
    fn name(&self) -> &'static str {
        "native_no_render_smoke"
    }

    fn attach(&mut self, _window: &Window, world: &mut runen_ecs::World) -> anyhow::Result<()> {
        self.observe_render_absence(world);
        if world
            .resource::<WindowStateRegistryResource>()?
            .records()
            .any(|record| {
                record.native_window_id != NativeWindowId::primary()
                    && record.lifecycle_state == NativeWindowLifecycleState::Created
            })
        {
            self.secondary_created.store(true, Ordering::SeqCst);
        }
        Ok(())
    }

    fn frame(&mut self, _window: &Window, world: &mut runen_ecs::World) -> anyhow::Result<()> {
        self.observe_render_absence(world);
        self.frame_seen.store(true, Ordering::SeqCst);
        let windows = world.resource_mut::<WindowStateRegistryResource>()?;
        if !self.secondary_requested {
            windows.request_window("Native no-Render secondary", (640, 480));
            self.secondary_requested = true;
        }
        windows
            .record_mut(NativeWindowId::primary())
            .ok_or_else(|| anyhow::anyhow!("native no-Render smoke primary window is missing"))?
            .request_close();
        Ok(())
    }
}

impl NativeNoRenderSmokeHook {
    fn observe_render_absence(&self, world: &runen_ecs::World) {
        if world.has_resource::<Gfx>() || world.has_resource::<RenderSurfaceRegistryResource>() {
            self.unexpected_render_state.store(true, Ordering::SeqCst);
        }
    }
}

struct NativeRenderHostSmokeHook {
    frame_seen: Arc<AtomicBool>,
    frame_submitted: Arc<AtomicBool>,
    primary_attached: Arc<AtomicBool>,
    secondary_attached: Arc<AtomicBool>,
    secondary_requested: bool,
    frames_seen: usize,
    counter_slot: Option<UiRuntimeSlotId>,
    counter_displayed: Arc<AtomicBool>,
}

impl NativeWindowHook for NativeRenderHostSmokeHook {
    fn name(&self) -> &'static str {
        "native_render_host_smoke"
    }

    fn attach(&mut self, _window: &Window, world: &mut runen_ecs::World) -> anyhow::Result<()> {
        match verify_primary_render_attachment(world) {
            Ok(()) => self.primary_attached.store(true, Ordering::SeqCst),
            Err(error) => {
                eprintln!("native-render-host-smoke: primary attachment check failed: {error:#}")
            }
        }

        let secondary_window_id = world
            .resource::<WindowStateRegistryResource>()?
            .records()
            .find(|record| {
                record.native_window_id != NativeWindowId::primary()
                    && record.lifecycle_state == NativeWindowLifecycleState::Created
            })
            .map(|record| record.native_window_id);
        let Some(secondary_window_id) = secondary_window_id else {
            return Ok(());
        };

        match verify_secondary_render_attachment(world, secondary_window_id) {
            Ok(()) => self.secondary_attached.store(true, Ordering::SeqCst),
            Err(error) => {
                eprintln!("native-render-host-smoke: secondary attachment check failed: {error:#}")
            }
        }
        Ok(())
    }

    fn frame(&mut self, _window: &Window, world: &mut runen_ecs::World) -> anyhow::Result<()> {
        match verify_primary_render_attachment(world) {
            Ok(()) => self.primary_attached.store(true, Ordering::SeqCst),
            Err(error) => {
                eprintln!("native-render-host-smoke: primary frame check failed: {error:#}")
            }
        }
        self.frame_seen.store(true, Ordering::SeqCst);
        self.frames_seen += 1;
        if let Some(slot) = self.counter_slot {
            let mapping = world
                .resource::<WindowStateRegistryResource>()?
                .record(NativeWindowId::primary())
                .and_then(|native| {
                    UiRuntimeNativeMapping::new(
                        NativeWindowId::primary(),
                        RenderSurfaceId::primary(),
                        native.size_px,
                        native.scale_factor,
                    )
                    .ok()
                });
            if mapping.is_some_and(|mapping| {
                world
                    .resource::<UiRuntimePresentationAssociationsResource>()
                    .ok()
                    .and_then(|associations| associations.displayed_for_mapping(slot, mapping))
                    .is_some()
            }) {
                self.counter_displayed.store(true, Ordering::SeqCst);
            }
        }
        if world
            .resource::<engine::DebugMetricsState>()
            .ok()
            .and_then(|metrics| metrics.last_timings)
            .is_some_and(|timings| timings.submitted)
        {
            self.frame_submitted.store(true, Ordering::SeqCst);
        }

        let should_close = self.frame_submitted.load(Ordering::SeqCst)
            && self.secondary_attached.load(Ordering::SeqCst)
            && (self.counter_slot.is_none() || self.counter_displayed.load(Ordering::SeqCst));
        let windows = world.resource_mut::<WindowStateRegistryResource>()?;
        if !self.secondary_requested {
            windows.request_window("Native selected-Render secondary", (640, 480));
            self.secondary_requested = true;
        }
        let primary = windows
            .record_mut(NativeWindowId::primary())
            .ok_or_else(|| anyhow::anyhow!("selected-Render smoke primary window is missing"))?;
        let safety_frame_limit = if self.counter_slot.is_some() { 80 } else { 16 };
        if should_close || self.frames_seen >= safety_frame_limit {
            primary.request_close();
        } else {
            primary.request_redraw();
        }
        Ok(())
    }
}

fn verify_primary_render_attachment(world: &runen_ecs::World) -> anyhow::Result<()> {
    let surfaces = world.resource::<RenderSurfaceRegistryResource>()?;
    let primary_surface = surfaces
        .surface_for_native_window(NativeWindowId::primary())
        .ok_or_else(|| anyhow::anyhow!("selected Render primary surface correlation is missing"))?;
    anyhow::ensure!(
        primary_surface == RenderSurfaceId::primary(),
        "selected Render primary native window must correlate to the primary surface"
    );
    anyhow::ensure!(
        surfaces
            .record(primary_surface)
            .map(|record| record.lifecycle_state)
            == Some(RenderSurfaceLifecycleState::Attached),
        "selected Render primary surface must be Attached"
    );
    anyhow::ensure!(
        world.resource::<Gfx>()?.has_surface(primary_surface),
        "selected Render Gfx must own the primary surface"
    );
    Ok(())
}

fn verify_secondary_render_attachment(
    world: &runen_ecs::World,
    native_window_id: NativeWindowId,
) -> anyhow::Result<()> {
    let surfaces = world.resource::<RenderSurfaceRegistryResource>()?;
    let render_surface_id = surfaces
        .surface_for_native_window(native_window_id)
        .ok_or_else(|| {
            anyhow::anyhow!("selected Render secondary surface correlation is missing")
        })?;
    anyhow::ensure!(
        render_surface_id != RenderSurfaceId::primary(),
        "selected Render secondary window must use a distinct surface"
    );
    anyhow::ensure!(
        surfaces
            .record(render_surface_id)
            .map(|record| record.lifecycle_state)
            == Some(RenderSurfaceLifecycleState::Attached),
        "selected Render secondary surface must be Attached"
    );
    anyhow::ensure!(
        world.resource::<Gfx>()?.has_surface(render_surface_id),
        "selected Render Gfx must own the secondary surface"
    );
    Ok(())
}

fn native_no_render_host_smoke() -> anyhow::Result<()> {
    let startup_ran = Arc::new(AtomicBool::new(false));
    let update_ran = Arc::new(AtomicBool::new(false));
    let frame_seen = Arc::new(AtomicBool::new(false));
    let secondary_created = Arc::new(AtomicBool::new(false));
    let unexpected_render_state = Arc::new(AtomicBool::new(false));

    let mut app = App::new();
    app.add_plugins(default_plugins());
    app.insert_resource(LifecycleProbe {
        startup_ran: Arc::clone(&startup_ran),
        update_ran: Arc::clone(&update_ran),
    });
    app.add_systems(Startup, observe_startup);
    app.add_systems(Update, observe_update);
    app.init_resource::<NativeWindowHookRegistryResource>();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(NativeNoRenderSmokeHook {
            frame_seen: Arc::clone(&frame_seen),
            secondary_created: Arc::clone(&secondary_created),
            unexpected_render_state: Arc::clone(&unexpected_render_state),
            secondary_requested: false,
        });

    anyhow::ensure!(
        app.world().resource::<Gfx>().is_err(),
        "no-Render composition must not contain Gfx before native Host realization"
    );
    anyhow::ensure!(
        app.world()
            .resource::<RenderSurfaceRegistryResource>()
            .is_err(),
        "no-Render composition must not contain a Render surface registry before native Host realization"
    );

    app.run()?;

    anyhow::ensure!(startup_ran.load(Ordering::SeqCst), "Startup did not run");
    anyhow::ensure!(update_ran.load(Ordering::SeqCst), "Update did not run");
    anyhow::ensure!(
        frame_seen.load(Ordering::SeqCst),
        "native frame hook did not run"
    );
    anyhow::ensure!(
        secondary_created.load(Ordering::SeqCst),
        "secondary native window did not reach Created without Render"
    );
    anyhow::ensure!(
        !unexpected_render_state.load(Ordering::SeqCst),
        "native Host materialized Gfx or Render surface state without RenderPlugin"
    );

    println!("native_no_render_host_smoke=pass");
    Ok(())
}

fn native_render_host_smoke(with_ui: bool) -> anyhow::Result<()> {
    let startup_ran = Arc::new(AtomicBool::new(false));
    let update_ran = Arc::new(AtomicBool::new(false));
    let frame_seen = Arc::new(AtomicBool::new(false));
    let frame_submitted = Arc::new(AtomicBool::new(false));
    let primary_attached = Arc::new(AtomicBool::new(false));
    let secondary_attached = Arc::new(AtomicBool::new(false));
    let counter_displayed = Arc::new(AtomicBool::new(false));

    let mut app = App::new();
    app.add_plugins(default_plugins());
    if with_ui {
        app.add_plugin(UiPlugin);
    }
    app.add_plugin(RenderPlugin);
    let counter_slot = if with_ui {
        let slot = app
            .ui()
            .mount_with_fonts(NativeCounterScreen, &native_counter_fonts())
            .slot_id()
            .ok_or_else(|| anyhow::anyhow!("native U5 Counter failed to mount"))?;
        app.world_mut()
            .resource_mut::<UiRuntimePresentationAssociationsResource>()?
            .bind(
                UiRuntimePresentationBinding::new(
                    slot,
                    RenderFrameProducerId::try_from_raw(71).expect("test producer is nonzero"),
                    RenderSurfaceId::primary(),
                    StyleEnvironment::default(),
                )
                .with_order(SurfaceFrameSubmissionOrder::new(20, 0)),
            )?;
        Some(slot)
    } else {
        None
    };
    let flow = RenderFlow::new("native.render.host.smoke")
        .with_surface_color()?
        .fullscreen_pass("native.render.host.smoke.clear")
        .main_surface_only()
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_surface_color()?
        .finish();
    let flow = if with_ui {
        flow.builtin_ui_composite_pass("native.render.host.smoke.runenui")?
            .main_surface_only()
            .finish()
    } else {
        flow
    };
    let flow = flow
        .present_pass("native.render.host.smoke.present")?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()?;
    app.add_render_flow(flow);
    app.insert_resource(LifecycleProbe {
        startup_ran: Arc::clone(&startup_ran),
        update_ran: Arc::clone(&update_ran),
    });
    app.add_systems(Startup, observe_render_startup);
    app.add_systems(Update, observe_update);
    app.init_resource::<NativeWindowHookRegistryResource>();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(NativeRenderHostSmokeHook {
            frame_seen: Arc::clone(&frame_seen),
            frame_submitted: Arc::clone(&frame_submitted),
            primary_attached: Arc::clone(&primary_attached),
            secondary_attached: Arc::clone(&secondary_attached),
            secondary_requested: false,
            frames_seen: 0,
            counter_slot,
            counter_displayed: Arc::clone(&counter_displayed),
        });

    anyhow::ensure!(
        app.world().resource::<Gfx>().is_err(),
        "selected Render must not create Gfx before native Host realization"
    );
    anyhow::ensure!(
        app.world()
            .resource::<RenderSurfaceRegistryResource>()
            .is_ok(),
        "RenderPlugin must install Render surface state before native Host realization"
    );
    app.run()?;

    anyhow::ensure!(startup_ran.load(Ordering::SeqCst), "Startup did not run");
    anyhow::ensure!(update_ran.load(Ordering::SeqCst), "Update did not run");
    anyhow::ensure!(
        frame_seen.load(Ordering::SeqCst),
        "selected-Render native frame hook did not run"
    );
    anyhow::ensure!(
        primary_attached.load(Ordering::SeqCst),
        "selected Render primary attachment was not observed"
    );
    anyhow::ensure!(
        secondary_attached.load(Ordering::SeqCst),
        "selected Render secondary attachment was not observed"
    );
    anyhow::ensure!(
        frame_submitted.load(Ordering::SeqCst),
        if with_ui {
            "Render + UiPlugin without ScenePlugin never submitted a native frame"
        } else {
            "Render without ScenePlugin never submitted a native frame"
        }
    );

    if with_ui {
        anyhow::ensure!(
            counter_displayed.load(Ordering::SeqCst),
            "native RunenUI Counter paint never completed F2 GPU work and terminal Present"
        );
    }
    println!(
        "native_render_host_smoke=pass ui_plugin={}",
        if with_ui { "selected" } else { "absent" }
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, runen_ecs::Resource)]
struct NativeHistoricalSource {
    latest: [u32; 4],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct NativeHistoricalUniform {
    meta: [u32; 4],
}

#[derive(Debug, Clone, Copy, GpuStorage)]
struct NativeHistoricalCounter {
    value: u32,
}

struct NativeHistoricalGpuHook {
    flow: RenderFlowId,
    frames: u32,
    first_submission: Option<runen_gpu::GpuSubmissionId>,
    saw_terminal: bool,
    saw_gpu_frame: bool,
}

impl NativeWindowHook for NativeHistoricalGpuHook {
    fn name(&self) -> &'static str {
        "native_historical_gpu_render_flow"
    }

    fn frame(&mut self, window: &Window, world: &mut runen_ecs::World) -> anyhow::Result<()> {
        self.frames += 1;
        if let Ok(streams) = world.resource::<RenderGpuFlowStepStreams>() {
            let producer = streams
                .stream(self.flow)
                .ok_or_else(|| anyhow::anyhow!("historical GPU producer disappeared"))?;
            anyhow::ensure!(
                producer.requested_tick() == 3 && producer.completed_tick() == 0,
                "native GPU execution must not invent verified semantic steps"
            );
            anyhow::ensure!(
                producer.outstanding() == 3,
                "native GPU step history was lost or duplicated"
            );
            if let Some(gpu) = producer.accepted_submission() {
                let gpu_id = gpu.id();
                if let Some(earlier) = self.first_submission {
                    anyhow::ensure!(
                        earlier == gpu_id,
                        "a deferred GPU producer was submitted more than once"
                    );
                } else {
                    self.first_submission = Some(gpu_id);
                }
                anyhow::ensure!(
                    producer.accepted_steps().iter().map(|step| step.tick()).collect::<Vec<_>>()
                        == vec![1, 2, 3],
                    "accepted native RenderFlow did not retain its precise historical interval"
                );
                match gpu.status() {
                    GpuSubmissionStatus::Accepted => {}
                    GpuSubmissionStatus::Completed => self.saw_terminal = true,
                    GpuSubmissionStatus::Failed(failure) => {
                        anyhow::bail!("native RenderFlow G5 execution failed: {failure:?}");
                    }
                }
            }
        }
        if let Ok(gfx) = world.resource::<Gfx>() {
            for receipt in gfx.pending_gpu_frame_submissions() {
                if receipt
                    .invocations()
                    .iter()
                    .any(|invocation| invocation.flow_id() == self.flow)
                {
                    let source_id = self.first_submission.ok_or_else(|| {
                        anyhow::anyhow!("G5 frame receipt exists without source step acceptance")
                    })?;
                    anyhow::ensure!(
                        receipt.submission().id() == source_id,
                        "composed RenderFlow receipt did not identify the exact producer work"
                    );
                    self.saw_gpu_frame = true;
                }
            }
        }

        if self.frames >= 40 {
            anyhow::ensure!(
                self.first_submission.is_some() && self.saw_terminal && self.saw_gpu_frame,
                "native historical RenderFlow must preserve exact renderer/G5 acceptance and terminal work"
            );
            // The renderer may already have retired its observation receipt at
            // terminal; source-owned G5 handles remain the execution truth.
            window.request_close();
        } else {
            window.request_redraw();
        }
        Ok(())
    }
}

fn native_historical_gpu_smoke() -> anyhow::Result<()> {
    let mut app = App::new();
    app.set_title("Native historical RenderFlow GPU continuity");
    app.add_plugins(default_plugins());
    app.add_plugin(RenderPlugin);

    let (flow, uniform) = RenderFlow::new("native.historical.gpu")
        .with_state::<NativeHistoricalSource>()
        .with_surface_color()?
        .uniform_buffer::<NativeHistoricalUniform>("native.historical.uniform")?;
    let (flow, counter) = flow
        .storage_array::<NativeHistoricalCounter>("native.historical.counter", 1)?;
    let flow = flow
        .compute_pass("native.historical.step")
        .shader_asset("assets/shaders/native_historical_gpu_step_smoke.wgsl")
        .uniform_from_state_to(
            GpuBindingKey::try_new(0, 0)?,
            uniform.clone(),
            |source: &NativeHistoricalSource| NativeHistoricalUniform {
                meta: source.latest,
            },
        )
        .bind_storage(GpuBindingKey::try_new(0, 1)?, counter)
        .dispatch([1, 1, 1])
        .finish()
        .fixed_step_region(
            "native.historical.region",
            4,
            [("native.historical.step", GpuBindingKey::try_new(0, 2)?)],
        )?
        .fullscreen_pass("native.historical.surface")
        .main_surface_only()
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_surface_color()?
        .finish()
        .present_pass("native.historical.present")?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()?;
    let flow_id = flow.id();
    let mut sources = RenderGpuFlowStepStreams::default();
    sources.register(flow_id, 0, 4)?;
    let input = |value: u32| RenderGpuFixedStepInput {
        uniforms: BTreeMap::from([(
            uniform.diagnostic_identity(),
            engine::plugins::render::bytemuck::bytes_of(
                &NativeHistoricalUniform { meta: [value, 0, 0, 0] }.to_gpu(),
            )
            .to_vec(),
        )]),
        iteration: RenderFixedStepIterationUniform::new(
            0, 1, 4, value as u64, 1.0 / (value as f32), 0.0,
        ),
    };
    sources.stream_mut(flow_id).unwrap().capture_batch(vec![
        input(1),
        input(2),
        input(3),
    ]).map_err(|(reason, _)| anyhow::anyhow!("producer refused input history: {reason}"))?;
    app.insert_resource(NativeHistoricalSource {
        latest: [999, 0, 0, 0],
    });
    app.insert_resource(sources);
    app.add_render_flow(flow);
    app.init_resource::<NativeWindowHookRegistryResource>();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(NativeHistoricalGpuHook {
            flow: flow_id,
            frames: 0,
            first_submission: None,
            saw_terminal: false,
            saw_gpu_frame: false,
        });
    app.run()?;
    println!("native_historical_gpu_render_flow=pass");
    Ok(())
}
