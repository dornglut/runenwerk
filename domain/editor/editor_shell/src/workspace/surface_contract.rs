//! File: domain/editor/editor_shell/src/workspace/surface_contract.rs
//! Purpose: Editor-shell tool-surface mapping into ui_surface mount contracts.

use ui_surface::{
    MountedSurfaceInstance, SessionRetentionClass, SurfaceCapabilitySet, SurfaceDefinition,
    SurfaceDefinitionId, SurfaceHostInstanceId, SurfaceInstanceId,
};

use crate::{ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState, WorkspaceState};

pub const OUTLINER_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(1);
pub const VIEWPORT_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(2);
pub const INSPECTOR_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(3);
pub const CONSOLE_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(4);
pub const PLACEHOLDER_SURFACE_DEFINITION_ID: SurfaceDefinitionId = SurfaceDefinitionId::new(5);

pub fn editor_surface_definitions() -> [SurfaceDefinition; 5] {
    [
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
    ]
}

pub fn tool_surface_definition_id(kind: ToolSurfaceKind) -> SurfaceDefinitionId {
    match kind {
        ToolSurfaceKind::Outliner => OUTLINER_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Viewport => VIEWPORT_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Inspector => INSPECTOR_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Console => CONSOLE_SURFACE_DEFINITION_ID,
        ToolSurfaceKind::Placeholder => PLACEHOLDER_SURFACE_DEFINITION_ID,
    }
}

pub fn tool_surface_capability_set(kind: ToolSurfaceKind) -> SurfaceCapabilitySet {
    match kind {
        ToolSurfaceKind::Outliner => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Viewport => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Inspector => SurfaceCapabilitySet::new(true, true, true, false),
        ToolSurfaceKind::Console => SurfaceCapabilitySet::new(true, true, false, false),
        ToolSurfaceKind::Placeholder => SurfaceCapabilitySet::new(true, false, false, false),
    }
}

pub fn tool_surface_session_retention_class(kind: ToolSurfaceKind) -> SessionRetentionClass {
    match kind {
        ToolSurfaceKind::Outliner => SessionRetentionClass::Restorable,
        ToolSurfaceKind::Viewport => SessionRetentionClass::Restorable,
        ToolSurfaceKind::Inspector => SessionRetentionClass::Persistent,
        ToolSurfaceKind::Console => SessionRetentionClass::Ephemeral,
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

        assert_eq!(workspace_id, WorkspaceId::new(1));
        assert_eq!(mounted.len(), 4);
        assert!(
            mounted
                .iter()
                .any(|instance| instance.definition_id == VIEWPORT_SURFACE_DEFINITION_ID)
        );
    }

    #[test]
    fn tool_surface_capabilities_are_explicit_per_surface_kind() {
        let outliner_caps = tool_surface_capability_set(ToolSurfaceKind::Outliner);
        let console_caps = tool_surface_capability_set(ToolSurfaceKind::Console);
        let placeholder_caps = tool_surface_capability_set(ToolSurfaceKind::Placeholder);

        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::Observe));
        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::Interact));
        assert!(outliner_caps.allows(ui_surface::SurfaceCapability::RequestMutation));
        assert!(!outliner_caps.allows(ui_surface::SurfaceCapability::Ratify));

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
