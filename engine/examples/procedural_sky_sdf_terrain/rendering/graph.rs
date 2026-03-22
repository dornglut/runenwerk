use crate::rendering::ProceduralSkyTerrainState;
use engine::plugins::render::RenderFlow;

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new("procedural_sky_sdf_terrain")
        .with_state::<ProceduralSkyTerrainState>()
        .with_surface_color()
        .fullscreen_pass("terrain.compose")
        .shader_asset("assets/shaders/procedural_sky_sdf_terrain_compose.wgsl")
        .uniform_from_state_with_surface(ProceduralSkyTerrainState::compose_params)
        .write_surface_color()
        .finish()
        .validate()
        .expect("procedural_sky_sdf_terrain should validate")
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;
    #[allow(deprecated)]
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
    fn flow_declares_single_fullscreen_pass() {
        let flow = build_render_flow();
        let graph = flow.graph();
        let pass_ids = graph
            .passes
            .passes
            .iter()
            .map(|pass| pass.id.as_str().to_string())
            .collect::<Vec<_>>();
        assert_eq!(pass_ids, vec!["terrain.compose"]);
        assert_eq!(
            pass_kind(&flow, "terrain.compose"),
            RenderPassKind::Fullscreen
        );
    }

    #[test]
    fn state_projects_compose_uniforms() {
        let flow = build_render_flow();
        let state = ProceduralSkyTerrainState::default();
        #[allow(deprecated)]
        let frame_data = RenderFrameDataRegistry::new().with(&state);

        let uniforms = flow
            .project_uniforms(&frame_data, (1600, 900))
            .expect("uniform projection should succeed");

        assert!(uniforms.pass("terrain.compose").is_some());
    }
}
