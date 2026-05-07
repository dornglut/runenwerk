use crate::rendering::{BoidAgent, BoidsRenderState, DEFAULT_BOID_COUNT};
use engine::plugins::render::{RenderFlow, RenderVertexBufferLayout, RenderVertexFormat};

pub(crate) fn build_render_flow() -> RenderFlow {
    let (flow, boid_instances) = RenderFlow::new("boids_render_flow")
        .with_state::<BoidsRenderState>()
        .with_surface_color()
        .with_color_target("boids.color")
        .with_history_texture("boids.history")
        .double_buffer_storage_array_with_handle::<BoidAgent>(
            "boids.instances",
            DEFAULT_BOID_COUNT as u64,
        );

    flow.compute_pass("boids.simulate")
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::compute_params)
        .bind_ping_pong_storage(boid_instances.name())
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .finish()
        .graphics_pass("boids.draw")
        .shader_asset("assets/shaders/boids_compose.wgsl")
        .uniform_from_state_with_surface(BoidsRenderState::compose_params)
        .bind_ping_pong_storage(boid_instances.name())
        .instance_buffer(
            boid_instances.a().clone(),
            RenderVertexBufferLayout::instance(
                0,
                std::mem::size_of::<<BoidAgent as engine::plugins::render::GpuParams>::Raw>()
                    as u64,
            )
            .attribute(0, 0, RenderVertexFormat::Float32x2)
            .attribute(1, 8, RenderVertexFormat::Float32x2),
        )
        .write_color_target("boids.color")
        .draw(3, DEFAULT_BOID_COUNT)
        .depends_on("boids.simulate")
        .finish()
        .copy_pass("boids.history")
        .source("boids.color")
        .destination("boids.history")
        .depends_on("boids.draw")
        .finish()
        .present_pass("boids.present")
        .source("boids.color")
        .depends_on("boids.history")
        .finish()
        .validate()
        .expect("boids_render_flow should validate")
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;
    #[allow(deprecated)]
    use engine::plugins::render::{RenderFrameDataRegistry, RenderPassId, RenderPassKind};

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
                "boids.draw",
                "boids.history",
                "boids.present"
            ]
        );
        assert_eq!(pass_kind(&flow, "boids.simulate"), RenderPassKind::Compute);
        assert_eq!(pass_kind(&flow, "boids.draw"), RenderPassKind::Graphics);
        assert_eq!(pass_kind(&flow, "boids.history"), RenderPassKind::Copy);
        assert_eq!(pass_kind(&flow, "boids.present"), RenderPassKind::Present);
    }

    #[test]
    fn graphics_pass_binds_instance_draw_buffer() {
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
        assert_eq!(
            draw_pass
                .draw
                .expect("draw descriptor should exist")
                .instance_count,
            DEFAULT_BOID_COUNT
        );
    }

    #[test]
    fn flow_orders_compute_graphics_history_copy_then_present() {
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
                "boids.draw",
                "boids.history",
                "boids.present"
            ]
        );
    }

    #[test]
    fn state_projects_simulation_and_compose_uniforms() {
        let flow = build_render_flow();
        let state = BoidsRenderState::default();
        // Projection-helper compatibility surface; active runtime submission uses PreparedRenderFrame.
        #[allow(deprecated)]
        let frame_data = RenderFrameDataRegistry::new().with(&state);

        let uniforms = flow
            .project_uniforms(&frame_data, (1600, 900))
            .expect("uniform projection should succeed");

        assert!(uniforms.pass(pass_id(&flow, "boids.simulate")).is_some());
        assert!(uniforms.pass(pass_id(&flow, "boids.draw")).is_some());
    }
}
