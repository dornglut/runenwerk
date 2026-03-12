use anyhow::Result;
use engine::plugins::render::graph::merge_flow_with_contributions;
use engine::plugins::render::{GpuStorage, GpuUniform, RenderFlow, RenderFlowContribution};

#[derive(Debug, Clone, Copy, GpuStorage)]
struct BoidInstance {
    position: [f32; 4],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ToneMapParams {
    exposure: f32,
    gamma: f32,
}

#[derive(Debug, Clone, ecs::Component)]
struct ToneMapState {
    exposure: f32,
    gamma: f32,
}

impl ToneMapState {
    fn params(&self) -> ToneMapParams {
        ToneMapParams {
            exposure: self.exposure,
            gamma: self.gamma,
        }
    }
}

fn main() -> Result<()> {
    let base = RenderFlow::new("main.flow")
        .import_texture("surface.color")
        .import_texture("ui.draw_list");

    let boids = RenderFlowContribution::new("boids")
        .storage_buffer::<BoidInstance>("boids.instances")
        .color_target("boids.color")
        .compute_pass("boids.simulate")
        .shader("assets/shaders/boids_sim.wgsl")
        .writes("boids.instances")
        .finish()
        .graphics_pass("boids.draw")
        .shader("assets/shaders/boids_draw.wgsl")
        .vertex_buffer("boids.instances")
        .writes("boids.color")
        .depends_on("boids.simulate")
        .finish();

    let post = RenderFlowContribution::new("post")
        .ecs_resource::<ToneMapState>()
        .uniform_buffer::<ToneMapParams>("post.tonemap.params")
        .history_texture("post.history")
        .fullscreen_pass("post.tonemap")
        .shader("assets/shaders/tonemap.wgsl")
        .uniform_state(ToneMapState::params)
        .sample_texture("boids.color")
        .writes("surface.color")
        .depends_on("boids.draw")
        .finish()
        .copy_pass("post.history_update")
        .reads("surface.color")
        .writes("post.history")
        .depends_on("post.tonemap")
        .finish();

    let ui = RenderFlowContribution::new("ui")
        .builtin_ui_composite_pass("ui.composite")
        .reads("ui.draw_list")
        .writes("surface.color")
        .depends_on("post.tonemap")
        .finish();

    let merged = merge_flow_with_contributions(&base, &[boids, post, ui])?;
    let order = merged.pass_order()?;
    println!("contribution flow order: {}", order.join(" -> "));
    Ok(())
}
