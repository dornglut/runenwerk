//! File: domain/editor/editor_shell/src/workspace/persisted.rs
//! Purpose: Versioned persisted DTO semantics for workspace structural identity.

use std::collections::{BTreeMap, BTreeSet};

use editor_viewport::{
    ExpressionProductId, ViewportCameraSettings, ViewportDebugStage,
    ViewportFieldVisualizerColorRamp, ViewportFieldVisualizerComponent,
    ViewportFieldVisualizerDebugMode, ViewportFieldVisualizerSettings, ViewportId,
    ViewportRuntimeSettings,
};
use serde::{Deserialize, Serialize};

use crate::{
    FloatingHostBounds, FloatingHostPlaceholderState, PanelHostId, PanelHostKind, PanelHostNode,
    PanelInstanceId, PanelInstanceState, PanelKind, SplitHostState, TabStackHostState, TabStackId,
    TabStackState, ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState,
    WorkspaceId, WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError,
    tool_suite::{
        ToolSurfaceRegistry, ToolSurfaceStableKey, stable_key_for_tool_surface_kind,
        tool_surface_kind_for_stable_key,
    },
};

pub const PERSISTED_WORKSPACE_STATE_VERSION_V1: u32 = 1;
pub const PERSISTED_WORKSPACE_STATE_VERSION_V2: u32 = 2;
pub const PERSISTED_WORKSPACE_STATE_VERSION_V3: u32 = 3;
pub const PERSISTED_WORKSPACE_STATE_VERSION_V4: u32 = 4;
pub const PERSISTED_WORKSPACE_STATE_VERSION_V5: u32 = 5;

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
pub struct PersistedWorkspaceStateV3 {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_profile_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_template_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_saved_at_unix_seconds: Option<u64>,
    pub workspace_id: u64,
    pub root_host_id: u64,
    pub hosts: Vec<PersistedPanelHostNodeV1>,
    pub tab_stacks: Vec<PersistedTabStackStateV1>,
    pub panels: Vec<PersistedPanelInstanceStateV2>,
    pub tool_surfaces: Vec<PersistedToolSurfaceStateV3>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedWorkspaceStateV4 {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_profile_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_template_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_saved_at_unix_seconds: Option<u64>,
    pub workspace_id: u64,
    pub root_host_id: u64,
    pub hosts: Vec<PersistedPanelHostNodeV1>,
    pub tab_stacks: Vec<PersistedTabStackStateV1>,
    pub panels: Vec<PersistedPanelInstanceStateV2>,
    pub tool_surfaces: Vec<PersistedToolSurfaceStateV3>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedWorkspaceStateV5 {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_profile_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_template_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_saved_at_unix_seconds: Option<u64>,
    pub workspace_id: u64,
    pub root_host_id: u64,
    pub hosts: Vec<PersistedPanelHostNodeV1>,
    pub tab_stacks: Vec<PersistedTabStackStateV5>,
    pub panels: Vec<PersistedPanelInstanceStateV2>,
    pub tool_surfaces: Vec<PersistedToolSurfaceStateV5>,
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
pub struct PersistedTabStackStateV5 {
    pub id: u64,
    pub ordered_panels: Vec<u64>,
    pub active_panel: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked_stable_surface_key: Option<String>,
    #[serde(
        default,
        alias = "locked_tool_surface_kind",
        skip_serializing_if = "Option::is_none"
    )]
    pub legacy_locked_tool_surface_kind: Option<PersistedToolSurfaceKindV2>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedToolSurfaceStateV2 {
    pub id: u64,
    pub tool_surface_kind: PersistedToolSurfaceKindV2,
    pub mount: PersistedToolSurfaceMountV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedToolSurfaceStateV3 {
    pub id: u64,
    pub tool_surface_kind: PersistedToolSurfaceKindV2,
    pub mount: PersistedToolSurfaceMountV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport_instance_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport_settings: Option<PersistedViewportSettingsV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedToolSurfaceStateV5 {
    pub id: u64,
    pub stable_surface_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_tool_surface_kind: Option<PersistedToolSurfaceKindV2>,
    pub mount: PersistedToolSurfaceMountV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport_instance_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport_settings: Option<PersistedViewportSettingsV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedViewportSettingsV1 {
    pub camera: PersistedViewportCameraSettingsV1,
    pub debug_stage: PersistedViewportDebugStageV1,
    pub root_background_opaque: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_primary_product_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_visualizer: Option<PersistedViewportFieldVisualizerSettingsV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PersistedViewportCameraSettingsV1 {
    pub orbit_target: [f32; 3],
    pub distance: f32,
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub fov_y_radians: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedViewportDebugStageV1 {
    Scene,
    ViewportCoverage,
    ViewportUvGradient,
    PrimitiveAvailability,
    PickingHitMiss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedViewportFieldVisualizerSettingsV1 {
    #[serde(default)]
    pub component: PersistedViewportFieldVisualizerComponentV1,
    #[serde(default)]
    pub slice_index: u32,
    #[serde(default)]
    pub color_ramp: PersistedViewportFieldVisualizerColorRampV1,
    #[serde(default)]
    pub debug_mode: PersistedViewportFieldVisualizerDebugModeV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedViewportFieldVisualizerComponentV1 {
    #[default]
    Auto,
    X,
    Y,
    Z,
    W,
    Magnitude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedViewportFieldVisualizerColorRampV1 {
    #[default]
    Grayscale,
    Heat,
    DivergingSigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(try_from = "String", into = "String")]
pub enum PersistedViewportFieldVisualizerDebugModeV1 {
    #[default]
    Values,
    Availability,
    Freshness,
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

fn legacy_tool_surface_kind_for_legacy_persistence(surface: &ToolSurfaceState) -> ToolSurfaceKind {
    tool_surface_kind_for_stable_key(surface.stable_surface_key())
        .expect("legacy persistence formats require legacy-compatible stable keys")
}

fn legacy_locked_tool_surface_kind_for_persistence(
    stack: &TabStackState,
) -> Option<ToolSurfaceKind> {
    stack
        .locked_stable_surface_key
        .as_ref()
        .and_then(tool_surface_kind_for_stable_key)
}

fn tab_stack_lock_from_legacy_persistence(
    legacy_kind: Option<ToolSurfaceKind>,
) -> Result<(Option<ToolSurfaceStableKey>, Option<ToolSurfaceKind>), WorkspaceStateError> {
    let Some(kind) = legacy_kind else {
        return Ok((None, None));
    };
    let stable_key = stable_key_for_tool_surface_kind(kind)
        .ok_or(crate::WorkspaceSurfaceIdentityError::UnmappedLegacySurface { kind })?;
    Ok((Some(stable_key), Some(kind)))
}

fn tool_surface_state_from_legacy_kind(
    surface_id: ToolSurfaceInstanceId,
    kind: ToolSurfaceKind,
    mount: ToolSurfaceMount,
) -> Result<ToolSurfaceState, WorkspaceStateError> {
    let stable_surface_key = stable_key_for_tool_surface_kind(kind)
        .ok_or(crate::WorkspaceSurfaceIdentityError::UnmappedLegacySurface { kind })?;
    Ok(ToolSurfaceState::new_with_stable_key(
        surface_id,
        stable_surface_key,
        mount,
    ))
}

fn persisted_v5_tab_stack_lock_identity(
    tab_stack_id: u64,
    locked_stable_surface_key: Option<String>,
    legacy_kind: Option<ToolSurfaceKind>,
    registry: Option<&ToolSurfaceRegistry>,
) -> Result<(Option<ToolSurfaceStableKey>, Option<ToolSurfaceKind>), WorkspaceStateError> {
    if legacy_kind.is_some() {
        return Err(WorkspaceStateError::PersistedSchemaViolation(
            "persisted v5 tab-stack lock legacy fallback metadata is unsupported",
        ));
    }

    let Some(stable_key) = locked_stable_surface_key else {
        return Ok((None, None));
    };
    let stable_key = ToolSurfaceStableKey::new(stable_key.clone()).map_err(|_| {
        WorkspaceStateError::PersistedTabStackLockStableKeyInvalidSyntax {
            tab_stack_id,
            stable_surface_key: stable_key,
        }
    })?;

    if registry.is_some_and(|registry| registry.get(&stable_key).is_none()) {
        return Err(WorkspaceStateError::PersistedTabStackLockStableKeyUnknown {
            tab_stack_id,
            stable_surface_key: stable_key,
        });
    }

    match tool_surface_kind_for_stable_key(&stable_key) {
        Some(legacy_kind) => Ok((Some(stable_key), Some(legacy_kind))),
        None if registry.is_some() => Ok((Some(stable_key), None)),
        None => Err(WorkspaceStateError::PersistedTabStackLockStableKeyUnknown {
            tab_stack_id,
            stable_surface_key: stable_key,
        }),
    }
}

impl WorkspaceState {
    pub fn to_persisted_v5(&self) -> Result<PersistedWorkspaceStateV5, WorkspaceStateError> {
        let mut tool_surfaces = Vec::with_capacity(self.tool_surfaces_by_id.len());

        for surface in self.tool_surfaces_by_id.values() {
            let stable_surface_key = persisted_v5_stable_surface_key_for_surface(surface)?;
            tool_surfaces.push(PersistedToolSurfaceStateV5 {
                id: surface.id.raw(),
                stable_surface_key: stable_surface_key.to_string(),
                legacy_tool_surface_kind: None,
                mount: persisted_mount(surface.mount),
                viewport_instance_id: surface.viewport_instance_id.map(|id| id.0),
                viewport_settings: surface.viewport_settings.map(persisted_viewport_settings),
            });
        }

        Ok(PersistedWorkspaceStateV5 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V5,
            workspace_profile_id: None,
            layout_template: None,
            layout_template_version: None,
            last_saved_at_unix_seconds: None,
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
                .map(|stack| PersistedTabStackStateV5 {
                    id: stack.id.raw(),
                    ordered_panels: stack.ordered_panels.iter().map(|id| id.raw()).collect(),
                    active_panel: stack.active_panel.map(|id| id.raw()),
                    locked_stable_surface_key: stack
                        .locked_stable_surface_key
                        .as_ref()
                        .map(ToString::to_string),
                    legacy_locked_tool_surface_kind: None,
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
            tool_surfaces,
        })
    }

    pub fn to_persisted_v4(&self) -> PersistedWorkspaceStateV4 {
        let persisted = self.to_persisted_v3();
        PersistedWorkspaceStateV4 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V4,
            workspace_profile_id: persisted.workspace_profile_id,
            layout_template: persisted.layout_template,
            layout_template_version: persisted.layout_template_version,
            last_saved_at_unix_seconds: persisted.last_saved_at_unix_seconds,
            workspace_id: persisted.workspace_id,
            root_host_id: persisted.root_host_id,
            hosts: persisted.hosts,
            tab_stacks: persisted.tab_stacks,
            panels: persisted.panels,
            tool_surfaces: persisted.tool_surfaces,
        }
    }

    pub fn to_persisted_v3(&self) -> PersistedWorkspaceStateV3 {
        PersistedWorkspaceStateV3 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V3,
            workspace_profile_id: None,
            layout_template: None,
            layout_template_version: None,
            last_saved_at_unix_seconds: None,
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
                    locked_tool_surface_kind: legacy_locked_tool_surface_kind_for_persistence(
                        stack,
                    )
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
                .map(|surface| {
                    let legacy_kind = legacy_tool_surface_kind_for_legacy_persistence(surface);
                    PersistedToolSurfaceStateV3 {
                        id: surface.id.raw(),
                        tool_surface_kind: persisted_tool_surface_kind_v2(legacy_kind),
                        mount: persisted_mount(surface.mount),
                        viewport_instance_id: surface.viewport_instance_id.map(|id| id.0),
                        viewport_settings: surface
                            .viewport_settings
                            .map(persisted_viewport_settings),
                    }
                })
                .collect(),
        }
    }

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
                    locked_tool_surface_kind: legacy_locked_tool_surface_kind_for_persistence(
                        stack,
                    )
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
                .map(|surface| {
                    let legacy_kind = legacy_tool_surface_kind_for_legacy_persistence(surface);
                    PersistedToolSurfaceStateV2 {
                        id: surface.id.raw(),
                        tool_surface_kind: persisted_tool_surface_kind_v2(legacy_kind),
                        mount: persisted_mount(surface.mount),
                    }
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
                .map(|surface| {
                    let legacy_kind = legacy_tool_surface_kind_for_legacy_persistence(surface);
                    PersistedToolSurfaceStateV1 {
                        id: surface.id.raw(),
                        tool_surface_kind: persisted_tool_surface_kind(legacy_kind),
                        mount: persisted_mount(surface.mount),
                    }
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
            let legacy_locked_tool_surface_kind = stack
                .locked_tool_surface_kind
                .map(workspace_tool_surface_kind_v2);
            let (locked_stable_surface_key, _legacy_locked_tool_surface_kind) =
                tab_stack_lock_from_legacy_persistence(legacy_locked_tool_surface_kind)?;
            tab_stacks_by_id.insert(
                stack_id,
                TabStackState {
                    id: stack_id,
                    ordered_panels,
                    active_panel,
                    locked_stable_surface_key,
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
                tool_surface_state_from_legacy_kind(
                    surface_id,
                    workspace_tool_surface_kind(surface.tool_surface_kind),
                    workspace_mount(surface.mount)?,
                )?,
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
            let legacy_locked_tool_surface_kind = stack
                .locked_tool_surface_kind
                .map(workspace_tool_surface_kind_v2);
            let (locked_stable_surface_key, _legacy_locked_tool_surface_kind) =
                tab_stack_lock_from_legacy_persistence(legacy_locked_tool_surface_kind)?;
            tab_stacks_by_id.insert(
                stack_id,
                TabStackState {
                    id: stack_id,
                    ordered_panels,
                    active_panel,
                    locked_stable_surface_key,
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
                tool_surface_state_from_legacy_kind(
                    surface_id,
                    workspace_tool_surface_kind_v2(surface.tool_surface_kind),
                    workspace_mount(surface.mount)?,
                )?,
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

    pub fn from_persisted_v3(
        persisted: PersistedWorkspaceStateV3,
    ) -> Result<Self, WorkspaceStateError> {
        if persisted.version != PERSISTED_WORKSPACE_STATE_VERSION_V3 {
            return Err(WorkspaceStateError::PersistedVersionUnsupported(
                persisted.version,
            ));
        }

        let viewport_instance_ids = persisted
            .tool_surfaces
            .iter()
            .filter_map(|surface| {
                surface
                    .viewport_instance_id
                    .map(|viewport_id| (surface.id, viewport_id))
            })
            .collect::<Vec<_>>();
        let viewport_settings = persisted
            .tool_surfaces
            .iter()
            .filter_map(|surface| {
                surface
                    .viewport_settings
                    .clone()
                    .map(|settings| (surface.id, settings))
            })
            .collect::<Vec<_>>();
        let mut seen_viewport_instance_ids = BTreeSet::new();
        for (_, viewport_id) in viewport_instance_ids.iter().copied() {
            if !seen_viewport_instance_ids.insert(viewport_id) {
                return Err(WorkspaceStateError::PersistedSchemaViolation(
                    "persisted viewport instance id must be unique",
                ));
            }
        }
        let mut state = Self::from_persisted_v2(PersistedWorkspaceStateV2 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V2,
            workspace_id: persisted.workspace_id,
            root_host_id: persisted.root_host_id,
            hosts: persisted.hosts,
            tab_stacks: persisted.tab_stacks,
            panels: persisted.panels,
            tool_surfaces: persisted
                .tool_surfaces
                .into_iter()
                .map(|surface| PersistedToolSurfaceStateV2 {
                    id: surface.id,
                    tool_surface_kind: surface.tool_surface_kind,
                    mount: surface.mount,
                })
                .collect(),
        })?;

        for (surface_raw, viewport_raw) in viewport_instance_ids {
            let surface_id = persisted_id!(
                ToolSurfaceInstanceId,
                surface_raw,
                "persisted viewport tool-surface id must be non-zero"
            )?;
            let viewport_id = persisted_viewport_id(viewport_raw)?;
            let surface = state
                .tool_surfaces_by_id
                .get_mut(&surface_id)
                .ok_or(WorkspaceStateError::MissingToolSurface(surface_id))?;
            if surface.stable_surface_key().as_str() != "runenwerk.scene.viewport" {
                return Err(WorkspaceStateError::PersistedSchemaViolation(
                    "persisted viewport instance id must belong to a viewport tool surface",
                ));
            }
            surface.viewport_instance_id = Some(viewport_id);
        }

        for (surface_raw, settings) in viewport_settings {
            let surface_id = persisted_id!(
                ToolSurfaceInstanceId,
                surface_raw,
                "persisted viewport settings tool-surface id must be non-zero"
            )?;
            let settings = workspace_viewport_settings(settings)?;
            let surface = state
                .tool_surfaces_by_id
                .get_mut(&surface_id)
                .ok_or(WorkspaceStateError::MissingToolSurface(surface_id))?;
            if surface.stable_surface_key().as_str() != "runenwerk.scene.viewport" {
                return Err(WorkspaceStateError::PersistedSchemaViolation(
                    "persisted viewport settings must belong to a viewport tool surface",
                ));
            }
            surface.viewport_settings = Some(settings);
        }

        state.validate_integrity()?;
        Ok(state)
    }

    pub fn from_persisted_v4(
        persisted: PersistedWorkspaceStateV4,
    ) -> Result<Self, WorkspaceStateError> {
        if persisted.version != PERSISTED_WORKSPACE_STATE_VERSION_V4 {
            return Err(WorkspaceStateError::PersistedVersionUnsupported(
                persisted.version,
            ));
        }
        Self::from_persisted_v3(PersistedWorkspaceStateV3 {
            version: PERSISTED_WORKSPACE_STATE_VERSION_V3,
            workspace_profile_id: persisted.workspace_profile_id,
            layout_template: persisted.layout_template,
            layout_template_version: persisted.layout_template_version,
            last_saved_at_unix_seconds: persisted.last_saved_at_unix_seconds,
            workspace_id: persisted.workspace_id,
            root_host_id: persisted.root_host_id,
            hosts: persisted.hosts,
            tab_stacks: persisted.tab_stacks,
            panels: persisted.panels,
            tool_surfaces: persisted.tool_surfaces,
        })
    }

    pub fn from_persisted_v5(
        persisted: PersistedWorkspaceStateV5,
        registry: Option<&ToolSurfaceRegistry>,
    ) -> Result<Self, WorkspaceStateError> {
        if persisted.version != PERSISTED_WORKSPACE_STATE_VERSION_V5 {
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
            let legacy_locked_tool_surface_kind = stack
                .legacy_locked_tool_surface_kind
                .map(workspace_tool_surface_kind_v2);
            let (locked_stable_surface_key, _legacy_locked_tool_surface_kind) =
                persisted_v5_tab_stack_lock_identity(
                    stack.id,
                    stack.locked_stable_surface_key,
                    legacy_locked_tool_surface_kind,
                    registry,
                )?;
            tab_stacks_by_id.insert(
                stack_id,
                TabStackState {
                    id: stack_id,
                    ordered_panels,
                    active_panel,
                    locked_stable_surface_key,
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
        let mut seen_viewport_instance_ids = BTreeSet::new();
        for surface in persisted.tool_surfaces {
            let surface_id = persisted_id!(
                ToolSurfaceInstanceId,
                surface.id,
                "persisted v5 stable-key tool-surface id must be non-zero"
            )?;
            let (stable_surface_key, _legacy_tool_surface_kind) =
                persisted_v5_tool_surface_identity(&surface, registry)?;
            let mut tool_surface = ToolSurfaceState::new_with_stable_key(
                surface_id,
                stable_surface_key,
                workspace_mount(surface.mount)?,
            );
            if let Some(viewport_raw) = surface.viewport_instance_id {
                if tool_surface.stable_surface_key().as_str() != "runenwerk.scene.viewport" {
                    return Err(WorkspaceStateError::PersistedSchemaViolation(
                        "persisted viewport instance id must belong to a viewport tool surface",
                    ));
                }
                let viewport_id = persisted_viewport_id(viewport_raw)?;
                if !seen_viewport_instance_ids.insert(viewport_raw) {
                    return Err(WorkspaceStateError::PersistedSchemaViolation(
                        "persisted viewport instance id must be unique",
                    ));
                }
                tool_surface.viewport_instance_id = Some(viewport_id);
            }
            if let Some(settings) = surface.viewport_settings {
                if tool_surface.stable_surface_key().as_str() != "runenwerk.scene.viewport" {
                    return Err(WorkspaceStateError::PersistedSchemaViolation(
                        "persisted viewport settings must belong to a viewport tool surface",
                    ));
                }
                tool_surface.viewport_settings = Some(workspace_viewport_settings(settings)?);
            }
            tool_surfaces_by_id.insert(surface_id, tool_surface);
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

fn persisted_v5_stable_surface_key_for_surface(
    surface: &ToolSurfaceState,
) -> Result<ToolSurfaceStableKey, WorkspaceStateError> {
    Ok(surface.stable_surface_key().clone())
}

fn persisted_v5_tool_surface_identity(
    surface: &PersistedToolSurfaceStateV5,
    registry: Option<&ToolSurfaceRegistry>,
) -> Result<(ToolSurfaceStableKey, Option<ToolSurfaceKind>), WorkspaceStateError> {
    if surface.legacy_tool_surface_kind.is_some() {
        return Err(WorkspaceStateError::PersistedSchemaViolation(
            "persisted v5 tool surface legacy fallback metadata is unsupported",
        ));
    }

    let stable_surface_key = ToolSurfaceStableKey::new(surface.stable_surface_key.clone())
        .map_err(
            |_| WorkspaceStateError::PersistedStableSurfaceKeyInvalidSyntax {
                tool_surface_id: surface.id,
                stable_surface_key: surface.stable_surface_key.clone(),
            },
        )?;

    if let Some(registry) = registry
        && registry.get(&stable_surface_key).is_none()
    {
        return Err(WorkspaceStateError::PersistedStableSurfaceKeyUnknown {
            tool_surface_id: surface.id,
            stable_surface_key,
        });
    }

    match tool_surface_kind_for_stable_key(&stable_surface_key) {
        Some(tool_surface_kind) => Ok((stable_surface_key, Some(tool_surface_kind))),
        None if registry.is_some() => Ok((stable_surface_key, None)),
        None => Err(WorkspaceStateError::PersistedStableSurfaceKeyUnknown {
            tool_surface_id: surface.id,
            stable_surface_key,
        }),
    }
}

fn persisted_viewport_id(raw: u64) -> Result<ViewportId, WorkspaceStateError> {
    if raw == 0 {
        Err(WorkspaceStateError::PersistedSchemaViolation(
            "persisted viewport instance id must be non-zero",
        ))
    } else {
        Ok(ViewportId(raw))
    }
}

fn persisted_viewport_settings(settings: ViewportRuntimeSettings) -> PersistedViewportSettingsV1 {
    PersistedViewportSettingsV1 {
        camera: PersistedViewportCameraSettingsV1 {
            orbit_target: settings.camera.orbit_target,
            distance: settings.camera.distance,
            yaw_radians: settings.camera.yaw_radians,
            pitch_radians: settings.camera.pitch_radians,
            fov_y_radians: settings.camera.fov_y_radians,
        },
        debug_stage: persisted_viewport_debug_stage(settings.debug_stage),
        root_background_opaque: settings.root_background_opaque,
        selected_primary_product_id: settings.selected_primary_product_id.map(|id| id.0),
        field_visualizer: Some(persisted_viewport_field_visualizer_settings(
            settings.field_visualizer_settings,
        )),
    }
}

fn workspace_viewport_settings(
    settings: PersistedViewportSettingsV1,
) -> Result<ViewportRuntimeSettings, WorkspaceStateError> {
    let selected_primary_product_id = settings
        .selected_primary_product_id
        .map(|product_id| {
            if product_id == 0 {
                Err(WorkspaceStateError::PersistedSchemaViolation(
                    "persisted viewport selected product id must be non-zero",
                ))
            } else {
                Ok(ExpressionProductId(product_id))
            }
        })
        .transpose()?;
    let camera = ViewportCameraSettings {
        orbit_target: settings.camera.orbit_target,
        distance: settings.camera.distance,
        yaw_radians: settings.camera.yaw_radians,
        pitch_radians: settings.camera.pitch_radians,
        fov_y_radians: settings.camera.fov_y_radians,
    };
    if !camera.is_valid() {
        return Err(WorkspaceStateError::PersistedSchemaViolation(
            "persisted viewport camera settings must be finite and positive",
        ));
    }
    Ok(ViewportRuntimeSettings {
        camera,
        debug_stage: workspace_viewport_debug_stage(settings.debug_stage),
        root_background_opaque: settings.root_background_opaque,
        selected_primary_product_id,
        field_visualizer_settings: settings
            .field_visualizer
            .map(workspace_viewport_field_visualizer_settings)
            .unwrap_or_default(),
    })
}

fn persisted_viewport_field_visualizer_settings(
    settings: ViewportFieldVisualizerSettings,
) -> PersistedViewportFieldVisualizerSettingsV1 {
    PersistedViewportFieldVisualizerSettingsV1 {
        component: persisted_viewport_field_visualizer_component(settings.component),
        slice_index: settings.slice_index,
        color_ramp: persisted_viewport_field_visualizer_color_ramp(settings.color_ramp),
        debug_mode: persisted_viewport_field_visualizer_debug_mode(settings.debug_mode),
    }
}

fn workspace_viewport_field_visualizer_settings(
    settings: PersistedViewportFieldVisualizerSettingsV1,
) -> ViewportFieldVisualizerSettings {
    ViewportFieldVisualizerSettings {
        component: workspace_viewport_field_visualizer_component(settings.component),
        slice_index: settings.slice_index,
        color_ramp: workspace_viewport_field_visualizer_color_ramp(settings.color_ramp),
        debug_mode: workspace_viewport_field_visualizer_debug_mode(settings.debug_mode),
    }
}

fn persisted_viewport_field_visualizer_component(
    component: ViewportFieldVisualizerComponent,
) -> PersistedViewportFieldVisualizerComponentV1 {
    match component {
        ViewportFieldVisualizerComponent::Auto => PersistedViewportFieldVisualizerComponentV1::Auto,
        ViewportFieldVisualizerComponent::X => PersistedViewportFieldVisualizerComponentV1::X,
        ViewportFieldVisualizerComponent::Y => PersistedViewportFieldVisualizerComponentV1::Y,
        ViewportFieldVisualizerComponent::Z => PersistedViewportFieldVisualizerComponentV1::Z,
        ViewportFieldVisualizerComponent::W => PersistedViewportFieldVisualizerComponentV1::W,
        ViewportFieldVisualizerComponent::Magnitude => {
            PersistedViewportFieldVisualizerComponentV1::Magnitude
        }
    }
}

fn workspace_viewport_field_visualizer_component(
    component: PersistedViewportFieldVisualizerComponentV1,
) -> ViewportFieldVisualizerComponent {
    match component {
        PersistedViewportFieldVisualizerComponentV1::Auto => ViewportFieldVisualizerComponent::Auto,
        PersistedViewportFieldVisualizerComponentV1::X => ViewportFieldVisualizerComponent::X,
        PersistedViewportFieldVisualizerComponentV1::Y => ViewportFieldVisualizerComponent::Y,
        PersistedViewportFieldVisualizerComponentV1::Z => ViewportFieldVisualizerComponent::Z,
        PersistedViewportFieldVisualizerComponentV1::W => ViewportFieldVisualizerComponent::W,
        PersistedViewportFieldVisualizerComponentV1::Magnitude => {
            ViewportFieldVisualizerComponent::Magnitude
        }
    }
}

fn persisted_viewport_field_visualizer_color_ramp(
    color_ramp: ViewportFieldVisualizerColorRamp,
) -> PersistedViewportFieldVisualizerColorRampV1 {
    match color_ramp {
        ViewportFieldVisualizerColorRamp::Grayscale => {
            PersistedViewportFieldVisualizerColorRampV1::Grayscale
        }
        ViewportFieldVisualizerColorRamp::Heat => PersistedViewportFieldVisualizerColorRampV1::Heat,
        ViewportFieldVisualizerColorRamp::DivergingSigned => {
            PersistedViewportFieldVisualizerColorRampV1::DivergingSigned
        }
    }
}

fn workspace_viewport_field_visualizer_color_ramp(
    color_ramp: PersistedViewportFieldVisualizerColorRampV1,
) -> ViewportFieldVisualizerColorRamp {
    match color_ramp {
        PersistedViewportFieldVisualizerColorRampV1::Grayscale => {
            ViewportFieldVisualizerColorRamp::Grayscale
        }
        PersistedViewportFieldVisualizerColorRampV1::Heat => ViewportFieldVisualizerColorRamp::Heat,
        PersistedViewportFieldVisualizerColorRampV1::DivergingSigned => {
            ViewportFieldVisualizerColorRamp::DivergingSigned
        }
    }
}

fn persisted_viewport_field_visualizer_debug_mode(
    debug_mode: ViewportFieldVisualizerDebugMode,
) -> PersistedViewportFieldVisualizerDebugModeV1 {
    match debug_mode {
        ViewportFieldVisualizerDebugMode::Values => {
            PersistedViewportFieldVisualizerDebugModeV1::Values
        }
        ViewportFieldVisualizerDebugMode::Availability => {
            PersistedViewportFieldVisualizerDebugModeV1::Availability
        }
        ViewportFieldVisualizerDebugMode::Freshness => {
            PersistedViewportFieldVisualizerDebugModeV1::Freshness
        }
    }
}

fn workspace_viewport_field_visualizer_debug_mode(
    debug_mode: PersistedViewportFieldVisualizerDebugModeV1,
) -> ViewportFieldVisualizerDebugMode {
    match debug_mode {
        PersistedViewportFieldVisualizerDebugModeV1::Values => {
            ViewportFieldVisualizerDebugMode::Values
        }
        PersistedViewportFieldVisualizerDebugModeV1::Availability => {
            ViewportFieldVisualizerDebugMode::Availability
        }
        PersistedViewportFieldVisualizerDebugModeV1::Freshness => {
            ViewportFieldVisualizerDebugMode::Freshness
        }
    }
}

fn persisted_viewport_debug_stage(stage: ViewportDebugStage) -> PersistedViewportDebugStageV1 {
    match stage {
        ViewportDebugStage::Scene => PersistedViewportDebugStageV1::Scene,
        ViewportDebugStage::ViewportCoverage => PersistedViewportDebugStageV1::ViewportCoverage,
        ViewportDebugStage::ViewportUvGradient => PersistedViewportDebugStageV1::ViewportUvGradient,
        ViewportDebugStage::PrimitiveAvailability => {
            PersistedViewportDebugStageV1::PrimitiveAvailability
        }
        ViewportDebugStage::PickingHitMiss => PersistedViewportDebugStageV1::PickingHitMiss,
    }
}

fn workspace_viewport_debug_stage(stage: PersistedViewportDebugStageV1) -> ViewportDebugStage {
    match stage {
        PersistedViewportDebugStageV1::Scene => ViewportDebugStage::Scene,
        PersistedViewportDebugStageV1::ViewportCoverage => ViewportDebugStage::ViewportCoverage,
        PersistedViewportDebugStageV1::ViewportUvGradient => ViewportDebugStage::ViewportUvGradient,
        PersistedViewportDebugStageV1::PrimitiveAvailability => {
            ViewportDebugStage::PrimitiveAvailability
        }
        PersistedViewportDebugStageV1::PickingHitMiss => ViewportDebugStage::PickingHitMiss,
    }
}

impl From<PersistedViewportDebugStageV1> for String {
    fn from(value: PersistedViewportDebugStageV1) -> Self {
        match value {
            PersistedViewportDebugStageV1::Scene => "scene".to_string(),
            PersistedViewportDebugStageV1::ViewportCoverage => "viewport_coverage".to_string(),
            PersistedViewportDebugStageV1::ViewportUvGradient => "viewport_uv_gradient".to_string(),
            PersistedViewportDebugStageV1::PrimitiveAvailability => {
                "primitive_availability".to_string()
            }
            PersistedViewportDebugStageV1::PickingHitMiss => "picking_hit_miss".to_string(),
        }
    }
}

impl TryFrom<String> for PersistedViewportDebugStageV1 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "scene" => Ok(Self::Scene),
            "viewport_coverage" => Ok(Self::ViewportCoverage),
            "viewport_uv_gradient" => Ok(Self::ViewportUvGradient),
            "primitive_availability" => Ok(Self::PrimitiveAvailability),
            "picking_hit_miss" => Ok(Self::PickingHitMiss),
            other => Err(format!("unsupported viewport debug stage: {other}")),
        }
    }
}

impl From<PersistedViewportFieldVisualizerComponentV1> for String {
    fn from(value: PersistedViewportFieldVisualizerComponentV1) -> Self {
        match value {
            PersistedViewportFieldVisualizerComponentV1::Auto => "auto".to_string(),
            PersistedViewportFieldVisualizerComponentV1::X => "x".to_string(),
            PersistedViewportFieldVisualizerComponentV1::Y => "y".to_string(),
            PersistedViewportFieldVisualizerComponentV1::Z => "z".to_string(),
            PersistedViewportFieldVisualizerComponentV1::W => "w".to_string(),
            PersistedViewportFieldVisualizerComponentV1::Magnitude => "magnitude".to_string(),
        }
    }
}

impl TryFrom<String> for PersistedViewportFieldVisualizerComponentV1 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "auto" => Ok(Self::Auto),
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            "z" => Ok(Self::Z),
            "w" => Ok(Self::W),
            "magnitude" => Ok(Self::Magnitude),
            other => Err(format!(
                "unsupported viewport field visualizer component: {other}"
            )),
        }
    }
}

impl From<PersistedViewportFieldVisualizerColorRampV1> for String {
    fn from(value: PersistedViewportFieldVisualizerColorRampV1) -> Self {
        match value {
            PersistedViewportFieldVisualizerColorRampV1::Grayscale => "grayscale".to_string(),
            PersistedViewportFieldVisualizerColorRampV1::Heat => "heat".to_string(),
            PersistedViewportFieldVisualizerColorRampV1::DivergingSigned => {
                "diverging_signed".to_string()
            }
        }
    }
}

impl TryFrom<String> for PersistedViewportFieldVisualizerColorRampV1 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "grayscale" => Ok(Self::Grayscale),
            "heat" => Ok(Self::Heat),
            "diverging_signed" => Ok(Self::DivergingSigned),
            other => Err(format!(
                "unsupported viewport field visualizer color ramp: {other}"
            )),
        }
    }
}

impl From<PersistedViewportFieldVisualizerDebugModeV1> for String {
    fn from(value: PersistedViewportFieldVisualizerDebugModeV1) -> Self {
        match value {
            PersistedViewportFieldVisualizerDebugModeV1::Values => "values".to_string(),
            PersistedViewportFieldVisualizerDebugModeV1::Availability => "availability".to_string(),
            PersistedViewportFieldVisualizerDebugModeV1::Freshness => "freshness".to_string(),
        }
    }
}

impl TryFrom<String> for PersistedViewportFieldVisualizerDebugModeV1 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "values" => Ok(Self::Values),
            "availability" => Ok(Self::Availability),
            "freshness" => Ok(Self::Freshness),
            other => Err(format!(
                "unsupported viewport field visualizer debug mode: {other}"
            )),
        }
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
        | PanelKind::CommandDiff
        | PanelKind::AssetBrowser
        | PanelKind::ImportInspector
        | PanelKind::FieldProductViewer
        | PanelKind::SdfBrushBrowser
        | PanelKind::GraphCanvas
        | PanelKind::Diagnostics
        | PanelKind::RuntimeDebug
        | PanelKind::FieldLayerStack
        | PanelKind::SdfGraphCanvas
        | PanelKind::MaterialGraphCanvas
        | PanelKind::MaterialInspector
        | PanelKind::MaterialPreview
        | PanelKind::TextureViewer
        | PanelKind::VolumeTextureViewer
        | PanelKind::ProcgenGraphCanvas
        | PanelKind::ProcgenPreview
        | PanelKind::GameplayGraphCanvas
        | PanelKind::GameplayCompilerDiagnostics
        | PanelKind::ParticleGraphCanvas
        | PanelKind::ParticlePreview
        | PanelKind::PhysicsAuthoring
        | PanelKind::PhysicsDebug
        | PanelKind::Timeline
        | PanelKind::CurveEditor
        | PanelKind::AnimationGraphCanvas
        | PanelKind::SimulationPreview
        | PanelKind::SimulationDiagnostics => PersistedPanelKindV1::Placeholder,
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
        PanelKind::AssetBrowser => PersistedPanelKindV2::AssetBrowser,
        PanelKind::ImportInspector => PersistedPanelKindV2::ImportInspector,
        PanelKind::FieldProductViewer => PersistedPanelKindV2::FieldProductViewer,
        PanelKind::SdfBrushBrowser => PersistedPanelKindV2::SdfBrushBrowser,
        PanelKind::GraphCanvas => PersistedPanelKindV2::GraphCanvas,
        PanelKind::Diagnostics => PersistedPanelKindV2::Diagnostics,
        PanelKind::RuntimeDebug => PersistedPanelKindV2::RuntimeDebug,
        PanelKind::FieldLayerStack => PersistedPanelKindV2::FieldLayerStack,
        PanelKind::SdfGraphCanvas => PersistedPanelKindV2::SdfGraphCanvas,
        PanelKind::MaterialGraphCanvas => PersistedPanelKindV2::MaterialGraphCanvas,
        PanelKind::MaterialInspector => PersistedPanelKindV2::MaterialInspector,
        PanelKind::MaterialPreview => PersistedPanelKindV2::MaterialPreview,
        PanelKind::TextureViewer => PersistedPanelKindV2::TextureViewer,
        PanelKind::VolumeTextureViewer => PersistedPanelKindV2::VolumeTextureViewer,
        PanelKind::ProcgenGraphCanvas => PersistedPanelKindV2::ProcgenGraphCanvas,
        PanelKind::ProcgenPreview => PersistedPanelKindV2::ProcgenPreview,
        PanelKind::GameplayGraphCanvas => PersistedPanelKindV2::GameplayGraphCanvas,
        PanelKind::GameplayCompilerDiagnostics => PersistedPanelKindV2::GameplayCompilerDiagnostics,
        PanelKind::ParticleGraphCanvas => PersistedPanelKindV2::ParticleGraphCanvas,
        PanelKind::ParticlePreview => PersistedPanelKindV2::ParticlePreview,
        PanelKind::PhysicsAuthoring => PersistedPanelKindV2::PhysicsAuthoring,
        PanelKind::PhysicsDebug => PersistedPanelKindV2::PhysicsDebug,
        PanelKind::Timeline => PersistedPanelKindV2::Timeline,
        PanelKind::CurveEditor => PersistedPanelKindV2::CurveEditor,
        PanelKind::AnimationGraphCanvas => PersistedPanelKindV2::AnimationGraphCanvas,
        PanelKind::SimulationPreview => PersistedPanelKindV2::SimulationPreview,
        PanelKind::SimulationDiagnostics => PersistedPanelKindV2::SimulationDiagnostics,
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
        PersistedPanelKindV2::AssetBrowser => PanelKind::AssetBrowser,
        PersistedPanelKindV2::ImportInspector => PanelKind::ImportInspector,
        PersistedPanelKindV2::FieldProductViewer => PanelKind::FieldProductViewer,
        PersistedPanelKindV2::SdfBrushBrowser => PanelKind::SdfBrushBrowser,
        PersistedPanelKindV2::GraphCanvas => PanelKind::GraphCanvas,
        PersistedPanelKindV2::Diagnostics => PanelKind::Diagnostics,
        PersistedPanelKindV2::RuntimeDebug => PanelKind::RuntimeDebug,
        PersistedPanelKindV2::FieldLayerStack => PanelKind::FieldLayerStack,
        PersistedPanelKindV2::SdfGraphCanvas => PanelKind::SdfGraphCanvas,
        PersistedPanelKindV2::MaterialGraphCanvas => PanelKind::MaterialGraphCanvas,
        PersistedPanelKindV2::MaterialInspector => PanelKind::MaterialInspector,
        PersistedPanelKindV2::MaterialPreview => PanelKind::MaterialPreview,
        PersistedPanelKindV2::TextureViewer => PanelKind::TextureViewer,
        PersistedPanelKindV2::VolumeTextureViewer => PanelKind::VolumeTextureViewer,
        PersistedPanelKindV2::ProcgenGraphCanvas => PanelKind::ProcgenGraphCanvas,
        PersistedPanelKindV2::ProcgenPreview => PanelKind::ProcgenPreview,
        PersistedPanelKindV2::GameplayGraphCanvas => PanelKind::GameplayGraphCanvas,
        PersistedPanelKindV2::GameplayCompilerDiagnostics => PanelKind::GameplayCompilerDiagnostics,
        PersistedPanelKindV2::ParticleGraphCanvas => PanelKind::ParticleGraphCanvas,
        PersistedPanelKindV2::ParticlePreview => PanelKind::ParticlePreview,
        PersistedPanelKindV2::PhysicsAuthoring => PanelKind::PhysicsAuthoring,
        PersistedPanelKindV2::PhysicsDebug => PanelKind::PhysicsDebug,
        PersistedPanelKindV2::Timeline => PanelKind::Timeline,
        PersistedPanelKindV2::CurveEditor => PanelKind::CurveEditor,
        PersistedPanelKindV2::AnimationGraphCanvas => PanelKind::AnimationGraphCanvas,
        PersistedPanelKindV2::SimulationPreview => PanelKind::SimulationPreview,
        PersistedPanelKindV2::SimulationDiagnostics => PanelKind::SimulationDiagnostics,
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
            PersistedPanelKindV2::AssetBrowser => "asset_browser".to_string(),
            PersistedPanelKindV2::ImportInspector => "import_inspector".to_string(),
            PersistedPanelKindV2::FieldProductViewer => "field_product_viewer".to_string(),
            PersistedPanelKindV2::SdfBrushBrowser => "sdf_brush_browser".to_string(),
            PersistedPanelKindV2::GraphCanvas => "graph_canvas".to_string(),
            PersistedPanelKindV2::Diagnostics => "diagnostics".to_string(),
            PersistedPanelKindV2::RuntimeDebug => "runtime_debug".to_string(),
            PersistedPanelKindV2::FieldLayerStack => "field_layer_stack".to_string(),
            PersistedPanelKindV2::SdfGraphCanvas => "sdf_graph_canvas".to_string(),
            PersistedPanelKindV2::MaterialGraphCanvas => "material_graph_canvas".to_string(),
            PersistedPanelKindV2::MaterialInspector => "material_inspector".to_string(),
            PersistedPanelKindV2::MaterialPreview => "material_preview".to_string(),
            PersistedPanelKindV2::TextureViewer => "texture_viewer".to_string(),
            PersistedPanelKindV2::VolumeTextureViewer => "volume_texture_viewer".to_string(),
            PersistedPanelKindV2::ProcgenGraphCanvas => "procgen_graph_canvas".to_string(),
            PersistedPanelKindV2::ProcgenPreview => "procgen_preview".to_string(),
            PersistedPanelKindV2::GameplayGraphCanvas => "gameplay_graph_canvas".to_string(),
            PersistedPanelKindV2::GameplayCompilerDiagnostics => {
                "gameplay_compiler_diagnostics".to_string()
            }
            PersistedPanelKindV2::ParticleGraphCanvas => "particle_graph_canvas".to_string(),
            PersistedPanelKindV2::ParticlePreview => "particle_preview".to_string(),
            PersistedPanelKindV2::PhysicsAuthoring => "physics_authoring".to_string(),
            PersistedPanelKindV2::PhysicsDebug => "physics_debug".to_string(),
            PersistedPanelKindV2::Timeline => "timeline".to_string(),
            PersistedPanelKindV2::CurveEditor => "curve_editor".to_string(),
            PersistedPanelKindV2::AnimationGraphCanvas => "animation_graph_canvas".to_string(),
            PersistedPanelKindV2::SimulationPreview => "simulation_preview".to_string(),
            PersistedPanelKindV2::SimulationDiagnostics => "simulation_diagnostics".to_string(),
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
            "asset_browser" => Ok(Self::AssetBrowser),
            "import_inspector" => Ok(Self::ImportInspector),
            "field_product_viewer" => Ok(Self::FieldProductViewer),
            "sdf_brush_browser" => Ok(Self::SdfBrushBrowser),
            "graph_canvas" => Ok(Self::GraphCanvas),
            "diagnostics" => Ok(Self::Diagnostics),
            "runtime_debug" => Ok(Self::RuntimeDebug),
            "field_layer_stack" => Ok(Self::FieldLayerStack),
            "sdf_graph_canvas" => Ok(Self::SdfGraphCanvas),
            "material_graph_canvas" => Ok(Self::MaterialGraphCanvas),
            "material_inspector" => Ok(Self::MaterialInspector),
            "material_preview" => Ok(Self::MaterialPreview),
            "texture_viewer" => Ok(Self::TextureViewer),
            "volume_texture_viewer" => Ok(Self::VolumeTextureViewer),
            "procgen_graph_canvas" => Ok(Self::ProcgenGraphCanvas),
            "procgen_preview" => Ok(Self::ProcgenPreview),
            "gameplay_graph_canvas" => Ok(Self::GameplayGraphCanvas),
            "gameplay_compiler_diagnostics" => Ok(Self::GameplayCompilerDiagnostics),
            "particle_graph_canvas" => Ok(Self::ParticleGraphCanvas),
            "particle_preview" => Ok(Self::ParticlePreview),
            "physics_authoring" => Ok(Self::PhysicsAuthoring),
            "physics_debug" => Ok(Self::PhysicsDebug),
            "timeline" => Ok(Self::Timeline),
            "curve_editor" => Ok(Self::CurveEditor),
            "animation_graph_canvas" => Ok(Self::AnimationGraphCanvas),
            "simulation_preview" => Ok(Self::SimulationPreview),
            "simulation_diagnostics" => Ok(Self::SimulationDiagnostics),
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
        | ToolSurfaceKind::CommandDiff
        | ToolSurfaceKind::AssetBrowser
        | ToolSurfaceKind::ImportInspector
        | ToolSurfaceKind::FieldProductViewer
        | ToolSurfaceKind::SdfBrushBrowser
        | ToolSurfaceKind::GraphCanvas
        | ToolSurfaceKind::Diagnostics
        | ToolSurfaceKind::RuntimeDebug
        | ToolSurfaceKind::FieldLayerStack
        | ToolSurfaceKind::SdfGraphCanvas
        | ToolSurfaceKind::MaterialGraphCanvas
        | ToolSurfaceKind::MaterialInspector
        | ToolSurfaceKind::MaterialPreview
        | ToolSurfaceKind::TextureViewer
        | ToolSurfaceKind::VolumeTextureViewer
        | ToolSurfaceKind::ProcgenGraphCanvas
        | ToolSurfaceKind::ProcgenPreview
        | ToolSurfaceKind::GameplayGraphCanvas
        | ToolSurfaceKind::GameplayCompilerDiagnostics
        | ToolSurfaceKind::ParticleGraphCanvas
        | ToolSurfaceKind::ParticlePreview
        | ToolSurfaceKind::PhysicsAuthoring
        | ToolSurfaceKind::PhysicsDebug
        | ToolSurfaceKind::Timeline
        | ToolSurfaceKind::CurveEditor
        | ToolSurfaceKind::AnimationGraphCanvas
        | ToolSurfaceKind::SimulationPreview
        | ToolSurfaceKind::SimulationDiagnostics => PersistedToolSurfaceKindV1::Placeholder,
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
        ToolSurfaceKind::AssetBrowser => PersistedToolSurfaceKindV2::AssetBrowser,
        ToolSurfaceKind::ImportInspector => PersistedToolSurfaceKindV2::ImportInspector,
        ToolSurfaceKind::FieldProductViewer => PersistedToolSurfaceKindV2::FieldProductViewer,
        ToolSurfaceKind::SdfBrushBrowser => PersistedToolSurfaceKindV2::SdfBrushBrowser,
        ToolSurfaceKind::GraphCanvas => PersistedToolSurfaceKindV2::GraphCanvas,
        ToolSurfaceKind::Diagnostics => PersistedToolSurfaceKindV2::Diagnostics,
        ToolSurfaceKind::RuntimeDebug => PersistedToolSurfaceKindV2::RuntimeDebug,
        ToolSurfaceKind::FieldLayerStack => PersistedToolSurfaceKindV2::FieldLayerStack,
        ToolSurfaceKind::SdfGraphCanvas => PersistedToolSurfaceKindV2::SdfGraphCanvas,
        ToolSurfaceKind::MaterialGraphCanvas => PersistedToolSurfaceKindV2::MaterialGraphCanvas,
        ToolSurfaceKind::MaterialInspector => PersistedToolSurfaceKindV2::MaterialInspector,
        ToolSurfaceKind::MaterialPreview => PersistedToolSurfaceKindV2::MaterialPreview,
        ToolSurfaceKind::TextureViewer => PersistedToolSurfaceKindV2::TextureViewer,
        ToolSurfaceKind::VolumeTextureViewer => PersistedToolSurfaceKindV2::VolumeTextureViewer,
        ToolSurfaceKind::ProcgenGraphCanvas => PersistedToolSurfaceKindV2::ProcgenGraphCanvas,
        ToolSurfaceKind::ProcgenPreview => PersistedToolSurfaceKindV2::ProcgenPreview,
        ToolSurfaceKind::GameplayGraphCanvas => PersistedToolSurfaceKindV2::GameplayGraphCanvas,
        ToolSurfaceKind::GameplayCompilerDiagnostics => {
            PersistedToolSurfaceKindV2::GameplayCompilerDiagnostics
        }
        ToolSurfaceKind::ParticleGraphCanvas => PersistedToolSurfaceKindV2::ParticleGraphCanvas,
        ToolSurfaceKind::ParticlePreview => PersistedToolSurfaceKindV2::ParticlePreview,
        ToolSurfaceKind::PhysicsAuthoring => PersistedToolSurfaceKindV2::PhysicsAuthoring,
        ToolSurfaceKind::PhysicsDebug => PersistedToolSurfaceKindV2::PhysicsDebug,
        ToolSurfaceKind::Timeline => PersistedToolSurfaceKindV2::Timeline,
        ToolSurfaceKind::CurveEditor => PersistedToolSurfaceKindV2::CurveEditor,
        ToolSurfaceKind::AnimationGraphCanvas => PersistedToolSurfaceKindV2::AnimationGraphCanvas,
        ToolSurfaceKind::SimulationPreview => PersistedToolSurfaceKindV2::SimulationPreview,
        ToolSurfaceKind::SimulationDiagnostics => PersistedToolSurfaceKindV2::SimulationDiagnostics,
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
        PersistedToolSurfaceKindV2::AssetBrowser => ToolSurfaceKind::AssetBrowser,
        PersistedToolSurfaceKindV2::ImportInspector => ToolSurfaceKind::ImportInspector,
        PersistedToolSurfaceKindV2::FieldProductViewer => ToolSurfaceKind::FieldProductViewer,
        PersistedToolSurfaceKindV2::SdfBrushBrowser => ToolSurfaceKind::SdfBrushBrowser,
        PersistedToolSurfaceKindV2::GraphCanvas => ToolSurfaceKind::GraphCanvas,
        PersistedToolSurfaceKindV2::Diagnostics => ToolSurfaceKind::Diagnostics,
        PersistedToolSurfaceKindV2::RuntimeDebug => ToolSurfaceKind::RuntimeDebug,
        PersistedToolSurfaceKindV2::FieldLayerStack => ToolSurfaceKind::FieldLayerStack,
        PersistedToolSurfaceKindV2::SdfGraphCanvas => ToolSurfaceKind::SdfGraphCanvas,
        PersistedToolSurfaceKindV2::MaterialGraphCanvas => ToolSurfaceKind::MaterialGraphCanvas,
        PersistedToolSurfaceKindV2::MaterialInspector => ToolSurfaceKind::MaterialInspector,
        PersistedToolSurfaceKindV2::MaterialPreview => ToolSurfaceKind::MaterialPreview,
        PersistedToolSurfaceKindV2::TextureViewer => ToolSurfaceKind::TextureViewer,
        PersistedToolSurfaceKindV2::VolumeTextureViewer => ToolSurfaceKind::VolumeTextureViewer,
        PersistedToolSurfaceKindV2::ProcgenGraphCanvas => ToolSurfaceKind::ProcgenGraphCanvas,
        PersistedToolSurfaceKindV2::ProcgenPreview => ToolSurfaceKind::ProcgenPreview,
        PersistedToolSurfaceKindV2::GameplayGraphCanvas => ToolSurfaceKind::GameplayGraphCanvas,
        PersistedToolSurfaceKindV2::GameplayCompilerDiagnostics => {
            ToolSurfaceKind::GameplayCompilerDiagnostics
        }
        PersistedToolSurfaceKindV2::ParticleGraphCanvas => ToolSurfaceKind::ParticleGraphCanvas,
        PersistedToolSurfaceKindV2::ParticlePreview => ToolSurfaceKind::ParticlePreview,
        PersistedToolSurfaceKindV2::PhysicsAuthoring => ToolSurfaceKind::PhysicsAuthoring,
        PersistedToolSurfaceKindV2::PhysicsDebug => ToolSurfaceKind::PhysicsDebug,
        PersistedToolSurfaceKindV2::Timeline => ToolSurfaceKind::Timeline,
        PersistedToolSurfaceKindV2::CurveEditor => ToolSurfaceKind::CurveEditor,
        PersistedToolSurfaceKindV2::AnimationGraphCanvas => ToolSurfaceKind::AnimationGraphCanvas,
        PersistedToolSurfaceKindV2::SimulationPreview => ToolSurfaceKind::SimulationPreview,
        PersistedToolSurfaceKindV2::SimulationDiagnostics => ToolSurfaceKind::SimulationDiagnostics,
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
            PersistedToolSurfaceKindV2::AssetBrowser => "asset_browser".to_string(),
            PersistedToolSurfaceKindV2::ImportInspector => "import_inspector".to_string(),
            PersistedToolSurfaceKindV2::FieldProductViewer => "field_product_viewer".to_string(),
            PersistedToolSurfaceKindV2::SdfBrushBrowser => "sdf_brush_browser".to_string(),
            PersistedToolSurfaceKindV2::GraphCanvas => "graph_canvas".to_string(),
            PersistedToolSurfaceKindV2::Diagnostics => "diagnostics".to_string(),
            PersistedToolSurfaceKindV2::RuntimeDebug => "runtime_debug".to_string(),
            PersistedToolSurfaceKindV2::FieldLayerStack => "field_layer_stack".to_string(),
            PersistedToolSurfaceKindV2::SdfGraphCanvas => "sdf_graph_canvas".to_string(),
            PersistedToolSurfaceKindV2::MaterialGraphCanvas => "material_graph_canvas".to_string(),
            PersistedToolSurfaceKindV2::MaterialInspector => "material_inspector".to_string(),
            PersistedToolSurfaceKindV2::MaterialPreview => "material_preview".to_string(),
            PersistedToolSurfaceKindV2::TextureViewer => "texture_viewer".to_string(),
            PersistedToolSurfaceKindV2::VolumeTextureViewer => "volume_texture_viewer".to_string(),
            PersistedToolSurfaceKindV2::ProcgenGraphCanvas => "procgen_graph_canvas".to_string(),
            PersistedToolSurfaceKindV2::ProcgenPreview => "procgen_preview".to_string(),
            PersistedToolSurfaceKindV2::GameplayGraphCanvas => "gameplay_graph_canvas".to_string(),
            PersistedToolSurfaceKindV2::GameplayCompilerDiagnostics => {
                "gameplay_compiler_diagnostics".to_string()
            }
            PersistedToolSurfaceKindV2::ParticleGraphCanvas => "particle_graph_canvas".to_string(),
            PersistedToolSurfaceKindV2::ParticlePreview => "particle_preview".to_string(),
            PersistedToolSurfaceKindV2::PhysicsAuthoring => "physics_authoring".to_string(),
            PersistedToolSurfaceKindV2::PhysicsDebug => "physics_debug".to_string(),
            PersistedToolSurfaceKindV2::Timeline => "timeline".to_string(),
            PersistedToolSurfaceKindV2::CurveEditor => "curve_editor".to_string(),
            PersistedToolSurfaceKindV2::AnimationGraphCanvas => {
                "animation_graph_canvas".to_string()
            }
            PersistedToolSurfaceKindV2::SimulationPreview => "simulation_preview".to_string(),
            PersistedToolSurfaceKindV2::SimulationDiagnostics => {
                "simulation_diagnostics".to_string()
            }
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
            "asset_browser" => Ok(Self::AssetBrowser),
            "import_inspector" => Ok(Self::ImportInspector),
            "field_product_viewer" => Ok(Self::FieldProductViewer),
            "sdf_brush_browser" => Ok(Self::SdfBrushBrowser),
            "graph_canvas" => Ok(Self::GraphCanvas),
            "diagnostics" => Ok(Self::Diagnostics),
            "runtime_debug" => Ok(Self::RuntimeDebug),
            "field_layer_stack" => Ok(Self::FieldLayerStack),
            "sdf_graph_canvas" => Ok(Self::SdfGraphCanvas),
            "material_graph_canvas" => Ok(Self::MaterialGraphCanvas),
            "material_inspector" => Ok(Self::MaterialInspector),
            "material_preview" => Ok(Self::MaterialPreview),
            "texture_viewer" => Ok(Self::TextureViewer),
            "volume_texture_viewer" => Ok(Self::VolumeTextureViewer),
            "procgen_graph_canvas" => Ok(Self::ProcgenGraphCanvas),
            "procgen_preview" => Ok(Self::ProcgenPreview),
            "gameplay_graph_canvas" => Ok(Self::GameplayGraphCanvas),
            "gameplay_compiler_diagnostics" => Ok(Self::GameplayCompilerDiagnostics),
            "particle_graph_canvas" => Ok(Self::ParticleGraphCanvas),
            "particle_preview" => Ok(Self::ParticlePreview),
            "physics_authoring" => Ok(Self::PhysicsAuthoring),
            "physics_debug" => Ok(Self::PhysicsDebug),
            "timeline" => Ok(Self::Timeline),
            "curve_editor" => Ok(Self::CurveEditor),
            "animation_graph_canvas" => Ok(Self::AnimationGraphCanvas),
            "simulation_preview" => Ok(Self::SimulationPreview),
            "simulation_diagnostics" => Ok(Self::SimulationDiagnostics),
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
    use crate::{
        EditorToolSuite, ProviderFamilyDefinition, ProviderFamilyId, ToolSuiteId,
        ToolSuiteRegistry, ToolSurfaceDefinition, ToolSurfacePersistence, ToolSurfaceRole,
        ToolSurfaceRoute, WorkspaceIdentityAllocator, WorkspaceMutation,
        default_workspace_profile_registry, reduce_workspace,
        saveable_tool_surface_stable_key_candidates,
    };

    fn bootstrap_workspace() -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator)
    }

    fn material_lab_workspace() -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let root_host_id = allocator.allocate_panel_host_id();
        let tab_stack_id = allocator.allocate_tab_stack_id();
        let mut ordered_panels = Vec::new();
        let mut panels_by_id = BTreeMap::new();
        let mut tool_surfaces_by_id = BTreeMap::new();

        for tool_surface_kind in [
            ToolSurfaceKind::MaterialGraphCanvas,
            ToolSurfaceKind::MaterialInspector,
            ToolSurfaceKind::MaterialPreview,
        ] {
            let panel_id = allocator.allocate_panel_instance_id();
            let surface_id = allocator.allocate_tool_surface_instance_id();
            ordered_panels.push(panel_id);
            panels_by_id.insert(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind: tool_surface_kind.panel_kind(),
                    active_tool_surface: Some(surface_id),
                },
            );
            tool_surfaces_by_id.insert(
                surface_id,
                tool_surface_state_from_legacy_kind(
                    surface_id,
                    tool_surface_kind,
                    ToolSurfaceMount::Mounted { panel_id },
                )
                .expect("material lab fixture surfaces should have stable keys"),
            );
        }

        let hosts_by_id = BTreeMap::from([(
            root_host_id,
            PanelHostNode {
                id: root_host_id,
                kind: PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }),
            },
        )]);
        let tab_stacks_by_id = BTreeMap::from([(
            tab_stack_id,
            TabStackState {
                id: tab_stack_id,
                ordered_panels,
                active_panel: panels_by_id.keys().next().copied(),
                locked_stable_surface_key: stable_key_for_tool_surface_kind(
                    ToolSurfaceKind::MaterialGraphCanvas,
                ),
            },
        )]);

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
            .expect("material lab fixture should be valid");
        workspace
    }

    fn workspace_with_tool_surfaces(surface_kinds: &[ToolSurfaceKind]) -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let root_host_id = allocator.allocate_panel_host_id();
        let tab_stack_id = allocator.allocate_tab_stack_id();
        let mut ordered_panels = Vec::new();
        let mut panels_by_id = BTreeMap::new();
        let mut tool_surfaces_by_id = BTreeMap::new();

        for &tool_surface_kind in surface_kinds {
            let panel_id = allocator.allocate_panel_instance_id();
            let surface_id = allocator.allocate_tool_surface_instance_id();
            ordered_panels.push(panel_id);
            panels_by_id.insert(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind: tool_surface_kind.panel_kind(),
                    active_tool_surface: Some(surface_id),
                },
            );
            tool_surfaces_by_id.insert(
                surface_id,
                tool_surface_state_from_legacy_kind(
                    surface_id,
                    tool_surface_kind,
                    ToolSurfaceMount::Mounted { panel_id },
                )
                .expect("test fixture surfaces should have stable keys"),
            );
        }

        let hosts_by_id = BTreeMap::from([(
            root_host_id,
            PanelHostNode {
                id: root_host_id,
                kind: PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }),
            },
        )]);
        let tab_stacks_by_id = BTreeMap::from([(
            tab_stack_id,
            TabStackState {
                id: tab_stack_id,
                ordered_panels,
                active_panel: panels_by_id.keys().next().copied(),
                locked_stable_surface_key: None,
            },
        )]);

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
            .expect("tool-surface fixture should be valid");
        workspace
    }

    fn stable_key_only_locked_test_workspace() -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let root_host_id = allocator.allocate_panel_host_id();
        let tab_stack_id = allocator.allocate_tab_stack_id();
        let panel_id = allocator.allocate_panel_instance_id();
        let surface_id = allocator.allocate_tool_surface_instance_id();
        let stable_surface_key = ToolSurfaceStableKey::new("runenwerk.test.stable_only_surface")
            .expect("test stable key should be valid");

        let workspace = WorkspaceState {
            workspace_id,
            root_host_id,
            hosts_by_id: BTreeMap::from([(
                root_host_id,
                PanelHostNode {
                    id: root_host_id,
                    kind: PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }),
                },
            )]),
            tab_stacks_by_id: BTreeMap::from([(
                tab_stack_id,
                TabStackState {
                    id: tab_stack_id,
                    ordered_panels: vec![panel_id],
                    active_panel: Some(panel_id),
                    locked_stable_surface_key: Some(stable_surface_key.clone()),
                },
            )]),
            panels_by_id: BTreeMap::from([(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind: PanelKind::Diagnostics,
                    active_tool_surface: Some(surface_id),
                },
            )]),
            tool_surfaces_by_id: BTreeMap::from([(
                surface_id,
                ToolSurfaceState::new_with_stable_key(
                    surface_id,
                    stable_surface_key,
                    ToolSurfaceMount::Mounted { panel_id },
                ),
            )]),
        };
        workspace
            .validate_integrity()
            .expect("stable-key-only test fixture should be valid");
        workspace
    }

    fn viewport_surface_id(workspace: &WorkspaceState) -> ToolSurfaceInstanceId {
        let viewport_key = stable_key_for_tool_surface_kind(ToolSurfaceKind::Viewport)
            .expect("viewport should have a stable key");
        workspace
            .tool_surfaces_by_id
            .values()
            .find(|surface| surface.stable_surface_key() == &viewport_key)
            .map(|surface| surface.id)
            .expect("workspace should contain a viewport surface")
    }

    fn default_profile_tool_surface_registry() -> ToolSuiteRegistry {
        let provider_family_id = ProviderFamilyId::new("runenwerk.test").unwrap();
        let mut stable_keys = saveable_tool_surface_stable_key_candidates()
            .iter()
            .map(|candidate| candidate.stable_key.to_string())
            .collect::<Vec<_>>();
        stable_keys.push("runenwerk.test.stable_only_surface".to_string());
        for profile in default_workspace_profile_registry().profiles() {
            for surface in &profile.default_surfaces {
                let key = surface.stable_surface_key().as_str();
                if !stable_keys.iter().any(|known| known == key) {
                    stable_keys.push(key.to_string());
                }
            }
        }
        ToolSuiteRegistry::new(vec![EditorToolSuite {
            suite_id: ToolSuiteId::new("runenwerk.test").unwrap(),
            label: "Test".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family_id.clone(),
                label: "Test".to_string(),
            }],
            surfaces: stable_keys
                .iter()
                .map(|key| {
                    material_lab_surface(
                        key,
                        key,
                        ToolSurfaceRole::Primary,
                        provider_family_id.clone(),
                        ToolSurfaceRoute::ProviderOwnedLocal,
                    )
                })
                .collect(),
        }])
        .expect("default profile tool-surface registry fixture should be valid")
    }

    fn material_lab_registry() -> ToolSuiteRegistry {
        let suite_id = ToolSuiteId::new("runenwerk.material_lab").unwrap();
        let provider_family_id = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        ToolSuiteRegistry::new(vec![EditorToolSuite {
            suite_id,
            label: "Material Lab".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family_id.clone(),
                label: "Material Lab".to_string(),
            }],
            surfaces: vec![
                material_lab_surface(
                    "runenwerk.material_lab.graph_canvas",
                    "Material Graph",
                    ToolSurfaceRole::Primary,
                    provider_family_id.clone(),
                    ToolSurfaceRoute::ProviderOwnedGraphCanvas,
                ),
                material_lab_surface(
                    "runenwerk.material_lab.inspector",
                    "Material Inspector",
                    ToolSurfaceRole::Inspector,
                    provider_family_id.clone(),
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
                material_lab_surface(
                    "runenwerk.material_lab.preview",
                    "Material Preview",
                    ToolSurfaceRole::Preview,
                    provider_family_id,
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
            ],
        }])
        .expect("material lab test registry should be valid")
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

    fn test_viewport_settings() -> ViewportRuntimeSettings {
        ViewportRuntimeSettings {
            camera: ViewportCameraSettings {
                orbit_target: [1.0, 2.0, 3.0],
                distance: 12.0,
                yaw_radians: 0.25,
                pitch_radians: -0.15,
                fov_y_radians: 60.0_f32.to_radians(),
            },
            debug_stage: ViewportDebugStage::PrimitiveAvailability,
            root_background_opaque: true,
            selected_primary_product_id: Some(ExpressionProductId(2)),
            field_visualizer_settings: ViewportFieldVisualizerSettings::default()
                .with_component(ViewportFieldVisualizerComponent::Magnitude)
                .with_slice_index(8)
                .with_color_ramp(ViewportFieldVisualizerColorRamp::Heat)
                .with_debug_mode(ViewportFieldVisualizerDebugMode::Freshness),
        }
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
            WorkspaceMutation::LockTabStackAreaStableKey {
                tab_stack_id: viewport_stack,
                locked_stable_surface_key: stable_key_for_tool_surface_kind(
                    ToolSurfaceKind::Viewport,
                ),
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
    fn persisted_v3_roundtrip_preserves_viewport_instance_restore_ids() {
        let mut workspace = bootstrap_workspace();
        let viewport_surface = viewport_surface_id(&workspace);
        workspace
            .tool_surfaces_by_id
            .get_mut(&viewport_surface)
            .expect("viewport surface should exist")
            .viewport_instance_id = Some(ViewportId(77));

        let persisted = workspace.to_persisted_v3();
        assert_eq!(
            persisted
                .tool_surfaces
                .iter()
                .find(|surface| surface.id == viewport_surface.raw())
                .and_then(|surface| surface.viewport_instance_id),
            Some(77)
        );
        let restored =
            WorkspaceState::from_persisted_v3(persisted).expect("v3 state should decode");

        assert_eq!(workspace, restored);
    }

    #[test]
    fn persisted_v3_roundtrip_preserves_viewport_runtime_settings() {
        let mut workspace = bootstrap_workspace();
        let viewport_surface = viewport_surface_id(&workspace);
        workspace
            .tool_surfaces_by_id
            .get_mut(&viewport_surface)
            .expect("viewport surface should exist")
            .viewport_settings = Some(test_viewport_settings());

        let persisted = workspace.to_persisted_v3();
        assert_eq!(
            persisted
                .tool_surfaces
                .iter()
                .find(|surface| surface.id == viewport_surface.raw())
                .and_then(|surface| surface.viewport_settings.as_ref())
                .map(|settings| settings.debug_stage),
            Some(PersistedViewportDebugStageV1::PrimitiveAvailability)
        );
        assert_eq!(
            persisted
                .tool_surfaces
                .iter()
                .find(|surface| surface.id == viewport_surface.raw())
                .and_then(|surface| surface.viewport_settings.as_ref())
                .and_then(|settings| settings.field_visualizer)
                .map(|settings| (
                    settings.component,
                    settings.slice_index,
                    settings.color_ramp,
                    settings.debug_mode
                )),
            Some((
                PersistedViewportFieldVisualizerComponentV1::Magnitude,
                8,
                PersistedViewportFieldVisualizerColorRampV1::Heat,
                PersistedViewportFieldVisualizerDebugModeV1::Freshness
            ))
        );
        let restored =
            WorkspaceState::from_persisted_v3(persisted).expect("v3 state should decode");

        assert_eq!(workspace, restored);
    }

    #[test]
    fn persisted_v3_defaults_legacy_viewport_field_visualizer_settings() {
        let mut workspace = bootstrap_workspace();
        let viewport_surface = viewport_surface_id(&workspace);
        workspace
            .tool_surfaces_by_id
            .get_mut(&viewport_surface)
            .expect("viewport surface should exist")
            .viewport_settings = Some(test_viewport_settings());

        let mut persisted = workspace.to_persisted_v3();
        let settings = persisted
            .tool_surfaces
            .iter_mut()
            .find(|surface| surface.id == viewport_surface.raw())
            .and_then(|surface| surface.viewport_settings.as_mut())
            .expect("persisted viewport settings should exist");
        settings.field_visualizer = None;

        let restored =
            WorkspaceState::from_persisted_v3(persisted).expect("legacy v3 settings should decode");

        assert_eq!(
            restored
                .tool_surface(viewport_surface)
                .and_then(|surface| surface.viewport_settings)
                .map(|settings| settings.field_visualizer_settings),
            Some(ViewportFieldVisualizerSettings::default())
        );
    }

    #[test]
    fn persisted_v3_decode_rejects_invalid_viewport_settings_product_id() {
        let workspace = bootstrap_workspace();
        let mut persisted = workspace.to_persisted_v3();
        let surface = persisted
            .tool_surfaces
            .iter_mut()
            .find(|surface| surface.tool_surface_kind == PersistedToolSurfaceKindV2::Viewport)
            .expect("persisted workspace should contain a viewport surface");
        let mut settings = persisted_viewport_settings(test_viewport_settings());
        settings.selected_primary_product_id = Some(0);
        surface.viewport_settings = Some(settings);

        let error = WorkspaceState::from_persisted_v3(persisted)
            .expect_err("zero selected product ids must fail decode");

        assert!(matches!(
            error,
            WorkspaceStateError::PersistedSchemaViolation(_)
        ));
    }

    #[test]
    fn duplicated_viewport_area_copies_runtime_settings_not_restore_identity() {
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
        let viewport_surface = viewport_surface_id(&workspace);
        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::SetToolSurfaceViewportSettings {
                tool_surface_id: viewport_surface,
                viewport_settings: Some(test_viewport_settings()),
            },
        )
        .expect("viewport settings mutation should be valid");
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let new_panel_id = allocator.allocate_panel_instance_id();
        let new_tool_surface_id = allocator.allocate_tool_surface_instance_id();

        let duplicated = reduce_workspace(
            &workspace,
            WorkspaceMutation::DuplicateTabStackArea {
                tab_stack_id: viewport_stack,
                new_panel_id,
                new_tool_surface_id,
            },
        )
        .expect("duplicating a viewport area should be valid");
        let duplicated_surface = duplicated
            .tool_surface(new_tool_surface_id)
            .expect("duplicate surface should exist");

        assert_eq!(
            duplicated_surface.viewport_settings,
            Some(test_viewport_settings())
        );
        assert_eq!(duplicated_surface.viewport_instance_id, None);
    }

    #[test]
    fn persisted_v3_decode_rejects_zero_viewport_instance_id() {
        let workspace = bootstrap_workspace();
        let mut persisted = workspace.to_persisted_v3();
        persisted
            .tool_surfaces
            .iter_mut()
            .find(|surface| surface.tool_surface_kind == PersistedToolSurfaceKindV2::Viewport)
            .expect("persisted workspace should contain a viewport surface")
            .viewport_instance_id = Some(0);

        let error = WorkspaceState::from_persisted_v3(persisted)
            .expect_err("zero viewport ids must fail decode");

        assert!(matches!(
            error,
            WorkspaceStateError::PersistedSchemaViolation(_)
        ));
    }

    #[test]
    fn persisted_v5_round_trip_writes_stable_keys_as_primary_identity() {
        let workspace = material_lab_workspace();
        let registry = material_lab_registry();

        let persisted = workspace
            .to_persisted_v5()
            .expect("material lab surfaces should have stable keys");

        assert_eq!(persisted.version, PERSISTED_WORKSPACE_STATE_VERSION_V5);
        assert_eq!(
            persisted
                .tool_surfaces
                .iter()
                .map(|surface| surface.stable_surface_key.as_str())
                .collect::<Vec<_>>(),
            vec![
                "runenwerk.material_lab.graph_canvas",
                "runenwerk.material_lab.inspector",
                "runenwerk.material_lab.preview"
            ]
        );

        let restored = WorkspaceState::from_persisted_v5(persisted, Some(registry.surfaces()))
            .expect("v5 material lab workspace should load");

        assert_eq!(workspace, restored);
    }

    #[test]
    fn v5_round_trip_preserves_stable_key_only_tab_stack_lock() {
        let workspace = stable_key_only_locked_test_workspace();
        let registry = default_profile_tool_surface_registry();
        let expected_lock = workspace
            .tab_stacks()
            .next()
            .and_then(|stack| stack.locked_stable_surface_key.clone())
            .expect("fixture should be locked by stable key");

        let persisted = workspace
            .to_persisted_v5()
            .expect("stable-key-only lock should persist");
        let restored = WorkspaceState::from_persisted_v5(persisted, Some(registry.surfaces()))
            .expect("stable-key-only test lock should restore with registry");

        let restored_stack = restored
            .tab_stacks()
            .next()
            .expect("restored fixture should contain a tab stack");
        assert_eq!(
            restored_stack.locked_stable_surface_key.as_ref(),
            Some(&expected_lock)
        );
        assert_eq!(workspace, restored);
    }

    #[test]
    fn v5_lock_writes_stable_key_as_primary_identity() {
        let workspace = stable_key_only_locked_test_workspace();
        let persisted = workspace
            .to_persisted_v5()
            .expect("stable-key-only lock should persist");

        assert_eq!(
            persisted.tab_stacks[0].locked_stable_surface_key.as_deref(),
            Some("runenwerk.test.stable_only_surface")
        );
        assert_eq!(
            persisted.tab_stacks[0].legacy_locked_tool_surface_kind,
            None
        );

        let serialized = ron::ser::to_string_pretty(
            &persisted,
            ron::ser::PrettyConfig::new().struct_names(false),
        )
        .expect("v5 workspace should serialize");

        assert!(serialized.contains("locked_stable_surface_key"));
        assert!(!serialized.contains("locked_tool_surface_kind"));
    }

    #[test]
    fn v5_lock_legacy_kind_is_metadata_only() {
        let workspace = material_lab_workspace();
        let registry = material_lab_registry();
        let mut persisted = workspace
            .to_persisted_v5()
            .expect("material lab lock should persist");
        persisted.tab_stacks[0].legacy_locked_tool_surface_kind = None;

        let restored = WorkspaceState::from_persisted_v5(persisted, Some(registry.surfaces()))
            .expect("stable lock should restore without legacy metadata");

        assert_eq!(
            restored
                .tab_stacks()
                .next()
                .and_then(|stack| stack.locked_stable_surface_key.as_ref())
                .map(|key| key.as_str()),
            Some("runenwerk.material_lab.graph_canvas")
        );
    }

    #[test]
    fn v5_lock_key_legacy_kind_mismatch_fails_closed() {
        let workspace = material_lab_workspace();
        let mut persisted = workspace
            .to_persisted_v5()
            .expect("material lab lock should persist");
        persisted.tab_stacks[0].legacy_locked_tool_surface_kind =
            Some(PersistedToolSurfaceKindV2::MaterialPreview);

        let error = WorkspaceState::from_persisted_v5(persisted, None)
            .expect_err("legacy lock metadata must fail");

        assert!(matches!(
            error,
            WorkspaceStateError::PersistedSchemaViolation(_)
        ));
    }

    #[test]
    fn v1_to_v4_locked_kind_migrates_to_stable_key_lock() {
        let workspace = bootstrap_workspace();

        let mut v1 = workspace.to_persisted_v1();
        v1.tab_stacks[0].locked_tool_surface_kind = Some(PersistedToolSurfaceKindV2::Viewport);
        let restored_v1 = WorkspaceState::from_persisted_v1(v1).expect("v1 lock should decode");

        let mut v2 = workspace.to_persisted_v2();
        v2.tab_stacks[0].locked_tool_surface_kind = Some(PersistedToolSurfaceKindV2::Viewport);
        let restored_v2 = WorkspaceState::from_persisted_v2(v2).expect("v2 lock should decode");

        let mut v3 = workspace.to_persisted_v3();
        v3.tab_stacks[0].locked_tool_surface_kind = Some(PersistedToolSurfaceKindV2::Viewport);
        let restored_v3 = WorkspaceState::from_persisted_v3(v3).expect("v3 lock should decode");

        let mut v4 = workspace.to_persisted_v4();
        v4.tab_stacks[0].locked_tool_surface_kind = Some(PersistedToolSurfaceKindV2::Viewport);
        let restored_v4 = WorkspaceState::from_persisted_v4(v4).expect("v4 lock should decode");

        for restored in [restored_v1, restored_v2, restored_v3, restored_v4] {
            let stack = restored
                .tab_stacks()
                .next()
                .expect("restored workspace should contain a tab stack");
            assert_eq!(
                stack
                    .locked_stable_surface_key
                    .as_ref()
                    .map(|key| key.as_str()),
                Some("runenwerk.scene.viewport")
            );
        }
    }

    #[test]
    fn persisted_v5_does_not_write_tool_surface_kind_as_primary_identity() {
        let workspace = material_lab_workspace();
        let persisted = workspace
            .to_persisted_v5()
            .expect("material lab surfaces should have stable keys");

        let serialized = ron::ser::to_string_pretty(
            &persisted,
            ron::ser::PrettyConfig::new().struct_names(false),
        )
        .expect("v5 workspace should serialize");

        assert!(serialized.contains("stable_surface_key"));
        assert!(!serialized.contains("legacy_tool_surface_kind"));
        assert!(
            !serialized
                .lines()
                .map(str::trim_start)
                .any(|line| line.starts_with("tool_surface_kind:")),
            "V5 tool surfaces must not serialize legacy tool_surface_kind as primary identity"
        );
    }

    #[test]
    fn persisted_v5_material_lab_surface_loads_with_stable_key_metadata() {
        let workspace = material_lab_workspace();
        let persisted = workspace
            .to_persisted_v5()
            .expect("material lab surfaces should have stable keys");

        let restored = WorkspaceState::from_persisted_v5(persisted, None)
            .expect("v5 material lab workspace should load through stable metadata");

        assert_eq!(workspace, restored);
    }

    #[test]
    fn v5_load_creates_stable_key_authoritative_tool_surface_state() {
        let workspace = material_lab_workspace();
        let persisted = workspace
            .to_persisted_v5()
            .expect("material lab surfaces should have stable keys");

        let restored = WorkspaceState::from_persisted_v5(persisted, None)
            .expect("v5 material lab workspace should load through stable metadata");
        let surface = restored
            .tool_surfaces()
            .find(|surface| {
                surface.stable_surface_key().as_str() == "runenwerk.material_lab.graph_canvas"
            })
            .expect("material graph canvas should restore");

        assert_eq!(
            surface.stable_surface_key().as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
    }

    #[test]
    fn persisted_v5_rejects_invalid_stable_key_syntax() {
        let workspace = material_lab_workspace();
        let mut persisted = workspace
            .to_persisted_v5()
            .expect("material lab surfaces should have stable keys");
        persisted.tool_surfaces[0].stable_surface_key =
            "Runenwerk.material_lab.graph_canvas".to_string();

        let error = WorkspaceState::from_persisted_v5(persisted, None)
            .expect_err("invalid stable keys must fail closed");

        assert!(matches!(
            error,
            WorkspaceStateError::PersistedStableSurfaceKeyInvalidSyntax { .. }
        ));
    }

    #[test]
    fn persisted_v5_rejects_unknown_stable_key_without_legacy_metadata() {
        let workspace = material_lab_workspace();
        let mut persisted = workspace
            .to_persisted_v5()
            .expect("material lab surfaces should have stable keys");
        persisted.tool_surfaces[0].stable_surface_key =
            "runenwerk.unknown.graph_canvas".to_string();
        persisted.tool_surfaces[0].legacy_tool_surface_kind = None;

        let error = WorkspaceState::from_persisted_v5(persisted, None)
            .expect_err("unknown stable keys without legacy metadata must fail closed");

        assert!(matches!(
            error,
            WorkspaceStateError::PersistedStableSurfaceKeyUnknown { .. }
        ));
    }

    #[test]
    fn persisted_v5_rejects_stable_key_legacy_kind_mismatch() {
        let workspace = material_lab_workspace();
        let mut persisted = workspace
            .to_persisted_v5()
            .expect("material lab surfaces should have stable keys");
        persisted.tool_surfaces[0].legacy_tool_surface_kind =
            Some(PersistedToolSurfaceKindV2::MaterialPreview);

        let error = WorkspaceState::from_persisted_v5(persisted, None)
            .expect_err("legacy metadata must fail");

        assert!(matches!(
            error,
            WorkspaceStateError::PersistedSchemaViolation(_)
        ));
    }

    #[test]
    fn persisted_v4_load_populates_material_lab_stable_key_metadata() {
        let workspace = material_lab_workspace();
        let persisted = workspace.to_persisted_v4();

        let restored = WorkspaceState::from_persisted_v4(persisted)
            .expect("v4 material lab workspace should load");

        assert_eq!(
            restored
                .tool_surfaces()
                .map(|surface| surface.stable_surface_key().as_str())
                .collect::<Vec<_>>(),
            vec![
                "runenwerk.material_lab.graph_canvas",
                "runenwerk.material_lab.inspector",
                "runenwerk.material_lab.preview"
            ]
        );
    }

    #[test]
    fn persisted_v1_to_v4_legacy_loads_still_work() {
        let workspace = bootstrap_workspace();

        assert!(
            WorkspaceState::from_persisted_v1(workspace.to_persisted_v1())
                .expect("v1 should load")
                .validate_integrity()
                .is_ok()
        );
        assert!(
            WorkspaceState::from_persisted_v2(workspace.to_persisted_v2())
                .expect("v2 should load")
                .validate_integrity()
                .is_ok()
        );
        assert!(
            WorkspaceState::from_persisted_v3(workspace.to_persisted_v3())
                .expect("v3 should load")
                .validate_integrity()
                .is_ok()
        );
        assert!(
            WorkspaceState::from_persisted_v4(workspace.to_persisted_v4())
                .expect("v4 should load")
                .validate_integrity()
                .is_ok()
        );
    }

    #[test]
    fn v1_to_v4_loads_create_stable_key_authoritative_tool_surface_state() {
        let workspace = material_lab_workspace();

        for restored in [
            WorkspaceState::from_persisted_v1(workspace.to_persisted_v1()).expect("v1 should load"),
            WorkspaceState::from_persisted_v2(workspace.to_persisted_v2()).expect("v2 should load"),
            WorkspaceState::from_persisted_v3(workspace.to_persisted_v3()).expect("v3 should load"),
            WorkspaceState::from_persisted_v4(workspace.to_persisted_v4()).expect("v4 should load"),
        ] {
            assert!(
                restored
                    .tool_surfaces()
                    .all(|surface| !surface.stable_surface_key().as_str().is_empty())
            );
        }
    }

    #[test]
    fn persisted_v5_writes_placeholder_with_explicit_fallback_key() {
        let mut workspace = material_lab_workspace();
        let surface = workspace
            .tool_surfaces_by_id
            .values_mut()
            .next()
            .expect("fixture should contain a surface");
        *surface = tool_surface_state_from_legacy_kind(
            surface.id,
            ToolSurfaceKind::Placeholder,
            surface.mount,
        )
        .expect("placeholder should have explicit fallback key");

        let persisted = workspace
            .to_persisted_v5()
            .expect("placeholder should persist through explicit fallback key");

        assert!(
            persisted
                .tool_surfaces
                .iter()
                .any(|surface| surface.stable_surface_key == "runenwerk.diagnostics.placeholder")
        );
    }

    #[test]
    fn persisted_v5_write_covers_all_saveable_tool_surface_kinds() {
        for candidate in saveable_tool_surface_stable_key_candidates() {
            let workspace = workspace_with_tool_surfaces(&[candidate.kind]);
            let persisted = workspace
                .to_persisted_v5()
                .unwrap_or_else(|error| panic!("failed to persist {:?}: {error}", candidate.kind));

            assert_eq!(persisted.tool_surfaces.len(), 1);
            assert_eq!(
                persisted.tool_surfaces[0].stable_surface_key,
                candidate.stable_key
            );
            assert_eq!(persisted.tool_surfaces[0].legacy_tool_surface_kind, None);
        }
    }

    #[test]
    fn persisted_v5_default_profiles_round_trip_preserves_layout_identity_and_tab_order() {
        let profile_registry = default_workspace_profile_registry();
        let tool_surface_registry = default_profile_tool_surface_registry();

        for profile in profile_registry.profiles() {
            let mut allocator = WorkspaceIdentityAllocator::new();
            let workspace_id = allocator.allocate_workspace_id();
            let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);
            let before_tab_order = workspace
                .tab_stacks()
                .map(|stack| (stack.id, stack.ordered_panels.clone()))
                .collect::<Vec<_>>();

            let persisted = workspace.to_persisted_v5().unwrap_or_else(|error| {
                panic!("failed to persist profile {}: {error}", profile.label)
            });
            let restored = WorkspaceState::from_persisted_v5(
                persisted,
                Some(tool_surface_registry.surfaces()),
            )
            .unwrap_or_else(|error| panic!("failed to restore profile {}: {error}", profile.label));
            let after_tab_order = restored
                .tab_stacks()
                .map(|stack| (stack.id, stack.ordered_panels.clone()))
                .collect::<Vec<_>>();

            assert_eq!(restored, workspace);
            assert_eq!(after_tab_order, before_tab_order);
        }
    }

    #[test]
    fn persisted_v5_round_trip_preserves_layout_identity_and_tab_order() {
        let workspace = material_lab_workspace();
        let before_tab_order = workspace
            .tab_stacks()
            .map(|stack| stack.ordered_panels.clone())
            .collect::<Vec<_>>();

        let persisted = workspace
            .to_persisted_v5()
            .expect("material lab surfaces should have stable keys");
        let restored = WorkspaceState::from_persisted_v5(persisted, None)
            .expect("v5 material lab workspace should load");
        let after_tab_order = restored
            .tab_stacks()
            .map(|stack| stack.ordered_panels.clone())
            .collect::<Vec<_>>();

        assert_eq!(before_tab_order, after_tab_order);
        assert_eq!(workspace.root_host_id(), restored.root_host_id());
        assert_eq!(workspace, restored);
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
                .all(|stack| stack.locked_stable_surface_key.is_none())
        );
    }
}
