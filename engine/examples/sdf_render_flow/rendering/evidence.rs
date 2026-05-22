use std::collections::BTreeMap;

use crate::rendering::build_render_flow;
use anyhow::Result;
use engine::plugins::render::features::world::sdf_raymarch::{
    RenderSdfRaymarchAccelerationConfig, inspect_sdf_raymarch_acceleration,
};
use engine::plugins::render::features::world::sdf_residency::{
    RenderSdfResidencyBudgetResource, RenderSdfResidencyResource, RenderSdfResidencySourceResource,
};
use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderGpuTimingDiagnostic, RenderSdfProductionEvidenceReport,
    RenderSdfProductionEvidenceRequest, RenderSdfProductionHardwareProfile,
    RenderSdfRuntimeVisualEvidence, inspect_compiled_render_flow_plan,
    inspect_render_sdf_production_evidence, inspect_render_sdf_residency,
};
use engine::plugins::render::{RenderBackendCapabilityProfile, compile_flow_plan_checked};
use product::{
    ProductAuthorityClass, ProductFreshness, ProductIdentity, ProductQueryPolicy, ProductResidency,
    ProductScaleBand, RenderProductSelection, RenderResidencyRequest, RenderSelectedProduct,
};
use spatial::{ChunkCoord3, ChunkId, WorldId};
use world_ops::{ChunkGeneration, ChunkRevision, OperationId};
use world_sdf::{
    SdfBrickMetadata, SdfBrickRecord, SdfBrickSamples, SdfChunkPayload, SdfPageCoord3,
    SdfPageRecord,
};

pub(crate) const SDF_RUNTIME_BENCHMARK_COMMAND: &str =
    "cargo bench -p engine --bench render_flow_planning";

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SdfRuntimeEvidenceReport {
    pub flow_label: String,
    pub pass_count: usize,
    pub production: RenderSdfProductionEvidenceReport,
}

impl SdfRuntimeEvidenceReport {
    pub(crate) fn format_text(&self) -> String {
        let mut lines = vec![
            format!(
                "sdf_runtime_evidence flow={} passes={} ready={} errors={} warnings={}",
                self.flow_label,
                self.pass_count,
                self.production.is_runtime_ready(),
                self.production.error_count(),
                self.production.warning_count()
            ),
            format!("benchmark_command={}", SDF_RUNTIME_BENCHMARK_COMMAND),
            format!(
                "residency selected={} resident={} pages={} bricks={} clipmaps={} resident_bytes={} upload_bytes={}",
                self.production.counts.selected_product_count,
                self.production.counts.resident_product_count,
                self.production.counts.resident_page_count,
                self.production.counts.resident_brick_count,
                self.production.counts.clipmap_window_count,
                self.production.counts.resident_bytes,
                self.production.counts.upload_bytes
            ),
            format!(
                "raymarch distance_mips={} candidate_lists={} candidates={} rejected={} max_steps={}",
                self.production.counts.distance_mip_count,
                self.production.counts.candidate_list_count,
                self.production.counts.total_candidate_count,
                self.production.counts.rejected_candidate_count,
                self.production.counts.max_steps_per_ray
            ),
            format!(
                "timing source={} cpu_samples={} cpu_total_ms={:.4} gpu_samples={} gpu_total_ms={:.4} gpu_diagnostics={}",
                self.production.timings.timing_source,
                self.production.timings.cpu_pass_sample_count,
                self.production.timings.cpu_total_pass_millis,
                self.production.timings.gpu_pass_sample_count,
                self.production.timings.gpu_total_pass_millis,
                self.production.timings.gpu_timing_diagnostic_count
            ),
        ];

        for visual in &self.production.visual_evidence {
            lines.push(format!(
                "visual view={} band={} artifact={} steps={} missed_surface_risk={} overstep_risk={}",
                visual.view_label,
                visual.coverage_band,
                visual.artifact_path,
                visual.step_count,
                visual.missed_surface_risk,
                visual.overstep_risk
            ));
        }

        lines.join("\n")
    }
}

pub(crate) fn production_evidence_report() -> Result<SdfRuntimeEvidenceReport> {
    let flow = build_render_flow();
    let profile = RenderBackendCapabilityProfile::runtime_default();
    let compiled = compile_flow_plan_checked(&flow, &profile)?;
    let compiled_inspection = inspect_compiled_render_flow_plan(&compiled);
    let residency = build_runtime_residency();
    let residency_inspection = inspect_render_sdf_residency(&residency);
    let raymarch = inspect_sdf_raymarch_acceleration(
        &residency,
        RenderSdfRaymarchAccelerationConfig {
            screen_tile_count: 2,
            depth_slice_count: 2,
            max_candidates_per_list: 8,
            max_steps_per_ray: 160,
            ..RenderSdfRaymarchAccelerationConfig::default()
        },
    );
    let production = inspect_render_sdf_production_evidence(RenderSdfProductionEvidenceRequest {
        hardware_profile: RenderSdfProductionHardwareProfile {
            profile_key: "portable-sdf-runtime".to_string(),
            adapter_name: Some("portable example profile".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        residency: residency_inspection,
        raymarch,
        timings: runtime_timing_evidence(),
        visual_evidence: visual_evidence(),
        benchmark_commands: vec![SDF_RUNTIME_BENCHMARK_COMMAND.to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-sdf-runtime-evidence/README.md".to_string(),
            "docs-site/src/content/docs/engine/benchmarks/render-sdf-runtime-evidence.md"
                .to_string(),
        ],
    });

    Ok(SdfRuntimeEvidenceReport {
        flow_label: compiled_inspection.flow_label,
        pass_count: compiled_inspection.pass_count,
        production,
    })
}

fn runtime_timing_evidence() -> RenderDebugTimingsState {
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[
        PassTimingSample {
            flow_id: "sdf_render_flow_3d".to_string(),
            pass_id: "sdf.prepare".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.18,
            dispatch_workgroups: Some([1, 1, 1]),
        },
        PassTimingSample {
            flow_id: "sdf_render_flow_3d".to_string(),
            pass_id: "sdf.compose".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.42,
            dispatch_workgroups: None,
        },
        PassTimingSample {
            flow_id: "sdf_render_flow_3d".to_string(),
            pass_id: "sdf.present".to_string(),
            pass_kind: "present".to_string(),
            millis: 0.05,
            dispatch_workgroups: None,
        },
    ]);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "portable SDF evidence command records unsupported timestamp queries explicitly",
    ));
    timings
}

fn visual_evidence() -> Vec<RenderSdfRuntimeVisualEvidence> {
    [
        ("sdf.lit.near", "near", 32),
        ("sdf.depth.mid", "mid", 48),
        ("sdf.normals.far", "far", 64),
        ("sdf.steps.summary", "summary", 96),
    ]
    .into_iter()
    .map(
        |(view_label, coverage_band, step_count)| RenderSdfRuntimeVisualEvidence {
            view_label: view_label.to_string(),
            coverage_band: coverage_band.to_string(),
            artifact_path: format!(
                "engine/benchmark-artifacts/render-sdf-runtime-evidence/{coverage_band}.txt"
            ),
            step_count,
            missed_surface_risk: false,
            overstep_risk: false,
        },
    )
    .collect()
}

fn build_runtime_residency() -> RenderSdfResidencyResource {
    let mut sources = RenderSdfResidencySourceResource::default();
    let mut selections = Vec::new();
    for (index, scale_band) in [
        ProductScaleBand::Near,
        ProductScaleBand::Mid,
        ProductScaleBand::Far,
        ProductScaleBand::Summary,
    ]
    .into_iter()
    .enumerate()
    {
        let generation = u64::try_from(index + 1).expect("example index fits in u64");
        let product_id = ProductIdentity::new(generation);
        sources.upsert_payload(product_id, generation, payload(generation, index + 1));
        selections.push(selection(product_id, generation, scale_band));
    }

    let mut residency = RenderSdfResidencyResource::default();
    residency.derive_from_sources(
        &selections,
        &sources,
        &RenderSdfResidencyBudgetResource::default(),
    );
    residency
}

fn selection(
    product_id: ProductIdentity,
    generation: u64,
    scale_band: ProductScaleBand,
) -> RenderProductSelection {
    RenderProductSelection::new("sdf.runtime")
        .with_selected_product(RenderSelectedProduct {
            product_id,
            scale_band,
            generation,
            freshness: ProductFreshness::Current,
            residency: ProductResidency::Resident,
            authority_class: ProductAuthorityClass::DeterministicDerived,
            query_policy: ProductQueryPolicy::StrictCurrentOnly,
        })
        .with_residency_request(RenderResidencyRequest::new(
            product_id,
            ProductResidency::Resident,
            10,
            false,
        ))
}

fn payload(product_index: u64, page_count: usize) -> SdfChunkPayload {
    let mut page_table = BTreeMap::new();
    for page_index in 0..page_count {
        let mut bricks = BTreeMap::new();
        bricks.insert(
            [0, 0, 0],
            SdfBrickRecord {
                metadata: SdfBrickMetadata {
                    occupancy_mask: 0b0011,
                    material_channel_mask: 0b0101,
                    last_touched_op_id: OperationId(11),
                    surface_band_present: true,
                    ..SdfBrickMetadata::default()
                },
                samples: SdfBrickSamples {
                    distances: vec![2; 8],
                },
            },
        );
        page_table.insert(
            SdfPageCoord3 {
                x: i16::try_from(page_index).expect("example page index fits in i16"),
                y: 0,
                z: 0,
            },
            SdfPageRecord {
                page_generation: u64::try_from(page_index).expect("example page index fits in u64"),
                bricks,
            },
        );
    }
    SdfChunkPayload {
        chunk_id: ChunkId::new(
            WorldId(1),
            ChunkCoord3 {
                x: i32::try_from(product_index).expect("example product index fits in i32"),
                y: 0,
                z: 0,
            },
        ),
        chunk_revision: ChunkRevision(product_index),
        chunk_generation: ChunkGeneration(product_index),
        page_table,
        hierarchy_revision: product_index,
        checksum: product_index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdf_runtime_evidence_report_is_ready_and_covers_all_bands() {
        let report = production_evidence_report().expect("SDF runtime evidence should build");

        assert!(report.production.is_runtime_ready());
        assert_eq!(report.pass_count, 4);
        assert_eq!(report.production.counts.visual_evidence_count, 4);
        assert_eq!(report.production.counts.resident_product_count, 4);
        assert_eq!(report.production.counts.distance_mip_count, 4);
        assert_eq!(report.production.timings.gpu_timing_diagnostic_count, 1);
    }

    #[test]
    fn sdf_runtime_evidence_text_names_residency_raymarch_and_visual_evidence() {
        let text = production_evidence_report()
            .expect("SDF runtime evidence should build")
            .format_text();

        assert!(text.contains("sdf_runtime_evidence"));
        assert!(text.contains("residency selected="));
        assert!(text.contains("raymarch distance_mips="));
        assert!(text.contains("visual view=sdf.steps.summary"));
    }
}
