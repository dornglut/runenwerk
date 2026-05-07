use crate::rendering::{Sdf3dRenderState, SdfHistoryProbe};
use engine::plugins::render::RenderFlow;

pub(crate) fn build_render_flow() -> RenderFlow {
    RenderFlow::new("sdf_render_flow_3d")
        .with_state::<Sdf3dRenderState>()
        .with_surface_color()
        .with_color_target("sdf.color")
        .with_history_texture("sdf.history")
        .double_buffer_storage_array::<SdfHistoryProbe>("sdf.history.probe", 4)
        .compute_pass("sdf.prepare")
        .uniform_from_state(Sdf3dRenderState::prepare_params)
        .bind_ping_pong_storage("sdf.history.probe")
        .dispatch([1, 1, 1])
        .finish()
        .fullscreen_pass("sdf.compose")
        .shader_asset("assets/shaders/sdf_render_flow_3d_compose.wgsl")
        .uniform_from_state_with_surface(Sdf3dRenderState::compose_params)
        .bind_ping_pong_storage("sdf.history.probe")
        .write_color_target("sdf.color")
        .depends_on("sdf.prepare")
        .finish()
        .copy_pass("sdf.history")
        .source("sdf.color")
        .destination("sdf.history")
        .depends_on("sdf.compose")
        .finish()
        .present_pass("sdf.present")
        .source("sdf.color")
        .depends_on("sdf.history")
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
    fn flow_declares_compute_fullscreen_history_then_present() {
        let flow = build_render_flow();
        let graph = flow.graph();
        let pass_ids = graph
            .passes
            .passes
            .iter()
            .map(|pass| pass.label.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            pass_ids,
            vec!["sdf.prepare", "sdf.compose", "sdf.history", "sdf.present"]
        );
        assert_eq!(pass_kind(&flow, "sdf.prepare"), RenderPassKind::Compute);
        assert_eq!(pass_kind(&flow, "sdf.compose"), RenderPassKind::Fullscreen);
        assert_eq!(pass_kind(&flow, "sdf.history"), RenderPassKind::Copy);
        assert_eq!(pass_kind(&flow, "sdf.present"), RenderPassKind::Present);
    }

    #[test]
    fn flow_orders_prepare_compose_history_before_terminal_present() {
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

        assert_eq!(
            order,
            vec!["sdf.prepare", "sdf.compose", "sdf.history", "sdf.present"]
        );
    }

    #[test]
    fn state_projects_prepare_and_compose_uniforms() {
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
        let prepare_id = flow
            .graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.label == "sdf.prepare")
            .map(|pass| pass.id)
            .expect("prepare pass should exist");
        assert!(uniforms.pass(prepare_id).is_some());
        assert!(uniforms.pass(compose_id).is_some());
    }
}
