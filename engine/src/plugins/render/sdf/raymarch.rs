use crate::plugins::render::frame_graph::RenderFeatureGraphSpec;
use anyhow::Result;

pub const SDF_FEATURE_ID: &str = "sdf_renderer";
pub const SDF_COMPUTE_PASS_ID: &str = "sdf.compute";
pub const SDF_COMPOSE_PASS_ID: &str = "sdf.compose";

pub fn build_default_sdf_graph_spec() -> Result<RenderFeatureGraphSpec> {
    RenderFeatureGraphSpec::builder(SDF_FEATURE_ID)
        .resource("sdf.params")
        .resource("sdf.color")
        .resource("surface.color")
        .pipeline_compute(
            "sdf.compute.raymarch",
            "assets/shaders/sdf_compute_3d_example.wgsl",
        )
        .pipeline_render_builtin("sdf.compose.fullscreen", "compose.fullscreen")
        .compute_pass(SDF_COMPUTE_PASS_ID)
        .pipeline("sdf.compute.raymarch")
        .executor_builtin_compute()
        .reads(["sdf.params"])
        .writes(["sdf.color"])
        .finish()
        .render_pass(SDF_COMPOSE_PASS_ID)
        .pipeline("sdf.compose.fullscreen")
        .executor_builtin_compose()
        .reads(["sdf.color"])
        .writes(["surface.color"])
        .depends_on([SDF_COMPUTE_PASS_ID])
        .finish()
        .build()
}
