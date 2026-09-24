//! Focused public workflow helpers for runtime-neutral editor definitions.

use crate::{
    EditorDefinitionDocument, EditorDefinitionDocumentContent, EditorDefinitionDocumentKind,
    EditorDefinitionId, EditorLabOperation, EditorLabOperationReport, EditorThemeDefinition,
    apply_editor_lab_operation, editor_definition_has_blocking_diagnostics, form_theme_tokens,
    validate_editor_definition_document,
};
use ui_definition::UiDefinitionDiagnostic;
use ui_theme::ThemeTokens;

pub fn new_editor_definition_document(
    id: impl Into<EditorDefinitionId>,
    display_name: impl Into<String>,
    kind: EditorDefinitionDocumentKind,
    content: EditorDefinitionDocumentContent,
) -> EditorDefinitionDocument {
    EditorDefinitionDocument::current(id.into(), display_name, kind, content)
}

pub fn validate_editor_document(
    document: &EditorDefinitionDocument,
) -> Vec<UiDefinitionDiagnostic> {
    validate_editor_definition_document(document)
}

pub fn editor_document_has_blocking_diagnostics(diagnostics: &[UiDefinitionDiagnostic]) -> bool {
    editor_definition_has_blocking_diagnostics(diagnostics)
}

pub fn apply_editor_lab_edit(
    document: &EditorDefinitionDocument,
    operation: &EditorLabOperation,
) -> EditorLabOperationReport {
    apply_editor_lab_operation(document, operation)
}

pub fn form_editor_theme_tokens(
    definition: &EditorThemeDefinition,
    base: &ThemeTokens,
) -> Result<ThemeTokens, crate::EditorThemeFormationError> {
    form_theme_tokens(definition, base)
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::collections::BTreeMap;

    #[test]
    fn prelude_supports_normal_editor_definition_workflow() {
        let document = new_editor_definition_document(
            "theme",
            "Theme",
            EditorDefinitionDocumentKind::Theme,
            EditorDefinitionDocumentContent::Theme(EditorThemeDefinition {
                id: "theme".to_string(),
                label: "Theme".to_string(),
                colors: BTreeMap::from([("accent".to_string(), "#3366ff".to_string())]),
                spacing: BTreeMap::new(),
                typography: BTreeMap::new(),
                radius: BTreeMap::new(),
            }),
        );

        let diagnostics = validate_editor_document(&document);
        assert!(
            !editor_document_has_blocking_diagnostics(&diagnostics),
            "{diagnostics:?}"
        );

        let operation = EditorLabOperation {
            id: "theme.accent".to_string(),
            document_id: document.id.clone(),
            target_profile: "editor.workbench".to_string(),
            kind: EditorLabOperationKind::SetThemeColor {
                token: "accent".to_string(),
                value: "#22aaff".to_string(),
            },
            preview_only: false,
            source: None,
        };

        let report = apply_editor_lab_edit(&document, &operation);

        assert!(report.accepted(), "{:?}", report.diagnostics);
        assert!(report.diff.is_some());
    }
}
