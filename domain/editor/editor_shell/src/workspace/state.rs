//! File: domain/editor/editor_shell/src/workspace/state.rs
//! Purpose: Editor-shell structural identity, layout metadata, and legacy surface vocabulary.
//!
//! Identity invariants:
//! - `WorkspaceId` identifies the workspace structural root only.
//! - `PanelHostId` identifies container/layout nodes only.
//! - `TabStackId` identifies tab containers only.
//! - `PanelInstanceId` identifies panel structure instances only.
//! - `ToolSurfaceInstanceId` identifies tool-surface content instances only.
//! - runtime `editor_viewport::ViewportId` can be retained for viewport restore metadata, but
//!   it is never a workspace structural id.

use editor_viewport::{ViewportId, ViewportRuntimeSettings};

use crate::{
    PanelHostId, PanelInstanceId, TabStackId, ToolSurfaceInstanceId,
    tool_suite::ToolSurfaceStableKey,
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
/// authored legacy keys, named legacy wrappers, shell/app command compatibility
/// pending final cleanup, and compatibility tests.
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
    pub mount: ToolSurfaceMount,
    pub viewport_instance_id: Option<ViewportId>,
    pub viewport_settings: Option<ViewportRuntimeSettings>,
}

impl ToolSurfaceState {
    pub fn new_with_stable_key(
        instance_id: ToolSurfaceInstanceId,
        stable_surface_key: ToolSurfaceStableKey,
        mount: ToolSurfaceMount,
    ) -> Self {
        Self {
            id: instance_id,
            stable_surface_key,
            mount,
            viewport_instance_id: None,
            viewport_settings: None,
        }
    }

    pub const fn stable_surface_key(&self) -> &ToolSurfaceStableKey {
        &self.stable_surface_key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDefaultToolSurface {
    pub stable_surface_key: ToolSurfaceStableKey,
    pub panel_kind: PanelKind,
}

impl WorkspaceDefaultToolSurface {
    pub fn new_with_panel_kind(
        stable_surface_key: ToolSurfaceStableKey,
        panel_kind: PanelKind,
    ) -> Self {
        Self {
            stable_surface_key,
            panel_kind,
        }
    }

    pub const fn stable_surface_key(&self) -> &ToolSurfaceStableKey {
        &self.stable_surface_key
    }

    pub const fn panel_kind(&self) -> PanelKind {
        self.panel_kind
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
