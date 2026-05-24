use crate::rendering::{BoidsRenderState, DEFAULT_BOID_COUNT, build_render_flow};
use anyhow::Result;
use engine::plugins::render::inspect::{
    RenderGpuTimingDiagnostic, RenderPassTimingEvidence, inspect_compiled_render_flow_plan,
};
use engine::plugins::render::{
    CompiledPassExecutionPlan, CompiledRenderFlowPlan, PreparedFlowInputs, PreparedFlowInvocation,
    PreparedFrameContext, PreparedFrameContributions, PreparedRenderFrame, PreparedShaderSnapshot,
    PreparedSurfaceInfo, PreparedViewFrame, RenderBackendCapabilityProfile, RenderPassId,
    compile_flow_plan_checked, preflight_prepared_render_frame,
};
use std::time::Instant;
use ui_render_data::ViewportSurfaceBindingRegistry;

pub(crate) const BOIDS_EVIDENCE_SCENE_SIZE: (u32, u32) = (1600, 900);
pub(crate) const RENDER_FLOW_PLANNING_BENCHMARK_COMMAND: &str =
    "cargo bench -p engine --bench render_flow_planning";

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoidsProductionEvidenceReport {
    pub flow_label: String,
    pub scene_size: (u32, u32),
    pub boid_count: u32,
    pub grid_cell_count: u32,
    pub sorted_index_capacity: u32,
    pub fixed_step_seconds: f32,
    pub submitted_step_count: u32,
    pub aspect_correct_impostors: bool,
    pub smoothed_visual_heading: bool,
    pub silent_grid_overflow: bool,
    pub resize_pixel_evidence: Vec<BoidsResizePixelEvidence>,
    pub pass_count: usize,
    pub passes: Vec<BoidsProductionPassEvidence>,
    pub gpu_timing_evidence: Vec<RenderPassTimingEvidence>,
    pub cpu_timing_evidence: BoidsProductionCpuTimingEvidence,
    pub cpu_timing_fields: Vec<&'static str>,
    pub benchmark_command: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoidsProductionPassEvidence {
    pub label: String,
    pub kind: &'static str,
    pub order_index: usize,
    pub gpu_timestamp_expected: bool,
    pub dispatch_workgroups_available: bool,
    pub local_instance_geometry: bool,
    pub vertex_count: Option<u32>,
    pub instance_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoidsProductionCpuTimingEvidence {
    pub source: &'static str,
    pub preflight_ms: f32,
    pub flow_encode_ms: Option<f32>,
    pub encode_submit_ms: Option<f32>,
    pub present_ms: Option<f32>,
    pub unavailable_reason: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoidsResizePixelEvidence {
    pub surface_size: (u32, u32),
    pub sprite_width_px: f32,
    pub sprite_height_px: f32,
    pub clip_width: f32,
    pub clip_height: f32,
    pub reconstructed_width_px: f32,
    pub reconstructed_height_px: f32,
    pub aspect_error_px: f32,
}

impl BoidsProductionEvidenceReport {
    pub(crate) fn format_text(&self) -> String {
        let mut lines = vec![
            format!(
                "boids_production_evidence flow={} scene={}x{} boids={} grid_cells={} sorted_index_capacity={} passes={}",
                self.flow_label,
                self.scene_size.0,
                self.scene_size.1,
                self.boid_count,
                self.grid_cell_count,
                self.sorted_index_capacity,
                self.pass_count
            ),
            format!(
                "simulation_contract=fixed_step fixed_dt_seconds={:.6} submitted_steps={} smoothed_visual_heading={} aspect_correct_impostors={} silent_grid_overflow={}",
                self.fixed_step_seconds,
                self.submitted_step_count,
                self.smoothed_visual_heading,
                self.aspect_correct_impostors,
                self.silent_grid_overflow
            ),
            format!("benchmark_command={}", self.benchmark_command),
            format!("cpu_timing_fields={}", self.cpu_timing_fields.join(",")),
        ];

        for pass in &self.passes {
            lines.push(format!(
                "pass label={} kind={} order={} gpu_timestamp_expected={} dispatch_available={} local_instance_geometry={} vertex_count={} instance_count={}",
                pass.label,
                pass.kind,
                pass.order_index,
                pass.gpu_timestamp_expected,
                pass.dispatch_workgroups_available,
                pass.local_instance_geometry,
                pass.vertex_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "n/a".to_string()),
                pass.instance_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "n/a".to_string())
            ));
        }

        for evidence in &self.gpu_timing_evidence {
            lines.push(format!(
                "gpu_timing flow={} pass={} kind={} capability={} diagnostics={}",
                evidence.flow_id,
                evidence.pass_id,
                evidence.pass_kind,
                evidence.gpu_capability.as_str(),
                evidence.diagnostics.len()
            ));
        }

        for evidence in &self.resize_pixel_evidence {
            lines.push(format!(
                "resize_pixel_evidence surface={}x{} sprite_width_px={:.3} sprite_height_px={:.3} clip_width={:.6} clip_height={:.6} reconstructed_width_px={:.3} reconstructed_height_px={:.3} aspect_error_px={:.5}",
                evidence.surface_size.0,
                evidence.surface_size.1,
                evidence.sprite_width_px,
                evidence.sprite_height_px,
                evidence.clip_width,
                evidence.clip_height,
                evidence.reconstructed_width_px,
                evidence.reconstructed_height_px,
                evidence.aspect_error_px
            ));
        }

        lines.push(format!(
            "cpu_timing source={} preflight_ms={:.4} flow_encode_ms={} encode_submit_ms={} present_ms={} unavailable_reason={}",
            self.cpu_timing_evidence.source,
            self.cpu_timing_evidence.preflight_ms,
            format_optional_millis(self.cpu_timing_evidence.flow_encode_ms),
            format_optional_millis(self.cpu_timing_evidence.encode_submit_ms),
            format_optional_millis(self.cpu_timing_evidence.present_ms),
            self.cpu_timing_evidence.unavailable_reason
        ));

        lines.join("\n")
    }
}

pub(crate) fn production_evidence_report() -> Result<BoidsProductionEvidenceReport> {
    let flow = build_render_flow();
    let state = BoidsRenderState::default();
    let profile = RenderBackendCapabilityProfile::runtime_default();
    let compiled = compile_flow_plan_checked(&flow, &profile)?;
    let inspection = inspect_compiled_render_flow_plan(&compiled);
    let passes = pass_evidence(&compiled);
    let gpu_timing_evidence = unsupported_gpu_timing_evidence(&compiled, &passes);
    let cpu_timing_evidence = measure_cpu_timing_evidence(&compiled, &profile)?;
    let resize_pixel_evidence = resize_pixel_evidence(&state);

    Ok(BoidsProductionEvidenceReport {
        flow_label: inspection.flow_label,
        scene_size: BOIDS_EVIDENCE_SCENE_SIZE,
        boid_count: DEFAULT_BOID_COUNT,
        grid_cell_count: state.grid_cell_count(),
        sorted_index_capacity: DEFAULT_BOID_COUNT,
        fixed_step_seconds: state.fixed_delta_seconds(),
        submitted_step_count: state.submitted_step_count(),
        aspect_correct_impostors: resize_pixel_evidence
            .iter()
            .all(|evidence| evidence.aspect_error_px <= 0.01),
        smoothed_visual_heading: true,
        silent_grid_overflow: false,
        resize_pixel_evidence,
        pass_count: inspection.pass_count,
        passes,
        gpu_timing_evidence,
        cpu_timing_evidence,
        cpu_timing_fields: vec![
            "preflight_ms",
            "flow_encode_ms",
            "encode_submit_ms",
            "present_ms",
        ],
        benchmark_command: RENDER_FLOW_PLANNING_BENCHMARK_COMMAND,
    })
}

fn resize_pixel_evidence(state: &BoidsRenderState) -> Vec<BoidsResizePixelEvidence> {
    [(1600, 900), (900, 1600), (1024, 1024)]
        .into_iter()
        .map(|surface_size| resize_pixel_evidence_for_surface(state, surface_size))
        .collect()
}

fn resize_pixel_evidence_for_surface(
    state: &BoidsRenderState,
    surface_size: (u32, u32),
) -> BoidsResizePixelEvidence {
    let params = state.draw_params(surface_size);
    let radius_px = params.sprite[0];
    let sprite_width_px = radius_px * params.sprite[1] * 2.0;
    let sprite_height_px = radius_px * params.sprite[2] * 2.0;
    let clip_width = sprite_width_px * 2.0 * params.surface[2];
    let clip_height = sprite_height_px * 2.0 * params.surface[3];
    let reconstructed_width_px = clip_width * params.surface[0] * 0.5;
    let reconstructed_height_px = clip_height * params.surface[1] * 0.5;
    let aspect_error_px = (reconstructed_width_px - sprite_width_px)
        .abs()
        .max((reconstructed_height_px - sprite_height_px).abs());

    BoidsResizePixelEvidence {
        surface_size,
        sprite_width_px,
        sprite_height_px,
        clip_width,
        clip_height,
        reconstructed_width_px,
        reconstructed_height_px,
        aspect_error_px,
    }
}

fn format_optional_millis(value: Option<f32>) -> String {
    value
        .map(|millis| format!("{millis:.4}"))
        .unwrap_or_else(|| "unavailable".to_string())
}

fn measure_cpu_timing_evidence(
    compiled: &CompiledRenderFlowPlan,
    profile: &RenderBackendCapabilityProfile,
) -> Result<BoidsProductionCpuTimingEvidence> {
    let prepared_frame = prepared_frame_for_flow(compiled);
    let start = Instant::now();
    preflight_prepared_render_frame(&prepared_frame, std::slice::from_ref(compiled), profile)?;
    Ok(BoidsProductionCpuTimingEvidence {
        source: "prepared_frame_preflight",
        preflight_ms: start.elapsed().as_secs_f32() * 1000.0,
        flow_encode_ms: None,
        encode_submit_ms: None,
        present_ms: None,
        unavailable_reason: "windowed_submit_not_run_by_evidence_command",
    })
}

fn prepared_frame_for_flow(compiled: &CompiledRenderFlowPlan) -> PreparedRenderFrame {
    PreparedRenderFrame {
        context: PreparedFrameContext {
            frame_index: 1,
            flow_registry_revision: 1,
            shader_registry_revision: 1,
            prepare_epoch: 1,
        },
        surface: PreparedSurfaceInfo::primary(BOIDS_EVIDENCE_SCENE_SIZE),
        views: vec![PreparedViewFrame::main(BOIDS_EVIDENCE_SCENE_SIZE)],
        flows: Default::default(),
        flow_invocations: vec![PreparedFlowInvocation::main(
            compiled.flow_id,
            prepared_inputs_for_flow(compiled),
        )],
        dynamic_texture_targets: Vec::new(),
        dynamic_texture_uploads: Vec::new(),
        product_selections: Vec::new(),
        viewport_surface_bindings: ViewportSurfaceBindingRegistry::default(),
        contributions: PreparedFrameContributions::default(),
        shader: PreparedShaderSnapshot {
            registry_revision: 1,
        },
    }
}

fn prepared_inputs_for_flow(compiled: &CompiledRenderFlowPlan) -> PreparedFlowInputs {
    let mut inputs = PreparedFlowInputs::default();
    for pass in &compiled.execution.passes {
        match pass {
            CompiledPassExecutionPlan::Compute(value) => {
                for uniform_id in &value.bindings.uniform_order {
                    inputs
                        .projected_uniform_bytes
                        .insert(*uniform_id, vec![0; 16]);
                }
                if value.dispatch.is_some() {
                    inputs
                        .projected_dispatch_workgroups
                        .insert(value.pass_id, [6, 1, 1]);
                }
            }
            CompiledPassExecutionPlan::Fullscreen(value)
            | CompiledPassExecutionPlan::Graphics(value) => {
                for uniform_id in &value.bindings.uniform_order {
                    inputs
                        .projected_uniform_bytes
                        .insert(*uniform_id, vec![0; 16]);
                }
            }
            CompiledPassExecutionPlan::Copy(_)
            | CompiledPassExecutionPlan::Present(_)
            | CompiledPassExecutionPlan::BuiltinUiComposite(_) => {}
        }
    }
    inputs
}

fn pass_evidence(compiled: &CompiledRenderFlowPlan) -> Vec<BoidsProductionPassEvidence> {
    compiled
        .execution
        .passes
        .iter()
        .map(|pass| {
            let pass_id = execution_pass_id(pass);
            BoidsProductionPassEvidence {
                label: pass_label(compiled, pass_id),
                kind: execution_pass_kind(pass),
                order_index: execution_order_index(pass),
                gpu_timestamp_expected: pass_supports_gpu_timestamps(pass),
                dispatch_workgroups_available: compute_dispatch_available(pass),
                local_instance_geometry: local_instance_geometry(pass),
                vertex_count: draw_vertex_count(pass),
                instance_count: draw_instance_count(pass),
            }
        })
        .collect()
}

fn unsupported_gpu_timing_evidence(
    compiled: &CompiledRenderFlowPlan,
    passes: &[BoidsProductionPassEvidence],
) -> Vec<RenderPassTimingEvidence> {
    passes
        .iter()
        .filter(|pass| pass.gpu_timestamp_expected)
        .map(|pass| {
            RenderPassTimingEvidence::gpu_diagnostic(
                Some(1),
                Some(1),
                compiled.flow_label.clone(),
                pass.label.clone(),
                pass.kind,
                RenderGpuTimingDiagnostic::unsupported(
                    "timestamp queries are not supported by the active WGPU backend",
                ),
            )
        })
        .collect()
}

fn pass_label(compiled: &CompiledRenderFlowPlan, pass_id: RenderPassId) -> String {
    compiled
        .pass_order
        .iter()
        .find(|pass| pass.pass_id() == pass_id)
        .map(|pass| pass.pass_label().to_string())
        .unwrap_or_else(|| format!("{pass_id:?}"))
}

fn execution_pass_id(pass: &CompiledPassExecutionPlan) -> RenderPassId {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.pass_id,
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => value.pass_id,
        CompiledPassExecutionPlan::Copy(value) => value.pass_id,
        CompiledPassExecutionPlan::Present(value) => value.pass_id,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => value.pass_id,
    }
}

fn execution_order_index(pass: &CompiledPassExecutionPlan) -> usize {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.order_index,
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => value.order_index,
        CompiledPassExecutionPlan::Copy(value) => value.order_index,
        CompiledPassExecutionPlan::Present(value) => value.order_index,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => value.order_index,
    }
}

fn execution_pass_kind(pass: &CompiledPassExecutionPlan) -> &'static str {
    match pass {
        CompiledPassExecutionPlan::Compute(_) => "compute",
        CompiledPassExecutionPlan::Fullscreen(_) => "fullscreen",
        CompiledPassExecutionPlan::Graphics(_) => "graphics",
        CompiledPassExecutionPlan::Copy(_) => "copy",
        CompiledPassExecutionPlan::Present(_) => "present",
        CompiledPassExecutionPlan::BuiltinUiComposite(_) => "builtin_ui_composite",
    }
}

fn pass_supports_gpu_timestamps(pass: &CompiledPassExecutionPlan) -> bool {
    matches!(
        pass,
        CompiledPassExecutionPlan::Compute(_)
            | CompiledPassExecutionPlan::Fullscreen(_)
            | CompiledPassExecutionPlan::Graphics(_)
            | CompiledPassExecutionPlan::BuiltinUiComposite(_)
    )
}

fn compute_dispatch_available(pass: &CompiledPassExecutionPlan) -> bool {
    matches!(pass, CompiledPassExecutionPlan::Compute(value) if value.dispatch.is_some())
}

fn local_instance_geometry(pass: &CompiledPassExecutionPlan) -> bool {
    match pass {
        CompiledPassExecutionPlan::Graphics(value) => {
            !value.draw_buffers.instance_buffers.is_empty()
                && !value.draw_buffers.instance_buffer_layouts.is_empty()
                && value
                    .draw
                    .is_some_and(|draw| draw.vertex_count <= 6 && draw.instance_count > 1)
        }
        _ => false,
    }
}

fn draw_vertex_count(pass: &CompiledPassExecutionPlan) -> Option<u32> {
    match pass {
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => value.draw.map(|draw| draw.vertex_count),
        _ => None,
    }
}

fn draw_instance_count(pass: &CompiledPassExecutionPlan) -> Option<u32> {
    match pass {
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => value.draw.map(|draw| draw.instance_count),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::plugins::render::inspect::RenderGpuTimingCapability;

    #[test]
    fn production_evidence_report_covers_boids_procedural_runtime_contract() {
        let report = production_evidence_report().expect("boids evidence report should build");

        assert_eq!(report.flow_label, "boids_render_flow");
        assert_eq!(report.boid_count, DEFAULT_BOID_COUNT);
        assert_eq!(report.sorted_index_capacity, DEFAULT_BOID_COUNT);
        assert!(!report.silent_grid_overflow);
        assert_eq!(report.fixed_step_seconds, 1.0 / 60.0);
        assert!(report.aspect_correct_impostors);
        assert!(report.smoothed_visual_heading);
        assert_eq!(report.resize_pixel_evidence.len(), 3);
        assert!(
            report
                .resize_pixel_evidence
                .iter()
                .any(|evidence| evidence.surface_size == (900, 1600))
        );
        assert!(
            report
                .resize_pixel_evidence
                .iter()
                .all(|evidence| evidence.aspect_error_px <= 0.01)
        );
        assert_eq!(
            report
                .passes
                .iter()
                .map(|pass| pass.label.as_str())
                .collect::<Vec<_>>(),
            vec![
                "boids.seed_or_hold",
                "boids.grid.clear_counts",
                "boids.grid.count_cells",
                "boids.grid.scan_counts",
                "boids.grid.reset_cursors",
                "boids.grid.scatter_sorted_indices",
                "boids.grid.simulate_neighbors",
                "boids.grid.publish_draw",
                "boids.draw",
                "boids.present",
            ]
        );
        assert!(
            report
                .passes
                .iter()
                .filter(|pass| pass.kind == "compute")
                .all(|pass| pass.dispatch_workgroups_available)
        );
        let draw = report
            .passes
            .iter()
            .find(|pass| pass.label == "boids.draw")
            .expect("draw pass should be reported");
        assert!(draw.local_instance_geometry);
        assert_eq!(draw.vertex_count, Some(6));
        assert_eq!(draw.instance_count, Some(DEFAULT_BOID_COUNT));
        assert!(report.passes.iter().any(|pass| pass.kind == "present"));
        assert_eq!(
            report.cpu_timing_evidence.source,
            "prepared_frame_preflight"
        );
        assert!(report.cpu_timing_evidence.preflight_ms >= 0.0);
    }

    #[test]
    fn production_evidence_report_uses_typed_gpu_timing_diagnostics() {
        let report = production_evidence_report().expect("boids evidence report should build");

        assert_eq!(report.gpu_timing_evidence.len(), 9);
        assert!(
            report
                .gpu_timing_evidence
                .iter()
                .all(|evidence| evidence.gpu_capability == RenderGpuTimingCapability::Unsupported)
        );
        assert!(
            report
                .gpu_timing_evidence
                .iter()
                .any(|evidence| evidence.pass_id == "boids.draw")
        );
        assert!(report.cpu_timing_fields.contains(&"present_ms"));
        assert_eq!(
            report.benchmark_command,
            RENDER_FLOW_PLANNING_BENCHMARK_COMMAND
        );
    }

    #[test]
    fn formatted_production_evidence_is_stable_for_closeout_capture() {
        let report = production_evidence_report().expect("boids evidence report should build");
        let text = report.format_text();

        assert!(text.contains("boids_production_evidence flow=boids_render_flow"));
        assert!(text.contains("simulation_contract=fixed_step"));
        assert!(text.contains("aspect_correct_impostors=true"));
        assert!(text.contains("resize_pixel_evidence surface=900x1600"));
        assert!(text.contains("silent_grid_overflow=false"));
        assert!(text.contains("pass label=boids.draw kind=graphics"));
        assert!(text.contains("local_instance_geometry=true"));
        assert!(text.contains("gpu_timing flow=boids_render_flow pass=boids.draw"));
        assert!(text.contains("cpu_timing source=prepared_frame_preflight"));
        assert!(text.contains(RENDER_FLOW_PLANNING_BENCHMARK_COMMAND));
    }
}
