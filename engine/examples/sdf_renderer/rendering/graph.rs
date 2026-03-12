use crate::runtime::SdfWorldState;
use engine::plugins::render::RenderFlow;

pub(crate) const COMPUTE_EXECUTOR_ID: &str = "sdf.compute";
pub(crate) const COMPOSE_EXECUTOR_ID: &str = "sdf.compose";
const COMPUTE_SHADER_PATH: &str = "assets/shaders/sdf_compute_3d_example.wgsl";
const COMPOSE_SHADER_PATH: &str = "assets/shaders/world_compose_fullscreen.wgsl";

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new("sdf_renderer_example")
        .ecs_resource::<SdfWorldState>()
        .uniform_buffer::<crate::rendering::SdfWorldParams>("sdf.params")
        .uniform_buffer::<crate::rendering::SdfComposeParams>("sdf.compose.params")
        .storage_buffer::<crate::rendering::SdfWorldAgent>("world.agents")
        .storage_buffer::<crate::rendering::SdfWorldModel>("world.models")
        .storage_texture("sdf.color")
        .import_texture("surface.color")
        .import_texture("ui.draw_list")
        .compute_pass(COMPUTE_EXECUTOR_ID)
        .shader(COMPUTE_SHADER_PATH)
        .uniform_state(SdfWorldState::compute_params)
        .reads("sdf.params")
        .reads("world.agents")
        .reads("world.models")
        .writes("sdf.color")
        .workgroup_size(8, 8, 1)
        .finish()
        .fullscreen_pass(COMPOSE_EXECUTOR_ID)
        .shader(COMPOSE_SHADER_PATH)
        .uniform_state_with_surface(SdfWorldState::compose_params)
        .sample_texture("sdf.color")
        .writes("surface.color")
        .depends_on(COMPUTE_EXECUTOR_ID)
        .finish()
        .builtin_ui_composite_pass("ui.composite")
        .reads("ui.draw_list")
        .writes("surface.color")
        .depends_on(COMPOSE_EXECUTOR_ID)
        .finish()
}
