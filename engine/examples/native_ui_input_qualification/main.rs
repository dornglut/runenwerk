//! Opt-in owner-operated native UI/game input observation.
//!
//! This is a real Winit window with an actual mounted RunenUI application and
//! Render Present path. It does not inject events, fabricate M13 receipts, or
//! claim a physical display scanout merely because Present completed.
//!
//! Run with: cargo run -p engine --example native_ui_input_qualification --locked -- --interactive
//! Close with the operating system's window close button.
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use engine::app::AppNativeHostExt;
use engine::plugins::input::domain::action;
use engine::plugins::render::{RenderFlow, RenderFrameProducerId, SurfaceFrameSubmissionOrder};
use engine::plugins::render::host::RenderSurfaceId;
use engine::plugins::ui::{
    AppUiExt, UiNativeInputRecipientsResource, UiRuntimeFontConfiguration,
    UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding, UiRuntimeSlotId,
    UiRuntimeSlotsResource, UiTypedScreenId,
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
}

impl UiApp for QualificationUi {
    type State = UiState;
    type Action = ();
    type HostProtocol = NoHostProtocol;

    fn root(_: &UiState) -> impl View<()> {
        column(vec![
            text("RunenUI native input qualification").into_element(),
            text("Click the button to focus it, then try W, Space and Tab.").into_element(),
            button("Focus / activate this button")
                .id("u9c.qualify.button")
                .key("u9c.qualify.button")
                .on_activate(|| ())
                .into_element(),
        ])
        .id("u9c.qualify.root")
    }

    fn update(
        state: &mut Self::State,
        _: Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        state.activations.fetch_add(1, Ordering::Relaxed);
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
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
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
            WindowEvent::MouseInput { device_id, state, button } => {
                self.pointer_buttons += 1;
                println!(
                    "U9C_OS window={:?} kind=mouse_button device={device_id:?} button={button:?} state={state:?}",
                    window.id(),
                );
            }
            WindowEvent::CursorMoved { device_id, position } => {
                self.cursor_moves += 1;
                if self.cursor_moves == 1 || self.cursor_moves.is_multiple_of(32) {
                    println!(
                        "U9C_OS window={:?} kind=cursor device={device_id:?} position=({:.1},{:.1}) observed_moves={}",
                        window.id(), position.x, position.y, self.cursor_moves,
                    );
                }
            }
            WindowEvent::MouseWheel { device_id, delta, phase, .. } => {
                println!(
                    "U9C_OS window={:?} kind=wheel device={device_id:?} delta={delta:?} phase={phase:?}",
                    window.id(),
                );
            }
            WindowEvent::Touch(touch) => {
                self.touch_events += 1;
                println!(
                    "U9C_OS window={:?} kind=touch device={:?} id={} phase={:?}",
                    window.id(), touch.device_id, touch.id, touch.phase,
                );
            }
            WindowEvent::Ime(Ime::Preedit(text, range)) => {
                self.ime_events += 1;
                println!(
                    "U9C_OS window={:?} kind=ime_preedit chars={} range={range:?}",
                    window.id(), text.chars().count(),
                );
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                self.ime_events += 1;
                println!(
                    "U9C_OS window={:?} kind=ime_commit chars={}",
                    window.id(), text.chars().count(),
                );
            }
            WindowEvent::Ime(Ime::Enabled | Ime::Disabled) => {
                println!("U9C_OS window={:?} kind=ime_lifecycle event={event:?}", window.id());
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
        self.frames += 1;
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
                ownership.pointers().iter().filter(|pointer| pointer.capture_owner().is_some()).count(),
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
            "frame={} native_focused={} selected_epoch={selected:?} completed_present={} M13=[{owner}] game_down=[W:{w},A:{a},S:{s},D:{d}] ui_activations={activations}",
            self.frames, window_focused, displayed
        );
        // Only semantic changes are logged; frame counter stays in the output
        // but must not cause one trace line for every continuously paced frame.
        let stable = format!(
            "focused={window_focused} selected={selected:?} present={displayed} {owner} game={w}{a}{s}{d} activations={activations}"
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

    fn detach(&mut self, _: &mut World) -> Result<()> {
        println!(
            "U9C_DETACH frames={} keyboard={} mouse_buttons={} cursor={} touch={} ime={} raw_motion={}",
            self.frames, self.keyboard_events, self.pointer_buttons, self.cursor_moves,
            self.touch_events, self.ime_events, self.raw_motion,
        );
        Ok(())
    }
}

fn main() -> Result<()> {
    anyhow::ensure!(
        std::env::args().skip(1).collect::<Vec<_>>() == ["--interactive"],
        "this owner-operated physical qualification requires --interactive; no events are injected"
    );

    let clicks = Arc::new(AtomicUsize::new(0));
    let fonts = UiRuntimeFontConfiguration::new(vec![FONT.to_vec()]).with_generic_mapping(
        GenericFontFamily::SansSerif,
        vec![FontFamilyName::new("JetBrains Mono").expect("bundled font family")],
    );

    let mut app = App::new();
    app.set_title("U9-C Native UI qualification");
    app.with_frame_pacing(FramePacingPolicyResource::continuous_capped(30));
    app.add_plugins(default_plugins());
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);

    let slot = app
        .ui()
        .mount_app::<QualificationUi>(
            UiTypedScreenId::new("u9c.native.input.qualification"),
            UiState { activations: Arc::clone(&clicks) },
            &fonts,
        )
        .slot_id()
        .context("qualifier RunenUI application did not mount")?;
    app.ui().bind_native_primary_input(slot)?;

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

    let flow = RenderFlow::new("native.u9c.qualifier")
        .with_surface_color()?
        .fullscreen_pass("native.u9c.qualifier.clear")
        .main_surface_only()
        .clear_color([0.07, 0.08, 0.10, 1.0])
        .write_surface_color()?
        .finish()
        .builtin_ui_composite_pass("native.u9c.qualifier.ui")?
        .main_surface_only()
        .finish()
        .present_pass("native.u9c.qualifier.present")?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()?;
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
        });

    println!("U9C_START mode=interactive input=real_os mounted_ui=true render=true");
    println!("U9C_GUIDANCE click UI button, try W/Space/Tab, Alt+Tab and refocus; close via X");
    app.run()?;
    println!("U9C_EXIT result=ok");
    Ok(())
}
