use crate::rendering::{BoidsGpuStepSchema, BoidsRenderState, BoidsRuleFamily, build_render_flow};
use anyhow::{Context, Result};
use engine::plugins::render::inspect::{CaptureStage, CaptureTextureClass, RenderCaptureSelector};
use engine::plugins::render::{Gfx, RenderGpuFlowStepStreams};
use engine::plugins::{DebugMetricsPlugin, RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, AppRenderExt, FixedTimeState, Update};
use engine::runtime::WorldMut;
use runen_gpu::{GpuReadbackStatus, GpuSubmissionStatus, GpuTextureFormat};

/// Bounded source-owned historical steps. Exhaustion is an explicit host failure;
/// never silently discard already-run CPU ticks or fabricate GPU simulation work.
const MAX_OUTSTANDING_BOIDS_STEPS: usize = 512;

pub(crate) fn run(rule: BoidsRuleFamily) -> Result<()> {
    let mut app = App::new();
    app.set_title(rule.window_title());
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(DebugMetricsPlugin);

    let flow = build_render_flow();
    let source = BoidsGpuStepSchema::from_flow(&flow)?;
    let mut streams = RenderGpuFlowStepStreams::default();
    streams.register(source.flow_id(), 0, MAX_OUTSTANDING_BOIDS_STEPS)?;
    // This is a bounded initial source-owned GPU result witness, not proof of
    // every boid-buffer invariant. Replace full-frame capture with a compact
    // authoritative state check before production acceptance.
    streams.require_capture(
        source.flow_id(),
        RenderCaptureSelector {
            flow_id: Some(source.flow_id().to_string()),
            pass_id: Some("boids.draw".to_string()),
            stage: CaptureStage::After,
            resource_id: "boids.color".to_string(),
            texture_class: CaptureTextureClass::ColorTarget,
        },
    )?;

    app.insert_resource(BoidsRenderState::with_rule(rule));
    app.insert_resource(source);
    app.insert_resource(streams);
    app.add_systems(Update, advance_boids_simulation_system);
    app.add_render_flow(flow);
    app.run()
}

fn advance_boids_simulation_system(mut world: WorldMut) -> Result<()> {
    let fixed = *world.resource::<FixedTimeState>()?;
    let state = world.resource::<BoidsRenderState>()?.clone();
    let source = world.resource::<BoidsGpuStepSchema>()?.clone();
    let current_affinity = world
        .resource::<Gfx>()
        .ok()
        .map(|gfx| gfx.gpu_context_affinity());
    let streams = world.resource_mut::<RenderGpuFlowStepStreams>()?;
    let flow = source.flow_id();

    // A completed physical G5 submission is never semantic completion by
    // itself. The producer checks its exact readback, captured steps and current
    // device generation, and retains the accepted interval on any uncertainty.
    let accepted = streams
        .stream(flow)
        .and_then(|stream| stream.accepted_submission().cloned());
    if let Some(accepted) = accepted {
        match accepted.status() {
            GpuSubmissionStatus::Accepted => {}
            GpuSubmissionStatus::Failed(failure) => {
                anyhow::bail!(
                    "Boids GPU submission failed ({failure:?}); explicit checkpoint restoration/reseed is required before replay"
                );
            }
            GpuSubmissionStatus::Completed => {
                let evidence = streams.accepted_capture_readbacks(flow)?;
                anyhow::ensure!(
                    evidence.len() == 1 && evidence[0].identity.pass_id() == "boids.draw",
                    "Boids GPU result has no exact source-correlated draw capture"
                );
                let capture = &evidence[0];
                let readback = accepted
                    .readback(capture.receipt.readback_id())
                    .context("Boids accepted G5 submission omitted required GPU readback")?;
                match readback.status() {
                    GpuReadbackStatus::Pending => {}
                    GpuReadbackStatus::Failed(failure) => {
                        anyhow::bail!("Boids source GPU readback failed: {failure:?}");
                    }
                    GpuReadbackStatus::Ready(bytes) => {
                        anyhow::ensure!(
                            capture.receipt.matches_completed_readback(&accepted)
                                && bytes.texture_format() == Some(capture.source_format)
                                && matches!(
                                    capture.source_format,
                                    GpuTextureFormat::Rgba8Unorm
                                        | GpuTextureFormat::Rgba8UnormSrgb
                                        | GpuTextureFormat::Bgra8Unorm
                                        | GpuTextureFormat::Bgra8UnormSrgb
                                ),
                            "Boids completed capture does not match its exact G5 source"
                        );
                        let expected_len = u64::from(capture.width)
                            .checked_mul(u64::from(capture.height))
                            .and_then(|pixels| pixels.checked_mul(4))
                            .and_then(|len| usize::try_from(len).ok())
                            .context("Boids GPU capture dimensions overflow")?;
                        anyhow::ensure!(
                            bytes.as_bytes().len() == expected_len
                                && bytes.as_bytes().chunks_exact(4).any(|rgba| {
                                    rgba[3] == 255 && rgba[..3].iter().any(|channel| *channel > 65)
                                }),
                            "Boids draw readback is missing visible non-background particle pixels"
                        );
                        let affinity = current_affinity
                            .context("Boids accepted result has no live current device-generation authority")?;
                        let stream = streams.stream_mut(flow).context("Boids source disappeared")?;
                        let previous_tick = stream.completed_tick();
                        let submission_id = accepted.id();
                        let verified_tick = stream.commit_verified(affinity, |submission, steps| {
                            anyhow::ensure!(
                                submission.id() == submission_id
                                    && !steps.is_empty()
                                    && steps
                                        .iter()
                                        .enumerate()
                                        .all(|(index, step)| step.tick() == previous_tick + index as u64 + 1),
                                "Boids source/G5 receipt is inconsistent with its immutable step prefix"
                            );
                            Ok::<(), anyhow::Error>(())
                        })?;
                        tracing::debug!(
                            verified_tick,
                            "Boids exact G5 readback and visible result admitted source step prefix"
                        );
                    }
                }
            }
        }
    }

    let start = fixed
        .total_completed_steps
        .checked_sub(u64::from(fixed.steps_ran_last_frame))
        .context("Boids FixedTimeState completed fewer ticks than its current interval")?;
    let stream = streams.stream_mut(flow).context("Boids source stream not registered")?;
    anyhow::ensure!(
        stream.requested_tick() == start,
        "Boids CPU fixed tick source skipped or duplicated work: source {}, host {start}",
        stream.requested_tick()
    );
    let history = (start..fixed.total_completed_steps)
        .map(|tick| source.capture(&state, tick))
        .collect::<Result<Vec<_>>>()?;
    if !history.is_empty() {
        stream
            .capture_batch(history)
            .map_err(|(error, _unconsumed)| {
                anyhow::anyhow!("Boids immutable GPU step queue rejected new CPU source ticks: {error}; application must pause rather than lose steps")
            })?;
    }
    Ok(())
}
