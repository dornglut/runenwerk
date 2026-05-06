//! File: domain/editor/editor_shell/src/workspace/state.rs
//! Purpose: Canonical workspace structural graph value model.
//!
//! Identity invariants:
//! - `WorkspaceId` identifies the workspace structural root only.
//! - `PanelHostId` identifies container/layout nodes only.
//! - `TabStackId` identifies tab containers only.
//! - `PanelInstanceId` identifies panel structure instances only.
//! - `ToolSurfaceInstanceId` identifies tool-surface content instances only.
//! - runtime `editor_viewport::ViewportId` is never a workspace structural id.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    PanelHostId, PanelInstanceId, TabStackId, ToolSurfaceInstanceId, WorkspaceId,
    WorkspaceIdentityAllocator, WorkspaceIdentitySeed,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitHostState {
    pub axis: WorkspaceSplitAxis,
    pub fraction: f32,
    pub first_child: PanelHostId,
    pub second_child: PanelHostId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabStackHostState {
    pub tab_stack_id: TabStackId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingHostPlaceholderState {
    pub tab_stack_id: Option<TabStackId>,
    pub bounds: FloatingHostBounds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingHostBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl FloatingHostBounds {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn is_valid(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelHostKind {
    SplitHost(SplitHostState),
    TabStackHost(TabStackHostState),
    FloatingHostPlaceholder(FloatingHostPlaceholderState),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelHostNode {
    pub id: PanelHostId,
    pub kind: PanelHostKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    Outliner,
    EntityTable,
    Viewport,
    Inspector,
    Console,
    Placeholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSurfaceKind {
    Outliner,
    EntityTable,
    Viewport,
    Inspector,
    Console,
    Placeholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSurfaceMount {
    Unmounted,
    Mounted { panel_id: PanelInstanceId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabStackState {
    pub id: TabStackId,
    pub ordered_panels: Vec<PanelInstanceId>,
    pub active_panel: Option<PanelInstanceId>,
    pub locked_tool_surface_kind: Option<ToolSurfaceKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelInstanceState {
    pub id: PanelInstanceId,
    pub panel_kind: PanelKind,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolSurfaceState {
    pub id: ToolSurfaceInstanceId,
    pub tool_surface_kind: ToolSurfaceKind,
    pub mount: ToolSurfaceMount,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceState {
    pub(crate) workspace_id: WorkspaceId,
    pub(crate) root_host_id: PanelHostId,
    pub(crate) hosts_by_id: BTreeMap<PanelHostId, PanelHostNode>,
    pub(crate) tab_stacks_by_id: BTreeMap<TabStackId, TabStackState>,
    pub(crate) panels_by_id: BTreeMap<PanelInstanceId, PanelInstanceState>,
    pub(crate) tool_surfaces_by_id: BTreeMap<ToolSurfaceInstanceId, ToolSurfaceState>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceStateError {
    MissingRootHost(PanelHostId),
    MissingHost(PanelHostId),
    MissingTabStack(TabStackId),
    MissingPanel(PanelInstanceId),
    MissingToolSurface(ToolSurfaceInstanceId),
    DuplicateHostId(PanelHostId),
    DuplicateTabStackId(TabStackId),
    DuplicatePanelId(PanelInstanceId),
    DuplicateToolSurfaceId(ToolSurfaceInstanceId),
    DuplicatePanelInTabStacks(PanelInstanceId),
    DuplicateTabStackHost(TabStackId),
    PanelNotInTabStack {
        tab_stack_id: TabStackId,
        panel_id: PanelInstanceId,
    },
    ActivePanelNotInStack {
        tab_stack_id: TabStackId,
        panel_id: PanelInstanceId,
    },
    InvalidSplitFraction {
        host_id: PanelHostId,
        fraction: f32,
    },
    InvalidFloatingHostBounds {
        host_id: PanelHostId,
        bounds: FloatingHostBounds,
    },
    PanelSurfaceMismatch {
        panel_id: PanelInstanceId,
        tool_surface_id: ToolSurfaceInstanceId,
    },
    PanelAlreadyHasToolSurface {
        panel_id: PanelInstanceId,
        tool_surface_id: ToolSurfaceInstanceId,
    },
    MountedSurfacePanelMismatch {
        tool_surface_id: ToolSurfaceInstanceId,
        panel_id: PanelInstanceId,
    },
    ToolSurfaceAlreadyMounted {
        tool_surface_id: ToolSurfaceInstanceId,
        panel_id: PanelInstanceId,
    },
    PanelHasMultipleMountedSurfaces(PanelInstanceId),
    ProjectionShapeMismatch(&'static str),
    PersistedVersionUnsupported(u32),
    PersistedSchemaViolation(&'static str),
}

impl std::fmt::Display for WorkspaceStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRootHost(host_id) => write!(f, "missing root host: {host_id:?}"),
            Self::MissingHost(host_id) => write!(f, "missing host: {host_id:?}"),
            Self::MissingTabStack(tab_stack_id) => {
                write!(f, "missing tab stack: {tab_stack_id:?}")
            }
            Self::MissingPanel(panel_id) => write!(f, "missing panel: {panel_id:?}"),
            Self::MissingToolSurface(tool_surface_id) => {
                write!(f, "missing tool surface: {tool_surface_id:?}")
            }
            Self::DuplicateHostId(host_id) => write!(f, "duplicate host id: {host_id:?}"),
            Self::DuplicateTabStackId(tab_stack_id) => {
                write!(f, "duplicate tab stack id: {tab_stack_id:?}")
            }
            Self::DuplicatePanelId(panel_id) => {
                write!(f, "duplicate panel id: {panel_id:?}")
            }
            Self::DuplicateToolSurfaceId(tool_surface_id) => {
                write!(f, "duplicate tool surface id: {tool_surface_id:?}")
            }
            Self::DuplicatePanelInTabStacks(panel_id) => {
                write!(f, "panel appears in multiple tab stacks: {panel_id:?}")
            }
            Self::DuplicateTabStackHost(tab_stack_id) => {
                write!(f, "tab stack appears in multiple hosts: {tab_stack_id:?}")
            }
            Self::PanelNotInTabStack {
                tab_stack_id,
                panel_id,
            } => write!(
                f,
                "panel {panel_id:?} is not present in tab stack {tab_stack_id:?}",
            ),
            Self::ActivePanelNotInStack {
                tab_stack_id,
                panel_id,
            } => write!(
                f,
                "active panel {panel_id:?} not present in tab stack {tab_stack_id:?}",
            ),
            Self::InvalidSplitFraction { host_id, fraction } => {
                write!(f, "split host {host_id:?} has invalid fraction {fraction}",)
            }
            Self::InvalidFloatingHostBounds { host_id, bounds } => write!(
                f,
                "floating host {host_id:?} has invalid bounds ({:.1},{:.1},{:.1},{:.1})",
                bounds.x, bounds.y, bounds.width, bounds.height
            ),
            Self::PanelSurfaceMismatch {
                panel_id,
                tool_surface_id,
            } => write!(
                f,
                "panel {panel_id:?} points to tool surface {tool_surface_id:?} with mismatched mount",
            ),
            Self::PanelAlreadyHasToolSurface {
                panel_id,
                tool_surface_id,
            } => write!(
                f,
                "panel {panel_id:?} already has active tool surface {tool_surface_id:?}",
            ),
            Self::MountedSurfacePanelMismatch {
                tool_surface_id,
                panel_id,
            } => write!(
                f,
                "tool surface {tool_surface_id:?} is mounted to panel {panel_id:?} but panel link mismatches",
            ),
            Self::ToolSurfaceAlreadyMounted {
                tool_surface_id,
                panel_id,
            } => write!(
                f,
                "tool surface {tool_surface_id:?} is already mounted to panel {panel_id:?}",
            ),
            Self::PanelHasMultipleMountedSurfaces(panel_id) => {
                write!(f, "panel has multiple mounted tool surfaces: {panel_id:?}")
            }
            Self::ProjectionShapeMismatch(message) => {
                write!(f, "projection shape mismatch: {message}")
            }
            Self::PersistedVersionUnsupported(version) => {
                write!(f, "persisted workspace version {version} is unsupported")
            }
            Self::PersistedSchemaViolation(message) => {
                write!(f, "persisted workspace schema violation: {message}")
            }
        }
    }
}

impl std::error::Error for WorkspaceStateError {}

impl WorkspaceState {
    /// Transitional seed for the current fixed scene-authoring layout only.
    /// This function is not the universal workspace-construction doctrine.
    pub fn bootstrap_current_layout(
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
    ) -> Self {
        let body_console_split_host = allocator.allocate_panel_host_id();
        let left_right_split_host = allocator.allocate_panel_host_id();
        let center_right_split_host = allocator.allocate_panel_host_id();
        let outliner_tab_host = allocator.allocate_panel_host_id();
        let viewport_tab_host = allocator.allocate_panel_host_id();
        let inspector_tab_host = allocator.allocate_panel_host_id();
        let console_tab_host = allocator.allocate_panel_host_id();

        let outliner_tab_stack = allocator.allocate_tab_stack_id();
        let viewport_tab_stack = allocator.allocate_tab_stack_id();
        let inspector_tab_stack = allocator.allocate_tab_stack_id();
        let console_tab_stack = allocator.allocate_tab_stack_id();

        let outliner_panel = allocator.allocate_panel_instance_id();
        let entity_table_panel = allocator.allocate_panel_instance_id();
        let viewport_panel = allocator.allocate_panel_instance_id();
        let inspector_panel = allocator.allocate_panel_instance_id();
        let console_panel = allocator.allocate_panel_instance_id();

        let outliner_surface = allocator.allocate_tool_surface_instance_id();
        let entity_table_surface = allocator.allocate_tool_surface_instance_id();
        let viewport_surface = allocator.allocate_tool_surface_instance_id();
        let inspector_surface = allocator.allocate_tool_surface_instance_id();
        let console_surface = allocator.allocate_tool_surface_instance_id();

        let mut hosts_by_id = BTreeMap::new();
        hosts_by_id.insert(
            body_console_split_host,
            PanelHostNode {
                id: body_console_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Vertical,
                    fraction: 0.78,
                    first_child: left_right_split_host,
                    second_child: console_tab_host,
                }),
            },
        );
        hosts_by_id.insert(
            left_right_split_host,
            PanelHostNode {
                id: left_right_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Horizontal,
                    fraction: 0.72,
                    first_child: viewport_tab_host,
                    second_child: center_right_split_host,
                }),
            },
        );
        hosts_by_id.insert(
            center_right_split_host,
            PanelHostNode {
                id: center_right_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Vertical,
                    fraction: 0.56,
                    first_child: outliner_tab_host,
                    second_child: inspector_tab_host,
                }),
            },
        );
        hosts_by_id.insert(
            outliner_tab_host,
            PanelHostNode {
                id: outliner_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: outliner_tab_stack,
                }),
            },
        );
        hosts_by_id.insert(
            viewport_tab_host,
            PanelHostNode {
                id: viewport_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: viewport_tab_stack,
                }),
            },
        );
        hosts_by_id.insert(
            inspector_tab_host,
            PanelHostNode {
                id: inspector_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: inspector_tab_stack,
                }),
            },
        );
        hosts_by_id.insert(
            console_tab_host,
            PanelHostNode {
                id: console_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: console_tab_stack,
                }),
            },
        );

        let mut tab_stacks_by_id = BTreeMap::new();
        tab_stacks_by_id.insert(
            outliner_tab_stack,
            TabStackState {
                id: outliner_tab_stack,
                ordered_panels: vec![outliner_panel, entity_table_panel],
                active_panel: Some(outliner_panel),
                locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            viewport_tab_stack,
            TabStackState {
                id: viewport_tab_stack,
                ordered_panels: vec![viewport_panel],
                active_panel: Some(viewport_panel),
                locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            inspector_tab_stack,
            TabStackState {
                id: inspector_tab_stack,
                ordered_panels: vec![inspector_panel],
                active_panel: Some(inspector_panel),
                locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            console_tab_stack,
            TabStackState {
                id: console_tab_stack,
                ordered_panels: vec![console_panel],
                active_panel: Some(console_panel),
                locked_tool_surface_kind: None,
            },
        );

        let mut panels_by_id = BTreeMap::new();
        panels_by_id.insert(
            outliner_panel,
            PanelInstanceState {
                id: outliner_panel,
                panel_kind: PanelKind::Outliner,
                active_tool_surface: Some(outliner_surface),
            },
        );
        panels_by_id.insert(
            entity_table_panel,
            PanelInstanceState {
                id: entity_table_panel,
                panel_kind: PanelKind::EntityTable,
                active_tool_surface: Some(entity_table_surface),
            },
        );
        panels_by_id.insert(
            viewport_panel,
            PanelInstanceState {
                id: viewport_panel,
                panel_kind: PanelKind::Viewport,
                active_tool_surface: Some(viewport_surface),
            },
        );
        panels_by_id.insert(
            inspector_panel,
            PanelInstanceState {
                id: inspector_panel,
                panel_kind: PanelKind::Inspector,
                active_tool_surface: Some(inspector_surface),
            },
        );
        panels_by_id.insert(
            console_panel,
            PanelInstanceState {
                id: console_panel,
                panel_kind: PanelKind::Console,
                active_tool_surface: Some(console_surface),
            },
        );

        let mut tool_surfaces_by_id = BTreeMap::new();
        tool_surfaces_by_id.insert(
            outliner_surface,
            ToolSurfaceState {
                id: outliner_surface,
                tool_surface_kind: ToolSurfaceKind::Outliner,
                mount: ToolSurfaceMount::Mounted {
                    panel_id: outliner_panel,
                },
            },
        );
        tool_surfaces_by_id.insert(
            entity_table_surface,
            ToolSurfaceState {
                id: entity_table_surface,
                tool_surface_kind: ToolSurfaceKind::EntityTable,
                mount: ToolSurfaceMount::Mounted {
                    panel_id: entity_table_panel,
                },
            },
        );
        tool_surfaces_by_id.insert(
            viewport_surface,
            ToolSurfaceState {
                id: viewport_surface,
                tool_surface_kind: ToolSurfaceKind::Viewport,
                mount: ToolSurfaceMount::Mounted {
                    panel_id: viewport_panel,
                },
            },
        );
        tool_surfaces_by_id.insert(
            inspector_surface,
            ToolSurfaceState {
                id: inspector_surface,
                tool_surface_kind: ToolSurfaceKind::Inspector,
                mount: ToolSurfaceMount::Mounted {
                    panel_id: inspector_panel,
                },
            },
        );
        tool_surfaces_by_id.insert(
            console_surface,
            ToolSurfaceState {
                id: console_surface,
                tool_surface_kind: ToolSurfaceKind::Console,
                mount: ToolSurfaceMount::Mounted {
                    panel_id: console_panel,
                },
            },
        );

        Self {
            workspace_id,
            root_host_id: body_console_split_host,
            hosts_by_id,
            tab_stacks_by_id,
            panels_by_id,
            tool_surfaces_by_id,
        }
    }

    pub fn bootstrap_modelling_layout(
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
    ) -> Self {
        Self::bootstrap_current_layout(workspace_id, allocator)
    }

    pub fn workspace_id(&self) -> WorkspaceId {
        self.workspace_id
    }

    pub fn root_host_id(&self) -> PanelHostId {
        self.root_host_id
    }

    pub fn host(&self, host_id: PanelHostId) -> Option<&PanelHostNode> {
        self.hosts_by_id.get(&host_id)
    }

    pub fn hosts(&self) -> impl Iterator<Item = &PanelHostNode> {
        self.hosts_by_id.values()
    }

    pub fn tab_stack(&self, tab_stack_id: TabStackId) -> Option<&TabStackState> {
        self.tab_stacks_by_id.get(&tab_stack_id)
    }

    pub fn tab_stacks(&self) -> impl Iterator<Item = &TabStackState> {
        self.tab_stacks_by_id.values()
    }

    pub fn panel(&self, panel_id: PanelInstanceId) -> Option<&PanelInstanceState> {
        self.panels_by_id.get(&panel_id)
    }

    pub fn panels(&self) -> impl Iterator<Item = &PanelInstanceState> {
        self.panels_by_id.values()
    }

    pub fn tool_surface(
        &self,
        tool_surface_id: ToolSurfaceInstanceId,
    ) -> Option<&ToolSurfaceState> {
        self.tool_surfaces_by_id.get(&tool_surface_id)
    }

    pub fn tool_surfaces(&self) -> impl Iterator<Item = &ToolSurfaceState> {
        self.tool_surfaces_by_id.values()
    }

    pub fn next_identity_seed(&self) -> WorkspaceIdentitySeed {
        WorkspaceIdentitySeed {
            next_workspace_id: self.workspace_id.raw().saturating_add(1).max(1),
            next_panel_host_id: self
                .hosts_by_id
                .keys()
                .map(|id| id.raw())
                .max()
                .unwrap_or(0)
                .saturating_add(1)
                .max(1),
            next_panel_instance_id: self
                .panels_by_id
                .keys()
                .map(|id| id.raw())
                .max()
                .unwrap_or(0)
                .saturating_add(1)
                .max(1),
            next_tool_surface_instance_id: self
                .tool_surfaces_by_id
                .keys()
                .map(|id| id.raw())
                .max()
                .unwrap_or(0)
                .saturating_add(1)
                .max(1),
            next_tab_stack_id: self
                .tab_stacks_by_id
                .keys()
                .map(|id| id.raw())
                .max()
                .unwrap_or(0)
                .saturating_add(1)
                .max(1),
        }
    }

    pub fn validate_integrity(&self) -> Result<(), WorkspaceStateError> {
        if !self.hosts_by_id.contains_key(&self.root_host_id) {
            return Err(WorkspaceStateError::MissingRootHost(self.root_host_id));
        }

        let mut tab_stacks_seen_in_hosts = BTreeSet::new();
        for host in self.hosts_by_id.values() {
            match host.kind {
                PanelHostKind::SplitHost(split) => {
                    if !(split.fraction > 0.0 && split.fraction < 1.0 && split.fraction.is_finite())
                    {
                        return Err(WorkspaceStateError::InvalidSplitFraction {
                            host_id: host.id,
                            fraction: split.fraction,
                        });
                    }
                    if !self.hosts_by_id.contains_key(&split.first_child) {
                        return Err(WorkspaceStateError::MissingHost(split.first_child));
                    }
                    if !self.hosts_by_id.contains_key(&split.second_child) {
                        return Err(WorkspaceStateError::MissingHost(split.second_child));
                    }
                }
                PanelHostKind::TabStackHost(tab_host) => {
                    if !self.tab_stacks_by_id.contains_key(&tab_host.tab_stack_id) {
                        return Err(WorkspaceStateError::MissingTabStack(tab_host.tab_stack_id));
                    }
                    if !tab_stacks_seen_in_hosts.insert(tab_host.tab_stack_id) {
                        return Err(WorkspaceStateError::DuplicateTabStackHost(
                            tab_host.tab_stack_id,
                        ));
                    }
                }
                PanelHostKind::FloatingHostPlaceholder(placeholder) => {
                    if !placeholder.bounds.is_valid() {
                        return Err(WorkspaceStateError::InvalidFloatingHostBounds {
                            host_id: host.id,
                            bounds: placeholder.bounds,
                        });
                    }
                    if let Some(tab_stack_id) = placeholder.tab_stack_id
                        && !self.tab_stacks_by_id.contains_key(&tab_stack_id)
                    {
                        return Err(WorkspaceStateError::MissingTabStack(tab_stack_id));
                    }
                    if let Some(tab_stack_id) = placeholder.tab_stack_id
                        && !tab_stacks_seen_in_hosts.insert(tab_stack_id)
                    {
                        return Err(WorkspaceStateError::DuplicateTabStackHost(tab_stack_id));
                    }
                }
            }
        }

        let mut panels_seen_in_stacks = BTreeSet::new();
        for tab_stack in self.tab_stacks_by_id.values() {
            let mut local_seen = BTreeSet::new();
            for panel_id in &tab_stack.ordered_panels {
                if !self.panels_by_id.contains_key(panel_id) {
                    return Err(WorkspaceStateError::MissingPanel(*panel_id));
                }
                if !local_seen.insert(*panel_id) {
                    return Err(WorkspaceStateError::DuplicatePanelInTabStacks(*panel_id));
                }
                if !panels_seen_in_stacks.insert(*panel_id) {
                    return Err(WorkspaceStateError::DuplicatePanelInTabStacks(*panel_id));
                }
            }
            if let Some(active_panel) = tab_stack.active_panel
                && !tab_stack.ordered_panels.contains(&active_panel)
            {
                return Err(WorkspaceStateError::ActivePanelNotInStack {
                    tab_stack_id: tab_stack.id,
                    panel_id: active_panel,
                });
            }
        }

        let mut mounted_surface_by_panel =
            BTreeMap::<PanelInstanceId, ToolSurfaceInstanceId>::new();

        for panel in self.panels_by_id.values() {
            if let Some(tool_surface_id) = panel.active_tool_surface
                && !self.tool_surfaces_by_id.contains_key(&tool_surface_id)
            {
                return Err(WorkspaceStateError::MissingToolSurface(tool_surface_id));
            }
        }

        for tool_surface in self.tool_surfaces_by_id.values() {
            if let ToolSurfaceMount::Mounted { panel_id } = tool_surface.mount {
                if !self.panels_by_id.contains_key(&panel_id) {
                    return Err(WorkspaceStateError::MissingPanel(panel_id));
                }
                if mounted_surface_by_panel
                    .insert(panel_id, tool_surface.id)
                    .is_some()
                {
                    return Err(WorkspaceStateError::PanelHasMultipleMountedSurfaces(
                        panel_id,
                    ));
                }
            }
        }

        for panel in self.panels_by_id.values() {
            if let Some(tool_surface_id) = panel.active_tool_surface {
                let tool_surface = self
                    .tool_surfaces_by_id
                    .get(&tool_surface_id)
                    .ok_or(WorkspaceStateError::MissingToolSurface(tool_surface_id))?;
                match tool_surface.mount {
                    ToolSurfaceMount::Mounted { panel_id } if panel_id == panel.id => {}
                    _ => {
                        return Err(WorkspaceStateError::PanelSurfaceMismatch {
                            panel_id: panel.id,
                            tool_surface_id,
                        });
                    }
                }
            }
        }

        for tool_surface in self.tool_surfaces_by_id.values() {
            if let ToolSurfaceMount::Mounted { panel_id } = tool_surface.mount {
                let panel = self
                    .panels_by_id
                    .get(&panel_id)
                    .ok_or(WorkspaceStateError::MissingPanel(panel_id))?;
                if panel.active_tool_surface != Some(tool_surface.id) {
                    return Err(WorkspaceStateError::MountedSurfacePanelMismatch {
                        tool_surface_id: tool_surface.id,
                        panel_id,
                    });
                }
            }
        }

        Ok(())
    }
}
