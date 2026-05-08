//! File: apps/runenwerk_editor/src/shell/applied_editor_definition.rs
//! Purpose: App-owned activation of applied editor definitions into live editor products.

use editor_definition::{
    EditorCommandBindingSetDefinition, EditorDefinitionBindings, EditorDefinitionDocument,
    EditorDefinitionDocumentContent, EditorMenuDefinition, EditorPanelRegistryDefinition,
    EditorShortcutSetDefinition, EditorToolSurfaceRegistryDefinition,
    EditorWorkspaceLayoutDefinition, editor_definition_has_blocking_diagnostics, form_theme_tokens,
    validate_editor_bindings, validate_editor_definition_document,
};
use editor_shell::{
    PanelKind, ToolSurfaceKind, WorkspaceState, tool_surface_kind_definition_key,
    tool_surface_kind_from_definition_key,
};
use std::collections::BTreeMap;
use ui_definition::UiDefinitionDiagnostic;
use ui_definition::{NormalizedUiTemplate, UiTemplateId, normalize_authored_template};
use ui_theme::ThemeTokens;

#[derive(Debug, Clone, PartialEq)]
pub enum EditorDefinitionActivation {
    ThemeChanged(ThemeTokens),
    UiTemplateCatalogChanged {
        template_id: String,
        template: NormalizedUiTemplate,
    },
    EditorBindingsCatalogChanged(EditorDefinitionBindings),
    MenuCatalogChanged {
        menu_id: String,
        menu: EditorMenuDefinition,
    },
    ShortcutCatalogChanged {
        shortcut_set_id: String,
        shortcuts: EditorShortcutSetDefinition,
    },
    CommandBindingCatalogChanged {
        command_binding_set_id: String,
        command_bindings: EditorCommandBindingSetDefinition,
    },
    PanelRegistryCatalogChanged {
        registry_id: String,
        registry: EditorPanelRegistryDefinition,
    },
    ToolSurfaceRegistryCatalogChanged {
        registry_id: String,
        registry: EditorToolSurfaceRegistryDefinition,
    },
    WorkspaceLayoutChanged {
        workspace_id: String,
        layout: EditorWorkspaceLayoutDefinition,
    },
    NoLiveActivation,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActiveEditorDefinitionCatalogs {
    templates: BTreeMap<UiTemplateId, NormalizedUiTemplate>,
    editor_bindings: Option<EditorDefinitionBindings>,
    menus: BTreeMap<String, EditorMenuDefinition>,
    shortcuts: BTreeMap<String, EditorShortcutSetDefinition>,
    command_bindings: BTreeMap<String, EditorCommandBindingSetDefinition>,
    panel_registry: Option<EditorPanelRegistryDefinition>,
    tool_surface_registry: Option<EditorToolSurfaceRegistryDefinition>,
}

impl ActiveEditorDefinitionCatalogs {
    pub fn templates(&self) -> &BTreeMap<UiTemplateId, NormalizedUiTemplate> {
        &self.templates
    }

    pub fn editor_bindings(&self) -> Option<&EditorDefinitionBindings> {
        self.editor_bindings.as_ref()
    }

    pub fn menus(&self) -> &BTreeMap<String, EditorMenuDefinition> {
        &self.menus
    }

    pub fn shortcuts(&self) -> &BTreeMap<String, EditorShortcutSetDefinition> {
        &self.shortcuts
    }

    pub fn command_bindings(&self) -> &BTreeMap<String, EditorCommandBindingSetDefinition> {
        &self.command_bindings
    }

    pub fn command_for_route_target(&self, route_target: &str) -> Option<&str> {
        self.command_bindings
            .values()
            .flat_map(|set| set.bindings.iter())
            .find(|binding| binding.route_target == route_target)
            .map(|binding| binding.command.as_str())
    }

    pub fn route_target_for_command(&self, command: &str) -> Option<&str> {
        self.command_bindings
            .values()
            .flat_map(|set| set.bindings.iter())
            .find(|binding| binding.command == command)
            .map(|binding| binding.route_target.as_str())
    }

    pub fn panel_registry(&self) -> Option<&EditorPanelRegistryDefinition> {
        self.panel_registry.as_ref()
    }

    pub fn tool_surface_registry(&self) -> Option<&EditorToolSurfaceRegistryDefinition> {
        self.tool_surface_registry.as_ref()
    }

    pub fn available_tool_surface_kinds(&self) -> Vec<ToolSurfaceKind> {
        self.tool_surface_registry
            .as_ref()
            .map(|registry| {
                registry
                    .tool_surfaces
                    .iter()
                    .filter_map(|definition| tool_surface_kind_from_definition_key(&definition.id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn install_template(&mut self, template: NormalizedUiTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn install_editor_bindings(
        &mut self,
        bindings: EditorDefinitionBindings,
    ) -> Result<(), Vec<UiDefinitionDiagnostic>> {
        let diagnostics = validate_editor_bindings(&bindings, self.templates.keys().cloned());
        if editor_definition_has_blocking_diagnostics(&diagnostics) {
            return Err(diagnostics);
        }
        self.editor_bindings = Some(bindings);
        Ok(())
    }

    pub fn install_menu(&mut self, menu: EditorMenuDefinition) {
        self.menus.insert(menu.id.clone(), menu);
    }

    pub fn install_shortcuts(&mut self, shortcuts: EditorShortcutSetDefinition) {
        self.shortcuts.insert(shortcuts.id.clone(), shortcuts);
    }

    pub fn install_command_bindings(&mut self, bindings: EditorCommandBindingSetDefinition) {
        self.command_bindings.insert(bindings.id.clone(), bindings);
    }

    pub fn install_panel_registry(
        &mut self,
        registry: EditorPanelRegistryDefinition,
        workspace: &WorkspaceState,
    ) -> Result<(), UiDefinitionDiagnostic> {
        for panel in workspace.panels() {
            let key = panel_kind_definition_key(panel.panel_kind);
            if !registry
                .panels
                .iter()
                .any(|definition| definition.id == key)
            {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.definition.panel_registry.active_workspace_missing_panel",
                    format!("active workspace uses panel '{key}' missing from panel registry"),
                ));
            }
        }
        self.panel_registry = Some(registry);
        Ok(())
    }

    pub fn install_tool_surface_registry(
        &mut self,
        registry: EditorToolSurfaceRegistryDefinition,
        workspace: &WorkspaceState,
    ) -> Result<(), UiDefinitionDiagnostic> {
        for surface in workspace.tool_surfaces() {
            let key = tool_surface_kind_definition_key(surface.tool_surface_kind);
            if !registry
                .tool_surfaces
                .iter()
                .any(|definition| definition.id == key)
            {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.definition.tool_surface_registry.active_workspace_missing_surface",
                    format!(
                        "active workspace uses tool surface '{key}' missing from tool-surface registry"
                    ),
                ));
            }
        }
        self.tool_surface_registry = Some(registry);
        Ok(())
    }
}

pub fn activate_editor_definition_document(
    document: &EditorDefinitionDocument,
    base_theme: &ThemeTokens,
) -> Result<EditorDefinitionActivation, Vec<UiDefinitionDiagnostic>> {
    match &document.content {
        EditorDefinitionDocumentContent::Theme(theme) => form_theme_tokens(theme, base_theme)
            .map(EditorDefinitionActivation::ThemeChanged)
            .map_err(|error| error.diagnostics),
        EditorDefinitionDocumentContent::WorkspaceLayout(layout) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::WorkspaceLayoutChanged {
                    workspace_id: layout.id.clone(),
                    layout: layout.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::UiTemplate(template) => {
            let normalized = normalize_authored_template(template.clone());
            if editor_definition_has_blocking_diagnostics(&normalized.diagnostics) {
                Err(normalized.diagnostics)
            } else {
                Ok(EditorDefinitionActivation::UiTemplateCatalogChanged {
                    template_id: normalized.id.as_str().to_string(),
                    template: normalized,
                })
            }
        }
        EditorDefinitionDocumentContent::EditorBindings(bindings) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::EditorBindingsCatalogChanged(
                    bindings.clone(),
                ))
            }
        }
        EditorDefinitionDocumentContent::Menu(menu) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::MenuCatalogChanged {
                    menu_id: menu.id.clone(),
                    menu: menu.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::Shortcuts(shortcuts) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::ShortcutCatalogChanged {
                    shortcut_set_id: shortcuts.id.clone(),
                    shortcuts: shortcuts.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::CommandBindings(command_bindings) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::CommandBindingCatalogChanged {
                    command_binding_set_id: command_bindings.id.clone(),
                    command_bindings: command_bindings.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::PanelRegistry(registry) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::PanelRegistryCatalogChanged {
                    registry_id: registry.id.clone(),
                    registry: registry.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::ToolSurfaceRegistry(registry) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(
                    EditorDefinitionActivation::ToolSurfaceRegistryCatalogChanged {
                        registry_id: registry.id.clone(),
                        registry: registry.clone(),
                    },
                )
            }
        }
        _ => Ok(EditorDefinitionActivation::NoLiveActivation),
    }
}

fn panel_kind_definition_key(kind: PanelKind) -> &'static str {
    match kind {
        PanelKind::Outliner => "outliner",
        PanelKind::EntityTable => "entity_table",
        PanelKind::Viewport => "viewport",
        PanelKind::Inspector => "inspector",
        PanelKind::Console => "console",
        PanelKind::EditorDesignOutliner => "editor_design_outliner",
        PanelKind::UiHierarchy => "ui_hierarchy",
        PanelKind::UiCanvas => "ui_canvas",
        PanelKind::StyleInspector => "style_inspector",
        PanelKind::Bindings => "bindings",
        PanelKind::DockLayoutPreview => "dock_layout_preview",
        PanelKind::ThemeEditor => "theme_editor",
        PanelKind::ShortcutEditor => "shortcut_editor",
        PanelKind::MenuEditor => "menu_editor",
        PanelKind::DefinitionValidation => "definition_validation",
        PanelKind::CommandDiff => "command_diff",
        PanelKind::Placeholder => "placeholder",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_definition::{
        EditorDefinitionDocumentKind, EditorDefinitionId, EditorThemeDefinition,
        EditorWorkspaceHostDefinition, EditorWorkspaceLayoutDefinition,
        EditorWorkspacePanelTabDefinition,
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
