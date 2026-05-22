use crate::plugins::render::RenderFlowValidationError;
use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::graph::{
    CompiledFlowExecutionPlan, RenderBackendCapabilityProfile, RenderExecutionGraphCompileError,
    RenderExecutionGraphDiagnostic, RenderExecutionGraphDiagnosticKind, RenderPassKind,
    RenderPassNode, ResourceGraph, compile_execution_plan, compile_resource_lifetime_windows,
    diagnose_compiled_pass_shapes, diagnose_resource_lifetime_windows,
    validate_compiled_flow_capabilities,
};
use crate::plugins::render::{RenderFlowId, RenderPassId, RenderResourceId};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct CompiledRenderFlowPlan {
    pub flow_id: RenderFlowId,
    pub flow_label: String,
    pub resource_ids_by_label: BTreeMap<String, RenderResourceId>,
    pub resources: ResourceGraph,
    pub pass_order: Vec<CompiledPassDescriptor>,
    pub execution: CompiledFlowExecutionPlan,
    pub resource_lifetime_windows: Vec<crate::plugins::render::CompiledResourceLifetimeWindow>,
    pub compiler_diagnostics: Vec<RenderExecutionGraphDiagnostic>,
}

impl CompiledRenderFlowPlan {
    pub fn resource_label(&self, resource_id: RenderResourceId) -> Option<String> {
        self.resource_ids_by_label
            .iter()
            .find_map(|(label, id)| (*id == resource_id).then(|| label.clone()))
    }

    pub fn resource_descriptor(
        &self,
        resource_id: RenderResourceId,
    ) -> Option<&crate::plugins::render::RenderResourceDescriptor> {
        self.resources
            .resources
            .iter()
            .find(|descriptor| *descriptor.id() == resource_id)
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

    pub fn order_index(&self) -> usize {
        match self {
            Self::Compute(value) => value.order_index,
            Self::Fullscreen(value) => value.order_index,
            Self::Graphics(value) => value.order_index,
            Self::Copy(value) => value.order_index,
            Self::Present(value) => value.order_index,
            Self::BuiltinUiComposite(value) => value.order_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledComputePass {
    pub order_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledFullscreenPass {
    pub order_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledGraphicsPass {
    pub order_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledCopyPass {
    pub order_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledPresentPass {
    pub order_index: usize,
    pub node: RenderPassNode,
}

#[derive(Debug, Clone)]
pub struct CompiledUiCompositePass {
    pub order_index: usize,
    pub node: RenderPassNode,
}

pub fn compile_flow_plan(
    flow: &RenderFlow,
) -> Result<CompiledRenderFlowPlan, RenderFlowValidationError> {
    let report = flow.validation_report()?;
    Ok(build_compiled_flow_plan(
        flow,
        report.pass_order,
        Vec::new(),
    ))
}

pub fn compile_flow_plan_checked(
    flow: &RenderFlow,
    profile: &RenderBackendCapabilityProfile,
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
    let mut plan = build_compiled_flow_plan(flow, report.pass_order, Vec::new());
    let mut diagnostics = Vec::<RenderExecutionGraphDiagnostic>::new();
    diagnostics.extend(diagnose_resource_lifetime_windows(
        plan.flow_id,
        plan.flow_label.as_str(),
        &plan.resource_lifetime_windows,
    ));
    diagnostics.extend(diagnose_compiled_pass_shapes(&plan));
    diagnostics.extend(validate_compiled_flow_capabilities(&plan, profile));
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
    pass_order_ids: Vec<RenderPassId>,
    compiler_diagnostics: Vec<RenderExecutionGraphDiagnostic>,
) -> CompiledRenderFlowPlan {
    let pass_lookup = flow
        .graph()
        .passes
        .passes
        .iter()
        .map(|pass| (pass.id, pass.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut pass_order = Vec::<CompiledPassDescriptor>::with_capacity(pass_order_ids.len());

    for (order_index, pass_id) in pass_order_ids.iter().copied().enumerate() {
        let pass = pass_lookup
            .get(&pass_id)
            .cloned()
            .expect("validated pass should remain in flow graph during planning");

        let compiled = match pass.kind {
            RenderPassKind::Compute => CompiledPassDescriptor::Compute(CompiledComputePass {
                order_index,
                node: pass,
            }),
            RenderPassKind::Fullscreen => {
                CompiledPassDescriptor::Fullscreen(CompiledFullscreenPass {
                    order_index,
                    node: pass,
                })
            }
            RenderPassKind::Graphics => CompiledPassDescriptor::Graphics(CompiledGraphicsPass {
                order_index,
                node: pass,
            }),
            RenderPassKind::Copy => CompiledPassDescriptor::Copy(CompiledCopyPass {
                order_index,
                node: pass,
            }),
            RenderPassKind::Present => CompiledPassDescriptor::Present(CompiledPresentPass {
                order_index,
                node: pass,
            }),
            RenderPassKind::BuiltinUiComposite => {
                CompiledPassDescriptor::BuiltinUiComposite(CompiledUiCompositePass {
                    order_index,
                    node: pass,
                })
            }
        };
        pass_order.push(compiled);
    }

    let execution = compile_execution_plan(&flow.graph().resources, &pass_order);
    let labels_by_id = flow
        .resource_ids_by_label()
        .iter()
        .map(|(label, id)| (*id, label.clone()))
        .collect::<BTreeMap<_, _>>();
    let resource_lifetime_windows = compile_resource_lifetime_windows(
        &flow.graph().resources.resources,
        &labels_by_id,
        &pass_order,
    );

    CompiledRenderFlowPlan {
        flow_id: flow.id(),
        flow_label: flow.label().to_string(),
        resource_ids_by_label: flow.resource_ids_by_label().clone(),
        resources: flow.graph().resources.clone(),
        pass_order,
        execution,
        resource_lifetime_windows,
        compiler_diagnostics,
    }
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
        | UnknownPassDependency { .. }
        | MultiplePresentPasses { .. }
        | PresentPassNotTerminal { .. }
        | PresentPassNotLast { .. }
        | PassDependencyCycleDetected { .. } => {
            RenderExecutionGraphDiagnosticKind::InvalidPassOrder
        }
        _ => RenderExecutionGraphDiagnosticKind::FlowValidationIssue,
    }
}
