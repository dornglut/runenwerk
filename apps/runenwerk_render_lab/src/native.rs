use super::*;
use engine::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationPolicyResource,
};
use engine::prelude::FrameEnd;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, runen_ecs::Resource)]
struct RenderLabFlowId(engine::plugins::render::RenderFlowId);

const RL2_MEASUREMENT_HISTORY_CAPACITY: usize = 4096;
const RL2_MEASUREMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, runen_ecs::Resource)]
struct RenderLabMeasurementConfig {
    output_path: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize)]
struct RenderLabMeasurementArtifact {
    schema_version: u32,
    scenario_id: &'static str,
    metric: &'static str,
    host_frame_pacing_target_fps: u32,
    samples: Vec<RenderLabMeasurementSample>,
    drop_stats: RenderLabMeasurementDropStats,
}

#[derive(Debug, serde::Serialize)]
struct RenderLabMeasurementSample {
    frame_index: u64,
    render_surface_id: u64,
    prepare_epoch: u64,
    target_size_px: [u32; 2],
    composed_timing_state: &'static str,
    gpu_composed_frame_ms: Option<f32>,
    diagnostics: Vec<RenderLabMeasurementDiagnostic>,
}

#[derive(Debug, serde::Serialize)]
struct RenderLabMeasurementDiagnostic {
    kind: &'static str,
    capability: &'static str,
    message: String,
}

#[derive(Debug, serde::Serialize)]
struct RenderLabMeasurementDropStats {
    evicted_observations: u64,
    sampled_out_observations: u64,
    dropped_correlated_evidence: u64,
    uncorrelated_evidence: u64,
}

#[derive(runen_ecs::SystemParam)]
struct RenderLabFramePublicationResources<'w> {
    targets: ResMut<'w, RenderDynamicTextureTargetRequestRegistryResource>,
    frame_requests: ResMut<'w, PreparedRenderFrameRequestResource>,
    contributions: ResMut<'w, RenderDeterministicFrameContributionResource>,
}

struct RenderLabPlugin;

impl Plugin for RenderLabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderLabCamera>();
        app.init_resource::<RenderLabMeasurementConfig>();
        app.add_systems(Update, camera::update_render_lab_camera_system);
        app.add_systems(FrameEnd, approve_render_lab_close_system);
        app.add_systems(
            RenderPrepare,
            publish_render_lab_frame_system.before(RenderRuntimeSet::FramePrepare),
        );
    }
}

pub fn run_native() -> Result<()> {
    run_native_with_measurement(None)
}

pub fn run_native_measurement(output_path: impl Into<PathBuf>) -> Result<()> {
    run_native_with_measurement(Some(output_path.into()))
}

fn run_native_with_measurement(output_path: Option<PathBuf>) -> Result<()> {
    let mut app = App::new();
    app.set_title("Runenwerk Render Lab — RL2 native interaction");
    app.with_frame_pacing(FramePacingPolicyResource::continuous_capped(60));
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(RenderLabPlugin);
    if let Some(output_path) = output_path {
        app.insert_resource(RenderLabMeasurementConfig {
            output_path: Some(output_path),
        });
        app.insert_resource(rl2_measurement_policy());
    }
    let flow = render_lab_flow()?;
    app.insert_resource(RenderLabFlowId(flow.id()));
    app.add_render_flow(flow);
    app.run()
}

fn rl2_measurement_policy() -> RenderFrameObservationPolicyResource {
    RenderFrameObservationPolicyResource::enabled(RL2_MEASUREMENT_HISTORY_CAPACITY)
}

fn build_measurement_artifact(history: &RenderFrameHistoryState) -> RenderLabMeasurementArtifact {
    let samples = history
        .observations()
        .map(|observation| {
            let evidence = observation.gpu.composed_timing_evidence.as_ref();
            let diagnostics = evidence
                .map(|evidence| {
                    evidence
                        .diagnostics
                        .iter()
                        .map(|diagnostic| RenderLabMeasurementDiagnostic {
                            kind: diagnostic.kind.as_str(),
                            capability: diagnostic.capability.as_str(),
                            message: diagnostic.message.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            RenderLabMeasurementSample {
                frame_index: observation.key.frame_index,
                render_surface_id: observation.key.render_surface_id,
                prepare_epoch: observation.prepare_epoch,
                target_size_px: [observation.target_size_px.0, observation.target_size_px.1],
                composed_timing_state: observation.gpu.composed_timing_capability.as_str(),
                gpu_composed_frame_ms: evidence.and_then(|evidence| evidence.gpu_composed_frame_ms),
                diagnostics,
            }
        })
        .collect();
    let drop_stats = history.drop_stats();
    RenderLabMeasurementArtifact {
        schema_version: RL2_MEASUREMENT_SCHEMA_VERSION,
        scenario_id: RL2_FLOW_ID,
        metric: "gpu_composed_frame_ms",
        host_frame_pacing_target_fps: 60,
        samples,
        drop_stats: RenderLabMeasurementDropStats {
            evicted_observations: drop_stats.evicted_observations,
            sampled_out_observations: drop_stats.sampled_out_observations,
            dropped_correlated_evidence: drop_stats.dropped_correlated_evidence,
            uncorrelated_evidence: drop_stats.uncorrelated_evidence,
        },
    }
}

fn write_measurement_artifact(output_path: &Path, history: &RenderFrameHistoryState) -> Result<()> {
    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "create RL2 measurement artifact directory {}",
                parent.display()
            )
        })?;
    }
    let bytes = serde_json::to_vec_pretty(&build_measurement_artifact(history))
        .context("serialize RL2 composed GPU measurement artifact")?;
    fs::write(output_path, bytes).with_context(|| {
        format!(
            "write RL2 composed GPU measurement artifact {}",
            output_path.display()
        )
    })
}

pub(super) fn render_lab_flow() -> Result<RenderFlow> {
    RenderFlow::new(RL2_FLOW_ID)
        .with_target_alias(RL2_RADIANCE_ALIAS, RenderTargetAliasKind::Texture)?
        .with_surface_color()?
        .fullscreen_pass(RL2_PASS_ID)
        .main_surface_only()
        .shader_asset("assets/shaders/runenwerk_render_lab_radiance.wgsl")
        .sample_texture_load(runen_gpu::GpuBindingKey::try_new(0, 0)?, RL2_RADIANCE_ALIAS)
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_surface_color()?
        .finish()
        .present_pass(RL2_PRESENT_ID)?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()
}

fn approve_render_lab_close_system(
    mut windows: ResMut<WindowStateRegistryResource>,
    measurement: Res<RenderLabMeasurementConfig>,
    history: Res<RenderFrameHistoryState>,
) -> Result<()> {
    let Some(primary_window_id) = windows.primary_window_id() else {
        return Ok(());
    };
    let close_intent_pending = windows
        .record(primary_window_id)
        .is_some_and(|window| window.close_intent_pending);
    if !close_intent_pending {
        return Ok(());
    }

    if let Some(output_path) = measurement.output_path.as_deref() {
        write_measurement_artifact(output_path, &history)?;
    }
    if let Some(primary_window) = windows.record_mut(primary_window_id) {
        primary_window.approve_close();
    }
    Ok(())
}

fn publish_render_lab_frame_system(
    camera: Res<RenderLabCamera>,
    flow_id: Res<RenderLabFlowId>,
    presentation: Res<engine::PrimaryPresentationMetricsResource>,
    publication: RenderLabFramePublicationResources<'_>,
) -> Result<()> {
    let RenderLabFramePublicationResources {
        mut targets,
        mut frame_requests,
        mut contributions,
    } = publication;
    let (width, height) = render_lab_extent(&presentation);
    let producer_id = engine::plugins::render::RenderFrameProducerId::try_from_raw(RL2_PRODUCER_ID)
        .expect("Render Lab producer id is non-zero");
    let target_key = RenderDynamicTextureTargetKey::new(RL2_TARGET_NAMESPACE, RL2_TARGET_ID);
    let target = RenderDynamicTextureTargetDescriptor::new(
        target_key.clone(),
        width,
        height,
        RenderTextureTargetFormat::R32Float,
        RenderTextureTargetUsage {
            color_attachment: false,
            depth_attachment: false,
            sampled: true,
            storage: false,
            copy_src: false,
            copy_dst: true,
        },
        RenderTextureSampleMode::NonFilterableFloat,
        RenderDynamicTextureRetention::RetainWhileRequested,
    );
    let invocation =
        PreparedFlowInvocationRequest::new(format!("{RL2_FLOW_ID}.main"), flow_id.0, "main")
            .bind_dynamic_texture_alias(RL2_RADIANCE_ALIAS, target_key.clone())?;
    let fixture =
        founding_fixture_with_observation_and_extent(camera.observation_to_scene(), width, height)?;
    let contribution = RenderDeterministicFrameContribution {
        producer_id,
        render_surface_id: RenderSurfaceId::primary(),
        scene: fixture.scene,
        request: fixture.request,
        semantic_inputs: fixture.semantic_inputs,
        availability: fixture.availability,
        output_index: 0,
        target_key,
    };
    stage_render_lab_frame_publication(
        &mut targets,
        &mut frame_requests,
        &mut contributions,
        producer_id,
        target,
        invocation,
        contribution,
    )
}

fn render_lab_extent(presentation: &engine::PrimaryPresentationMetricsResource) -> (u32, u32) {
    presentation.size_px()
}

/// Validate every RL2 publication against cloned registries before replacing any live product
/// state. The registries retain their other producers, while a failed replacement leaves the
/// previous complete frame request intact.
fn stage_render_lab_frame_publication(
    targets: &mut RenderDynamicTextureTargetRequestRegistryResource,
    frame_requests: &mut PreparedRenderFrameRequestResource,
    contributions: &mut RenderDeterministicFrameContributionResource,
    producer_id: engine::plugins::render::RenderFrameProducerId,
    target: RenderDynamicTextureTargetDescriptor,
    invocation: PreparedFlowInvocationRequest,
    contribution: RenderDeterministicFrameContribution,
) -> Result<()> {
    if contribution.render_surface_id != RenderSurfaceId::primary() {
        bail!("RL2 supports exactly one deterministic surface: the primary Render Lab surface");
    }
    let mut staged_targets = targets.clone();
    staged_targets.replace_contribution(producer_id, [target])?;

    let mut staged_frame_requests = frame_requests.clone();
    staged_frame_requests.replace_contribution(producer_id, [], [invocation])?;

    let mut staged_contributions = contributions.clone();
    staged_contributions.replace(contribution);

    *targets = staged_targets;
    *frame_requests = staged_frame_requests;
    *contributions = staged_contributions;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rl2_measurement_policy_is_explicit_and_bounded() {
        let policy = rl2_measurement_policy();
        assert!(policy.enabled);
        assert_eq!(policy.capacity, RL2_MEASUREMENT_HISTORY_CAPACITY);
        assert_eq!(policy.sample_every_nth_frame, 1);
        assert!(RenderLabMeasurementConfig::default().output_path.is_none());
        assert!(!RenderFrameObservationPolicyResource::default().enabled);
    }

    #[test]
    fn measurement_artifact_preserves_exact_extent_and_nonzero_semantics() {
        use engine::plugins::render::inspect::{
            RenderComposedFrameGpuTimingEvidence, RenderGpuTimingCapability,
            RenderGpuTimingDiagnostic,
        };

        let policy = rl2_measurement_policy();
        let mut history = RenderFrameHistoryState::default();
        history.observe_submitted_frame(
            policy,
            21,
            RenderSurfaceId::primary().raw(),
            121,
            (1600, 1200),
            0.0,
            Default::default(),
            &[],
            RenderGpuTimingCapability::UnavailableThisFrame,
        );
        history.observe_composed_gpu_timing_evidence(
            policy,
            &[RenderComposedFrameGpuTimingEvidence::gpu_sample(
                21,
                RenderSurfaceId::primary().raw(),
                6.25,
            )],
        );
        history.observe_submitted_frame(
            policy,
            22,
            RenderSurfaceId::primary().raw(),
            122,
            (3024, 1964),
            0.0,
            Default::default(),
            &[],
            RenderGpuTimingCapability::UnavailableThisFrame,
        );
        history.observe_composed_gpu_timing_evidence(
            policy,
            &[RenderComposedFrameGpuTimingEvidence::gpu_diagnostic(
                22,
                RenderSurfaceId::primary().raw(),
                RenderGpuTimingDiagnostic::readback_pending("pending"),
            )],
        );

        let artifact = build_measurement_artifact(&history);
        assert_eq!(artifact.metric, "gpu_composed_frame_ms");
        assert_eq!(artifact.samples.len(), 2);
        assert_eq!(artifact.samples[0].frame_index, 21);
        assert_eq!(
            artifact.samples[0].render_surface_id,
            RenderSurfaceId::primary().raw()
        );
        assert_eq!(artifact.samples[0].prepare_epoch, 121);
        assert_eq!(artifact.samples[0].target_size_px, [1600, 1200]);
        assert_eq!(artifact.samples[0].gpu_composed_frame_ms, Some(6.25));
        assert_eq!(artifact.samples[1].frame_index, 22);
        assert_eq!(
            artifact.samples[1].render_surface_id,
            RenderSurfaceId::primary().raw()
        );
        assert_eq!(artifact.samples[1].prepare_epoch, 122);
        assert_eq!(artifact.samples[1].target_size_px, [3024, 1964]);
        assert_eq!(
            artifact.samples[1].composed_timing_state,
            "readback_pending"
        );
        assert_eq!(artifact.samples[1].gpu_composed_frame_ms, None);
        assert_eq!(artifact.samples[1].diagnostics.len(), 1);
    }

    #[test]
    fn render_lab_extent_uses_primary_presentation_metrics() {
        let presentation = engine::PrimaryPresentationMetricsResource::new((901, 577), 1.25);
        assert_eq!(render_lab_extent(&presentation), (901, 577));
    }

    fn producer(raw: u64) -> engine::plugins::render::RenderFrameProducerId {
        engine::plugins::render::RenderFrameProducerId::try_from_raw(raw)
            .expect("test producer id should be non-zero")
    }

    fn target(
        key: RenderDynamicTextureTargetKey,
        width: u32,
    ) -> RenderDynamicTextureTargetDescriptor {
        RenderDynamicTextureTargetDescriptor::new(
            key,
            width,
            width,
            RenderTextureTargetFormat::R32Float,
            RenderTextureTargetUsage {
                color_attachment: false,
                depth_attachment: false,
                sampled: true,
                storage: false,
                copy_src: false,
                copy_dst: true,
            },
            RenderTextureSampleMode::NonFilterableFloat,
            RenderDynamicTextureRetention::RetainWhileRequested,
        )
    }

    #[test]
    fn failed_publication_preserves_the_last_complete_frame() {
        let flow = render_lab_flow().expect("RL2 flow should author");
        let rl2_producer = producer(RL2_PRODUCER_ID);
        let other_producer = producer(RL2_PRODUCER_ID + 1);
        let old_key = RenderDynamicTextureTargetKey::new(RL2_TARGET_NAMESPACE, "old");
        let new_key = RenderDynamicTextureTargetKey::new(RL2_TARGET_NAMESPACE, "new");

        let old_target = target(old_key.clone(), 4);
        let new_target = target(new_key.clone(), 8);
        let old_invocation =
            PreparedFlowInvocationRequest::new("runenwerk.render_lab.rl2.old", flow.id(), "main");
        let conflicting_invocation =
            PreparedFlowInvocationRequest::new(format!("{RL2_FLOW_ID}.main"), flow.id(), "main");
        let fixture = founding_fixture().expect("test fixture should build");
        let old_contribution = RenderDeterministicFrameContribution {
            producer_id: rl2_producer,
            render_surface_id: RenderSurfaceId::primary(),
            scene: fixture.scene,
            request: fixture.request,
            semantic_inputs: fixture.semantic_inputs,
            availability: fixture.availability,
            output_index: 0,
            target_key: old_key.clone(),
        };
        let new_fixture =
            founding_fixture_with_observation_and_extent(RenderAffineTransform3::identity(), 8, 8)
                .expect("new fixture should build");
        let new_contribution = RenderDeterministicFrameContribution {
            producer_id: rl2_producer,
            render_surface_id: RenderSurfaceId::primary(),
            scene: new_fixture.scene,
            request: new_fixture.request,
            semantic_inputs: new_fixture.semantic_inputs,
            availability: new_fixture.availability,
            output_index: 0,
            target_key: new_key,
        };

        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        targets
            .replace_contribution(rl2_producer, [old_target.clone()])
            .expect("old target should be valid");
        let mut frame_requests = PreparedRenderFrameRequestResource::default();
        frame_requests
            .replace_contribution(rl2_producer, [], [old_invocation.clone()])
            .expect("old invocation should be valid");
        frame_requests
            .replace_contribution(other_producer, [], [conflicting_invocation])
            .expect("foreign conflicting invocation should be valid");
        let mut contributions = RenderDeterministicFrameContributionResource::default();
        contributions.replace(old_contribution.clone());

        let result = stage_render_lab_frame_publication(
            &mut targets,
            &mut frame_requests,
            &mut contributions,
            rl2_producer,
            new_target,
            PreparedFlowInvocationRequest::new(format!("{RL2_FLOW_ID}.main"), flow.id(), "main"),
            new_contribution,
        );

        assert!(
            result.is_err(),
            "foreign invocation collision must reject publication"
        );
        assert_eq!(targets.snapshot(), vec![old_target]);
        assert_eq!(
            frame_requests.requested_flow_invocations(),
            vec![
                &old_invocation,
                &PreparedFlowInvocationRequest::new(
                    format!("{RL2_FLOW_ID}.main"),
                    flow.id(),
                    "main",
                )
            ]
        );
        assert_eq!(contributions.remove(rl2_producer), Some(old_contribution));
    }

    #[test]
    fn rl2_rejects_deterministic_publication_to_a_secondary_surface() {
        let flow = render_lab_flow().expect("RL2 flow should author");
        let fixture = founding_fixture().expect("test fixture should build");
        let producer_id = producer(RL2_PRODUCER_ID);
        let target_key = RenderDynamicTextureTargetKey::new(RL2_TARGET_NAMESPACE, "secondary");
        let target = target(target_key.clone(), 4);
        let invocation = PreparedFlowInvocationRequest::new(
            format!("{RL2_FLOW_ID}.secondary"),
            flow.id(),
            "main",
        );
        let contribution = RenderDeterministicFrameContribution {
            producer_id,
            render_surface_id: RenderSurfaceId::try_from_raw(2).expect("secondary surface id"),
            scene: fixture.scene,
            request: fixture.request,
            semantic_inputs: fixture.semantic_inputs,
            availability: fixture.availability,
            output_index: 0,
            target_key,
        };
        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        let mut frame_requests = PreparedRenderFrameRequestResource::default();
        let mut contributions = RenderDeterministicFrameContributionResource::default();

        assert!(
            stage_render_lab_frame_publication(
                &mut targets,
                &mut frame_requests,
                &mut contributions,
                producer_id,
                target,
                invocation,
                contribution,
            )
            .is_err()
        );
        assert!(targets.snapshot().is_empty());
        assert!(frame_requests.is_empty());
        assert!(contributions.take_all().is_empty());
    }
}
