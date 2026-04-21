use crate::plugins::render::RenderFlowValidationError;
use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::graph::{
    CompiledFlowExecutionPlan, RenderPassKind, RenderPassNode, ResourceGraph,
    compile_execution_plan,
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
    let pass_lookup = flow
        .graph()
        .passes
        .passes
        .iter()
        .map(|pass| (pass.id, pass.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut pass_order = Vec::<CompiledPassDescriptor>::with_capacity(report.pass_order.len());

    for (order_index, pass_id) in report.pass_order.iter().copied().enumerate() {
        let pass = pass_lookup
            .get(&pass_id)
            .cloned()
            .ok_or_else(|| RenderFlowValidationError {
                issues: vec![],
                message: format!(
                    "internal planning error: validated pass '{pass_id:?}' missing from flow graph"
                ),
            })?;

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

    Ok(CompiledRenderFlowPlan {
        flow_id: flow.id(),
        flow_label: flow.label().to_string(),
        resource_ids_by_label: flow.resource_ids_by_label().clone(),
        resources: flow.graph().resources.clone(),
        pass_order,
        execution,
    })
}
