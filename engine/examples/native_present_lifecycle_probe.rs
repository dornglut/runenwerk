//! Owner-hardware lifecycle qualification for #1302.
//!
//! Intentionally a separate example, not the shared U9-C native_host_smoke
//! fixture. Runs real native surfaces; never synthesizes Outdated/Suboptimal.
//! Physical status is reported only when observed.
use anyhow::{Context, Result, ensure};
use engine::plugins::render::host::{
    RenderSurfaceId, RenderSurfaceLifecycleState, RenderSurfacePresentationDisposition,
    RenderSurfacePresentationRequest, RenderSurfacePresentationSnapshot,
    RenderSurfaceRegistryResource,
};
use engine::plugins::render::{Gfx, RenderFlow};
use engine::plugins::{RenderPlugin, default_plugins};
use engine::prelude::{App, AppRenderExt};
use engine::runtime::{
    NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId, NativeWindowLifecycleState,
    WindowStateRegistryResource,
};
use runen_gpu::GpuSurfacePresentMode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};

const SIZE_A: (u32, u32) = (914, 568);
const SIZE_B: (u32, u32) = (1038, 582);

#[derive(Debug, Clone, Copy)]
enum Phase {
    CreateSecondary,
    WaitSecondary,
    WaitResizeA,
    WaitResizeB,
    WaitSecondaryRetirement,
    WaitSecondaryRecreation,
    Steady,
    Finished,
}

struct PhysicalProbe {
    primary_window: Option<WindowId>,
    first_secondary: Option<(NativeWindowId, RenderSurfaceId)>,
    second_secondary: Option<(NativeWindowId, RenderSurfaceId)>,
    phase: Phase,
    frames: u32,
    resized_events: u32,
    expected_revision: Option<u64>,
    steady_epoch: Option<String>,
    steady_frames: u32,
    steady_epoch_changes: u32,
    started: Instant,
    completed: Arc<AtomicBool>,
    failure: Arc<Mutex<Option<String>>>,
}

impl PhysicalProbe {
    fn new(completed: Arc<AtomicBool>, failure: Arc<Mutex<Option<String>>>) -> Self {
        Self {
            primary_window: None,
            first_secondary: None,
            second_secondary: None,
            phase: Phase::CreateSecondary,
            frames: 0,
            resized_events: 0,
            expected_revision: None,
            steady_epoch: None,
            steady_frames: 0,
            steady_epoch_changes: 0,
            started: Instant::now(),
            completed,
            failure,
        }
    }

    fn presentation(
        world: &runen_ecs::World,
        id: RenderSurfaceId,
    ) -> Result<RenderSurfacePresentationSnapshot> {
        let record = world
            .resource::<RenderSurfaceRegistryResource>()?
            .record(id)
            .context("Render surface record absent")?;
        Ok(world.resource::<Gfx>()?.surface_presentation(record))
    }

    fn applied(
        world: &runen_ecs::World,
        id: RenderSurfaceId,
        mode: GpuSurfacePresentMode,
        latency: u32,
    ) -> Result<bool> {
        let record = world
            .resource::<RenderSurfaceRegistryResource>()?
            .record(id)
            .context("Render surface record absent")?;
        if record.lifecycle_state != RenderSurfaceLifecycleState::Attached {
            return Ok(false);
        }
        let observation = world.resource::<Gfx>()?.surface_presentation(record);
        Ok(
            observation.disposition == RenderSurfacePresentationDisposition::Applied
                && observation.applied.is_some_and(|a| {
                    a.revision == record.presentation_revision
                        && a.request.present_mode == mode
                        && a.request.desired_maximum_frame_latency.get() == latency
                }),
        )
    }

    fn request_secondary(
        world: &mut runen_ecs::World,
        label: &str,
    ) -> Result<(NativeWindowId, RenderSurfaceId)> {
        let request = world
            .resource_mut::<WindowStateRegistryResource>()?
            .request_window(label, (480, 320));
        let surface = world
            .resource_mut::<RenderSurfaceRegistryResource>()?
            .reserve_surface_for_native_window(request.native_window_id, request.size_px);
        world
            .resource_mut::<RenderSurfaceRegistryResource>()?
            .set_presentation_request(
                surface,
                RenderSurfacePresentationRequest::new(GpuSurfacePresentMode::Fifo, 1)?,
            )?;
        Ok((request.native_window_id, surface))
    }

    fn advance(&mut self, window: &Window, world: &mut runen_ecs::World) -> Result<()> {
        if self.primary_window != Some(window.id()) {
            return Ok(());
        }
        self.frames += 1;
        ensure!(
            self.frames <= 600 && self.started.elapsed() < Duration::from_secs(60),
            "physical qualification timed out: phase={:?}, frames={}, resized_events={}",
            self.phase,
            self.frames,
            self.resized_events
        );

        match self.phase {
            Phase::CreateSecondary => {
                ensure!(
                    Self::applied(
                        world,
                        RenderSurfaceId::primary(),
                        GpuSurfacePresentMode::Fifo,
                        2
                    )?,
                    "primary startup did not apply FIFO/2"
                );
                let secondary = Self::request_secondary(world, "RPRES Stage3 secondary A")?;
                println!(
                    "rpres3=secondary_requested id={:?} surface={:?}",
                    secondary.0, secondary.1
                );
                self.first_secondary = Some(secondary);
                self.phase = Phase::WaitSecondary;
            }
            Phase::WaitSecondary => {
                let (_, secondary) = self.first_secondary.context("first secondary missing")?;
                if Self::applied(world, secondary, GpuSurfacePresentMode::Fifo, 1)? {
                    println!("rpres3=secondary_attached fifo_latency=1");
                    window.request_inner_size(PhysicalSize::new(SIZE_A.0, SIZE_A.1));
                    println!("rpres3=primary_resize_requested size={:?}", SIZE_A);
                    self.phase = Phase::WaitResizeA;
                }
            }
            Phase::WaitResizeA => {
                let size = window.inner_size();
                if (size.width, size.height) == SIZE_A
                    && world
                        .resource::<Gfx>()?
                        .surface_size(RenderSurfaceId::primary())
                        == Some(SIZE_A)
                    && Self::applied(
                        world,
                        RenderSurfaceId::primary(),
                        GpuSurfacePresentMode::Fifo,
                        2,
                    )?
                    && self.resized_events > 0
                {
                    let previous = Self::presentation(world, RenderSurfaceId::primary())?;
                    println!(
                        "rpres3=resize_a_applied mode=Fifo latency=2 epoch={:?}",
                        previous.attachment_epoch
                    );
                    ensure!(
                        previous
                            .supported_at_attachment
                            .as_ref()
                            .is_some_and(|modes| modes.contains(&GpuSurfacePresentMode::Immediate)),
                        "Immediate was not supported on the primary attached epoch; no mode fallback"
                    );
                    let revision = world
                        .resource_mut::<RenderSurfaceRegistryResource>()?
                        .set_presentation_request(
                            RenderSurfaceId::primary(),
                            RenderSurfacePresentationRequest::new(
                                GpuSurfacePresentMode::Immediate,
                                2,
                            )?,
                        )?;
                    self.expected_revision = Some(revision);
                    window.request_inner_size(PhysicalSize::new(SIZE_B.0, SIZE_B.1));
                    println!(
                        "rpres3=mode_and_resize_requested mode=Immediate size={:?} revision={revision}",
                        SIZE_B
                    );
                    self.phase = Phase::WaitResizeB;
                }
            }
            Phase::WaitResizeB => {
                let size = window.inner_size();
                if (size.width, size.height) == SIZE_B
                    && world
                        .resource::<Gfx>()?
                        .surface_size(RenderSurfaceId::primary())
                        == Some(SIZE_B)
                    && Self::applied(
                        world,
                        RenderSurfaceId::primary(),
                        GpuSurfacePresentMode::Immediate,
                        2,
                    )?
                {
                    let observation = Self::presentation(world, RenderSurfaceId::primary())?;
                    ensure!(
                        observation
                            .applied
                            .is_some_and(|a| Some(a.revision) == self.expected_revision),
                        "resize and requested mode were not applied at the exact revision"
                    );
                    println!(
                        "rpres3=resize_b_applied mode=Immediate latency=2 revision={:?} epoch={:?}",
                        self.expected_revision, observation.attachment_epoch
                    );
                    let (native, _) = self.first_secondary.context("first secondary missing")?;
                    world
                        .resource_mut::<WindowStateRegistryResource>()?
                        .record_mut(native)
                        .context("first secondary native record absent")?
                        .approve_close();
                    println!("rpres3=secondary_close_requested id={native:?}");
                    self.phase = Phase::WaitSecondaryRetirement;
                }
            }
            Phase::WaitSecondaryRetirement => {
                let (native, surface) = self.first_secondary.context("first secondary missing")?;
                let windows_gone = world
                    .resource::<WindowStateRegistryResource>()?
                    .record(native)
                    .is_none();
                let retired = world
                    .resource::<RenderSurfaceRegistryResource>()?
                    .record(surface)
                    .is_some_and(|r| r.lifecycle_state == RenderSurfaceLifecycleState::Retired);
                let detached = !world.resource::<Gfx>()?.has_surface(surface);
                if windows_gone && retired && detached {
                    println!("rpres3=secondary_retired id={native:?} surface={surface:?}");
                    let next = Self::request_secondary(world, "RPRES Stage3 secondary B")?;
                    ensure!(
                        next.0 != native && next.1 != surface,
                        "retired secondary identity was reused"
                    );
                    self.second_secondary = Some(next);
                    self.phase = Phase::WaitSecondaryRecreation;
                }
            }
            Phase::WaitSecondaryRecreation => {
                let (_, next) = self
                    .second_secondary
                    .context("recreated secondary missing")?;
                if Self::applied(world, next, GpuSurfacePresentMode::Fifo, 1)? {
                    let observation = Self::presentation(world, next)?;
                    println!(
                        "rpres3=secondary_recreated surface={next:?} epoch={:?}",
                        observation.attachment_epoch
                    );
                    self.steady_epoch = Some(format!(
                        "{:?}",
                        Self::presentation(world, RenderSurfaceId::primary())?.attachment_epoch
                    ));
                    self.phase = Phase::Steady;
                }
            }
            Phase::Steady => {
                ensure!(
                    Self::applied(
                        world,
                        RenderSurfaceId::primary(),
                        GpuSurfacePresentMode::Immediate,
                        2
                    )?,
                    "primary lost its applied Immediate/2 policy during steady-state"
                );
                let current = format!(
                    "{:?}",
                    Self::presentation(world, RenderSurfaceId::primary())?.attachment_epoch
                );
                if self.steady_epoch.as_ref() != Some(&current) {
                    self.steady_epoch_changes += 1;
                    println!(
                        "rpres3=steady_attachment_epoch_changed before={:?} after={} count={}",
                        self.steady_epoch, current, self.steady_epoch_changes
                    );
                    self.steady_epoch = Some(current);
                }
                self.steady_frames += 1;
                if self.steady_frames >= 120 {
                    ensure!(
                        self.steady_epoch_changes == 0,
                        "unexpected repeated native primary physical attachment changes in unchanged steady-state"
                    );
                    println!(
                        "rpres3=pass resize_events={} steady_frames={} steady_epoch_changes={} frames={}",
                        self.resized_events,
                        self.steady_frames,
                        self.steady_epoch_changes,
                        self.frames
                    );
                    self.completed.store(true, Ordering::SeqCst);
                    self.phase = Phase::Finished;
                    world
                        .resource_mut::<WindowStateRegistryResource>()?
                        .record_mut(NativeWindowId::primary())
                        .context("primary native record missing")?
                        .approve_close();
                }
            }
            Phase::Finished => {}
        }
        Ok(())
    }
}

impl NativeWindowHook for PhysicalProbe {
    fn name(&self) -> &'static str {
        "rpres_owner_physical_lifecycle_probe"
    }

    fn attach(&mut self, window: &Window, _world: &mut runen_ecs::World) -> Result<()> {
        if self.primary_window.is_none() {
            self.primary_window = Some(window.id());
            println!("rpres3=primary_window_attached");
        }
        Ok(())
    }

    fn window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
        _world: &mut runen_ecs::World,
    ) -> Result<()> {
        if self.primary_window == Some(window.id()) {
            if let WindowEvent::Resized(size) = event {
                self.resized_events += 1;
                println!(
                    "rpres3=primary_resized_event size=({}, {})",
                    size.width, size.height
                );
            }
        }
        Ok(())
    }

    fn frame(&mut self, window: &Window, world: &mut runen_ecs::World) -> Result<()> {
        if let Err(err) = self.advance(window, world) {
            if let Ok(mut failure) = self.failure.lock() {
                if failure.is_none() {
                    *failure = Some(format!("{err:#}"));
                }
            }
            if let Ok(windows) = world.resource_mut::<WindowStateRegistryResource>() {
                if let Some(primary) = windows.record_mut(NativeWindowId::primary()) {
                    primary.approve_close();
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = App::new();
    app.set_title("RPRES-03 Native Physical Lifecycle Qualification");
    app.add_plugins(default_plugins());
    app.add_plugin(RenderPlugin);
    let flow = RenderFlow::new("rpres.physical.lifecycle")
        .with_surface_color()?
        .fullscreen_pass("rpres.physical.lifecycle.clear")
        .main_surface_only()
        .clear_color([0.06, 0.08, 0.12, 1.0])
        .write_surface_color()?
        .finish()
        .present_pass("rpres.physical.lifecycle.present")?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()?;
    app.add_render_flow(flow);
    let completed = Arc::new(AtomicBool::new(false));
    let failure = Arc::new(Mutex::new(None));
    app.init_resource::<NativeWindowHookRegistryResource>();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(PhysicalProbe::new(
            Arc::clone(&completed),
            Arc::clone(&failure),
        ));
    app.run()?;
    if let Some(err) = failure
        .lock()
        .map_err(|_| anyhow::anyhow!("failure lock poisoned"))?
        .take()
    {
        anyhow::bail!("RPRES physical lifecycle qualification failed: {err}");
    }
    ensure!(
        completed.load(Ordering::SeqCst),
        "RPRES physical probe closed without completion"
    );
    Ok(())
}
