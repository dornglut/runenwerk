//! Native, bounded validation of the *actual* Boids example in both modes.
//! Counts accepted surface Present operations and terminal RunenGPU frame
//! receipts separately; neither is mistaken for hardware scanout timing.
use anyhow::{Context, Result};
use engine::plugins::render::host::RenderSurfaceId;
use engine::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationPolicyResource,
};
use engine::plugins::render::renderer::RenderGpuFrameTerminalDisposition;
use engine::plugins::render::{Gfx, RenderFlowId, RenderGpuFlowStepStreams};
use engine::prelude::App;
use engine::runtime::{
    NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId,
    WindowStateRegistryResource,
};
use runen_ecs::World;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::window::Window;

const REQUIRED_PRESENT_SUBMISSIONS: usize = 120;
const REQUIRED_VERIFIED_TICKS: u64 = 4;
const REQUIRED_TERMINAL_FRAMES: usize = 2;
const MAX_WINDOW_OPPORTUNITIES: u32 = 3_000;
const MAX_DURATION: Duration = Duration::from_secs(180);

#[derive(Debug, Default)]
pub(crate) struct NativeBoidsSmokeStatus {
    frames: u32,
    presented_opportunities: usize,
    terminal_gpu_frames: usize,
    verified_ticks: u64,
    succeeded: bool,
    error: Option<String>,
}

type SharedStatus = Arc<Mutex<NativeBoidsSmokeStatus>>;

pub(crate) fn install(app: &mut App, flow: RenderFlowId) -> Result<SharedStatus> {
    app.insert_resource(RenderFrameObservationPolicyResource::enabled(512));
    app.init_resource::<NativeWindowHookRegistryResource>();
    let status = SharedStatus::default();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(NativeBoidsSmoke {
            status: Arc::clone(&status),
            flow,
            started: Instant::now(),
            terminal_delivery_enabled: false,
        });
    Ok(status)
}

pub(crate) fn assert_finished(status: &SharedStatus) -> Result<()> {
    let status = status.lock().map_err(|_| anyhow::anyhow!("Boids native smoke status poisoned"))?;
    anyhow::ensure!(
        status.succeeded && status.error.is_none(),
        "Boids native smoke failed or stopped early (frames={}, present-submitted={}, terminal={}, verified={}): {}",
        status.frames,
        status.presented_opportunities,
        status.terminal_gpu_frames,
        status.verified_ticks,
        status.error.as_deref().unwrap_or("native window stopped before evidence completed")
    );
    println!(
        "native_boids_gpu_smoke=pass frames={} present_submitted={} terminal_gpu_frames={} verified_gpu_ticks={}",
        status.frames, status.presented_opportunities, status.terminal_gpu_frames,
        status.verified_ticks
    );
    Ok(())
}

struct NativeBoidsSmoke {
    status: SharedStatus,
    flow: RenderFlowId,
    started: Instant,
    terminal_delivery_enabled: bool,
}

impl NativeWindowHook for NativeBoidsSmoke {
    fn name(&self) -> &'static str {
        "boids_real_native_gpu_execution"
    }

    fn frame(&mut self, _window: &Window, world: &mut World) -> Result<()> {
        let result = self.observe(world);
        let mut status = self.status.lock()
            .map_err(|_| anyhow::anyhow!("Boids native smoke status poisoned"))?;
        if let Err(error) = result {
            status.error = Some(format!("{error:#}"));
        }
        let finished = status.succeeded || status.error.is_some();
        drop(status);
        if finished {
            if let Ok(windows) = world.resource_mut::<WindowStateRegistryResource>()
                && let Some(primary) = windows.record_mut(NativeWindowId::primary())
            {
                primary.request_close();
            }
        }
        // Native hooks are diagnostic observers. Preserve failures in the
        // shared result checked after the host actually closes.
        Ok(())
    }
}

impl NativeBoidsSmoke {
    fn observe(&mut self, world: &mut World) -> Result<()> {
        let presented = world.resource::<RenderFrameHistoryState>()?
            .observations()
            .filter(|observation| {
                observation.key.render_surface_id == RenderSurfaceId::primary().raw()
                    && observation.presentation.present_operation_submitted
            })
            .count();
        let verified = world.resource::<RenderGpuFlowStepStreams>()?
            .stream(self.flow)
            .context("Boids GPU source stream missing while rendering")?
            .completed_tick();
        let mut terminal = 0;
        if let Ok(gfx) = world.resource_mut::<Gfx>() {
            if !self.terminal_delivery_enabled {
                gfx.set_gpu_frame_terminal_delivery(true);
                self.terminal_delivery_enabled = true;
            }
            let batch = gfx.take_gpu_frame_terminal_outcomes();
            anyhow::ensure!(
                batch.delivery_enabled() && !batch.has_delivery_gap(),
                "native Boids GPU terminal evidence disabled or overflowed"
            );
            for outcome in batch.outcomes() {
                if !outcome.invocations().iter().any(|invocation| invocation.flow_id() == self.flow) {
                    continue;
                }
                match outcome.disposition() {
                    RenderGpuFrameTerminalDisposition::Completed => terminal += 1,
                    other => anyhow::bail!("Boids submitted native GPU frame failed: {other:?}"),
                }
            }
        }
        let mut state = self.status.lock()
            .map_err(|_| anyhow::anyhow!("Boids native smoke status poisoned"))?;
        state.frames += 1;
        state.presented_opportunities = presented;
        state.terminal_gpu_frames += terminal;
        state.verified_ticks = verified;
        if state.presented_opportunities >= REQUIRED_PRESENT_SUBMISSIONS
            && state.terminal_gpu_frames >= REQUIRED_TERMINAL_FRAMES
            && state.verified_ticks >= REQUIRED_VERIFIED_TICKS
        {
            state.succeeded = true;
        } else {
            anyhow::ensure!(
                state.frames < MAX_WINDOW_OPPORTUNITIES && self.started.elapsed() < MAX_DURATION,
                "native Boids missed 120 accepted Present opportunities or source-verified G5 progress                  (opportunities={}, completed GPU frames={}, verified ticks={}, elapsed={:?})",
                state.presented_opportunities, state.terminal_gpu_frames,
                state.verified_ticks, self.started.elapsed()
            );
        }
        Ok(())
    }
}
