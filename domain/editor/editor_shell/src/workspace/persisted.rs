//! File: domain/editor/editor_shell/src/workspace/persisted.rs
//! Purpose: Versioned persisted DTO semantics for workspace structural identity.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    FloatingHostBounds, FloatingHostPlaceholderState, PanelHostId, PanelHostKind, PanelHostNode,
    PanelInstanceId, PanelInstanceState, PanelKind, SplitHostState, TabStackHostState, TabStackId,
    TabStackState, ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState,
    WorkspaceId, WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError,
};

pub const PERSISTED_WORKSPACE_STATE_VERSION_V1: u32 = 1;
pub const PERSISTED_WORKSPACE_STATE_VERSION_V2: u32 = 2;

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
pub struct PersistedWorkspaceStateV2 {
    pub version: u32,
    pub workspace_id: u64,
    pub root_host_id: u64,
    pub hosts: Vec<PersistedPanelHostNodeV1>,
    pub tab_stacks: Vec<PersistedTabStackStateV1>,
    pub panels: Vec<PersistedPanelInstanceStateV2>,
    pub tool_surfaces: Vec<PersistedToolSurfaceStateV2>,
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
        #[serde(default = "default_floating_host_bounds_v1")]
        bounds: PersistedFloatingHostBoundsV1,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PersistedFloatingHostBoundsV1 {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedWorkspaceSplitAxisV1 {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedTabStackStateV1 {
    pub id: u64,
    pub ordered_panels: Vec<u64>,
    pub active_panel: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked_tool_surface_kind: Option<PersistedToolSurfaceKindV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedPanelInstanceStateV1 {
    pub id: u64,
    pub panel_kind: PersistedPanelKindV1,
    pub active_tool_surface: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedPanelKindV1 {
    Outliner,
    Viewport,
    Inspector,
    Console,
    Placeholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedPanelKindV2 {
    Outliner,
    EntityTable,
    Viewport,
    Inspector,
    Console,
    EditorDesignOutliner,
    UiHierarchy,
    UiCanvas,
    StyleInspector,
    Bindings,
    DockLayoutPreview,
    ThemeEditor,
    ShortcutEditor,
    MenuEditor,
    DefinitionValidation,
    CommandDiff,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedPanelInstanceStateV2 {
    pub id: u64,
    pub panel_kind: PersistedPanelKindV2,
    pub active_tool_surface: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedToolSurfaceStateV1 {
    pub id: u64,
    pub tool_surface_kind: PersistedToolSurfaceKindV1,
    pub mount: PersistedToolSurfaceMountV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedToolSurfaceKindV1 {
    Outliner,
    Viewport,
    Inspector,
    Console,
    Placeholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedToolSurfaceKindV2 {
    Outliner,
    EntityTable,
    Viewport,
    Inspector,
    Console,
    EditorDesignOutliner,
    UiHierarchy,
    UiCanvas,
    StyleInspector,
    Bindings,
    DockLayoutPreview,
    ThemeEditor,
    ShortcutEditor,
    MenuEditor,
    DefinitionValidation,
    CommandDiff,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedToolSurfaceStateV2 {
    pub id: u64,
    pub tool_surface_kind: PersistedToolSurfaceKindV2,
    pub mount: PersistedToolSurfaceMountV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistedToolSurfaceMountV1 {
    Unmounted,
    Mounted { panel_id: u64 },
}

macro_rules! persisted_id {
    ($ty:ty, $raw:expr, $message:literal) => {
        <$ty>::try_from_raw($raw)
            .map_err(|_| WorkspaceStateError::PersistedSchemaViolation($message))
    };
}

impl WorkspaceState {
    pub fn to_persisted_v2(&self) -> PersistedWorkspaceStateV2 {
        PersistedWorkspaceStateV2 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V2,
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
                    locked_tool_surface_kind: stack
                        .locked_tool_surface_kind
                        .map(persisted_tool_surface_kind_v2),
                })
                .collect(),
            panels: self
                .panels_by_id
                .values()
                .map(|panel| PersistedPanelInstanceStateV2 {
                    id: panel.id.raw(),
                    panel_kind: persisted_panel_kind_v2(panel.panel_kind),
                    active_tool_surface: panel.active_tool_surface.map(|id| id.raw()),
                })
                .collect(),
            tool_surfaces: self
                .tool_surfaces_by_id
                .values()
                .map(|surface| PersistedToolSurfaceStateV2 {
                    id: surface.id.raw(),
                    tool_surface_kind: persisted_tool_surface_kind_v2(surface.tool_surface_kind),
                    mount: persisted_mount(surface.mount),
                })
                .collect(),
        }
    }

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
                    locked_tool_surface_kind: None,
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
            let host_id =
                persisted_id!(PanelHostId, host.id, "persisted host id must be non-zero")?;
            let kind = workspace_host_kind(host.kind)?;
            hosts_by_id.insert(host_id, PanelHostNode { id: host_id, kind });
        }

        let mut tab_stacks_by_id = BTreeMap::new();
        for stack in persisted.tab_stacks {
            let stack_id = persisted_id!(
                TabStackId,
                stack.id,
                "persisted tab-stack id must be non-zero"
            )?;
            let ordered_panels = stack
                .ordered_panels
                .into_iter()
                .map(|panel_id| {
                    persisted_id!(
                        PanelInstanceId,
                        panel_id,
                        "persisted ordered panel id must be non-zero"
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let active_panel = stack
                .active_panel
                .map(|panel_id| {
                    persisted_id!(
                        PanelInstanceId,
                        panel_id,
                        "persisted active panel id must be non-zero"
                    )
                })
                .transpose()?;
            tab_stacks_by_id.insert(
                stack_id,
                TabStackState {
                    id: stack_id,
                    ordered_panels,
                    active_panel,
                    locked_tool_surface_kind: stack
                        .locked_tool_surface_kind
                        .map(workspace_tool_surface_kind_v2),
                },
            );
        }

        let mut panels_by_id = BTreeMap::new();
        for panel in persisted.panels {
            let panel_id = persisted_id!(
                PanelInstanceId,
                panel.id,
                "persisted panel id must be non-zero"
            )?;
            let active_tool_surface = panel
                .active_tool_surface
                .map(|surface_id| {
                    persisted_id!(
                        ToolSurfaceInstanceId,
                        surface_id,
                        "persisted active tool-surface id must be non-zero"
                    )
                })
                .transpose()?;
            panels_by_id.insert(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind: workspace_panel_kind(panel.panel_kind),
                    active_tool_surface,
                },
            );
        }

        let mut tool_surfaces_by_id = BTreeMap::new();
        for surface in persisted.tool_surfaces {
            let surface_id = persisted_id!(
                ToolSurfaceInstanceId,
                surface.id,
                "persisted tool-surface id must be non-zero"
            )?;
            tool_surfaces_by_id.insert(
                surface_id,
                ToolSurfaceState {
                    id: surface_id,
                    tool_surface_kind: workspace_tool_surface_kind(surface.tool_surface_kind),
                    mount: workspace_mount(surface.mount)?,
                },
            );
        }

        let state = WorkspaceState {
            workspace_id: persisted_id!(
                WorkspaceId,
                persisted.workspace_id,
                "persisted workspace id must be non-zero"
            )?,
            root_host_id: persisted_id!(
                PanelHostId,
                persisted.root_host_id,
                "persisted root host id must be non-zero"
            )?,
            hosts_by_id,
            tab_stacks_by_id,
            panels_by_id,
            tool_surfaces_by_id,
        };
        state.validate_integrity()?;
        Ok(state)
    }

    pub fn from_persisted_v2(
        persisted: PersistedWorkspaceStateV2,
    ) -> Result<Self, WorkspaceStateError> {
        if persisted.version != PERSISTED_WORKSPACE_STATE_VERSION_V2 {
            return Err(WorkspaceStateError::PersistedVersionUnsupported(
                persisted.version,
            ));
        }

        let mut hosts_by_id = BTreeMap::new();
        for host in persisted.hosts {
            let host_id =
                persisted_id!(PanelHostId, host.id, "persisted host id must be non-zero")?;
            let kind = workspace_host_kind(host.kind)?;
            hosts_by_id.insert(host_id, PanelHostNode { id: host_id, kind });
        }

        let mut tab_stacks_by_id = BTreeMap::new();
        for stack in persisted.tab_stacks {
            let stack_id = persisted_id!(
                TabStackId,
                stack.id,
                "persisted tab-stack id must be non-zero"
            )?;
            let ordered_panels = stack
                .ordered_panels
                .into_iter()
                .map(|panel_id| {
                    persisted_id!(
                        PanelInstanceId,
                        panel_id,
                        "persisted ordered panel id must be non-zero"
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let active_panel = stack
                .active_panel
                .map(|panel_id| {
                    persisted_id!(
                        PanelInstanceId,
                        panel_id,
                        "persisted active panel id must be non-zero"
                    )
                })
                .transpose()?;
            tab_stacks_by_id.insert(
                stack_id,
                TabStackState {
                    id: stack_id,
                    ordered_panels,
                    active_panel,
                    locked_tool_surface_kind: stack
                        .locked_tool_surface_kind
                        .map(workspace_tool_surface_kind_v2),
                },
            );
        }

        let mut panels_by_id = BTreeMap::new();
        for panel in persisted.panels {
            let panel_id = persisted_id!(
                PanelInstanceId,
                panel.id,
                "persisted panel id must be non-zero"
            )?;
            let active_tool_surface = panel
                .active_tool_surface
                .map(|surface_id| {
                    persisted_id!(
                        ToolSurfaceInstanceId,
                        surface_id,
                        "persisted active tool-surface id must be non-zero"
                    )
                })
                .transpose()?;
            panels_by_id.insert(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind: workspace_panel_kind_v2(panel.panel_kind),
                    active_tool_surface,
                },
            );
        }

        let mut tool_surfaces_by_id = BTreeMap::new();
        for surface in persisted.tool_surfaces {
            let surface_id = persisted_id!(
                ToolSurfaceInstanceId,
                surface.id,
                "persisted tool-surface id must be non-zero"
            )?;
            tool_surfaces_by_id.insert(
                surface_id,
                ToolSurfaceState {
                    id: surface_id,
                    tool_surface_kind: workspace_tool_surface_kind_v2(surface.tool_surface_kind),
                    mount: workspace_mount(surface.mount)?,
                },
            );
        }

        let state = WorkspaceState {
            workspace_id: persisted_id!(
                WorkspaceId,
                persisted.workspace_id,
                "persisted workspace id must be non-zero"
            )?,
            root_host_id: persisted_id!(
                PanelHostId,
                persisted.root_host_id,
                "persisted root host id must be non-zero"
            )?,
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
                bounds: persisted_floating_bounds(placeholder.bounds),
            }
        }
    }
}

fn workspace_host_kind(
    kind: PersistedPanelHostKindV1,
) -> Result<PanelHostKind, WorkspaceStateError> {
    match kind {
        PersistedPanelHostKindV1::SplitHost {
            axis,
            fraction,
            first_child,
            second_child,
        } => Ok(PanelHostKind::SplitHost(SplitHostState {
            axis: workspace_axis(axis),
            fraction,
            first_child: persisted_id!(
                PanelHostId,
                first_child,
                "persisted split first-child host id must be non-zero"
            )?,
            second_child: persisted_id!(
                PanelHostId,
                second_child,
                "persisted split second-child host id must be non-zero"
            )?,
        })),
        PersistedPanelHostKindV1::TabStackHost { tab_stack_id } => {
            Ok(PanelHostKind::TabStackHost(TabStackHostState {
                tab_stack_id: persisted_id!(
                    TabStackId,
                    tab_stack_id,
                    "persisted tab-stack host id must be non-zero"
                )?,
            }))
        }
        PersistedPanelHostKindV1::FloatingHostPlaceholder {
            tab_stack_id,
            bounds,
        } => Ok(PanelHostKind::FloatingHostPlaceholder(
            FloatingHostPlaceholderState {
                tab_stack_id: tab_stack_id
                    .map(|id| {
                        persisted_id!(
                            TabStackId,
                            id,
                            "persisted floating tab-stack id must be non-zero"
                        )
                    })
                    .transpose()?,
                bounds: workspace_floating_bounds(bounds),
            },
        )),
    }
}

fn persisted_floating_bounds(bounds: FloatingHostBounds) -> PersistedFloatingHostBoundsV1 {
    PersistedFloatingHostBoundsV1 {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
    }
}

fn workspace_floating_bounds(bounds: PersistedFloatingHostBoundsV1) -> FloatingHostBounds {
    FloatingHostBounds::new(bounds.x, bounds.y, bounds.width, bounds.height)
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

impl From<PersistedWorkspaceSplitAxisV1> for String {
    fn from(value: PersistedWorkspaceSplitAxisV1) -> Self {
        match value {
            PersistedWorkspaceSplitAxisV1::Horizontal => "horizontal".to_string(),
            PersistedWorkspaceSplitAxisV1::Vertical => "vertical".to_string(),
        }
    }
}

impl TryFrom<String> for PersistedWorkspaceSplitAxisV1 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "horizontal" => Ok(Self::Horizontal),
            "vertical" => Ok(Self::Vertical),
            other => Err(format!("unsupported workspace split axis: {other}")),
        }
    }
}

fn persisted_panel_kind(kind: PanelKind) -> PersistedPanelKindV1 {
    match kind {
        PanelKind::Outliner => PersistedPanelKindV1::Outliner,
        PanelKind::EntityTable => PersistedPanelKindV1::Placeholder,
        PanelKind::Viewport => PersistedPanelKindV1::Viewport,
        PanelKind::Inspector => PersistedPanelKindV1::Inspector,
        PanelKind::Console => PersistedPanelKindV1::Console,
        PanelKind::EditorDesignOutliner
        | PanelKind::UiHierarchy
        | PanelKind::UiCanvas
        | PanelKind::StyleInspector
        | PanelKind::Bindings
        | PanelKind::DockLayoutPreview
        | PanelKind::ThemeEditor
        | PanelKind::ShortcutEditor
        | PanelKind::MenuEditor
        | PanelKind::DefinitionValidation
        | PanelKind::CommandDiff => PersistedPanelKindV1::Placeholder,
        PanelKind::Placeholder => PersistedPanelKindV1::Placeholder,
    }
}

fn persisted_panel_kind_v2(kind: PanelKind) -> PersistedPanelKindV2 {
    match kind {
        PanelKind::Outliner => PersistedPanelKindV2::Outliner,
        PanelKind::EntityTable => PersistedPanelKindV2::EntityTable,
        PanelKind::Viewport => PersistedPanelKindV2::Viewport,
        PanelKind::Inspector => PersistedPanelKindV2::Inspector,
        PanelKind::Console => PersistedPanelKindV2::Console,
        PanelKind::EditorDesignOutliner => PersistedPanelKindV2::EditorDesignOutliner,
        PanelKind::UiHierarchy => PersistedPanelKindV2::UiHierarchy,
        PanelKind::UiCanvas => PersistedPanelKindV2::UiCanvas,
        PanelKind::StyleInspector => PersistedPanelKindV2::StyleInspector,
        PanelKind::Bindings => PersistedPanelKindV2::Bindings,
        PanelKind::DockLayoutPreview => PersistedPanelKindV2::DockLayoutPreview,
        PanelKind::ThemeEditor => PersistedPanelKindV2::ThemeEditor,
        PanelKind::ShortcutEditor => PersistedPanelKindV2::ShortcutEditor,
        PanelKind::MenuEditor => PersistedPanelKindV2::MenuEditor,
        PanelKind::DefinitionValidation => PersistedPanelKindV2::DefinitionValidation,
        PanelKind::CommandDiff => PersistedPanelKindV2::CommandDiff,
        PanelKind::Placeholder => PersistedPanelKindV2::Placeholder,
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

fn workspace_panel_kind_v2(kind: PersistedPanelKindV2) -> PanelKind {
    match kind {
        PersistedPanelKindV2::Outliner => PanelKind::Outliner,
        PersistedPanelKindV2::EntityTable => PanelKind::EntityTable,
        PersistedPanelKindV2::Viewport => PanelKind::Viewport,
        PersistedPanelKindV2::Inspector => PanelKind::Inspector,
        PersistedPanelKindV2::Console => PanelKind::Console,
        PersistedPanelKindV2::EditorDesignOutliner => PanelKind::EditorDesignOutliner,
        PersistedPanelKindV2::UiHierarchy => PanelKind::UiHierarchy,
        PersistedPanelKindV2::UiCanvas => PanelKind::UiCanvas,
        PersistedPanelKindV2::StyleInspector => PanelKind::StyleInspector,
        PersistedPanelKindV2::Bindings => PanelKind::Bindings,
        PersistedPanelKindV2::DockLayoutPreview => PanelKind::DockLayoutPreview,
        PersistedPanelKindV2::ThemeEditor => PanelKind::ThemeEditor,
        PersistedPanelKindV2::ShortcutEditor => PanelKind::ShortcutEditor,
        PersistedPanelKindV2::MenuEditor => PanelKind::MenuEditor,
        PersistedPanelKindV2::DefinitionValidation => PanelKind::DefinitionValidation,
        PersistedPanelKindV2::CommandDiff => PanelKind::CommandDiff,
        PersistedPanelKindV2::Placeholder => PanelKind::Placeholder,
    }
}

impl From<PersistedPanelKindV1> for String {
    fn from(value: PersistedPanelKindV1) -> Self {
        match value {
            PersistedPanelKindV1::Outliner => "outliner".to_string(),
            PersistedPanelKindV1::Viewport => "viewport".to_string(),
            PersistedPanelKindV1::Inspector => "inspector".to_string(),
            PersistedPanelKindV1::Console => "console".to_string(),
            PersistedPanelKindV1::Placeholder => "placeholder".to_string(),
        }
    }
}

impl TryFrom<String> for PersistedPanelKindV1 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "outliner" => Ok(Self::Outliner),
            "viewport" => Ok(Self::Viewport),
            "inspector" => Ok(Self::Inspector),
            "console" => Ok(Self::Console),
            "placeholder" => Ok(Self::Placeholder),
            other => Err(format!("unsupported panel kind: {other}")),
        }
    }
}

impl From<PersistedPanelKindV2> for String {
    fn from(value: PersistedPanelKindV2) -> Self {
        match value {
            PersistedPanelKindV2::Outliner => "outliner".to_string(),
            PersistedPanelKindV2::EntityTable => "entity_table".to_string(),
            PersistedPanelKindV2::Viewport => "viewport".to_string(),
            PersistedPanelKindV2::Inspector => "inspector".to_string(),
            PersistedPanelKindV2::Console => "console".to_string(),
            PersistedPanelKindV2::EditorDesignOutliner => "editor_design_outliner".to_string(),
            PersistedPanelKindV2::UiHierarchy => "ui_hierarchy".to_string(),
            PersistedPanelKindV2::UiCanvas => "ui_canvas".to_string(),
            PersistedPanelKindV2::StyleInspector => "style_inspector".to_string(),
            PersistedPanelKindV2::Bindings => "bindings".to_string(),
            PersistedPanelKindV2::DockLayoutPreview => "dock_layout_preview".to_string(),
            PersistedPanelKindV2::ThemeEditor => "theme_editor".to_string(),
            PersistedPanelKindV2::ShortcutEditor => "shortcut_editor".to_string(),
            PersistedPanelKindV2::MenuEditor => "menu_editor".to_string(),
            PersistedPanelKindV2::DefinitionValidation => "definition_validation".to_string(),
            PersistedPanelKindV2::CommandDiff => "command_diff".to_string(),
            PersistedPanelKindV2::Placeholder => "placeholder".to_string(),
        }
    }
}

impl TryFrom<String> for PersistedPanelKindV2 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "outliner" => Ok(Self::Outliner),
            "entity_table" => Ok(Self::EntityTable),
            "viewport" => Ok(Self::Viewport),
            "inspector" => Ok(Self::Inspector),
            "console" => Ok(Self::Console),
            "editor_design_outliner" => Ok(Self::EditorDesignOutliner),
            "ui_hierarchy" => Ok(Self::UiHierarchy),
            "ui_canvas" => Ok(Self::UiCanvas),
            "style_inspector" => Ok(Self::StyleInspector),
            "bindings" => Ok(Self::Bindings),
            "dock_layout_preview" => Ok(Self::DockLayoutPreview),
            "theme_editor" => Ok(Self::ThemeEditor),
            "shortcut_editor" => Ok(Self::ShortcutEditor),
            "menu_editor" => Ok(Self::MenuEditor),
            "definition_validation" => Ok(Self::DefinitionValidation),
            "command_diff" => Ok(Self::CommandDiff),
            "placeholder" => Ok(Self::Placeholder),
            other => Err(format!("unsupported panel kind: {other}")),
        }
    }
}

fn persisted_tool_surface_kind(kind: ToolSurfaceKind) -> PersistedToolSurfaceKindV1 {
    match kind {
        ToolSurfaceKind::Outliner => PersistedToolSurfaceKindV1::Outliner,
        ToolSurfaceKind::EntityTable => PersistedToolSurfaceKindV1::Placeholder,
        ToolSurfaceKind::Viewport => PersistedToolSurfaceKindV1::Viewport,
        ToolSurfaceKind::Inspector => PersistedToolSurfaceKindV1::Inspector,
        ToolSurfaceKind::Console => PersistedToolSurfaceKindV1::Console,
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
        | ToolSurfaceKind::CommandDiff => PersistedToolSurfaceKindV1::Placeholder,
        ToolSurfaceKind::Placeholder => PersistedToolSurfaceKindV1::Placeholder,
    }
}

fn persisted_tool_surface_kind_v2(kind: ToolSurfaceKind) -> PersistedToolSurfaceKindV2 {
    match kind {
        ToolSurfaceKind::Outliner => PersistedToolSurfaceKindV2::Outliner,
        ToolSurfaceKind::EntityTable => PersistedToolSurfaceKindV2::EntityTable,
        ToolSurfaceKind::Viewport => PersistedToolSurfaceKindV2::Viewport,
        ToolSurfaceKind::Inspector => PersistedToolSurfaceKindV2::Inspector,
        ToolSurfaceKind::Console => PersistedToolSurfaceKindV2::Console,
        ToolSurfaceKind::EditorDesignOutliner => PersistedToolSurfaceKindV2::EditorDesignOutliner,
        ToolSurfaceKind::UiHierarchy => PersistedToolSurfaceKindV2::UiHierarchy,
        ToolSurfaceKind::UiCanvas => PersistedToolSurfaceKindV2::UiCanvas,
        ToolSurfaceKind::StyleInspector => PersistedToolSurfaceKindV2::StyleInspector,
        ToolSurfaceKind::Bindings => PersistedToolSurfaceKindV2::Bindings,
        ToolSurfaceKind::DockLayoutPreview => PersistedToolSurfaceKindV2::DockLayoutPreview,
        ToolSurfaceKind::ThemeEditor => PersistedToolSurfaceKindV2::ThemeEditor,
        ToolSurfaceKind::ShortcutEditor => PersistedToolSurfaceKindV2::ShortcutEditor,
        ToolSurfaceKind::MenuEditor => PersistedToolSurfaceKindV2::MenuEditor,
        ToolSurfaceKind::DefinitionValidation => PersistedToolSurfaceKindV2::DefinitionValidation,
        ToolSurfaceKind::CommandDiff => PersistedToolSurfaceKindV2::CommandDiff,
        ToolSurfaceKind::Placeholder => PersistedToolSurfaceKindV2::Placeholder,
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

fn workspace_tool_surface_kind_v2(kind: PersistedToolSurfaceKindV2) -> ToolSurfaceKind {
    match kind {
        PersistedToolSurfaceKindV2::Outliner => ToolSurfaceKind::Outliner,
        PersistedToolSurfaceKindV2::EntityTable => ToolSurfaceKind::EntityTable,
        PersistedToolSurfaceKindV2::Viewport => ToolSurfaceKind::Viewport,
        PersistedToolSurfaceKindV2::Inspector => ToolSurfaceKind::Inspector,
        PersistedToolSurfaceKindV2::Console => ToolSurfaceKind::Console,
        PersistedToolSurfaceKindV2::EditorDesignOutliner => ToolSurfaceKind::EditorDesignOutliner,
        PersistedToolSurfaceKindV2::UiHierarchy => ToolSurfaceKind::UiHierarchy,
        PersistedToolSurfaceKindV2::UiCanvas => ToolSurfaceKind::UiCanvas,
        PersistedToolSurfaceKindV2::StyleInspector => ToolSurfaceKind::StyleInspector,
        PersistedToolSurfaceKindV2::Bindings => ToolSurfaceKind::Bindings,
        PersistedToolSurfaceKindV2::DockLayoutPreview => ToolSurfaceKind::DockLayoutPreview,
        PersistedToolSurfaceKindV2::ThemeEditor => ToolSurfaceKind::ThemeEditor,
        PersistedToolSurfaceKindV2::ShortcutEditor => ToolSurfaceKind::ShortcutEditor,
        PersistedToolSurfaceKindV2::MenuEditor => ToolSurfaceKind::MenuEditor,
        PersistedToolSurfaceKindV2::DefinitionValidation => ToolSurfaceKind::DefinitionValidation,
        PersistedToolSurfaceKindV2::CommandDiff => ToolSurfaceKind::CommandDiff,
        PersistedToolSurfaceKindV2::Placeholder => ToolSurfaceKind::Placeholder,
    }
}

impl From<PersistedToolSurfaceKindV1> for String {
    fn from(value: PersistedToolSurfaceKindV1) -> Self {
        match value {
            PersistedToolSurfaceKindV1::Outliner => "outliner".to_string(),
            PersistedToolSurfaceKindV1::Viewport => "viewport".to_string(),
            PersistedToolSurfaceKindV1::Inspector => "inspector".to_string(),
            PersistedToolSurfaceKindV1::Console => "console".to_string(),
            PersistedToolSurfaceKindV1::Placeholder => "placeholder".to_string(),
        }
    }
}

impl TryFrom<String> for PersistedToolSurfaceKindV1 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "outliner" => Ok(Self::Outliner),
            "viewport" => Ok(Self::Viewport),
            "inspector" => Ok(Self::Inspector),
            "console" => Ok(Self::Console),
            "placeholder" => Ok(Self::Placeholder),
            other => Err(format!("unsupported tool-surface kind: {other}")),
        }
    }
}

impl From<PersistedToolSurfaceKindV2> for String {
    fn from(value: PersistedToolSurfaceKindV2) -> Self {
        match value {
            PersistedToolSurfaceKindV2::Outliner => "outliner".to_string(),
            PersistedToolSurfaceKindV2::EntityTable => "entity_table".to_string(),
            PersistedToolSurfaceKindV2::Viewport => "viewport".to_string(),
            PersistedToolSurfaceKindV2::Inspector => "inspector".to_string(),
            PersistedToolSurfaceKindV2::Console => "console".to_string(),
            PersistedToolSurfaceKindV2::EditorDesignOutliner => {
                "editor_design_outliner".to_string()
            }
            PersistedToolSurfaceKindV2::UiHierarchy => "ui_hierarchy".to_string(),
            PersistedToolSurfaceKindV2::UiCanvas => "ui_canvas".to_string(),
            PersistedToolSurfaceKindV2::StyleInspector => "style_inspector".to_string(),
            PersistedToolSurfaceKindV2::Bindings => "bindings".to_string(),
            PersistedToolSurfaceKindV2::DockLayoutPreview => "dock_layout_preview".to_string(),
            PersistedToolSurfaceKindV2::ThemeEditor => "theme_editor".to_string(),
            PersistedToolSurfaceKindV2::ShortcutEditor => "shortcut_editor".to_string(),
            PersistedToolSurfaceKindV2::MenuEditor => "menu_editor".to_string(),
            PersistedToolSurfaceKindV2::DefinitionValidation => "definition_validation".to_string(),
            PersistedToolSurfaceKindV2::CommandDiff => "command_diff".to_string(),
            PersistedToolSurfaceKindV2::Placeholder => "placeholder".to_string(),
        }
    }
}

impl TryFrom<String> for PersistedToolSurfaceKindV2 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "outliner" => Ok(Self::Outliner),
            "entity_table" => Ok(Self::EntityTable),
            "viewport" => Ok(Self::Viewport),
            "inspector" => Ok(Self::Inspector),
            "console" => Ok(Self::Console),
            "editor_design_outliner" => Ok(Self::EditorDesignOutliner),
            "ui_hierarchy" => Ok(Self::UiHierarchy),
            "ui_canvas" => Ok(Self::UiCanvas),
            "style_inspector" => Ok(Self::StyleInspector),
            "bindings" => Ok(Self::Bindings),
            "dock_layout_preview" => Ok(Self::DockLayoutPreview),
            "theme_editor" => Ok(Self::ThemeEditor),
            "shortcut_editor" => Ok(Self::ShortcutEditor),
            "menu_editor" => Ok(Self::MenuEditor),
            "definition_validation" => Ok(Self::DefinitionValidation),
            "command_diff" => Ok(Self::CommandDiff),
            "placeholder" => Ok(Self::Placeholder),
            other => Err(format!("unsupported tool-surface kind: {other}")),
        }
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

fn workspace_mount(
    mount: PersistedToolSurfaceMountV1,
) -> Result<ToolSurfaceMount, WorkspaceStateError> {
    match mount {
        PersistedToolSurfaceMountV1::Unmounted => Ok(ToolSurfaceMount::Unmounted),
        PersistedToolSurfaceMountV1::Mounted { panel_id } => Ok(ToolSurfaceMount::Mounted {
            panel_id: persisted_id!(
                PanelInstanceId,
                panel_id,
                "persisted mounted panel id must be non-zero"
            )?,
        }),
    }
}

fn default_floating_host_bounds_v1() -> PersistedFloatingHostBoundsV1 {
    PersistedFloatingHostBoundsV1 {
        x: 96.0,
        y: 96.0,
        width: 560.0,
        height: 360.0,
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
        let persisted = workspace.to_persisted_v2();
        let restored =
            WorkspaceState::from_persisted_v2(persisted).expect("roundtrip should decode");
        assert_eq!(workspace, restored);
    }

    #[test]
    fn persisted_decode_rejects_invalid_references() {
        let workspace = bootstrap_workspace();
        let mut persisted = workspace.to_persisted_v2();
        persisted.panels.clear();
        let error = WorkspaceState::from_persisted_v2(persisted)
            .expect_err("invalid references must fail decode");
        assert!(matches!(error, WorkspaceStateError::MissingPanel(_)));
    }

    #[test]
    fn persisted_decode_rejects_unsupported_version() {
        let workspace = bootstrap_workspace();
        let mut persisted = workspace.to_persisted_v2();
        persisted.version = 99;
        let error = WorkspaceState::from_persisted_v2(persisted)
            .expect_err("unsupported versions must fail");
        assert!(matches!(
            error,
            WorkspaceStateError::PersistedVersionUnsupported(99)
        ));
    }

    #[test]
    fn persisted_decode_rejects_zero_raw_ids() {
        let workspace = bootstrap_workspace();
        let mut persisted = workspace.to_persisted_v2();
        persisted.workspace_id = 0;
        let error = WorkspaceState::from_persisted_v2(persisted)
            .expect_err("zero persisted ids must fail decode");
        assert!(matches!(
            error,
            WorkspaceStateError::PersistedSchemaViolation(_)
        ));
    }

    #[test]
    fn persisted_v2_roundtrip_handles_detached_tool_surface_state() {
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

        let persisted = detached.to_persisted_v2();
        let restored =
            WorkspaceState::from_persisted_v2(persisted).expect("detached state should decode");
        assert_eq!(detached, restored);
    }

    #[test]
    fn persisted_v2_roundtrip_preserves_area_type_locks() {
        let workspace = bootstrap_workspace();
        let viewport_stack = workspace
            .tab_stacks_by_id
            .values()
            .find(|stack| {
                stack.ordered_panels.iter().any(|panel| {
                    workspace.panel(*panel).map(|value| value.panel_kind)
                        == Some(PanelKind::Viewport)
                })
            })
            .expect("viewport stack should exist")
            .id;
        let locked = reduce_workspace(
            &workspace,
            WorkspaceMutation::LockTabStackAreaType {
                tab_stack_id: viewport_stack,
                locked_tool_surface_kind: Some(ToolSurfaceKind::Viewport),
            },
        )
        .expect("locking should produce valid state");

        let persisted = locked.to_persisted_v2();
        assert_eq!(
            persisted
                .tab_stacks
                .iter()
                .find(|stack| stack.id == viewport_stack.raw())
                .and_then(|stack| stack.locked_tool_surface_kind),
            Some(PersistedToolSurfaceKindV2::Viewport)
        );
        let restored =
            WorkspaceState::from_persisted_v2(persisted).expect("locked state should decode");
        assert_eq!(locked, restored);
    }

    #[test]
    fn persisted_v1_decode_remains_supported_for_legacy_layouts() {
        let workspace = bootstrap_workspace();
        let persisted = workspace.to_persisted_v1();
        let restored = WorkspaceState::from_persisted_v1(persisted)
            .expect("legacy v1 layout should still decode");

        assert!(restored.validate_integrity().is_ok());
        assert!(
            restored
                .tab_stacks()
                .all(|stack| stack.locked_tool_surface_kind.is_none())
        );
    }
}
