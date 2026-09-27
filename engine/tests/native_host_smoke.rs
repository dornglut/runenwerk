use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use engine::plugins::render::{Gfx, RenderFlow};
use engine::plugins::render::backend::{
    RenderSurfaceId, RenderSurfaceLifecycleState, RenderSurfaceRegistryResource,
};
use engine::plugins::{RenderPlugin, UiPlugin, default_plugins};
use engine::prelude::{App, AppRenderExt, Res, Startup, Update};
use engine::runtime::{
    NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId, NativeWindowLifecycleState,
    WindowStateRegistryResource,
};
use winit::window::Window;

const NO_RENDER_ENV: &str = "RUNENWERK_NATIVE_NO_RENDER_SMOKE";
const RENDER_HOST_ENV: &str = "RUNENWERK_NATIVE_RENDER_HOST_SMOKE";

fn main() {
    match (
        std::env::var_os(NO_RENDER_ENV).is_some(),
        std::env::var_os(RENDER_HOST_ENV).is_some(),
    ) {
        (false, false) => {}
        (true, false) => {
            native_no_render_host_smoke().expect("native no-Render Host smoke should succeed")
        }
        (false, true) => {
            native_render_host_smoke().expect("native selected-Render Host smoke should succeed")
        }
        (true, true) => panic!("native Host smoke modes are mutually exclusive"),
    }
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
        if world
            .resource::<engine::DebugMetricsState>()
            .ok()
            .and_then(|metrics| metrics.last_timings)
            .is_some_and(|timings| timings.submitted)
        {
            self.frame_submitted.store(true, Ordering::SeqCst);
        }

        let should_close = self.frame_submitted.load(Ordering::SeqCst)
            && self.secondary_attached.load(Ordering::SeqCst);
        let windows = world.resource_mut::<WindowStateRegistryResource>()?;
        if !self.secondary_requested {
            windows.request_window("Native selected-Render secondary", (640, 480));
            self.secondary_requested = true;
        }
        if should_close || self.frames_seen >= 16 {
            windows
                .record_mut(NativeWindowId::primary())
                .ok_or_else(|| anyhow::anyhow!("selected-Render smoke primary window is missing"))?
                .request_close();
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

fn native_render_host_smoke() -> anyhow::Result<()> {
    let startup_ran = Arc::new(AtomicBool::new(false));
    let update_ran = Arc::new(AtomicBool::new(false));
    let frame_seen = Arc::new(AtomicBool::new(false));
    let frame_submitted = Arc::new(AtomicBool::new(false));
    let primary_attached = Arc::new(AtomicBool::new(false));
    let secondary_attached = Arc::new(AtomicBool::new(false));

    let mut app = App::new();
    app.add_plugins(default_plugins());
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);
    let flow = RenderFlow::new("native.render.host.smoke")
        .with_surface_color()?
        .fullscreen_pass("native.render.host.smoke.clear")
        .main_surface_only()
        .write_surface_color()?
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
    anyhow::ensure!(
        app.world()
            .resource::<engine::plugins::SceneResource>()?
            .manager
            .is_none(),
        "native Render + UI smoke must not activate a Scene manager"
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
        "Render + UiPlugin without ScenePlugin never submitted a native frame"
    );

    println!("native_render_host_smoke=pass");
    Ok(())
}
