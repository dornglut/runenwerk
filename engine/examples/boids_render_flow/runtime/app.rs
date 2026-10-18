use crate::rendering::{BoidsGpuStepSchema, BoidsRenderState, BoidsRuleFamily, build_render_flow};
use anyhow::{Context, Result};
use engine::plugins::render::inspect::{CaptureStage, CaptureTextureClass, RenderCaptureSelector};
use engine::plugins::render::{
    Gfx, RenderGpuFixedStepInput, RenderGpuFlowStepStreams, RenderGpuStepDemand,
};
use engine::plugins::{DebugMetricsPlugin, RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, AppRenderExt, FixedTimeState, Res, ResMut, Update};
use runen_gpu::{GpuContextAffinity, GpuReadbackStatus, GpuSubmissionStatus, GpuTextureFormat};

/// Diagnostic evidence only: a GPU-rendered image is not a retained Boids
/// storage checkpoint or a numerical oracle. Track visual change independently
/// of exact source-tick semantic admission so the native consumer proof cannot
/// pass on a permanently frozen, yet non-background, draw result.
#[derive(Debug, Default, runen_ecs::Resource)]
pub(super) struct BoidsVisualContinuityEvidence {
    previous_fingerprint: Option<u64>,
    sampled_frames: u64,
    changed_frames: u64,
}

impl BoidsVisualContinuityEvidence {
    fn observe(&mut self, pixels: &[u8], extent: (u32, u32)) {
        // Fixed deterministic sampling avoids hashing every pixel of every
        // retained frame. Source-visible boids occupy many independent pixels.
        const HASH_OFFSET: u64 = 0xcbf29ce484222325;
        const HASH_MULTIPLIER: u64 = 0x100000001b3;
        let pixels_rgba = pixels.as_chunks::<4>().0;
        let stride = pixels_rgba.len().div_ceil(8_192).max(1);
        let mut hash = HASH_OFFSET;
        for value in [extent.0, extent.1] {
            hash = (hash ^ u64::from(value)).wrapping_mul(HASH_MULTIPLIER);
        }
        for rgba in pixels_rgba.iter().step_by(stride) {
            for byte in rgba {
                hash = (hash ^ u64::from(*byte)).wrapping_mul(HASH_MULTIPLIER);
            }
        }
        if self
            .previous_fingerprint
            .is_some_and(|previous| previous != hash)
        {
            self.changed_frames = self.changed_frames.saturating_add(1);
        }
        self.previous_fingerprint = Some(hash);
        self.sampled_frames = self.sampled_frames.saturating_add(1);
    }

    pub(super) const fn sampled_frames(&self) -> u64 {
        self.sampled_frames
    }

    pub(super) const fn changed_frames(&self) -> u64 {
        self.changed_frames
    }
}

/// Source-owner recovery contract: when a GPU generation is replaced after
/// verified work, the previous retained boid buffers cannot be considered
/// reconstructed. Stop before admitting newer tick snapshots; a clean app
/// restart starts an entirely new source epoch at tick zero and reseeds.
#[derive(Debug, Default, runen_ecs::Resource)]
struct BoidsVerifiedGpuGeneration {
    last_verified: Option<GpuContextAffinity>,
}

impl BoidsVerifiedGpuGeneration {
    fn validate(&self, current: GpuContextAffinity) -> Result<()> {
        anyhow::ensure!(
            self.last_verified.is_none_or(|previous| previous == current),
            "Boids previously verified GPU contents belong to a different context/generation;              stop and clean-restart to reseed from source tick zero. Reusing previous              completed ticks against zeroed GPU buffers is prohibited"
        );
        Ok(())
    }

    fn note_verified(&mut self, generation: GpuContextAffinity) {
        self.last_verified = Some(generation);
    }
}

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
    let source_flow = source.flow_id();
    let mut streams = RenderGpuFlowStepStreams::default();
    streams.register(source.flow_id(), 0, MAX_OUTSTANDING_BOIDS_STEPS)?;
    // This is a bounded initial source-owned GPU result witness, not proof of
    // every boid-buffer invariant. Replace full-frame capture with a compact
    // authoritative state check before production acceptance.
    streams.require_capture(
        source.flow_id(),
        RenderCaptureSelector {
            flow_id: Some(source.flow_id().to_string()),
            pass_id: Some(source.draw_pass_id().to_string()),
            stage: CaptureStage::After,
            resource_id: "boids.color".to_string(),
            texture_class: CaptureTextureClass::ColorTarget,
        },
    )?;

    app.insert_resource(BoidsRenderState::with_rule(rule));
    app.insert_resource(source);
    app.insert_resource(streams);
    app.insert_resource(BoidsVisualContinuityEvidence::default());
    app.insert_resource(BoidsVerifiedGpuGeneration::default());
    app.add_systems(Update, advance_boids_simulation_system);
    app.add_render_flow(flow);
    let smoke = if std::env::var("RUNENWERK_BOIDS_NATIVE_SMOKE").as_deref() == Ok("1") {
        Some(super::smoke::install(&mut app, source_flow)?)
    } else {
        None
    };
    app.run()?;
    if let Some(proof) = smoke {
        super::smoke::assert_finished(&proof)?;
    }
    Ok(())
}

fn advance_boids_simulation_system(
    fixed: Res<FixedTimeState>,
    state: Res<BoidsRenderState>,
    source: Res<BoidsGpuStepSchema>,
    gfx: Res<Gfx>,
    mut streams: ResMut<RenderGpuFlowStepStreams>,
    mut visual: ResMut<BoidsVisualContinuityEvidence>,
    mut generation: ResMut<BoidsVerifiedGpuGeneration>,
) -> Result<()> {
    let current_affinity = gfx.gpu_context_affinity();
    generation.validate(current_affinity)?;
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
                    evidence.len() == 1
                        && evidence[0].identity.pass_id() == source.draw_pass_id().to_string(),
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
                        let capture_extent = (capture.width, capture.height);
                        let expected_len = u64::from(capture.width)
                            .checked_mul(u64::from(capture.height))
                            .and_then(|pixels| pixels.checked_mul(4))
                            .and_then(|len| usize::try_from(len).ok())
                            .context("Boids GPU capture dimensions overflow")?;
                        anyhow::ensure!(
                            bytes.as_bytes().len() == expected_len
                                && bytes.as_bytes().as_chunks::<4>().0.iter().any(|rgba| {
                                    rgba[3] == 255 && rgba[..3].iter().any(|channel| *channel > 65)
                                }),
                            "Boids draw readback is missing visible non-background particle pixels"
                        );
                        let affinity = current_affinity;
                        let stream = streams
                            .stream_mut(flow)
                            .context("Boids source disappeared")?;
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
                        }).map_err(|error| anyhow::anyhow!("Boids source semantic verification rejected: {error}"))?;
                        generation.note_verified(affinity);
                        visual.observe(bytes.as_bytes(), capture_extent);
                        tracing::debug!(
                            verified_tick,
                            "Boids exact G5 readback and visible result admitted source step prefix"
                        );
                    }
                }
            }
        }
    }

    let stream = streams
        .stream_mut(flow)
        .context("Boids source stream not registered")?;
    capture_fixed_steps(stream, &source, &state, *fixed)?;
    Ok(())
}

fn capture_fixed_steps(
    stream: &mut RenderGpuStepDemand<RenderGpuFixedStepInput>,
    source: &BoidsGpuStepSchema,
    state: &BoidsRenderState,
    fixed: FixedTimeState,
) -> Result<()> {
    let start = fixed
        .total_completed_steps
        .checked_sub(u64::from(fixed.steps_ran_last_frame))
        .context("Boids FixedTimeState completed fewer ticks than its current interval")?;
    anyhow::ensure!(
        stream.requested_tick() == start,
        "Boids CPU fixed tick source skipped or duplicated work: source {}, host {start}",
        stream.requested_tick()
    );
    let history = (start..fixed.total_completed_steps)
        .map(|tick| source.capture(state, tick))
        .collect::<Result<Vec<_>>>()?;
    if !history.is_empty() {
        stream.capture_batch(history).map_err(|(error, _unconsumed)| {
            anyhow::anyhow!(
                "Boids immutable GPU step queue rejected new CPU source ticks: {error}; application must pause rather than lose steps"
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod visual_continuity_tests {
    use super::BoidsVisualContinuityEvidence;

    #[test]
    fn color_only_motion_witness_rejects_static_frames() {
        let mut witness = BoidsVisualContinuityEvidence::default();
        let pixels = vec![0_u8; 16 * 16 * 4];
        witness.observe(&pixels, (16, 16));
        witness.observe(&pixels, (16, 16));
        assert_eq!(witness.sampled_frames(), 2);
        assert_eq!(
            witness.changed_frames(),
            0,
            "two static frames prove no visual motion"
        );

        let mut moved = pixels.clone();
        moved[0] = 128;
        witness.observe(&moved, (16, 16));
        assert_eq!(witness.changed_frames(), 1);
        witness.observe(&moved, (16, 16));
        assert_eq!(witness.changed_frames(), 1);
    }
}

#[cfg(test)]
mod source_interval_tests {
    use super::*;

    fn fixed(total: u64, count: u32) -> FixedTimeState {
        FixedTimeState {
            total_completed_steps: total,
            steps_ran_last_frame: count,
            ..FixedTimeState::default()
        }
    }

    #[test]
    fn zero_one_many_and_backpressure_preserve_unsubmitted_boids_inputs() {
        let flow = build_render_flow();
        let schema = BoidsGpuStepSchema::from_flow(&flow).unwrap();
        let state = BoidsRenderState::default();
        let mut demand = RenderGpuStepDemand::new(0, 5);

        capture_fixed_steps(&mut demand, &schema, &state, fixed(0, 0)).unwrap();
        assert_eq!((demand.requested_tick(), demand.pending_len()), (0, 0));
        capture_fixed_steps(&mut demand, &schema, &state, fixed(1, 1)).unwrap();
        assert_eq!((demand.requested_tick(), demand.pending_len()), (1, 1));
        capture_fixed_steps(&mut demand, &schema, &state, fixed(1, 0)).unwrap();
        assert_eq!((demand.requested_tick(), demand.pending_len()), (1, 1));
        capture_fixed_steps(&mut demand, &schema, &state, fixed(4, 3)).unwrap();
        assert_eq!((demand.requested_tick(), demand.pending_len()), (4, 4));

        assert!(capture_fixed_steps(&mut demand, &schema, &state, fixed(6, 2)).is_err());
        assert_eq!((demand.requested_tick(), demand.pending_len()), (4, 4));
        assert!(capture_fixed_steps(&mut demand, &schema, &state, fixed(4, 1)).is_err());
        let ticks = demand
            .candidate(8)
            .iter()
            .map(|input| input.tick())
            .collect::<Vec<_>>();
        assert_eq!(ticks, vec![1, 2, 3, 4]);
    }
}
