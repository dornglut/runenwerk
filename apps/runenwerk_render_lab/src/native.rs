use super::*;
use engine::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationPolicyResource,
};
use engine::prelude::FrameEnd;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, runen_ecs::Resource)]
struct RenderLabFlowId(engine::plugins::render::RenderFlowId);

#[derive(Debug, Clone, Default, runen_ecs::Resource)]
struct RenderLabFixedQualityPlans {
    scene: Option<engine::plugins::render::CompiledRenderFlowPlan>,
    resolve: Option<engine::plugins::render::CompiledRenderFlowPlan>,
}

const RL2_QUALITY_FLOW_ID: &str = "runenwerk.render_lab.rl2.fixed_quality";
const RL2_QUALITY_PASS_ID: &str = "runenwerk.render_lab.rl2.fixed_quality.compose";
const RL2_QUALITY_COLOR_ALIAS: &str = "runenwerk.render_lab.rl2.fixed_quality.color";

const RL2_MEASUREMENT_HISTORY_CAPACITY: usize = 4096;
const RL2_MEASUREMENT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Default, runen_ecs::Resource)]
struct RenderLabMeasurementConfig {
    output_path: Option<PathBuf>,
    submitted_frame_limit: Option<usize>,
    primary_window_size_px: Option<(u32, u32)>,
    radiance_target_size_px: Option<(u32, u32)>,
    quality_capture_output_dir: Option<PathBuf>,
    completed: bool,
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
    radiance_target_size_px: [u32; 2],
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
    fixed_quality_plans: Res<'w, RenderLabFixedQualityPlans>,
}

struct RenderLabPlugin;

impl Plugin for RenderLabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderLabCamera>();
        app.init_resource::<RenderLabMeasurementConfig>();
        app.init_resource::<RenderLabFixedQualityPlans>();
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

pub fn run_native_measurement(
    output_path: impl Into<PathBuf>,
    submitted_frame_limit: Option<usize>,
    primary_window_size_px: Option<(u32, u32)>,
    radiance_target_size_px: Option<(u32, u32)>,
) -> Result<()> {
    let submitted_frame_limit = validate_measurement_frame_limit(submitted_frame_limit)?;
    let primary_window_size_px = validate_measurement_window_size(primary_window_size_px)?;
    let radiance_target_size_px = validate_measurement_radiance_size(radiance_target_size_px)?;
    run_native_with_measurement(Some(RenderLabMeasurementConfig {
        output_path: Some(output_path.into()),
        submitted_frame_limit,
        primary_window_size_px,
        radiance_target_size_px,
        quality_capture_output_dir: None,
        completed: false,
    }))
}

pub fn run_native_temporal_quality(
    output_root: impl Into<PathBuf>,
    submitted_frame_limit: Option<usize>,
    primary_window_size_px: (u32, u32),
    internal_size_px: (u32, u32),
) -> Result<()> {
    let submitted_frame_limit = validate_measurement_frame_limit(submitted_frame_limit)?;
    let primary_window_size_px = validate_measurement_window_size(Some(primary_window_size_px))?
        .expect("validated explicit quality output extent");
    let internal_size_px = validate_measurement_radiance_size(Some(internal_size_px))?
        .expect("validated explicit quality internal extent");
    if internal_size_px.0 > primary_window_size_px.0
        || internal_size_px.1 > primary_window_size_px.1
    {
        bail!(
            "temporal quality internal extent {}x{} exceeds output extent {}x{}",
            internal_size_px.0,
            internal_size_px.1,
            primary_window_size_px.0,
            primary_window_size_px.1
        );
    }
    if u64::from(internal_size_px.0) * u64::from(primary_window_size_px.1)
        != u64::from(internal_size_px.1) * u64::from(primary_window_size_px.0)
    {
        bail!(
            "temporal quality internal extent {}x{} must preserve output aspect {}x{}",
            internal_size_px.0,
            internal_size_px.1,
            primary_window_size_px.0,
            primary_window_size_px.1
        );
    }

    let output_root = output_root.into();
    run_native_with_measurement(Some(RenderLabMeasurementConfig {
        output_path: Some(output_root.join("timing.json")),
        submitted_frame_limit,
        primary_window_size_px: Some(primary_window_size_px),
        radiance_target_size_px: Some(internal_size_px),
        quality_capture_output_dir: Some(output_root.join("captures")),
        completed: false,
    }))
}

fn validate_measurement_window_size(size_px: Option<(u32, u32)>) -> Result<Option<(u32, u32)>> {
    if let Some((width, height)) = size_px
        && (width == 0 || height == 0)
    {
        bail!("RL2 measurement window size must have positive width and height");
    }
    Ok(size_px)
}

fn validate_measurement_radiance_size(size_px: Option<(u32, u32)>) -> Result<Option<(u32, u32)>> {
    if let Some((width, height)) = size_px
        && (width == 0 || height == 0)
    {
        bail!("RL2 measurement radiance size must have positive width and height");
    }
    Ok(size_px)
}

fn validate_measurement_frame_limit(limit: Option<usize>) -> Result<Option<usize>> {
    if let Some(limit) = limit {
        if limit == 0 {
            bail!("RL2 measurement frame limit must be positive");
        }
        if limit > RL2_MEASUREMENT_HISTORY_CAPACITY {
            bail!(
                "RL2 measurement frame limit {limit} exceeds retained history capacity {RL2_MEASUREMENT_HISTORY_CAPACITY}"
            );
        }
    }
    Ok(limit)
}

fn run_native_with_measurement(measurement: Option<RenderLabMeasurementConfig>) -> Result<()> {
    let quality_mode = measurement.as_ref().and_then(|measurement| {
        measurement.quality_capture_output_dir.as_ref().map(|_| {
            (
                measurement
                    .primary_window_size_px
                    .expect("quality mode requires explicit output extent"),
                measurement
                    .radiance_target_size_px
                    .expect("quality mode requires explicit internal extent"),
            )
        })
    });
    let quality_capture_output_dir = measurement
        .as_ref()
        .and_then(|measurement| measurement.quality_capture_output_dir.clone());
    let mut app = App::new();
    app.set_title("Runenwerk Render Lab — RL2 native interaction");
    app.with_frame_pacing(FramePacingPolicyResource::continuous_capped(60));
    if let Some(size_px) = measurement
        .as_ref()
        .and_then(|measurement| measurement.primary_window_size_px)
    {
        app.with_primary_window_size_px(size_px);
    }
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(RenderLabPlugin);
    if let Some(measurement) = measurement {
        app.insert_resource(measurement);
        app.insert_resource(rl2_measurement_policy());
    }
    if let Some((output_size, internal_size)) = quality_mode {
        app.update_render_debug_control(|control| {
            control.capture_enabled = true;
            control.readback_enabled = true;
            control.artifact_export_enabled = true;
            control.artifact_output_dir = quality_capture_output_dir
                .clone()
                .expect("quality mode requires capture output directory");
        });

        if internal_size == output_size {
            let flow = render_lab_flow()?;
            app.update_render_debug_config(|config| {
                config.capture_selectors.clear();
                config.capture_selectors.push(
                    engine::plugins::render::inspect::RenderCaptureSelector::named_pass_surface_color(
                        RL2_FLOW_ID,
                        RL2_PASS_ID,
                    ),
                );
            });
            app.insert_resource(RenderLabFlowId(flow.id()));
            app.add_render_flow(flow);
        } else {
            let scene_flow = render_lab_fixed_quality_flow()?;
            let resolve_flow = engine::plugins::render::fixed_resolution_resolve_flow()?;
            let scene_plan = engine::plugins::render::compile_flow_plan(&scene_flow)?;
            let resolve_plan = engine::plugins::render::compile_flow_plan(&resolve_flow)?;
            app.insert_resource(RenderLabFixedQualityPlans {
                scene: Some(scene_plan),
                resolve: Some(resolve_plan),
            });
            app.update_render_debug_config(|config| {
                config.capture_selectors.clear();
                config.capture_selectors.push(
                    engine::plugins::render::inspect::RenderCaptureSelector::named_pass_surface_color(
                        engine::plugins::render::FIXED_RESOLUTION_RESOLVE_FLOW_LABEL,
                        engine::plugins::render::FIXED_RESOLUTION_RESOLVE_PASS_LABEL,
                    ),
                );
            });
            app.insert_resource(RenderLabFlowId(scene_flow.id()));
            app.add_render_flow(scene_flow);
            app.add_render_flow(resolve_flow);
        }
    } else {
        let flow = render_lab_flow()?;
        app.insert_resource(RenderLabFlowId(flow.id()));
        app.add_render_flow(flow);
    }
    app.run()
}

fn rl2_measurement_policy() -> RenderFrameObservationPolicyResource {
    RenderFrameObservationPolicyResource::enabled(RL2_MEASUREMENT_HISTORY_CAPACITY)
}

fn build_measurement_artifact(
    history: &RenderFrameHistoryState,
    measurement: &RenderLabMeasurementConfig,
) -> RenderLabMeasurementArtifact {
    let samples = history
        .observations()
        .map(|observation| {
            let evidence = observation.gpu.composed_timing_evidence.as_ref();
            let radiance_target_size_px = measurement
                .radiance_target_size_px
                .unwrap_or(observation.target_size_px);
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
                radiance_target_size_px: [radiance_target_size_px.0, radiance_target_size_px.1],
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

fn write_measurement_artifact(
    output_path: &Path,
    history: &RenderFrameHistoryState,
    measurement: &RenderLabMeasurementConfig,
) -> Result<()> {
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
    let bytes = serde_json::to_vec_pretty(&build_measurement_artifact(history, measurement))
        .context("serialize RL2 composed GPU measurement artifact")?;
    fs::write(output_path, bytes).with_context(|| {
        format!(
            "write RL2 composed GPU measurement artifact {}",
            output_path.display()
        )
    })
}

pub(super) fn render_lab_fixed_quality_flow() -> Result<RenderFlow> {
    RenderFlow::new(RL2_QUALITY_FLOW_ID)
        .with_target_alias(RL2_RADIANCE_ALIAS, RenderTargetAliasKind::Texture)?
        .with_color_target_alias(RL2_QUALITY_COLOR_ALIAS)?
        .fullscreen_pass(RL2_QUALITY_PASS_ID)
        .shader_asset("assets/shaders/runenwerk_render_lab_radiance.wgsl")
        .sample_texture_load(runen_gpu::GpuBindingKey::try_new(0, 0)?, RL2_RADIANCE_ALIAS)
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_target_alias(RL2_QUALITY_COLOR_ALIAS)?
        .finish()
        .validate()
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

fn bounded_measurement_complete(
    measurement: &RenderLabMeasurementConfig,
    history: &RenderFrameHistoryState,
) -> bool {
    !measurement.completed
        && measurement
            .submitted_frame_limit
            .is_some_and(|limit| history.len() >= limit)
}

fn complete_render_lab_measurement_if_requested(
    windows: &mut WindowStateRegistryResource,
    measurement: &mut RenderLabMeasurementConfig,
    history: &RenderFrameHistoryState,
) -> Result<()> {
    let Some(primary_window_id) = windows.primary_window_id() else {
        return Ok(());
    };
    let close_intent_pending = windows
        .record(primary_window_id)
        .is_some_and(|window| window.close_intent_pending);
    let bounded_complete = bounded_measurement_complete(measurement, history);
    if !close_intent_pending && !bounded_complete {
        return Ok(());
    }

    if !measurement.completed {
        if let Some(output_path) = measurement.output_path.as_deref() {
            write_measurement_artifact(output_path, history, measurement)?;
        }
        measurement.completed = true;
    }
    if let Some(primary_window) = windows.record_mut(primary_window_id) {
        if bounded_complete {
            primary_window.request_close();
        } else {
            primary_window.approve_close();
        }
    }
    Ok(())
}

fn approve_render_lab_close_system(
    mut windows: ResMut<WindowStateRegistryResource>,
    mut measurement: ResMut<RenderLabMeasurementConfig>,
    history: Res<RenderFrameHistoryState>,
) -> Result<()> {
    complete_render_lab_measurement_if_requested(&mut windows, &mut measurement, &history)
}

fn publish_render_lab_frame_system(
    camera: Res<RenderLabCamera>,
    flow_id: Res<RenderLabFlowId>,
    presentation: Res<engine::PrimaryPresentationMetricsResource>,
    measurement: Res<RenderLabMeasurementConfig>,
    publication: RenderLabFramePublicationResources<'_>,
) -> Result<()> {
    let RenderLabFramePublicationResources {
        mut targets,
        mut frame_requests,
        mut contributions,
        fixed_quality_plans,
    } = publication;
    let (width, height) = render_lab_radiance_extent(&presentation, &measurement)?;
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
        target_key: target_key.clone(),
    };

    let output_size = render_lab_extent(&presentation);
    if measurement.quality_capture_output_dir.is_some() && (width, height) != output_size {
        let scene_plan = fixed_quality_plans
            .scene
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("temporal quality scene plan is unavailable"))?;
        let resolve_plan = fixed_quality_plans
            .resolve
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("temporal quality resolve plan is unavailable"))?;
        let mut fixed = engine::plugins::render::RenderFixedResolutionExecutionRequest::new(
            producer_id,
            RenderSurfaceId::primary(),
            flow_id.0,
            engine::plugins::render::RenderTargetAliasKey::new(RL2_QUALITY_COLOR_ALIAS)?,
            (width, height),
        )
        .prepare_against_compiled_flows(output_size, scene_plan, resolve_plan)?;
        fixed.scene_invocation = fixed
            .scene_invocation
            .clone()
            .bind_dynamic_texture_alias(RL2_RADIANCE_ALIAS, target_key)?;
        return stage_render_lab_fixed_quality_publication(
            &mut targets,
            &mut frame_requests,
            &mut contributions,
            producer_id,
            target,
            fixed,
            contribution,
        );
    }

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

fn render_lab_radiance_extent(
    presentation: &engine::PrimaryPresentationMetricsResource,
    measurement: &RenderLabMeasurementConfig,
) -> Result<(u32, u32)> {
    let output = render_lab_extent(presentation);
    let Some(radiance) = measurement.radiance_target_size_px else {
        return Ok(output);
    };
    if radiance.0 > output.0 || radiance.1 > output.1 {
        bail!(
            "RL2 measurement radiance extent {}x{} exceeds realized output extent {}x{}",
            radiance.0,
            radiance.1,
            output.0,
            output.1
        );
    }
    if u64::from(radiance.0) * u64::from(output.1) != u64::from(radiance.1) * u64::from(output.0) {
        bail!(
            "RL2 measurement radiance extent {}x{} must preserve realized output aspect {}x{}",
            radiance.0,
            radiance.1,
            output.0,
            output.1
        );
    }
    Ok(radiance)
}

/// Validate every RL2 publication against cloned registries before replacing any live product
/// state. The registries retain their other producers, while a failed replacement leaves the
/// previous complete frame request intact.
fn stage_render_lab_fixed_quality_publication(
    targets: &mut RenderDynamicTextureTargetRequestRegistryResource,
    frame_requests: &mut PreparedRenderFrameRequestResource,
    contributions: &mut RenderDeterministicFrameContributionResource,
    producer_id: engine::plugins::render::RenderFrameProducerId,
    radiance_target: RenderDynamicTextureTargetDescriptor,
    fixed: engine::plugins::render::PreparedFixedResolutionExecution,
    contribution: RenderDeterministicFrameContribution,
) -> Result<()> {
    let mut staged_targets = targets.clone();
    staged_targets.replace_surface_contribution(
        producer_id,
        fixed.render_surface_id,
        [radiance_target, fixed.dynamic_target.clone()],
    )?;

    let mut staged_frame_requests = frame_requests.clone();
    staged_frame_requests.replace_surface_contribution_with_automatic_main_replacements(
        producer_id,
        fixed.render_surface_id,
        [fixed.internal_view.clone()],
        [
            fixed.scene_invocation.clone(),
            fixed.resolve_invocation.clone(),
        ],
        [fixed.automatic_main_replacement],
    )?;

    let mut staged_contributions = contributions.clone();
    staged_contributions.replace(contribution);

    *targets = staged_targets;
    *frame_requests = staged_frame_requests;
    *contributions = staged_contributions;
    Ok(())
}

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
        let measurement = RenderLabMeasurementConfig::default();
        assert!(measurement.output_path.is_none());
        assert!(measurement.submitted_frame_limit.is_none());
        assert!(measurement.primary_window_size_px.is_none());
        assert!(measurement.radiance_target_size_px.is_none());
        assert!(!measurement.completed);
        assert!(!RenderFrameObservationPolicyResource::default().enabled);
    }

    #[test]
    fn measurement_frame_limit_must_be_positive_and_retainable() {
        assert_eq!(validate_measurement_frame_limit(None).unwrap(), None);
        assert_eq!(validate_measurement_frame_limit(Some(1)).unwrap(), Some(1));
        assert_eq!(
            validate_measurement_frame_limit(Some(RL2_MEASUREMENT_HISTORY_CAPACITY)).unwrap(),
            Some(RL2_MEASUREMENT_HISTORY_CAPACITY)
        );
        assert!(validate_measurement_frame_limit(Some(0)).is_err());
        assert!(
            validate_measurement_frame_limit(Some(RL2_MEASUREMENT_HISTORY_CAPACITY + 1)).is_err()
        );
    }

    #[test]
    fn measurement_window_size_must_be_positive_when_requested() {
        assert_eq!(validate_measurement_window_size(None).unwrap(), None);
        assert_eq!(
            validate_measurement_window_size(Some((1600, 1200))).unwrap(),
            Some((1600, 1200))
        );
        assert!(validate_measurement_window_size(Some((0, 1200))).is_err());
        assert!(validate_measurement_window_size(Some((1600, 0))).is_err());
    }

    #[test]
    fn measurement_radiance_size_must_be_positive_when_requested() {
        assert_eq!(validate_measurement_radiance_size(None).unwrap(), None);
        assert_eq!(
            validate_measurement_radiance_size(Some((1280, 720))).unwrap(),
            Some((1280, 720))
        );
        assert!(validate_measurement_radiance_size(Some((0, 720))).is_err());
        assert!(validate_measurement_radiance_size(Some((1280, 0))).is_err());
    }

    #[test]
    fn bounded_measurement_completion_counts_retained_submitted_frames_once() {
        use engine::plugins::render::inspect::RenderGpuTimingCapability;

        let policy = rl2_measurement_policy();
        let mut history = RenderFrameHistoryState::default();
        let mut measurement = RenderLabMeasurementConfig {
            output_path: None,
            submitted_frame_limit: Some(2),
            primary_window_size_px: None,
            radiance_target_size_px: None,
            quality_capture_output_dir: None,
            completed: false,
        };
        assert!(!bounded_measurement_complete(&measurement, &history));

        for frame_index in 1..=2 {
            history.observe_submitted_frame(
                policy,
                frame_index,
                RenderSurfaceId::primary().raw(),
                frame_index + 100,
                (1600, 1200),
                0.0,
                Default::default(),
                &[],
                RenderGpuTimingCapability::Unsupported,
            );
            assert_eq!(
                bounded_measurement_complete(&measurement, &history),
                frame_index == 2
            );
        }

        measurement.completed = true;
        assert!(!bounded_measurement_complete(&measurement, &history));
    }

    #[test]
    fn bounded_measurement_completion_uses_the_product_close_path_once() {
        use engine::plugins::render::inspect::RenderGpuTimingCapability;

        let policy = rl2_measurement_policy();
        let mut history = RenderFrameHistoryState::default();
        history.observe_submitted_frame(
            policy,
            1,
            RenderSurfaceId::primary().raw(),
            101,
            (1600, 1200),
            0.0,
            Default::default(),
            &[],
            RenderGpuTimingCapability::Unsupported,
        );
        let mut measurement = RenderLabMeasurementConfig {
            output_path: None,
            submitted_frame_limit: Some(1),
            primary_window_size_px: None,
            radiance_target_size_px: None,
            quality_capture_output_dir: None,
            completed: false,
        };
        let mut windows = WindowStateRegistryResource::default();
        let primary = windows.register_primary_window("RL2", (1600, 1200), 1.0, true);

        complete_render_lab_measurement_if_requested(&mut windows, &mut measurement, &history)
            .unwrap();

        assert!(measurement.completed);
        let primary_record = windows.record(primary).expect("primary window");
        assert!(primary_record.close_requested);
        assert!(!primary_record.close_intent_pending);
        assert!(!bounded_measurement_complete(&measurement, &history));

        complete_render_lab_measurement_if_requested(&mut windows, &mut measurement, &history)
            .unwrap();
        assert!(measurement.completed);
        assert!(
            windows
                .record(primary)
                .expect("primary window")
                .close_requested
        );
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

        let measurement = RenderLabMeasurementConfig {
            radiance_target_size_px: Some((1280, 720)),
            ..Default::default()
        };
        let artifact = build_measurement_artifact(&history, &measurement);
        assert_eq!(artifact.metric, "gpu_composed_frame_ms");
        assert_eq!(artifact.samples.len(), 2);
        assert_eq!(artifact.samples[0].frame_index, 21);
        assert_eq!(
            artifact.samples[0].render_surface_id,
            RenderSurfaceId::primary().raw()
        );
        assert_eq!(artifact.samples[0].prepare_epoch, 121);
        assert_eq!(artifact.samples[0].target_size_px, [1600, 1200]);
        assert_eq!(artifact.samples[0].radiance_target_size_px, [1280, 720]);
        assert_eq!(artifact.samples[0].gpu_composed_frame_ms, Some(6.25));
        assert_eq!(artifact.samples[1].frame_index, 22);
        assert_eq!(
            artifact.samples[1].render_surface_id,
            RenderSurfaceId::primary().raw()
        );
        assert_eq!(artifact.samples[1].prepare_epoch, 122);
        assert_eq!(artifact.samples[1].target_size_px, [3024, 1964]);
        assert_eq!(artifact.samples[1].radiance_target_size_px, [1280, 720]);
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

    #[test]
    fn render_lab_radiance_extent_defaults_to_output_and_accepts_fixed_measurement_override() {
        let presentation = engine::PrimaryPresentationMetricsResource::new((1920, 1080), 1.0);
        let coupled = RenderLabMeasurementConfig::default();
        assert_eq!(
            render_lab_radiance_extent(&presentation, &coupled).unwrap(),
            (1920, 1080)
        );

        let fixed = RenderLabMeasurementConfig {
            radiance_target_size_px: Some((1280, 720)),
            ..Default::default()
        };
        assert_eq!(
            render_lab_radiance_extent(&presentation, &fixed).unwrap(),
            (1280, 720)
        );
        assert_eq!(render_lab_extent(&presentation), (1920, 1080));
    }

    #[test]
    fn fixed_radiance_measurement_rejects_aspect_mismatch_and_supersampling() {
        let presentation = engine::PrimaryPresentationMetricsResource::new((1920, 1080), 1.0);
        let mismatched = RenderLabMeasurementConfig {
            radiance_target_size_px: Some((1600, 1200)),
            ..Default::default()
        };
        assert!(render_lab_radiance_extent(&presentation, &mismatched).is_err());

        let supersampled = RenderLabMeasurementConfig {
            radiance_target_size_px: Some((2560, 1440)),
            ..Default::default()
        };
        assert!(render_lab_radiance_extent(&presentation, &supersampled).is_err());
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
    fn fixed_quality_flow_is_alias_driven_and_admits_production_fixed_execution() {
        let scene = render_lab_fixed_quality_flow().expect("quality flow should author");
        let resolve =
            engine::plugins::render::fixed_resolution_resolve_flow().expect("resolve flow");
        let scene_plan = engine::plugins::render::compile_flow_plan(&scene)
            .expect("quality flow should compile");
        let resolve_plan = engine::plugins::render::compile_flow_plan(&resolve)
            .expect("resolve flow should compile");

        let prepared = engine::plugins::render::RenderFixedResolutionExecutionRequest::new(
            producer(RL2_PRODUCER_ID),
            RenderSurfaceId::primary(),
            scene.id(),
            engine::plugins::render::RenderTargetAliasKey::new(RL2_QUALITY_COLOR_ALIAS)
                .expect("quality alias"),
            (1280, 720),
        )
        .prepare_against_compiled_flows((1920, 1080), &scene_plan, &resolve_plan)
        .expect("quality flow should admit fixed execution");

        let radiance_key = RenderDynamicTextureTargetKey::new(RL2_TARGET_NAMESPACE, RL2_TARGET_ID);
        let scene_invocation = prepared
            .scene_invocation
            .clone()
            .bind_dynamic_texture_alias(RL2_RADIANCE_ALIAS, radiance_key.clone())
            .expect("radiance alias should bind");

        assert_eq!(prepared.output_size, (1920, 1080));
        assert_eq!(prepared.internal_size, (1280, 720));
        assert_eq!(prepared.internal_view.target_size_px, (1280, 720));
        assert_eq!(
            scene_invocation.target_alias_bindings.get(
                &engine::plugins::render::RenderTargetAliasKey::new(RL2_RADIANCE_ALIAS)
                    .expect("radiance alias")
            ),
            Some(&engine::plugins::render::PreparedTargetBinding::DynamicTexture(radiance_key))
        );
    }

    #[test]
    fn fixed_quality_publication_is_surface_scoped_and_atomic() {
        let scene = render_lab_fixed_quality_flow().expect("quality flow should author");
        let resolve =
            engine::plugins::render::fixed_resolution_resolve_flow().expect("resolve flow");
        let scene_plan =
            engine::plugins::render::compile_flow_plan(&scene).expect("quality flow should compile");
        let resolve_plan =
            engine::plugins::render::compile_flow_plan(&resolve).expect("resolve flow should compile");
        let producer_id = producer(RL2_PRODUCER_ID);
        let fixed = engine::plugins::render::RenderFixedResolutionExecutionRequest::new(
            producer_id,
            RenderSurfaceId::primary(),
            scene.id(),
            engine::plugins::render::RenderTargetAliasKey::new(RL2_QUALITY_COLOR_ALIAS)
                .expect("quality alias"),
            (1280, 720),
        )
        .prepare_against_compiled_flows((1920, 1080), &scene_plan, &resolve_plan)
        .expect("quality flow should admit fixed execution");

        let radiance_key =
            RenderDynamicTextureTargetKey::new(RL2_TARGET_NAMESPACE, RL2_TARGET_ID);
        let radiance_target = RenderDynamicTextureTargetDescriptor::new(
            radiance_key.clone(),
            1280,
            720,
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
        let scene_invocation = fixed
            .scene_invocation
            .clone()
            .bind_dynamic_texture_alias(RL2_RADIANCE_ALIAS, radiance_key.clone())
            .expect("radiance alias should bind");
        let fixture = founding_fixture_with_observation_and_extent(
            RenderAffineTransform3::identity(),
            1280,
            720,
        )
        .expect("fixture");
        let contribution = RenderDeterministicFrameContribution {
            producer_id,
            render_surface_id: RenderSurfaceId::primary(),
            scene: fixture.scene,
            request: fixture.request,
            semantic_inputs: fixture.semantic_inputs,
            availability: fixture.availability,
            output_index: 0,
            target_key: radiance_key,
        };
        let fixed_target_key = fixed.target_key.clone();
        let fixed_view_id = fixed.internal_view.view_id.clone();
        let resolve_invocation_id = fixed.resolve_invocation.invocation_id.clone();

        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        let mut frame_requests = PreparedRenderFrameRequestResource::default();
        let mut contributions = RenderDeterministicFrameContributionResource::default();
        stage_render_lab_fixed_quality_publication(
            &mut targets,
            &mut frame_requests,
            &mut contributions,
            producer_id,
            radiance_target,
            fixed,
            contribution,
        )
        .expect("fixed quality publication");

        let primary_targets = targets.snapshot_for_surface(RenderSurfaceId::primary());
        assert_eq!(primary_targets.len(), 2);
        assert!(
            primary_targets
                .iter()
                .any(|target| target.key == fixed_target_key)
        );
        assert!(
            frame_requests
                .requested_views_for_surface(RenderSurfaceId::primary())
                .iter()
                .any(|view| view.view_id == fixed_view_id)
        );
        assert!(
            frame_requests
                .requested_flow_invocations_for_surface(RenderSurfaceId::primary())
                .iter()
                .any(|invocation| invocation.invocation_id == resolve_invocation_id)
        );
        assert!(
            frame_requests.replaces_automatic_main_flow(RenderSurfaceId::primary(), scene.id())
        );
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
