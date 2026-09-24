//! File: apps/runenwerk_editor/src/shell/applied_editor_definition.rs
//! Purpose: App-owned activation of applied editor definitions into live editor products.

mod activation;
mod catalogs;
mod compatibility;

pub use activation::{EditorDefinitionActivation, activate_editor_definition_document};
pub use catalogs::ActiveEditorDefinitionCatalogs;

#[cfg(test)]
mod tests {
    use super::*;
    use editor_definition::{
        EditorDefinitionDocument, EditorDefinitionDocumentContent, EditorDefinitionDocumentKind,
        EditorDefinitionId, EditorThemeDefinition, EditorWorkspaceHostDefinition,
        EditorWorkspaceLayoutDefinition, EditorWorkspacePanelTabDefinition,
    };
    use std::collections::BTreeMap;
    use ui_theme::{ThemeTokens, UiColor};

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

    #[test]
    fn activating_workspace_layout_returns_live_layout_activation() {
        let document = EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.layout.test"),
            "test_layout.ron",
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceLayout(EditorWorkspaceLayoutDefinition {
                id: "runenwerk.editor.layout.test".to_string(),
                label: "Test Layout".to_string(),
                root: EditorWorkspaceHostDefinition::TabStack {
                    id: "root".to_string(),
                    tabs: vec![EditorWorkspacePanelTabDefinition {
                        id: "root.validation".to_string(),
                        label: "Validation".to_string(),
                        tool_surface: "definition_validation".to_string(),
                    }],
                    active_tab: Some("root.validation".to_string()),
                },
                floating_hosts: Vec::new(),
            }),
        );

        let activation = activate_editor_definition_document(&document, &ThemeTokens::default())
            .expect("workspace layout definition should activate");

        assert!(matches!(
            activation,
            EditorDefinitionActivation::WorkspaceLayoutChanged { workspace_id, layout }
                if workspace_id == "runenwerk.editor.layout.test"
                    && layout.root
                        == EditorWorkspaceHostDefinition::TabStack {
                            id: "root".to_string(),
                            tabs: vec![EditorWorkspacePanelTabDefinition {
                                id: "root.validation".to_string(),
                                label: "Validation".to_string(),
                                tool_surface: "definition_validation".to_string(),
                            }],
                            active_tab: Some("root.validation".to_string()),
                        }
        ));
    }
}
