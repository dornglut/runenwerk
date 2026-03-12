// Owner: Game of Life SDF Example - Render Graph Contract
use engine::plugins::render::RenderFlow;

pub(crate) const COMPUTE_PASS_ID: &str = "gol.compute";
pub(crate) const COMPOSE_PASS_ID: &str = "gol.compose";

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new("game_of_life_sdf_example")
        .import_texture("surface.color")
        .import_texture("ui.draw_list")
        .compute_pass(COMPUTE_PASS_ID)
        .workgroup_size(8, 8, 1)
        .finish()
        .fullscreen_pass(COMPOSE_PASS_ID)
        .writes("surface.color")
        .clear_color([0.03, 0.045, 0.042, 1.0])
        .depends_on(COMPUTE_PASS_ID)
        .finish()
        .builtin_ui_composite_pass("ui.composite")
        .reads("ui.draw_list")
        .writes("surface.color")
        .depends_on(COMPOSE_PASS_ID)
        .finish()
}
