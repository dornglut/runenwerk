//! File: domain/editor/editor_shell/src/lib.rs
//! Crate: editor_shell

pub mod commands;
pub mod composition;
pub mod expression;
pub mod ids;
pub mod observation;
pub mod runtime;
pub mod view_models;
pub mod workspace;

#[cfg(test)]
mod tests;

pub use commands::*;
pub use composition::*;
pub use expression::*;
pub use ids::*;
pub use observation::*;
pub use runtime::*;
pub use view_models::*;
pub use workspace::{
    CONSOLE_SURFACE_DEFINITION_ID, ENTITY_TABLE_SURFACE_DEFINITION_ID, FixedLayoutProjection,
    FloatingHostBounds, FloatingHostPlaceholderState, INSPECTOR_SURFACE_DEFINITION_ID,
    OUTLINER_SURFACE_DEFINITION_ID, PERSISTED_WORKSPACE_STATE_VERSION_V1,
    PERSISTED_WORKSPACE_STATE_VERSION_V2, PLACEHOLDER_SURFACE_DEFINITION_ID, PanelHostId,
    PanelHostKind, PanelHostNode, PanelInstanceId, PanelInstanceState, PanelKind,
    PersistedPanelHostKindV1, PersistedPanelHostNodeV1, PersistedPanelInstanceStateV1,
    PersistedPanelInstanceStateV2, PersistedPanelKindV1, PersistedPanelKindV2,
    PersistedTabStackStateV1, PersistedToolSurfaceKindV1, PersistedToolSurfaceKindV2,
    PersistedToolSurfaceMountV1, PersistedToolSurfaceStateV1, PersistedToolSurfaceStateV2,
    PersistedWorkspaceSplitAxisV1, PersistedWorkspaceStateV1, PersistedWorkspaceStateV2,
    ProjectedFloatingHostSlot, ProjectedPanelSlot, ProjectedTabButton, ProjectedTabButtonRoute,
    ProjectedTabDropRoute, ProjectedTabDropSlot, ProjectedTabDropTarget, ProjectedTabStackSlot,
    SplitHostState, StructuralWidgetRoutingContext, TabStackHostState, TabStackId, TabStackState,
    ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState,
    VIEWPORT_SURFACE_DEFINITION_ID, WorkspaceId, WorkspaceIdentityAllocator, WorkspaceIdentitySeed,
    WorkspaceMutation, WorkspaceProjectionArtifact, WorkspaceSplitAxis, WorkspaceState,
    WorkspaceStateError, editor_surface_definitions, mounted_surface_instance,
    mounted_surface_instances, project_fixed_layout, project_workspace_for_shell, reduce_workspace,
    tool_surface_capability_set, tool_surface_session_retention_class, viewport_embed_slot_for,
};
