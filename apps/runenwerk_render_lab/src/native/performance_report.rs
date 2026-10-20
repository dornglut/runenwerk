//! Read-only NP1 reduction of the Render owner's bounded frame-history snapshot.
//! This is a Render Lab output format, not another recording/timing authority.

use super::temporal_quality::{RenderLabTemporalQualityGpuEvidence, temporal_quality_gpu_evidence};
use super::*;
use engine::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservation, RenderGpuTimingCapability,
    RenderGpuTimingDiagnostic, RenderTimingSource,
};
use engine::runtime::FramePacingMode;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Write;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub(super) struct PerformanceReport {
    schema_version: u32,
    scenario_id: &'static str,
    source_revision_env: Option<String>,
    configuration: PerformanceConfiguration,
    retained_frame_count: usize,
    drop_stats: PerformanceDropStats,
    samples: Vec<PerformanceSample>,
    per_surface_extent: Vec<SurfaceExtentSummary>,
    interpretation: Interpretation,
}

#[derive(Debug, Serialize)]
struct PerformanceConfiguration {
    host_pacing_mode_at_close: &'static str,
    host_pacing_target_fps_at_close: Option<u32>,
    gpu_adapter_at_close: Option<RenderLabTemporalQualityGpuEvidence>,
    present_requested: &'static str,
    present_supported: &'static str,
    present_applied: &'static str,
    source_fixed_step: &'static str,
    sample_every_nth_frame: u64,
    retained_capacity: usize,
}

#[derive(Debug, Serialize)]
struct PerformanceDropStats {
    evicted_observations: u64,
    sampled_out_observations: u64,
    dropped_correlated_evidence: u64,
    uncorrelated_evidence: u64,
}

#[derive(Debug, Serialize)]
struct Interpretation {
    cpu: &'static str,
    gpu_composed: &'static str,
    gpu_pass: &'static str,
    present: &'static str,
    physical_scanout: &'static str,
    host_frame_wall_time: &'static str,
    gpu_queue_submission_completion: &'static str,
    percentile_method: &'static str,
    grouping: &'static str,
}

#[derive(Debug, Serialize)]
struct PerformanceSample {
    frame_index: u64,
    render_surface_id: u64,
    prepare_epoch: u64,
    output_size_px: [u32; 2],
    radiance_size_px: [u32; 2],
    cpu_stages_ms: BTreeMap<&'static str, Option<f32>>,
    cpu_passes: Vec<CpuPassSample>,
    composed_gpu: GpuDuration,
    gpu_pass_capability: &'static str,
    gpu_passes: Vec<GpuPassSample>,
    present_operation_submitted: bool,
    physical_display_timing: &'static str,
}

#[derive(Debug, Serialize)]
struct CpuPassSample {
    flow_id: String,
    pass_id: String,
    pass_kind: String,
    cpu_encode_ms: Option<f32>,
}

#[derive(Debug, Serialize)]
struct GpuDuration {
    source: &'static str,
    state: &'static str,
    duration_ms: Option<f32>,
    diagnostics: Vec<PerformanceDiagnostic>,
}

#[derive(Debug, Serialize)]
struct GpuPassSample {
    flow_id: String,
    pass_id: String,
    pass_kind: String,
    timing: GpuDuration,
}

#[derive(Debug, Serialize)]
struct PerformanceDiagnostic {
    kind: &'static str,
    capability: &'static str,
    source: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct DurationDistribution {
    observed_count: usize,
    eligible_count: usize,
    missing_count: usize,
    median_ms: Option<f32>,
    p95_ms: Option<f32>,
}

#[derive(Debug, Serialize)]
struct SurfaceExtentSummary {
    render_surface_id: u64,
    output_size_px: [u32; 2],
    radiance_size_px: [u32; 2],
    retained_frame_count: usize,
    cpu_stages: BTreeMap<&'static str, DurationDistribution>,
    composed_gpu: DurationDistribution,
    composed_gpu_states: BTreeMap<&'static str, usize>,
    frames_with_gpu_pass_evidence: usize,
    gpu_passes: Vec<PassSummary>,
}

#[derive(Debug, Serialize)]
struct PassSummary {
    flow_id: String,
    pass_id: String,
    pass_kind: String,
    timings: DurationDistribution,
    states: BTreeMap<&'static str, usize>,
}

#[derive(Default)]
struct DurationAccumulator {
    observed: usize,
    values: Vec<f32>,
}

impl DurationAccumulator {
    fn push(&mut self, candidate: Option<f32>) {
        self.observed += 1;
        if let Some(value) = candidate.and_then(valid_duration) {
            self.values.push(value);
        }
    }

    fn finish(mut self) -> DurationDistribution {
        self.values.sort_by(f32::total_cmp);
        let eligible_count = self.values.len();
        let median_ms = if eligible_count == 0 {
            None
        } else if eligible_count.is_multiple_of(2) {
            let upper = eligible_count / 2;
            Some(self.values[upper - 1] / 2.0 + self.values[upper] / 2.0)
        } else {
            Some(self.values[eligible_count / 2])
        };
        let p95_ms = if eligible_count == 0 {
            None
        } else {
            // Nearest rank: ceil(0.95 * n), indexing from zero.
            let rank = (eligible_count * 95).div_ceil(100);
            Some(self.values[rank - 1])
        };
        DurationDistribution {
            observed_count: self.observed,
            eligible_count,
            missing_count: self.observed - eligible_count,
            median_ms,
            p95_ms,
        }
    }
}

#[derive(Default)]
struct PassAccumulator {
    timings: DurationAccumulator,
    states: BTreeMap<&'static str, usize>,
}

#[derive(Default)]
struct GroupAccumulator {
    retained: usize,
    cpu: BTreeMap<&'static str, DurationAccumulator>,
    composed: DurationAccumulator,
    composed_states: BTreeMap<&'static str, usize>,
    frames_with_gpu_pass_evidence: usize,
    gpu_passes: BTreeMap<(String, String, String), PassAccumulator>,
}

fn valid_duration(value: f32) -> Option<f32> {
    (value.is_finite() && value >= 0.0).then_some(value)
}

fn stages(frame: &RenderFrameObservation) -> BTreeMap<&'static str, Option<f32>> {
    let cpu = &frame.cpu;
    let r = &cpu.renderer;
    // Child metrics under encode_submit_ms are NOT additive to their parent.
    [
        ("acquire_ms", cpu.acquire_ms),
        ("renderer.prepare_ui_ms", r.prepare_ui_ms),
        ("renderer.prepare_mesh_ms", r.prepare_mesh_ms),
        ("renderer.world_prepare_ms", r.world_prepare_ms),
        ("renderer.preflight_ms", r.preflight_ms),
        ("renderer.flow_encode_ms", r.flow_encode_ms),
        ("renderer.encode_submit_ms", r.encode_submit_ms),
        (
            "renderer.encode_submit_ms.gpu_graph_build_ms",
            r.gpu_graph_build_ms,
        ),
        (
            "renderer.encode_submit_ms.gpu_prepare_submission_ms",
            r.gpu_prepare_submission_ms,
        ),
        (
            "renderer.encode_submit_ms.gpu_submit_prepared_ms",
            r.gpu_submit_prepared_ms,
        ),
    ]
    .into_iter()
    .map(|(name, ms)| (name, valid_duration(ms)))
    .collect()
}

fn diagnostics(values: &[RenderGpuTimingDiagnostic]) -> Vec<PerformanceDiagnostic> {
    values
        .iter()
        .map(|d| PerformanceDiagnostic {
            kind: d.kind.as_str(),
            capability: d.capability.as_str(),
            source: d.source.as_str(),
            message: d.message.clone(),
        })
        .collect()
}

fn gpu_duration(
    source: Option<RenderTimingSource>,
    capability: RenderGpuTimingCapability,
    millis: Option<f32>,
    details: &[RenderGpuTimingDiagnostic],
) -> GpuDuration {
    let duration_ms = if capability == RenderGpuTimingCapability::Supported
        && source == Some(RenderTimingSource::GpuTimestampQuery)
    {
        millis.and_then(valid_duration)
    } else {
        None
    };
    let state = if duration_ms.is_some() {
        "measured"
    } else if capability == RenderGpuTimingCapability::Supported {
        "invalid_or_missing_sample"
    } else {
        capability.as_str()
    };
    GpuDuration {
        source: source.map_or("none", RenderTimingSource::as_str),
        state,
        duration_ms,
        diagnostics: diagnostics(details),
    }
}

pub(super) fn build_report(
    history: &RenderFrameHistoryState,
    radiance_override: Option<(u32, u32)>,
    pacing_mode: FramePacingMode,
    policy: RenderFrameObservationPolicyResource,
    gpu_adapter_facts: Option<&runen_gpu::GpuAdapterFacts>,
) -> PerformanceReport {
    let mut samples = Vec::with_capacity(history.len());
    let mut groups: BTreeMap<(u64, [u32; 2], [u32; 2]), GroupAccumulator> = BTreeMap::new();

    for observation in history.observations() {
        let size = [observation.target_size_px.0, observation.target_size_px.1];
        let source = radiance_override.unwrap_or(observation.target_size_px);
        let radiance = [source.0, source.1];
        let group = groups
            .entry((observation.key.render_surface_id, size, radiance))
            .or_default();
        group.retained += 1;

        let cpu_stages_ms = stages(observation);
        for (name, value) in &cpu_stages_ms {
            group.cpu.entry(*name).or_default().push(*value);
        }

        let composed = observation.gpu.composed_timing_evidence.as_ref();
        let composed_gpu = gpu_duration(
            composed.map(|e| e.source),
            observation.gpu.composed_timing_capability,
            composed.and_then(|e| e.gpu_composed_frame_ms),
            composed.map_or(&[], |e| e.diagnostics.as_slice()),
        );
        group.composed.push(composed_gpu.duration_ms);
        *group.composed_states.entry(composed_gpu.state).or_default() += 1;

        if !observation.gpu.pass_evidence.is_empty() {
            group.frames_with_gpu_pass_evidence += 1;
        }
        let gpu_passes = observation
            .gpu
            .pass_evidence
            .iter()
            .map(|e| {
                let timing =
                    gpu_duration(Some(e.source), e.gpu_capability, e.millis, &e.diagnostics);
                let pass = group
                    .gpu_passes
                    .entry((e.flow_id.clone(), e.pass_id.clone(), e.pass_kind.clone()))
                    .or_default();
                pass.timings.push(timing.duration_ms);
                *pass.states.entry(timing.state).or_default() += 1;
                GpuPassSample {
                    flow_id: e.flow_id.clone(),
                    pass_id: e.pass_id.clone(),
                    pass_kind: e.pass_kind.clone(),
                    timing,
                }
            })
            .collect();
        let cpu_passes = observation
            .cpu
            .pass_timings
            .iter()
            .map(|p| CpuPassSample {
                flow_id: p.flow_id.clone(),
                pass_id: p.pass_id.clone(),
                pass_kind: p.pass_kind.clone(),
                cpu_encode_ms: valid_duration(p.millis),
            })
            .collect();
        samples.push(PerformanceSample {
            frame_index: observation.key.frame_index,
            render_surface_id: observation.key.render_surface_id,
            prepare_epoch: observation.prepare_epoch,
            output_size_px: size,
            radiance_size_px: radiance,
            cpu_stages_ms,
            cpu_passes,
            composed_gpu,
            gpu_pass_capability: observation.gpu.pass_timing_capability.as_str(),
            gpu_passes,
            present_operation_submitted: observation.presentation.present_operation_submitted,
            physical_display_timing: "unavailable",
        });
    }

    let per_surface_extent = groups
        .into_iter()
        .map(
            |((render_surface_id, output_size_px, radiance_size_px), group)| SurfaceExtentSummary {
                render_surface_id,
                output_size_px,
                radiance_size_px,
                retained_frame_count: group.retained,
                cpu_stages: group
                    .cpu
                    .into_iter()
                    .map(|(name, acc)| (name, acc.finish()))
                    .collect(),
                composed_gpu: group.composed.finish(),
                composed_gpu_states: group.composed_states,
                frames_with_gpu_pass_evidence: group.frames_with_gpu_pass_evidence,
                gpu_passes: group
                    .gpu_passes
                    .into_iter()
                    .map(|((flow_id, pass_id, pass_kind), pass)| PassSummary {
                        flow_id,
                        pass_id,
                        pass_kind,
                        timings: pass.timings.finish(),
                        states: pass.states,
                    })
                    .collect(),
            },
        )
        .collect();

    let drop = history.drop_stats();
    PerformanceReport {
        schema_version: SCHEMA_VERSION,
        scenario_id: RL2_FLOW_ID,
        source_revision_env: std::env::var("RUNENWERK_SOURCE_REVISION").ok(),
        configuration: PerformanceConfiguration {
            host_pacing_mode_at_close: pacing_mode.as_str(),
            host_pacing_target_fps_at_close: pacing_mode.target_fps(),
            gpu_adapter_at_close: gpu_adapter_facts.map(temporal_quality_gpu_evidence),
            present_requested: "unknown",
            present_supported: "unknown",
            present_applied: "unknown",
            source_fixed_step: "unknown",
            sample_every_nth_frame: policy.sample_every_nth_frame,
            retained_capacity: policy.capacity,
        },
        retained_frame_count: samples.len(),
        drop_stats: PerformanceDropStats {
            evicted_observations: drop.evicted_observations,
            sampled_out_observations: drop.sampled_out_observations,
            dropped_correlated_evidence: drop.dropped_correlated_evidence,
            uncorrelated_evidence: drop.uncorrelated_evidence,
        },
        samples,
        per_surface_extent,
        interpretation: Interpretation {
            cpu: "Renderer-owned CPU stage observations; encode_submit child durations are nested, not additive; neither stage sum nor pass samples establish application frame wall time.",
            gpu_composed: "GPU timestamp interval for Render composition only; excludes RunenGPU initial-content preparation, readback tail, queue completion, Present, and scanout.",
            gpu_pass: "GPU timestamp pass evidence when correlated; absent pass records do not establish zero execution or a complete pass inventory.",
            present: "Present operation submission is not Present completion or a display timestamp.",
            physical_scanout: "unavailable",
            host_frame_wall_time: "unavailable",
            gpu_queue_submission_completion: "unavailable in this frame history; never inferred from Render composition duration",
            percentile_method: "median = middle value (mean of two middle values for even N); p95 = nearest rank ceil(0.95*N), among eligible observations only",
            grouping: "surface identity + output extent + radiance extent; adapter and Host policy facts are sampled only at report close, not per frame; device generation and cross-clock synchronization unavailable, so comparisons require external run configuration",
        },
    }
}

pub(super) fn write_report(path: &std::path::Path, report: &PerformanceReport) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "create Render Lab performance report directory {}",
                parent.display()
            )
        })?;
    }
    let bytes = serde_json::to_vec_pretty(report)
        .context("serialize native Render Lab performance report")?;
    // No overwrite: a differently spelled path, symlink, or hardlink must never
    // replace the existing schema-v2 artifact or an older performance report.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create new Render Lab performance report {}", path.display()))?;
    if let Err(error) = file.write_all(&bytes).and_then(|()| file.flush()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(error).with_context(|| format!("write Render Lab performance report {}", path.display()));
    }
    Ok(())
}

pub(super) fn terminal_summary(report: &PerformanceReport) -> String {
    let measured: usize = report
        .per_surface_extent
        .iter()
        .map(|group| group.composed_gpu.eligible_count)
        .sum();
    format!(
        "Render Lab NP1: {} retained frame(s), {} surface/extent group(s), \
         {}/{} eligible composed GPU timestamp(s), {} sampled out, {} evicted; \
         no Host wall-time or display FPS claim",
        report.retained_frame_count,
        report.per_surface_extent.len(),
        measured,
        report.retained_frame_count,
        report.drop_stats.sampled_out_observations,
        report.drop_stats.evicted_observations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::plugins::render::inspect::{
        RenderComposedFrameGpuTimingEvidence, RenderGpuTimingDiagnostic, RenderPassTimingEvidence,
    };
    use engine::plugins::render::renderer::RendererFrameTimings;

    fn add(
        history: &mut RenderFrameHistoryState,
        policy: RenderFrameObservationPolicyResource,
        frame: u64,
        surface: u64,
        width: u32,
        cpu_ms: f32,
    ) {
        history.observe_submitted_frame(
            policy,
            frame,
            surface,
            frame + 100,
            (width, 600),
            0.5,
            RendererFrameTimings {
                flow_encode_ms: cpu_ms,
                gpu_graph_build_ms: 0.25,
                ..Default::default()
            },
            &[],
            RenderGpuTimingCapability::ReadbackPending,
        );
    }

    fn build(
        history: &RenderFrameHistoryState,
        policy: RenderFrameObservationPolicyResource,
    ) -> PerformanceReport {
        build_report(
            history,
            None,
            FramePacingMode::ContinuousUncapped,
            policy,
            None,
        )
    }

    #[test]
    fn interleaved_surfaces_delayed_completion_and_statistics_are_correlated() {
        let policy = RenderFrameObservationPolicyResource::enabled(8);
        let mut history = RenderFrameHistoryState::default();
        add(&mut history, policy, 7, 1, 800, 4.0);
        add(&mut history, policy, 7, 2, 1200, 12.0);
        add(&mut history, policy, 8, 1, 800, 6.0);
        history.observe_composed_gpu_timing_evidence(
            policy,
            &[
                RenderComposedFrameGpuTimingEvidence::gpu_sample(8, 1, 10.0),
                RenderComposedFrameGpuTimingEvidence::gpu_sample(7, 2, 30.0),
                RenderComposedFrameGpuTimingEvidence::gpu_sample(7, 1, 20.0),
            ],
        );
        history.observe_gpu_pass_timing_evidence(
            policy,
            &[RenderPassTimingEvidence::gpu_sample(
                Some(7),
                Some(2),
                "flow",
                "pass",
                "compute",
                2.0,
            )],
        );
        let report = build(&history, policy);
        assert_eq!(report.samples.len(), 3);
        assert_eq!(report.samples[0].frame_index, 7);
        assert_eq!(report.samples[0].render_surface_id, 1);
        assert_eq!(report.samples[0].composed_gpu.duration_ms, Some(20.0));
        assert_eq!(report.samples[1].composed_gpu.duration_ms, Some(30.0));
        assert_eq!(report.samples[2].composed_gpu.duration_ms, Some(10.0));
        assert_eq!(report.per_surface_extent.len(), 2);
        let a = &report.per_surface_extent[0];
        assert_eq!(a.render_surface_id, 1);
        assert_eq!(a.cpu_stages["renderer.flow_encode_ms"].median_ms, Some(5.0));
        assert_eq!(a.composed_gpu.eligible_count, 2);
        assert_eq!(a.composed_gpu.p95_ms, Some(20.0));
        let b = &report.per_surface_extent[1];
        assert_eq!(
            b.cpu_stages["renderer.flow_encode_ms"].median_ms,
            Some(12.0)
        );
        assert_eq!(b.gpu_passes[0].timings.eligible_count, 1);
        assert_eq!(report.configuration.host_pacing_target_fps_at_close, None);
    }

    #[test]
    fn unsupported_pending_and_inconsistent_are_not_zero_samples() {
        let policy = RenderFrameObservationPolicyResource::enabled(8);
        let mut history = RenderFrameHistoryState::default();
        for i in 0..4 {
            add(&mut history, policy, i, 1, 800, i as f32);
        }
        history.observe_composed_gpu_timing_evidence(
            policy,
            &[
                RenderComposedFrameGpuTimingEvidence::gpu_diagnostic(
                    0,
                    1,
                    RenderGpuTimingDiagnostic::unsupported("feature absent"),
                ),
                RenderComposedFrameGpuTimingEvidence::gpu_diagnostic(
                    1,
                    1,
                    RenderGpuTimingDiagnostic::readback_pending("not finished"),
                ),
                RenderComposedFrameGpuTimingEvidence::gpu_sample(2, 1, 0.0),
            ],
        );
        let report = build(&history, policy);
        let group = &report.per_surface_extent[0];
        assert_eq!(group.composed_gpu.observed_count, 4);
        assert_eq!(group.composed_gpu.eligible_count, 1);
        assert_eq!(group.composed_gpu.missing_count, 3);
        assert_eq!(group.composed_gpu.median_ms, Some(0.0));
        assert_eq!(group.composed_gpu_states["unsupported"], 1);
        assert_eq!(group.composed_gpu_states["readback_pending"], 1);
        assert_eq!(group.composed_gpu_states["unavailable_this_frame"], 1);
        assert_eq!(
            report.samples[0].composed_gpu.diagnostics[0].kind,
            "timestamp_queries_unsupported"
        );
        assert_eq!(report.interpretation.physical_scanout, "unavailable");
    }

    #[test]
    fn sampling_eviction_missing_correlation_and_extent_changes_are_visible() {
        let policy = RenderFrameObservationPolicyResource::enabled(2).with_sampling(2);
        let mut history = RenderFrameHistoryState::default();
        for i in 0..6 {
            add(
                &mut history,
                policy,
                i,
                1,
                if i == 4 { 1024 } else { 800 },
                5.0,
            );
        }
        history.observe_gpu_pass_timing_evidence(
            policy,
            &[RenderPassTimingEvidence::gpu_sample(
                None,
                Some(1),
                "f",
                "p",
                "compute",
                1.0,
            )],
        );
        history.observe_composed_gpu_timing_evidence(
            policy,
            &[RenderComposedFrameGpuTimingEvidence::gpu_sample(0, 1, 10.0)],
        );
        let report = build(&history, policy);
        assert_eq!(report.retained_frame_count, 2);
        assert_eq!(report.drop_stats.sampled_out_observations, 3);
        assert_eq!(report.drop_stats.evicted_observations, 1);
        assert_eq!(report.drop_stats.uncorrelated_evidence, 1);
        assert_eq!(report.drop_stats.dropped_correlated_evidence, 1);
        assert_eq!(report.per_surface_extent.len(), 2);
        assert_eq!(report.samples[1].output_size_px, [1024, 600]);
        assert!(
            report
                .samples
                .iter()
                .all(|s| s.composed_gpu.duration_ms.is_none())
        );
    }

    #[test]
    fn invalid_duration_excluded_but_observation_count_retained() {
        let policy = RenderFrameObservationPolicyResource::enabled(4);
        let mut history = RenderFrameHistoryState::default();
        add(&mut history, policy, 0, 1, 800, f32::NAN);
        add(&mut history, policy, 1, 1, 800, 4.0);
        let report = build(&history, policy);
        let metric = &report.per_surface_extent[0].cpu_stages["renderer.flow_encode_ms"];
        assert_eq!(
            (
                metric.observed_count,
                metric.eligible_count,
                metric.missing_count
            ),
            (2, 1, 1)
        );
        assert_eq!(
            report.samples[0].cpu_stages_ms["renderer.flow_encode_ms"],
            None
        );
        assert!(serde_json::to_vec(&report).is_ok());
    }

    #[test]
    fn supported_without_valid_gpu_timestamp_is_classified_as_missing() {
        let missing = gpu_duration(
            Some(RenderTimingSource::GpuTimestampQuery),
            RenderGpuTimingCapability::Supported,
            None,
            &[],
        );
        assert_eq!(missing.state, "invalid_or_missing_sample");
        assert_eq!(missing.duration_ms, None);
        let bad_source = gpu_duration(
            Some(RenderTimingSource::CpuEncodeSubmit),
            RenderGpuTimingCapability::Supported,
            Some(10.0),
            &[],
        );
        assert_eq!(bad_source.state, "invalid_or_missing_sample");
        assert_eq!(bad_source.duration_ms, None);
        let nonfinite = gpu_duration(
            Some(RenderTimingSource::GpuTimestampQuery),
            RenderGpuTimingCapability::Supported,
            Some(f32::INFINITY),
            &[],
        );
        assert_eq!(nonfinite.state, "invalid_or_missing_sample");
        assert_eq!(nonfinite.duration_ms, None);
    }

    #[test]
    fn report_writer_never_overwrites_an_existing_measurement() {
        let path = std::env::temp_dir().join(format!(
            "runenwerk_np1_no_overwrite_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, b"existing schema-v2 evidence").unwrap();
        let report = build(
            &RenderFrameHistoryState::default(),
            RenderFrameObservationPolicyResource::enabled(4),
        );
        assert!(write_report(&path, &report).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"existing schema-v2 evidence".to_vec());
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn report_is_versioned_and_does_not_expose_frame_rate_from_identifiers() {
        let policy = RenderFrameObservationPolicyResource::enabled(4);
        let history = RenderFrameHistoryState::default();
        let report = build(&history, policy);
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(
            json["configuration"]["host_pacing_mode_at_close"],
            "continuous_uncapped"
        );
        assert_eq!(json["configuration"]["present_applied"], "unknown");
        assert!(json.get("fps").is_none());
        assert_eq!(json["per_surface_extent"].as_array().unwrap().len(), 0);
        assert!(terminal_summary(&report).contains("no Host wall-time or display FPS claim"));
    }
}
