//! File: domain/editor/editor_shell/src/workspace/state.rs
//! Purpose: Canonical workspace structural graph value model.
//!
//! Identity invariants:
//! - `WorkspaceId` identifies the workspace structural root only.
//! - `PanelHostId` identifies container/layout nodes only.
//! - `TabStackId` identifies tab containers only.
//! - `PanelInstanceId` identifies panel structure instances only.
//! - `ToolSurfaceInstanceId` identifies tool-surface content instances only.
//! - runtime `editor_viewport::ViewportId` can be retained for viewport restore metadata, but
//!   it is never a workspace structural id.

use std::collections::{BTreeMap, BTreeSet};

use editor_viewport::{ViewportId, ViewportRuntimeSettings};

use crate::{
    PanelHostId, PanelInstanceId, TabStackId, ToolSurfaceInstanceId, WorkspaceId,
    WorkspaceIdentityAllocator, WorkspaceIdentitySeed,
    tool_suite::{ToolSurfaceRegistry, ToolSurfaceStableKey, stable_key_for_tool_surface_kind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSplitSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl DockSplitSide {
    pub fn axis(self) -> WorkspaceSplitAxis {
        match self {
            Self::Left | Self::Right => WorkspaceSplitAxis::Horizontal,
            Self::Top | Self::Bottom => WorkspaceSplitAxis::Vertical,
        }
    }

    pub fn target_is_first_child(self) -> bool {
        match self {
            Self::Left | Self::Top => false,
            Self::Right | Self::Bottom => true,
        }
    }
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

/// Structural shell/layout grouping for a panel instance.
///
/// `PanelKind` is retained after C6A as chrome and layout grouping metadata:
/// it describes where a panel belongs and which shell affordances it uses. It
/// is not tool-surface identity and must not be used to infer provider or tool
/// semantics. `ToolSurfaceStableKey` remains the authoritative tool-surface
/// identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PanelKind {
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
    AssetBrowser,
    ImportInspector,
    FieldProductViewer,
    SdfBrushBrowser,
    GraphCanvas,
    Diagnostics,
    RuntimeDebug,
    FieldLayerStack,
    SdfGraphCanvas,
    MaterialGraphCanvas,
    MaterialInspector,
    MaterialPreview,
    TextureViewer,
    VolumeTextureViewer,
    ProcgenGraphCanvas,
    ProcgenPreview,
    GameplayGraphCanvas,
    GameplayCompilerDiagnostics,
    ParticleGraphCanvas,
    ParticlePreview,
    PhysicsAuthoring,
    PhysicsDebug,
    Timeline,
    CurveEditor,
    AnimationGraphCanvas,
    SimulationPreview,
    SimulationDiagnostics,
    Placeholder,
}

/// Legacy boundary enum for pre-registry tool-surface compatibility.
///
/// `ToolSurfaceKind` is not live tool-surface identity after Option C. New
/// normal APIs should carry `ToolSurfaceStableKey`; this enum remains only for
/// V1-V4 persistence migration, V5 legacy metadata validation, authored legacy
/// keys, named legacy wrappers, shell/app command compatibility pending final
/// cleanup, and compatibility tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolSurfaceKind {
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
    AssetBrowser,
    ImportInspector,
    FieldProductViewer,
    SdfBrushBrowser,
    GraphCanvas,
    Diagnostics,
    RuntimeDebug,
    FieldLayerStack,
    SdfGraphCanvas,
    MaterialGraphCanvas,
    MaterialInspector,
    MaterialPreview,
    TextureViewer,
    VolumeTextureViewer,
    ProcgenGraphCanvas,
    ProcgenPreview,
    GameplayGraphCanvas,
    GameplayCompilerDiagnostics,
    ParticleGraphCanvas,
    ParticlePreview,
    PhysicsAuthoring,
    PhysicsDebug,
    Timeline,
    CurveEditor,
    AnimationGraphCanvas,
    SimulationPreview,
    SimulationDiagnostics,
    Placeholder,
}

impl ToolSurfaceKind {
    pub const fn panel_kind(self) -> PanelKind {
        match self {
            Self::Outliner => PanelKind::Outliner,
            Self::EntityTable => PanelKind::EntityTable,
            Self::Viewport => PanelKind::Viewport,
            Self::Inspector => PanelKind::Inspector,
            Self::Console => PanelKind::Console,
            Self::EditorDesignOutliner => PanelKind::EditorDesignOutliner,
            Self::UiHierarchy => PanelKind::UiHierarchy,
            Self::UiCanvas => PanelKind::UiCanvas,
            Self::StyleInspector => PanelKind::StyleInspector,
            Self::Bindings => PanelKind::Bindings,
            Self::DockLayoutPreview => PanelKind::DockLayoutPreview,
            Self::ThemeEditor => PanelKind::ThemeEditor,
            Self::ShortcutEditor => PanelKind::ShortcutEditor,
            Self::MenuEditor => PanelKind::MenuEditor,
            Self::DefinitionValidation => PanelKind::DefinitionValidation,
            Self::CommandDiff => PanelKind::CommandDiff,
            Self::AssetBrowser => PanelKind::AssetBrowser,
            Self::ImportInspector => PanelKind::ImportInspector,
            Self::FieldProductViewer => PanelKind::FieldProductViewer,
            Self::SdfBrushBrowser => PanelKind::SdfBrushBrowser,
            Self::GraphCanvas => PanelKind::GraphCanvas,
            Self::Diagnostics => PanelKind::Diagnostics,
            Self::RuntimeDebug => PanelKind::RuntimeDebug,
            Self::FieldLayerStack => PanelKind::FieldLayerStack,
            Self::SdfGraphCanvas => PanelKind::SdfGraphCanvas,
            Self::MaterialGraphCanvas => PanelKind::MaterialGraphCanvas,
            Self::MaterialInspector => PanelKind::MaterialInspector,
            Self::MaterialPreview => PanelKind::MaterialPreview,
            Self::TextureViewer => PanelKind::TextureViewer,
            Self::VolumeTextureViewer => PanelKind::VolumeTextureViewer,
            Self::ProcgenGraphCanvas => PanelKind::ProcgenGraphCanvas,
            Self::ProcgenPreview => PanelKind::ProcgenPreview,
            Self::GameplayGraphCanvas => PanelKind::GameplayGraphCanvas,
            Self::GameplayCompilerDiagnostics => PanelKind::GameplayCompilerDiagnostics,
            Self::ParticleGraphCanvas => PanelKind::ParticleGraphCanvas,
            Self::ParticlePreview => PanelKind::ParticlePreview,
            Self::PhysicsAuthoring => PanelKind::PhysicsAuthoring,
            Self::PhysicsDebug => PanelKind::PhysicsDebug,
            Self::Timeline => PanelKind::Timeline,
            Self::CurveEditor => PanelKind::CurveEditor,
            Self::AnimationGraphCanvas => PanelKind::AnimationGraphCanvas,
            Self::SimulationPreview => PanelKind::SimulationPreview,
            Self::SimulationDiagnostics => PanelKind::SimulationDiagnostics,
            Self::Placeholder => PanelKind::Placeholder,
        }
    }
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
    pub locked_stable_surface_key: Option<ToolSurfaceStableKey>,
    pub legacy_locked_tool_surface_kind: Option<ToolSurfaceKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelInstanceState {
    pub id: PanelInstanceId,
    /// Structural shell/layout grouping, not active tool-surface identity.
    pub panel_kind: PanelKind,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolSurfaceState {
    pub id: ToolSurfaceInstanceId,
    pub stable_surface_key: ToolSurfaceStableKey,
    pub legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    pub mount: ToolSurfaceMount,
    pub viewport_instance_id: Option<ViewportId>,
    pub viewport_settings: Option<ViewportRuntimeSettings>,
}

impl ToolSurfaceState {
    pub fn new_with_stable_key(
        instance_id: ToolSurfaceInstanceId,
        stable_surface_key: ToolSurfaceStableKey,
        legacy_tool_surface_kind: Option<ToolSurfaceKind>,
        mount: ToolSurfaceMount,
    ) -> Self {
        Self {
            id: instance_id,
            stable_surface_key,
            legacy_tool_surface_kind,
            mount,
            viewport_instance_id: None,
            viewport_settings: None,
        }
    }

    pub fn new_legacy(
        instance_id: ToolSurfaceInstanceId,
        tool_surface_kind: ToolSurfaceKind,
        mount: ToolSurfaceMount,
    ) -> Result<Self, WorkspaceSurfaceIdentityError> {
        Self::new_legacy_with_resolver(
            instance_id,
            tool_surface_kind,
            mount,
            stable_key_for_tool_surface_kind,
        )
    }

    fn new_legacy_with_resolver(
        instance_id: ToolSurfaceInstanceId,
        tool_surface_kind: ToolSurfaceKind,
        mount: ToolSurfaceMount,
        stable_key_for_kind: impl FnOnce(ToolSurfaceKind) -> Option<ToolSurfaceStableKey>,
    ) -> Result<Self, WorkspaceSurfaceIdentityError> {
        let stable_surface_key = stable_key_for_kind(tool_surface_kind).ok_or(
            WorkspaceSurfaceIdentityError::UnmappedLegacySurface {
                kind: tool_surface_kind,
            },
        )?;
        Ok(Self::new_with_stable_key(
            instance_id,
            stable_surface_key,
            Some(tool_surface_kind),
            mount,
        ))
    }

    pub const fn stable_surface_key(&self) -> &ToolSurfaceStableKey {
        &self.stable_surface_key
    }

    pub const fn legacy_tool_surface_kind(&self) -> Option<ToolSurfaceKind> {
        self.legacy_tool_surface_kind
    }

    pub fn legacy_tool_surface_kind_or_error(
        &self,
    ) -> Result<ToolSurfaceKind, WorkspaceSurfaceIdentityError> {
        self.legacy_tool_surface_kind.ok_or_else(|| {
            WorkspaceSurfaceIdentityError::MissingLegacyCompatibilityKind {
                stable_surface_key: self.stable_surface_key.clone(),
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDefaultToolSurface {
    pub stable_surface_key: ToolSurfaceStableKey,
    pub panel_kind: PanelKind,
    pub legacy_tool_surface_kind: Option<ToolSurfaceKind>,
}

impl WorkspaceDefaultToolSurface {
    pub fn new_with_panel_kind(
        stable_surface_key: ToolSurfaceStableKey,
        panel_kind: PanelKind,
        legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    ) -> Self {
        Self {
            stable_surface_key,
            panel_kind,
            legacy_tool_surface_kind,
        }
    }

    pub fn new_legacy(
        tool_surface_kind: ToolSurfaceKind,
    ) -> Result<Self, WorkspaceSurfaceIdentityError> {
        let stable_surface_key = stable_key_for_tool_surface_kind(tool_surface_kind).ok_or(
            WorkspaceSurfaceIdentityError::UnmappedLegacySurface {
                kind: tool_surface_kind,
            },
        )?;
        Ok(Self::new_with_panel_kind(
            stable_surface_key,
            tool_surface_kind.panel_kind(),
            Some(tool_surface_kind),
        ))
    }

    pub const fn stable_surface_key(&self) -> &ToolSurfaceStableKey {
        &self.stable_surface_key
    }

    pub const fn panel_kind(&self) -> PanelKind {
        self.panel_kind
    }

    pub const fn legacy_tool_surface_kind(&self) -> Option<ToolSurfaceKind> {
        self.legacy_tool_surface_kind
    }

    pub fn legacy_tool_surface_kind_or_error(
        &self,
    ) -> Result<ToolSurfaceKind, WorkspaceSurfaceIdentityError> {
        self.legacy_tool_surface_kind.ok_or_else(|| {
            WorkspaceSurfaceIdentityError::MissingLegacyCompatibilityKind {
                stable_surface_key: self.stable_surface_key.clone(),
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSurfaceIdentityError {
    UnmappedLegacySurface {
        kind: ToolSurfaceKind,
    },
    MissingLegacyCompatibilityKind {
        stable_surface_key: ToolSurfaceStableKey,
    },
    StableKeyLegacyKindMismatch {
        stable_surface_key: ToolSurfaceStableKey,
        legacy_tool_surface_kind: ToolSurfaceKind,
        expected_stable_surface_key: Option<ToolSurfaceStableKey>,
    },
}

impl std::fmt::Display for WorkspaceSurfaceIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnmappedLegacySurface { kind } => write!(
                f,
                "legacy tool surface {kind:?} has no safe stable-key mapping"
            ),
            Self::MissingLegacyCompatibilityKind { stable_surface_key } => write!(
                f,
                "stable-key authoritative tool surface `{stable_surface_key}` has no legacy compatibility kind"
            ),
            Self::StableKeyLegacyKindMismatch {
                stable_surface_key,
                legacy_tool_surface_kind,
                expected_stable_surface_key,
            } => match expected_stable_surface_key {
                Some(expected) => write!(
                    f,
                    "stable key `{stable_surface_key}` does not match legacy tool surface {legacy_tool_surface_kind:?}; expected `{expected}`"
                ),
                None => write!(
                    f,
                    "stable key `{stable_surface_key}` has no safe legacy mapping for {legacy_tool_surface_kind:?}"
                ),
            },
        }
    }
}

impl std::error::Error for WorkspaceSurfaceIdentityError {}

fn compiled_in_legacy_tool_surface_state(
    instance_id: ToolSurfaceInstanceId,
    tool_surface_kind: ToolSurfaceKind,
    mount: ToolSurfaceMount,
) -> ToolSurfaceState {
    ToolSurfaceState::new_legacy(instance_id, tool_surface_kind, mount)
        .expect("compiled-in saveable ToolSurfaceKind should have a stable key")
}

pub(crate) fn is_viewport_stable_surface_key(key: &ToolSurfaceStableKey) -> bool {
    stable_key_for_tool_surface_kind(ToolSurfaceKind::Viewport).as_ref() == Some(key)
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceToolSurfaceRegistryCompatibilityReport {
    pub compatible_surfaces: Vec<WorkspaceToolSurfaceRegistryCompatibleSurface>,
    pub unregistered_legacy_surfaces: Vec<WorkspaceToolSurfaceRegistryLegacySurface>,
    pub unmapped_legacy_surfaces: Vec<WorkspaceToolSurfaceRegistryUnmappedLegacySurface>,
    pub unknown_stable_keys: Vec<WorkspaceToolSurfaceRegistryUnknownStableKey>,
    pub incompatible_surfaces: Vec<WorkspaceToolSurfaceRegistryIncompatibleSurface>,
}

impl WorkspaceToolSurfaceRegistryCompatibilityReport {
    pub fn is_fully_compatible(&self) -> bool {
        self.unregistered_legacy_surfaces.is_empty()
            && self.unmapped_legacy_surfaces.is_empty()
            && self.unknown_stable_keys.is_empty()
            && self.incompatible_surfaces.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceToolSurfaceRegistryCompatibleSurface {
    pub tool_surface_id: ToolSurfaceInstanceId,
    pub tool_surface_kind: Option<ToolSurfaceKind>,
    pub stable_surface_key: ToolSurfaceStableKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceToolSurfaceRegistryLegacySurface {
    pub tool_surface_id: ToolSurfaceInstanceId,
    pub tool_surface_kind: Option<ToolSurfaceKind>,
    pub stable_surface_key: ToolSurfaceStableKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceToolSurfaceRegistryUnmappedLegacySurface {
    pub tool_surface_id: ToolSurfaceInstanceId,
    pub tool_surface_kind: Option<ToolSurfaceKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceToolSurfaceRegistryUnknownStableKey {
    pub tool_surface_id: ToolSurfaceInstanceId,
    pub tool_surface_kind: Option<ToolSurfaceKind>,
    pub stable_surface_key: ToolSurfaceStableKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceToolSurfaceRegistryIncompatibleSurface {
    pub tool_surface_id: ToolSurfaceInstanceId,
    pub tool_surface_kind: Option<ToolSurfaceKind>,
    pub expected_stable_surface_key: Option<ToolSurfaceStableKey>,
    pub actual_stable_surface_key: ToolSurfaceStableKey,
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
    PersistedStableSurfaceKeyInvalidSyntax {
        tool_surface_id: u64,
        stable_surface_key: String,
    },
    PersistedStableSurfaceKeyUnknown {
        tool_surface_id: u64,
        stable_surface_key: ToolSurfaceStableKey,
    },
    PersistedStableSurfaceKeyLegacyMismatch {
        tool_surface_id: u64,
        stable_surface_key: ToolSurfaceStableKey,
        legacy_tool_surface_kind: ToolSurfaceKind,
        expected_stable_surface_key: Option<ToolSurfaceStableKey>,
    },
    PersistedTabStackLockStableKeyInvalidSyntax {
        tab_stack_id: u64,
        stable_surface_key: String,
    },
    PersistedTabStackLockStableKeyUnknown {
        tab_stack_id: u64,
        stable_surface_key: ToolSurfaceStableKey,
    },
    PersistedTabStackLockStableKeyLegacyMismatch {
        tab_stack_id: u64,
        stable_surface_key: ToolSurfaceStableKey,
        legacy_tool_surface_kind: ToolSurfaceKind,
        expected_stable_surface_key: Option<ToolSurfaceStableKey>,
    },
    TabStackLockStableKeyLegacyMismatch {
        tab_stack_id: TabStackId,
        locked_stable_surface_key: Option<ToolSurfaceStableKey>,
        legacy_locked_tool_surface_kind: ToolSurfaceKind,
        expected_stable_surface_key: Option<ToolSurfaceStableKey>,
    },
    PersistedLegacySurfaceUnmappedForStableWrite {
        tool_surface_id: ToolSurfaceInstanceId,
        tool_surface_kind: ToolSurfaceKind,
    },
    SurfaceIdentity(WorkspaceSurfaceIdentityError),
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
            Self::PersistedStableSurfaceKeyInvalidSyntax {
                tool_surface_id,
                stable_surface_key,
            } => write!(
                f,
                "persisted tool surface {tool_surface_id} has invalid stable key `{stable_surface_key}`",
            ),
            Self::PersistedStableSurfaceKeyUnknown {
                tool_surface_id,
                stable_surface_key,
            } => write!(
                f,
                "persisted tool surface {tool_surface_id} references unknown stable key `{stable_surface_key}`",
            ),
            Self::PersistedStableSurfaceKeyLegacyMismatch {
                tool_surface_id,
                stable_surface_key,
                legacy_tool_surface_kind,
                expected_stable_surface_key,
            } => match expected_stable_surface_key {
                Some(expected) => write!(
                    f,
                    "persisted tool surface {tool_surface_id} stable key `{stable_surface_key}` does not match legacy kind {legacy_tool_surface_kind:?}; expected `{expected}`",
                ),
                None => write!(
                    f,
                    "persisted tool surface {tool_surface_id} stable key `{stable_surface_key}` has no safe legacy mapping for {legacy_tool_surface_kind:?}",
                ),
            },
            Self::PersistedTabStackLockStableKeyInvalidSyntax {
                tab_stack_id,
                stable_surface_key,
            } => write!(
                f,
                "persisted tab stack {tab_stack_id} has invalid lock stable key `{stable_surface_key}`",
            ),
            Self::PersistedTabStackLockStableKeyUnknown {
                tab_stack_id,
                stable_surface_key,
            } => write!(
                f,
                "persisted tab stack {tab_stack_id} references unknown lock stable key `{stable_surface_key}`",
            ),
            Self::PersistedTabStackLockStableKeyLegacyMismatch {
                tab_stack_id,
                stable_surface_key,
                legacy_tool_surface_kind,
                expected_stable_surface_key,
            } => match expected_stable_surface_key {
                Some(expected) => write!(
                    f,
                    "persisted tab stack {tab_stack_id} lock stable key `{stable_surface_key}` does not match legacy kind {legacy_tool_surface_kind:?}; expected `{expected}`",
                ),
                None => write!(
                    f,
                    "persisted tab stack {tab_stack_id} lock stable key `{stable_surface_key}` has no safe legacy mapping for {legacy_tool_surface_kind:?}",
                ),
            },
            Self::TabStackLockStableKeyLegacyMismatch {
                tab_stack_id,
                locked_stable_surface_key,
                legacy_locked_tool_surface_kind,
                expected_stable_surface_key,
            } => match (locked_stable_surface_key, expected_stable_surface_key) {
                (Some(actual), Some(expected)) => write!(
                    f,
                    "tab stack {tab_stack_id:?} lock stable key `{actual}` does not match legacy kind {legacy_locked_tool_surface_kind:?}; expected `{expected}`",
                ),
                (None, Some(expected)) => write!(
                    f,
                    "tab stack {tab_stack_id:?} lock legacy kind {legacy_locked_tool_surface_kind:?} requires stable key `{expected}`",
                ),
                (_, None) => write!(
                    f,
                    "tab stack {tab_stack_id:?} lock legacy kind {legacy_locked_tool_surface_kind:?} has no safe stable-key mapping",
                ),
            },
            Self::PersistedLegacySurfaceUnmappedForStableWrite {
                tool_surface_id,
                tool_surface_kind,
            } => write!(
                f,
                "tool surface {tool_surface_id:?} with legacy kind {tool_surface_kind:?} cannot be written as V5 because it has no stable key mapping",
            ),
            Self::SurfaceIdentity(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for WorkspaceStateError {}

impl From<WorkspaceSurfaceIdentityError> for WorkspaceStateError {
    fn from(error: WorkspaceSurfaceIdentityError) -> Self {
        Self::SurfaceIdentity(error)
    }
}

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
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            viewport_tab_stack,
            TabStackState {
                id: viewport_tab_stack,
                ordered_panels: vec![viewport_panel],
                active_panel: Some(viewport_panel),
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            inspector_tab_stack,
            TabStackState {
                id: inspector_tab_stack,
                ordered_panels: vec![inspector_panel],
                active_panel: Some(inspector_panel),
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            console_tab_stack,
            TabStackState {
                id: console_tab_stack,
                ordered_panels: vec![console_panel],
                active_panel: Some(console_panel),
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
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
            compiled_in_legacy_tool_surface_state(
                outliner_surface,
                ToolSurfaceKind::Outliner,
                ToolSurfaceMount::Mounted {
                    panel_id: outliner_panel,
                },
            ),
        );
        tool_surfaces_by_id.insert(
            entity_table_surface,
            compiled_in_legacy_tool_surface_state(
                entity_table_surface,
                ToolSurfaceKind::EntityTable,
                ToolSurfaceMount::Mounted {
                    panel_id: entity_table_panel,
                },
            ),
        );
        tool_surfaces_by_id.insert(
            viewport_surface,
            compiled_in_legacy_tool_surface_state(
                viewport_surface,
                ToolSurfaceKind::Viewport,
                ToolSurfaceMount::Mounted {
                    panel_id: viewport_panel,
                },
            ),
        );
        tool_surfaces_by_id.insert(
            inspector_surface,
            compiled_in_legacy_tool_surface_state(
                inspector_surface,
                ToolSurfaceKind::Inspector,
                ToolSurfaceMount::Mounted {
                    panel_id: inspector_panel,
                },
            ),
        );
        tool_surfaces_by_id.insert(
            console_surface,
            compiled_in_legacy_tool_surface_state(
                console_surface,
                ToolSurfaceKind::Console,
                ToolSurfaceMount::Mounted {
                    panel_id: console_panel,
                },
            ),
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
        let mut workspace = Self::bootstrap_current_layout(workspace_id, allocator);

        let PanelHostKind::SplitHost(root_split) = workspace
            .hosts_by_id
            .get(&workspace.root_host_id)
            .expect("bootstrap scene layout should have a root host")
            .kind
        else {
            return workspace;
        };
        let left_center_right_host = root_split.first_child;
        let PanelHostKind::SplitHost(scene_left_right) = workspace
            .hosts_by_id
            .get(&left_center_right_host)
            .expect("bootstrap scene layout should have a body split")
            .kind
        else {
            return workspace;
        };
        let viewport_host = scene_left_right.first_child;
        let center_right_host = scene_left_right.second_child;
        let PanelHostKind::SplitHost(scene_right_sidebar) = workspace
            .hosts_by_id
            .get(&center_right_host)
            .expect("bootstrap scene layout should have a sidebar split")
            .kind
        else {
            return workspace;
        };
        let outliner_host = scene_right_sidebar.first_child;
        let inspector_host = scene_right_sidebar.second_child;

        if let Some(host) = workspace.hosts_by_id.get_mut(&left_center_right_host) {
            host.kind = PanelHostKind::SplitHost(SplitHostState {
                axis: WorkspaceSplitAxis::Horizontal,
                fraction: 0.20,
                first_child: outliner_host,
                second_child: center_right_host,
            });
        }
        if let Some(host) = workspace.hosts_by_id.get_mut(&center_right_host) {
            host.kind = PanelHostKind::SplitHost(SplitHostState {
                axis: WorkspaceSplitAxis::Horizontal,
                fraction: 0.76,
                first_child: viewport_host,
                second_child: inspector_host,
            });
        }

        workspace
    }

    pub fn bootstrap_editor_design_layout(
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
    ) -> Self {
        let body_validation_split_host = allocator.allocate_panel_host_id();
        let left_center_right_split_host = allocator.allocate_panel_host_id();
        let center_right_split_host = allocator.allocate_panel_host_id();
        let outliner_tab_host = allocator.allocate_panel_host_id();
        let canvas_tab_host = allocator.allocate_panel_host_id();
        let inspector_tab_host = allocator.allocate_panel_host_id();
        let validation_tab_host = allocator.allocate_panel_host_id();

        let outliner_tab_stack = allocator.allocate_tab_stack_id();
        let canvas_tab_stack = allocator.allocate_tab_stack_id();
        let inspector_tab_stack = allocator.allocate_tab_stack_id();
        let validation_tab_stack = allocator.allocate_tab_stack_id();

        let outliner_panel = allocator.allocate_panel_instance_id();
        let hierarchy_panel = allocator.allocate_panel_instance_id();
        let canvas_panel = allocator.allocate_panel_instance_id();
        let layout_preview_panel = allocator.allocate_panel_instance_id();
        let style_panel = allocator.allocate_panel_instance_id();
        let bindings_panel = allocator.allocate_panel_instance_id();
        let validation_panel = allocator.allocate_panel_instance_id();
        let diff_panel = allocator.allocate_panel_instance_id();

        let outliner_surface = allocator.allocate_tool_surface_instance_id();
        let hierarchy_surface = allocator.allocate_tool_surface_instance_id();
        let canvas_surface = allocator.allocate_tool_surface_instance_id();
        let layout_preview_surface = allocator.allocate_tool_surface_instance_id();
        let style_surface = allocator.allocate_tool_surface_instance_id();
        let bindings_surface = allocator.allocate_tool_surface_instance_id();
        let validation_surface = allocator.allocate_tool_surface_instance_id();
        let diff_surface = allocator.allocate_tool_surface_instance_id();

        let mut hosts_by_id = BTreeMap::new();
        hosts_by_id.insert(
            body_validation_split_host,
            PanelHostNode {
                id: body_validation_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Vertical,
                    fraction: 0.76,
                    first_child: left_center_right_split_host,
                    second_child: validation_tab_host,
                }),
            },
        );
        hosts_by_id.insert(
            left_center_right_split_host,
            PanelHostNode {
                id: left_center_right_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Horizontal,
                    fraction: 0.72,
                    first_child: outliner_tab_host,
                    second_child: center_right_split_host,
                }),
            },
        );
        hosts_by_id.insert(
            center_right_split_host,
            PanelHostNode {
                id: center_right_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Horizontal,
                    fraction: 0.68,
                    first_child: canvas_tab_host,
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
            canvas_tab_host,
            PanelHostNode {
                id: canvas_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: canvas_tab_stack,
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
            validation_tab_host,
            PanelHostNode {
                id: validation_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: validation_tab_stack,
                }),
            },
        );

        let mut tab_stacks_by_id = BTreeMap::new();
        tab_stacks_by_id.insert(
            outliner_tab_stack,
            TabStackState {
                id: outliner_tab_stack,
                ordered_panels: vec![outliner_panel, hierarchy_panel],
                active_panel: Some(outliner_panel),
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            canvas_tab_stack,
            TabStackState {
                id: canvas_tab_stack,
                ordered_panels: vec![canvas_panel, layout_preview_panel],
                active_panel: Some(canvas_panel),
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            inspector_tab_stack,
            TabStackState {
                id: inspector_tab_stack,
                ordered_panels: vec![style_panel, bindings_panel],
                active_panel: Some(style_panel),
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );
        tab_stacks_by_id.insert(
            validation_tab_stack,
            TabStackState {
                id: validation_tab_stack,
                ordered_panels: vec![validation_panel, diff_panel],
                active_panel: Some(validation_panel),
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );

        let mut panels_by_id = BTreeMap::new();
        for (panel_id, panel_kind, surface_id) in [
            (
                outliner_panel,
                PanelKind::EditorDesignOutliner,
                outliner_surface,
            ),
            (hierarchy_panel, PanelKind::UiHierarchy, hierarchy_surface),
            (canvas_panel, PanelKind::UiCanvas, canvas_surface),
            (
                layout_preview_panel,
                PanelKind::DockLayoutPreview,
                layout_preview_surface,
            ),
            (style_panel, PanelKind::StyleInspector, style_surface),
            (bindings_panel, PanelKind::Bindings, bindings_surface),
            (
                validation_panel,
                PanelKind::DefinitionValidation,
                validation_surface,
            ),
            (diff_panel, PanelKind::CommandDiff, diff_surface),
        ] {
            panels_by_id.insert(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind,
                    active_tool_surface: Some(surface_id),
                },
            );
        }

        let mut tool_surfaces_by_id = BTreeMap::new();
        for (surface_id, tool_surface_kind, panel_id) in [
            (
                outliner_surface,
                ToolSurfaceKind::EditorDesignOutliner,
                outliner_panel,
            ),
            (
                hierarchy_surface,
                ToolSurfaceKind::UiHierarchy,
                hierarchy_panel,
            ),
            (canvas_surface, ToolSurfaceKind::UiCanvas, canvas_panel),
            (
                layout_preview_surface,
                ToolSurfaceKind::DockLayoutPreview,
                layout_preview_panel,
            ),
            (style_surface, ToolSurfaceKind::StyleInspector, style_panel),
            (bindings_surface, ToolSurfaceKind::Bindings, bindings_panel),
            (
                validation_surface,
                ToolSurfaceKind::DefinitionValidation,
                validation_panel,
            ),
            (diff_surface, ToolSurfaceKind::CommandDiff, diff_panel),
        ] {
            tool_surfaces_by_id.insert(
                surface_id,
                compiled_in_legacy_tool_surface_state(
                    surface_id,
                    tool_surface_kind,
                    ToolSurfaceMount::Mounted { panel_id },
                ),
            );
        }

        Self {
            workspace_id,
            root_host_id: body_validation_split_host,
            hosts_by_id,
            tab_stacks_by_id,
            panels_by_id,
            tool_surfaces_by_id,
        }
    }

    pub fn bootstrap_tool_workspace_layout(
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
        default_surfaces: &[ToolSurfaceKind],
    ) -> Self {
        let default_surfaces = default_surfaces
            .iter()
            .copied()
            .map(WorkspaceDefaultToolSurface::new_legacy)
            .collect::<Result<Vec<_>, _>>()
            .expect("compiled-in legacy tool workspace surfaces should have stable keys");
        Self::bootstrap_tool_workspace_layout_with_stable_surfaces(
            workspace_id,
            allocator,
            &default_surfaces,
        )
        .expect("compiled-in legacy tool workspace surfaces should keep C3 legacy metadata")
    }

    pub fn bootstrap_tool_workspace_layout_with_stable_surfaces(
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
        default_surfaces: &[WorkspaceDefaultToolSurface],
    ) -> Result<Self, WorkspaceStateError> {
        let body_bottom_split_host = allocator.allocate_panel_host_id();
        let left_center_right_split_host = allocator.allocate_panel_host_id();
        let center_right_split_host = allocator.allocate_panel_host_id();
        let left_tab_host = allocator.allocate_panel_host_id();
        let primary_tab_host = allocator.allocate_panel_host_id();
        let right_tab_host = allocator.allocate_panel_host_id();
        let bottom_tab_host = allocator.allocate_panel_host_id();

        let groups = tool_workspace_surface_groups(default_surfaces);

        let mut tab_stacks_by_id = BTreeMap::new();
        let mut panels_by_id = BTreeMap::new();
        let mut tool_surfaces_by_id = BTreeMap::new();

        let left_tab_stack = insert_tool_workspace_stack(
            &mut tab_stacks_by_id,
            &mut panels_by_id,
            &mut tool_surfaces_by_id,
            allocator,
            &groups.left,
        )?;
        let primary_tab_stack = insert_tool_workspace_stack(
            &mut tab_stacks_by_id,
            &mut panels_by_id,
            &mut tool_surfaces_by_id,
            allocator,
            &groups.primary,
        )?;
        let right_tab_stack = insert_tool_workspace_stack(
            &mut tab_stacks_by_id,
            &mut panels_by_id,
            &mut tool_surfaces_by_id,
            allocator,
            &groups.right,
        )?;
        let bottom_tab_stack = insert_tool_workspace_stack(
            &mut tab_stacks_by_id,
            &mut panels_by_id,
            &mut tool_surfaces_by_id,
            allocator,
            &groups.bottom,
        )?;

        let mut hosts_by_id = BTreeMap::new();
        hosts_by_id.insert(
            body_bottom_split_host,
            PanelHostNode {
                id: body_bottom_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Vertical,
                    fraction: 0.76,
                    first_child: left_center_right_split_host,
                    second_child: bottom_tab_host,
                }),
            },
        );
        hosts_by_id.insert(
            left_center_right_split_host,
            PanelHostNode {
                id: left_center_right_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Horizontal,
                    fraction: 0.22,
                    first_child: left_tab_host,
                    second_child: center_right_split_host,
                }),
            },
        );
        hosts_by_id.insert(
            center_right_split_host,
            PanelHostNode {
                id: center_right_split_host,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Horizontal,
                    fraction: 0.70,
                    first_child: primary_tab_host,
                    second_child: right_tab_host,
                }),
            },
        );
        hosts_by_id.insert(
            left_tab_host,
            PanelHostNode {
                id: left_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: left_tab_stack,
                }),
            },
        );
        hosts_by_id.insert(
            primary_tab_host,
            PanelHostNode {
                id: primary_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: primary_tab_stack,
                }),
            },
        );
        hosts_by_id.insert(
            right_tab_host,
            PanelHostNode {
                id: right_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: right_tab_stack,
                }),
            },
        );
        hosts_by_id.insert(
            bottom_tab_host,
            PanelHostNode {
                id: bottom_tab_host,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: bottom_tab_stack,
                }),
            },
        );

        Ok(Self {
            workspace_id,
            root_host_id: body_bottom_split_host,
            hosts_by_id,
            tab_stacks_by_id,
            panels_by_id,
            tool_surfaces_by_id,
        })
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

    pub fn populate_stable_surface_keys_from_legacy(&mut self) {
        // C1 compatibility shim: stable keys are already authoritative on each surface.
    }

    pub fn validate_tool_surface_registry_compatibility(
        &self,
        registry: &ToolSurfaceRegistry,
    ) -> WorkspaceToolSurfaceRegistryCompatibilityReport {
        let mut report = WorkspaceToolSurfaceRegistryCompatibilityReport::default();

        for surface in self.tool_surfaces_by_id.values() {
            let legacy_tool_surface_kind = surface.legacy_tool_surface_kind();
            let legacy_candidate =
                legacy_tool_surface_kind.and_then(stable_key_for_tool_surface_kind);
            let actual = surface.stable_surface_key();
            match (legacy_tool_surface_kind, legacy_candidate.as_ref()) {
                (Some(_), Some(expected)) if actual != expected => {
                    report.incompatible_surfaces.push(
                        WorkspaceToolSurfaceRegistryIncompatibleSurface {
                            tool_surface_id: surface.id,
                            tool_surface_kind: legacy_tool_surface_kind,
                            expected_stable_surface_key: Some(expected.clone()),
                            actual_stable_surface_key: actual.clone(),
                        },
                    );
                }
                (Some(_), Some(_)) => {
                    push_legacy_registry_compatibility(
                        surface,
                        actual.clone(),
                        legacy_tool_surface_kind,
                        registry,
                        &mut report,
                    );
                }
                (None, Some(_)) => {
                    push_legacy_registry_compatibility(
                        surface,
                        actual.clone(),
                        legacy_tool_surface_kind,
                        registry,
                        &mut report,
                    );
                }
                (_, None) => {
                    if registry.get(actual).is_some() && legacy_tool_surface_kind.is_none() {
                        push_legacy_registry_compatibility(
                            surface,
                            actual.clone(),
                            legacy_tool_surface_kind,
                            registry,
                            &mut report,
                        );
                    } else if registry.get(actual).is_some() {
                        report.incompatible_surfaces.push(
                            WorkspaceToolSurfaceRegistryIncompatibleSurface {
                                tool_surface_id: surface.id,
                                tool_surface_kind: legacy_tool_surface_kind,
                                expected_stable_surface_key: None,
                                actual_stable_surface_key: actual.clone(),
                            },
                        );
                    } else {
                        report.unknown_stable_keys.push(
                            WorkspaceToolSurfaceRegistryUnknownStableKey {
                                tool_surface_id: surface.id,
                                tool_surface_kind: legacy_tool_surface_kind,
                                stable_surface_key: actual.clone(),
                            },
                        );
                    }
                }
            }
        }

        report
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
            if (tool_surface.viewport_instance_id.is_some()
                || tool_surface.viewport_settings.is_some())
                && !is_viewport_stable_surface_key(tool_surface.stable_surface_key())
            {
                return Err(WorkspaceStateError::ProjectionShapeMismatch(
                    "viewport restore metadata must belong to viewport tool surfaces",
                ));
            }
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

fn push_legacy_registry_compatibility(
    surface: &ToolSurfaceState,
    stable_surface_key: ToolSurfaceStableKey,
    legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    registry: &ToolSurfaceRegistry,
    report: &mut WorkspaceToolSurfaceRegistryCompatibilityReport,
) {
    if registry.get(&stable_surface_key).is_some() {
        report
            .compatible_surfaces
            .push(WorkspaceToolSurfaceRegistryCompatibleSurface {
                tool_surface_id: surface.id,
                tool_surface_kind: legacy_tool_surface_kind,
                stable_surface_key,
            });
    } else {
        report
            .unregistered_legacy_surfaces
            .push(WorkspaceToolSurfaceRegistryLegacySurface {
                tool_surface_id: surface.id,
                tool_surface_kind: legacy_tool_surface_kind,
                stable_surface_key,
            });
    }
}

#[derive(Debug, Default)]
struct ToolWorkspaceSurfaceGroups {
    left: Vec<WorkspaceDefaultToolSurface>,
    primary: Vec<WorkspaceDefaultToolSurface>,
    right: Vec<WorkspaceDefaultToolSurface>,
    bottom: Vec<WorkspaceDefaultToolSurface>,
}

fn tool_workspace_surface_groups(
    default_surfaces: &[WorkspaceDefaultToolSurface],
) -> ToolWorkspaceSurfaceGroups {
    let mut groups = ToolWorkspaceSurfaceGroups::default();
    for surface in unique_tool_workspace_surfaces(default_surfaces) {
        match surface.panel_kind() {
            PanelKind::Console
            | PanelKind::Diagnostics
            | PanelKind::RuntimeDebug
            | PanelKind::GameplayCompilerDiagnostics
            | PanelKind::SimulationDiagnostics
            | PanelKind::DefinitionValidation
            | PanelKind::CommandDiff
            | PanelKind::PhysicsDebug => groups.bottom.push(surface),
            PanelKind::Inspector
            | PanelKind::ImportInspector
            | PanelKind::MaterialInspector
            | PanelKind::PhysicsAuthoring
            | PanelKind::StyleInspector
            | PanelKind::Bindings => groups.right.push(surface),
            PanelKind::Outliner
            | PanelKind::EntityTable
            | PanelKind::EditorDesignOutliner
            | PanelKind::UiHierarchy
            | PanelKind::AssetBrowser
            | PanelKind::SdfBrushBrowser
            | PanelKind::FieldLayerStack => groups.left.push(surface),
            _ => groups.primary.push(surface),
        }
    }

    if groups.primary.is_empty() {
        groups.primary.push(compiled_in_default_tool_surface(
            ToolSurfaceKind::Placeholder,
        ));
    }
    if groups.right.is_empty() {
        groups.right.push(compiled_in_default_tool_surface(
            ToolSurfaceKind::Placeholder,
        ));
    }
    if groups.left.is_empty() {
        groups.left.push(compiled_in_default_tool_surface(
            ToolSurfaceKind::Placeholder,
        ));
    }
    if groups.bottom.is_empty() {
        groups
            .bottom
            .push(compiled_in_default_tool_surface(ToolSurfaceKind::Console));
    }

    groups
}

fn unique_tool_workspace_surfaces(
    default_surfaces: &[WorkspaceDefaultToolSurface],
) -> Vec<WorkspaceDefaultToolSurface> {
    let mut surfaces = Vec::new();
    for surface in default_surfaces {
        if !surfaces
            .iter()
            .any(|existing: &WorkspaceDefaultToolSurface| {
                existing.stable_surface_key() == surface.stable_surface_key()
            })
        {
            surfaces.push(surface.clone());
        }
    }
    if surfaces.is_empty() {
        surfaces.push(compiled_in_default_tool_surface(
            ToolSurfaceKind::Placeholder,
        ));
    }
    surfaces
}

fn compiled_in_default_tool_surface(kind: ToolSurfaceKind) -> WorkspaceDefaultToolSurface {
    WorkspaceDefaultToolSurface::new_legacy(kind)
        .expect("compiled-in saveable ToolSurfaceKind should have a stable key")
}

fn insert_tool_workspace_stack(
    tab_stacks_by_id: &mut BTreeMap<TabStackId, TabStackState>,
    panels_by_id: &mut BTreeMap<PanelInstanceId, PanelInstanceState>,
    tool_surfaces_by_id: &mut BTreeMap<ToolSurfaceInstanceId, ToolSurfaceState>,
    allocator: &mut WorkspaceIdentityAllocator,
    surfaces: &[WorkspaceDefaultToolSurface],
) -> Result<TabStackId, WorkspaceStateError> {
    let tab_stack_id = allocator.allocate_tab_stack_id();
    let mut ordered_panels = Vec::new();

    for surface in surfaces {
        let panel_id = allocator.allocate_panel_instance_id();
        let surface_id = allocator.allocate_tool_surface_instance_id();
        ordered_panels.push(panel_id);
        panels_by_id.insert(
            panel_id,
            PanelInstanceState {
                id: panel_id,
                panel_kind: surface.panel_kind(),
                active_tool_surface: Some(surface_id),
            },
        );
        tool_surfaces_by_id.insert(
            surface_id,
            ToolSurfaceState::new_with_stable_key(
                surface_id,
                surface.stable_surface_key.clone(),
                surface.legacy_tool_surface_kind,
                ToolSurfaceMount::Mounted { panel_id },
            ),
        );
    }

    tab_stacks_by_id.insert(
        tab_stack_id,
        TabStackState {
            id: tab_stack_id,
            active_panel: ordered_panels.first().copied(),
            ordered_panels,
            locked_stable_surface_key: None,
            legacy_locked_tool_surface_kind: None,
        },
    );

    Ok(tab_stack_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EditorToolSuite, ProviderFamilyDefinition, ProviderFamilyId, ToolSuiteId,
        ToolSuiteRegistry, ToolSurfaceDefinition, ToolSurfacePersistence, ToolSurfaceRole,
        ToolSurfaceRoute,
    };

    #[test]
    fn workspace_tool_surface_state_requires_stable_key_authority() {
        let stable_key = ToolSurfaceStableKey::new("runenwerk.scene.viewport").unwrap();

        let surface = ToolSurfaceState::new_with_stable_key(
            ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
            stable_key.clone(),
            Some(ToolSurfaceKind::Viewport),
            ToolSurfaceMount::Unmounted,
        );

        assert_eq!(surface.stable_surface_key(), &stable_key);
        assert_eq!(
            surface.legacy_tool_surface_kind(),
            Some(ToolSurfaceKind::Viewport)
        );
    }

    #[test]
    fn stable_key_constructor_does_not_require_legacy_kind() {
        let stable_key = ToolSurfaceStableKey::new("runenwerk.scene.viewport").unwrap();

        let surface = ToolSurfaceState::new_with_stable_key(
            ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
            stable_key.clone(),
            None,
            ToolSurfaceMount::Unmounted,
        );

        assert_eq!(surface.stable_surface_key(), &stable_key);
        assert_eq!(surface.legacy_tool_surface_kind(), None);
        assert!(matches!(
            surface.legacy_tool_surface_kind_or_error(),
            Err(WorkspaceSurfaceIdentityError::MissingLegacyCompatibilityKind { .. })
        ));
    }

    #[test]
    fn legacy_tool_surface_constructor_populates_compatibility_kind() {
        let surface = ToolSurfaceState::new_legacy(
            ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
            ToolSurfaceKind::MaterialGraphCanvas,
            ToolSurfaceMount::Unmounted,
        )
        .expect("material graph should have a stable key");

        assert_eq!(
            surface.stable_surface_key().as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
        assert_eq!(
            surface.legacy_tool_surface_kind(),
            Some(ToolSurfaceKind::MaterialGraphCanvas)
        );
    }

    #[test]
    fn legacy_tool_surface_constructor_rejects_unmapped_kind() {
        let error = ToolSurfaceState::new_legacy_with_resolver(
            ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
            ToolSurfaceKind::MaterialGraphCanvas,
            ToolSurfaceMount::Unmounted,
            |_| None,
        )
        .expect_err("unmapped legacy kind must be rejected");

        assert_eq!(
            error,
            WorkspaceSurfaceIdentityError::UnmappedLegacySurface {
                kind: ToolSurfaceKind::MaterialGraphCanvas
            }
        );
    }

    #[test]
    fn workspace_state_stable_key_authority_preserves_layout_identity() {
        let workspace = workspace_with_surfaces(&[
            ToolSurfaceKind::MaterialGraphCanvas,
            ToolSurfaceKind::Viewport,
        ]);

        assert_eq!(
            tool_surface_identity_snapshot(&workspace),
            vec![
                (
                    ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
                    ToolSurfaceKind::MaterialGraphCanvas,
                    ToolSurfaceMount::Mounted {
                        panel_id: PanelInstanceId::try_from_raw(1).unwrap()
                    },
                    None,
                    None,
                ),
                (
                    ToolSurfaceInstanceId::try_from_raw(2).unwrap(),
                    ToolSurfaceKind::Viewport,
                    ToolSurfaceMount::Mounted {
                        panel_id: PanelInstanceId::try_from_raw(2).unwrap()
                    },
                    None,
                    None,
                ),
            ]
        );
        assert_surface_key(
            &workspace,
            ToolSurfaceKind::MaterialGraphCanvas,
            "runenwerk.material_lab.graph_canvas",
        );
        assert_surface_key(
            &workspace,
            ToolSurfaceKind::Viewport,
            "runenwerk.scene.viewport",
        );
    }

    #[test]
    fn panel_kind_remains_authoritative_in_c1() {
        let workspace = workspace_with_surfaces(&[ToolSurfaceKind::Viewport]);
        let panel = workspace
            .panels()
            .next()
            .expect("test workspace should contain a panel");

        assert_eq!(panel.panel_kind, PanelKind::Viewport);
    }

    #[test]
    fn panel_kind_is_structural_layout_grouping_not_tool_surface_identity() {
        let workspace = workspace_with_surfaces(&[ToolSurfaceKind::Viewport]);
        let surface = surface_by_kind(&workspace, ToolSurfaceKind::Viewport);
        let ToolSurfaceMount::Mounted { panel_id } = surface.mount else {
            panic!("test surface should be mounted");
        };
        let panel = workspace
            .panel(panel_id)
            .expect("mounted panel should exist");

        assert_eq!(panel.panel_kind, PanelKind::Viewport);
        assert_eq!(
            surface.stable_surface_key().as_str(),
            "runenwerk.scene.viewport"
        );
    }

    #[test]
    fn panel_kind_does_not_replace_stable_surface_key_identity() {
        let mut workspace = workspace_with_surfaces(&[ToolSurfaceKind::Viewport]);
        let (surface_id, panel_id) = {
            let surface = surface_by_kind(&workspace, ToolSurfaceKind::Viewport);
            let ToolSurfaceMount::Mounted { panel_id } = surface.mount else {
                panic!("test surface should be mounted");
            };
            (surface.id, panel_id)
        };

        workspace
            .panels_by_id
            .get_mut(&panel_id)
            .expect("mounted panel should exist")
            .panel_kind = PanelKind::Console;

        workspace
            .validate_integrity()
            .expect("panel kind is structural and should not rewrite surface identity");
        assert_eq!(
            workspace
                .tool_surface(surface_id)
                .expect("surface should remain present")
                .stable_surface_key()
                .as_str(),
            "runenwerk.scene.viewport"
        );
        assert_eq!(
            workspace
                .panel(panel_id)
                .expect("panel should remain present")
                .panel_kind,
            PanelKind::Console
        );
    }

    #[test]
    fn tool_surface_stable_key_remains_surface_identity_when_panel_kind_is_same() {
        let mut workspace =
            workspace_with_surfaces(&[ToolSurfaceKind::Viewport, ToolSurfaceKind::Console]);
        let (viewport_panel_id, console_panel_id, console_surface_id) = {
            let viewport_surface = surface_by_kind(&workspace, ToolSurfaceKind::Viewport);
            let console_surface = surface_by_kind(&workspace, ToolSurfaceKind::Console);
            let ToolSurfaceMount::Mounted {
                panel_id: viewport_panel_id,
            } = viewport_surface.mount
            else {
                panic!("viewport surface should be mounted");
            };
            let ToolSurfaceMount::Mounted {
                panel_id: console_panel_id,
            } = console_surface.mount
            else {
                panic!("console surface should be mounted");
            };
            (viewport_panel_id, console_panel_id, console_surface.id)
        };

        workspace
            .panels_by_id
            .get_mut(&console_panel_id)
            .expect("console panel should exist")
            .panel_kind = PanelKind::Viewport;

        workspace
            .validate_integrity()
            .expect("distinct stable-key surfaces may share a structural panel grouping");
        assert_eq!(
            workspace
                .panel(viewport_panel_id)
                .map(|panel| panel.panel_kind),
            Some(PanelKind::Viewport)
        );
        assert_eq!(
            workspace
                .panel(console_panel_id)
                .map(|panel| panel.panel_kind),
            Some(PanelKind::Viewport)
        );
        assert_eq!(
            workspace
                .tool_surface(console_surface_id)
                .expect("console surface should remain present")
                .stable_surface_key()
                .as_str(),
            "runenwerk.editor.console"
        );
    }

    #[test]
    fn workspace_populates_material_lab_stable_keys_from_legacy_kinds() {
        let mut workspace = workspace_with_surfaces(&[
            ToolSurfaceKind::MaterialGraphCanvas,
            ToolSurfaceKind::MaterialInspector,
            ToolSurfaceKind::MaterialPreview,
        ]);

        workspace.populate_stable_surface_keys_from_legacy();

        assert_surface_key(
            &workspace,
            ToolSurfaceKind::MaterialGraphCanvas,
            "runenwerk.material_lab.graph_canvas",
        );
        assert_surface_key(
            &workspace,
            ToolSurfaceKind::MaterialInspector,
            "runenwerk.material_lab.inspector",
        );
        assert_surface_key(
            &workspace,
            ToolSurfaceKind::MaterialPreview,
            "runenwerk.material_lab.preview",
        );
    }

    #[test]
    fn workspace_populates_placeholder_fallback_stable_key() {
        let mut workspace = workspace_with_surfaces(&[ToolSurfaceKind::Placeholder]);

        workspace.populate_stable_surface_keys_from_legacy();

        let surface = surface_by_kind(&workspace, ToolSurfaceKind::Placeholder);
        assert_eq!(
            surface.stable_surface_key().as_str(),
            "runenwerk.diagnostics.placeholder"
        );
    }

    #[test]
    fn workspace_registry_compatibility_reports_unregistered_legacy_surface() {
        let mut workspace = workspace_with_surfaces(&[ToolSurfaceKind::MaterialGraphCanvas]);
        workspace.populate_stable_surface_keys_from_legacy();
        let registry = ToolSuiteRegistry::new(Vec::new()).expect("empty registry is valid");

        let report = workspace.validate_tool_surface_registry_compatibility(registry.surfaces());

        assert_eq!(report.compatible_surfaces.len(), 0);
        assert_eq!(report.unregistered_legacy_surfaces.len(), 1);
        assert_eq!(report.unknown_stable_keys.len(), 0);
        assert_eq!(report.incompatible_surfaces.len(), 0);
        assert_eq!(
            report.unregistered_legacy_surfaces[0]
                .stable_surface_key
                .as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
    }

    #[test]
    fn workspace_registry_compatibility_accepts_registered_material_lab_surfaces() {
        let mut workspace = workspace_with_surfaces(&[
            ToolSurfaceKind::MaterialGraphCanvas,
            ToolSurfaceKind::MaterialInspector,
            ToolSurfaceKind::MaterialPreview,
        ]);
        workspace.populate_stable_surface_keys_from_legacy();
        let registry = material_lab_registry();

        let report = workspace.validate_tool_surface_registry_compatibility(registry.surfaces());

        assert!(report.is_fully_compatible());
        let compatible_keys = report
            .compatible_surfaces
            .iter()
            .map(|surface| surface.stable_surface_key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            compatible_keys,
            vec![
                "runenwerk.material_lab.graph_canvas",
                "runenwerk.material_lab.inspector",
                "runenwerk.material_lab.preview",
            ]
        );
    }

    #[test]
    fn populate_stable_surface_keys_does_not_change_workspace_layout_identity() {
        let mut workspace = workspace_with_surfaces(&[
            ToolSurfaceKind::MaterialGraphCanvas,
            ToolSurfaceKind::Viewport,
        ]);
        let root_host_id = workspace.root_host_id;
        let hosts = workspace.hosts_by_id.clone();
        let tab_stacks = workspace.tab_stacks_by_id.clone();
        let panels = workspace.panels_by_id.clone();
        let tool_surface_identity = tool_surface_identity_snapshot(&workspace);
        let next_identity_seed = workspace.next_identity_seed();

        workspace.populate_stable_surface_keys_from_legacy();

        assert_eq!(workspace.root_host_id, root_host_id);
        assert_eq!(workspace.hosts_by_id, hosts);
        assert_eq!(workspace.tab_stacks_by_id, tab_stacks);
        assert_eq!(workspace.panels_by_id, panels);
        assert_eq!(
            tool_surface_identity_snapshot(&workspace),
            tool_surface_identity
        );
        assert_eq!(workspace.next_identity_seed(), next_identity_seed);
    }

    #[test]
    fn workspace_registry_compatibility_rejects_unknown_stable_key_metadata_for_mapped_surface() {
        let unknown_key = ToolSurfaceStableKey::new("runenwerk.unknown.surface").unwrap();
        let mut workspace = workspace_with_surfaces(&[ToolSurfaceKind::Placeholder]);
        let surface = workspace
            .tool_surfaces_by_id
            .values_mut()
            .next()
            .expect("surface should exist");
        surface.stable_surface_key = unknown_key.clone();
        let registry = material_lab_registry();

        let report = workspace.validate_tool_surface_registry_compatibility(registry.surfaces());

        assert_eq!(report.incompatible_surfaces.len(), 1);
        assert_eq!(
            report.incompatible_surfaces[0].actual_stable_surface_key,
            unknown_key
        );
        assert_eq!(
            report.incompatible_surfaces[0]
                .expected_stable_surface_key
                .as_ref()
                .map(ToolSurfaceStableKey::as_str),
            Some("runenwerk.diagnostics.placeholder")
        );
        assert_eq!(report.unknown_stable_keys.len(), 0);
        assert_eq!(report.unregistered_legacy_surfaces.len(), 0);
    }

    #[test]
    fn workspace_registry_compatibility_reports_incompatible_stable_key_metadata() {
        let mut workspace = workspace_with_surfaces(&[ToolSurfaceKind::MaterialGraphCanvas]);
        let surface = workspace
            .tool_surfaces_by_id
            .values_mut()
            .next()
            .expect("surface should exist");
        surface.stable_surface_key =
            ToolSurfaceStableKey::new("runenwerk.material_lab.preview").unwrap();
        let registry = material_lab_registry();

        let report = workspace.validate_tool_surface_registry_compatibility(registry.surfaces());

        assert_eq!(report.incompatible_surfaces.len(), 1);
        assert_eq!(
            report.incompatible_surfaces[0]
                .expected_stable_surface_key
                .as_ref()
                .map(ToolSurfaceStableKey::as_str),
            Some("runenwerk.material_lab.graph_canvas")
        );
        assert_eq!(
            report.incompatible_surfaces[0]
                .actual_stable_surface_key
                .as_str(),
            "runenwerk.material_lab.preview"
        );
    }

    fn workspace_with_surfaces(surface_kinds: &[ToolSurfaceKind]) -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let root_host_id = allocator.allocate_panel_host_id();
        let tab_stack_id = allocator.allocate_tab_stack_id();
        let mut ordered_panels = Vec::new();
        let mut panels_by_id = BTreeMap::new();
        let mut tool_surfaces_by_id = BTreeMap::new();

        for kind in surface_kinds.iter().copied() {
            let panel_id = allocator.allocate_panel_instance_id();
            let surface_id = allocator.allocate_tool_surface_instance_id();
            ordered_panels.push(panel_id);
            panels_by_id.insert(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind: kind.panel_kind(),
                    active_tool_surface: Some(surface_id),
                },
            );
            tool_surfaces_by_id.insert(
                surface_id,
                compiled_in_legacy_tool_surface_state(
                    surface_id,
                    kind,
                    ToolSurfaceMount::Mounted { panel_id },
                ),
            );
        }

        let mut hosts_by_id = BTreeMap::new();
        hosts_by_id.insert(
            root_host_id,
            PanelHostNode {
                id: root_host_id,
                kind: PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }),
            },
        );
        let mut tab_stacks_by_id = BTreeMap::new();
        tab_stacks_by_id.insert(
            tab_stack_id,
            TabStackState {
                id: tab_stack_id,
                active_panel: ordered_panels.first().copied(),
                ordered_panels,
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );

        let workspace = WorkspaceState {
            workspace_id,
            root_host_id,
            hosts_by_id,
            tab_stacks_by_id,
            panels_by_id,
            tool_surfaces_by_id,
        };
        workspace
            .validate_integrity()
            .expect("test workspace should be structurally valid");
        workspace
    }

    fn material_lab_registry() -> ToolSuiteRegistry {
        let provider_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        ToolSuiteRegistry::new(vec![EditorToolSuite {
            suite_id: ToolSuiteId::new("runenwerk.material_lab").unwrap(),
            label: "Material Lab".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family.clone(),
                label: "Material Lab".to_string(),
            }],
            surfaces: vec![
                material_lab_surface(
                    "runenwerk.material_lab.graph_canvas",
                    "Material Graph",
                    ToolSurfaceRole::Primary,
                    provider_family.clone(),
                    ToolSurfaceRoute::ProviderOwnedGraphCanvas,
                ),
                material_lab_surface(
                    "runenwerk.material_lab.inspector",
                    "Material Inspector",
                    ToolSurfaceRole::Inspector,
                    provider_family.clone(),
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
                material_lab_surface(
                    "runenwerk.material_lab.preview",
                    "Material Preview",
                    ToolSurfaceRole::Preview,
                    provider_family,
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
            ],
        }])
        .expect("material lab registry fixture should be valid")
    }

    fn material_lab_surface(
        key: &str,
        label: &str,
        role: ToolSurfaceRole,
        provider_family: ProviderFamilyId,
        route: ToolSurfaceRoute,
    ) -> ToolSurfaceDefinition {
        ToolSurfaceDefinition {
            key: ToolSurfaceStableKey::new(key).unwrap(),
            label: label.to_string(),
            role,
            panel_kind: match role {
                ToolSurfaceRole::Primary => crate::PanelKind::MaterialGraphCanvas,
                ToolSurfaceRole::Inspector => crate::PanelKind::MaterialInspector,
                ToolSurfaceRole::Preview => crate::PanelKind::MaterialPreview,
            },
            provider_family,
            route,
            persistence: ToolSurfacePersistence::StableKey,
        }
    }

    fn assert_surface_key(workspace: &WorkspaceState, kind: ToolSurfaceKind, expected_key: &str) {
        let surface = surface_by_kind(workspace, kind);
        assert_eq!(surface.stable_surface_key().as_str(), expected_key);
    }

    fn surface_by_kind(workspace: &WorkspaceState, kind: ToolSurfaceKind) -> &ToolSurfaceState {
        workspace
            .tool_surfaces_by_id
            .values()
            .find(|surface| surface.legacy_tool_surface_kind() == Some(kind))
            .expect("surface kind should exist")
    }

    fn tool_surface_identity_snapshot(
        workspace: &WorkspaceState,
    ) -> Vec<(
        ToolSurfaceInstanceId,
        ToolSurfaceKind,
        ToolSurfaceMount,
        Option<ViewportId>,
        Option<ViewportRuntimeSettings>,
    )> {
        workspace
            .tool_surfaces_by_id
            .values()
            .map(|surface| {
                (
                    surface.id,
                    surface
                        .legacy_tool_surface_kind_or_error()
                        .expect("test workspace should retain legacy compatibility metadata"),
                    surface.mount,
                    surface.viewport_instance_id,
                    surface.viewport_settings,
                )
            })
            .collect()
    }
}
