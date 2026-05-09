//! File: apps/runenwerk_editor/src/shell/applied_editor_definition/compatibility.rs
//! Purpose: Compatibility checks for replacing active editor definition catalogs.

use editor_definition::{EditorPanelRegistryDefinition, EditorToolSurfaceRegistryDefinition};
use editor_shell::{
    PanelKind, ToolSurfaceKind, WorkspaceState, panel_kind_definition_key,
    panel_kind_for_tool_surface_kind, tool_surface_kind_definition_key,
    tool_surface_kind_from_definition_key,
};
use ui_definition::UiDefinitionDiagnostic;

pub fn known_tool_surface_kinds_in_authored_order(
    registry: &EditorToolSurfaceRegistryDefinition,
) -> Vec<ToolSurfaceKind> {
    let mut kinds = Vec::new();
    for definition in &registry.tool_surfaces {
        let Some(kind) = tool_surface_kind_from_definition_key(&definition.id) else {
            continue;
        };
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds
}

pub fn known_panel_kinds_in_authored_order(
    registry: &EditorPanelRegistryDefinition,
) -> Vec<PanelKind> {
    let mut kinds = Vec::new();
    for definition in &registry.panels {
        let Some(surface_kind) = tool_surface_kind_from_definition_key(&definition.id) else {
            continue;
        };
        let kind = panel_kind_for_tool_surface_kind(surface_kind);
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds
}

pub fn panel_registry_covers_workspace(
    registry: &EditorPanelRegistryDefinition,
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
    Ok(())
}

pub fn panel_registry_compatible_with_tool_surfaces(
    registry: &EditorPanelRegistryDefinition,
    tool_surface_registry: Option<&EditorToolSurfaceRegistryDefinition>,
) -> Result<(), UiDefinitionDiagnostic> {
    for panel in &registry.panels {
        if tool_surface_kind_from_definition_key(&panel.default_tool_surface).is_none() {
            return Err(UiDefinitionDiagnostic::error(
                "editor.definition.panel_registry.default_surface_unknown",
                format!(
                    "panel '{}' uses unknown default tool surface '{}'",
                    panel.id, panel.default_tool_surface
                ),
            ));
        }
        if let Some(tool_surface_registry) = tool_surface_registry {
            let default_surface_present = tool_surface_registry
                .tool_surfaces
                .iter()
                .any(|surface| surface.id == panel.default_tool_surface);
            if !default_surface_present {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.definition.panel_registry.default_surface_missing",
                    format!(
                        "panel '{}' uses default tool surface '{}' missing from active tool-surface registry",
                        panel.id, panel.default_tool_surface
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub fn tool_surface_registry_covers_panel_defaults(
    registry: &EditorToolSurfaceRegistryDefinition,
    panel_registry: &EditorPanelRegistryDefinition,
) -> Result<(), UiDefinitionDiagnostic> {
    for panel in &panel_registry.panels {
        if !registry
            .tool_surfaces
            .iter()
            .any(|surface| surface.id == panel.default_tool_surface)
        {
            return Err(UiDefinitionDiagnostic::error(
                "editor.definition.tool_surface_registry.panel_default_missing",
                format!(
                    "active panel '{}' uses default tool surface '{}' missing from tool-surface registry",
                    panel.id, panel.default_tool_surface
                ),
            ));
        }
    }
    Ok(())
}

pub fn tool_surface_registry_covers_workspace(
    registry: &EditorToolSurfaceRegistryDefinition,
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
    Ok(())
}
