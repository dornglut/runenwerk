//! File: domain/editor/editor_shell/src/workspace/persisted.rs
//! Purpose: Versioned persisted DTO semantics for workspace structural identity.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    FloatingHostPlaceholderState, PanelHostId, PanelHostKind, PanelHostNode, PanelInstanceId,
    PanelInstanceState, PanelKind, SplitHostState, TabStackHostState, TabStackId, TabStackState,
    ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState, WorkspaceId,
    WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError,
};

pub const PERSISTED_WORKSPACE_STATE_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedWorkspaceStateV1 {
    pub version: u32,
    pub workspace_id: u64,
    pub root_host_id: u64,
    pub hosts: Vec<PersistedPanelHostNodeV1>,
    pub tab_stacks: Vec<PersistedTabStackStateV1>,
    pub panels: Vec<PersistedPanelInstanceStateV1>,
    pub tool_surfaces: Vec<PersistedToolSurfaceStateV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedPanelHostNodeV1 {
    pub id: u64,
    pub kind: PersistedPanelHostKindV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistedPanelHostKindV1 {
    SplitHost {
        axis: PersistedWorkspaceSplitAxisV1,
        fraction: f32,
        first_child: u64,
        second_child: u64,
    },
    TabStackHost {
        tab_stack_id: u64,
    },
    FloatingHostPlaceholder {
        tab_stack_id: Option<u64>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedWorkspaceSplitAxisV1 {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedTabStackStateV1 {
    pub id: u64,
    pub ordered_panels: Vec<u64>,
    pub active_panel: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedPanelInstanceStateV1 {
    pub id: u64,
    pub panel_kind: PersistedPanelKindV1,
    pub active_tool_surface: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedPanelKindV1 {
    Outliner,
    Viewport,
    Inspector,
    Console,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedToolSurfaceStateV1 {
    pub id: u64,
    pub tool_surface_kind: PersistedToolSurfaceKindV1,
    pub mount: PersistedToolSurfaceMountV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedToolSurfaceKindV1 {
    Outliner,
    Viewport,
    Inspector,
    Console,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistedToolSurfaceMountV1 {
    Unmounted,
    Mounted { panel_id: u64 },
}

impl WorkspaceState {
    pub fn to_persisted_v1(&self) -> PersistedWorkspaceStateV1 {
        PersistedWorkspaceStateV1 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V1,
            workspace_id: self.workspace_id.raw(),
            root_host_id: self.root_host_id.raw(),
            hosts: self
                .hosts_by_id
                .values()
                .map(|host| PersistedPanelHostNodeV1 {
                    id: host.id.raw(),
                    kind: persisted_host_kind(host.kind),
                })
                .collect(),
            tab_stacks: self
                .tab_stacks_by_id
                .values()
                .map(|stack| PersistedTabStackStateV1 {
                    id: stack.id.raw(),
                    ordered_panels: stack.ordered_panels.iter().map(|id| id.raw()).collect(),
                    active_panel: stack.active_panel.map(|id| id.raw()),
                })
                .collect(),
            panels: self
                .panels_by_id
                .values()
                .map(|panel| PersistedPanelInstanceStateV1 {
                    id: panel.id.raw(),
                    panel_kind: persisted_panel_kind(panel.panel_kind),
                    active_tool_surface: panel.active_tool_surface.map(|id| id.raw()),
                })
                .collect(),
            tool_surfaces: self
                .tool_surfaces_by_id
                .values()
                .map(|surface| PersistedToolSurfaceStateV1 {
                    id: surface.id.raw(),
                    tool_surface_kind: persisted_tool_surface_kind(surface.tool_surface_kind),
                    mount: persisted_mount(surface.mount),
                })
                .collect(),
        }
    }

    pub fn from_persisted_v1(
        persisted: PersistedWorkspaceStateV1,
    ) -> Result<Self, WorkspaceStateError> {
        if persisted.version != PERSISTED_WORKSPACE_STATE_VERSION_V1 {
            return Err(WorkspaceStateError::PersistedVersionUnsupported(
                persisted.version,
            ));
        }

        let mut hosts_by_id = BTreeMap::new();
        for host in persisted.hosts {
            let host_id = PanelHostId::new(host.id);
            let kind = workspace_host_kind(host.kind);
            hosts_by_id.insert(host_id, PanelHostNode { id: host_id, kind });
        }

        let mut tab_stacks_by_id = BTreeMap::new();
        for stack in persisted.tab_stacks {
            let stack_id = TabStackId::new(stack.id);
            tab_stacks_by_id.insert(
                stack_id,
                TabStackState {
                    id: stack_id,
                    ordered_panels: stack
                        .ordered_panels
                        .into_iter()
                        .map(PanelInstanceId::new)
                        .collect(),
                    active_panel: stack.active_panel.map(PanelInstanceId::new),
                },
            );
        }

        let mut panels_by_id = BTreeMap::new();
        for panel in persisted.panels {
            let panel_id = PanelInstanceId::new(panel.id);
            panels_by_id.insert(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind: workspace_panel_kind(panel.panel_kind),
                    active_tool_surface: panel.active_tool_surface.map(ToolSurfaceInstanceId::new),
                },
            );
        }

        let mut tool_surfaces_by_id = BTreeMap::new();
        for surface in persisted.tool_surfaces {
            let surface_id = ToolSurfaceInstanceId::new(surface.id);
            tool_surfaces_by_id.insert(
                surface_id,
                ToolSurfaceState {
                    id: surface_id,
                    tool_surface_kind: workspace_tool_surface_kind(surface.tool_surface_kind),
                    mount: workspace_mount(surface.mount),
                },
            );
        }

        let state = WorkspaceState {
            workspace_id: WorkspaceId::new(persisted.workspace_id),
            root_host_id: PanelHostId::new(persisted.root_host_id),
            hosts_by_id,
            tab_stacks_by_id,
            panels_by_id,
            tool_surfaces_by_id,
        };
        state.validate_integrity()?;
        Ok(state)
    }
}

fn persisted_host_kind(kind: PanelHostKind) -> PersistedPanelHostKindV1 {
    match kind {
        PanelHostKind::SplitHost(split) => PersistedPanelHostKindV1::SplitHost {
            axis: persisted_axis(split.axis),
            fraction: split.fraction,
            first_child: split.first_child.raw(),
            second_child: split.second_child.raw(),
        },
        PanelHostKind::TabStackHost(tab) => PersistedPanelHostKindV1::TabStackHost {
            tab_stack_id: tab.tab_stack_id.raw(),
        },
        PanelHostKind::FloatingHostPlaceholder(placeholder) => {
            PersistedPanelHostKindV1::FloatingHostPlaceholder {
                tab_stack_id: placeholder.tab_stack_id.map(|id| id.raw()),
            }
        }
    }
}

fn workspace_host_kind(kind: PersistedPanelHostKindV1) -> PanelHostKind {
    match kind {
        PersistedPanelHostKindV1::SplitHost {
            axis,
            fraction,
            first_child,
            second_child,
        } => PanelHostKind::SplitHost(SplitHostState {
            axis: workspace_axis(axis),
            fraction,
            first_child: PanelHostId::new(first_child),
            second_child: PanelHostId::new(second_child),
        }),
        PersistedPanelHostKindV1::TabStackHost { tab_stack_id } => {
            PanelHostKind::TabStackHost(TabStackHostState {
                tab_stack_id: TabStackId::new(tab_stack_id),
            })
        }
        PersistedPanelHostKindV1::FloatingHostPlaceholder { tab_stack_id } => {
            PanelHostKind::FloatingHostPlaceholder(FloatingHostPlaceholderState {
                tab_stack_id: tab_stack_id.map(TabStackId::new),
            })
        }
    }
}

fn persisted_axis(axis: WorkspaceSplitAxis) -> PersistedWorkspaceSplitAxisV1 {
    match axis {
        WorkspaceSplitAxis::Horizontal => PersistedWorkspaceSplitAxisV1::Horizontal,
        WorkspaceSplitAxis::Vertical => PersistedWorkspaceSplitAxisV1::Vertical,
    }
}

fn workspace_axis(axis: PersistedWorkspaceSplitAxisV1) -> WorkspaceSplitAxis {
    match axis {
        PersistedWorkspaceSplitAxisV1::Horizontal => WorkspaceSplitAxis::Horizontal,
        PersistedWorkspaceSplitAxisV1::Vertical => WorkspaceSplitAxis::Vertical,
    }
}

fn persisted_panel_kind(kind: PanelKind) -> PersistedPanelKindV1 {
    match kind {
        PanelKind::Outliner => PersistedPanelKindV1::Outliner,
        PanelKind::Viewport => PersistedPanelKindV1::Viewport,
        PanelKind::Inspector => PersistedPanelKindV1::Inspector,
        PanelKind::Console => PersistedPanelKindV1::Console,
        PanelKind::Placeholder => PersistedPanelKindV1::Placeholder,
    }
}

fn workspace_panel_kind(kind: PersistedPanelKindV1) -> PanelKind {
    match kind {
        PersistedPanelKindV1::Outliner => PanelKind::Outliner,
        PersistedPanelKindV1::Viewport => PanelKind::Viewport,
        PersistedPanelKindV1::Inspector => PanelKind::Inspector,
        PersistedPanelKindV1::Console => PanelKind::Console,
        PersistedPanelKindV1::Placeholder => PanelKind::Placeholder,
    }
}

fn persisted_tool_surface_kind(kind: ToolSurfaceKind) -> PersistedToolSurfaceKindV1 {
    match kind {
        ToolSurfaceKind::Outliner => PersistedToolSurfaceKindV1::Outliner,
        ToolSurfaceKind::Viewport => PersistedToolSurfaceKindV1::Viewport,
        ToolSurfaceKind::Inspector => PersistedToolSurfaceKindV1::Inspector,
        ToolSurfaceKind::Console => PersistedToolSurfaceKindV1::Console,
        ToolSurfaceKind::Placeholder => PersistedToolSurfaceKindV1::Placeholder,
    }
}

fn workspace_tool_surface_kind(kind: PersistedToolSurfaceKindV1) -> ToolSurfaceKind {
    match kind {
        PersistedToolSurfaceKindV1::Outliner => ToolSurfaceKind::Outliner,
        PersistedToolSurfaceKindV1::Viewport => ToolSurfaceKind::Viewport,
        PersistedToolSurfaceKindV1::Inspector => ToolSurfaceKind::Inspector,
        PersistedToolSurfaceKindV1::Console => ToolSurfaceKind::Console,
        PersistedToolSurfaceKindV1::Placeholder => ToolSurfaceKind::Placeholder,
    }
}

fn persisted_mount(mount: ToolSurfaceMount) -> PersistedToolSurfaceMountV1 {
    match mount {
        ToolSurfaceMount::Unmounted => PersistedToolSurfaceMountV1::Unmounted,
        ToolSurfaceMount::Mounted { panel_id } => PersistedToolSurfaceMountV1::Mounted {
            panel_id: panel_id.raw(),
        },
    }
}

fn workspace_mount(mount: PersistedToolSurfaceMountV1) -> ToolSurfaceMount {
    match mount {
        PersistedToolSurfaceMountV1::Unmounted => ToolSurfaceMount::Unmounted,
        PersistedToolSurfaceMountV1::Mounted { panel_id } => ToolSurfaceMount::Mounted {
            panel_id: PanelInstanceId::new(panel_id),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WorkspaceIdentityAllocator, WorkspaceMutation, reduce_workspace};

    fn bootstrap_workspace() -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator)
    }

    #[test]
    fn persisted_roundtrip_preserves_structural_identity() {
        let workspace = bootstrap_workspace();
        let persisted = workspace.to_persisted_v1();
        let restored =
            WorkspaceState::from_persisted_v1(persisted).expect("roundtrip should decode");
        assert_eq!(workspace, restored);
    }

    #[test]
    fn persisted_decode_rejects_invalid_references() {
        let workspace = bootstrap_workspace();
        let mut persisted = workspace.to_persisted_v1();
        persisted.panels.clear();
        let error = WorkspaceState::from_persisted_v1(persisted)
            .expect_err("invalid references must fail decode");
        assert!(matches!(error, WorkspaceStateError::MissingPanel(_)));
    }

    #[test]
    fn persisted_decode_rejects_unsupported_version() {
        let workspace = bootstrap_workspace();
        let mut persisted = workspace.to_persisted_v1();
        persisted.version = 99;
        let error = WorkspaceState::from_persisted_v1(persisted)
            .expect_err("unsupported versions must fail");
        assert!(matches!(
            error,
            WorkspaceStateError::PersistedVersionUnsupported(99)
        ));
    }

    #[test]
    fn persisted_roundtrip_handles_detached_tool_surface_state() {
        let workspace = bootstrap_workspace();
        let viewport_panel = workspace
            .panels_by_id
            .values()
            .find(|panel| panel.panel_kind == PanelKind::Viewport)
            .expect("viewport panel should exist")
            .id;
        let detached = reduce_workspace(
            &workspace,
            WorkspaceMutation::DetachToolSurfaceFromPanel {
                panel_id: viewport_panel,
            },
        )
        .expect("detaching should produce valid state");

        let persisted = detached.to_persisted_v1();
        let restored =
            WorkspaceState::from_persisted_v1(persisted).expect("detached state should decode");
        assert_eq!(detached, restored);
    }
}
