use crate::plugins::render::graph::{
    CompiledPassExecutionPlan, CompiledRasterExecutionPlan, CompiledRenderFlowPlan,
    RenderExecutionGraphDiagnostic, RenderExecutionGraphDiagnosticKind, RenderPassShapeIntent,
};

pub fn diagnose_compiled_pass_shapes(
    flow: &CompiledRenderFlowPlan,
) -> Vec<RenderExecutionGraphDiagnostic> {
    flow.execution
        .passes
        .iter()
        .flat_map(|pass| match pass {
            CompiledPassExecutionPlan::Graphics(raster) => {
                diagnose_graphics_pass_shape(flow, raster)
            }
            CompiledPassExecutionPlan::Compute(_)
            | CompiledPassExecutionPlan::Fullscreen(_)
            | CompiledPassExecutionPlan::Copy(_)
            | CompiledPassExecutionPlan::Present(_)
            | CompiledPassExecutionPlan::BuiltinUiComposite(_) => Vec::new(),
        })
        .collect()
}

fn diagnose_graphics_pass_shape(
    flow: &CompiledRenderFlowPlan,
    raster: &CompiledRasterExecutionPlan,
) -> Vec<RenderExecutionGraphDiagnostic> {
    let mut diagnostics = Vec::new();
    let Some(draw) = raster.draw else {
        return diagnostics;
    };
    let Some(node) = flow
        .pass_order
        .iter()
        .find(|pass| pass.pass_id() == raster.pass_id)
        .map(|pass| pass.node())
    else {
        return diagnostics;
    };

    if let RenderPassShapeIntent::AdvancedInstancedFullscreen {
        max_instances,
        reason,
    } = &node.shape_intent
    {
        if *max_instances == 0 || reason.trim().is_empty() {
            diagnostics.push(
                pass_shape_diagnostic(
                    flow,
                    raster,
                    RenderExecutionGraphDiagnosticKind::InvalidPassShapeIntent,
                    "advanced instanced fullscreen intent must declare a non-zero max_instances and a non-empty reason",
                )
                .with_capability("pass_shape::advanced_instanced_fullscreen_intent"),
            );
        }
        if draw.instance_count > *max_instances {
            diagnostics.push(
                pass_shape_diagnostic(
                    flow,
                    raster,
                    RenderExecutionGraphDiagnosticKind::FullscreenInstancedWork,
                    format!(
                        "graphics pass '{}' draws {} fullscreen-style instances but its advanced intent allows at most {}",
                        node.label, draw.instance_count, max_instances
                    ),
                )
                .with_capability("pass_shape::advanced_instanced_fullscreen_limit"),
            );
        }
    }

    if draw.instance_count <= 1 {
        return diagnostics;
    }

    let has_local_geometry = !raster.draw_buffers.vertex_buffers.is_empty()
        || !raster.draw_buffers.index_buffers.is_empty()
        || !raster.draw_buffers.instance_buffers.is_empty();
    let generated_fullscreen_geometry = draw.vertex_count <= 6 && !has_local_geometry;
    let storage_backed = !raster.bindings.storage_order.is_empty();
    let explicit_advanced_intent = matches!(
        node.shape_intent,
        RenderPassShapeIntent::AdvancedInstancedFullscreen { .. }
    );

    if generated_fullscreen_geometry && !explicit_advanced_intent {
        diagnostics.push(
            pass_shape_diagnostic(
                flow,
                raster,
                RenderExecutionGraphDiagnosticKind::FullscreenInstancedWork,
                format!(
                    "graphics pass '{}' draws fullscreen-style generated geometry {} times; add explicit advanced intent or use local instance geometry",
                    node.label, draw.instance_count
                ),
            )
            .with_capability("pass_shape::fullscreen_instanced_work"),
        );
    } else if storage_backed && !has_local_geometry && !explicit_advanced_intent {
        diagnostics.push(
            pass_shape_diagnostic(
                flow,
                raster,
                RenderExecutionGraphDiagnosticKind::AmbiguousProceduralShape,
                format!(
                    "graphics pass '{}' uses storage-backed procedural data with {} instances but declares no local vertex, index, or instance geometry",
                    node.label, draw.instance_count
                ),
            )
            .with_capability("pass_shape::ambiguous_procedural_shape"),
        );
    }

    diagnostics
}

fn pass_shape_diagnostic(
    flow: &CompiledRenderFlowPlan,
    raster: &CompiledRasterExecutionPlan,
    kind: RenderExecutionGraphDiagnosticKind,
    message: impl Into<String>,
) -> RenderExecutionGraphDiagnostic {
    let label = flow
        .pass_order
        .iter()
        .find(|pass| pass.pass_id() == raster.pass_id)
        .map(|pass| pass.pass_label().to_string())
        .unwrap_or_else(|| raster.pass_id.to_string());
    RenderExecutionGraphDiagnostic::error(kind, message)
        .with_flow(flow.flow_id, flow.flow_label.clone())
        .with_pass(raster.pass_id, label)
}
