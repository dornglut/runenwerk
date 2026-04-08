use crate::plugins::render::api::SURFACE_COLOR_RESOURCE_ID;
use crate::plugins::render::graph::{RenderPassKind, RenderPassNode};

pub fn validate_builtin_ui_pass_shape(pass: &RenderPassNode, issues: &mut Vec<String>) {
    if !matches!(pass.kind, RenderPassKind::BuiltinUiComposite) {
        return;
    }

    if pass.feature_id.is_some() {
        issues.push(format!(
			"builtin_ui_composite pass '{}' cannot declare explicit feature id; feature is fixed to 'ui'",
			pass.id.as_str()
		));
    }
    if pass.shader.is_some() {
        issues.push(format!(
            "builtin_ui_composite pass '{}' cannot declare shader",
            pass.id.as_str()
        ));
    }
    if pass.workgroup_size.is_some() {
        issues.push(format!(
            "builtin_ui_composite pass '{}' cannot declare workgroup_size",
            pass.id.as_str()
        ));
    }
    if pass.clear_color.is_some() {
        issues.push(format!(
            "builtin_ui_composite pass '{}' cannot declare clear_color",
            pass.id.as_str()
        ));
    }
    if pass.compute_dispatch.is_some() {
        issues.push(format!(
            "builtin_ui_composite pass '{}' cannot declare compute dispatch",
            pass.id.as_str()
        ));
    }
    if pass.depth_target.is_some() {
        issues.push(format!(
            "builtin_ui_composite pass '{}' cannot declare depth target",
            pass.id.as_str()
        ));
    }
    if !pass.uniform_bindings.is_empty() {
        issues.push(format!(
            "builtin_ui_composite pass '{}' cannot declare uniform bindings",
            pass.id.as_str()
        ));
    }
    if !pass.sampled_textures.is_empty()
        || !pass.write_textures.is_empty()
        || !pass.vertex_buffers.is_empty()
        || !pass.index_buffers.is_empty()
        || !pass.instance_buffers.is_empty()
        || !pass.indirect_buffers.is_empty()
    {
        issues.push(format!(
            "builtin_ui_composite pass '{}' only supports writes/depends_on",
            pass.id.as_str()
        ));
    }
    if !pass.reads.is_empty() {
        issues.push(format!(
			"builtin_ui_composite pass '{}' must not declare reads(...); UI input comes from PreparedRenderFrame::ui()",
			pass.id.as_str()
		));
    }
    if pass.writes.len() != 1 || pass.writes[0].as_str() != SURFACE_COLOR_RESOURCE_ID {
        issues.push(format!(
            "builtin_ui_composite pass '{}' must write exactly '{}' as color output",
            pass.id.as_str(),
            SURFACE_COLOR_RESOURCE_ID
        ));
    }
}
