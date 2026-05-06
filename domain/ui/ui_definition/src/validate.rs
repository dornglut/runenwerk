//! UI definition validation.

use crate::{
    AuthoredId, AuthoredUiNodePath, AuthoredUiTemplate, UiDefinitionDiagnostic, UiNodeDefinition,
};
use std::collections::BTreeSet;

pub fn validate_authored_template(template: &AuthoredUiTemplate) -> Vec<UiDefinitionDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_id(&template.id, "template", &mut diagnostics);
    let mut ids = BTreeSet::new();
    validate_node(
        &template.root,
        AuthoredUiNodePath::root(template.root.id()),
        &mut ids,
        &mut diagnostics,
    );
    diagnostics
}

fn validate_node(
    node: &UiNodeDefinition,
    path: AuthoredUiNodePath,
    ids: &mut BTreeSet<AuthoredId>,
    diagnostics: &mut Vec<UiDefinitionDiagnostic>,
) {
    validate_id(node.id(), "node", diagnostics);
    if !ids.insert(node.id().clone()) {
        diagnostics.push(
            UiDefinitionDiagnostic::error(
                "ui.definition.node.duplicate",
                format!("duplicate node id '{}'", node.id()),
            )
            .at_path(path.clone()),
        );
    }

    if matches!(node, UiNodeDefinition::Repeat { .. }) && node.children().is_empty() {
        // Repeat children are supplied by referenced templates.
    }

    if let UiNodeDefinition::Split { children, .. } = node
        && children.len() != 2
    {
        diagnostics.push(
            UiDefinitionDiagnostic::error(
                "ui.definition.split.child_count",
                "split nodes require exactly two authored children",
            )
            .at_path(path.clone()),
        );
    }

    for child in node.children() {
        validate_node(child, path.child(child.id()), ids, diagnostics);
    }
}

fn validate_id(id: &AuthoredId, kind: &str, diagnostics: &mut Vec<UiDefinitionDiagnostic>) {
    let value = id.as_str();
    if value.trim().is_empty() {
        diagnostics.push(UiDefinitionDiagnostic::error(
            "ui.definition.id.empty",
            format!("{kind} id must not be empty"),
        ));
        return;
    }
    if value.chars().any(char::is_whitespace) {
        diagnostics.push(UiDefinitionDiagnostic::error(
            "ui.definition.id.whitespace",
            format!("{kind} id '{value}' must not contain whitespace"),
        ));
    }
}
