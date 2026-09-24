//! File: apps/runenwerk_editor/src/shell/ui_definition_assets.rs
//! Purpose: App-owned loading policy for checked-in editor UI definition fixtures.

use anyhow::{Context, Result};
use editor_definition::EditorDefinitionBindings;
use std::collections::BTreeMap;
use ui_definition::{
    AuthoredUiTemplate, NormalizedUiTemplate, UiTemplateId, normalize_authored_template,
};

pub const EDITOR_UI_ASSET_SOURCES: &[(&str, &str)] = &[
    (
        "assets/editor/ui/toolbar.ron",
        include_str!("../../../../assets/editor/ui/toolbar.ron"),
    ),
    (
        "assets/editor/ui/shell_chrome.ron",
        include_str!("../../../../assets/editor/ui/shell_chrome.ron"),
    ),
    (
        "assets/editor/ui/surfaces/inspector.ron",
        include_str!("../../../../assets/editor/ui/surfaces/inspector.ron"),
    ),
    (
        "assets/editor/ui/surfaces/outliner.ron",
        include_str!("../../../../assets/editor/ui/surfaces/outliner.ron"),
    ),
    (
        "assets/editor/ui/surfaces/entity_table.ron",
        include_str!("../../../../assets/editor/ui/surfaces/entity_table.ron"),
    ),
    (
        "assets/editor/ui/surfaces/console.ron",
        include_str!("../../../../assets/editor/ui/surfaces/console.ron"),
    ),
    (
        "assets/editor/ui/surfaces/viewport.ron",
        include_str!("../../../../assets/editor/ui/surfaces/viewport.ron"),
    ),
];

pub const EDITOR_BINDINGS_SOURCE: &str =
    include_str!("../../../../assets/editor/ui/editor_bindings.ron");

#[derive(Debug, Clone)]
pub struct LoadedEditorUiDefinitions {
    pub templates: BTreeMap<UiTemplateId, NormalizedUiTemplate>,
    pub bindings: EditorDefinitionBindings,
}

pub fn load_checked_in_editor_ui_definitions() -> Result<LoadedEditorUiDefinitions> {
    let mut templates = BTreeMap::new();
    for (path, source) in EDITOR_UI_ASSET_SOURCES {
        let template: AuthoredUiTemplate =
            ron::from_str(source).with_context(|| format!("failed to parse {path}"))?;
        let normalized = normalize_authored_template(template);
        if normalized.has_errors() {
            anyhow::bail!(
                "invalid UI definition fixture {path}: {:?}",
                normalized.diagnostics
            );
        }
        templates.insert(normalized.id.clone(), normalized);
    }
    let bindings: EditorDefinitionBindings = ron::from_str(EDITOR_BINDINGS_SOURCE)
        .context("failed to parse assets/editor/ui/editor_bindings.ron")?;
    let diagnostics = editor_definition::validate_editor_bindings(
        &bindings,
        templates.keys().cloned().collect::<Vec<_>>(),
    );
    if !diagnostics.is_empty() {
        anyhow::bail!("invalid editor UI binding fixture: {diagnostics:?}");
    }
    Ok(LoadedEditorUiDefinitions {
        templates,
        bindings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_accepts_checked_in_editor_ui_definition_fixtures() {
        let loaded = load_checked_in_editor_ui_definitions()
            .expect("checked-in editor UI definitions should load");
        assert!(
            loaded
                .templates
                .contains_key(&"runenwerk.editor.toolbar".into())
        );
        assert_eq!(loaded.bindings.surface_templates.len(), 5);
    }
}
