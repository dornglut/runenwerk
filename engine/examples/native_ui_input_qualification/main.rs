//! Opt-in owner-operated native UI/game input observation.
//!
//! This is a real Winit window with an actual mounted RunenUI application and
//! Render Present path. It does not inject events, fabricate M13 receipts, or
//! claim a physical display scanout merely because Present completed.
//!
//! Run with: cargo run -p engine --example native_ui_input_qualification --locked -- --interactive
//! Close with the operating system's window close button.
mod lifetime;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use engine::app::AppNativeHostExt;
use engine::plugins::input::domain::action;
use engine::plugins::render::host::RenderSurfaceId;
use engine::plugins::render::{
    RenderFlow, RenderFlowId, RenderFrameProducerId, RunenUiPaintSubmissionRegistryResource,
    SurfaceFrameSubmissionOrder,
};
use engine::plugins::ui::{
    AppUiExt, UiNativeInputRecipientsResource, UiRuntimeFontConfiguration,
    UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding, UiRuntimeSlotId,
    UiRuntimeSlotsResource, UiTypedScreenId, transact_ui_slot,
};
use engine::plugins::{ActionState, RenderPlugin, UiPlugin, default_plugins};
use engine::prelude::{App, AppRenderExt};
use engine::runtime::{
    FramePacingPolicyResource, NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId,
    WindowStateRegistryResource,
};
use runen_ecs::World;
use runenui_core::{
    Effects, FontFamilyName, GenericFontFamily, NoHostProtocol, StyleEnvironment, UiApp, View,
    button, column, text,
};
use winit::event::{DeviceEvent, Ime, WindowEvent};
use winit::window::Window;

const FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/fonts/JetBrainsMono-Regular.ttf"
));

struct QualificationUi;

struct UiState {
    activations: Arc<AtomicUsize>,
    text_generation: u64,
}

enum QualificationAction {
    Activate,
    ChangeText(u64),
}

impl UiApp for QualificationUi {
    type State = UiState;
    type Action = QualificationAction;
    type HostProtocol = NoHostProtocol;

    fn root(state: &UiState) -> impl View<QualificationAction> {
        column(vec![
            text(if state.text_generation == 0 {
                "RunenUI native input qualification".to_owned()
            } else {
                format!("RunenUI changing text {:08}", state.text_generation)
            })
            .into_element(),
            text("Click the button to focus it, then try W, Space and Tab.").into_element(),
            button("Focus / activate this button")
                .id("u9c.qualify.button")
                .key("u9c.qualify.button")
                .on_activate(|| QualificationAction::Activate)
                .into_element(),
        ])
        .id("u9c.qualify.root")
    }

    fn update(
        state: &mut Self::State,
        action: Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        match action {
            QualificationAction::Activate => {
                state.activations.fetch_add(1, Ordering::Relaxed);
            }
            QualificationAction::ChangeText(generation) => state.text_generation = generation,
        }
        Effects::none()
    }
}

struct QualificationHook {
    slot: UiRuntimeSlotId,
    activations: Arc<AtomicUsize>,
    frames: u64,
    keyboard_events: u64,
    pointer_buttons: u64,
    cursor_moves: u64,
    touch_events: u64,
    ime_events: u64,
    raw_motion: u64,
    last_observation: Option<String>,
    lifetime: Option<lifetime::Probe>,
    flow: RenderFlowId,
}

impl NativeWindowHook for QualificationHook {
    fn name(&self) -> &'static str {
        "native_ui_input_qualification"
    }

    fn attach(&mut self, window: &Window, _: &mut World) -> Result<()> {
        window.set_visible(true);
        println!("U9C_ATTACH window={:?} visible=true", window.id());
        Ok(())
    }

    fn window_event(&mut self, window: &Window, event: &WindowEvent, _: &mut World) -> Result<()> {
        // This callback observes the *original Winit event before Host routing*.
        // Post-routing facts are observed separately through owner-owned
        // ActionState, RunenUI input_ownership and completed-Present association.
        match event {
            WindowEvent::Focused(focused) => {
                println!("U9C_OS window={:?} focus={focused}", window.id());
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                self.keyboard_events += 1;
                println!(
                    "U9C_OS window={:?} kind=keyboard device={device_id:?} key={:?} phase={:?} repeat={} synthetic={} text_chars={}",
                    window.id(),
                    event.physical_key,
                    event.state,
                    event.repeat,
                    is_synthetic,
                    event.text.as_ref().map_or(0, |text| text.chars().count()),
                );
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                self.pointer_buttons += 1;
                println!(
                    "U9C_OS window={:?} kind=mouse_button device={device_id:?} button={button:?} state={state:?}",
                    window.id(),
                );
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                self.cursor_moves += 1;
                if self.cursor_moves == 1 || self.cursor_moves.is_multiple_of(32) {
                    println!(
                        "U9C_OS window={:?} kind=cursor device={device_id:?} position=({:.1},{:.1}) observed_moves={}",
                        window.id(),
                        position.x,
                        position.y,
                        self.cursor_moves,
                    );
                }
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                ..
            } => {
                println!(
                    "U9C_OS window={:?} kind=wheel device={device_id:?} delta={delta:?} phase={phase:?}",
                    window.id(),
                );
            }
            WindowEvent::Touch(touch) => {
                self.touch_events += 1;
                println!(
                    "U9C_OS window={:?} kind=touch device={:?} id={} phase={:?}",
                    window.id(),
                    touch.device_id,
                    touch.id,
                    touch.phase,
                );
            }
            WindowEvent::Ime(Ime::Preedit(text, range)) => {
                self.ime_events += 1;
                println!(
                    "U9C_OS window={:?} kind=ime_preedit chars={} range={range:?}",
                    window.id(),
                    text.chars().count(),
                );
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                self.ime_events += 1;
                println!(
                    "U9C_OS window={:?} kind=ime_commit chars={}",
                    window.id(),
                    text.chars().count(),
                );
            }
            WindowEvent::Ime(Ime::Enabled | Ime::Disabled) => {
                println!(
                    "U9C_OS window={:?} kind=ime_lifecycle event={event:?}",
                    window.id()
                );
            }
            WindowEvent::CloseRequested => {
                println!("U9C_OS window={:?} close_intent=true", window.id());
            }
            _ => {}
        }
        Ok(())
    }

    fn device_event(&mut self, event: &DeviceEvent, _: &mut World) -> Result<()> {
        match event {
            DeviceEvent::MouseMotion { .. } => {
                self.raw_motion += 1;
                if self.raw_motion == 1 || self.raw_motion.is_multiple_of(64) {
                    println!("U9C_OS kind=raw_device_motion observed={}", self.raw_motion);
                }
            }
            DeviceEvent::Added | DeviceEvent::Removed => {
                println!("U9C_OS kind=device_lifecycle event={event:?}");
            }
            _ => {}
        }
        Ok(())
    }

    fn frame(&mut self, _: &Window, world: &mut World) -> Result<()> {
        let result = self.observe_frame(world);
        if let Err(error) = &result
            && let Some(probe) = &self.lifetime
        {
            // Host hooks retain errors as diagnostics. Make every failure, including
            // the observation tail and final close, invalidate this example's result.
            probe
                .report
                .lock()
                .map_err(|_| anyhow::anyhow!("report poisoned"))?
                .record_failure(format!("{error:#}"));
            if let Ok(registry) = world.resource_mut::<WindowStateRegistryResource>()
                && let Some(record) = registry.record_mut(NativeWindowId::primary())
            {
                record.request_close();
                record.approve_close();
            }
        }
        result
    }

    fn detach(&mut self, _: &mut World) -> Result<()> {
        println!(
            "U9C_DETACH frames={} keyboard={} mouse_buttons={} cursor={} touch={} ime={} raw_motion={}",
            self.frames,
            self.keyboard_events,
            self.pointer_buttons,
            self.cursor_moves,
            self.touch_events,
            self.ime_events,
            self.raw_motion,
        );
        Ok(())
    }
}

fn main() -> Result<()> {
    if std::env::var_os("RUST_LOG").is_some() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()?)
            .with_ansi(false)
            .with_writer(std::io::stderr)
            .try_init()
            .map_err(|error| anyhow::anyhow!("native qualification tracing failed: {error}"))?;
    }
    let options = lifetime::parse(&std::env::args().skip(1).collect::<Vec<_>>())?;
    let report = Arc::new(Mutex::new(lifetime::Report::default()));

    let clicks = Arc::new(AtomicUsize::new(0));
    let fonts = UiRuntimeFontConfiguration::new(vec![FONT.to_vec()]).with_generic_mapping(
        GenericFontFamily::SansSerif,
        vec![FontFamilyName::new("JetBrains Mono").expect("bundled font family")],
    );

    let mut app = App::new();
    app.set_title("U9-C Native UI qualification");
    app.with_frame_pacing(if options.is_some() {
        FramePacingPolicyResource::continuous_capped(30)
    } else {
        FramePacingPolicyResource::on_demand()
    });
    app.add_plugins(default_plugins());
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);

    let slot = app
        .ui()
        .mount_app::<QualificationUi>(
            UiTypedScreenId::new("u9c.native.input.qualification"),
            UiState {
                activations: Arc::clone(&clicks),
                text_generation: 0,
            },
            &fonts,
        )
        .slot_id()
        .context("qualifier RunenUI application did not mount")?;
    app.ui().bind_native_primary_input(slot)?;

    if options.is_none_or(|options| options.content != lifetime::Content::Control) {
        app.world_mut()
            .resource_mut::<UiRuntimePresentationAssociationsResource>()?
            .bind(
                UiRuntimePresentationBinding::new(
                    slot,
                    RenderFrameProducerId::try_from_raw(931).expect("nonzero producer"),
                    RenderSurfaceId::primary(),
                    StyleEnvironment::default(),
                )
                .with_order(SurfaceFrameSubmissionOrder::new(20, 0)),
            )?;
    }

    let flow = qualification_flow(
        options.is_none_or(|options| options.content != lifetime::Content::Control),
    )?;
    let flow_id = flow.id();
    app.add_render_flow(flow);

    app.init_resource::<NativeWindowHookRegistryResource>();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(QualificationHook {
            slot,
            activations: clicks,
            frames: 0,
            keyboard_events: 0,
            pointer_buttons: 0,
            cursor_moves: 0,
            touch_events: 0,
            ime_events: 0,
            raw_motion: 0,
            last_observation: None,
            lifetime: options.map(|options| lifetime::Probe::new(options, Arc::clone(&report))),
            flow: flow_id,
        });

    println!(
        "U9C_START options={options:?} pacing={} input=real_os mounted_ui=true render=true pid={} source_claim={:?}",
        if options.is_some() {
            "continuous_30"
        } else {
            "on_demand"
        },
        std::process::id(),
        std::env::var("RUNENWERK_SOURCE_REVISION").ok()
    );
    println!("U9C_GUIDANCE click UI button, try W/Space/Tab, Alt+Tab and refocus; close via X");
    app.run()?;
    if options.is_some() {
        let report = report
            .lock()
            .map_err(|_| anyhow::anyhow!("lifetime report poisoned"))?;
        report.ensure_complete()?;
    }
    println!("U9C_EXIT result=ok");
    Ok(())
}

fn qualification_flow(with_ui: bool) -> Result<RenderFlow> {
    let flow = RenderFlow::new("native.u9c.qualifier")
        .with_surface_color()?
        .fullscreen_pass("native.u9c.qualifier.clear")
        .main_surface_only()
        .clear_color([0.07, 0.08, 0.10, 1.0])
        .write_surface_color()?
        .finish();
    let flow = if with_ui {
        flow.builtin_ui_composite_pass("native.u9c.qualifier.ui")?
            .main_surface_only()
            .finish()
    } else {
        flow
    };
    let flow = flow
        .present_pass("native.u9c.qualifier.present")?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()?;

    Ok(flow)
}

impl QualificationHook {
    fn observe_frame(&mut self, world: &mut World) -> Result<()> {
        self.frames += 1;
        if let Some(probe) = self.lifetime.as_mut() {
            let stop = probe.observe(world, &mut self.flow, self.frames, self.slot)?;
            let os_close_pending = world
                .resource::<WindowStateRegistryResource>()?
                .record(NativeWindowId::primary())
                .context("primary native window missing after lifetime observation")?
                .close_intent_pending;
            if !stop
                && !os_close_pending
                && let Some(generation) = probe.next_text_change()
            {
                transact_ui_slot(world, self.slot, |slots| {
                    slots.submit_app_action::<QualificationUi>(
                        self.slot,
                        QualificationAction::ChangeText(generation),
                    )
                })?;
            }
            if stop {
                let record = world
                    .resource_mut::<WindowStateRegistryResource>()?
                    .record_mut(NativeWindowId::primary())
                    .context("primary missing at lifetime stop")?;
                record.request_close();
                record.approve_close();
            }
        }
        let (window_focused, close_intent) = {
            let record = world
                .resource::<WindowStateRegistryResource>()?
                .record(NativeWindowId::primary())
                .context("primary native window missing")?;
            (record.focused, record.close_intent_pending)
        };
        let selected = world
            .resource::<UiNativeInputRecipientsResource>()?
            .binding_epoch_for_window(NativeWindowId::primary());
        let displayed = world
            .resource::<UiRuntimePresentationAssociationsResource>()?
            .displayed_for_native_recipient(self.slot, NativeWindowId::primary())
            .is_some();
        // Renderer intake is an immutable paint candidate, not proof of a
        // submitted/completed frame or an input permission. Observe it separately
        // from the one terminal completed-Present association above.
        let paint_intake = world
            .resource::<UiRuntimePresentationAssociationsResource>()?
            .binding(self.slot)
            .and_then(|binding| {
                world
                    .resource::<RunenUiPaintSubmissionRegistryResource>()
                    .ok()?
                    .get_for_surface(&binding.producer_id, binding.render_surface_id)
                    .map(|paint| paint.publication_id.raw())
            });
        let owner = match world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .input_ownership(self.slot)
        {
            Ok(ownership) => format!(
                "status={:?} revision={} focus={:?} composition={} pointers={} capture_count={} space_owner={}",
                ownership.status(),
                ownership.revision().get(),
                ownership.keyboard().focused_node(),
                ownership.keyboard().composition_generation().is_some(),
                ownership.pointers().len(),
                ownership
                    .pointers()
                    .iter()
                    .filter(|pointer| pointer.capture_owner().is_some())
                    .count(),
                ownership.keyboard().space_activation_owner().is_some(),
            ),
            Err(error) => format!("unavailable={error:?}"),
        };
        let actions = world.resource::<ActionState>()?;
        let w = actions.action_down(action::WORLD_MOVE_UP);
        let a = actions.action_down(action::WORLD_MOVE_LEFT);
        let s = actions.action_down(action::WORLD_MOVE_DOWN);
        let d = actions.action_down(action::WORLD_MOVE_RIGHT);
        let activations = self.activations.load(Ordering::Relaxed);
        let observation = format!(
            "frame={} native_focused={} selected_epoch={selected:?} paint_intake={paint_intake:?} completed_present={} M13=[{owner}] game_down=[W:{w},A:{a},S:{s},D:{d}] ui_activations={activations}",
            self.frames, window_focused, displayed
        );
        // Only semantic changes are logged; frame counter stays in the output
        // but must not cause one trace line for every continuously paced frame.
        let stable = format!(
            "focused={window_focused} selected={selected:?} paint={paint_intake:?} present={displayed} {owner} game={w}{a}{s}{d} activations={activations}"
        );
        if self.last_observation.as_deref() != Some(stable.as_str()) {
            println!("U9C_FRAME {observation}");
            self.last_observation = Some(stable);
            world
                .resource_mut::<WindowStateRegistryResource>()?
                .record_mut(NativeWindowId::primary())
                .context("primary native window disappeared")?
                .set_title(format!(
                    "U9-C Native UI | present={displayed} | focus={window_focused} | game W={w} | UI clicks={activations}"
                ));
        }
        if close_intent {
            // Physical close remains an original Host event. This is only
            // the application's existing explicit approval policy.
            world
                .resource_mut::<WindowStateRegistryResource>()?
                .record_mut(NativeWindowId::primary())
                .context("primary native window disappeared at close")?
                .approve_close();
        }
        Ok(())
    }
}
