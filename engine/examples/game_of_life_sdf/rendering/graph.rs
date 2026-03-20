use crate::rendering::{
    DEFAULT_GRID_CELL_COUNT, GameOfLifeCell, GameOfLifeRenderState,
};
use engine::plugins::render::RenderFlow;

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new("game_of_life_sdf")
        .with_state::<GameOfLifeRenderState>()
        .with_surface_color()
        .with_builtin_ui()
        .double_buffer_storage_array::<GameOfLifeCell>("cells", DEFAULT_GRID_CELL_COUNT)
        .compute_pass("simulate")
        .shader_asset("assets/shaders/game_of_life_compute.wgsl")
        .uniform_from_state(GameOfLifeRenderState::compute_params)
        .bind_ping_pong_storage("cells")
        .dispatch_from_state(GameOfLifeRenderState::dispatch_workgroups)
        .finish()
        .fullscreen_pass("compose")
        .shader_asset("assets/shaders/game_of_life_compose.wgsl")
        .uniform_from_state_with_surface(GameOfLifeRenderState::compose_params)
        .bind_ping_pong_storage("cells")
        .write_surface_color()
        .depends_on("simulate")
        .finish()
        .builtin_ui_composite_pass("ui")
        .depends_on("compose")
        .finish()
        .validate()
        .expect("game_of_life_sdf flow should validate")
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::plugins::render::{RenderFrameDataRegistry, RenderPassKind};

    fn pass_kind(flow: &RenderFlow, pass_id: &str) -> RenderPassKind {
        flow.graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.id.as_str() == pass_id)
            .map(|pass| pass.kind)
            .expect("requested pass should exist")
    }

    #[test]
    fn flow_declares_expected_contract() {
        let flow = build_render_flow();
        let graph = flow.graph();
        let pass_ids = graph
            .passes
            .passes
            .iter()
            .map(|pass| pass.id.as_str().to_string())
            .collect::<Vec<_>>();
        assert_eq!(pass_ids, vec!["simulate", "compose", "ui"]);

        assert_eq!(pass_kind(&flow, "simulate"), RenderPassKind::Compute);
        assert_eq!(pass_kind(&flow, "compose"), RenderPassKind::Fullscreen);
    }

    #[test]
    fn state_projects_compute_and_compose_uniforms() {
        let flow = build_render_flow();
        let state = GameOfLifeRenderState::default();
        let frame_data = RenderFrameDataRegistry::new().with(&state);

        let uniforms = flow
            .project_uniforms(&frame_data, (1280, 720))
            .expect("uniform projection should succeed");

        assert!(uniforms.pass("simulate").is_some());
        assert!(uniforms.pass("compose").is_some());
    }
}
