use anyhow::Result;
use engine::plugins::render::{GpuStorage, RenderFlow};

#[derive(Debug, Clone, Copy, GpuStorage)]
struct BoidInstance {
    position: [f32; 4],
    velocity: [f32; 4],
}

fn main() -> Result<()> {
    let flow = RenderFlow::new("boids.flow")
        .storage_buffer::<BoidInstance>("boids.instances")
        .color_target("boids.color")
        .import_texture("surface.color")
        .compute_pass("boids.simulate")
        .writes("boids.instances")
        .finish()
        .graphics_pass("boids.draw")
        .instance_buffer("boids.instances")
        .writes("boids.color")
        .depends_on("boids.simulate")
        .finish()
        .copy_pass("boids.copy_to_surface")
        .reads("boids.color")
        .writes("surface.color")
        .depends_on("boids.draw")
        .finish()
        .present_pass("boids.present")
        .reads("surface.color")
        .depends_on("boids.copy_to_surface")
        .finish();

    let order = flow.pass_order()?;
    println!("boids flow order: {}", order.join(" -> "));
    Ok(())
}
