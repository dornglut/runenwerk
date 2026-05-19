use super::*;

pub(super) fn material_graph_diagnostic(
    code: &MaterialGraphIssueCode,
    subject: &MaterialGraphIssueSubject,
    message: String,
) -> AssetDiagnosticRecord {
    material_diagnostic(
        AssetDiagnosticCode::RatificationRejected,
        format!("material graph rejected {code:?} {subject:?}: {message}"),
    )
    .with_subject(material_graph_diagnostic_subject(subject))
}

pub(super) fn material_graph_diagnostic_subject(subject: &MaterialGraphIssueSubject) -> String {
    match subject {
        MaterialGraphIssueSubject::Document => "material_graph.document".to_string(),
        MaterialGraphIssueSubject::Graph => "material_graph.graph".to_string(),
        MaterialGraphIssueSubject::Node(node_id) => {
            format!("material_graph.node:{}", node_id.raw())
        }
        MaterialGraphIssueSubject::Output => "material_graph.output".to_string(),
    }
}

pub(super) fn material_graph_subject_from_diagnostic(
    subject: Option<&str>,
) -> (Option<graph::NodeId>, Option<graph::PortId>) {
    let Some(subject) = subject else {
        return (None, None);
    };
    if let Some(raw) = subject.strip_prefix("material_graph.node:")
        && let Ok(node_id) = raw.parse::<u64>()
    {
        return (Some(graph::NodeId::new(node_id)), None);
    }
    if let Some(raw) = subject.strip_prefix("material_graph.port:")
        && let Ok(port_id) = raw.parse::<u64>()
    {
        return (None, Some(graph::PortId::new(port_id)));
    }
    (None, None)
}

pub(super) fn material_diagnostic(
    code: AssetDiagnosticCode,
    message: impl Into<String>,
) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::new(code, AssetDiagnosticSeverity::Error, message)
}

