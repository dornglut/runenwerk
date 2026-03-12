use engine::plugins::render::{GpuStorage, RenderFlow};

#[derive(Debug, Clone, Copy, GpuStorage)]
struct BoidInstance {
    position: [f32; 4],
}

#[test]
fn compute_to_graphics_buffer_chain_validates() {
    let flow = RenderFlow::new("boids.flow")
        .storage_buffer::<BoidInstance>("boids.instances")
        .color_target("surface.color")
        .compute_pass("boids.simulate")
        .writes("boids.instances")
        .finish()
        .graphics_pass("boids.draw")
        .vertex_buffer("boids.instances")
        .writes("surface.color")
        .depends_on("boids.simulate")
        .finish();

    let report = flow.validate().expect("flow should validate");
    assert_eq!(report.pass_order, vec!["boids.simulate", "boids.draw"]);
}

#[test]
fn graphics_pass_depth_target_validation() {
    let flow = RenderFlow::new("scene.flow")
        .storage_buffer::<BoidInstance>("scene.mesh.instances")
        .color_target("surface.color")
        .depth_target("scene.depth")
        .graphics_pass("scene.draw")
        .vertex_buffer("scene.mesh.instances")
        .depth_target("scene.depth")
        .writes("surface.color")
        .finish();

    flow.validate().expect("depth target should validate");
}

#[test]
fn graphics_instance_buffer_consumption_validates() {
    let flow = RenderFlow::new("instance.flow")
        .storage_buffer::<BoidInstance>("instance.data")
        .color_target("surface.color")
        .graphics_pass("instance.draw")
        .instance_buffer("instance.data")
        .writes("surface.color")
        .finish();

    flow.validate()
        .expect("instance buffer usage should validate");
}
