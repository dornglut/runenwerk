use std::process::Command;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use engine::plugins::render::host::{
    RenderSurfaceId, RenderSurfaceLifecycleState, RenderSurfaceRegistryResource,
};
use engine::plugins::render::{
    Gfx, RenderFlow, RenderFrameProducerId, SurfaceFrameSubmissionOrder,
};
use engine::plugins::ui::{
    AppUiExt, UiRuntimeFontConfiguration, UiRuntimeNativeMapping,
    UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding, UiRuntimeSlotId,
    UiScreen, UiTypedActionDescriptor, UiTypedScreenId, UiTypedSource,
};
use runenui_core::{FontFamilyName, GenericFontFamily, StyleEnvironment};
use ui_controls::BUTTON_CONTROL_KIND_ID;
use ui_definition::{
    AuthoredBindingRef, AuthoredControlAccessibilityDefinition, AuthoredControlKindId,
    AuthoredControlValue, AuthoredId, AuthoredRouteId, UiNodeDefinition, UiValueBinding,
};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion, UiProgramSourceId};
use ui_schema::UiSchemaRef;
use engine::plugins::{RenderPlugin, UiPlugin, default_plugins};
use engine::prelude::{App, AppRenderExt, Res, Startup, Update};
use engine::runtime::{
    NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId, NativeWindowLifecycleState,
    WindowStateRegistryResource,
};
use winit::window::Window;

const NO_RENDER_ENV: &str = "RUNENWERK_NATIVE_NO_RENDER_SMOKE";
const RENDER_HOST_ENV: &str = "RUNENWERK_NATIVE_RENDER_HOST_SMOKE";
const RENDER_UI_HOST_ENV: &str = "RUNENWERK_NATIVE_RENDER_UI_HOST_SMOKE";

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
    match (
        std::env::var_os(NO_RENDER_ENV).is_some(),
        std::env::var_os(RENDER_HOST_ENV).is_some(),
        std::env::var_os(RENDER_UI_HOST_ENV).is_some(),
    ) {
        (false, false, false) => {}
        (true, false, false) => {
            native_no_render_host_smoke().expect("native no-Render Host smoke should succeed")
        }
        (false, true, false) => {
            native_render_host_smoke(false).expect("bare native Render Host smoke should succeed");
            run_native_render_ui_child_smoke()
                .expect("native Render + UI Host child smoke should succeed");
        }
        (false, false, true) => {
            native_render_host_smoke(true).expect("native Render + UI Host smoke should succeed")
        }
        _ => panic!("native Host smoke modes are mutually exclusive"),
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
                    RenderFrameProducerId::try_from_raw(71)
                        .expect("test producer is nonzero"),
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
