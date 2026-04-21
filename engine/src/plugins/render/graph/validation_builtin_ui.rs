use crate::plugins::render::graph::RenderPassNode;
use crate::plugins::render::graph::validation::RenderFlowValidationIssue;

pub fn validate_builtin_ui_pass_shape(
    pass: &RenderPassNode,
    issues: &mut Vec<RenderFlowValidationIssue>,
) {
    if pass.feature_id.is_some() {
        issues.push(RenderFlowValidationIssue::BuiltinUiExplicitFeatureId {
            pass_label: pass.label.clone(),
        });
    }

    if pass.shader.is_some() {
        issues.push(RenderFlowValidationIssue::BuiltinUiHasShader {
            pass_label: pass.label.clone(),
        });
    }

    if pass.workgroup_size.is_some() {
        issues.push(RenderFlowValidationIssue::BuiltinUiHasWorkgroupSize {
            pass_label: pass.label.clone(),
        });
    }

    if pass.clear_color.is_some() {
        issues.push(RenderFlowValidationIssue::BuiltinUiHasClearColor {
            pass_label: pass.label.clone(),
        });
    }

    if pass.compute_dispatch.is_some() {
        issues.push(RenderFlowValidationIssue::BuiltinUiHasComputeDispatch {
            pass_label: pass.label.clone(),
        });
    }

    if pass.depth_target.is_some() {
        issues.push(RenderFlowValidationIssue::BuiltinUiHasDepthTarget {
            pass_label: pass.label.clone(),
        });
    }

    if !pass.uniform_bindings.is_empty() {
        issues.push(RenderFlowValidationIssue::BuiltinUiHasUniformBindings {
            pass_label: pass.label.clone(),
        });
    }

    if !pass.sampled_textures.is_empty()
      || !pass.write_textures.is_empty()
      || !pass.vertex_buffers.is_empty()
      || !pass.index_buffers.is_empty()
      || !pass.instance_buffers.is_empty()
      || !pass.indirect_buffers.is_empty()
    {
        issues.push(RenderFlowValidationIssue::BuiltinUiInvalidResourceBindings {
            pass_label: pass.label.clone(),
        });
    }

    if !pass.reads.is_empty() {
        issues.push(RenderFlowValidationIssue::BuiltinUiHasReads {
            pass_label: pass.label.clone(),
        });
    }

    if pass.writes.len() != 1 {
        issues.push(RenderFlowValidationIssue::BuiltinUiInvalidWriteArity {
            pass_label: pass.label.clone(),
            write_count: pass.writes.len(),
        });
    }
}