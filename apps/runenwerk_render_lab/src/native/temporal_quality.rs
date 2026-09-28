use super::*;
use engine::plugins::render::inspect::{
    RenderCaptureSelector, RenderCaptureTerminalCode, inspect_fixed_resolution_execution,
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
struct RenderLabTemporalQualityGpuEvidence {
    backend: &'static str,
    adapter_class: &'static str,
    software_status: &'static str,
    fallback_status: &'static str,
    diagnostic_name: Option<String>,
    driver: Option<String>,
    driver_info: Option<String>,
    vendor: Option<u32>,
    device: Option<u32>,
    device_request_profile: &'static str,
    device_request_profile_supported: bool,
    evidence_profile_fingerprint: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RenderLabTemporalReconstructionEvidence {
    frame_index: u64,
    requested_size_px: [u32; 2],
    evaluation_size_px: [u32; 2],
    semantic_input_generation_classes: Vec<u64>,
    sequence_revision: u32,
    reconstruction_revision: u32,
    phase: u32,
    history_generation: u64,
    history_age: u32,
    history_reset: bool,
}

#[derive(Debug, serde::Serialize)]
struct RenderLabTemporalQualityArtifact {
    schema_version: u32,
    scenario_id: &'static str,
    scenario_revision: u32,
    source_git_revision: Option<String>,
    requested_internal_size_px: [u32; 2],
    requested_output_size_px: [u32; 2],
    capture_submission_ordinal: usize,
    total_submitted_frames: usize,
    gpu: RenderLabTemporalQualityGpuEvidence,
    execution: RenderLabTemporalQualityExecutionEvidence,
    temporal_reconstruction: Vec<RenderLabTemporalReconstructionEvidence>,
    capture_route: &'static str,
    capture: RenderLabTemporalQualityCaptureEvidence,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RenderLabCameraMotionFrameEvidence {
    frame_index: u64,
    requested_size_px: [u32; 2],
    evaluation_size_px: [u32; 2],
    semantic_input_generation_classes: Vec<u64>,
    phase: u32,
    history_generation: u64,
    history_age: u32,
    history_reset: bool,
    camera_reprojection_eligible: bool,
    previous_observation_available: bool,
    camera_pose_changed: bool,
    camera_reprojection_revision: Option<u32>,
    depth_policy_revision: Option<u32>,
}

#[derive(Debug, serde::Serialize)]
struct RenderLabCameraMotionArtifact {
    schema_version: u32,
    scenario_id: &'static str,
    scenario_revision: u32,
    source_git_revision: Option<String>,
    requested_output_size_px: [u32; 2],
    capture_submission_ordinal: usize,
    total_submitted_frames: usize,
    gpu: RenderLabTemporalQualityGpuEvidence,
    execution: RenderLabTemporalQualityExecutionEvidence,
    frames: Vec<RenderLabCameraMotionFrameEvidence>,
    capture_route: &'static str,
    capture: RenderLabTemporalQualityCaptureEvidence,
}

const RL2_QUALITY_SCHEMA_VERSION: u32 = 5;
const RL2_QUALITY_SCENARIO_ID: &str = "runenwerk.render_lab.rl2.temporal_quality";
const RL2_QUALITY_SCENARIO_REVISION: u32 = 5;
const RL2_CAMERA_MOTION_SCHEMA_VERSION: u32 = 1;
const RL2_CAMERA_MOTION_SCENARIO_ID: &str = "runenwerk.render_lab.rl2.camera_motion_p100";
const RL2_CAMERA_MOTION_SCENARIO_REVISION: u32 = 1;
pub(super) const RL2_QUALITY_FLOW_ID: &str = "runenwerk.render_lab.rl2.fixed_quality";
pub(super) const RL2_QUALITY_PASS_ID: &str = "runenwerk.render_lab.rl2.fixed_quality.compose";
pub(super) const RL2_QUALITY_COLOR_ALIAS: &str = "runenwerk.render_lab.rl2.fixed_quality.color";
const RL2_QUALITY_PRESENT_FLOW_ID: &str = "runenwerk.render_lab.rl2.quality_present";
const RL2_QUALITY_PRESENT_PASS_ID: &str = "runenwerk.render_lab.rl2.quality_present.present";

pub(super) fn render_lab_quality_present_flow() -> Result<RenderFlow> {
    RenderFlow::new(RL2_QUALITY_PRESENT_FLOW_ID)
        .with_surface_color()?
        .present_pass(RL2_QUALITY_PRESENT_PASS_ID)?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()
}

fn compiled_surface_color_selector(
    plan: &engine::plugins::render::CompiledRenderFlowPlan,
    pass_label: &str,
) -> Result<RenderCaptureSelector> {
    let pass = plan
        .render_passes
        .iter()
        .find(|pass| pass.pass_label() == pass_label)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "temporal quality capture pass '{pass_label}' is absent from compiled flow '{}'",
                plan.flow_label
            )
        })?;
    Ok(RenderCaptureSelector::named_pass_surface_color(
        plan.flow_id.to_string(),
        pass.pass_id().to_string(),
    ))
}

pub(super) fn temporal_quality_capture_selector(
    admission: Option<&engine::plugins::render::RenderFixedResolutionExecutionAdmission>,
    scene_plan: &engine::plugins::render::CompiledRenderFlowPlan,
    resolve_plan: &engine::plugins::render::CompiledRenderFlowPlan,
) -> Result<RenderCaptureSelector> {
    match admission {
        Some(engine::plugins::render::RenderFixedResolutionExecutionAdmission::Fixed(_)) => {
            compiled_surface_color_selector(
                resolve_plan,
                engine::plugins::render::FIXED_RESOLUTION_RESOLVE_PASS_LABEL,
            )
        }
        Some(engine::plugins::render::RenderFixedResolutionExecutionAdmission::NativeFallback(
            _,
        ))
        | None => compiled_surface_color_selector(scene_plan, RL2_QUALITY_PASS_ID),
    }
}

pub(super) fn render_lab_fixed_quality_flow() -> Result<RenderFlow> {
    RenderFlow::new(RL2_QUALITY_FLOW_ID)
        .with_target_alias(RL2_RADIANCE_ALIAS, RenderTargetAliasKind::Texture)?
        .with_color_target_alias(RL2_QUALITY_COLOR_ALIAS)?
        .fullscreen_pass(RL2_QUALITY_PASS_ID)
        .shader_asset("assets/shaders/runenwerk_render_lab_quality.wgsl")
        .sample_texture_load(runen_gpu::GpuBindingKey::try_new(0, 0)?, RL2_RADIANCE_ALIAS)
        .clear_color([0.0, 0.0, 0.0, 1.0])
        .write_target_alias(RL2_QUALITY_COLOR_ALIAS)
        .finish()
        .validate()
}

pub(super) fn temporal_quality_capture_evidence(
    report_state: &RenderDebugFrameReportState,
    target_frame_index: u64,
) -> Result<Option<RenderLabTemporalQualityCaptureEvidence>> {
    let Some(results) = report_state.capture_results_for_frame(target_frame_index) else {
        return Ok(None);
    };
    let Some(result) = results.first() else {
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
            target_frame_index,
            result.terminal.code.as_str()
        );
    }
    let artifact_path = result.artifact_path.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "temporal quality capture for frame {} completed without an exported artifact path",
            target_frame_index
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
        .unwrap_or(target_frame_index);

    Ok(Some(RenderLabTemporalQualityCaptureEvidence {
        frame_index,
        flow_id: result.capture_point.flow_id.clone(),
        pass_id: result.capture_point.pass_id.clone(),
        resource_id: result.capture_point.resource_id.clone(),
        terminal: result.terminal.code.as_str(),
        artifact_path: artifact_path.to_string_lossy().into_owned(),
        artifact_blake3: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        artifact_manifest_path: report_state
            .capture_artifact_manifest_for_frame(target_frame_index)
            .map(|path| path.to_string_lossy().into_owned()),
    }))
}

#[derive(serde::Serialize)]
struct RenderLabTemporalQualityGpuFingerprintFacts<'a> {
    backend: &'a str,
    adapter_class: &'a str,
    software_status: &'a str,
    fallback_status: &'a str,
    diagnostic_name: Option<&'a str>,
    driver: Option<&'a str>,
    driver_info: Option<&'a str>,
    vendor: Option<u32>,
    device: Option<u32>,
    device_request_profile: &'a str,
    device_request_profile_supported: bool,
}

fn temporal_quality_evidence_profile_fingerprint(
    facts: &RenderLabTemporalQualityGpuFingerprintFacts<'_>,
) -> String {
    let canonical = serde_json::to_vec(facts)
        .expect("fixed temporal-quality adapter evidence tuple must serialize");
    format!("blake3:{}", blake3::hash(&canonical).to_hex())
}

fn temporal_quality_gpu_evidence(
    facts: &runen_gpu::GpuAdapterFacts,
) -> RenderLabTemporalQualityGpuEvidence {
    let backend = gpu_backend_token(facts.backend());
    let adapter_class = gpu_adapter_class_token(facts.class());
    let software_status = gpu_software_token(facts.software());
    let fallback_status = gpu_fallback_token(facts.fallback());
    let device_request_profile = gpu_device_profile_token(facts.device_request_profile());
    let fingerprint_facts = RenderLabTemporalQualityGpuFingerprintFacts {
        backend,
        adapter_class,
        software_status,
        fallback_status,
        diagnostic_name: facts.diagnostic_name(),
        driver: facts.driver(),
        driver_info: facts.driver_info(),
        vendor: facts.vendor(),
        device: facts.device(),
        device_request_profile,
        device_request_profile_supported: facts.device_request_profile_supported(),
    };
    RenderLabTemporalQualityGpuEvidence {
        backend,
        adapter_class,
        software_status,
        fallback_status,
        diagnostic_name: facts.diagnostic_name().map(str::to_owned),
        driver: facts.driver().map(str::to_owned),
        driver_info: facts.driver_info().map(str::to_owned),
        vendor: facts.vendor(),
        device: facts.device(),
        device_request_profile,
        device_request_profile_supported: facts.device_request_profile_supported(),
        evidence_profile_fingerprint: temporal_quality_evidence_profile_fingerprint(
            &fingerprint_facts,
        ),
    }
}

fn gpu_backend_token(value: runen_gpu::GpuBackendFamily) -> &'static str {
    match value {
        runen_gpu::GpuBackendFamily::Vulkan => "vulkan",
        runen_gpu::GpuBackendFamily::Metal => "metal",
        runen_gpu::GpuBackendFamily::Direct3D12 => "direct3d12",
        runen_gpu::GpuBackendFamily::OpenGl => "opengl",
        runen_gpu::GpuBackendFamily::BrowserWebGpu => "browser_webgpu",
        runen_gpu::GpuBackendFamily::UnknownBackend => "unknown",
    }
}

fn gpu_adapter_class_token(value: runen_gpu::GpuAdapterClass) -> &'static str {
    match value {
        runen_gpu::GpuAdapterClass::Discrete => "discrete",
        runen_gpu::GpuAdapterClass::Integrated => "integrated",
        runen_gpu::GpuAdapterClass::Virtual => "virtual",
        runen_gpu::GpuAdapterClass::Cpu => "cpu",
        runen_gpu::GpuAdapterClass::Other => "other",
        runen_gpu::GpuAdapterClass::Unknown => "unknown",
    }
}

fn gpu_software_token(value: runen_gpu::GpuSoftwareStatus) -> &'static str {
    match value {
        runen_gpu::GpuSoftwareStatus::Software => "software",
        runen_gpu::GpuSoftwareStatus::Hardware => "hardware",
        runen_gpu::GpuSoftwareStatus::Unknown => "unknown",
    }
}

fn gpu_fallback_token(value: runen_gpu::GpuFallbackStatus) -> &'static str {
    match value {
        runen_gpu::GpuFallbackStatus::ConfirmedFallback => "confirmed_fallback",
        runen_gpu::GpuFallbackStatus::ConfirmedNotFallback => "confirmed_not_fallback",
        runen_gpu::GpuFallbackStatus::Unknown => "unknown",
    }
}

fn gpu_device_profile_token(value: runen_gpu::GpuDeviceRequestProfile) -> &'static str {
    match value {
        runen_gpu::GpuDeviceRequestProfile::ModernPortable => "modern_portable",
        runen_gpu::GpuDeviceRequestProfile::Downlevel => "downlevel",
        runen_gpu::GpuDeviceRequestProfile::BrowserWebGpu => "browser_webgpu",
        runen_gpu::GpuDeviceRequestProfile::DownlevelWebGl2 => "downlevel_webgl2",
    }
}

pub(super) fn write_temporal_quality_artifact(
    measurement: &RenderLabMeasurementConfig,
    history: &RenderFrameHistoryState,
    quality_execution: &RenderLabTemporalQualityExecutionState,
    capture: RenderLabTemporalQualityCaptureEvidence,
    gfx: &engine::plugins::render::Gfx,
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
    let capture_submission_ordinal = history
        .observations()
        .enumerate()
        .find_map(|(index, observation)| {
            (observation.key.frame_index == capture.frame_index).then_some(index + 1)
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "temporal quality capture frame {} is absent from retained submitted-frame history",
                capture.frame_index
            )
        })?;
    let expected_capture_ordinal = measurement.submitted_frame_limit.unwrap_or(1);
    if capture_submission_ordinal != expected_capture_ordinal {
        bail!(
            "temporal quality capture submission ordinal {} does not match configured target {}",
            capture_submission_ordinal,
            expected_capture_ordinal
        );
    }
    // Source-owner generation tokens are intentionally opaque. For persisted Render Lab evidence,
    // assign only per-representation, run-local equivalence classes in first-seen order. This proves
    // "same generation" versus "changed generation" without exposing source numbering or comparing
    // numeric tokens across distinct representation/source contracts.
    let mut generation_classes = Vec::new();
    let mut temporal_reconstruction = Vec::new();
    for observation in history.observations().take(capture_submission_ordinal) {
        for evidence in gfx.deterministic_temporal_evidence(observation.key.frame_index) {
            let semantic_input_generation_classes = evidence
                .semantic_input_generations
                .iter()
                .map(|(representation_id, generation)| {
                    let representation_index = if let Some(index) = generation_classes
                        .iter()
                        .position(|(known_id, _)| known_id == representation_id)
                    {
                        index
                    } else {
                        generation_classes.push((*representation_id, Vec::new()));
                        generation_classes.len() - 1
                    };
                    let seen = &mut generation_classes[representation_index].1;
                    let class_index =
                        if let Some(index) = seen.iter().position(|known| known == generation) {
                            index
                        } else {
                            seen.push(*generation);
                            seen.len() - 1
                        };
                    u64::try_from(class_index + 1)
                        .expect("bounded temporal quality generation class fits u64")
                })
                .collect();

            temporal_reconstruction.push(RenderLabTemporalReconstructionEvidence {
                frame_index: observation.key.frame_index,
                requested_size_px: [evidence.requested_extent.0, evidence.requested_extent.1],
                evaluation_size_px: [evidence.evaluation_extent.0, evidence.evaluation_extent.1],
                semantic_input_generation_classes,
                sequence_revision: evidence.sequence_revision,
                reconstruction_revision: evidence.reconstruction_revision,
                phase: evidence.phase,
                history_generation: evidence.history_generation,
                history_age: evidence.history_age,
                history_reset: evidence.history_reset,
            });
        }
    }
    if temporal_reconstruction.len() != capture_submission_ordinal {
        bail!(
            "temporal quality expected one renderer reconstruction record for each of {} submitted frames, found {}",
            capture_submission_ordinal,
            temporal_reconstruction.len()
        );
    }

    let artifact = RenderLabTemporalQualityArtifact {
        schema_version: RL2_QUALITY_SCHEMA_VERSION,
        scenario_id: RL2_QUALITY_SCENARIO_ID,
        scenario_revision: RL2_QUALITY_SCENARIO_REVISION,
        source_git_revision: std::env::var("RUNENWERK_SOURCE_REVISION").ok(),
        requested_internal_size_px: [requested_internal.0, requested_internal.1],
        requested_output_size_px: [requested_output.0, requested_output.1],
        capture_submission_ordinal,
        total_submitted_frames: history.len(),
        gpu: temporal_quality_gpu_evidence(gfx.adapter_facts()),
        temporal_reconstruction,
        capture_route: match execution.policy {
            "native" | "static_footprint" => "native_scene",
            "fixed" => "fixed_resolve",
            _ => "unexpected",
        },
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

pub(super) fn write_camera_motion_quality_artifact(
    measurement: &RenderLabMeasurementConfig,
    history: &RenderFrameHistoryState,
    quality_execution: &RenderLabTemporalQualityExecutionState,
    capture: RenderLabTemporalQualityCaptureEvidence,
    gfx: &engine::plugins::render::Gfx,
) -> Result<()> {
    let capture_root = measurement
        .quality_capture_output_dir
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("camera-motion capture output directory is unavailable"))?;
    let output_root = capture_root.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "camera-motion capture directory {} has no evidence output parent",
            capture_root.display()
        )
    })?;
    let requested_output = measurement
        .primary_window_size_px
        .ok_or_else(|| anyhow::anyhow!("camera-motion requested output extent is unavailable"))?;
    let requested_internal = measurement.radiance_target_size_px.ok_or_else(|| {
        anyhow::anyhow!("camera-motion requested evaluation extent is unavailable")
    })?;
    if requested_internal != requested_output {
        bail!(
            "camera-motion evidence is P100-only: evaluation {}x{} != requested {}x{}",
            requested_internal.0,
            requested_internal.1,
            requested_output.0,
            requested_output.1
        );
    }

    let execution = quality_execution
        .frame(capture.frame_index)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "camera-motion capture frame {} has no correlated execution evidence",
                capture.frame_index
            )
        })?;
    let capture_submission_ordinal = history
        .observations()
        .enumerate()
        .find_map(|(index, observation)| {
            (observation.key.frame_index == capture.frame_index).then_some(index + 1)
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "camera-motion capture frame {} is absent from submitted-frame history",
                capture.frame_index
            )
        })?;
    let expected_capture_ordinal = measurement.submitted_frame_limit.unwrap_or(1);
    if capture_submission_ordinal != expected_capture_ordinal {
        bail!(
            "camera-motion capture submission ordinal {} does not match configured target {}",
            capture_submission_ordinal,
            expected_capture_ordinal
        );
    }

    let mut generation_classes = Vec::new();
    let mut frames = Vec::new();
    for observation in history.observations().take(capture_submission_ordinal) {
        for evidence in gfx.deterministic_temporal_evidence(observation.key.frame_index) {
            let semantic_input_generation_classes = evidence
                .semantic_input_generations
                .iter()
                .map(|(representation_id, generation)| {
                    let representation_index = if let Some(index) = generation_classes
                        .iter()
                        .position(|(known_id, _)| known_id == representation_id)
                    {
                        index
                    } else {
                        generation_classes.push((*representation_id, Vec::new()));
                        generation_classes.len() - 1
                    };
                    let seen = &mut generation_classes[representation_index].1;
                    let class_index =
                        if let Some(index) = seen.iter().position(|known| known == generation) {
                            index
                        } else {
                            seen.push(*generation);
                            seen.len() - 1
                        };
                    u64::try_from(class_index + 1)
                        .expect("bounded camera-motion generation class fits u64")
                })
                .collect();

            frames.push(RenderLabCameraMotionFrameEvidence {
                frame_index: observation.key.frame_index,
                requested_size_px: [evidence.requested_extent.0, evidence.requested_extent.1],
                evaluation_size_px: [evidence.evaluation_extent.0, evidence.evaluation_extent.1],
                semantic_input_generation_classes,
                phase: evidence.phase,
                history_generation: evidence.history_generation,
                history_age: evidence.history_age,
                history_reset: evidence.history_reset,
                camera_reprojection_eligible: evidence.camera_reprojection_eligible,
                previous_observation_available: evidence.previous_observation_available,
                camera_pose_changed: evidence.camera_pose_changed,
                camera_reprojection_revision: evidence.camera_reprojection_revision,
                depth_policy_revision: evidence.depth_policy_revision,
            });
        }
    }
    if frames.len() != capture_submission_ordinal {
        bail!(
            "camera-motion quality expected one reconstruction record for each of {} submitted frames, found {}",
            capture_submission_ordinal,
            frames.len()
        );
    }

    let artifact = RenderLabCameraMotionArtifact {
        schema_version: RL2_CAMERA_MOTION_SCHEMA_VERSION,
        scenario_id: RL2_CAMERA_MOTION_SCENARIO_ID,
        scenario_revision: RL2_CAMERA_MOTION_SCENARIO_REVISION,
        source_git_revision: std::env::var("RUNENWERK_SOURCE_REVISION").ok(),
        requested_output_size_px: [requested_output.0, requested_output.1],
        capture_submission_ordinal,
        total_submitted_frames: history.len(),
        gpu: temporal_quality_gpu_evidence(gfx.adapter_facts()),
        execution,
        frames,
        capture_route: "native_scene",
        capture,
    };
    let path = output_root.join("camera-motion-evidence.json");
    fs::create_dir_all(output_root).with_context(|| {
        format!(
            "create camera-motion evidence directory {}",
            output_root.display()
        )
    })?;
    let bytes = serde_json::to_vec_pretty(&artifact).context("serialize camera-motion evidence")?;
    fs::write(&path, bytes)
        .with_context(|| format!("write camera-motion evidence {}", path.display()))
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
            policy: "static_footprint",
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

#[cfg(test)]
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

pub(super) struct RenderLabNativeQualityPublication {
    pub producer_id: engine::plugins::render::RenderFrameProducerId,
    pub render_surface_id: RenderSurfaceId,
    pub radiance_target: RenderDynamicTextureTargetDescriptor,
    pub native_scene_invocation: PreparedFlowInvocationRequest,
    pub contribution: RenderDeterministicFrameContribution,
}

pub(super) fn stage_render_lab_native_quality_publication(
    targets: &mut RenderDynamicTextureTargetRequestRegistryResource,
    frame_requests: &mut PreparedRenderFrameRequestResource,
    contributions: &mut RenderDeterministicFrameContributionResource,
    publication: RenderLabNativeQualityPublication,
) -> Result<()> {
    let RenderLabNativeQualityPublication {
        producer_id,
        render_surface_id,
        radiance_target,
        native_scene_invocation,
        contribution,
    } = publication;

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
    use std::{fs, path::PathBuf};

    fn producer(raw: u64) -> engine::plugins::render::RenderFrameProducerId {
        engine::plugins::render::RenderFrameProducerId::try_from_raw(raw)
            .expect("test producer id should be nonzero")
    }

    #[test]
    fn temporal_quality_evidence_profile_fingerprint_preserves_optional_fact_shape() {
        let absent = RenderLabTemporalQualityGpuFingerprintFacts {
            backend: "vulkan",
            adapter_class: "cpu",
            software_status: "software",
            fallback_status: "confirmed_not_fallback",
            diagnostic_name: None,
            driver: None,
            driver_info: None,
            vendor: None,
            device: None,
            device_request_profile: "modern_portable",
            device_request_profile_supported: true,
        };
        let present_empty = RenderLabTemporalQualityGpuFingerprintFacts {
            diagnostic_name: Some(""),
            driver: Some(""),
            driver_info: Some(""),
            ..absent
        };
        let absent_fingerprint = temporal_quality_evidence_profile_fingerprint(&absent);
        let present_empty_fingerprint =
            temporal_quality_evidence_profile_fingerprint(&present_empty);

        assert_ne!(absent_fingerprint, present_empty_fingerprint);
        assert!(absent_fingerprint.starts_with("blake3:"));
        assert!(present_empty_fingerprint.starts_with("blake3:"));
    }

    #[test]
    fn temporal_quality_gpu_tokens_are_stable() {
        assert_eq!(
            gpu_backend_token(runen_gpu::GpuBackendFamily::Vulkan),
            "vulkan"
        );
        assert_eq!(
            gpu_adapter_class_token(runen_gpu::GpuAdapterClass::Cpu),
            "cpu"
        );
        assert_eq!(
            gpu_software_token(runen_gpu::GpuSoftwareStatus::Software),
            "software"
        );
        assert_eq!(
            gpu_fallback_token(runen_gpu::GpuFallbackStatus::ConfirmedFallback),
            "confirmed_fallback"
        );
        assert_eq!(
            gpu_device_profile_token(runen_gpu::GpuDeviceRequestProfile::ModernPortable),
            "modern_portable"
        );
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

        let evidence = temporal_quality_capture_evidence(&report_state, 42)
            .expect("capture evidence should inspect")
            .expect("completed capture should produce evidence");
        assert_eq!(evidence.frame_index, 42);
        assert!(
            temporal_quality_capture_evidence(&report_state, 41)
                .expect("other frame inspection should succeed")
                .is_none()
        );
        assert_eq!(evidence.terminal, "completed");
        assert_eq!(
            evidence.artifact_blake3,
            format!("blake3:{}", blake3::hash(b"quality-capture-bytes").to_hex())
        );

        let _ = fs::remove_file(artifact_path);
    }

    #[test]
    fn temporal_quality_present_flow_supplies_terminal_native_presentation() {
        let flow = render_lab_quality_present_flow().expect("quality present flow should author");
        assert_eq!(flow.label(), RL2_QUALITY_PRESENT_FLOW_ID);
        assert!(
            flow.resource_id(engine::plugins::render::SURFACE_COLOR_RESOURCE_LABEL)
                .is_some()
        );
        assert!(flow.pass_id(RL2_QUALITY_PRESENT_PASS_ID).is_some());
        let compiled = engine::plugins::render::compile_flow_plan(&flow)
            .expect("quality present flow should compile");
        assert_eq!(compiled.render_passes.len(), 1);
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
        let fixed_admission =
            engine::plugins::render::RenderFixedResolutionExecutionAdmission::Fixed(
                prepared.clone(),
            );
        let selector =
            temporal_quality_capture_selector(Some(&fixed_admission), &scene_plan, &resolve_plan)
                .expect("fixed capture selector should resolve compiled ids");
        assert_eq!(
            selector.flow_id.as_deref(),
            Some(resolve_plan.flow_id.to_string().as_str())
        );
        assert_eq!(
            selector.pass_id.as_deref(),
            Some(resolve_plan.render_passes[0].pass_id().to_string().as_str())
        );
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

        let scene_plan = engine::plugins::render::compile_flow_plan(&scene)
            .expect("quality flow should compile");
        let resolve = engine::plugins::render::fixed_resolution_resolve_flow()
            .expect("resolve flow should author");
        let resolve_plan = engine::plugins::render::compile_flow_plan(&resolve)
            .expect("resolve flow should compile");
        let selector = temporal_quality_capture_selector(None, &scene_plan, &resolve_plan)
            .expect("native capture selector should resolve compiled ids");
        assert_eq!(
            selector.flow_id.as_deref(),
            Some(scene_plan.flow_id.to_string().as_str())
        );
        assert_eq!(
            selector.pass_id.as_deref(),
            Some(scene_plan.render_passes[0].pass_id().to_string().as_str())
        );

        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        let mut frame_requests = PreparedRenderFrameRequestResource::default();
        let mut contributions = RenderDeterministicFrameContributionResource::default();
        stage_render_lab_native_quality_publication(
            &mut targets,
            &mut frame_requests,
            &mut contributions,
            RenderLabNativeQualityPublication {
                producer_id,
                render_surface_id: RenderSurfaceId::primary(),
                radiance_target: target,
                native_scene_invocation: invocation.clone(),
                contribution,
            },
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
            finite_evaluation_extent: None,
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
        let selector =
            temporal_quality_capture_selector(Some(&admission), &scene_plan, &resolve_plan)
                .expect("native fallback capture selector should resolve compiled ids");
        assert_eq!(
            selector.flow_id.as_deref(),
            Some(scene_plan.flow_id.to_string().as_str())
        );
        assert_eq!(
            selector.pass_id.as_deref(),
            Some(scene_plan.render_passes[0].pass_id().to_string().as_str())
        );

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
            RenderLabNativeQualityPublication {
                producer_id,
                render_surface_id: fallback.render_surface_id,
                radiance_target: target,
                native_scene_invocation: native_scene_invocation.clone(),
                contribution,
            },
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
