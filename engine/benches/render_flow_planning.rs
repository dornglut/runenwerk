use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine::plugins::render::resource::{
    build_transient_alias_assignments, build_transient_windows, find_aliasable_transients,
};
use engine::plugins::render::{GpuStorage, GpuUniform, RenderFlow};
use engine::prelude::Resource;

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
