//! Native, bounded validation of the *actual* Boids example in both modes.
//! Counts accepted surface Present operations and terminal RunenGPU frame
//! receipts separately; neither is mistaken for hardware scanout timing.
use super::app::{BoidsSourcePacing, BoidsVisualContinuityEvidence};
use anyhow::{Context, Result};
use engine::plugins::render::host::RenderSurfaceId;
use engine::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationPolicyResource,
};
use engine::plugins::render::renderer::{RenderGpuFrameTerminalDisposition, RenderGpuGraphStats};
use engine::plugins::render::{Gfx, RenderFlowId, RenderGpuFlowStepStreams};
use engine::prelude::{App, FixedTimeState};
use engine::runtime::{
    NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId, WindowStateRegistryResource,
};
use runen_ecs::World;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::window::Window;

const REQUIRED_PRESENT_SUBMISSIONS: usize = 120;
const MAX_NATIVE_PRESENT_TARGET: usize = 1_200;
const REQUIRED_VERIFIED_TICKS: u64 = 4;
const REQUIRED_TERMINAL_FRAMES: usize = 2;
const REQUIRED_CHANGED_GPU_IMAGES: u64 = 3;
const MAX_WINDOW_OPPORTUNITIES: u32 = 3_000;
const MAX_DURATION: Duration = Duration::from_secs(180);

#[derive(Debug, Default)]
pub(crate) struct NativeBoidsSmokeStatus {
    frames: u32,
    presented_opportunities: usize,
    terminal_gpu_frames: usize,
    verified_ticks: u64,
    sampled_gpu_images: u64,
    changed_gpu_images: u64,
    require_visual_readback: bool,
    peak_outstanding_steps: usize,
    peak_uncaptured_cpu_ticks: u64,
    source_time_dilated_host_ticks: u64,
    frame_gate_observation: Option<engine::plugins::render::renderer::GfxFrameGateCounters>,
    first_present: Option<Instant>,
    last_present: Option<Instant>,
    present_intervals: Vec<Duration>,
    terminal_host_ages: Vec<Duration>,
    cpu_renderer_samples: Vec<NativeBoidsFrameSample>,
    required_present_submissions: usize,
    succeeded: bool,
    error: Option<String>,
}

/// Observation from the existing bounded submitted-frame history, not a new collector.
#[derive(Debug, Clone, Copy)]
struct NativeBoidsFrameSample {
    frame_index: u64,
    render_surface_id: u64,
    prepare_epoch: u64,
    stages_ms: [f32; 6],
    graph_stats: Option<RenderGpuGraphStats>,
}

fn graph_count_range(
    samples: &[RenderGpuGraphStats],
    count: fn(RenderGpuGraphStats) -> usize,
) -> Option<(usize, usize, usize)> {
    let mut counts = samples.iter().copied().map(count).collect::<Vec<_>>();
    if counts.is_empty() {
        return None;
    }
    counts.sort_unstable();
    Some((
        counts[0],
        counts[(counts.len() - 1) / 2],
        *counts.last().unwrap(),
    ))
}

type SharedStatus = Arc<Mutex<NativeBoidsSmokeStatus>>;

/// Host-observed G5 acceptance-to-terminal age, sampled on a later host
/// progress point. Quantized by progress cadence; not a shader GPU timestamp.
fn summarize_terminal_host_ages(samples: &[Duration]) -> (f64, f64) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }
    let average_ms =
        samples.iter().map(Duration::as_secs_f64).sum::<f64>() * 1000.0 / samples.len() as f64;
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let p95_index = (sorted.len() * 95).div_ceil(100) - 1;
    (average_ms, sorted[p95_index].as_secs_f64() * 1000.0)
}

fn parse_native_present_target(value: Option<&str>) -> Result<usize> {
    let Some(value) = value else {
        return Ok(REQUIRED_PRESENT_SUBMISSIONS);
    };
    let target = value
        .parse::<usize>()
        .context("invalid RUNENWERK_BOIDS_NATIVE_PRESENT_TARGET")?;
    anyhow::ensure!(
        (REQUIRED_PRESENT_SUBMISSIONS..=MAX_NATIVE_PRESENT_TARGET).contains(&target),
        "native Boids Present target must be between {REQUIRED_PRESENT_SUBMISSIONS} and {MAX_NATIVE_PRESENT_TARGET}"
    );
    Ok(target)
}

pub(crate) fn install(
    app: &mut App,
    flow: RenderFlowId,
    require_visual_readback: bool,
) -> Result<SharedStatus> {
    let present_target = parse_native_present_target(
        std::env::var("RUNENWERK_BOIDS_NATIVE_PRESENT_TARGET")
            .ok()
            .as_deref(),
    )?;
    app.insert_resource(RenderFrameObservationPolicyResource::enabled(
        present_target,
    ));
    app.init_resource::<NativeWindowHookRegistryResource>();
    let status = Arc::new(Mutex::new(NativeBoidsSmokeStatus {
        require_visual_readback,
        required_present_submissions: present_target,
        ..NativeBoidsSmokeStatus::default()
    }));
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
    let status = status
        .lock()
        .map_err(|_| anyhow::anyhow!("Boids native smoke status poisoned"))?;
    anyhow::ensure!(
        status.succeeded && status.error.is_none(),
        "Boids native smoke failed or stopped early (frames={}, present-submitted={}, terminal={}, verified={}, image_changes={}): {}",
        status.frames,
        status.presented_opportunities,
        status.terminal_gpu_frames,
        status.verified_ticks,
        status.changed_gpu_images,
        status
            .error
            .as_deref()
            .unwrap_or("native window stopped before evidence completed")
    );
    // Measures host-observed Present submissions, not display scanout or GPU
    // shader duration. Exclude shader startup from the first-to-last interval.
    let elapsed_seconds = match (status.first_present, status.last_present) {
        (Some(first), Some(last)) => last.duration_since(first).as_secs_f64(),
        _ => 0.0,
    };
    let present_submit_fps = if elapsed_seconds > 0.0 {
        (status.presented_opportunities.saturating_sub(1) as f64) / elapsed_seconds
    } else {
        0.0
    };
    let mut intervals = status.present_intervals.clone();
    intervals.sort_unstable();
    let p95_ms = if intervals.is_empty() {
        0.0
    } else {
        let index = (intervals.len() * 95).div_ceil(100).saturating_sub(1);
        intervals[index].as_secs_f64() * 1000.0
    };
    let mean_cpu_ms = |index: usize| -> f64 {
        if status.cpu_renderer_samples.is_empty() {
            return 0.0;
        }
        status
            .cpu_renderer_samples
            .iter()
            .map(|sample| f64::from(sample.stages_ms[index]))
            .sum::<f64>()
            / status.cpu_renderer_samples.len() as f64
    };
    let graph_stats = status
        .cpu_renderer_samples
        .iter()
        .filter_map(|sample| sample.graph_stats)
        .collect::<Vec<_>>();
    // This native proof explicitly enables and retains frame-history samples.
    // Missing selected-final graph counts are an observation regression.
    anyhow::ensure!(
        !graph_stats.is_empty() && graph_stats.len() == status.cpu_renderer_samples.len(),
        "native Boids submitted frame(s) missing selected-final GPU graph counts: observed {} of {}",
        graph_stats.len(),
        status.cpu_renderer_samples.len(),
    );
    let cardinalities = |field: fn(RenderGpuGraphStats) -> usize| {
        graph_count_range(&graph_stats, field).map_or_else(
            || "unavailable".to_string(),
            |(min, median, max)| format!("{min}:{median}:{max}"),
        )
    };
    let gates = status.frame_gate_observation.unwrap_or_default();
    let (terminal_age_avg_ms, terminal_age_p95_ms) =
        summarize_terminal_host_ages(&status.terminal_host_ages);
    println!(
        "native_boids_gpu_smoke=pass frames={} present_target={} present_submitted={} terminal_gpu_frames={} verified_gpu_ticks={} sampled_images={} changed_images={} visual_readback={} present_submit_fps={:.2} p95_present_submit_interval_ms={:.2} cpu_renderer_stage_ms_avg={:.2} cpu_flow_encode_ms_avg={:.2} cpu_encode_submit_ms_avg={:.2} gpu_graph_build_ms_avg={:.2} gpu_prepare_submission_ms_avg={:.2} gpu_submit_prepared_ms_avg={:.2} cpu_samples={} peak_outstanding_steps={} peak_uncaptured_cpu_ticks={} source_time_dilated_host_ticks={} gate_decisions={} gate_deferred={} gate_deterministic={} gate_step_stream={} gate_pending_flow={} host_terminal_age_ms_avg={:.2} host_terminal_age_ms_p95={:.2} host_terminal_age_samples={}",
        status.frames,
        status.required_present_submissions,
        status.presented_opportunities,
        status.terminal_gpu_frames,
        status.verified_ticks,
        status.sampled_gpu_images,
        status.changed_gpu_images,
        status.require_visual_readback,
        present_submit_fps,
        p95_ms,
        mean_cpu_ms(0),
        mean_cpu_ms(1),
        mean_cpu_ms(2),
        mean_cpu_ms(3),
        mean_cpu_ms(4),
        mean_cpu_ms(5),
        status.cpu_renderer_samples.len(),
        status.peak_outstanding_steps,
        status.peak_uncaptured_cpu_ticks,
        status.source_time_dilated_host_ticks,
        gates.decisions,
        gates.deferred,
        gates.deterministic_in_flight,
        gates.step_stream_backpressure,
        gates.registered_flow_in_flight,
        terminal_age_avg_ms,
        terminal_age_p95_ms,
        status.terminal_host_ages.len(),
    );
    // A real Boids consumer sample, not a representative all-application workload.
    // Counts, stages and identities belong to exactly the same submitted frame.
    println!(
        "native_boids_graph_shape=observed counted={} retained={} fragments_min_p50_lower_max={} nodes_min_p50_lower_max={} accesses_min_p50_lower_max={} dependencies_min_p50_lower_max={}",
        graph_stats.len(),
        status.cpu_renderer_samples.len(),
        cardinalities(|stats| stats.fragment_count),
        cardinalities(|stats| stats.node_count),
        cardinalities(|stats| stats.resource_access_count),
        cardinalities(|stats| stats.dependency_count),
    );
    if std::env::var("RUNENWERK_BOIDS_GRAPH_EVIDENCE")
        .ok()
        .as_deref()
        == Some("1")
    {
        for sample in &status.cpu_renderer_samples {
            if let Some(stats) = sample.graph_stats {
                println!(
                    "native_boids_graph_frame frame={} surface={} epoch={} fragments={} nodes={} accesses={} dependencies={} gpu_graph_build_ms={:.6} gpu_prepare_submission_ms={:.6} gpu_submit_prepared_ms={:.6}",
                    sample.frame_index,
                    sample.render_surface_id,
                    sample.prepare_epoch,
                    stats.fragment_count,
                    stats.node_count,
                    stats.resource_access_count,
                    stats.dependency_count,
                    sample.stages_ms[3],
                    sample.stages_ms[4],
                    sample.stages_ms[5],
                );
            }
        }
    }
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
        let mut status = self
            .status
            .lock()
            .map_err(|_| anyhow::anyhow!("Boids native smoke status poisoned"))?;
        if let Err(error) = result {
            status.error = Some(format!("{error:#}"));
        }
        let finished = status.succeeded || status.error.is_some();
        drop(status);
        if finished
            && let Ok(windows) = world.resource_mut::<WindowStateRegistryResource>()
            && let Some(primary) = windows.record_mut(NativeWindowId::primary())
        {
            primary.request_close();
        }
        // Native hooks are diagnostic observers. Preserve failures in the
        // shared result checked after the host actually closes.
        Ok(())
    }
}

impl NativeBoidsSmoke {
    fn observe(&mut self, world: &mut World) -> Result<()> {
        let cpu_samples = world
            .resource::<RenderFrameHistoryState>()?
            .observations()
            .filter(|observation| {
                observation.key.render_surface_id == RenderSurfaceId::primary().raw()
                    && observation.presentation.present_operation_submitted
            })
            .map(|observation| NativeBoidsFrameSample {
                frame_index: observation.key.frame_index,
                render_surface_id: observation.key.render_surface_id,
                prepare_epoch: observation.prepare_epoch,
                stages_ms: [
                    observation.cpu.observed_stage_sum_with_acquire_ms(),
                    observation.cpu.renderer.flow_encode_ms,
                    observation.cpu.renderer.encode_submit_ms,
                    observation.cpu.renderer.gpu_graph_build_ms,
                    observation.cpu.renderer.gpu_prepare_submission_ms,
                    observation.cpu.renderer.gpu_submit_prepared_ms,
                ],
                graph_stats: observation.cpu.renderer.graph_stats,
            })
            .collect::<Vec<_>>();
        let presented = cpu_samples.len();
        let source = world
            .resource::<RenderGpuFlowStepStreams>()?
            .stream(self.flow)
            .context("Boids GPU source stream missing while rendering")?;
        let verified = source.completed_tick();
        let outstanding = source.outstanding();
        // Host-vs-producer divergence now measures deliberate time dilation,
        // not a backlog that the simulation will subsequently replay.
        let uncaptured = world
            .resource::<FixedTimeState>()?
            .total_completed_steps
            .saturating_sub(source.requested_tick());
        let dilated = world
            .resource::<BoidsSourcePacing>()?
            .time_dilated_host_ticks();
        let mut terminal = 0;
        let mut terminal_host_ages = Vec::new();
        let mut frame_gate_observation = None;
        if let Ok(gfx) = world.resource_mut::<Gfx>() {
            let primary_surface = RenderSurfaceId::primary();
            if gfx.frame_gate_observation(primary_surface).is_none() {
                gfx.set_frame_gate_observation_enabled(true);
            }
            frame_gate_observation = gfx.frame_gate_observation(primary_surface);
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
                if !outcome
                    .invocations()
                    .iter()
                    .any(|invocation| invocation.flow_id() == self.flow)
                {
                    continue;
                }
                match outcome.disposition() {
                    RenderGpuFrameTerminalDisposition::Completed => {
                        terminal += 1;
                        terminal_host_ages.push(outcome.host_observed_terminal_age());
                    }
                    other => anyhow::bail!("Boids submitted native GPU frame failed: {other:?}"),
                }
            }
        }
        let image_evidence = world.resource::<BoidsVisualContinuityEvidence>()?;
        let sampled_gpu_images = image_evidence.sampled_frames();
        let changed_gpu_images = image_evidence.changed_frames();
        let mut state = self
            .status
            .lock()
            .map_err(|_| anyhow::anyhow!("Boids native smoke status poisoned"))?;
        state.frames += 1;
        state.peak_outstanding_steps = state.peak_outstanding_steps.max(outstanding);
        state.peak_uncaptured_cpu_ticks = state.peak_uncaptured_cpu_ticks.max(uncaptured);
        state.source_time_dilated_host_ticks = dilated;
        state.frame_gate_observation = frame_gate_observation;
        if presented > state.presented_opportunities {
            let previous_count = state.presented_opportunities;
            state
                .cpu_renderer_samples
                .extend(cpu_samples.iter().skip(previous_count).copied());
            let now = Instant::now();
            if let Some(previous) = state.last_present {
                state.present_intervals.push(now.duration_since(previous));
            } else {
                state.first_present = Some(now);
            }
            state.last_present = Some(now);
        }
        state.presented_opportunities = presented;
        state.terminal_gpu_frames += terminal;
        state.terminal_host_ages.extend(terminal_host_ages);
        state.verified_ticks = verified;
        state.sampled_gpu_images = sampled_gpu_images;
        state.changed_gpu_images = changed_gpu_images;
        if state.presented_opportunities >= state.required_present_submissions
            && state.terminal_gpu_frames >= REQUIRED_TERMINAL_FRAMES
            && state.verified_ticks >= REQUIRED_VERIFIED_TICKS
            && (!state.require_visual_readback
                || state.changed_gpu_images >= REQUIRED_CHANGED_GPU_IMAGES)
        {
            state.succeeded = true;
        } else {
            anyhow::ensure!(
                state.frames < MAX_WINDOW_OPPORTUNITIES && self.started.elapsed() < MAX_DURATION,
                "native Boids missed 120 Present opportunities, required GPU image motion or G5 source progress (opportunities={}, completed GPU frames={}, verified ticks={}, changed images={}, visual_readback={}, elapsed={:?})",
                state.presented_opportunities,
                state.terminal_gpu_frames,
                state.verified_ticks,
                state.changed_gpu_images,
                state.require_visual_readback,
                self.started.elapsed()
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod present_target_tests {
    use super::*;

    #[test]
    fn terminal_host_observation_summary_is_bounded_and_empty_safe() {
        assert_eq!(summarize_terminal_host_ages(&[]), (0.0, 0.0));
        let samples = [
            Duration::from_millis(4),
            Duration::from_millis(10),
            Duration::from_millis(18),
            Duration::from_millis(8),
        ];
        let (avg, p95) = summarize_terminal_host_ages(&samples);
        assert_eq!(avg, 10.0);
        assert_eq!(p95, 18.0);
    }

    #[test]
    fn graph_count_summary_preserves_absence_and_actual_zero() {
        assert_eq!(graph_count_range(&[], |stats| stats.node_count), None);
        let counts = [
            RenderGpuGraphStats {
                fragment_count: 2,
                node_count: 7,
                resource_access_count: 9,
                dependency_count: 6,
            },
            RenderGpuGraphStats {
                fragment_count: 0,
                node_count: 0,
                resource_access_count: 0,
                dependency_count: 0,
            },
            RenderGpuGraphStats {
                fragment_count: 3,
                node_count: 5,
                resource_access_count: 8,
                dependency_count: 4,
            },
        ];
        assert_eq!(
            graph_count_range(&counts, |stats| stats.node_count),
            Some((0, 5, 7))
        );
        assert_eq!(
            graph_count_range(&counts, |stats| stats.fragment_count),
            Some((0, 2, 3))
        );
    }

    #[test]
    fn native_target_is_bounded_and_defaults_to_120() {
        assert_eq!(parse_native_present_target(None).unwrap(), 120);
        assert_eq!(parse_native_present_target(Some("120")).unwrap(), 120);
        assert_eq!(parse_native_present_target(Some("600")).unwrap(), 600);
        assert_eq!(parse_native_present_target(Some("1200")).unwrap(), 1_200);
        for invalid in ["", "0", "119", "1201", "abc", "-5"] {
            assert!(parse_native_present_target(Some(invalid)).is_err());
        }
    }
}
