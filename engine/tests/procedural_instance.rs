use engine::plugins::render::{CompiledPassExecutionPlan, RenderFlow};
use engine::plugins::render::{
    GpuStorage, ProceduralBufferBinding, ProceduralPassDescriptor, ProceduralRenderPolicy,
    ProceduralValidationError, RenderBackendCapabilityProfile, RenderBlendMode, RenderCullMode,
    RenderDepthPolicy, RenderPassKind, RenderPrimitiveTopology, RenderVertexBufferLayout,
    RenderVertexFormat, SURFACE_COLOR_RESOURCE_LABEL, compile_flow_plan_checked,
};

#[derive(Debug, Clone, Copy, GpuStorage)]
struct Instance {
    position: [f32; 2],
    radius: f32,
    flags: u32,
}

#[derive(Debug, Clone, Copy, GpuStorage)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

fn instance_layout(slot: u32) -> RenderVertexBufferLayout {
    RenderVertexBufferLayout::instance(slot, 16)
        .attribute(0, 0, RenderVertexFormat::Float32x2)
        .attribute(1, 8, RenderVertexFormat::Float32)
        .attribute(2, 12, RenderVertexFormat::Uint32)
}

fn vertex_layout(slot: u32) -> RenderVertexBufferLayout {
    RenderVertexBufferLayout::vertex(slot, 20)
        .attribute(3, 0, RenderVertexFormat::Float32x3)
        .attribute(4, 12, RenderVertexFormat::Float32x2)
}

#[test]
fn procedural_instance_quad_sprite_descriptor_builds_pass_shape_safe_graphics_pass() {
    let (flow, instances) = RenderFlow::new("procedural.quad")
        .with_surface_color()
        .storage_array::<Instance>("instances", 64);

    let flow = flow
        .procedural_pass(
            ProceduralPassDescriptor::quad_sprites(
                "draw.quads",
                ProceduralBufferBinding::storage(instances, instance_layout(0)),
                64,
            )
            .shader_asset("assets/shaders/procedural_quad.wgsl")
            .write_color_target(SURFACE_COLOR_RESOURCE_LABEL),
        )
        .expect("valid quad sprite descriptor should build")
        .validate()
        .expect("procedural flow should validate");

    let pass = flow
        .graph()
        .passes
        .passes
        .iter()
        .find(|pass| pass.label == "draw.quads")
        .expect("procedural pass should exist");

    assert_eq!(pass.kind, RenderPassKind::Graphics);
    assert_eq!(pass.instance_buffers.len(), 1);
    assert_eq!(pass.instance_buffer_layouts.len(), 1);
    assert_eq!(pass.draw.expect("draw descriptor").vertex_count, 6);
    assert_eq!(pass.draw.expect("draw descriptor").instance_count, 64);

    compile_flow_plan_checked(&flow, &RenderBackendCapabilityProfile::runtime_default())
        .expect("local instance geometry should satisfy pass-shape guards");
}

#[test]
fn procedural_instance_mesh_sprite_descriptor_preserves_vertex_and_instance_buffers() {
    let (flow, vertices) = RenderFlow::new("procedural.mesh")
        .with_surface_color()
        .storage_array::<Vertex>("vertices", 6);
    let (flow, instances) = flow.storage_array::<Instance>("instances", 16);

    let flow = flow
        .procedural_pass(
            ProceduralPassDescriptor::mesh_sprites(
                "draw.mesh",
                ProceduralBufferBinding::storage(vertices, vertex_layout(0)),
                6,
                ProceduralBufferBinding::storage(instances, instance_layout(1)),
                16,
            )
            .shader_asset("assets/shaders/procedural_mesh.wgsl")
            .write_color_target(SURFACE_COLOR_RESOURCE_LABEL),
        )
        .expect("valid mesh sprite descriptor should build");

    let pass = flow
        .graph()
        .passes
        .passes
        .iter()
        .find(|pass| pass.label == "draw.mesh")
        .expect("procedural pass should exist");

    assert_eq!(pass.vertex_buffers.len(), 1);
    assert_eq!(pass.vertex_buffer_layouts.len(), 1);
    assert_eq!(pass.instance_buffers.len(), 1);
    assert_eq!(pass.instance_buffer_layouts.len(), 1);
}

#[test]
fn procedural_instance_local_sdf_2d_impostor_descriptor_is_local_2d_and_pass_shape_safe() {
    let (flow, instances) = RenderFlow::new("procedural.sdf2d")
        .with_surface_color()
        .storage_array::<Instance>("instances", 32);

    let flow = flow
        .procedural_pass(
            ProceduralPassDescriptor::local_sdf_2d_impostors(
                "draw.sdf2d",
                ProceduralBufferBinding::storage(instances, instance_layout(0)),
                32,
            )
            .shader_asset("assets/shaders/procedural_sdf2d.wgsl")
            .write_color_target(SURFACE_COLOR_RESOURCE_LABEL),
        )
        .expect("valid local 2D SDF impostor descriptor should build");

    compile_flow_plan_checked(&flow, &RenderBackendCapabilityProfile::runtime_default())
        .expect("local SDF impostor instance geometry should satisfy pass-shape guards");
}

#[test]
fn procedural_instance_policy_survives_compilation() {
    let (flow, instances) = RenderFlow::new("procedural.policy")
        .with_surface_color()
        .with_depth_target("procedural.depth")
        .storage_array::<Instance>("instances", 8);

    let policy = ProceduralRenderPolicy::default()
        .blend_mode(RenderBlendMode::Replace)
        .depth_policy(RenderDepthPolicy::ReadWrite)
        .cull_mode(RenderCullMode::Back)
        .primitive_topology(RenderPrimitiveTopology::TriangleList);

    let flow = flow
        .procedural_pass(
            ProceduralPassDescriptor::quad_sprites(
                "draw.policy",
                ProceduralBufferBinding::storage(instances, instance_layout(0)),
                8,
            )
            .shader_asset("assets/shaders/procedural_policy.wgsl")
            .target(
                engine::plugins::render::ProceduralTargetDescriptor::color(
                    SURFACE_COLOR_RESOURCE_LABEL,
                )
                .depth_target("procedural.depth"),
            )
            .policy(policy),
        )
        .expect("valid policy descriptor should build");

    let plan = compile_flow_plan_checked(&flow, &RenderBackendCapabilityProfile::runtime_default())
        .expect("procedural policy flow should compile");
    let raster = plan
        .execution
        .passes
        .iter()
        .find_map(|pass| match pass {
            CompiledPassExecutionPlan::Graphics(raster) if raster.pass_id.to_string() == "2" => {
                Some(raster)
            }
            CompiledPassExecutionPlan::Graphics(raster) => Some(raster),
            _ => None,
        })
        .expect("compiled graphics pass should exist");

    assert_eq!(
        raster.raster_state.state.blend_mode,
        RenderBlendMode::Replace
    );
    assert_eq!(
        raster.raster_state.state.depth_policy,
        RenderDepthPolicy::ReadWrite
    );
    assert_eq!(raster.raster_state.state.cull_mode, RenderCullMode::Back);
}

#[test]
fn procedural_instance_descriptor_rejects_vertex_step_mode_for_instance_buffer() {
    let (flow, instances) = RenderFlow::new("procedural.invalid")
        .with_surface_color()
        .storage_array::<Instance>("instances", 4);

    let err = flow
        .procedural_pass(
            ProceduralPassDescriptor::quad_sprites(
                "draw.invalid",
                ProceduralBufferBinding::storage(
                    instances,
                    RenderVertexBufferLayout::vertex(0, 16).attribute(
                        0,
                        0,
                        RenderVertexFormat::Float32x2,
                    ),
                ),
                4,
            )
            .shader_asset("assets/shaders/procedural_invalid.wgsl")
            .write_color_target(SURFACE_COLOR_RESOURCE_LABEL),
        )
        .expect_err("instance buffers must use instance step mode");

    assert!(matches!(
        err,
        ProceduralValidationError::InvalidStepMode {
            role: "instance",
            ..
        }
    ));
}

#[test]
fn procedural_instance_descriptor_rejects_depth_policy_without_depth_target() {
    let (flow, instances) = RenderFlow::new("procedural.invalid_depth")
        .with_surface_color()
        .storage_array::<Instance>("instances", 4);

    let err = flow
        .procedural_pass(
            ProceduralPassDescriptor::quad_sprites(
                "draw.invalid_depth",
                ProceduralBufferBinding::storage(instances, instance_layout(0)),
                4,
            )
            .shader_asset("assets/shaders/procedural_invalid_depth.wgsl")
            .write_color_target(SURFACE_COLOR_RESOURCE_LABEL)
            .policy(ProceduralRenderPolicy::default().depth_policy(RenderDepthPolicy::ReadWrite)),
        )
        .expect_err("read/write depth policy requires a depth target");

    assert!(matches!(
        err,
        ProceduralValidationError::MissingDepthTargetForPolicy { .. }
    ));
}
