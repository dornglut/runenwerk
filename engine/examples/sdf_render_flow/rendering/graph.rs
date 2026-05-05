use crate::rendering::Sdf3dRenderState;
use engine::plugins::render::RenderFlow;

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new("sdf_render_flow_3d")
        .with_state::<Sdf3dRenderState>()
        .with_surface_color()
        .with_color_target("sdf.color")
        .fullscreen_pass("sdf.compose")
        .shader_asset("assets/shaders/sdf_render_flow_3d_compose.wgsl")
        .uniform_from_state_with_surface(Sdf3dRenderState::compose_params)
        .write_color_target("sdf.color")
        .finish()
        .present_pass("sdf.present")
        .source("sdf.color")
        .depends_on("sdf.compose")
        .finish()
        .validate()
        .expect("sdf_render_flow_3d should validate")
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
            .find(|pass| pass.label == pass_id)
            .map(|pass| pass.kind)
            .expect("requested pass should exist")
    }

    #[test]
    fn flow_declares_fullscreen_compose_then_present() {
        let flow = build_render_flow();
        let graph = flow.graph();
        let pass_ids = graph
            .passes
            .passes
            .iter()
            .map(|pass| pass.label.clone())
            .collect::<Vec<_>>();
        assert_eq!(pass_ids, vec!["sdf.compose", "sdf.present"]);
        assert_eq!(pass_kind(&flow, "sdf.compose"), RenderPassKind::Fullscreen);
        assert_eq!(pass_kind(&flow, "sdf.present"), RenderPassKind::Present);
    }

    #[test]
    fn flow_orders_compose_before_terminal_present() {
        let flow = build_render_flow();
        let order = flow
            .pass_order()
            .expect("sdf_render_flow pass order should validate")
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

        assert_eq!(order, vec!["sdf.compose", "sdf.present"]);
    }

    #[test]
    fn state_projects_compose_uniforms() {
        let flow = build_render_flow();
        let state = Sdf3dRenderState::default();
        #[allow(deprecated)]
        let frame_data = RenderFrameDataRegistry::new().with(&state);

        let uniforms = flow
            .project_uniforms(&frame_data, (1600, 900))
            .expect("uniform projection should succeed");

        let compose_id = flow
            .graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.label == "sdf.compose")
            .map(|pass| pass.id)
            .expect("compose pass should exist");
        assert!(uniforms.pass(compose_id).is_some());
    }
}
