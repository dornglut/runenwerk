use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine::plugins::render::graph::{
    CompiledPassExecutionPlan, CompiledRenderFlowPlan, RenderBackendCapabilityProfile,
};
use engine::plugins::render::resource::{
    build_transient_alias_assignments, build_transient_windows, find_aliasable_transients,
};
use engine::plugins::render::{
    GpuStorage, GpuUniform, PreparedFlowInputs, PreparedFlowInvocation, PreparedFrameContext,
    PreparedFrameContributions, PreparedRenderFrame, PreparedShaderSnapshot, PreparedSurfaceInfo,
    PreparedViewFrame, RenderExecutionGraphPreparedReport, RenderFlow,
    RenderPreparedFramePreflightCacheKey, compile_flow_plan, preflight_prepared_render_frame,
    preflight_prepared_render_frame_runtime_guards, prepared_render_frame_preflight_cache_key,
};
use engine::prelude::Resource;
use ui_render_data::ViewportSurfaceBindingRegistry;

#[derive(Debug, Clone, Copy, GpuStorage)]
struct BoidInstance {
    position: [f32; 4],
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
}

criterion_group!(render_flow_planning, bench_render_flow_planning);
criterion_main!(render_flow_planning);
