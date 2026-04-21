use crate::rendering::{BoidAgent, BoidsRenderState, DEFAULT_BOID_COUNT};
use engine::plugins::render::RenderFlow;

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new("boids_render_flow")
        .with_state::<BoidsRenderState>()
        .with_surface_color()
        .double_buffer_storage_array::<BoidAgent>("boids.instances", DEFAULT_BOID_COUNT as u64)
        .compute_pass("boids.simulate")
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::compute_params)
        .bind_ping_pong_storage("boids.instances")
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .finish()
        .fullscreen_pass("boids.compose")
        .shader_asset("assets/shaders/boids_compose.wgsl")
        .uniform_from_state_with_surface(BoidsRenderState::compose_params)
        .bind_ping_pong_storage("boids.instances")
        .write_surface_color()
        .depends_on("boids.simulate")
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
        assert_eq!(pass_ids, vec!["boids.simulate", "boids.compose"]);
        assert_eq!(pass_kind(&flow, "boids.simulate"), RenderPassKind::Compute);
        assert_eq!(
            pass_kind(&flow, "boids.compose"),
            RenderPassKind::Fullscreen
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
        assert!(uniforms.pass(pass_id(&flow, "boids.compose")).is_some());
    }
}
