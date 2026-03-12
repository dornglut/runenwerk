use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::graph::{RenderPassKind, RenderPassNode, ResourceGraph};
use crate::plugins::render::RenderFlowValidationError;

#[derive(Debug, Clone)]
pub struct CompiledRenderFlowPlan {
    pub flow_id: String,
    pub resources: ResourceGraph,
    pub pass_order: Vec<CompiledPassDescriptor>,
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
    pub fn pass_id(&self) -> &str {
        self.node().id.as_str()
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
    let report = flow.validate()?;
    let pass_lookup = flow
        .graph()
        .passes
        .passes
        .iter()
        .map(|pass| (pass.id.as_str().to_string(), pass.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut pass_order = Vec::<CompiledPassDescriptor>::with_capacity(report.pass_order.len());

    for (order_index, pass_id) in report.pass_order.iter().enumerate() {
        let pass = pass_lookup.get(pass_id).cloned().ok_or_else(|| {
            RenderFlowValidationError {
                issues: vec![format!(
                    "internal planning error: validated pass '{}' missing from flow graph",
                    pass_id
                )],
            }
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

    Ok(CompiledRenderFlowPlan {
        flow_id: flow.id().as_str().to_string(),
        resources: flow.graph().resources.clone(),
        pass_order,
    })
}
