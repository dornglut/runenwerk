//! Editor definition validation.

use crate::{
    CURRENT_EDITOR_DEFINITION_SCHEMA_VERSION, EditorDefinitionBindings, EditorDefinitionDocument,
    EditorDefinitionDocumentContent, EditorWorkspaceHostDefinition, form_theme_tokens,
};
use std::collections::BTreeSet;
use ui_definition::{UiDefinitionDiagnostic, UiDefinitionDiagnosticSeverity, UiTemplateId};
use ui_theme::ThemeTokens;

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

pub fn validate_editor_definition_document(
    document: &EditorDefinitionDocument,
) -> Vec<UiDefinitionDiagnostic> {
    let mut diagnostics = Vec::new();
    if document.schema_version != CURRENT_EDITOR_DEFINITION_SCHEMA_VERSION {
        diagnostics.push(UiDefinitionDiagnostic::error(
            "editor.definition.schema.unsupported_version",
            format!(
                "editor definition schema version '{}' is not supported by version '{}'",
                document.schema_version, CURRENT_EDITOR_DEFINITION_SCHEMA_VERSION
            ),
        ));
    }
    if document.id.as_str().trim().is_empty() {
        diagnostics.push(UiDefinitionDiagnostic::error(
            "editor.definition.id.empty",
            "editor definition id must not be empty",
        ));
    }
    if document.display_name.trim().is_empty() {
        diagnostics.push(UiDefinitionDiagnostic::error(
            "editor.definition.display_name.empty",
            "editor definition display name must not be empty",
        ));
    }
    guard_durable_id(&document.id.0, "editor.definition.id", &mut diagnostics);

    match &document.content {
        EditorDefinitionDocumentContent::UiTemplate(template) => {
            let normalized = ui_definition::normalize_authored_template(template.clone());
            diagnostics.extend(normalized.diagnostics);
        }
        EditorDefinitionDocumentContent::WorkspaceProfile(profile) => {
            guard_required(
                &profile.id,
                "editor.definition.workspace_profile.id",
                &mut diagnostics,
            );
            guard_required(
                &profile.default_layout,
                "editor.definition.workspace_profile.default_layout",
                &mut diagnostics,
            );
            guard_durable_id(
                &profile.id,
                "editor.definition.workspace_profile.id",
                &mut diagnostics,
            );
        }
        EditorDefinitionDocumentContent::WorkspaceLayout(layout) => {
            guard_required(
                &layout.id,
                "editor.definition.workspace_layout.id",
                &mut diagnostics,
            );
            guard_durable_id(
                &layout.id,
                "editor.definition.workspace_layout.id",
                &mut diagnostics,
            );
            validate_workspace_host(&layout.root, &mut BTreeSet::new(), &mut diagnostics);
            for floating in &layout.floating_hosts {
                guard_required(
                    &floating.id,
                    "editor.definition.workspace_layout.floating_host.id",
                    &mut diagnostics,
                );
                guard_durable_id(
                    &floating.id,
                    "editor.definition.workspace_layout.floating_host.id",
                    &mut diagnostics,
                );
                validate_workspace_host(&floating.host, &mut BTreeSet::new(), &mut diagnostics);
            }
        }
        EditorDefinitionDocumentContent::Menu(menu) => {
            guard_required(&menu.id, "editor.definition.menu.id", &mut diagnostics);
            guard_durable_id(&menu.id, "editor.definition.menu.id", &mut diagnostics);
            let mut item_ids = BTreeSet::new();
            for item in &menu.items {
                validate_menu_item(item, &mut item_ids, &mut diagnostics);
            }
        }
        EditorDefinitionDocumentContent::Theme(theme) => {
            guard_required(&theme.id, "editor.definition.theme.id", &mut diagnostics);
            guard_durable_id(&theme.id, "editor.definition.theme.id", &mut diagnostics);
            if let Err(error) = form_theme_tokens(theme, &ThemeTokens::default()) {
                diagnostics.extend(error.diagnostics);
            }
        }
        EditorDefinitionDocumentContent::Shortcuts(shortcuts) => {
            guard_required(
                &shortcuts.id,
                "editor.definition.shortcut_set.id",
                &mut diagnostics,
            );
            guard_durable_id(
                &shortcuts.id,
                "editor.definition.shortcut_set.id",
                &mut diagnostics,
            );
            let mut chords = BTreeSet::new();
            let mut shortcut_ids = BTreeSet::new();
            for shortcut in &shortcuts.shortcuts {
                guard_required(
                    &shortcut.id,
                    "editor.definition.shortcut.id",
                    &mut diagnostics,
                );
                guard_required(
                    &shortcut.command,
                    "editor.definition.shortcut.command",
                    &mut diagnostics,
                );
                guard_required(
                    &shortcut.chord,
                    "editor.definition.shortcut.chord",
                    &mut diagnostics,
                );
                guard_durable_id(
                    &shortcut.id,
                    "editor.definition.shortcut.id",
                    &mut diagnostics,
                );
                if !shortcut_ids.insert(shortcut.id.clone()) {
                    diagnostics.push(UiDefinitionDiagnostic::error(
                        "editor.definition.shortcut.duplicate",
                        format!("duplicate shortcut id '{}'", shortcut.id),
                    ));
                }
                let key = (
                    shortcut.context.clone().unwrap_or_default(),
                    shortcut.chord.clone(),
                );
                if !chords.insert(key) {
                    diagnostics.push(UiDefinitionDiagnostic::error(
                        "editor.definition.shortcut.conflict",
                        format!("duplicate shortcut chord '{}'", shortcut.chord),
                    ));
                }
            }
        }
        EditorDefinitionDocumentContent::CommandBindings(bindings) => {
            guard_required(
                &bindings.id,
                "editor.definition.command_binding_set.id",
                &mut diagnostics,
            );
            guard_durable_id(
                &bindings.id,
                "editor.definition.command_binding_set.id",
                &mut diagnostics,
            );
            let mut binding_ids = BTreeSet::new();
            for binding in &bindings.bindings {
                guard_required(
                    &binding.id,
                    "editor.definition.command_binding.id",
                    &mut diagnostics,
                );
                guard_required(
                    &binding.command,
                    "editor.definition.command_binding.command",
                    &mut diagnostics,
                );
                guard_required(
                    &binding.route_target,
                    "editor.definition.command_binding.route_target",
                    &mut diagnostics,
                );
                guard_durable_id(
                    &binding.id,
                    "editor.definition.command_binding.id",
                    &mut diagnostics,
                );
                if !binding_ids.insert(binding.id.clone()) {
                    diagnostics.push(UiDefinitionDiagnostic::error(
                        "editor.definition.command_binding.duplicate",
                        format!("duplicate command binding id '{}'", binding.id),
                    ));
                }
            }
        }
        EditorDefinitionDocumentContent::PanelRegistry(registry) => {
            guard_required(
                &registry.id,
                "editor.definition.panel_registry.id",
                &mut diagnostics,
            );
            guard_durable_id(
                &registry.id,
                "editor.definition.panel_registry.id",
                &mut diagnostics,
            );
            let mut panel_ids = BTreeSet::new();
            for panel in &registry.panels {
                guard_required(&panel.id, "editor.definition.panel.id", &mut diagnostics);
                guard_required(
                    &panel.label,
                    "editor.definition.panel.label",
                    &mut diagnostics,
                );
                guard_required(
                    &panel.default_tool_surface,
                    "editor.definition.panel.default_tool_surface",
                    &mut diagnostics,
                );
                guard_durable_id(&panel.id, "editor.definition.panel.id", &mut diagnostics);
                guard_durable_id(
                    &panel.default_tool_surface,
                    "editor.definition.panel.default_tool_surface",
                    &mut diagnostics,
                );
                if !panel_ids.insert(panel.id.clone()) {
                    diagnostics.push(UiDefinitionDiagnostic::error(
                        "editor.definition.panel.duplicate",
                        format!("duplicate panel id '{}'", panel.id),
                    ));
                }
            }
        }
        EditorDefinitionDocumentContent::ToolSurfaceRegistry(registry) => {
            guard_required(
                &registry.id,
                "editor.definition.tool_surface_registry.id",
                &mut diagnostics,
            );
            guard_durable_id(
                &registry.id,
                "editor.definition.tool_surface_registry.id",
                &mut diagnostics,
            );
            let mut surface_ids = BTreeSet::new();
            for surface in &registry.tool_surfaces {
                guard_required(
                    &surface.id,
                    "editor.definition.tool_surface.id",
                    &mut diagnostics,
                );
                guard_required(
                    &surface.label,
                    "editor.definition.tool_surface.label",
                    &mut diagnostics,
                );
                guard_required(
                    &surface.provider_family,
                    "editor.definition.tool_surface.provider_family",
                    &mut diagnostics,
                );
                guard_durable_id(
                    &surface.id,
                    "editor.definition.tool_surface.id",
                    &mut diagnostics,
                );
                if !surface_ids.insert(surface.id.clone()) {
                    diagnostics.push(UiDefinitionDiagnostic::error(
                        "editor.definition.tool_surface.duplicate",
                        format!("duplicate tool-surface id '{}'", surface.id),
                    ));
                }
            }
        }
        EditorDefinitionDocumentContent::EditorBindings(bindings) => {
            guard_required(
                bindings.toolbar.template.as_str(),
                "editor.definition.bindings.toolbar.template",
                &mut diagnostics,
            );
            guard_required(
                bindings.shell_chrome_template.as_str(),
                "editor.definition.bindings.shell_chrome_template",
                &mut diagnostics,
            );
        }
    }

    diagnostics
}

pub fn editor_definition_has_blocking_diagnostics(diagnostics: &[UiDefinitionDiagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == UiDefinitionDiagnosticSeverity::Error)
}

fn validate_menu_item(
    item: &crate::EditorMenuItemDefinition,
    seen_ids: &mut BTreeSet<String>,
    diagnostics: &mut Vec<UiDefinitionDiagnostic>,
) {
    guard_required(&item.id, "editor.definition.menu_item.id", diagnostics);
    guard_required(
        &item.label,
        "editor.definition.menu_item.label",
        diagnostics,
    );
    guard_durable_id(&item.id, "editor.definition.menu_item.id", diagnostics);
    if let Some(command) = &item.command {
        guard_required(command, "editor.definition.menu_item.command", diagnostics);
    }
    if !seen_ids.insert(item.id.clone()) {
        diagnostics.push(UiDefinitionDiagnostic::error(
            "editor.definition.menu_item.duplicate",
            format!("duplicate menu item id '{}'", item.id),
        ));
    }
    for child in &item.children {
        validate_menu_item(child, seen_ids, diagnostics);
    }
}

fn validate_workspace_host(
    host: &EditorWorkspaceHostDefinition,
    seen_ids: &mut BTreeSet<String>,
    diagnostics: &mut Vec<UiDefinitionDiagnostic>,
) {
    match host {
        EditorWorkspaceHostDefinition::Split {
            id,
            fraction,
            first,
            second,
            ..
        } => {
            validate_unique_host_id(id, seen_ids, diagnostics);
            if !(*fraction > 0.0 && *fraction < 1.0 && fraction.is_finite()) {
                diagnostics.push(UiDefinitionDiagnostic::error(
                    "editor.definition.workspace_layout.split_fraction.invalid",
                    format!("split host '{id}' has invalid fraction '{fraction}'"),
                ));
            }
            validate_workspace_host(first, seen_ids, diagnostics);
            validate_workspace_host(second, seen_ids, diagnostics);
        }
        EditorWorkspaceHostDefinition::TabStack {
            id,
            tabs,
            active_tab,
        } => {
            validate_unique_host_id(id, seen_ids, diagnostics);
            if tabs.is_empty() {
                diagnostics.push(UiDefinitionDiagnostic::error(
                    "editor.definition.workspace_layout.tab_stack.empty",
                    format!("tab stack '{id}' must contain at least one tab"),
                ));
            }
            let mut tab_ids = BTreeSet::new();
            for tab in tabs {
                guard_required(
                    &tab.id,
                    "editor.definition.workspace_layout.tab.id",
                    diagnostics,
                );
                guard_durable_id(
                    &tab.id,
                    "editor.definition.workspace_layout.tab.id",
                    diagnostics,
                );
                guard_required(
                    &tab.tool_surface,
                    "editor.definition.workspace_layout.tab.tool_surface",
                    diagnostics,
                );
                guard_durable_id(
                    &tab.tool_surface,
                    "editor.definition.workspace_layout.tab.tool_surface",
                    diagnostics,
                );
                if !tab_ids.insert(tab.id.clone()) {
                    diagnostics.push(UiDefinitionDiagnostic::error(
                        "editor.definition.workspace_layout.tab.duplicate",
                        format!("duplicate authored tab id '{}'", tab.id),
                    ));
                }
            }
            if let Some(active_tab) = active_tab
                && !tab_ids.contains(active_tab)
            {
                diagnostics.push(UiDefinitionDiagnostic::error(
                    "editor.definition.workspace_layout.active_tab.unresolved",
                    format!("active tab '{active_tab}' is not in tab stack '{id}'"),
                ));
            }
        }
    }
}

fn validate_unique_host_id(
    id: &str,
    seen_ids: &mut BTreeSet<String>,
    diagnostics: &mut Vec<UiDefinitionDiagnostic>,
) {
    guard_required(
        id,
        "editor.definition.workspace_layout.host.id",
        diagnostics,
    );
    guard_durable_id(
        id,
        "editor.definition.workspace_layout.host.id",
        diagnostics,
    );
    if !seen_ids.insert(id.to_string()) {
        diagnostics.push(UiDefinitionDiagnostic::error(
            "editor.definition.workspace_layout.host.duplicate",
            format!("duplicate authored host id '{id}'"),
        ));
    }
}

fn guard_required(value: &str, field: &str, diagnostics: &mut Vec<UiDefinitionDiagnostic>) {
    if value.trim().is_empty() {
        diagnostics.push(UiDefinitionDiagnostic::error(
            format!("{field}.empty"),
            format!("{field} must not be empty"),
        ));
    }
}

fn guard_durable_id(value: &str, field: &str, diagnostics: &mut Vec<UiDefinitionDiagnostic>) {
    const FORBIDDEN_FRAGMENTS: &[&str] = &[
        "panel_instance",
        "tool_surface_instance",
        "widget_id",
        "focus_id",
        "capture_id",
        "ecs_entity",
        "session_id",
        "runtime_id",
    ];
    if FORBIDDEN_FRAGMENTS
        .iter()
        .any(|fragment| value.contains(fragment))
    {
        diagnostics.push(UiDefinitionDiagnostic::error(
            "editor.definition.authored_id.runtime_identity",
            format!("{field} '{value}' contains runtime/session identity vocabulary"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EditorCommandBindingDefinition, EditorCommandBindingSetDefinition,
        EditorDefinitionDocumentContent, EditorDefinitionDocumentKind, EditorDefinitionId,
        EditorMenuDefinition, EditorMenuItemDefinition, EditorPanelDefinition,
        EditorPanelRegistryDefinition, EditorShortcutDefinition, EditorShortcutSetDefinition,
        EditorToolSurfaceDefinition, EditorToolSurfaceRegistryDefinition,
        EditorWorkspaceHostDefinition, EditorWorkspaceLayoutDefinition,
        EditorWorkspacePanelTabDefinition, EditorWorkspaceSplitAxisDefinition,
    };

    #[test]
    fn duplicate_shortcut_chords_are_blocking() {
        let document = EditorDefinitionDocument::current(
            EditorDefinitionId::from("editor.shortcuts"),
            "Shortcuts",
            EditorDefinitionDocumentKind::Shortcut,
            EditorDefinitionDocumentContent::Shortcuts(EditorShortcutSetDefinition {
                id: "editor.shortcuts".to_string(),
                label: "Shortcuts".to_string(),
                shortcuts: vec![
                    EditorShortcutDefinition {
                        id: "save".to_string(),
                        command: "editor.save".to_string(),
                        chord: "Cmd+S".to_string(),
                        context: None,
                    },
                    EditorShortcutDefinition {
                        id: "save_as".to_string(),
                        command: "editor.save_as".to_string(),
                        chord: "Cmd+S".to_string(),
                        context: None,
                    },
                ],
            }),
        );

        let diagnostics = validate_editor_definition_document(&document);

        assert!(editor_definition_has_blocking_diagnostics(&diagnostics));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "editor.definition.shortcut.conflict" })
        );
    }

    #[test]
    fn workspace_layout_rejects_runtime_session_ids() {
        let document = EditorDefinitionDocument::current(
            EditorDefinitionId::from("workspace.layout"),
            "Layout",
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceLayout(EditorWorkspaceLayoutDefinition {
                id: "workspace.layout".to_string(),
                label: "Layout".to_string(),
                root: EditorWorkspaceHostDefinition::Split {
                    id: "panel_instance:1".to_string(),
                    axis: EditorWorkspaceSplitAxisDefinition::Horizontal,
                    fraction: 0.5,
                    first: Box::new(EditorWorkspaceHostDefinition::TabStack {
                        id: "left".to_string(),
                        tabs: vec![EditorWorkspacePanelTabDefinition {
                            id: "left.outliner".to_string(),
                            label: "Outliner".to_string(),
                            tool_surface: "outliner".to_string(),
                        }],
                        active_tab: Some("left.outliner".to_string()),
                    }),
                    second: Box::new(EditorWorkspaceHostDefinition::TabStack {
                        id: "right".to_string(),
                        tabs: vec![EditorWorkspacePanelTabDefinition {
                            id: "right.inspector".to_string(),
                            label: "Inspector".to_string(),
                            tool_surface: "inspector".to_string(),
                        }],
                        active_tab: Some("right.inspector".to_string()),
                    }),
                },
                floating_hosts: Vec::new(),
            }),
        );

        let diagnostics = validate_editor_definition_document(&document);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "editor.definition.authored_id.runtime_identity"
        }));
    }

    #[test]
    fn workspace_layout_requires_durable_tab_tool_surface_keys() {
        let document = EditorDefinitionDocument::current(
            EditorDefinitionId::from("workspace.layout"),
            "Layout",
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceLayout(EditorWorkspaceLayoutDefinition {
                id: "workspace.layout".to_string(),
                label: "Layout".to_string(),
                root: EditorWorkspaceHostDefinition::TabStack {
                    id: "root".to_string(),
                    tabs: vec![EditorWorkspacePanelTabDefinition {
                        id: "root.bad".to_string(),
                        label: "Bad".to_string(),
                        tool_surface: "tool_surface_instance:7".to_string(),
                    }],
                    active_tab: Some("root.bad".to_string()),
                },
                floating_hosts: Vec::new(),
            }),
        );

        let diagnostics = validate_editor_definition_document(&document);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "editor.definition.authored_id.runtime_identity"
        }));
    }

    #[test]
    fn catalog_definition_validation_rejects_malformed_activation_inputs() {
        let documents = [
            EditorDefinitionDocument::current(
                EditorDefinitionId::from("editor.menu"),
                "Menu",
                EditorDefinitionDocumentKind::Menu,
                EditorDefinitionDocumentContent::Menu(EditorMenuDefinition {
                    id: "editor.menu".to_string(),
                    label: "Menu".to_string(),
                    items: vec![EditorMenuItemDefinition {
                        id: String::new(),
                        label: String::new(),
                        command: Some(String::new()),
                        children: Vec::new(),
                        availability: None,
                    }],
                }),
            ),
            EditorDefinitionDocument::current(
                EditorDefinitionId::from("editor.commands"),
                "Commands",
                EditorDefinitionDocumentKind::CommandBinding,
                EditorDefinitionDocumentContent::CommandBindings(
                    EditorCommandBindingSetDefinition {
                        id: "editor.commands".to_string(),
                        label: "Commands".to_string(),
                        bindings: vec![EditorCommandBindingDefinition {
                            id: String::new(),
                            command: String::new(),
                            route_target: String::new(),
                            capability_requirements: Vec::new(),
                            undoable: false,
                        }],
                    },
                ),
            ),
            EditorDefinitionDocument::current(
                EditorDefinitionId::from("editor.panels"),
                "Panels",
                EditorDefinitionDocumentKind::PanelRegistry,
                EditorDefinitionDocumentContent::PanelRegistry(EditorPanelRegistryDefinition {
                    id: "editor.panels".to_string(),
                    label: "Panels".to_string(),
                    panels: vec![EditorPanelDefinition {
                        id: String::new(),
                        label: String::new(),
                        default_tool_surface: String::new(),
                        allowed_document_kinds: Vec::new(),
                        allowed_workspace_profiles: Vec::new(),
                    }],
                }),
            ),
            EditorDefinitionDocument::current(
                EditorDefinitionId::from("editor.surfaces"),
                "Surfaces",
                EditorDefinitionDocumentKind::ToolSurfaceDefinition,
                EditorDefinitionDocumentContent::ToolSurfaceRegistry(
                    EditorToolSurfaceRegistryDefinition {
                        id: "editor.surfaces".to_string(),
                        label: "Surfaces".to_string(),
                        tool_surfaces: vec![EditorToolSurfaceDefinition {
                            id: String::new(),
                            label: String::new(),
                            provider_family: String::new(),
                            required_capabilities: Vec::new(),
                            allowed_document_kinds: Vec::new(),
                            allowed_workspace_profiles: Vec::new(),
                        }],
                    },
                ),
            ),
        ];

        for document in documents {
            let diagnostics = validate_editor_definition_document(&document);
            assert!(
                editor_definition_has_blocking_diagnostics(&diagnostics),
                "malformed {:?} should be rejected",
                document.kind
            );
        }
    }
}
