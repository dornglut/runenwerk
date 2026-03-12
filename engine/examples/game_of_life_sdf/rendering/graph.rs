// Owner: Game of Life SDF Example - Render Graph Contract
use crate::runtime::GameOfLifeSdfState;
use engine::plugins::render::RenderFlow;

pub(crate) const COMPUTE_EXECUTOR_ID: &str = "gol.compute";
pub(crate) const COMPOSE_EXECUTOR_ID: &str = "gol.compose";
const SHADER_PATH: &str = "assets/shaders/game_of_life_sdf.wgsl";

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new("game_of_life_sdf_example")
        .ecs_resource::<GameOfLifeSdfState>()
        .uniform_buffer::<crate::rendering::GameOfLifeComputeParams>("gol.params")
        .uniform_buffer::<crate::rendering::GameOfLifeComposeParams>("gol.compose.params")
        .storage_texture("gol.cells")
        .import_texture("surface.color")
        .import_texture("ui.draw_list")
        .compute_pass("gol.compute")
        .shader(SHADER_PATH)
        .uniform_state(GameOfLifeSdfState::compute_params)
        .reads("gol.params")
        .writes("gol.cells")
        .finish()
        .fullscreen_pass("gol.compose")
        .shader(SHADER_PATH)
        .uniform_state_with_surface(GameOfLifeSdfState::compose_params)
        .reads("gol.cells")
        .writes("surface.color")
        .depends_on("gol.compute")
        .finish()
        .builtin_ui_composite_pass("ui.composite")
        .reads("ui.draw_list")
        .writes("surface.color")
        .depends_on("gol.compose")
        .finish()
}
