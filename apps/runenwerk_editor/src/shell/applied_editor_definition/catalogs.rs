//! File: apps/runenwerk_editor/src/shell/applied_editor_definition/catalogs.rs
//! Purpose: Active editor definition catalogs used by live app activation.

use std::collections::BTreeMap;

use editor_definition::{
    EditorCommandBindingSetDefinition, EditorDefinitionBindings, EditorMenuDefinition,
    EditorPanelRegistryDefinition, EditorShortcutSetDefinition,
    EditorToolSurfaceRegistryDefinition, editor_definition_has_blocking_diagnostics,
    validate_editor_bindings,
};
use editor_shell::{EditorCompositionRuntime, ToolSurfaceStableKey};
use ui_definition::{NormalizedUiTemplate, UiDefinitionDiagnostic, UiTemplateId};

use super::compatibility::{
    known_panel_kinds_in_authored_order, known_tool_surface_keys_in_authored_order,
    panel_registry_compatible_with_tool_surfaces, panel_registry_covers_composition,
    tool_surface_registry_covers_composition, tool_surface_registry_covers_panel_defaults,
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

    pub fn available_tool_surface_keys(&self) -> Vec<ToolSurfaceStableKey> {
        self.tool_surface_registry
            .as_ref()
            .map(known_tool_surface_keys_in_authored_order)
            .unwrap_or_default()
    }

    pub fn available_panel_kinds(&self) -> Vec<editor_shell::PanelKind> {
        self.panel_registry
            .as_ref()
            .map(known_panel_kinds_in_authored_order)
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

    pub fn install_shortcuts(
        &mut self,
        shortcuts: EditorShortcutSetDefinition,
        validate_shortcuts: impl FnOnce(&EditorShortcutSetDefinition) -> Vec<UiDefinitionDiagnostic>,
    ) -> Result<(), Vec<UiDefinitionDiagnostic>> {
        let diagnostics = validate_shortcuts(&shortcuts);
        if editor_definition_has_blocking_diagnostics(&diagnostics) {
            return Err(diagnostics);
        }
        self.shortcuts.insert(shortcuts.id.clone(), shortcuts);
        Ok(())
    }

    pub fn install_command_bindings(
        &mut self,
        bindings: EditorCommandBindingSetDefinition,
        is_known_command_key: impl Fn(&str) -> bool,
    ) -> Result<(), Vec<UiDefinitionDiagnostic>> {
        let mut candidate = self.command_bindings.clone();
        candidate.insert(bindings.id.clone(), bindings);
        let diagnostics = validate_command_binding_catalog(&candidate, is_known_command_key);
        if editor_definition_has_blocking_diagnostics(&diagnostics) {
            return Err(diagnostics);
        }
        self.command_bindings = candidate;
        Ok(())
    }

    pub fn install_panel_registry(
        &mut self,
        registry: EditorPanelRegistryDefinition,
        runtime: &EditorCompositionRuntime,
    ) -> Result<(), UiDefinitionDiagnostic> {
        panel_registry_covers_composition(&registry, runtime)?;
        panel_registry_compatible_with_tool_surfaces(
            &registry,
            self.tool_surface_registry.as_ref(),
        )?;
        self.panel_registry = Some(registry);
        Ok(())
    }

    pub fn install_tool_surface_registry(
        &mut self,
        registry: EditorToolSurfaceRegistryDefinition,
        runtime: &EditorCompositionRuntime,
    ) -> Result<(), UiDefinitionDiagnostic> {
        tool_surface_registry_covers_composition(&registry, runtime)?;
        if let Some(panel_registry) = self.panel_registry.as_ref() {
            tool_surface_registry_covers_panel_defaults(&registry, panel_registry)?;
        }
        self.tool_surface_registry = Some(registry);
        Ok(())
    }
}

fn validate_command_binding_catalog(
    command_bindings: &BTreeMap<String, EditorCommandBindingSetDefinition>,
    is_known_command_key: impl Fn(&str) -> bool,
) -> Vec<UiDefinitionDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut binding_ids = BTreeMap::<String, String>::new();
    let mut route_targets = BTreeMap::<String, String>::new();
    for set in command_bindings.values() {
        for binding in &set.bindings {
            if !is_known_command_key(&binding.command) {
                diagnostics.push(UiDefinitionDiagnostic::error(
                    "editor.definition.command_binding.command.unknown",
                    format!(
                        "command binding '{}' references unknown editor command '{}'",
                        binding.id, binding.command
                    ),
                ));
            }
            if let Some(previous_set) = binding_ids.insert(binding.id.clone(), set.id.clone()) {
                diagnostics.push(UiDefinitionDiagnostic::error(
                    "editor.definition.command_binding.duplicate_active_id",
                    format!(
                        "command binding id '{}' is already active in set '{}'",
                        binding.id, previous_set
                    ),
                ));
            }
            if let Some(previous_set) =
                route_targets.insert(binding.route_target.clone(), set.id.clone())
            {
                diagnostics.push(UiDefinitionDiagnostic::error(
                    "editor.definition.command_binding.duplicate_route_target",
                    format!(
                        "route target '{}' is already active in set '{}'",
                        binding.route_target, previous_set
                    ),
                ));
            }
        }
    }
    diagnostics
}
