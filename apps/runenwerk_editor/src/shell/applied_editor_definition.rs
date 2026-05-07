//! File: apps/runenwerk_editor/src/shell/applied_editor_definition.rs
//! Purpose: App-owned activation of applied editor definitions into live editor products.

use editor_definition::{
    EditorDefinitionDocument, EditorDefinitionDocumentContent, form_theme_tokens,
};
use ui_definition::UiDefinitionDiagnostic;
use ui_theme::ThemeTokens;

#[derive(Debug, Clone, PartialEq)]
pub enum EditorDefinitionActivation {
    ThemeChanged(ThemeTokens),
    UiTemplateChanged { template_id: String },
    WorkspaceLayoutChanged { workspace_id: String },
    NoLiveActivation,
}

pub fn activate_editor_definition_document(
    document: &EditorDefinitionDocument,
    base_theme: &ThemeTokens,
) -> Result<EditorDefinitionActivation, Vec<UiDefinitionDiagnostic>> {
    match &document.content {
        EditorDefinitionDocumentContent::Theme(theme) => form_theme_tokens(theme, base_theme)
            .map(EditorDefinitionActivation::ThemeChanged)
            .map_err(|error| error.diagnostics),
        _ => Ok(EditorDefinitionActivation::NoLiveActivation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_definition::{
        EditorDefinitionDocumentKind, EditorDefinitionId, EditorThemeDefinition,
    };
    use std::collections::BTreeMap;
    use ui_theme::UiColor;

    #[test]
    fn activating_theme_definition_returns_theme_changed_activation() {
        let document = EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.theme.test"),
            "test_theme.ron",
            EditorDefinitionDocumentKind::Theme,
            EditorDefinitionDocumentContent::Theme(EditorThemeDefinition {
                id: "runenwerk.editor.theme.test".to_string(),
                label: "Test Theme".to_string(),
                colors: BTreeMap::from([("accent".to_string(), "#3366ff".to_string())]),
                spacing: BTreeMap::new(),
                typography: BTreeMap::new(),
                radius: BTreeMap::new(),
            }),
        );

        let activation = activate_editor_definition_document(&document, &ThemeTokens::default())
            .expect("theme definition should activate");

        assert!(matches!(
            activation,
            EditorDefinitionActivation::ThemeChanged(theme)
                if theme.accent == UiColor::new(0.2, 0.4, 1.0, 1.0)
        ));
    }
}
