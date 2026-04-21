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

    lines.push(format!("flow: {}", flow.id()));
    lines.push("resources:".to_string());
    for resource in &flow.graph().resources.resources {
        lines.push(format!(
            "  - {} ({:?}, {:?})",
            resource.id(),
            resource,
            resource.lifetime()
        ));
    }

    lines.push("passes:".to_string());
    for pass in &flow.graph().passes.passes {
        lines.push(format!("  - {} [{:?}]", pass.id, pass.kind));
        if !pass.reads.is_empty() {
            lines.push(format!(
                "    reads: {}",
                pass.reads
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !pass.writes.is_empty() {
            lines.push(format!(
                "    writes: {}",
                pass.writes
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !pass.depends_on.is_empty() {
            lines.push(format!(
                "    depends_on: {}",
                pass.depends_on
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    lines.push(format!(
        "execution_order: {}",
        report
            .pass_order
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" -> ")
    ));
    Ok(RenderFlowGraphDump {
        flow_id: flow.id().to_string(),
        lines,
    })
}
