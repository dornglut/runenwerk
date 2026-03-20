use crate::plugins::render::{RenderFlow, RenderFlowValidationError};

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct RenderDebugGraphDumpState {
    pub revision: u64,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFlowGraphDump {
    pub flow_id: String,
    pub lines: Vec<String>,
}

pub fn dump_flow_graph(
    flow: &RenderFlow,
) -> Result<RenderFlowGraphDump, RenderFlowValidationError> {
    let report = flow.validation_report()?;
    let mut lines = Vec::<String>::new();

    lines.push(format!("flow: {}", flow.id().as_str()));
    lines.push("resources:".to_string());
    for resource in &flow.graph().resources.resources {
        lines.push(format!(
            "  - {} ({:?}, {:?})",
            resource.id().as_str(),
            resource,
            resource.lifetime()
        ));
    }

    lines.push("passes:".to_string());
    for pass in &flow.graph().passes.passes {
        lines.push(format!("  - {} [{:?}]", pass.id.as_str(), pass.kind));
        if !pass.reads.is_empty() {
            lines.push(format!(
                "    reads: {}",
                pass.reads
                    .iter()
                    .map(|id| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !pass.writes.is_empty() {
            lines.push(format!(
                "    writes: {}",
                pass.writes
                    .iter()
                    .map(|id| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !pass.depends_on.is_empty() {
            lines.push(format!(
                "    depends_on: {}",
                pass.depends_on
                    .iter()
                    .map(|id| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    lines.push(format!(
        "execution_order: {}",
        report.pass_order.join(" -> ")
    ));
    Ok(RenderFlowGraphDump {
        flow_id: flow.id().as_str().to_string(),
        lines,
    })
}
