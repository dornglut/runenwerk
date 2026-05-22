use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine::plugins::render::features::world::sdf_raymarch::{
    RenderSdfDistanceMipLevel, RenderSdfRaymarchAccelerationReport, RenderSdfRaymarchCandidate,
    RenderSdfRaymarchCandidateList,
};
use engine::plugins::render::graph::{
    CompiledPassExecutionPlan, CompiledRenderFlowPlan, RenderBackendCapabilityProfile,
};
use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuResidencyBudgetInspection,
    RenderGpuResidencyInspection, RenderGpuTimingCapability, RenderGpuTimingDiagnostic,
    RenderPassTimingEvidence, RenderScaleProductionEvidenceRequest,
    RenderScaleProductionHardwareProfile, RenderScaleVisibilityCandidate,
    RenderScaleVisibilityCapabilities, RenderScaleVisibilityCapabilityStatus,
    RenderSdfProductionEvidenceRequest, RenderSdfProductionHardwareProfile,
    RenderSdfResidencyBudgetInspection, RenderSdfResidencyInspection,
    RenderSdfRuntimeVisualEvidence, inspect_render_scale_production_evidence,
    inspect_render_scale_visibility, inspect_render_sdf_production_evidence,
};
use engine::plugins::render::resource::{
    build_transient_alias_assignments, build_transient_windows, find_aliasable_transients,
};
use engine::plugins::render::{
    GpuStorage, GpuUniform, PreparedFlowInputs, PreparedFlowInvocation, PreparedFrameContext,
    PreparedFrameContributions, PreparedRenderFrame, PreparedShaderSnapshot, PreparedSurfaceInfo,
    PreparedViewFrame, ProceduralBufferBinding, ProceduralPassDescriptor,
    ProceduralTargetDescriptor, RenderExecutionGraphPreparedReport, RenderFlow,
    RenderPreparedFramePreflightCacheKey, RenderVertexBufferLayout, RenderVertexFormat,
    compile_flow_plan, preflight_prepared_render_frame,
    preflight_prepared_render_frame_runtime_guards, prepared_render_frame_preflight_cache_key,
};
use engine::prelude::Resource;
use ui_render_data::ViewportSurfaceBindingRegistry;

#[derive(Debug, Clone, Copy, GpuStorage)]
struct BoidInstance {
    position: [f32; 4],
}

#[derive(Debug, Clone, Copy, GpuStorage)]
struct ProceduralBoidInstance {
    position: [f32; 2],
    velocity: [f32; 2],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComputeParams {
    tick: u32,
    step: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComposeParams {
    surface_size: [u32; 2],
    intensity: f32,
}

#[derive(Debug, Clone, Resource)]
struct BenchState {
    tick: u32,
    dispatch: [u32; 3],
}

impl Default for BenchState {
    fn default() -> Self {
        Self {
            tick: 1,
            dispatch: [16, 16, 1],
        }
    }
}

impl BenchState {
    fn compute_params(&self) -> ComputeParams {
        ComputeParams {
            tick: self.tick,
            step: 1,
        }
    }

    fn compose_params(&self, surface: (u32, u32)) -> ComposeParams {
        ComposeParams {
            surface_size: [surface.0, surface.1],
            intensity: 1.0,
        }
    }

    fn dispatch_workgroups(&self) -> [u32; 3] {
        self.dispatch
    }
}

fn run_validation_and_planning(flow: &RenderFlow) {
    let report = flow.validation_report().expect("flow should validate");
    let windows = build_transient_windows(flow.graph());
    let alias_candidates = find_aliasable_transients(&windows);
    let alias_assignments = build_transient_alias_assignments(&windows);
    black_box(report.pass_order.len());
    black_box(windows.len());
    black_box(alias_candidates.len());
    black_box(alias_assignments.len());
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
                        .insert(value.pass_id, [16, 16, 1]);
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

fn prepared_frame_for_flow(compiled: &CompiledRenderFlowPlan) -> PreparedRenderFrame {
    PreparedRenderFrame {
        context: PreparedFrameContext {
            frame_index: 1,
            flow_registry_revision: 1,
            shader_registry_revision: 1,
            prepare_epoch: 1,
        },
        surface: PreparedSurfaceInfo::primary((1280, 720)),
        views: vec![PreparedViewFrame::main((1280, 720))],
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

fn run_cold_preflight(compiled: &CompiledRenderFlowPlan, frame: &PreparedRenderFrame) {
    let profile = RenderBackendCapabilityProfile::runtime_default();
    let key =
        prepared_render_frame_preflight_cache_key(frame, std::slice::from_ref(compiled), &profile);
    preflight_prepared_render_frame_runtime_guards(frame, std::slice::from_ref(compiled))
        .expect("runtime guards should pass");
    let report = preflight_prepared_render_frame(frame, std::slice::from_ref(compiled), &profile)
        .expect("prepared frame should preflight");
    black_box(key.prepared_structure_hash);
    black_box(report.error_count());
}

fn run_warm_cached_preflight(
    compiled: &CompiledRenderFlowPlan,
    frame: &PreparedRenderFrame,
    cached_key: &RenderPreparedFramePreflightCacheKey,
    cached_report: &RenderExecutionGraphPreparedReport,
) {
    let profile = RenderBackendCapabilityProfile::runtime_default();
    preflight_prepared_render_frame_runtime_guards(frame, std::slice::from_ref(compiled))
        .expect("runtime guards should pass");
    let key =
        prepared_render_frame_preflight_cache_key(frame, std::slice::from_ref(compiled), &profile);
    if &key == cached_key {
        black_box(cached_report.error_count());
    } else {
        run_cold_preflight(compiled, frame);
    }
}

fn run_scale_production_evidence(candidate_count: usize) {
    let residency = RenderGpuResidencyInspection {
        addressable_count: 1_000_000,
        selected_count: candidate_count,
        requested_count: candidate_count,
        accepted_count: candidate_count,
        resident_count: candidate_count,
        allocated_count: candidate_count,
        preserved_count: 0,
        invalidated_count: 0,
        evicted_count: 0,
        rejected_count: 0,
        resident_bytes: candidate_count as u64 * 4096,
        upload_bytes: candidate_count.min(4096) as u64 * 1024,
        budget: RenderGpuResidencyBudgetInspection {
            max_resident_entries: 1_000_000,
            max_resident_bytes: 4_u64 * 1024 * 1024 * 1024,
            max_upload_bytes_per_frame: 128_u64 * 1024 * 1024,
            resident_entry_status: "within_budget".to_string(),
            resident_byte_status: "within_budget".to_string(),
            upload_byte_status: "within_budget".to_string(),
            hard_pinned_over_entry_budget: false,
        },
        diagnostic_count: 0,
        entries: Vec::new(),
        journal: Vec::new(),
    };
    let candidates = (0..candidate_count)
        .map(|index| RenderScaleVisibilityCandidate {
            product_id: index as u64,
            cache_id: format!("scale.chunk.{index}"),
            center: [((index % 64) as f32 / 64.0) - 0.5, 0.0, 0.0],
            radius: 0.01,
            screen_size_px: if index % 4 == 0 { 0.25 } else { 64.0 },
            resident_bytes: 4096,
        })
        .collect::<Vec<_>>();
    let visibility = inspect_render_scale_visibility(
        &candidates,
        Default::default(),
        RenderScaleVisibilityCapabilities::supported(),
    );
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[PassTimingSample {
        flow_id: "bench.scale".to_string(),
        pass_id: "scale.visibility".to_string(),
        pass_kind: "compute".to_string(),
        millis: 0.5,
        dispatch_workgroups: Some([64, 1, 1]),
    }]);
    timings.observe_gpu_pass_timing_evidence(&[RenderPassTimingEvidence::gpu_sample(
        Some(1),
        Some(1),
        "bench.scale",
        "scale.visibility",
        "compute",
        0.45,
    )]);
    let report = inspect_render_scale_production_evidence(RenderScaleProductionEvidenceRequest {
        hardware_profile: RenderScaleProductionHardwareProfile {
            profile_key: "bench-scale-profile".to_string(),
            adapter_name: Some("criterion".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Supported,
            storage_compaction: RenderScaleVisibilityCapabilityStatus::Supported,
            indirect_submission: RenderScaleVisibilityCapabilityStatus::Supported,
            readback: RenderScaleVisibilityCapabilityStatus::Supported,
        },
        residency,
        visibility,
        timings,
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-scale-evidence/criterion.md".to_string(),
        ],
    });
    black_box(report.is_runtime_ready());
    black_box(report.counts.submitted_draw_count);
    black_box(report.timings.gpu_total_pass_millis);
}

fn run_sdf_runtime_evidence(product_count: usize) {
    let residency = RenderSdfResidencyInspection {
        addressable_product_count: product_count,
        selected_product_count: product_count,
        requested_product_count: product_count,
        resident_product_count: product_count,
        resident_page_count: product_count * 2,
        resident_brick_count: product_count * 2,
        clipmap_window_count: product_count,
        invalidated_product_count: 0,
        rejected_product_count: 0,
        resident_bytes: product_count as u64 * 8192,
        upload_bytes: product_count as u64 * 2048,
        budget: RenderSdfResidencyBudgetInspection {
            page_status: "within_budget".to_string(),
            brick_status: "within_budget".to_string(),
            resident_byte_status: "within_budget".to_string(),
            upload_byte_status: "within_budget".to_string(),
            clipmap_page_status: "within_budget".to_string(),
        },
        diagnostic_count: 0,
        diagnostics: Vec::new(),
        entries: Vec::new(),
        clipmap_windows: Vec::new(),
    };
    let candidates = (0..product_count)
        .map(|index| RenderSdfRaymarchCandidate {
            product_id: index as u64,
            cache_generation: index as u64 + 1,
            scale_band: match index % 4 {
                0 => "Near",
                1 => "Mid",
                2 => "Far",
                _ => "Summary",
            }
            .to_string(),
            page_count: 2,
            brick_count: 2,
            resident_bytes: 8192,
        })
        .collect::<Vec<_>>();
    let raymarch = RenderSdfRaymarchAccelerationReport {
        resident_product_count: product_count,
        resident_page_count: product_count * 2,
        resident_brick_count: product_count * 2,
        clipmap_window_count: product_count,
        distance_mips: (0..product_count.min(8))
            .map(|level| RenderSdfDistanceMipLevel {
                level: u8::try_from(level).expect("bench mip level fits in u8"),
                source_page_count: 2,
                source_brick_count: 2,
                conservative_min_distance: 0.0,
                max_safe_step: 1.0 / (level as f32 + 1.0),
                unsafe_overstep_risk: false,
            })
            .collect(),
        candidate_lists: vec![RenderSdfRaymarchCandidateList {
            tile_index: 0,
            depth_slice: 0,
            candidate_count: candidates.len(),
            rejected_candidate_count: 0,
            candidates,
        }],
        total_candidate_count: product_count,
        rejected_candidate_count: 0,
        max_candidates_per_list: product_count.max(1),
        max_steps_per_ray: 160,
        fullscreen_entity_multiplier: 1,
        diagnostics: Vec::new(),
    };
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[PassTimingSample {
        flow_id: "bench.sdf.runtime".to_string(),
        pass_id: "sdf.compose".to_string(),
        pass_kind: "fullscreen".to_string(),
        millis: 0.5,
        dispatch_workgroups: None,
    }]);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "criterion SDF runtime evidence uses portable unsupported timestamp diagnostics",
    ));
    let visual_evidence = ["near", "mid", "far", "summary"]
        .into_iter()
        .map(|coverage_band| RenderSdfRuntimeVisualEvidence {
            view_label: format!("bench.{coverage_band}"),
            coverage_band: coverage_band.to_string(),
            artifact_path: format!(
                "engine/benchmark-artifacts/render-sdf-runtime-evidence/{coverage_band}.txt"
            ),
            step_count: 64,
            missed_surface_risk: false,
            overstep_risk: false,
        })
        .collect::<Vec<_>>();
    let report = inspect_render_sdf_production_evidence(RenderSdfProductionEvidenceRequest {
        hardware_profile: RenderSdfProductionHardwareProfile {
            profile_key: "bench-sdf-runtime-profile".to_string(),
            adapter_name: Some("criterion".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        residency,
        raymarch,
        timings,
        visual_evidence,
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-sdf-runtime-evidence/criterion.md".to_string(),
        ],
    });
    black_box(report.is_runtime_ready());
    black_box(report.counts.total_candidate_count);
    black_box(report.timings.cpu_total_pass_millis);
}

fn build_simple_fullscreen_flow() -> RenderFlow {
    RenderFlow::new("bench.fullscreen")
        .with_surface_color()
        .fullscreen_pass("bench.compose")
        .write_surface_color()
        .finish()
        .validate()
        .expect("fullscreen flow should validate")
}

fn build_boids_flow() -> RenderFlow {
    RenderFlow::new("bench.boids")
        .with_state::<BenchState>()
        .double_buffer_storage_array::<BoidInstance>("boids.instances", 4096)
        .compute_pass("boids.simulate")
        .bind_ping_pong_storage("boids.instances")
        .uniform_from_state(BenchState::compute_params)
        .dispatch_from_state(BenchState::dispatch_workgroups)
        .finish()
        .validate()
        .expect("boids flow should validate")
}

fn build_procedural_boids_flow() -> RenderFlow {
    let (flow, boid_instances) = RenderFlow::new("bench.procedural_boids")
        .with_state::<BenchState>()
        .with_surface_color()
        .with_color_target("bench.procedural_boids.color")
        .double_buffer_storage_array_with_handle::<ProceduralBoidInstance>(
            "bench.procedural_boids.instances",
            4096,
        );
    let instance_buffer = ProceduralBufferBinding::storage(
        boid_instances.a().clone(),
        procedural_boid_instance_layout(),
    );

    let flow = flow
        .compute_pass("bench.procedural_boids.simulate")
        .bind_ping_pong_storage(boid_instances.name())
        .uniform_from_state(BenchState::compute_params)
        .dispatch_from_state(BenchState::dispatch_workgroups)
        .finish()
        .compute_pass("bench.procedural_boids.publish")
        .bind_ping_pong_storage(boid_instances.name())
        .uniform_from_state(BenchState::compute_params)
        .dispatch_from_state(BenchState::dispatch_workgroups)
        .depends_on("bench.procedural_boids.simulate")
        .finish();

    let flow = flow
        .procedural_pass(
            ProceduralPassDescriptor::local_sdf_2d_impostors(
                "bench.procedural_boids.draw",
                instance_buffer,
                4096,
            )
            .shader_asset("assets/shaders/boids_compose.wgsl")
            .target(ProceduralTargetDescriptor::color(
                "bench.procedural_boids.color",
            ))
            .depends_on("bench.procedural_boids.publish"),
        )
        .expect("procedural boids pass should be valid");

    flow.present_pass("bench.procedural_boids.present")
        .source("bench.procedural_boids.color")
        .depends_on("bench.procedural_boids.draw")
        .finish()
        .validate()
        .expect("procedural boids flow should validate")
}

fn procedural_boid_instance_layout() -> RenderVertexBufferLayout {
    RenderVertexBufferLayout::instance(0, std::mem::size_of::<ProceduralBoidInstance>() as u64)
        .attribute(0, 0, RenderVertexFormat::Float32x2)
        .attribute(1, 8, RenderVertexFormat::Float32x2)
}

fn build_compositor_flow() -> RenderFlow {
    RenderFlow::new("bench.compositor")
        .with_state::<BenchState>()
        .with_surface_color()
        .with_builtin_ui()
        .double_buffer_storage_array::<BoidInstance>("post.history", 2048)
        .compute_pass("post.extract")
        .bind_ping_pong_storage("post.history")
        .uniform_from_state(BenchState::compute_params)
        .dispatch([8, 8, 1])
        .finish()
        .fullscreen_pass("post.compose")
        .bind_ping_pong_storage("post.history")
        .uniform_from_state_with_surface(BenchState::compose_params)
        .write_surface_color()
        .depends_on("post.extract")
        .finish()
        .builtin_ui_composite_pass("post.ui")
        .depends_on("post.compose")
        .finish()
        .validate()
        .expect("compositor flow should validate")
}

fn build_sdf_like_flow() -> RenderFlow {
    let (flow, field) =
        RenderFlow::new("bench.sdf").storage_array::<BoidInstance>("sdf.field", 2048);
    flow.with_state::<BenchState>()
        .with_surface_color()
        .compute_pass("sdf.compute")
        .bind_storage(field.clone())
        .uniform_from_state(BenchState::compute_params)
        .dispatch([8, 8, 1])
        .finish()
        .fullscreen_pass("sdf.compose")
        .bind_storage(field)
        .uniform_from_state_with_surface(BenchState::compose_params)
        .write_surface_color()
        .depends_on("sdf.compute")
        .finish()
        .validate()
        .expect("sdf flow should validate")
}

fn build_mixed_ui_flow() -> RenderFlow {
    RenderFlow::new("bench.mixed_ui")
        .with_state::<BenchState>()
        .with_surface_color()
        .with_builtin_ui()
        .double_buffer_storage_array::<BoidInstance>("mixed.cells", 1024)
        .compute_pass("mixed.simulate")
        .bind_ping_pong_storage("mixed.cells")
        .uniform_from_state(BenchState::compute_params)
        .dispatch_from_state(BenchState::dispatch_workgroups)
        .finish()
        .fullscreen_pass("mixed.compose")
        .bind_ping_pong_storage("mixed.cells")
        .uniform_from_state_with_surface(BenchState::compose_params)
        .write_surface_color()
        .depends_on("mixed.simulate")
        .finish()
        .builtin_ui_composite_pass("mixed.ui")
        .depends_on("mixed.compose")
        .finish()
        .validate()
        .expect("mixed ui flow should validate")
}

fn bench_render_flow_planning(c: &mut Criterion) {
    let fullscreen = build_simple_fullscreen_flow();
    c.bench_function("render_flow/simple_fullscreen", |b| {
        b.iter(|| run_validation_and_planning(black_box(&fullscreen)))
    });

    let boids = build_boids_flow();
    c.bench_function("render_flow/boids_ping_pong", |b| {
        b.iter(|| run_validation_and_planning(black_box(&boids)))
    });
    let compiled_boids = compile_flow_plan(&boids).expect("boids flow should compile");
    let boids_frame = prepared_frame_for_flow(&compiled_boids);
    let profile = RenderBackendCapabilityProfile::runtime_default();
    let boids_preflight_key = prepared_render_frame_preflight_cache_key(
        &boids_frame,
        std::slice::from_ref(&compiled_boids),
        &profile,
    );
    let boids_preflight_report = preflight_prepared_render_frame(
        &boids_frame,
        std::slice::from_ref(&compiled_boids),
        &profile,
    )
    .expect("boids prepared frame should preflight");
    c.bench_function("render_flow/boids_preflight_cold", |b| {
        b.iter(|| run_cold_preflight(black_box(&compiled_boids), black_box(&boids_frame)))
    });
    c.bench_function("render_flow/boids_preflight_cached", |b| {
        b.iter(|| {
            run_warm_cached_preflight(
                black_box(&compiled_boids),
                black_box(&boids_frame),
                black_box(&boids_preflight_key),
                black_box(&boids_preflight_report),
            )
        })
    });

    let procedural_boids = build_procedural_boids_flow();
    c.bench_function("render_flow/procedural_boids_production_shape", |b| {
        b.iter(|| run_validation_and_planning(black_box(&procedural_boids)))
    });
    let compiled_procedural_boids =
        compile_flow_plan(&procedural_boids).expect("procedural boids flow should compile");
    let procedural_boids_frame = prepared_frame_for_flow(&compiled_procedural_boids);
    c.bench_function("render_flow/procedural_boids_preflight_cold", |b| {
        b.iter(|| {
            run_cold_preflight(
                black_box(&compiled_procedural_boids),
                black_box(&procedural_boids_frame),
            )
        })
    });

    let compositor = build_compositor_flow();
    c.bench_function("render_flow/multi_pass_compute_compose", |b| {
        b.iter(|| run_validation_and_planning(black_box(&compositor)))
    });

    let sdf = build_sdf_like_flow();
    c.bench_function("render_flow/sdf_compute_compose", |b| {
        b.iter(|| run_validation_and_planning(black_box(&sdf)))
    });

    let mixed_ui = build_mixed_ui_flow();
    c.bench_function("render_flow/mixed_ui_chain", |b| {
        b.iter(|| {
            run_validation_and_planning(black_box(&mixed_ui));
        })
    });

    c.bench_function("render_scale/production_evidence_report_4096", |b| {
        b.iter(|| run_scale_production_evidence(black_box(4096)))
    });
    c.bench_function("render_sdf/runtime_evidence_report_4096", |b| {
        b.iter(|| run_sdf_runtime_evidence(black_box(4096)))
    });
}

criterion_group!(render_flow_planning, bench_render_flow_planning);
criterion_main!(render_flow_planning);
