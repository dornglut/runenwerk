use crate::plugins::gpu::{GpuCapabilities, GpuWorkResourceId};
use crate::plugins::render::RenderFlowValidationError;
use crate::plugins::render::api::{ComputeDispatchDescriptor, RenderFlow};
use crate::plugins::render::graph::{
    CompiledFlowExecutionPlan, RenderExecutionGraphCompileError, RenderExecutionGraphDiagnostic,
    RenderExecutionGraphDiagnosticKind, RenderPassKind, RenderPassNode, ResourceGraph,
    compile_execution_plan, diagnose_compiled_pass_shapes,
};
use crate::plugins::render::{
    PreparedRenderWorkPlan, RenderFlowId, RenderGpuWorkInstrumentation, RenderPassId,
    prepare_render_gpu_work, validate_compiled_flow_capabilities,
    validate_prepared_gpu_work_capabilities,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct CompiledRenderFlowPlan {
    pub flow_id: RenderFlowId,
    pub flow_label: String,
    pub resource_ids_by_label: BTreeMap<String, GpuWorkResourceId>,
    pub resources: ResourceGraph,
    /// Lexical render-owned pass descriptors and later-phase payload inputs.
    /// Runtime scheduling comes only from prepared G3 work.
    pub render_passes: Vec<CompiledPassDescriptor>,
    pub execution: CompiledFlowExecutionPlan,
    pub compiler_diagnostics: Vec<RenderExecutionGraphDiagnostic>,
    structural_work: Option<PreparedRenderWorkPlan>,
}

impl CompiledRenderFlowPlan {
    pub fn resource_label(&self, resource_id: GpuWorkResourceId) -> Option<String> {
        self.resource_ids_by_label
            .iter()
            .find_map(|(label, id)| (*id == resource_id).then(|| label.clone()))
    }

    pub fn resource_descriptor(
        &self,
        resource_id: GpuWorkResourceId,
    ) -> Option<&crate::plugins::render::RenderResourceDeclaration> {
        self.resources
            .resources
            .iter()
            .find(|descriptor| *descriptor.id() == resource_id)
    }

    /// Structurally prepared G3 authority for compile-time inspection and
    /// admission. Runtime uses the exact per-invocation prepared work stored in
    /// `PreparedFlowInputs` after ECS/domain projection.
    pub fn structural_work(&self) -> Option<&PreparedRenderWorkPlan> {
        self.structural_work.as_ref()
    }
}

#[derive(Debug, Clone)]
pub enum CompiledPassDescriptor {
    Compute(CompiledComputePass),
    Fullscreen(CompiledFullscreenPass),
    Graphics(CompiledGraphicsPass),
    Copy(CompiledCopyPass),
    Present(CompiledPresentPass),
    BuiltinUiComposite(CompiledUiCompositePass),
}

impl CompiledPassDescriptor {
    pub fn pass_id(&self) -> RenderPassId {
        self.node().id
    }

    pub fn pass_label(&self) -> &str {
        self.node().label.as_str()
    }

    pub fn node(&self) -> &RenderPassNode {
        match self {
            Self::Compute(value) => &value.node,
            Self::Fullscreen(value) => &value.node,
            Self::Graphics(value) => &value.node,
            Self::Copy(value) => &value.node,
            Self::Present(value) => &value.node,
            Self::BuiltinUiComposite(value) => &value.node,
        }
    }

    /// Lexical source position retained for provenance only. Prepared G3 work
    /// owns dependency and execution order.
    pub fn authoring_index(&self) -> usize {
        match self {
            Self::Compute(value) => value.authoring_index,
            Self::Fullscreen(value) => value.authoring_index,
            Self::Graphics(value) => value.authoring_index,
            Self::Copy(value) => value.authoring_index,
            Self::Present(value) => value.authoring_index,
            Self::BuiltinUiComposite(value) => value.authoring_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledComputePass {
    /// Lexical source position retained for provenance, not scheduling.
    pub authoring_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledFullscreenPass {
    /// Lexical source position retained for provenance, not scheduling.
    pub authoring_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledGraphicsPass {
    /// Lexical source position retained for provenance, not scheduling.
    pub authoring_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledCopyPass {
    /// Lexical source position retained for provenance, not scheduling.
    pub authoring_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledPresentPass {
    /// Lexical source position retained for provenance, not scheduling.
    pub authoring_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledUiCompositePass {
    /// Lexical source position retained for provenance, not scheduling.
    pub authoring_index: usize,
    pub node: RenderPassNode,
}

pub fn compile_flow_plan(
    flow: &RenderFlow,
) -> Result<CompiledRenderFlowPlan, RenderFlowValidationError> {
    let report = flow.validation_report()?;
    let mut plan = build_compiled_flow_plan(flow, report.lexical_pass_ids, Vec::new());
    let work = prepare_structural_gpu_work(&plan).map_err(|error| {
        RenderFlowValidationError::from(vec![
            crate::plugins::render::RenderFlowValidationIssue::GpuWorkLoweringFailed {
                message: error.to_string(),
            },
        ])
    })?;
    plan.structural_work = Some(work);
    Ok(plan)
}

pub fn compile_flow_plan_checked(
    flow: &RenderFlow,
    capabilities: &GpuCapabilities,
) -> Result<CompiledRenderFlowPlan, RenderExecutionGraphCompileError> {
    let report = match flow.validation_report() {
        Ok(report) => report,
        Err(err) => {
            let diagnostics = err
                .issues
                .iter()
                .map(|issue| {
                    RenderExecutionGraphDiagnostic::error(
                        validation_issue_kind(issue),
                        issue.to_string(),
                    )
                    .with_flow(flow.id(), flow.label().to_string())
                })
                .collect::<Vec<_>>();
            return Err(RenderExecutionGraphCompileError::new(diagnostics));
        }
    };
    let mut plan = build_compiled_flow_plan(flow, report.lexical_pass_ids, Vec::new());
    let mut diagnostics = Vec::<RenderExecutionGraphDiagnostic>::new();
    diagnostics.extend(diagnose_compiled_pass_shapes(&plan));
    diagnostics.extend(validate_compiled_flow_capabilities(&plan, capabilities));
    match prepare_structural_gpu_work(&plan) {
        Ok(work) => {
            diagnostics.extend(validate_prepared_gpu_work_capabilities(
                &plan,
                work.graph(),
                capabilities,
            ));
            plan.structural_work = Some(work);
        }
        Err(error) => diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::FlowValidationIssue,
                error.to_string(),
            )
            .with_flow(plan.flow_id, plan.flow_label.clone()),
        ),
    }
    plan.compiler_diagnostics = diagnostics.clone();
    if diagnostics
        .iter()
        .any(RenderExecutionGraphDiagnostic::is_error)
    {
        Err(RenderExecutionGraphCompileError::new(diagnostics))
    } else {
        Ok(plan)
    }
}

fn build_compiled_flow_plan(
    flow: &RenderFlow,
    lexical_pass_ids: Vec<RenderPassId>,
    compiler_diagnostics: Vec<RenderExecutionGraphDiagnostic>,
) -> CompiledRenderFlowPlan {
    let pass_lookup = flow
        .graph()
        .passes
        .passes
        .iter()
        .map(|pass| (pass.id, pass.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut render_passes = Vec::<CompiledPassDescriptor>::with_capacity(lexical_pass_ids.len());

    for (authoring_index, pass_id) in lexical_pass_ids.iter().copied().enumerate() {
        let pass = pass_lookup
            .get(&pass_id)
            .cloned()
            .expect("validated pass should remain in flow graph during planning");

        let compiled = match pass.kind {
            RenderPassKind::Compute => CompiledPassDescriptor::Compute(CompiledComputePass {
                authoring_index,
                node: pass,
            }),
            RenderPassKind::Fullscreen => {
                CompiledPassDescriptor::Fullscreen(CompiledFullscreenPass {
                    authoring_index,
                    node: pass,
                })
            }
            RenderPassKind::Graphics => CompiledPassDescriptor::Graphics(CompiledGraphicsPass {
                authoring_index,
                node: pass,
            }),
            RenderPassKind::Copy => CompiledPassDescriptor::Copy(CompiledCopyPass {
                authoring_index,
                node: pass,
            }),
            RenderPassKind::Present => CompiledPassDescriptor::Present(CompiledPresentPass {
                authoring_index,
                node: pass,
            }),
            RenderPassKind::BuiltinUiComposite => {
                CompiledPassDescriptor::BuiltinUiComposite(CompiledUiCompositePass {
                    authoring_index,
                    node: pass,
                })
            }
        };
        render_passes.push(compiled);
    }

    let execution = compile_execution_plan(&flow.graph().resources, &render_passes);
    CompiledRenderFlowPlan {
        flow_id: flow.id(),
        flow_label: flow.label().to_string(),
        resource_ids_by_label: flow.resource_ids_by_label().clone(),
        resources: flow.graph().resources.clone(),
        render_passes,
        execution,
        compiler_diagnostics,
        structural_work: None,
    }
}

fn prepare_structural_gpu_work(
    plan: &CompiledRenderFlowPlan,
) -> Result<PreparedRenderWorkPlan, crate::plugins::render::RenderGpuWorkAdapterError> {
    let structural_dispatches = plan
        .render_passes
        .iter()
        .filter(|pass| pass.node().kind == RenderPassKind::Compute)
        .filter_map(|pass| {
            let dispatch = match pass.node().compute_dispatch.as_ref()? {
                ComputeDispatchDescriptor::Fixed(value) => *value,
                // ECS/domain projection occurs before exact per-invocation
                // preparation. This nonzero value validates invariant shape.
                ComputeDispatchDescriptor::State(_) => [1, 1, 1],
            };
            Some((pass.pass_id(), dispatch))
        })
        .collect::<BTreeMap<_, _>>();
    prepare_render_gpu_work(
        plan,
        &structural_dispatches,
        (1, 1),
        RenderGpuWorkInstrumentation::Disabled,
    )
}

fn validation_issue_kind(
    issue: &crate::plugins::render::RenderFlowValidationIssue,
) -> RenderExecutionGraphDiagnosticKind {
    use crate::plugins::render::RenderFlowValidationIssue::*;

    match issue {
        DuplicateResourceId { .. }
        | ZeroLengthStorageBuffer { .. }
        | InvalidTextureFormatClass { .. }
        | InvalidTextureFormatPolicy { .. }
        | InvalidTextureUsageForFormat { .. }
        | InvalidTextureSampleModeForFormat { .. }
        | SampledNonTextureResource { .. }
        | WriteTextureOnInvalidResource { .. }
        | InvalidRasterColorOutputResource { .. }
        | InvalidDepthTargetResource { .. }
        | CopyPassMixedResourceClasses { .. }
        | PresentPassReadsNonTexture { .. }
        | InvalidImportedTextureWriteSemantic { .. }
        | UnsupportedImportedTextureWriteKind { .. }
        | InvalidBufferRoleResource { .. }
        | UnsupportedExternalImportedTexture { .. }
        | UnsupportedExternalImportedBuffer { .. }
        | MultipleSurfaceColorImports { .. }
        | MultipleSurfaceDepthImports { .. } => RenderExecutionGraphDiagnosticKind::InvalidResource,
        DuplicatePassId { .. }
        | UnknownNonDataOrderTarget { .. }
        | MultiplePresentPasses { .. }
        | PresentPassNotTerminal { .. }
        | PresentPassNotLast { .. } => RenderExecutionGraphDiagnosticKind::InvalidPassOrder,
        GpuWorkLoweringFailed { .. } => RenderExecutionGraphDiagnosticKind::FlowValidationIssue,
        _ => RenderExecutionGraphDiagnosticKind::FlowValidationIssue,
    }
}
