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
    FixedLayoutProjection, FloatingHostPlaceholderState, PERSISTED_WORKSPACE_STATE_VERSION_V1,
    PanelHostId, PanelHostKind, PanelHostNode, PanelInstanceId, PanelInstanceState, PanelKind,
    PersistedPanelHostKindV1, PersistedPanelHostNodeV1, PersistedPanelInstanceStateV1,
    PersistedPanelKindV1, PersistedTabStackStateV1, PersistedToolSurfaceKindV1,
    PersistedToolSurfaceMountV1, PersistedToolSurfaceStateV1, PersistedWorkspaceSplitAxisV1,
    PersistedWorkspaceStateV1, ProjectedHostNode, ProjectedPanelSlot, ProjectedTabItem,
    ProjectedTabStack, SplitHostState, StructuralWidgetRoutingContext, TabStackHostState,
    TabStackId, TabStackState, ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount,
    ToolSurfaceState, WorkspaceId, WorkspaceIdentityAllocator, WorkspaceIdentitySeed,
    WorkspaceMutation, WorkspaceProjectionArtifact, WorkspaceSplitAxis, WorkspaceState,
    WorkspaceStateError, project_fixed_layout, project_workspace_for_shell, reduce_workspace,
    tab_button_widget_id, tab_float_button_widget_id, tab_stack_content_widget_id,
    tab_stack_header_widget_id,
};
