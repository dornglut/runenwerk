use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine::plugins::render::graph::merge_flow_with_contributions;
use engine::plugins::render::resource::{
    build_transient_alias_assignments, build_transient_windows, find_aliasable_transients,
};
use engine::plugins::render::{GpuStorage, GpuUniform, RenderFlow, RenderFlowContribution};

#[derive(Debug, Clone, Copy, GpuStorage)]
struct BoidInstance {
    position: [f32; 4],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComputeParams {
    tick: u32,
}

fn run_validation_and_planning(flow: &RenderFlow) {
    let report = flow.validate().expect("flow should validate");
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
        .import_texture("surface.color")
        .import_texture("scene.color")
        .fullscreen_pass("bench.compose")
        .reads("scene.color")
        .writes("surface.color")
        .finish()
}

fn build_boids_flow() -> RenderFlow {
    RenderFlow::new("bench.boids")
        .storage_buffer::<BoidInstance>("boids.instances")
        .color_target("surface.color")
        .compute_pass("boids.simulate")
        .writes("boids.instances")
        .finish()
        .graphics_pass("boids.draw")
        .vertex_buffer("boids.instances")
        .writes("surface.color")
        .depends_on("boids.simulate")
        .finish()
}

fn build_compositor_flow() -> RenderFlow {
    RenderFlow::new("bench.compositor")
        .import_texture("surface.color")
        .transient_color_target("post.a")
        .transient_color_target("post.b")
        .transient_color_target("post.c")
        .fullscreen_pass("post.extract")
        .reads("surface.color")
        .writes("post.a")
        .finish()
        .fullscreen_pass("post.blur_x")
        .reads("post.a")
        .writes("post.b")
        .depends_on("post.extract")
        .finish()
        .fullscreen_pass("post.blur_y")
        .reads("post.b")
        .writes("post.c")
        .depends_on("post.blur_x")
        .finish()
        .copy_pass("post.copy")
        .reads("post.c")
        .writes("surface.color")
        .depends_on("post.blur_y")
        .finish()
        .present_pass("post.present")
        .reads("surface.color")
        .depends_on("post.copy")
        .finish()
}

fn build_sdf_like_flow() -> RenderFlow {
    RenderFlow::new("bench.sdf")
        .uniform_buffer::<ComputeParams>("sdf.params")
        .storage_texture("sdf.field")
        .import_texture("surface.color")
        .compute_pass("sdf.compute")
        .reads("sdf.params")
        .write_texture("sdf.field")
        .finish()
        .fullscreen_pass("sdf.compose")
        .sample_texture("sdf.field")
        .writes("surface.color")
        .depends_on("sdf.compute")
        .finish()
}

fn build_mixed_contribution_flow() -> RenderFlow {
    let base = RenderFlow::new("bench.main")
        .import_texture("surface.color")
        .import_texture("ui.draw_list");

    let boids = RenderFlowContribution::new("boids")
        .storage_buffer::<BoidInstance>("boids.instances")
        .color_target("boids.color")
        .compute_pass("boids.simulate")
        .writes("boids.instances")
        .finish()
        .graphics_pass("boids.draw")
        .vertex_buffer("boids.instances")
        .writes("boids.color")
        .depends_on("boids.simulate")
        .finish();

    let post = RenderFlowContribution::new("post")
        .fullscreen_pass("post.tonemap")
        .sample_texture("boids.color")
        .writes("surface.color")
        .depends_on("boids.draw")
        .finish();

    let ui = RenderFlowContribution::new("ui")
        .builtin_ui_composite_pass("ui.composite")
        .reads("ui.draw_list")
        .writes("surface.color")
        .depends_on("post.tonemap")
        .finish();

    merge_flow_with_contributions(&base, &[boids, post, ui]).expect("merged flow should validate")
}

fn bench_render_flow_planning(c: &mut Criterion) {
    let fullscreen = build_simple_fullscreen_flow();
    c.bench_function("render_flow/simple_fullscreen", |b| {
        b.iter(|| run_validation_and_planning(black_box(&fullscreen)))
    });

    let boids = build_boids_flow();
    c.bench_function("render_flow/boids_sim_draw", |b| {
        b.iter(|| run_validation_and_planning(black_box(&boids)))
    });

    let compositor = build_compositor_flow();
    c.bench_function("render_flow/multi_pass_compositor", |b| {
        b.iter(|| run_validation_and_planning(black_box(&compositor)))
    });

    let sdf = build_sdf_like_flow();
    c.bench_function("render_flow/sdf_compute_compose", |b| {
        b.iter(|| run_validation_and_planning(black_box(&sdf)))
    });

    c.bench_function("render_flow/mixed_contribution_merge", |b| {
        b.iter(|| {
            let merged = build_mixed_contribution_flow();
            run_validation_and_planning(black_box(&merged));
        })
    });
}

criterion_group!(render_flow_planning, bench_render_flow_planning);
criterion_main!(render_flow_planning);
