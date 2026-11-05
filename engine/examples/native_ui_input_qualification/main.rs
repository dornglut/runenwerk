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
    Color, EdgeInsets, EditIntent, EditResolution, Effects, FlexContainerStyle, FlexDirection,
    FontFamily, FontFamilyName, GenericFontFamily, ItemAlignment, LayoutBound, LayoutContainer,
    LayoutDimension, LayoutFactor, LayoutStyle, LogicalLength, MainAxisAlignment, NoHostProtocol,
    SemanticEditableMode, StyleEnvironment, TextAffinity, TextDocumentId, TextDocumentRevision,
    TextDocumentSnapshot, TextPosition, TextSelection, Typography, UiApp, UpdateOutput, View,
    button, column, text, text_field,
};
use winit::event::{DeviceEvent, Ime, WindowEvent};
use winit::window::Window;

const FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/fonts/JetBrainsMono-Regular.ttf"
));

struct QualificationUi;

fn qualifier_font(size: u16) -> Typography {
    Typography::new(
        FontFamily::generic(GenericFontFamily::SansSerif),
        LogicalLength::from(size),
    )
}

fn qualifier_screen_layout() -> LayoutStyle {
    LayoutStyle::default()
        .with_width(LayoutDimension::Fill)
        .with_height(LayoutDimension::Fill)
        .with_container(LayoutContainer::Flex(
            FlexContainerStyle::default()
                .with_direction(FlexDirection::Column)
                .with_justify_content(MainAxisAlignment::Center)
                .with_align_items(ItemAlignment::Center),
        ))
}

fn qualifier_panel_layout() -> LayoutStyle {
    LayoutStyle::default()
        .with_width(LayoutDimension::percent(
            LayoutFactor::new(0.9).expect("finite panel width"),
        ))
        .with_max_width(LayoutBound::length(LogicalLength::from(640_u16)))
        .with_container(LayoutContainer::Flex(
            FlexContainerStyle::default()
                .with_direction(FlexDirection::Column)
                .with_align_items(ItemAlignment::Stretch),
        ))
        .with_gap(LogicalLength::from(16_u16))
}

struct UiState {
    activations: Arc<AtomicUsize>,
    text_generation: u64,
    // Application-owned editable state: original native text/IME must actually
    // settle through RunenUI before this value or its revision may change.
    entered_text: String,
    edit_revision: u64,
    activation_count: u64,
}

enum QualificationAction {
    Activate,
    ChangeText(u64),
    Edit(Box<EditIntent>),
}

impl UiApp for QualificationUi {
    fn initial_producer_budget(_state: &Self::State) -> runenui_core::ProducerBudget {
        // Eight stable owners: screen, centered panel, title, two instructions,
        // editable, actual button and application-derived result.
        runenui_core::ProducerBudget::new(9, 0).with_reconciliation(8, 0, 8)
    }

    fn action_producer_budget(
        _state: &Self::State,
        _action: &Self::Action,
    ) -> runenui_core::ProducerBudget {
        // The same eight owners reconcile after accepted edits/activations.
        runenui_core::ProducerBudget::new(10, 1).with_reconciliation(8, 8, 8)
    }

    fn interaction_producer_budget(_state: &Self::State) -> runenui_core::ProducerBudget {
        // Preserve the pinned M11 text-field service frontier, including
        // clipboard/IME terminal work; noninteractive layout owners reconcile.
        runenui_core::ProducerBudget::new(28, 13)
            .with_reconciliation(8, 8, 8)
            .with_work_starts(runenui_core::WorkStartBudget::new(0, 0, 0, 0, 2))
            .with_work_residency(runenui_core::WorkResidencyBudget::new(0, 0, 0, 0, 0, 2))
    }

    type State = UiState;
    type Action = QualificationAction;
    type HostProtocol = NoHostProtocol;

    fn root(state: &UiState) -> impl View<QualificationAction> {
        let document = TextDocumentSnapshot::new(
            TextDocumentId::new(1276),
            TextDocumentRevision::new(state.edit_revision),
        );
        let caret = TextPosition::new(
            document,
            &state.entered_text,
            state.entered_text.len(),
            TextAffinity::Downstream,
        )
        .expect("the authored UTF-8 text ends at a valid scalar boundary");
        let editable = text_field(
            document,
            state.entered_text.clone(),
            TextSelection::collapsed(caret),
            SemanticEditableMode::SingleLine,
            |intent| QualificationAction::Edit(Box::new(intent)),
        )
        .expect("the native qualification text source is well formed")
        .placeholder("CLICK HERE - type Unicode text")
        .id("u9c.qualify.editable")
        .with_layout(
            LayoutStyle::default()
                .with_width(LayoutDimension::Fill)
                .with_min_height(LayoutBound::length(LogicalLength::from(52_u16))),
        )
        .foreground(Color::rgb(20, 29, 45))
        .background(Color::rgb(242, 246, 252))
        .padding(EdgeInsets::symmetric(
            LogicalLength::from(14_u16),
            LogicalLength::from(12_u16),
        ))
        .radius(LogicalLength::from(10_u16))
        .typography(qualifier_font(16));

        // Styling never substitutes for native Winit input or exact M13 focus.
        let panel = column(vec![
            text(if state.text_generation == 0 {
                "RUNENWERK  /  U9-C INPUT TEST".to_owned()
            } else {
                format!("RUNENWERK  /  UI CHANGE {:08}", state.text_generation)
            })
            .foreground(Color::rgb(244, 247, 253))
            .typography(qualifier_font(26))
            .into_element(),
            text("1   W halten, dann ins Textfeld klicken.")
                .foreground(Color::rgb(180, 192, 211))
                .typography(qualifier_font(14))
                .into_element(),
            text("2   Unicode tippen, Tab + Space und Button testen.")
                .foreground(Color::rgb(180, 192, 211))
                .typography(qualifier_font(14))
                .into_element(),
            editable.into_element(),
            button("BUTTON AKTIVIEREN")
                .id("u9c.qualify.button")
                .key("u9c.qualify.button")
                .on_activate(|| QualificationAction::Activate)
                .with_layout(
                    LayoutStyle::default()
                        .with_width(LayoutDimension::Fill)
                        .with_min_height(LayoutBound::length(LogicalLength::from(52_u16))),
                )
                .foreground(Color::WHITE)
                .background(Color::rgb(55, 111, 208))
                .padding(EdgeInsets::symmetric(
                    LogicalLength::from(16_u16),
                    LogicalLength::from(14_u16),
                ))
                .radius(LogicalLength::from(10_u16))
                .typography(qualifier_font(16))
                .into_element(),
            text(format!(
                "UI-Klicks: {}     Textzeichen: {}",
                state.activation_count,
                state.entered_text.chars().count(),
            ))
            .foreground(Color::rgb(147, 199, 255))
            .typography(qualifier_font(14))
            .into_element(),
        ])
        .id("u9c.qualify.panel")
        .with_layout(qualifier_panel_layout())
        .background(Color::rgb(24, 33, 49))
        .padding(EdgeInsets::all(LogicalLength::from(28_u16)))
        .radius(LogicalLength::from(18_u16));

        column(vec![panel.into_element()])
            .id("u9c.qualify.root")
            .with_layout(qualifier_screen_layout())
            .background(Color::rgb(13, 19, 30))
    }

    fn update(
        state: &mut Self::State,
        action: Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        match action {
            QualificationAction::Activate => {
                state.activations.fetch_add(1, Ordering::Relaxed);
                state.activation_count = state
                    .activation_count
                    .checked_add(1)
                    .expect("native activation count must not wrap");
                UpdateOutput::effects(Effects::none())
            }
            QualificationAction::ChangeText(generation) => {
                state.text_generation = generation;
                UpdateOutput::effects(Effects::none())
            }
            QualificationAction::Edit(intent) => {
                let range = intent.replacement();
                state
                    .entered_text
                    .replace_range(range.start()..range.end(), intent.replacement_text());
                state.edit_revision = state
                    .edit_revision
                    .checked_add(1)
                    .expect("the native typed text revision must not wrap");
                let document = TextDocumentSnapshot::new(
                    TextDocumentId::new(1276),
                    TextDocumentRevision::new(state.edit_revision),
                );
                UpdateOutput::edit(EditResolution::accepted(intent.request().clone(), document))
            }
        }
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

    fn detach(&mut self, world: &mut World) -> Result<()> {
        // Preserve the framework's original causal integrity trace on a native
        // debug-build failure. A terminal pump report alone cannot identify
        // the producer contract that failed; never manufacture a receipt.
        #[cfg(debug_assertions)]
        if let Ok(slots) = world.resource::<UiRuntimeSlotsResource>() {
            let tail = slots.debug_native_trace_tail::<QualificationUi>(self.slot, 96);
            if tail.iter().any(|line| {
                line.contains("RuntimeTerminal") || line.contains("RoutedIntegrityFailed")
            }) {
                for line in tail {
                    println!("U9C_M13_TRACE {line}");
                }
            }
        }
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
                entered_text: String::new(),
                edit_revision: 0,
                activation_count: 0,
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
    println!(
        "U9C_GUIDANCE click text field, type Unicode text, Tab to button, click and hold W; Alt+Tab, resize, refocus, then close via X"
    );
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
        let (edit_revision, edit_chars) = world
            .resource::<UiRuntimeSlotsResource>()?
            .observe_app::<QualificationUi, _>(self.slot, |runtime| {
                (
                    runtime.state().edit_revision,
                    runtime.state().entered_text.chars().count(),
                )
            })
            .map_err(|error| anyhow::anyhow!("native typed text state unavailable: {error:?}"))?;
        let actions = world.resource::<ActionState>()?;
        let w = actions.action_down(action::WORLD_MOVE_UP);
        let a = actions.action_down(action::WORLD_MOVE_LEFT);
        let s = actions.action_down(action::WORLD_MOVE_DOWN);
        let d = actions.action_down(action::WORLD_MOVE_RIGHT);
        let activations = self.activations.load(Ordering::Relaxed);
        let observation = format!(
            "frame={} native_focused={} selected_epoch={selected:?} paint_intake={paint_intake:?} completed_present={} M13=[{owner}] game_down=[W:{w},A:{a},S:{s},D:{d}] ui_activations={activations} typed_edit_revision={edit_revision} typed_chars={edit_chars}",
            self.frames, window_focused, displayed
        );
        // Only semantic changes are logged; frame counter stays in the output
        // but must not cause one trace line for every continuously paced frame.
        let stable = format!(
            "focused={window_focused} selected={selected:?} paint={paint_intake:?} present={displayed} {owner} game={w}{a}{s}{d} activations={activations} edit_revision={edit_revision} edit_chars={edit_chars}"
        );
        if self.last_observation.as_deref() != Some(stable.as_str()) {
            println!("U9C_FRAME {observation}");
            self.last_observation = Some(stable);
            world
                .resource_mut::<WindowStateRegistryResource>()?
                .record_mut(NativeWindowId::primary())
                .context("primary native window disappeared")?
                .set_title(format!(
                    "U9-C Native UI | present={displayed} | focus={window_focused} | game W={w} | text chars={edit_chars} | UI clicks={activations}"
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
