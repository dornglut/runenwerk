//! Editor definition validation.

use crate::EditorDefinitionBindings;
use ui_definition::{UiDefinitionDiagnostic, UiDefinitionDiagnosticSeverity, UiTemplateId};

pub fn validate_editor_bindings(
    bindings: &EditorDefinitionBindings,
    available_templates: impl IntoIterator<Item = UiTemplateId>,
) -> Vec<UiDefinitionDiagnostic> {
    let templates = available_templates
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    if !templates.contains(&bindings.toolbar.template) {
        diagnostics.push(UiDefinitionDiagnostic {
            severity: UiDefinitionDiagnosticSeverity::Error,
            code: "editor.definition.toolbar.template.unresolved".to_string(),
            message: format!(
                "unresolved toolbar template '{}'",
                bindings.toolbar.template
            ),
            path: None,
        });
    }
    if !templates.contains(&bindings.shell_chrome_template) {
        diagnostics.push(UiDefinitionDiagnostic {
            severity: UiDefinitionDiagnosticSeverity::Error,
            code: "editor.definition.shell_chrome.template.unresolved".to_string(),
            message: format!(
                "unresolved shell chrome template '{}'",
                bindings.shell_chrome_template
            ),
            path: None,
        });
    }
    for surface in &bindings.surface_templates {
        if !templates.contains(&surface.template) {
            diagnostics.push(UiDefinitionDiagnostic {
                severity: UiDefinitionDiagnosticSeverity::Error,
                code: "editor.definition.surface.template.unresolved".to_string(),
                message: format!("unresolved surface template '{}'", surface.template),
                path: None,
            });
        }
    }
    diagnostics
}
