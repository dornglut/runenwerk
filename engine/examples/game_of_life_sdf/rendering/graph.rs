// Owner: Game of Life SDF Example - Render Graph Contract
use crate::rendering::{
    GameOfLifeCell, GameOfLifeComposeParams, GameOfLifeComputeParams, GameOfLifeRenderState,
};
use engine::plugins::render::RenderFlow;

pub(crate) const FLOW_ID: &str = "game_of_life_sdf";
pub(crate) const COMPUTE_PASS_ID: &str = "gol.simulate";
pub(crate) const COMPOSE_PASS_ID: &str = "gol.compose";
pub(crate) const SHADER_ID: &str = "game_of_life_sdf";
pub(crate) const CELL_STORAGE_RESOURCE_ID: &str = "gol.cells";
pub(crate) const COMPUTE_PARAMS_RESOURCE_ID: &str = "gol.compute.params";
pub(crate) const COMPOSE_PARAMS_RESOURCE_ID: &str = "gol.compose.params";
pub(crate) const SURFACE_COLOR_RESOURCE_ID: &str = "surface.color";
pub(crate) const UI_DRAW_LIST_RESOURCE_ID: &str = "ui.draw_list";

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new(FLOW_ID)
        .ecs_resource::<GameOfLifeRenderState>()
        .uniform_buffer::<GameOfLifeComputeParams>(COMPUTE_PARAMS_RESOURCE_ID)
        .uniform_buffer::<GameOfLifeComposeParams>(COMPOSE_PARAMS_RESOURCE_ID)
        .storage_buffer::<GameOfLifeCell>(CELL_STORAGE_RESOURCE_ID)
        .import_texture(SURFACE_COLOR_RESOURCE_ID)
        .import_texture(UI_DRAW_LIST_RESOURCE_ID)
        .compute_pass(COMPUTE_PASS_ID)
        .shader(SHADER_ID)
        .uniform_state(GameOfLifeRenderState::compute_params)
        .writes(CELL_STORAGE_RESOURCE_ID)
        .workgroup_size(8, 8, 1)
        .finish()
        .fullscreen_pass(COMPOSE_PASS_ID)
        .shader(SHADER_ID)
        .uniform_state_with_surface(GameOfLifeRenderState::compose_params)
        .reads(CELL_STORAGE_RESOURCE_ID)
        .writes(SURFACE_COLOR_RESOURCE_ID)
        .depends_on(COMPUTE_PASS_ID)
        .finish()
        .builtin_ui_composite_pass("ui.composite")
        .reads(UI_DRAW_LIST_RESOURCE_ID)
        .writes(SURFACE_COLOR_RESOURCE_ID)
        .depends_on(COMPOSE_PASS_ID)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::plugins::render::{
        RenderFlow, RenderFrameDataRegistry, RenderPassKind, RenderResourceDescriptor,
    };

    #[test]
    fn flow_declares_required_resources_and_pass_contract() {
        let flow = build_render_flow();
        let report = flow.validate().expect("flow should validate");
        assert_eq!(
            report.pass_order,
            vec![COMPUTE_PASS_ID, COMPOSE_PASS_ID, "ui.composite"]
        );

        assert!(
            flow.graph().resources.ecs_resources.iter().any(
                |resource| resource.type_name == std::any::type_name::<GameOfLifeRenderState>()
            )
        );
        assert!(flow.graph().resources.resources.iter().any(|resource| {
            matches!(resource, RenderResourceDescriptor::UniformBuffer(value) if value.id.as_str() == COMPUTE_PARAMS_RESOURCE_ID)
        }));
        assert!(flow.graph().resources.resources.iter().any(|resource| {
            matches!(resource, RenderResourceDescriptor::UniformBuffer(value) if value.id.as_str() == COMPOSE_PARAMS_RESOURCE_ID)
        }));
        assert!(flow.graph().resources.resources.iter().any(|resource| {
            matches!(resource, RenderResourceDescriptor::StorageBuffer(value) if value.id.as_str() == CELL_STORAGE_RESOURCE_ID)
        }));
        assert!(flow.graph().resources.resources.iter().any(|resource| {
            matches!(resource, RenderResourceDescriptor::ImportedTexture(value) if value.id.as_str() == SURFACE_COLOR_RESOURCE_ID)
        }));
        assert!(flow.graph().resources.resources.iter().any(|resource| {
            matches!(resource, RenderResourceDescriptor::ImportedTexture(value) if value.id.as_str() == UI_DRAW_LIST_RESOURCE_ID)
        }));

        assert_compute_pass_contract(&flow);
        assert_compose_pass_contract(&flow);
    }

    #[test]
    fn uniform_projection_uses_state_param_methods() {
        let flow = build_render_flow();
        flow.validate().expect("flow should validate");

        let state = GameOfLifeRenderState::default();
        let frame_data = RenderFrameDataRegistry::new().with(&state);
        let projections = flow
            .project_uniform_buffers(&frame_data, (1280, 720))
            .expect("uniform projection should succeed");

        assert!(projections.iter().any(|pass| {
            pass.pass_id == COMPUTE_PASS_ID
                && pass.buffers.iter().any(|buffer| {
                    buffer.buffer_id.as_str() == COMPUTE_PARAMS_RESOURCE_ID
                        && !buffer.bytes.is_empty()
                })
        }));
        assert!(projections.iter().any(|pass| {
            pass.pass_id == COMPOSE_PASS_ID
                && pass.buffers.iter().any(|buffer| {
                    buffer.buffer_id.as_str() == COMPOSE_PARAMS_RESOURCE_ID
                        && !buffer.bytes.is_empty()
                })
        }));
    }

    fn assert_compute_pass_contract(flow: &RenderFlow) {
        let compute = flow
            .graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.id.as_str() == COMPUTE_PASS_ID)
            .expect("compute pass should exist");
        assert_eq!(compute.kind, RenderPassKind::Compute);
        assert_eq!(compute.shader.as_deref(), Some(SHADER_ID));
        assert!(compute.reads.is_empty());
        assert_eq!(compute.writes.len(), 1);
        assert_eq!(compute.writes[0].as_str(), CELL_STORAGE_RESOURCE_ID);
        assert_eq!(compute.uniform_bindings.len(), 1);
    }

    fn assert_compose_pass_contract(flow: &RenderFlow) {
        let compose = flow
            .graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.id.as_str() == COMPOSE_PASS_ID)
            .expect("compose pass should exist");
        assert_eq!(compose.kind, RenderPassKind::Fullscreen);
        assert_eq!(compose.shader.as_deref(), Some(SHADER_ID));
        assert_eq!(compose.reads.len(), 1);
        assert_eq!(compose.reads[0].as_str(), CELL_STORAGE_RESOURCE_ID);
        assert_eq!(compose.writes.len(), 1);
        assert_eq!(compose.writes[0].as_str(), SURFACE_COLOR_RESOURCE_ID);
        assert_eq!(compose.uniform_bindings.len(), 1);
    }
}
