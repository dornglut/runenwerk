// Owner: Game of Life SDF Example - Render Graph Contract
use anyhow::Result;
use engine::plugins::render::domain::RenderFeatureGraphSpec;

pub(crate) const FEATURE_ID: &str = "game_of_life_sdf_example";
pub(crate) const COMPUTE_EXECUTOR_ID: &str = "gol.compute";
pub(crate) const COMPOSE_EXECUTOR_ID: &str = "gol.compose";
const COMPUTE_PIPELINE_ID: &str = "gol.compute.simulation";
const COMPOSE_PIPELINE_ID: &str = "gol.compose.sdf";
const SHADER_PATH: &str = "assets/shaders/game_of_life_sdf.wgsl";

pub(crate) fn build_feature_graph_spec() -> Result<RenderFeatureGraphSpec> {
    let mut builder = RenderFeatureGraphSpec::builder(FEATURE_ID)
        .resource("gol.params")
        .resource("gol.cells")
        .resource("surface.color")
        .resource("ui.draw_list")
        .pipeline_compute(COMPUTE_PIPELINE_ID, SHADER_PATH)
        .pipeline_render_builtin(COMPOSE_PIPELINE_ID, "compose.fullscreen")
        .pipeline_render_builtin("ui.compose", "ui.composite");

    builder = builder
        .compute_pass("gol.compute")
        .pipeline(COMPUTE_PIPELINE_ID)
        .executor(COMPUTE_EXECUTOR_ID)
        .reads(["gol.params"])
        .writes(["gol.cells"])
        .finish();
    builder = builder
        .render_pass("gol.compose")
        .pipeline(COMPOSE_PIPELINE_ID)
        .executor(COMPOSE_EXECUTOR_ID)
        .reads(["gol.cells"])
        .writes(["surface.color"])
        .depends_on(["gol.compute"])
        .finish();
    builder = builder
        .render_pass("ui_composite")
        .pipeline("ui.compose")
        .executor_builtin_ui_composite()
        .reads(["ui.draw_list"])
        .writes(["surface.color"])
        .depends_on(["gol.compose"])
        .finish();

    builder.build()
}
