//! Real X11/Vulkan Runenwerk primary surface retirement against the pinned
//! RunenGPU G7 lease owner and independent secondary G5 terminal Present.
//! This is the Render-owned prerequisite, not Native #1450 App-lifetime proof.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, ensure};
use runen_gpu::*;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::{Window, WindowId};

use super::*;
use crate::app::App;
use crate::plugins::RenderPlugin;

fn label(name: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(name).expect("test GPU resource label is valid")
}

fn provenance(name: &str) -> GpuResourceProvenance {
    GpuResourceProvenance::new(label(name), None, None)
}

fn secondary_present_graph(image: &GpuAcquiredSurfaceImage) -> GpuPreparedWorkGraph {
    let texture = image.texture().clone();
    let view = image.default_view().clone();
    let attachment = GpuRenderColorAttachment::new(
        view.clone(),
        GpuColorAttachmentLoad::Clear(
            GpuColorClearValue::new(0.125, 0.25, 0.5, 1.0).expect("finite test clear"),
        ),
        GpuAttachmentStore::Store,
        None,
    )
    .expect("supported real color attachment");
    let render = GpuRenderOperation::new(
        [attachment],
        None,
        std::iter::empty::<GpuRenderDraw>(),
        None,
    )
    .expect("valid secondary GPU render operation");
    let present = GpuPresentOperation::new(
        view.clone().into(),
        view.descriptor().subresources(),
    )
    .expect("valid secondary GPU Present operation");

    let name = "Runenwerk surviving secondary clear Present";
    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder.declare_resource(texture.into()).unwrap();
    builder.declare_resource(view.into()).unwrap();
    builder
        .add_node(
            label("secondary physical clear"),
            GpuWorkOperation::Render(render),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::GraphicsRequired,
            provenance("secondary physical clear"),
        )
        .unwrap();
    builder
        .add_node(
            label("secondary physical Present"),
            GpuWorkOperation::Present(present),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::GraphicsRequired,
            provenance("secondary physical Present"),
        )
        .unwrap();
    GpuPreparedWorkGraph::prepare(label("secondary continuity"), [builder.finish().unwrap()])
        .unwrap()
}

fn physical_probe(event_loop: &ActiveEventLoop) -> Result<()> {
    let primary_native = NativeWindowId::primary();
    let secondary_native = NativeWindowId::try_from_raw(2).expect("valid native secondary");
    let primary_surface = RenderSurfaceId::primary();

    let window = |title| -> Result<Arc<Window>> {
        Ok(Arc::new(event_loop.create_window(
            Window::default_attributes()
                .with_title(title)
                .with_inner_size(PhysicalSize::new(320, 240)),
        )?))
    };
    let primary_window = window("Runenwerk retire primary")?;
    let secondary_window = window("Runenwerk surviving secondary")?;
    let mut app = App::new();
    app.add_plugin(RenderPlugin);

    let mut gfx = Gfx::new(Arc::clone(&primary_window))?;
    let secondary_surface = app
        .world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()?
        .reserve_surface_for_native_window(secondary_native, (320, 240));
    gfx.attach_surface(secondary_surface, Arc::clone(&secondary_window), (320, 240))?;
    app.world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()?
        .confirm_surface_attachment(primary_surface, primary_native, (320, 240))?;
    app.world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()?
        .confirm_surface_attachment(secondary_surface, secondary_native, (320, 240))?;
    app.world_mut().insert_resource(gfx);

    let acquired_primary = {
        let gfx = app.world().resource::<Gfx>()?;
        let (gpu, surface) = gfx
            .physical_surface_proof_handles(primary_surface)
            .ok_or_else(|| anyhow!("primary physical handle absent"))?;
        gpu.acquire_surface_image(surface)?
    };
    ensure!(
        try_retire_primary_render_surface(app.world_mut())?
            == PrimaryRenderRetirementOutcome::Deferred(
                PrimaryRenderRetirementDeferral::ActiveSurfaceLease,
            ),
        "real acquired primary G7 lease must atomically defer physical retirement"
    );
    ensure!(
        app.world()
            .resource::<RenderSurfaceRegistryResource>()?
            .surface_for_native_window(primary_native)
            == Some(primary_surface),
        "G7 busy outcome must preserve primary identity"
    );

    acquired_primary.abandon();

    ensure!(
        try_retire_primary_render_surface(app.world_mut())?
            == PrimaryRenderRetirementOutcome::Retired,
        "primary G7 conditional detach must retire only after released lease"
    );
    let gfx = app.world().resource::<Gfx>()?;
    ensure!(
        !gfx.has_surface(primary_surface) && gfx.has_surface(secondary_surface),
        "same Render/Gfx context must retain only the secondary physical surface"
    );
    let surfaces = app.world().resource::<RenderSurfaceRegistryResource>()?;
    ensure!(
        surfaces.surface_for_native_window(primary_native).is_none()
            && surfaces.surface_for_native_window(secondary_native) == Some(secondary_surface),
        "primary retirement must preserve secondary Render registry identity"
    );

    let (gpu, secondary_handle) = gfx
        .physical_surface_proof_handles(secondary_surface)
        .ok_or_else(|| anyhow!("surviving secondary GPU handle absent"))?;
    ensure!(
        gpu.adapter_facts().backend() == GpuBackendFamily::Vulkan,
        "native continuation must execute Vulkan, not a simulated backend"
    );
    let image = gpu.acquire_surface_image(secondary_handle)?;
    let prepared = pollster::block_on(
        gpu.prepare_submission(secondary_present_graph(&image)),
    )?;
    let submitted = gpu.submit_prepared(prepared)?;
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        gpu.progress();
        match submitted.status() {
            GpuSubmissionStatus::Completed => break,
            GpuSubmissionStatus::Failed(failure) => {
                return Err(anyhow!("secondary Vulkan G5 Present failed: {failure:?}"));
            }
            GpuSubmissionStatus::Accepted => {}
        }
        ensure!(Instant::now() < deadline, "surviving secondary G5 did not complete");
        std::thread::yield_now();
    }
    let next = gpu.acquire_surface_image(secondary_handle)?;
    next.abandon();
    println!(
        "native_primary_render_retirement_os=pass physical_x11=true vulkan=true primary_lease_refused=true primary_detached=true secondary_g5_present_completed=true"
    );
    Ok(())
}

#[derive(Default)]
struct PhysicalProof {
    attempted: bool,
    outcome: Option<Result<()>>,
}

impl ApplicationHandler for PhysicalProof {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.attempted {
            return;
        }
        self.attempted = true;
        self.outcome = Some(physical_probe(event_loop));
        event_loop.exit();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _id: WindowId,
        _event: WindowEvent,
    ) {
    }
}

#[test]
fn physical_primary_render_retirement_os_proof() -> Result<()> {
    if std::env::var_os("RUNENWERK_NATIVE_PRIMARY_RENDER_RETIRE_OS_SMOKE").is_none() {
        return Ok(());
    }
    let mut builder = EventLoop::builder();
    builder.with_x11().with_any_thread(true);
    let event_loop = builder.build()?;
    let mut probe = PhysicalProof::default();
    event_loop.run_app(&mut probe)?;
    ensure!(probe.attempted, "X11 must invoke one real Winit resumed callback");
    probe
        .outcome
        .take()
        .ok_or_else(|| anyhow!("no physical Runenwerk retirement result"))??;
    Ok(())
}
