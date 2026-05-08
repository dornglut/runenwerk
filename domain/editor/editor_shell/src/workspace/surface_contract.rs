//! File: domain/editor/editor_shell/src/workspace/surface_contract.rs
//! Purpose: Editor-shell tool-surface mapping into ui_surface mount contracts.

use ui_surface::{
    MountedSurfaceInstance, SessionRetentionClass, SurfaceCapabilitySet, SurfaceDefinition,
    SurfaceDefinitionId, SurfaceHostInstanceId, SurfaceInstanceId,
};

use crate::{PanelKind, ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState, WorkspaceState};

pub const OUTLINER_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(1);
pub const VIEWPORT_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(2);
pub const INSPECTOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(3);
pub const CONSOLE_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(4);
pub const PLACEHOLDER_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(5);
pub const ENTITY_TABLE_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(6);
pub const EDITOR_DESIGN_OUTLINER_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(7);
pub const UI_HIERARCHY_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(8);
pub const UI_CANVAS_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(9);
pub const STYLE_INSPECTOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(10);
pub const BINDINGS_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(11);
pub const DOCK_LAYOUT_PREVIEW_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(12);
pub const THEME_EDITOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(13);
pub const SHORTCUT_EDITOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(14);
pub const MENU_EDITOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(15);
pub const DEFINITION_VALIDATION_SURFACE_DEFINITION_ID: SurfaceDefinitionId =
    SurfaceDefinitionId::new(16);
pub const COMMAND_DIFF_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(17);

pub fn editor_surface_definitions() -> Vec<SurfaceDefinition> {
    vec![
        SurfaceDefinition::new(
            OUTLINER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.outliner",
            "Outliner",
        ),
        SurfaceDefinition::new(
            VIEWPORT_SURFACE_DEFINITION_ID,
            "editor.tool_surface.viewport",
            "Viewport",
        ),
        SurfaceDefinition::new(
            INSPECTOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.inspector",
            "Inspector",
        ),
        SurfaceDefinition::new(
            CONSOLE_SURFACE_DEFINITION_ID,
            "editor.tool_surface.console",
            "Console",
        ),
        SurfaceDefinition::new(
            PLACEHOLDER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.placeholder",
            "Placeholder",
        ),
        SurfaceDefinition::new(
            ENTITY_TABLE_SURFACE_DEFINITION_ID,
            "editor.tool_surface.entity_table",
            "Entity Table",
        ),
        SurfaceDefinition::new(
            EDITOR_DESIGN_OUTLINER_SURFACE_DEFINITION_ID,
            "editor.tool_surface.editor_design_outliner",
            "Definition Outliner",
        ),
        SurfaceDefinition::new(
            UI_HIERARCHY_SURFACE_DEFINITION_ID,
            "editor.tool_surface.ui_hierarchy",
            "UI Hierarchy",
        ),
        SurfaceDefinition::new(
            UI_CANVAS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.ui_canvas",
            "UI Canvas",
        ),
        SurfaceDefinition::new(
            STYLE_INSPECTOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.style_inspector",
            "Style Inspector",
        ),
        SurfaceDefinition::new(
            BINDINGS_SURFACE_DEFINITION_ID,
            "editor.tool_surface.bindings",
            "Bindings",
        ),
        SurfaceDefinition::new(
            DOCK_LAYOUT_PREVIEW_SURFACE_DEFINITION_ID,
            "editor.tool_surface.dock_layout_preview",
            "Dock Layout Preview",
        ),
        SurfaceDefinition::new(
            THEME_EDITOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.theme_editor",
            "Theme Editor",
        ),
        SurfaceDefinition::new(
            SHORTCUT_EDITOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.shortcut_editor",
            "Shortcut Editor",
        ),
        SurfaceDefinition::new(
            MENU_EDITOR_SURFACE_DEFINITION_ID,
            "editor.tool_surface.menu_editor",
            "Menu Editor",
        ),
        SurfaceDefinition::new(
            DEFINITION_VALIDATION_SURFACE_DEFINITION_ID,
            "editor.tool_surface.definition_validation",
            "Definition Validation",
        ),
        SurfaceDefinition::new(
            COMMAND_DIFF_SURFACE_DEFINITION_ID,
            "editor.tool_surface.command_diff",
            "Command Diff",
        ),
    ]
}

pub fn tool_surface_definition_id(kind: ToolSurfaceKind) -> SurfaceDefinitionId {
    match kind {
        ToolSurfaceKind::Outliner => OUTLINER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::EntityTable => ENTITY_TABLE_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Viewport => VIEWPORT_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Inspector => INSPECTOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Console => CONSOLE_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::EditorDesignOutliner => EDITOR_DESIGN_OUTLINER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::UiHierarchy => UI_HIERARCHY_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::UiCanvas => UI_CANVAS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::StyleInspector => STYLE_INSPECTOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Bindings => BINDINGS_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::DockLayoutPreview => DOCK_LAYOUT_PREVIEW_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::ThemeEditor => THEME_EDITOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::ShortcutEditor => SHORTCUT_EDITOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::MenuEditor => MENU_EDITOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::DefinitionValidation => DEFINITION_VALIDATION_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::CommandDiff => COMMAND_DIFF_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Placeholder => PLACEHOLDER_SURFACE_DEFINITION_ID,
    }
}

pub fn tool_surface_kind_definition_key(kind: ToolSurfaceKind) -> &'static str {
    match kind {
        ToolSurfaceKind::Outliner => "outliner",
        ToolSurfaceKind::EntityTable => "entity_table",
        ToolSurfaceKind::Viewport => "viewport",
        ToolSurfaceKind::Inspector => "inspector",
        ToolSurfaceKind::Console => "console",
        ToolSurfaceKind::EditorDesignOutliner => "editor_design_outliner",
        ToolSurfaceKind::UiHierarchy => "ui_hierarchy",
        ToolSurfaceKind::UiCanvas => "ui_canvas",
        ToolSurfaceKind::StyleInspector => "style_inspector",
        ToolSurfaceKind::Bindings => "bindings",
        ToolSurfaceKind::DockLayoutPreview => "dock_layout_preview",
        ToolSurfaceKind::ThemeEditor => "theme_editor",
        ToolSurfaceKind::ShortcutEditor => "shortcut_editor",
        ToolSurfaceKind::MenuEditor => "menu_editor",
        ToolSurfaceKind::DefinitionValidation => "definition_validation",
        ToolSurfaceKind::CommandDiff => "command_diff",
        ToolSurfaceKind::Placeholder => "placeholder",
    }
}

pub fn panel_kind_definition_key(kind: PanelKind) -> &'static str {
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

pub fn tool_surface_kind_from_definition_key(value: &str) -> Option<ToolSurfaceKind> {
    match value {
        "outliner" => Some(ToolSurfaceKind::Outliner),
        "entity_table" => Some(ToolSurfaceKind::EntityTable),
        "viewport" => Some(ToolSurfaceKind::Viewport),
        "inspector" => Some(ToolSurfaceKind::Inspector),
        "console" => Some(ToolSurfaceKind::Console),
        "editor_design_outliner" => Some(ToolSurfaceKind::EditorDesignOutliner),
        "ui_hierarchy" => Some(ToolSurfaceKind::UiHierarchy),
        "ui_canvas" => Some(ToolSurfaceKind::UiCanvas),
        "style_inspector" => Some(ToolSurfaceKind::StyleInspector),
        "bindings" => Some(ToolSurfaceKind::Bindings),
        "dock_layout_preview" => Some(ToolSurfaceKind::DockLayoutPreview),
        "theme_editor" => Some(ToolSurfaceKind::ThemeEditor),
        "shortcut_editor" => Some(ToolSurfaceKind::ShortcutEditor),
        "menu_editor" => Some(ToolSurfaceKind::MenuEditor),
        "definition_validation" => Some(ToolSurfaceKind::DefinitionValidation),
        "command_diff" => Some(ToolSurfaceKind::CommandDiff),
        "placeholder" => Some(ToolSurfaceKind::Placeholder),
        _ => None,
    }
}

pub fn panel_kind_for_tool_surface_kind(kind: ToolSurfaceKind) -> PanelKind {
    match kind {
        ToolSurfaceKind::Outliner => PanelKind::Outliner,
        ToolSurfaceKind::EntityTable => PanelKind::EntityTable,
        ToolSurfaceKind::Viewport => PanelKind::Viewport,
        ToolSurfaceKind::Inspector => PanelKind::Inspector,
        ToolSurfaceKind::Console => PanelKind::Console,
        ToolSurfaceKind::EditorDesignOutliner => PanelKind::EditorDesignOutliner,
        ToolSurfaceKind::UiHierarchy => PanelKind::UiHierarchy,
        ToolSurfaceKind::UiCanvas => PanelKind::UiCanvas,
        ToolSurfaceKind::StyleInspector => PanelKind::StyleInspector,
        ToolSurfaceKind::Bindings => PanelKind::Bindings,
        ToolSurfaceKind::DockLayoutPreview => PanelKind::DockLayoutPreview,
        ToolSurfaceKind::ThemeEditor => PanelKind::ThemeEditor,
        ToolSurfaceKind::ShortcutEditor => PanelKind::ShortcutEditor,
        ToolSurfaceKind::MenuEditor => PanelKind::MenuEditor,
        ToolSurfaceKind::DefinitionValidation => PanelKind::DefinitionValidation,
        ToolSurfaceKind::CommandDiff => PanelKind::CommandDiff,
        ToolSurfaceKind::Placeholder => PanelKind::Placeholder,
    }
}

pub fn tool_surface_capability_set(kind: ToolSurfaceKind) -> SurfaceCapabilitySet {
    match kind {
        ToolSurfaceKind::Outliner => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::EntityTable => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Viewport => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Inspector => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Console => SurfaceCapabilitySet::new(true, true, false, false),
        ToolSurfaceKind::EditorDesignOutliner
        | ToolSurfaceKind::UiHierarchy
        | ToolSurfaceKind::UiCanvas
        | ToolSurfaceKind::StyleInspector
        | ToolSurfaceKind::Bindings
        | ToolSurfaceKind::DockLayoutPreview
        | ToolSurfaceKind::ThemeEditor
        | ToolSurfaceKind::ShortcutEditor
        | ToolSurfaceKind::MenuEditor
        | ToolSurfaceKind::DefinitionValidation
        | ToolSurfaceKind::CommandDiff => SurfaceCapabilitySet::new(true, true, true, true),
        ToolSurfaceKind::Placeholder => SurfaceCapabilitySet::new(true, false, false, false),
    }
}

pub fn tool_surface_session_retention_class(kind: ToolSurfaceKind) -> SessionRetentionClass {
    match kind {
        ToolSurfaceKind::Outliner => SessionRetentionClass::Restorable,
        ToolSurfaceKind::EntityTable => SessionRetentionClass::Restorable,
        ToolSurfaceKind::Viewport => SessionRetentionClass::Restorable,
        ToolSurfaceKind::Inspector => SessionRetentionClass::Persistent,
        ToolSurfaceKind::Console => SessionRetentionClass::Ephemeral,
        ToolSurfaceKind::EditorDesignOutliner
        | ToolSurfaceKind::UiHierarchy
        | ToolSurfaceKind::UiCanvas
        | ToolSurfaceKind::StyleInspector
        | ToolSurfaceKind::Bindings
        | ToolSurfaceKind::DockLayoutPreview
        | ToolSurfaceKind::ThemeEditor
        | ToolSurfaceKind::ShortcutEditor
        | ToolSurfaceKind::MenuEditor
        | ToolSurfaceKind::DefinitionValidation
        | ToolSurfaceKind::CommandDiff => SessionRetentionClass::Restorable,
        ToolSurfaceKind::Placeholder => SessionRetentionClass::Ephemeral,
    }
}

pub fn mounted_surface_instance(tool_surface: ToolSurfaceState) -> Option<MountedSurfaceInstance> {
    let ToolSurfaceMount::Mounted { panel_id } = tool_surface.mount else {
        return None;
    };

    Some(MountedSurfaceInstance::new(
        SurfaceInstanceId::new(tool_surface.id.raw()),
        tool_surface_definition_id(tool_surface.tool_surface_kind),
        SurfaceHostInstanceId::new(panel_id.raw()),
    ))
}

pub fn mounted_surface_instances(
    workspace_state: &WorkspaceState,
) -> impl Iterator<Item = MountedSurfaceInstance> + '_ {
    workspace_state
        .tool_surfaces()
        .copied()
        .filter_map(mounted_surface_instance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WorkspaceId, WorkspaceIdentityAllocator};

    #[test]
    fn tool_surface_kind_maps_to_stable_definition_identity() {
        assert_eq!(
            tool_surface_definition_id(ToolSurfaceKind::Viewport),
            VIEWPORT_SURFACE_DEFINITION_ID
        );
        assert_eq!(
            tool_surface_definition_id(ToolSurfaceKind::Outliner),
            OUTLINER_SURFACE_DEFINITION_ID
        );
    }

    #[test]
    fn mounted_surface_instances_follow_workspace_mount_state() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);

        let mounted = mounted_surface_instances(&workspace).collect::<Vec<_>>();

        assert_eq!(workspace_id, WorkspaceId::try_from_raw(1).unwrap());
        assert_eq!(mounted.len(), 5);
        assert!(
            mounted
                .iter()
                .any(|instance| instance.definition_id == VIEWPORT_SURFACE_DEFINITION_ID)
        );
    }

    #[test]
    fn tool_surface_capabilities_are_explicit_per_surface_kind() {
        let outliner_caps = tool_surface_capability_set(ToolSurfaceKind::Outliner);
        let entity_table_caps = tool_surface_capability_set(ToolSurfaceKind::EntityTable);
        let console_caps = tool_surface_capability_set(ToolSurfaceKind::Console);
        let placeholder_caps = tool_surface_capability_set(ToolSurfaceKind::Placeholder);

        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!outliner_caps.allows(ui_surface::SurfaceCapability::Ratify));

        assert!(entity_table_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(entity_table_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(entity_table_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!entity_table_caps.allows(ui_surface::SurfaceCapability::Ratify));

        assert!(console_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(console_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(!console_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!console_caps.allows(ui_surface::SurfaceCapability::Ratify));

        assert!(placeholder_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(!placeholder_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(!placeholder_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!placeholder_caps.allows(ui_surface::SurfaceCapability::Ratify));
    }

    #[test]
    fn tool_surface_retention_class_is_explicit_per_surface_kind() {
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Outliner),
            SessionRetentionClass::Restorable,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Viewport),
            SessionRetentionClass::Restorable,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::EntityTable),
            SessionRetentionClass::Restorable,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Inspector),
            SessionRetentionClass::Persistent,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Console),
            SessionRetentionClass::Ephemeral,
        );
        assert_eq!(
            tool_surface_session_retention_class(ToolSurfaceKind::Placeholder),
            SessionRetentionClass::Ephemeral,
        );
    }
}
