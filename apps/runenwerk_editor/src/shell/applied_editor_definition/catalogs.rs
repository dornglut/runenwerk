//! File: apps/runenwerk_editor/src/shell/applied_editor_definition/catalogs.rs
//! Purpose: Active editor definition catalogs used by live app activation.

use std::collections::BTreeMap;

use editor_definition::{
    EditorCommandBindingSetDefinition, EditorDefinitionBindings, EditorMenuDefinition,
    EditorPanelRegistryDefinition, EditorShortcutSetDefinition,
    EditorToolSurfaceRegistryDefinition, editor_definition_has_blocking_diagnostics,
    validate_editor_bindings,
};
use editor_shell::{ToolSurfaceKind, WorkspaceState};
use ui_definition::{NormalizedUiTemplate, UiDefinitionDiagnostic, UiTemplateId};

use super::compatibility::{
    known_tool_surface_kinds_in_authored_order, panel_registry_covers_workspace,
    tool_surface_registry_covers_workspace,
};

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
            .map(known_tool_surface_kinds_in_authored_order)
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
        panel_registry_covers_workspace(&registry, workspace)?;
        self.panel_registry = Some(registry);
        Ok(())
    }

    pub fn install_tool_surface_registry(
        &mut self,
        registry: EditorToolSurfaceRegistryDefinition,
        workspace: &WorkspaceState,
    ) -> Result<(), UiDefinitionDiagnostic> {
        tool_surface_registry_covers_workspace(&registry, workspace)?;
        self.tool_surface_registry = Some(registry);
        Ok(())
    }
}
