//! File: domain/editor/editor_shell/src/lib.rs
//! Crate: editor_shell

pub mod commands;
pub mod composition;
pub mod expression;
pub mod ids;
pub mod observation;
pub mod runtime;
pub mod surface_provider;
pub mod surfaces;
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
pub use surface_provider::*;
pub use surfaces::{
    EditorDefinitionSurfaceAction, EntityTableComponentFilter, EntityTableComponentFilterItem,
    EntityTableDomainMutation, EntityTableFilter, EntityTableHierarchyFilter, EntityTableQuery,
    EntityTableSessionMutation, EntityTableSort, EntityTableSurfaceAction,
    InspectorFieldControlKind, InspectorFieldEditIntent, InspectorSessionMutation,
    InspectorSurfaceAction, OutlinerDomainMutation, OutlinerSurfaceAction, ViewportDomainMutation,
    ViewportSessionMutation, ViewportSurfaceAction,
};
pub use view_models::*;
pub use workspace::{
    BINDINGS_SURFACE_DEFINITION_ID, COMMAND_DIFF_SURFACE_DEFINITION_ID,
    CONSOLE_SURFACE_DEFINITION_ID, DEFINITION_VALIDATION_SURFACE_DEFINITION_ID,
    DOCK_LAYOUT_PREVIEW_SURFACE_DEFINITION_ID, DockSplitSide,
    EDITOR_DESIGN_OUTLINER_SURFACE_DEFINITION_ID, EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
    ENTITY_TABLE_SURFACE_DEFINITION_ID, FloatingHostBounds, FloatingHostPlaceholderState,
    INSPECTOR_SURFACE_DEFINITION_ID, LAYOUT_WORKSPACE_PROFILE_ID,
    MENU_EDITOR_SURFACE_DEFINITION_ID, MODELLING_WORKSPACE_PROFILE_ID,
    OUTLINER_SURFACE_DEFINITION_ID, PERSISTED_WORKSPACE_STATE_VERSION_V1,
    PERSISTED_WORKSPACE_STATE_VERSION_V2, PERSISTED_WORKSPACE_STATE_VERSION_V3,
    PLACEHOLDER_SURFACE_DEFINITION_ID, PanelHostId, PanelHostKind, PanelHostNode, PanelInstanceId,
    PanelInstanceState, PanelKind, PersistedPanelHostKindV1, PersistedPanelHostNodeV1,
    PersistedPanelInstanceStateV1, PersistedPanelInstanceStateV2, PersistedPanelKindV1,
    PersistedPanelKindV2, PersistedTabStackStateV1, PersistedToolSurfaceKindV1,
    PersistedToolSurfaceKindV2, PersistedToolSurfaceMountV1, PersistedToolSurfaceStateV1,
    PersistedToolSurfaceStateV2, PersistedToolSurfaceStateV3, PersistedWorkspaceSplitAxisV1,
    PersistedWorkspaceStateV1, PersistedWorkspaceStateV2, PersistedWorkspaceStateV3,
    ProjectedFloatingHostSlot, ProjectedPanelSlot, ProjectedTabButton, ProjectedTabButtonRoute,
    ProjectedTabDropRoute, ProjectedTabDropSlot, ProjectedTabDropTarget, ProjectedTabStackSlot,
    ProjectedWorkspaceHostSlot, SCENE_WORKSPACE_PROFILE_ID, SHORTCUT_EDITOR_SURFACE_DEFINITION_ID,
    STYLE_INSPECTOR_SURFACE_DEFINITION_ID, SplitHostState, StructuralWidgetRoutingContext,
    THEME_EDITOR_SURFACE_DEFINITION_ID, TabStackHostState, TabStackId, TabStackState,
    ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState,
    UI_CANVAS_SURFACE_DEFINITION_ID, UI_HIERARCHY_SURFACE_DEFINITION_ID,
    VIEWPORT_SURFACE_DEFINITION_ID, WorkspaceDefinitionFormationError, WorkspaceId,
    WorkspaceIdentityAllocator, WorkspaceIdentitySeed, WorkspaceLayoutTemplate, WorkspaceMutation,
    WorkspaceProfile, WorkspaceProfileId, WorkspaceProfileRegistry, WorkspaceProjectionArtifact,
    WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError, compact_empty_tab_stack_areas,
    default_workspace_profile_registry, editor_surface_definitions,
    form_workspace_state_from_definition, mounted_surface_instance, mounted_surface_instances,
    panel_kind_definition_key, panel_kind_for_tool_surface_kind, project_workspace_for_shell,
    projected_host_tab_stacks, reduce_workspace, tool_surface_capability_set,
    tool_surface_definition_id, tool_surface_kind_definition_key,
    tool_surface_kind_from_definition_key, tool_surface_session_retention_class,
    viewport_embed_slot_for,
};
