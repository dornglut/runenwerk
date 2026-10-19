use crate::rendering::{
    BoidsGpuStepSchema, BoidsRenderState, BoidsRuleFamily, DEFAULT_BOID_COUNT, build_render_flow,
};
use anyhow::{Context, Result};
use engine::plugins::render::inspect::{CaptureStage, CaptureTextureClass, RenderCaptureSelector};
use engine::plugins::render::{
    Gfx, RenderGpuFixedStepInput, RenderGpuFlowStepStreams, RenderGpuStepDemand,
};
use engine::plugins::{DebugMetricsPlugin, RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, AppRenderExt, FixedTimeState, Res, ResMut, Update};
use engine::runtime::WindowStateRegistryResource;
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
    // Only installed for the opt-in native fault fixture. This represents a
    // producer refusing further source work, not an injected physical GPU loss.
    native_fault_at_verified_tick: Option<u64>,
    // A full-frame pixel readback is required only by the opt-in visual conformance fixture.
    require_visual_readback: bool,
}

impl BoidsVerifiedGpuGeneration {
    fn validate(&self, current: GpuContextAffinity, completed_tick: u64) -> Result<()> {
        anyhow::ensure!(
            !self.should_fail_source(completed_tick),
            "Boids native injected source fault after verified GPU tick {completed_tick}; fail-stop before new GPU source work"
        );
        anyhow::ensure!(
            self.last_verified
                .is_none_or(|previous| previous == current),
            "Boids GPU device generation changed after verified steps: stop and perform a clean application restart to reseed tick zero; zeroed retained buffers do not reconstruct previous simulation contents"
        );
        Ok(())
    }

    fn should_fail_source(&self, completed_tick: u64) -> bool {
        self.native_fault_at_verified_tick
            .is_some_and(|threshold| completed_tick >= threshold)
    }

    fn note_verified(&mut self, generation: GpuContextAffinity) {
        self.last_verified = Some(generation);
    }
}

/// The ephemeral Boids example time-dilates its source clock when GPU capacity
/// is exhausted. Engine fixed steps remain host cadence observations, not
/// immutable Boids inputs until admitted by this producer. Applications with
/// externally authored simulation events must preserve those events instead.
#[derive(Debug, Default, runen_ecs::Resource)]
pub(super) struct BoidsSourcePacing {
    last_host_fixed_tick: u64,
    time_dilated_host_ticks: u64,
}

impl BoidsSourcePacing {
    pub(super) const fn time_dilated_host_ticks(&self) -> u64 {
        self.time_dilated_host_ticks
    }
}

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
    let native_smoke = std::env::var("RUNENWERK_BOIDS_NATIVE_SMOKE").as_deref() == Ok("1");
    let require_visual_readback =
        native_smoke && std::env::var("RUNENWERK_BOIDS_NATIVE_NO_READBACK").as_deref() != Ok("1");
    let mut streams = RenderGpuFlowStepStreams::default();
    // Retain at most one authored four-substep region. A larger resident
    // queue would delay visible simulation by many seconds under sustained
    // GPU backpressure before time dilation could begin.
    streams.register(source.flow_id(), 0, source.max_substeps() as usize)?;
    streams.require_buffer_readback(&flow, source.witness_buffer().clone())?;
    // Normal simulation requires a 24-byte GPU-written final-stage witness.
    // Full-frame image capture remains opt-in visual conformance only.
    if require_visual_readback {
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
    }

    app.insert_resource(BoidsRenderState::with_rule(rule));
    app.insert_resource(source);
    app.insert_resource(streams);
    app.insert_resource(BoidsVisualContinuityEvidence::default());
    app.insert_resource(BoidsSourcePacing::default());
    let native_fault_at_verified_tick = if native_smoke {
        std::env::var("RUNENWERK_BOIDS_NATIVE_SOURCE_FAULT_AFTER_TICK")
            .ok()
            .map(|value| value.parse::<u64>())
            .transpose()
            .context("invalid native Boids source-fault tick")?
    } else {
        None
    };
    if let Some(threshold) = native_fault_at_verified_tick {
        anyhow::ensure!(
            threshold > 0,
            "native Boids source-fault threshold must be nonzero"
        );
    }
    app.insert_resource(BoidsVerifiedGpuGeneration {
        last_verified: None,
        native_fault_at_verified_tick,
        require_visual_readback,
    });
    app.add_systems(Update, advance_boids_simulation_system);
    app.add_render_flow(flow);
    let smoke = if native_smoke {
        Some(super::smoke::install(
            &mut app,
            source_flow,
            require_visual_readback,
        )?)
    } else {
        None
    };
    app.run()?;
    if let Some(proof) = smoke {
        super::smoke::assert_finished(&proof)?;
    }
    Ok(())
}

/// This interactive example has no unsaved-work close confirmation.
fn approve_boids_window_close(windows: &mut WindowStateRegistryResource) -> bool {
    let Some(primary_id) = windows.primary_window_id() else {
        return false;
    };
    let Some(primary) = windows.record_mut(primary_id) else {
        return false;
    };
    if !primary.close_intent_pending {
        return false;
    }
    primary.approve_close();
    true
}

// All eight parameters are distinct ECS resource contracts; keeping the
// fail-stop close check in this system prevents another GPU source admission
// after the window's close intent is observed.
#[allow(clippy::too_many_arguments)]
fn advance_boids_simulation_system(
    mut windows: ResMut<WindowStateRegistryResource>,
    fixed: Res<FixedTimeState>,
    state: Res<BoidsRenderState>,
    source: Res<BoidsGpuStepSchema>,
    gfx: Res<Gfx>,
    mut streams: ResMut<RenderGpuFlowStepStreams>,
    mut visual: ResMut<BoidsVisualContinuityEvidence>,
    mut generation: ResMut<BoidsVerifiedGpuGeneration>,
    mut source_pacing: ResMut<BoidsSourcePacing>,
) -> Result<()> {
    // Winit's CloseRequested is deliberately an intent, not an automatic
    // Engine exit. Boids owns the decision to approve it. Do not schedule
    // another simulation batch after that decision.
    if approve_boids_window_close(&mut windows) {
        return Ok(());
    }
    let current_affinity = gfx.gpu_context_affinity();
    let flow = source.flow_id();
    let completed_tick = streams
        .stream(flow)
        .context("Boids source stream not registered")?
        .completed_tick();
    generation.validate(current_affinity, completed_tick)?;

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
                // GPU completion alone cannot commit source ticks. The final
                // compute pass must supply an actual GPU-produced result for
                // this same accepted source interval and its final source tick.
                let proof = streams.accepted_buffer_readback(flow)?;
                let readback = accepted
                    .readback(proof.readback_id())
                    .context("Boids accepted G5 graph omitted its required buffer result")?;
                let mut ready = false;
                match readback.status() {
                    GpuReadbackStatus::Pending => {}
                    GpuReadbackStatus::Failed(failure) => {
                        anyhow::bail!("Boids GPU final-stage witness readback failed: {failure:?}");
                    }
                    GpuReadbackStatus::Ready(bytes) => {
                        anyhow::ensure!(
                            proof.matches_submission(&accepted)
                                && bytes.as_bytes().len() as u64 == proof.expected_byte_len(),
                            "Boids GPU final-stage witness is not from its exact completed G5 graph"
                        );
                        let last_step_tick = streams
                            .stream(flow)
                            .and_then(|stream| stream.accepted_steps().last())
                            .context("Boids GPU result has no accepted source steps")?
                            .tick();
                        let source_tick = last_step_tick
                            .checked_sub(1)
                            .and_then(|value| u32::try_from(value).ok())
                            .context("Boids GPU result source tick exceeds its shader domain")?;
                        verify_gpu_simulation_witness(bytes.as_bytes(), source_tick)?;
                        ready = true;
                    }
                }
                if ready && generation.require_visual_readback {
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
                                        rgba[3] == 255
                                            && rgba[..3].iter().any(|channel| *channel > 65)
                                    }),
                                "Boids draw readback is missing visible non-background particle pixels"
                            );
                            // Diagnostics do not authorize semantic advancement:
                            // the exact G5 receipt and immutable tick check below do.
                            visual.observe(bytes.as_bytes(), capture_extent);
                            ready = true;
                        }
                    }
                }
                if ready {
                    let affinity = current_affinity;
                    let stream = streams
                        .stream_mut(flow)
                        .context("Boids source disappeared")?;
                    let previous_tick = stream.completed_tick();
                    let submission_id = accepted.id();
                    let verified_tick = stream
                        .commit_verified(affinity, |submission, steps| {
                            anyhow::ensure!(
                                submission.id() == submission_id
                                    && !steps.is_empty()
                                    && steps.iter().enumerate().all(|(index, step)| {
                                        step.tick() == previous_tick + index as u64 + 1
                                    }),
                                "Boids source/G5 receipt is inconsistent with its immutable step prefix"
                            );
                            Ok::<(), anyhow::Error>(())
                        })
                        .map_err(|error| {
                            anyhow::anyhow!("Boids source semantic verification rejected: {error}")
                        })?;
                    generation.note_verified(affinity);
                    tracing::debug!(
                        verified_tick,
                        native_visual_check = generation.require_visual_readback,
                        "Boids terminal G5 submission and source step prefix committed"
                    );
                }
            }
        }
    }

    let stream = streams
        .stream_mut(flow)
        .context("Boids source stream not registered")?;
    capture_fixed_steps(stream, &source, &state, &mut source_pacing, *fixed)?;
    Ok(())
}

fn capture_fixed_steps(
    stream: &mut RenderGpuStepDemand<RenderGpuFixedStepInput>,
    source: &BoidsGpuStepSchema,
    state: &BoidsRenderState,
    pacing: &mut BoidsSourcePacing,
    fixed: FixedTimeState,
) -> Result<()> {
    let frame_steps = u64::from(fixed.steps_ran_last_frame);
    let preceding_host_tick = fixed
        .total_completed_steps
        .checked_sub(frame_steps)
        .context("Boids FixedTimeState completed fewer ticks than its current interval")?;
    // A skipped Update (including an already-approved close) cannot mint
    // source steps retrospectively. Count its host opportunities as dilated.
    let prior_skipped_host_steps = preceding_host_tick
        .checked_sub(pacing.last_host_fixed_tick)
        .context("Boids host fixed-time cadence regressed behind its previous observation")?;
    // Only this frame's cadence opportunities can become Boids source ticks.
    // An exhausted GPU source stream causes explicit simulation time dilation,
    // not a retained queue of unbounded wall-time debt. Existing captured and
    // accepted ticks retain their original identities and G5 proofs.
    let count = (fixed.steps_ran_last_frame as usize)
        .min(stream.remaining_capacity())
        .min(source.max_substeps() as usize);
    let start = stream.requested_tick();
    let end = start
        .checked_add(u64::try_from(count).context("Boids capture count overflow")?)
        .context("Boids requested tick identity exhausted")?;
    let next_dilated = pacing
        .time_dilated_host_ticks
        .checked_add(prior_skipped_host_steps)
        .and_then(|value| value.checked_add(frame_steps - count as u64))
        .context("Boids source time-dilation count exhausted")?;
    if start != end {
        let history = (start..end)
            .map(|tick| source.capture(state, tick))
            .collect::<Result<Vec<_>>>()?;
        stream
            .capture_batch(history)
            .map_err(|(error, _unconsumed)| {
                anyhow::anyhow!("Boids immutable source admission rejected: {error}")
            })?;
    }
    // Publish pace evidence only after any immutable input admission succeeded.
    pacing.last_host_fixed_tick = fixed.total_completed_steps;
    pacing.time_dilated_host_ticks = next_dilated;
    Ok(())
}

/// Validate a real GPU-computed final-stage result, not a CPU tick echo or
/// population left in retained storage by an earlier accepted submission.
/// The witness proves a bounded invariant (tick, population, plausible boid 0);
/// it does not numerically verify all particles or restore lost GPU contents.
fn verify_gpu_simulation_witness(bytes: &[u8], expected_tick: u32) -> Result<()> {
    anyhow::ensure!(
        bytes.len() == 24,
        "Boids GPU simulation witness must contain exactly 24 bytes"
    );
    let words = bytes.as_chunks::<4>().0;
    let word = |index: usize| u32::from_le_bytes(words[index]);
    anyhow::ensure!(
        word(0) == expected_tick,
        "Boids GPU simulation witness is stale or belongs to another source tick"
    );
    anyhow::ensure!(
        word(1) == DEFAULT_BOID_COUNT,
        "Boids GPU computed population does not match authored source count"
    );
    let px = f32::from_bits(word(2));
    let py = f32::from_bits(word(3));
    let vx = f32::from_bits(word(4));
    let vy = f32::from_bits(word(5));
    anyhow::ensure!(
        px.is_finite() && py.is_finite()
            && (0.0..1.0).contains(&px) && (0.0..1.0).contains(&py),
        "Boids GPU final published particle position is non-finite or outside world"
    );
    anyhow::ensure!(
        vx.is_finite() && vy.is_finite() && vx * vx + vy * vy <= 1.0,
        "Boids GPU final published particle velocity is implausible"
    );
    Ok(())
}

#[cfg(test)]
mod compact_gpu_result_tests {
    use super::*;

    fn encoded(tick: u32, population: u32, position: [f32; 2], velocity: [f32; 2]) -> Vec<u8> {
        [
            tick,
            population,
            position[0].to_bits(),
            position[1].to_bits(),
            velocity[0].to_bits(),
            velocity[1].to_bits(),
        ]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect()
    }

    #[test]
    fn final_gpu_result_rejects_stale_corrupt_and_nonphysical_state() {
        let valid = encoded(21, DEFAULT_BOID_COUNT, [0.5, 0.4], [0.1, -0.2]);
        verify_gpu_simulation_witness(&valid, 21).unwrap();
        assert!(verify_gpu_simulation_witness(&valid, 22).is_err());
        assert!(verify_gpu_simulation_witness(&valid[..23], 21).is_err());
        assert!(verify_gpu_simulation_witness(
            &encoded(21, DEFAULT_BOID_COUNT - 1, [0.5, 0.4], [0.1, -0.2]),
            21
        ).is_err());
        assert!(verify_gpu_simulation_witness(
            &encoded(21, DEFAULT_BOID_COUNT, [f32::NAN, 0.4], [0.1, -0.2]),
            21
        ).is_err());
        assert!(verify_gpu_simulation_witness(
            &encoded(21, DEFAULT_BOID_COUNT, [1.2, 0.4], [0.1, -0.2]),
            21
        ).is_err());
        assert!(verify_gpu_simulation_witness(
            &encoded(21, DEFAULT_BOID_COUNT, [0.5, 0.4], [f32::INFINITY, 0.0]),
            21
        ).is_err());
        assert!(verify_gpu_simulation_witness(
            &encoded(21, DEFAULT_BOID_COUNT, [0.5, 0.4], [1.1, 0.0]),
            21
        ).is_err());
    }
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
    fn clean_restart_reseeds_new_epoch_without_reusing_old_producer_history() {
        for rule in [
            crate::rendering::BoidsRuleFamily::Flocking,
            crate::rendering::BoidsRuleFamily::SpeciesInteraction,
        ] {
            let previous_flow = build_render_flow();
            let previous_schema = BoidsGpuStepSchema::from_flow(&previous_flow).unwrap();
            let state = BoidsRenderState::with_rule(rule);
            // Previous verified source tick six is an illustrative producer
            // checkpoint. Following an unknown/partial GPU write, retaining
            // any newer pending inputs is NOT proof of reconstructed buffers.
            let mut quarantined = RenderGpuStepDemand::new(6, 8);
            let prior_input = previous_schema.capture(&state, 6).unwrap();
            quarantined.capture(prior_input).unwrap();
            assert_eq!(quarantined.requested_tick(), 7);
            assert_eq!(quarantined.completed_tick(), 6);
            assert_eq!(quarantined.pending_len(), 1);

            // A clean application restart must form a different RenderFlow and
            // NEW source epoch. It does not mutate or requeue the stale state.
            let restarted_flow = build_render_flow();
            let restarted_schema = BoidsGpuStepSchema::from_flow(&restarted_flow).unwrap();
            assert_ne!(previous_schema.flow_id(), restarted_schema.flow_id());
            let mut restarted = RenderGpuStepDemand::new(0, 8);
            let mut pacing = BoidsSourcePacing::default();
            capture_fixed_steps(
                &mut restarted,
                &restarted_schema,
                &state,
                &mut pacing,
                fixed(1, 1),
            )
            .unwrap();
            assert_eq!(restarted.completed_tick(), 0);
            assert_eq!(restarted.requested_tick(), 1);
            let restarted_first = restarted.candidate(1);
            let first = restarted_first[0].input();
            // The new source tick zero carries the exact deterministic
            // shader-seed and per-law compute uniforms, not the old tick six.
            let expected = previous_schema.capture(&state, 0).unwrap();
            assert_eq!(
                first.uniforms.values().collect::<Vec<_>>(),
                expected.uniforms.values().collect::<Vec<_>>()
            );
            assert_eq!(
                first.dispatch_workgroups.values().collect::<Vec<_>>(),
                expected.dispatch_workgroups.values().collect::<Vec<_>>()
            );
            assert_ne!(
                first.uniforms.values().collect::<Vec<_>>(),
                quarantined.candidate(1)[0]
                    .input()
                    .uniforms
                    .values()
                    .collect::<Vec<_>>()
            );
            assert_eq!(quarantined.pending_len(), 1);
            assert_eq!(quarantined.completed_tick(), 6);
        }
    }

    #[test]
    fn native_source_fault_threshold_is_strictly_after_verified_work() {
        let guard = BoidsVerifiedGpuGeneration {
            last_verified: None,
            native_fault_at_verified_tick: Some(12),
            require_visual_readback: true,
        };
        assert!(!guard.should_fail_source(11));
        assert!(guard.should_fail_source(12));
        assert!(guard.should_fail_source(13));
        assert!(!BoidsVerifiedGpuGeneration::default().should_fail_source(u64::MAX));
    }

    #[test]
    fn admitted_boids_history_never_exceeds_one_gpu_region() {
        for rule in [
            crate::rendering::BoidsRuleFamily::Flocking,
            crate::rendering::BoidsRuleFamily::SpeciesInteraction,
        ] {
            let schema = BoidsGpuStepSchema::from_flow(&build_render_flow()).unwrap();
            let state = BoidsRenderState::with_rule(rule);
            let mut demand = RenderGpuStepDemand::new(0, schema.max_substeps() as usize);
            let mut pacing = BoidsSourcePacing::default();
            capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(4, 4)).unwrap();
            assert_eq!(demand.outstanding(), schema.max_substeps() as usize);
            capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(8, 4)).unwrap();
            capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(9, 1)).unwrap();
            assert_eq!(demand.outstanding(), schema.max_substeps() as usize);
            assert_eq!(demand.requested_tick(), 4);
            assert_eq!(pacing.time_dilated_host_ticks(), 5);
        }
    }

    #[test]
    fn zero_one_many_and_backpressure_preserve_unsubmitted_boids_inputs() {
        let schema = BoidsGpuStepSchema::from_flow(&build_render_flow()).unwrap();
        let state = BoidsRenderState::default();
        let mut demand = RenderGpuStepDemand::new(0, 5);
        let mut pacing = BoidsSourcePacing::default();

        capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(0, 0)).unwrap();
        capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(1, 1)).unwrap();
        capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(1, 0)).unwrap();
        capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(4, 3)).unwrap();
        assert_eq!((demand.requested_tick(), demand.pending_len()), (4, 4));
        capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(6, 2)).unwrap();
        assert_eq!((demand.requested_tick(), demand.pending_len()), (5, 5));
        assert_eq!(pacing.time_dilated_host_ticks(), 1);
        for host_tick in 7..=1006 {
            capture_fixed_steps(
                &mut demand,
                &schema,
                &state,
                &mut pacing,
                fixed(host_tick, 1),
            )
            .unwrap();
        }
        // No backlog replay, silent source-tick skip, or unbounded memory growth.
        assert_eq!((demand.requested_tick(), demand.pending_len()), (5, 5));
        assert_eq!(pacing.time_dilated_host_ticks(), 1001);
        assert!(
            capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(4, 1)).is_err()
        );
        assert_eq!(pacing.time_dilated_host_ticks(), 1001);
        assert_eq!(
            demand
                .candidate(8)
                .iter()
                .map(|input| input.tick())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn original_and_species_source_clocks_do_not_replay_dilated_host_ticks() {
        for rule in [
            crate::rendering::BoidsRuleFamily::Flocking,
            crate::rendering::BoidsRuleFamily::SpeciesInteraction,
        ] {
            let schema = BoidsGpuStepSchema::from_flow(&build_render_flow()).unwrap();
            let state = BoidsRenderState::with_rule(rule);
            let mut pacing = BoidsSourcePacing::default();
            let mut demand = RenderGpuStepDemand::new(0, 2);
            capture_fixed_steps(&mut demand, &schema, &state, &mut pacing, fixed(2, 2)).unwrap();
            for host_tick in 3..=1002 {
                capture_fixed_steps(
                    &mut demand,
                    &schema,
                    &state,
                    &mut pacing,
                    fixed(host_tick, 1),
                )
                .unwrap();
            }
            assert_eq!(demand.requested_tick(), 2);
            assert_eq!(demand.outstanding(), 2);
            assert_eq!(pacing.time_dilated_host_ticks(), 1000);
            let captured = demand.candidate(2);
            assert_eq!(
                captured
                    .iter()
                    .map(|intent| intent.tick())
                    .collect::<Vec<_>>(),
                vec![1, 2]
            );
            for (tick, intent) in captured.iter().enumerate() {
                assert_eq!(
                    intent.input().uniforms,
                    schema.capture(&state, tick as u64).unwrap().uniforms
                );
            }

            // Model the *post-verified* source stream identity, not GPU proof:
            // actual terminal G5 and source verification must occur separately.
            let mut successor = RenderGpuStepDemand::new(2, 2);
            capture_fixed_steps(&mut successor, &schema, &state, &mut pacing, fixed(1003, 1))
                .unwrap();
            assert_eq!(successor.requested_tick(), 3);
            assert_eq!(successor.candidate(2)[0].tick(), 3);
            assert_eq!(pacing.time_dilated_host_ticks(), 1000);
            assert_eq!(
                successor.candidate(2)[0].input().uniforms,
                schema.capture(&state, 2).unwrap().uniforms
            );
        }
    }
}

#[cfg(test)]
mod native_close_policy_tests {
    use super::*;
    use engine::runtime::{NativeWindowId, NativeWindowLifecycleState};

    #[test]
    fn boids_native_close_intent_is_approved_without_continuing_gpu_source_work() {
        let mut windows = WindowStateRegistryResource::default();
        windows.register_primary_window("Boids", (1280, 720), 1.0, true);
        assert!(!approve_boids_window_close(&mut windows));
        let primary = windows
            .record_mut(NativeWindowId::primary())
            .expect("registered primary window");
        primary.receive_close_intent();
        assert!(primary.close_intent_pending);
        assert!(!primary.close_requested);
        assert!(approve_boids_window_close(&mut windows));
        let primary = windows
            .record(NativeWindowId::primary())
            .expect("registered primary window");
        assert!(!primary.close_intent_pending);
        assert!(primary.close_requested);
        assert_eq!(
            primary.lifecycle_state,
            NativeWindowLifecycleState::CloseApproved
        );
        assert!(!approve_boids_window_close(&mut windows));
    }
}
