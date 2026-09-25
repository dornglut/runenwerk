use super::*;
use engine::plugins::render::inspect::{
    RenderCaptureTerminalCode, inspect_fixed_resolution_execution,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, runen_ecs::Resource)]
pub(super) struct RenderLabFixedQualityPlans {
    pub(super) scene: Option<engine::plugins::render::CompiledRenderFlowPlan>,
    pub(super) resolve: Option<engine::plugins::render::CompiledRenderFlowPlan>,
}

const RL2_QUALITY_EXECUTION_HISTORY_CAPACITY: usize = RL2_MEASUREMENT_HISTORY_CAPACITY;

#[derive(Debug, Clone, Default, runen_ecs::Resource)]
pub(super) struct RenderLabTemporalQualityExecutionState {
    pub(super) pending_admission:
        Option<engine::plugins::render::RenderFixedResolutionExecutionAdmission>,
    by_frame: BTreeMap<u64, RenderLabTemporalQualityExecutionEvidence>,
}

impl RenderLabTemporalQualityExecutionState {
    fn observe(&mut self, evidence: RenderLabTemporalQualityExecutionEvidence) {
        self.by_frame.insert(evidence.frame_index, evidence);
        while self.by_frame.len() > RL2_QUALITY_EXECUTION_HISTORY_CAPACITY {
            let Some(oldest) = self.by_frame.keys().next().copied() else {
                break;
            };
            self.by_frame.remove(&oldest);
        }
    }

    fn frame(&self, frame_index: u64) -> Option<&RenderLabTemporalQualityExecutionEvidence> {
        self.by_frame.get(&frame_index)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct RenderLabTemporalQualityExecutionEvidence {
    frame_index: u64,
    prepare_epoch: u64,
    policy: &'static str,
    internal_size_px: [u32; 2],
    output_size_px: [u32; 2],
    native_fallback_active: bool,
    native_fallback_reason: Option<String>,
    target_key: Option<String>,
    internal_view_id: Option<String>,
    scene_invocation_id: Option<String>,
    resolve_invocation_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub(super) struct RenderLabTemporalQualityCaptureEvidence {
    frame_index: u64,
    flow_id: String,
    pass_id: String,
    resource_id: String,
    terminal: &'static str,
    artifact_path: String,
    artifact_blake3: String,
    artifact_manifest_path: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct RenderLabTemporalQualityArtifact {
    schema_version: u32,
    scenario_id: &'static str,
    scenario_revision: u32,
    source_git_revision: Option<String>,
    requested_internal_size_px: [u32; 2],
    requested_output_size_px: [u32; 2],
    execution: RenderLabTemporalQualityExecutionEvidence,
    capture: RenderLabTemporalQualityCaptureEvidence,
}

const RL2_QUALITY_SCENARIO_ID: &str = "runenwerk.render_lab.rl2.temporal_quality";
const RL2_QUALITY_SCENARIO_REVISION: u32 = 1;
pub(super) const RL2_QUALITY_FLOW_ID: &str = "runenwerk.render_lab.rl2.fixed_quality";
pub(super) const RL2_QUALITY_PASS_ID: &str = "runenwerk.render_lab.rl2.fixed_quality.compose";
pub(super) const RL2_QUALITY_COLOR_ALIAS: &str = "runenwerk.render_lab.rl2.fixed_quality.color";

pub(super) fn render_lab_fixed_quality_flow() -> Result<RenderFlow> {
    RenderFlow::new(RL2_QUALITY_FLOW_ID)
        .with_target_alias(RL2_RADIANCE_ALIAS, RenderTargetAliasKind::Texture)?
        .with_color_target_alias(RL2_QUALITY_COLOR_ALIAS)?
        .fullscreen_pass(RL2_QUALITY_PASS_ID)
        .shader_asset("assets/shaders/runenwerk_render_lab_radiance.wgsl")
        .sample_texture_load(runen_gpu::GpuBindingKey::try_new(0, 0)?, RL2_RADIANCE_ALIAS)
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_target_alias(RL2_QUALITY_COLOR_ALIAS)
        .finish()
        .validate()
}

pub(super) fn temporal_quality_capture_evidence(
    report_state: &RenderDebugFrameReportState,
) -> Result<Option<RenderLabTemporalQualityCaptureEvidence>> {
    let Some(report) = report_state.latest.as_ref() else {
        return Ok(None);
    };
    let Some(result) = report.capture_results.first() else {
        return Ok(None);
    };
    if result.terminal.code != RenderCaptureTerminalCode::Completed {
        let reason = result
            .terminal
            .reason
            .as_ref()
            .map(|reason| format!("{}: {}", reason.code, reason.detail))
            .unwrap_or_else(|| "no terminal reason".to_string());
        bail!(
            "temporal quality capture for frame {} terminated as {} ({reason})",
            report.frame_index,
            result.terminal.code.as_str()
        );
    }
    let artifact_path = result.artifact_path.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "temporal quality capture for frame {} completed without an exported artifact path",
            report.frame_index
        )
    })?;
    let bytes = fs::read(artifact_path).with_context(|| {
        format!(
            "read temporal quality capture artifact {}",
            artifact_path.display()
        )
    })?;
    let frame_index = result
        .frame_identity
        .as_ref()
        .map(|identity| identity.frame_index)
        .unwrap_or(report.frame_index);

    Ok(Some(RenderLabTemporalQualityCaptureEvidence {
        frame_index,
        flow_id: result.capture_point.flow_id.clone(),
        pass_id: result.capture_point.pass_id.clone(),
        resource_id: result.capture_point.resource_id.clone(),
        terminal: result.terminal.code.as_str(),
        artifact_path: artifact_path.to_string_lossy().into_owned(),
        artifact_blake3: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        artifact_manifest_path: report
            .artifact_manifest_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
    }))
}

pub(super) fn write_temporal_quality_artifact(
    measurement: &RenderLabMeasurementConfig,
    quality_execution: &RenderLabTemporalQualityExecutionState,
    capture: RenderLabTemporalQualityCaptureEvidence,
) -> Result<()> {
    let capture_root = measurement
        .quality_capture_output_dir
        .as_deref()
        .ok_or_else(|| {
            anyhow::anyhow!("temporal quality capture output directory is unavailable")
        })?;
    let output_root = capture_root.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "temporal quality capture directory {} has no evidence output parent",
            capture_root.display()
        )
    })?;
    let requested_internal = measurement.radiance_target_size_px.ok_or_else(|| {
        anyhow::anyhow!("temporal quality requested internal extent is unavailable")
    })?;
    let requested_output = measurement.primary_window_size_px.ok_or_else(|| {
        anyhow::anyhow!("temporal quality requested output extent is unavailable")
    })?;
    let execution = quality_execution
        .frame(capture.frame_index)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "temporal quality capture frame {} has no correlated prepared-frame execution evidence",
                capture.frame_index
            )
        })?;
    let artifact = RenderLabTemporalQualityArtifact {
        schema_version: 1,
        scenario_id: RL2_QUALITY_SCENARIO_ID,
        scenario_revision: RL2_QUALITY_SCENARIO_REVISION,
        source_git_revision: std::env::var("RUNENWERK_SOURCE_REVISION").ok(),
        requested_internal_size_px: [requested_internal.0, requested_internal.1],
        requested_output_size_px: [requested_output.0, requested_output.1],
        execution,
        capture,
    };
    let path = output_root.join("quality-evidence.json");
    fs::create_dir_all(output_root).with_context(|| {
        format!(
            "create temporal quality evidence directory {}",
            output_root.display()
        )
    })?;
    let bytes = serde_json::to_vec_pretty(&artifact)
        .context("serialize temporal quality execution and capture evidence")?;
    fs::write(&path, bytes)
        .with_context(|| format!("write temporal quality evidence {}", path.display()))
}

pub(super) fn inspect_render_lab_temporal_quality_execution_system(
    measurement: Res<RenderLabMeasurementConfig>,
    flow_id: Res<RenderLabFlowId>,
    prepared_frames: Res<engine::plugins::render::PreparedRenderFrameResource>,
    mut quality_execution: ResMut<RenderLabTemporalQualityExecutionState>,
) -> Result<()> {
    if measurement.quality_capture_output_dir.is_none() {
        return Ok(());
    }

    let Some(frame) = prepared_frames.frame() else {
        return Ok(());
    };
    let output_size = measurement
        .primary_window_size_px
        .ok_or_else(|| anyhow::anyhow!("temporal quality output extent is unavailable"))?;
    let internal_size = measurement
        .radiance_target_size_px
        .ok_or_else(|| anyhow::anyhow!("temporal quality internal extent is unavailable"))?;

    let evidence = if let Some(admission) = quality_execution.pending_admission.as_ref() {
        let fixed = inspect_fixed_resolution_execution(admission, frame)?;
        let policy = match fixed.resolution.policy {
            engine::plugins::render::inspect::RenderTemporalResolutionPolicy::Native => "native",
            engine::plugins::render::inspect::RenderTemporalResolutionPolicy::Fixed => "fixed",
            engine::plugins::render::inspect::RenderTemporalResolutionPolicy::Dynamic {
                ..
            } => "dynamic",
        };
        RenderLabTemporalQualityExecutionEvidence {
            frame_index: frame.context.frame_index,
            prepare_epoch: frame.context.prepare_epoch,
            policy,
            internal_size_px: fixed.resolution.internal_size,
            output_size_px: fixed.resolution.output_size,
            native_fallback_active: fixed.native_fallback_active,
            native_fallback_reason: fixed.native_fallback_reason,
            target_key: fixed.target_key.map(|key| key.to_string()),
            internal_view_id: fixed.internal_view_id,
            scene_invocation_id: fixed.scene_invocation_id.map(|id| id.to_string()),
            resolve_invocation_id: fixed.resolve_invocation_id.map(|id| id.to_string()),
        }
    } else {
        if internal_size != output_size {
            bail!(
                "temporal quality requested {}x{} -> {}x{} without a retained fixed admission",
                internal_size.0,
                internal_size.1,
                output_size.0,
                output_size.1
            );
        }
        if frame.surface.target_size_px != output_size {
            bail!(
                "temporal quality native frame output {}x{} does not match requested {}x{}",
                frame.surface.target_size_px.0,
                frame.surface.target_size_px.1,
                output_size.0,
                output_size.1
            );
        }
        let main_view = frame
            .main_view()
            .ok_or_else(|| anyhow::anyhow!("temporal quality native frame has no main view"))?;
        if main_view.target_size_px != output_size {
            bail!(
                "temporal quality native main view {}x{} does not match requested {}x{}",
                main_view.target_size_px.0,
                main_view.target_size_px.1,
                output_size.0,
                output_size.1
            );
        }
        let scene_invocation = frame
            .flow_invocations_for_flow(flow_id.0)
            .find(|invocation| invocation.view_id == "main")
            .ok_or_else(|| {
                anyhow::anyhow!("temporal quality native frame is missing the RL2 main invocation")
            })?;

        RenderLabTemporalQualityExecutionEvidence {
            frame_index: frame.context.frame_index,
            prepare_epoch: frame.context.prepare_epoch,
            policy: "native",
            internal_size_px: [internal_size.0, internal_size.1],
            output_size_px: [output_size.0, output_size.1],
            native_fallback_active: false,
            native_fallback_reason: None,
            target_key: None,
            internal_view_id: None,
            scene_invocation_id: Some(scene_invocation.invocation_id.to_string()),
            resolve_invocation_id: None,
        }
    };

    quality_execution.observe(evidence);
    Ok(())
}

pub(super) fn stage_render_lab_fixed_quality_publication(
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

pub(super) fn stage_render_lab_native_quality_publication(
    targets: &mut RenderDynamicTextureTargetRequestRegistryResource,
    frame_requests: &mut PreparedRenderFrameRequestResource,
    contributions: &mut RenderDeterministicFrameContributionResource,
    producer_id: engine::plugins::render::RenderFrameProducerId,
    render_surface_id: RenderSurfaceId,
    radiance_target: RenderDynamicTextureTargetDescriptor,
    native_scene_invocation: PreparedFlowInvocationRequest,
    contribution: RenderDeterministicFrameContribution,
) -> Result<()> {
    if contribution.render_surface_id != render_surface_id {
        bail!(
            "temporal quality native fallback contribution surface does not match admitted surface"
        );
    }

    let mut staged_targets = targets.clone();
    staged_targets.replace_surface_contribution(
        producer_id,
        render_surface_id,
        [radiance_target],
    )?;

    let mut staged_frame_requests = frame_requests.clone();
    staged_frame_requests.replace_surface_contribution_with_automatic_main_replacements(
        producer_id,
        render_surface_id,
        [],
        [native_scene_invocation],
        [],
    )?;

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
    use super::super::*;
    use std::{fs, path::PathBuf};

    fn producer(raw: u64) -> engine::plugins::render::RenderFrameProducerId {
        engine::plugins::render::RenderFrameProducerId::try_from_raw(raw)
            .expect("test producer id should be nonzero")
    }

    #[test]
    fn temporal_quality_extent_defers_fixed_policy_rejection_to_renderer_admission() {
        let presentation = engine::PrimaryPresentationMetricsResource::new((1920, 1080), 1.0);
        let quality = RenderLabMeasurementConfig {
            primary_window_size_px: Some((1920, 1080)),
            radiance_target_size_px: Some((1280, 800)),
            quality_capture_output_dir: Some(PathBuf::from("quality-captures")),
            ..Default::default()
        };
        assert_eq!(
            render_lab_radiance_extent(&presentation, &quality)
                .expect("quality mode should preserve the requested extent for admission"),
            (1280, 800)
        );

        let legacy = RenderLabMeasurementConfig {
            primary_window_size_px: Some((1920, 1080)),
            radiance_target_size_px: Some((1280, 800)),
            ..Default::default()
        };
        assert!(
            render_lab_radiance_extent(&presentation, &legacy).is_err(),
            "legacy radiance measurement keeps its own same-aspect validation"
        );
    }

    #[test]
    fn temporal_quality_execution_history_is_bounded_and_frame_addressable() {
        let mut state = RenderLabTemporalQualityExecutionState::default();
        for frame_index in 0..=RL2_QUALITY_EXECUTION_HISTORY_CAPACITY as u64 {
            state.observe(RenderLabTemporalQualityExecutionEvidence {
                frame_index,
                prepare_epoch: frame_index + 100,
                policy: "fixed",
                internal_size_px: [1280, 720],
                output_size_px: [1920, 1080],
                native_fallback_active: false,
                native_fallback_reason: None,
                target_key: Some(format!("target-{frame_index}")),
                internal_view_id: Some(format!("view-{frame_index}")),
                scene_invocation_id: Some(format!("scene-{frame_index}")),
                resolve_invocation_id: Some(format!("resolve-{frame_index}")),
            });
        }

        assert!(state.frame(0).is_none());
        assert!(
            state
                .frame(RL2_QUALITY_EXECUTION_HISTORY_CAPACITY as u64)
                .is_some()
        );
        assert_eq!(state.by_frame.len(), RL2_QUALITY_EXECUTION_HISTORY_CAPACITY);
    }

    #[test]
    fn temporal_quality_capture_evidence_hashes_the_exported_frame() {
        use engine::plugins::render::inspect::{
            RenderCaptureIdentity, RenderCaptureSelector, RenderCaptureSelectorResult,
            RenderCaptureTerminal, RenderDebugFrameReport,
        };

        let artifact_path = std::env::temp_dir().join(format!(
            "runenwerk-render-lab-quality-capture-{}.png",
            std::process::id()
        ));
        fs::write(&artifact_path, b"quality-capture-bytes").expect("test capture should write");
        let selector = RenderCaptureSelector::named_pass_surface_color("quality.flow", "resolve");
        let capture_point = selector.stable_point_fallback();
        let identity = RenderCaptureIdentity {
            frame_index: 42,
            pass_label: "resolve".to_string(),
            capture_point: capture_point.clone(),
        };
        let mut report_state = RenderDebugFrameReportState::default();
        report_state.observe_frame(RenderDebugFrameReport {
            frame_index: 42,
            capture_results: vec![RenderCaptureSelectorResult {
                selector_index: 0,
                selector,
                capture_point,
                frame_identity: Some(identity),
                terminal: RenderCaptureTerminal::completed(),
                artifact_path: Some(artifact_path.clone()),
            }],
            ..RenderDebugFrameReport::default()
        });

        let evidence = temporal_quality_capture_evidence(&report_state)
            .expect("capture evidence should inspect")
            .expect("completed capture should produce evidence");
        assert_eq!(evidence.frame_index, 42);
        assert_eq!(evidence.terminal, "completed");
        assert_eq!(
            evidence.artifact_blake3,
            format!("blake3:{}", blake3::hash(b"quality-capture-bytes").to_hex())
        );

        let _ = fs::remove_file(artifact_path);
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
    fn native_quality_reference_uses_the_same_alias_driven_scene_flow() {
        let scene = render_lab_fixed_quality_flow().expect("quality flow should author");
        let producer_id = producer(RL2_PRODUCER_ID);
        let camera = RenderLabCamera::default();
        let (target_key, target, contribution) =
            build_render_lab_radiance_publication(&camera, producer_id, (1920, 1080))
                .expect("native quality radiance publication should build");
        let invocation = PreparedFlowInvocationRequest::new(
            format!("{RL2_QUALITY_FLOW_ID}.native"),
            scene.id(),
            "main",
        )
        .bind_dynamic_texture_alias(RL2_RADIANCE_ALIAS, target_key.clone())
        .expect("native quality radiance alias should bind")
        .bind_surface_color_alias(RL2_QUALITY_COLOR_ALIAS)
        .expect("native quality color alias should bind");

        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        let mut frame_requests = PreparedRenderFrameRequestResource::default();
        let mut contributions = RenderDeterministicFrameContributionResource::default();
        stage_render_lab_native_quality_publication(
            &mut targets,
            &mut frame_requests,
            &mut contributions,
            producer_id,
            RenderSurfaceId::primary(),
            target,
            invocation.clone(),
            contribution,
        )
        .expect("native quality publication should remain atomic");

        let published_targets = targets.snapshot_for_surface(RenderSurfaceId::primary());
        assert_eq!(published_targets.len(), 1);
        assert_eq!(published_targets[0].key, target_key);
        assert_eq!(
            (published_targets[0].width, published_targets[0].height),
            (1920, 1080)
        );
        assert!(
            frame_requests
                .requested_views_for_surface(RenderSurfaceId::primary())
                .is_empty(),
            "native quality reference must not publish a fixed internal view"
        );
        let invocations =
            frame_requests.requested_flow_invocations_for_surface(RenderSurfaceId::primary());
        assert_eq!(invocations.len(), 1);
        assert_eq!(invocations[0].flow_id, scene.id());
        assert_eq!(invocations[0].view_id, "main");
        assert_eq!(
            invocations[0].target_alias_bindings.get(
                &engine::plugins::render::RenderTargetAliasKey::new(RL2_QUALITY_COLOR_ALIAS)
                    .expect("quality color alias")
            ),
            Some(&engine::plugins::render::PreparedTargetBinding::SurfaceColor)
        );
        assert!(
            !frame_requests.replaces_automatic_main_flow(RenderSurfaceId::primary(), scene.id()),
            "native quality reference must not claim fixed automatic-main replacement"
        );
    }

    #[test]
    fn fixed_quality_publication_is_surface_scoped_and_atomic() {
        let scene = render_lab_fixed_quality_flow().expect("quality flow should author");
        let resolve =
            engine::plugins::render::fixed_resolution_resolve_flow().expect("resolve flow");
        let scene_plan = engine::plugins::render::compile_flow_plan(&scene)
            .expect("quality flow should compile");
        let resolve_plan = engine::plugins::render::compile_flow_plan(&resolve)
            .expect("resolve flow should compile");
        let producer_id = producer(RL2_PRODUCER_ID);
        let mut fixed = engine::plugins::render::RenderFixedResolutionExecutionRequest::new(
            producer_id,
            RenderSurfaceId::primary(),
            scene.id(),
            engine::plugins::render::RenderTargetAliasKey::new(RL2_QUALITY_COLOR_ALIAS)
                .expect("quality alias"),
            (1280, 720),
        )
        .prepare_against_compiled_flows((1920, 1080), &scene_plan, &resolve_plan)
        .expect("quality flow should admit fixed execution");

        let radiance_key = RenderDynamicTextureTargetKey::new(RL2_TARGET_NAMESPACE, RL2_TARGET_ID);
        fixed.scene_invocation = fixed
            .scene_invocation
            .clone()
            .bind_dynamic_texture_alias(RL2_RADIANCE_ALIAS, radiance_key.clone())
            .expect("radiance alias should bind");
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
    fn fixed_quality_native_fallback_discards_all_partial_fixed_state() {
        let scene = render_lab_fixed_quality_flow().expect("quality flow should author");
        let resolve =
            engine::plugins::render::fixed_resolution_resolve_flow().expect("resolve flow");
        let scene_plan = engine::plugins::render::compile_flow_plan(&scene)
            .expect("quality flow should compile");
        let resolve_plan = engine::plugins::render::compile_flow_plan(&resolve)
            .expect("resolve flow should compile");
        let producer_id = producer(RL2_PRODUCER_ID);
        let admission = engine::plugins::render::RenderFixedResolutionExecutionRequest::new(
            producer_id,
            RenderSurfaceId::primary(),
            scene.id(),
            engine::plugins::render::RenderTargetAliasKey::new(RL2_QUALITY_COLOR_ALIAS)
                .expect("quality alias"),
            (1280, 800),
        )
        .admit_against_compiled_flows((1920, 1080), &scene_plan, &resolve_plan);
        let engine::plugins::render::RenderFixedResolutionExecutionAdmission::NativeFallback(
            fallback,
        ) = &admission
        else {
            panic!("aspect mismatch should produce explicit native fallback");
        };

        let native_scene_invocation = fallback
            .native_scene_invocation
            .clone()
            .expect("valid alias-driven scene should provide native fallback");
        let camera = RenderLabCamera::default();
        let (target_key, target, contribution) =
            build_render_lab_radiance_publication(&camera, producer_id, (1920, 1080))
                .expect("native fallback radiance publication should build");
        let native_scene_invocation = native_scene_invocation
            .bind_dynamic_texture_alias(RL2_RADIANCE_ALIAS, target_key.clone())
            .expect("native fallback radiance alias should bind");

        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        let mut frame_requests = PreparedRenderFrameRequestResource::default();
        let mut contributions = RenderDeterministicFrameContributionResource::default();
        stage_render_lab_native_quality_publication(
            &mut targets,
            &mut frame_requests,
            &mut contributions,
            producer_id,
            fallback.render_surface_id,
            target,
            native_scene_invocation.clone(),
            contribution,
        )
        .expect("native fallback publication should remain atomic");

        let published_targets = targets.snapshot_for_surface(RenderSurfaceId::primary());
        assert_eq!(published_targets.len(), 1);
        assert_eq!(published_targets[0].key, target_key);
        assert_eq!(
            (published_targets[0].width, published_targets[0].height),
            (1920, 1080)
        );
        assert!(
            frame_requests
                .requested_views_for_surface(RenderSurfaceId::primary())
                .is_empty(),
            "native fallback must not retain the fixed internal view"
        );
        let invocations =
            frame_requests.requested_flow_invocations_for_surface(RenderSurfaceId::primary());
        assert_eq!(invocations.len(), 1);
        assert_eq!(
            invocations[0].invocation_id,
            native_scene_invocation.invocation_id
        );
        assert_eq!(invocations[0].view_id, "main");
        assert!(
            invocations
                .iter()
                .all(|invocation| invocation.flow_id != resolve.id()),
            "native fallback must not publish the fixed resolve invocation"
        );
        assert!(
            !frame_requests.replaces_automatic_main_flow(RenderSurfaceId::primary(), scene.id()),
            "explicit native main invocation must not retain a fixed replacement claim"
        );
    }
}
