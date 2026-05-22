use crate::rendering::{BoidAgent, BoidsRenderState, DEFAULT_BOID_COUNT};
use engine::plugins::render::{
    ProceduralBufferBinding, ProceduralPassDescriptor, ProceduralTargetDescriptor, RenderFlow,
    RenderVertexBufferLayout, RenderVertexFormat,
};

pub(crate) fn build_render_flow() -> RenderFlow {
    let (flow, boid_instances) = RenderFlow::new("boids_render_flow")
        .with_state::<BoidsRenderState>()
        .with_surface_color()
        .with_color_target("boids.color")
        .double_buffer_storage_array_with_handle::<BoidAgent>(
            "boids.instances",
            DEFAULT_BOID_COUNT as u64,
        );

    let instance_layout = boid_instance_layout();
    let instance_buffer =
        ProceduralBufferBinding::storage(boid_instances.a().clone(), instance_layout);

    let flow = flow
        .compute_pass("boids.simulate")
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::compute_params)
        .bind_ping_pong_storage(boid_instances.name())
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .finish()
        .compute_pass("boids.publish_instances")
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::publish_params)
        .bind_ping_pong_storage(boid_instances.name())
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .depends_on("boids.simulate")
        .finish();

    let flow = flow
        .procedural_pass(
            ProceduralPassDescriptor::local_sdf_2d_impostors(
                "boids.draw",
                instance_buffer,
                DEFAULT_BOID_COUNT,
            )
            .shader_asset("assets/shaders/boids_compose.wgsl")
            .target(
                ProceduralTargetDescriptor::color("boids.color")
                    .clear_color([0.020, 0.028, 0.040, 1.0]),
            )
            .depends_on("boids.publish_instances"),
        )
        .expect("boids.draw procedural pass should be valid");

    flow.present_pass("boids.present")
        .source("boids.color")
        .depends_on("boids.draw")
        .finish()
        .validate()
        .expect("boids_render_flow should validate")
}

fn boid_instance_layout() -> RenderVertexBufferLayout {
    RenderVertexBufferLayout::instance(
        0,
        std::mem::size_of::<<BoidAgent as engine::plugins::render::GpuParams>::Raw>() as u64,
    )
    .attribute(0, 0, RenderVertexFormat::Float32x2)
    .attribute(1, 8, RenderVertexFormat::Float32x2)
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;
    #[allow(deprecated)]
    use engine::plugins::render::{
        RenderBackendCapabilityProfile, RenderFrameDataRegistry, RenderPassId, RenderPassKind,
        compile_flow_plan_checked,
    };

    fn pass_kind(flow: &RenderFlow, pass_id: &str) -> RenderPassKind {
        flow.graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.label == pass_id)
            .map(|pass| pass.kind)
            .expect("requested pass should exist")
    }

    fn pass_id(flow: &RenderFlow, pass_label: &str) -> RenderPassId {
        flow.graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.label == pass_label)
            .map(|pass| pass.id)
            .expect("requested pass should exist")
    }

    #[test]
    fn flow_declares_expected_passes() {
        let flow = build_render_flow();
        let graph = flow.graph();
        let pass_ids = graph
            .passes
            .passes
            .iter()
            .map(|pass| pass.label.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            pass_ids,
            vec![
                "boids.simulate",
                "boids.publish_instances",
                "boids.draw",
                "boids.present",
            ]
        );
        assert_eq!(pass_kind(&flow, "boids.simulate"), RenderPassKind::Compute);
        assert_eq!(
            pass_kind(&flow, "boids.publish_instances"),
            RenderPassKind::Compute
        );
        assert_eq!(pass_kind(&flow, "boids.draw"), RenderPassKind::Graphics);
        assert_eq!(pass_kind(&flow, "boids.present"), RenderPassKind::Present);
    }

    #[test]
    fn flow_does_not_keep_history_copy_without_history_consumer() {
        let flow = build_render_flow();
        assert!(
            flow.graph()
                .passes
                .passes
                .iter()
                .all(|pass| pass.kind != RenderPassKind::Copy)
        );
        assert!(
            flow.graph()
                .passes
                .passes
                .iter()
                .all(|pass| !pass.label.contains("history"))
        );
        assert!(flow.resource_id("boids.history").is_none());
    }

    #[test]
    fn procedural_draw_pass_binds_local_instance_geometry() {
        let flow = build_render_flow();
        let draw_pass = flow
            .graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.label == "boids.draw")
            .expect("draw pass should exist");

        assert_eq!(draw_pass.instance_buffers.len(), 1);
        assert_eq!(draw_pass.instance_buffer_layouts.len(), 1);
        assert_eq!(draw_pass.instance_buffer_layouts[0], boid_instance_layout());
        assert_eq!(
            draw_pass
                .draw
                .expect("draw descriptor should exist")
                .vertex_count,
            6
        );
        assert_eq!(
            draw_pass
                .draw
                .expect("draw descriptor should exist")
                .instance_count,
            DEFAULT_BOID_COUNT
        );

        compile_flow_plan_checked(&flow, &RenderBackendCapabilityProfile::runtime_default())
            .expect("boids procedural draw should satisfy pass-shape guards");
    }

    #[test]
    fn flow_orders_compute_publish_procedural_draw_then_present() {
        let flow = build_render_flow();
        let order = flow
            .pass_order()
            .expect("boids_render_flow pass order should validate")
            .into_iter()
            .map(|id| {
                flow.graph()
                    .passes
                    .passes
                    .iter()
                    .find(|pass| pass.id == id)
                    .map(|pass| pass.label.clone())
                    .expect("ordered pass should exist")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            order,
            vec![
                "boids.simulate",
                "boids.publish_instances",
                "boids.draw",
                "boids.present",
            ]
        );
    }

    #[test]
    fn state_projects_simulation_and_publish_uniforms() {
        let flow = build_render_flow();
        let state = BoidsRenderState::default();
        // Projection-helper compatibility surface; active runtime submission uses PreparedRenderFrame.
        #[allow(deprecated)]
        let frame_data = RenderFrameDataRegistry::new().with(&state);

        let uniforms = flow
            .project_uniforms(&frame_data, (1600, 900))
            .expect("uniform projection should succeed");

        assert!(uniforms.pass(pass_id(&flow, "boids.simulate")).is_some());
        assert!(
            uniforms
                .pass(pass_id(&flow, "boids.publish_instances"))
                .is_some()
        );
        assert!(uniforms.pass(pass_id(&flow, "boids.draw")).is_none());
    }

    #[test]
    fn compose_shader_uses_instance_inputs_without_storage_loop() {
        let shader = include_str!("../../../../assets/shaders/boids_compose.wgsl");
        assert!(shader.contains("@location(0) instance_position"));
        assert!(shader.contains("@location(1) instance_velocity"));
        assert!(!shader.contains("var<storage"));
        assert!(!shader.contains("for (var i"));
    }
}
