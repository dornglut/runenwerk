//! File: domain/editor/editor_shell/src/workspace/profile.rs
//! Purpose: Workspace profile contracts for task-focused editor layout presets.

use std::fmt;

use editor_core::{
    DocumentKind, EDIT_MODE_ID, ModeId, PLAY_MODE_ID, PREVIEW_MODE_ID, SIMULATE_MODE_ID,
};
use editor_definition::EditorWorkspaceLayoutDefinition;
use id_macros::id;

use crate::{
    PanelHostKind, PanelKind, ToolSurfaceKind, WorkspaceId, WorkspaceIdentityAllocator,
    WorkspaceSplitAxis,
    tool_suite::{ProfileRef, ToolSurfaceRegistry, ToolSurfaceStableKey},
};

use super::definition_form::{
    WorkspaceDefinitionFormationError, form_workspace_state_from_definition_with_registry,
};
use super::state::{
    WorkspaceDefaultToolSurface, WorkspaceState, WorkspaceStateError,
    WorkspaceSurfaceIdentityError, WorkspaceToolSurfaceRegistryCompatibilityReport,
};

#[id]
pub struct WorkspaceProfileId;

pub const SCENE_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(1);
pub const MODELLING_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(2);
pub const EDITOR_DESIGN_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(3);
pub const FIELD_WORLD_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(4);
pub const MATERIAL_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(5);
pub const TEXTURE_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(6);
pub const PROCGEN_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(7);
pub const GAMEPLAY_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(8);
pub const PARTICLE_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(9);
pub const PHYSICS_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(10);
pub const ANIMATION_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(11);
pub const SIMULATION_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(12);
pub const RUNTIME_DEBUG_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(13);
pub const GRAPH_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(14);
pub const LAYOUT_WORKSPACE_PROFILE_ID: WorkspaceProfileId = SCENE_WORKSPACE_PROFILE_ID;
const TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY: &str =
    "runenwerk.diagnostics.tool_suite_registry_inspector";

const fn workspace_profile_id(raw: u64) -> WorkspaceProfileId {
    match WorkspaceProfileId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("workspace profile id constants must be non-zero"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceLayoutTemplate {
    Scene,
    Modelling,
    EditorDesign,
    ToolWorkspace,
    CurrentFixedEditor,
}

impl WorkspaceLayoutTemplate {
    pub const fn contract_id(self) -> &'static str {
        match self {
            Self::Scene => "scene",
            Self::Modelling => "modelling",
            Self::EditorDesign => "editor-design",
            Self::ToolWorkspace => "tool-workspace",
            Self::CurrentFixedEditor => "current-fixed-editor",
        }
    }

    pub const fn contract_version(self) -> u32 {
        match self {
            Self::Scene | Self::Modelling | Self::ToolWorkspace | Self::CurrentFixedEditor => 1,
            Self::EditorDesign => 1,
        }
    }

    pub fn build_workspace_state(
        self,
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
    ) -> WorkspaceState {
        match self {
            Self::Scene | Self::CurrentFixedEditor => {
                WorkspaceState::bootstrap_current_layout(workspace_id, allocator)
            }
            Self::Modelling => WorkspaceState::bootstrap_modelling_layout(workspace_id, allocator),
            Self::EditorDesign => {
                WorkspaceState::bootstrap_editor_design_layout(workspace_id, allocator)
            }
            Self::ToolWorkspace => {
                WorkspaceState::bootstrap_tool_workspace_layout(workspace_id, allocator, &[])
            }
        }
    }

    pub fn default_graph_matches(self, workspace_state: &WorkspaceState) -> bool {
        match self {
            Self::Scene | Self::CurrentFixedEditor => {
                scene_derived_default_graph_matches(workspace_state)
            }
            Self::Modelling => modelling_default_graph_matches(workspace_state),
            Self::EditorDesign => workspace_state.validate_integrity().is_ok(),
            Self::ToolWorkspace => workspace_state.validate_integrity().is_ok(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceProfileLayoutSource {
    Template(WorkspaceLayoutTemplate),
    AuthoredLayout {
        layout_ref: String,
        layout: EditorWorkspaceLayoutDefinition,
    },
}

impl WorkspaceProfileLayoutSource {
    pub const fn template(&self) -> WorkspaceLayoutTemplate {
        match self {
            Self::Template(template) => *template,
            Self::AuthoredLayout { .. } => WorkspaceLayoutTemplate::ToolWorkspace,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceProfile {
    pub id: WorkspaceProfileId,
    pub profile_ref: ProfileRef,
    pub label: String,
    pub default_layout_template: WorkspaceLayoutTemplate,
    pub layout_source: WorkspaceProfileLayoutSource,
    pub default_surfaces: Vec<WorkspaceDefaultToolSurface>,
    pub default_modes: Vec<ModeId>,
    pub document_kind_filters: Vec<DocumentKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceProfileToolSurfaceCompatibilityReport {
    pub compatible_surfaces: Vec<WorkspaceProfileToolSurfaceCompatibleSurface>,
    pub unregistered_legacy_surfaces: Vec<WorkspaceProfileToolSurfaceLegacySurface>,
    pub unmapped_legacy_surfaces: Vec<WorkspaceProfileToolSurfaceUnmappedLegacySurface>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProfileToolSurfaceCompatibleSurface {
    pub legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    pub stable_surface_key: ToolSurfaceStableKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProfileToolSurfaceLegacySurface {
    pub legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    pub stable_surface_key: ToolSurfaceStableKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProfileToolSurfaceUnmappedLegacySurface {
    pub tool_surface_kind: ToolSurfaceKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceProfileRegistryBackedBuildError {
    UnknownWorkspaceProfile {
        profile_id: WorkspaceProfileId,
    },
    UnregisteredDefaultToolSurface {
        profile_id: WorkspaceProfileId,
        legacy_tool_surface_kind: Option<ToolSurfaceKind>,
        stable_surface_key: ToolSurfaceStableKey,
    },
    UnmappedDefaultToolSurface {
        profile_id: WorkspaceProfileId,
        tool_surface_kind: ToolSurfaceKind,
    },
    WorkspaceCompatibility {
        profile_id: WorkspaceProfileId,
        report: Box<WorkspaceToolSurfaceRegistryCompatibilityReport>,
    },
    WorkspaceState {
        profile_id: WorkspaceProfileId,
        error: Box<WorkspaceStateError>,
    },
    WorkspaceDefinitionFormation {
        profile_id: WorkspaceProfileId,
        error: Box<WorkspaceDefinitionFormationError>,
    },
}

impl fmt::Display for WorkspaceProfileRegistryBackedBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownWorkspaceProfile { profile_id } => write!(
                f,
                "workspace profile {} is not registered",
                profile_id.raw()
            ),
            Self::UnregisteredDefaultToolSurface {
                profile_id,
                legacy_tool_surface_kind,
                stable_surface_key,
            } => match legacy_tool_surface_kind {
                Some(kind) => write!(
                    f,
                    "workspace profile {} references {kind:?} with unregistered stable key `{stable_surface_key}`",
                    profile_id.raw()
                ),
                None => write!(
                    f,
                    "workspace profile {} references unregistered stable key `{stable_surface_key}` without legacy compatibility metadata",
                    profile_id.raw()
                ),
            },
            Self::UnmappedDefaultToolSurface {
                profile_id,
                tool_surface_kind,
            } => write!(
                f,
                "workspace profile {} references {tool_surface_kind:?} without a safe stable-key mapping",
                profile_id.raw()
            ),
            Self::WorkspaceCompatibility { profile_id, .. } => write!(
                f,
                "workspace profile {} produced a workspace that is not compatible with the tool-surface registry",
                profile_id.raw()
            ),
            Self::WorkspaceState { profile_id, error } => write!(
                f,
                "workspace profile {} failed to build workspace state: {error}",
                profile_id.raw()
            ),
            Self::WorkspaceDefinitionFormation { profile_id, error } => write!(
                f,
                "workspace profile {} failed to form authored workspace layout: {error:?}",
                profile_id.raw()
            ),
        }
    }
}

impl std::error::Error for WorkspaceProfileRegistryBackedBuildError {}

impl WorkspaceProfile {
    pub fn new(
        id: WorkspaceProfileId,
        label: impl Into<String>,
        default_layout_template: WorkspaceLayoutTemplate,
        default_surfaces: Vec<WorkspaceDefaultToolSurface>,
        default_modes: Vec<ModeId>,
        document_kind_filters: Vec<DocumentKind>,
    ) -> Self {
        Self::new_with_profile_ref(
            workspace_profile_ref_for_id(id),
            id,
            label,
            WorkspaceProfileLayoutSource::Template(default_layout_template),
            default_surfaces,
            default_modes,
            document_kind_filters,
        )
    }

    pub fn new_with_profile_ref(
        profile_ref: ProfileRef,
        id: WorkspaceProfileId,
        label: impl Into<String>,
        layout_source: WorkspaceProfileLayoutSource,
        default_surfaces: Vec<WorkspaceDefaultToolSurface>,
        default_modes: Vec<ModeId>,
        document_kind_filters: Vec<DocumentKind>,
    ) -> Self {
        let default_layout_template = layout_source.template();
        Self {
            id,
            profile_ref,
            label: label.into(),
            default_layout_template,
            layout_source,
            default_surfaces,
            default_modes,
            document_kind_filters,
        }
    }

    pub fn new_legacy(
        id: WorkspaceProfileId,
        label: impl Into<String>,
        default_layout_template: WorkspaceLayoutTemplate,
        default_tool_surfaces: Vec<ToolSurfaceKind>,
        default_modes: Vec<ModeId>,
        document_kind_filters: Vec<DocumentKind>,
    ) -> Result<Self, WorkspaceSurfaceIdentityError> {
        let default_surfaces = default_tool_surfaces
            .into_iter()
            .map(stable_default_surface_for_kind)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(
            id,
            label,
            default_layout_template,
            default_surfaces,
            default_modes,
            document_kind_filters,
        ))
    }

    pub fn build_default_workspace_state(
        &self,
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
    ) -> WorkspaceState {
        self.try_build_default_workspace_state(workspace_id, allocator)
            .expect("compiled-in workspace profile default surfaces should keep C3 legacy metadata")
    }

    pub fn try_build_default_workspace_state(
        &self,
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
    ) -> Result<WorkspaceState, WorkspaceStateError> {
        if self.default_layout_template == WorkspaceLayoutTemplate::ToolWorkspace {
            return WorkspaceState::bootstrap_tool_workspace_layout_with_stable_surfaces(
                workspace_id,
                allocator,
                &self.default_surfaces,
            );
        }
        Ok(self
            .default_layout_template
            .build_workspace_state(workspace_id, allocator))
    }

    pub fn build_default_workspace_state_with_registry(
        &self,
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
        registry: &ToolSurfaceRegistry,
    ) -> Result<WorkspaceState, WorkspaceProfileRegistryBackedBuildError> {
        self.require_tool_surface_registry_compatibility(registry)?;
        let workspace = match &self.layout_source {
            WorkspaceProfileLayoutSource::Template(_) => self
                .try_build_default_workspace_state(workspace_id, allocator)
                .map_err(
                    |error| WorkspaceProfileRegistryBackedBuildError::WorkspaceState {
                        profile_id: self.id,
                        error: Box::new(error),
                    },
                )?,
            WorkspaceProfileLayoutSource::AuthoredLayout { layout, .. } => {
                form_workspace_state_from_definition_with_registry(
                    layout,
                    workspace_id,
                    allocator,
                    registry,
                )
                .map_err(|error| {
                    WorkspaceProfileRegistryBackedBuildError::WorkspaceDefinitionFormation {
                        profile_id: self.id,
                        error: Box::new(error),
                    }
                })?
            }
        };
        let report = workspace.validate_tool_surface_registry_compatibility(registry);
        if report.is_fully_compatible() {
            Ok(workspace)
        } else {
            Err(
                WorkspaceProfileRegistryBackedBuildError::WorkspaceCompatibility {
                    profile_id: self.id,
                    report: Box::new(report),
                },
            )
        }
    }

    pub fn required_tool_surfaces_are_present(&self, workspace_state: &WorkspaceState) -> bool {
        self.default_surfaces.iter().all(|required_surface| {
            workspace_state.tool_surfaces().any(|surface| {
                surface.stable_surface_key() == required_surface.stable_surface_key()
            })
        })
    }

    pub fn validate_tool_surface_registry_compatibility(
        &self,
        registry: &ToolSurfaceRegistry,
    ) -> WorkspaceProfileToolSurfaceCompatibilityReport {
        let mut report = WorkspaceProfileToolSurfaceCompatibilityReport::default();

        for default_surface in &self.default_surfaces {
            let stable_surface_key = default_surface.stable_surface_key().clone();
            match registry.get(&stable_surface_key) {
                Some(_) => {
                    report
                        .compatible_surfaces
                        .push(WorkspaceProfileToolSurfaceCompatibleSurface {
                            legacy_tool_surface_kind: None,
                            stable_surface_key,
                        });
                }
                None => {
                    report.unregistered_legacy_surfaces.push(
                        WorkspaceProfileToolSurfaceLegacySurface {
                            legacy_tool_surface_kind: None,
                            stable_surface_key,
                        },
                    );
                }
            }
        }

        report
    }

    pub fn require_tool_surface_registry_compatibility(
        &self,
        registry: &ToolSurfaceRegistry,
    ) -> Result<(), WorkspaceProfileRegistryBackedBuildError> {
        let report = self.validate_tool_surface_registry_compatibility(registry);
        if let Some(surface) = report.unregistered_legacy_surfaces.first() {
            return Err(
                WorkspaceProfileRegistryBackedBuildError::UnregisteredDefaultToolSurface {
                    profile_id: self.id,
                    legacy_tool_surface_kind: surface.legacy_tool_surface_kind,
                    stable_surface_key: surface.stable_surface_key.clone(),
                },
            );
        }
        if let Some(surface) = report.unmapped_legacy_surfaces.first() {
            return Err(
                WorkspaceProfileRegistryBackedBuildError::UnmappedDefaultToolSurface {
                    profile_id: self.id,
                    tool_surface_kind: surface.tool_surface_kind,
                },
            );
        }
        Ok(())
    }
}

fn scene_derived_default_graph_matches(workspace_state: &WorkspaceState) -> bool {
    if workspace_state.validate_integrity().is_err() {
        return false;
    }

    let Some(root) = split_host_with_axis(
        workspace_state,
        workspace_state.root_host_id(),
        WorkspaceSplitAxis::Vertical,
    ) else {
        return false;
    };
    let Some(left_right) = split_host_with_axis(
        workspace_state,
        root.first_child,
        WorkspaceSplitAxis::Horizontal,
    ) else {
        return false;
    };
    let Some(right_sidebar) = split_host_with_axis(
        workspace_state,
        left_right.second_child,
        WorkspaceSplitAxis::Vertical,
    ) else {
        return false;
    };

    tab_stack_panel_kinds_by_host(workspace_state, left_right.first_child)
        == Some(vec![PanelKind::Viewport])
        && tab_stack_panel_kinds_by_host(workspace_state, right_sidebar.first_child)
            == Some(vec![PanelKind::Outliner, PanelKind::EntityTable])
        && tab_stack_panel_kinds_by_host(workspace_state, right_sidebar.second_child)
            == Some(vec![PanelKind::Inspector])
        && tab_stack_panel_kinds_by_host(workspace_state, root.second_child)
            == Some(vec![PanelKind::Console])
}

fn modelling_default_graph_matches(workspace_state: &WorkspaceState) -> bool {
    if workspace_state.validate_integrity().is_err() {
        return false;
    }

    let Some(root) = split_host_with_axis(
        workspace_state,
        workspace_state.root_host_id(),
        WorkspaceSplitAxis::Vertical,
    ) else {
        return false;
    };
    let Some(left_center_right) = split_host_with_axis(
        workspace_state,
        root.first_child,
        WorkspaceSplitAxis::Horizontal,
    ) else {
        return false;
    };
    let Some(center_right) = split_host_with_axis(
        workspace_state,
        left_center_right.second_child,
        WorkspaceSplitAxis::Horizontal,
    ) else {
        return false;
    };

    tab_stack_panel_kinds_by_host(workspace_state, left_center_right.first_child)
        == Some(vec![PanelKind::Outliner, PanelKind::EntityTable])
        && tab_stack_panel_kinds_by_host(workspace_state, center_right.first_child)
            == Some(vec![PanelKind::Viewport])
        && tab_stack_panel_kinds_by_host(workspace_state, center_right.second_child)
            == Some(vec![PanelKind::Inspector])
        && tab_stack_panel_kinds_by_host(workspace_state, root.second_child)
            == Some(vec![PanelKind::Console])
}

fn split_host_with_axis(
    workspace_state: &WorkspaceState,
    host_id: crate::PanelHostId,
    axis: WorkspaceSplitAxis,
) -> Option<crate::SplitHostState> {
    let host = workspace_state.host(host_id)?;
    match host.kind {
        PanelHostKind::SplitHost(split) if split.axis == axis => Some(split),
        _ => None,
    }
}

fn tab_stack_panel_kinds_by_host(
    workspace_state: &WorkspaceState,
    host_id: crate::PanelHostId,
) -> Option<Vec<PanelKind>> {
    let host = workspace_state.host(host_id)?;
    let PanelHostKind::TabStackHost(tab_host) = host.kind else {
        return None;
    };
    let stack = workspace_state.tab_stack(tab_host.tab_stack_id)?;
    Some(
        stack
            .ordered_panels
            .iter()
            .filter_map(|panel_id| {
                let panel = workspace_state.panel(*panel_id)?;
                panel.active_tool_surface?;
                Some(panel.panel_kind)
            })
            .collect(),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceProfileRegistry {
    default_profile_id: WorkspaceProfileId,
    default_profile_ref: ProfileRef,
    profiles: Vec<WorkspaceProfile>,
}

impl WorkspaceProfileRegistry {
    pub fn new(default_profile_id: WorkspaceProfileId, profiles: Vec<WorkspaceProfile>) -> Self {
        let default_profile_ref = profiles
            .iter()
            .find(|profile| profile.id == default_profile_id)
            .map(|profile| profile.profile_ref.clone())
            .unwrap_or_else(|| workspace_profile_ref_for_id(default_profile_id));
        Self::new_with_default_ref(default_profile_ref, default_profile_id, profiles)
    }

    pub fn new_with_default_ref(
        default_profile_ref: ProfileRef,
        default_profile_id: WorkspaceProfileId,
        profiles: Vec<WorkspaceProfile>,
    ) -> Self {
        Self {
            default_profile_id,
            default_profile_ref,
            profiles,
        }
    }

    pub fn default_profile_id(&self) -> WorkspaceProfileId {
        self.default_profile_id
    }

    pub fn default_profile_ref(&self) -> &ProfileRef {
        &self.default_profile_ref
    }

    pub fn default_profile(&self) -> Option<&WorkspaceProfile> {
        self.profile(self.default_profile_id)
    }

    pub fn default_profile_by_ref(&self) -> Option<&WorkspaceProfile> {
        self.profile_by_ref(&self.default_profile_ref)
    }

    pub fn profile(&self, profile_id: WorkspaceProfileId) -> Option<&WorkspaceProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.id == profile_id)
    }

    pub fn profile_by_ref(&self, profile_ref: &ProfileRef) -> Option<&WorkspaceProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.profile_ref == *profile_ref)
    }

    pub fn profiles(&self) -> impl Iterator<Item = &WorkspaceProfile> {
        self.profiles.iter()
    }
}

pub fn workspace_profile_ref_for_id(profile_id: WorkspaceProfileId) -> ProfileRef {
    let stable_key = if profile_id == SCENE_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.scene".to_string()
    } else if profile_id == MODELLING_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.modelling".to_string()
    } else if profile_id == EDITOR_DESIGN_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.editor_design".to_string()
    } else if profile_id == FIELD_WORLD_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.field_world".to_string()
    } else if profile_id == MATERIAL_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.materials".to_string()
    } else if profile_id == TEXTURE_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.textures".to_string()
    } else if profile_id == PROCGEN_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.procgen".to_string()
    } else if profile_id == GAMEPLAY_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.gameplay".to_string()
    } else if profile_id == PARTICLE_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.particles".to_string()
    } else if profile_id == PHYSICS_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.physics".to_string()
    } else if profile_id == ANIMATION_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.animation".to_string()
    } else if profile_id == SIMULATION_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.simulation".to_string()
    } else if profile_id == RUNTIME_DEBUG_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.runtime_debug".to_string()
    } else if profile_id == GRAPH_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.graph".to_string()
    } else {
        format!("runenwerk.workspace.profile_{}", profile_id.raw())
    };

    ProfileRef::new(stable_key).expect("workspace profile ref should be a valid stable key")
}

pub fn default_workspace_profile_registry() -> WorkspaceProfileRegistry {
    WorkspaceProfileRegistry::new(
        SCENE_WORKSPACE_PROFILE_ID,
        vec![
            compiled_in_workspace_profile(
                SCENE_WORKSPACE_PROFILE_ID,
                "Scene",
                WorkspaceLayoutTemplate::Scene,
                vec![
                    ("runenwerk.scene.viewport", PanelKind::Viewport),
                    ("runenwerk.scene.outliner", PanelKind::Outliner),
                    ("runenwerk.scene.inspector", PanelKind::Inspector),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![
                    EDIT_MODE_ID,
                    PREVIEW_MODE_ID,
                    SIMULATE_MODE_ID,
                    PLAY_MODE_ID,
                ],
                vec![DocumentKind::Scene],
            ),
            compiled_in_workspace_profile(
                MODELLING_WORKSPACE_PROFILE_ID,
                "Modelling",
                WorkspaceLayoutTemplate::Modelling,
                vec![
                    ("runenwerk.scene.viewport", PanelKind::Viewport),
                    ("runenwerk.scene.outliner", PanelKind::Outliner),
                    ("runenwerk.scene.inspector", PanelKind::Inspector),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![EDIT_MODE_ID, PREVIEW_MODE_ID],
                vec![DocumentKind::Scene, DocumentKind::SdfBrushLayer],
            ),
            compiled_in_workspace_profile(
                EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                "Editor Design",
                WorkspaceLayoutTemplate::EditorDesign,
                vec![
                    (
                        "runenwerk.editor_design.definition_outliner",
                        PanelKind::EditorDesignOutliner,
                    ),
                    (
                        "runenwerk.editor_design.ui_hierarchy",
                        PanelKind::UiHierarchy,
                    ),
                    ("runenwerk.editor_design.ui_canvas", PanelKind::UiCanvas),
                    (
                        "runenwerk.editor_design.style_inspector",
                        PanelKind::StyleInspector,
                    ),
                    ("runenwerk.editor_design.bindings", PanelKind::Bindings),
                    (
                        "runenwerk.editor_design.dock_layout_preview",
                        PanelKind::DockLayoutPreview,
                    ),
                    (
                        "runenwerk.editor_design.theme_editor",
                        PanelKind::ThemeEditor,
                    ),
                    (
                        "runenwerk.editor_design.shortcut_editor",
                        PanelKind::ShortcutEditor,
                    ),
                    ("runenwerk.editor_design.menu_editor", PanelKind::MenuEditor),
                    (
                        "runenwerk.editor_design.definition_validation",
                        PanelKind::DefinitionValidation,
                    ),
                    (
                        "runenwerk.editor_design.command_diff",
                        PanelKind::CommandDiff,
                    ),
                ],
                vec![EDIT_MODE_ID, PREVIEW_MODE_ID],
                vec![
                    DocumentKind::UiLayout,
                    DocumentKind::WorkspaceDefinition,
                    DocumentKind::Theme,
                    DocumentKind::Shortcut,
                    DocumentKind::Menu,
                    DocumentKind::CommandBinding,
                    DocumentKind::PanelRegistry,
                    DocumentKind::ToolSurfaceDefinition,
                ],
            ),
            tool_workspace_profile(
                FIELD_WORLD_WORKSPACE_PROFILE_ID,
                "Field World",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    (
                        "runenwerk.field_world.layer_stack",
                        PanelKind::FieldLayerStack,
                    ),
                    (
                        "runenwerk.field_world.sdf_graph_canvas",
                        PanelKind::SdfGraphCanvas,
                    ),
                    (
                        "runenwerk.field_world.product_viewer",
                        PanelKind::FieldProductViewer,
                    ),
                    (
                        "runenwerk.field_world.sdf_brush_browser",
                        PanelKind::SdfBrushBrowser,
                    ),
                    ("runenwerk.diagnostics.diagnostics", PanelKind::Diagnostics),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![
                    DocumentKind::Scene,
                    DocumentKind::SdfGraph,
                    DocumentKind::SdfBrushLayer,
                    DocumentKind::FieldWorldDefinition,
                    DocumentKind::FieldProductPreview,
                ],
            ),
            tool_workspace_profile(
                MATERIAL_WORKSPACE_PROFILE_ID,
                "Materials",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    (
                        "runenwerk.material_lab.graph_canvas",
                        PanelKind::MaterialGraphCanvas,
                    ),
                    (
                        "runenwerk.material_lab.inspector",
                        PanelKind::MaterialInspector,
                    ),
                    ("runenwerk.material_lab.preview", PanelKind::MaterialPreview),
                    ("runenwerk.texture.viewer_2d", PanelKind::TextureViewer),
                    ("runenwerk.diagnostics.diagnostics", PanelKind::Diagnostics),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![
                    DocumentKind::Scene,
                    DocumentKind::MaterialGraph,
                    DocumentKind::Material,
                ],
            ),
            tool_workspace_profile(
                TEXTURE_WORKSPACE_PROFILE_ID,
                "Textures",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    ("runenwerk.texture.viewer_2d", PanelKind::TextureViewer),
                    (
                        "runenwerk.texture.viewer_3d",
                        PanelKind::VolumeTextureViewer,
                    ),
                    ("runenwerk.diagnostics.diagnostics", PanelKind::Diagnostics),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![DocumentKind::ProceduralTexture, DocumentKind::VolumeTexture],
            ),
            tool_workspace_profile(
                PROCGEN_WORKSPACE_PROFILE_ID,
                "Procedural Generation",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    (
                        "runenwerk.procgen.graph_canvas",
                        PanelKind::ProcgenGraphCanvas,
                    ),
                    ("runenwerk.procgen.preview", PanelKind::ProcgenPreview),
                    ("runenwerk.diagnostics.diagnostics", PanelKind::Diagnostics),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![DocumentKind::ProceduralGenerationGraph],
            ),
            tool_workspace_profile(
                GAMEPLAY_WORKSPACE_PROFILE_ID,
                "Gameplay Graph",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    (
                        "runenwerk.gameplay.graph_canvas",
                        PanelKind::GameplayGraphCanvas,
                    ),
                    (
                        "runenwerk.gameplay.compiler_diagnostics",
                        PanelKind::GameplayCompilerDiagnostics,
                    ),
                    (
                        "runenwerk.diagnostics.runtime_debug",
                        PanelKind::RuntimeDebug,
                    ),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![
                    DocumentKind::GameplayGraph,
                    DocumentKind::GameplayRuleTrigger,
                    DocumentKind::Ability,
                    DocumentKind::Quest,
                ],
            ),
            tool_workspace_profile(
                PARTICLE_WORKSPACE_PROFILE_ID,
                "Particles",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    (
                        "runenwerk.particle.graph_canvas",
                        PanelKind::ParticleGraphCanvas,
                    ),
                    ("runenwerk.particle.preview", PanelKind::ParticlePreview),
                    ("runenwerk.diagnostics.diagnostics", PanelKind::Diagnostics),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![DocumentKind::ParticleGraph, DocumentKind::ParticleEmitter],
            ),
            tool_workspace_profile(
                PHYSICS_WORKSPACE_PROFILE_ID,
                "Physics",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    ("runenwerk.physics.authoring", PanelKind::PhysicsAuthoring),
                    ("runenwerk.physics.debug", PanelKind::PhysicsDebug),
                    (
                        "runenwerk.diagnostics.runtime_debug",
                        PanelKind::RuntimeDebug,
                    ),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![DocumentKind::PhysicsScene, DocumentKind::PhysicsConfig],
            ),
            tool_workspace_profile(
                ANIMATION_WORKSPACE_PROFILE_ID,
                "Animation",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    ("runenwerk.animation.timeline", PanelKind::Timeline),
                    ("runenwerk.animation.curve_editor", PanelKind::CurveEditor),
                    (
                        "runenwerk.animation.graph_canvas",
                        PanelKind::AnimationGraphCanvas,
                    ),
                    ("runenwerk.diagnostics.diagnostics", PanelKind::Diagnostics),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![
                    DocumentKind::AnimationClip,
                    DocumentKind::AnimationGraph,
                    DocumentKind::Timeline,
                ],
            ),
            tool_workspace_profile(
                SIMULATION_WORKSPACE_PROFILE_ID,
                "Simulation Processes",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    ("runenwerk.simulation.preview", PanelKind::SimulationPreview),
                    (
                        "runenwerk.simulation.diagnostics",
                        PanelKind::SimulationDiagnostics,
                    ),
                    (
                        "runenwerk.diagnostics.runtime_debug",
                        PanelKind::RuntimeDebug,
                    ),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![
                    DocumentKind::FieldWorldDefinition,
                    DocumentKind::FieldProductPreview,
                    DocumentKind::RuntimeDebug,
                ],
            ),
            tool_workspace_profile(
                RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
                "Runtime Debug",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    (
                        "runenwerk.diagnostics.runtime_debug",
                        PanelKind::RuntimeDebug,
                    ),
                    ("runenwerk.diagnostics.diagnostics", PanelKind::Diagnostics),
                    (
                        TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
                        PanelKind::Diagnostics,
                    ),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![DocumentKind::RuntimeDebug, DocumentKind::Scene],
            ),
            tool_workspace_profile(
                GRAPH_WORKSPACE_PROFILE_ID,
                "Graph",
                vec![
                    ("runenwerk.assets.browser", PanelKind::AssetBrowser),
                    ("runenwerk.graph.canvas", PanelKind::GraphCanvas),
                    ("runenwerk.diagnostics.diagnostics", PanelKind::Diagnostics),
                    ("runenwerk.editor.console", PanelKind::Console),
                ],
                vec![DocumentKind::Graph],
            ),
        ],
    )
}

fn tool_workspace_profile(
    id: WorkspaceProfileId,
    label: impl Into<String>,
    default_surfaces: Vec<(&'static str, PanelKind)>,
    document_kind_filters: Vec<DocumentKind>,
) -> WorkspaceProfile {
    compiled_in_workspace_profile(
        id,
        label,
        WorkspaceLayoutTemplate::ToolWorkspace,
        default_surfaces,
        vec![EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters,
    )
}

fn stable_default_surface_for_kind(
    kind: ToolSurfaceKind,
) -> Result<WorkspaceDefaultToolSurface, WorkspaceSurfaceIdentityError> {
    let stable_surface_key = crate::stable_key_for_tool_surface_kind(kind)
        .ok_or(WorkspaceSurfaceIdentityError::UnmappedLegacySurface { kind })?;
    Ok(WorkspaceDefaultToolSurface::new_with_panel_kind(
        stable_surface_key,
        kind.panel_kind(),
    ))
}

fn compiled_in_default_surface(
    stable_surface_key: &str,
    panel_kind: PanelKind,
) -> WorkspaceDefaultToolSurface {
    WorkspaceDefaultToolSurface::new_with_panel_kind(
        ToolSurfaceStableKey::new(stable_surface_key)
            .expect("compiled-in workspace profile stable surface key should be valid"),
        panel_kind,
    )
}

fn compiled_in_workspace_profile(
    id: WorkspaceProfileId,
    label: impl Into<String>,
    default_layout_template: WorkspaceLayoutTemplate,
    default_surfaces: Vec<(&'static str, PanelKind)>,
    default_modes: Vec<ModeId>,
    document_kind_filters: Vec<DocumentKind>,
) -> WorkspaceProfile {
    WorkspaceProfile::new(
        id,
        label,
        default_layout_template,
        default_surfaces
            .into_iter()
            .map(|(stable_surface_key, panel_kind)| {
                compiled_in_default_surface(stable_surface_key, panel_kind)
            })
            .collect(),
        default_modes,
        document_kind_filters,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EditorToolSuite, PanelKind, ProviderFamilyDefinition, ProviderFamilyId, ToolSuiteId,
        ToolSuiteRegistry, ToolSurfaceDefinition, ToolSurfacePersistence, ToolSurfaceRole,
        ToolSurfaceRoute, saveable_tool_surface_stable_key_candidates,
        stable_key_for_tool_surface_kind,
    };

    #[test]
    fn default_registry_exposes_layout_profile() {
        let registry = default_workspace_profile_registry();
        let profile = registry
            .default_profile()
            .expect("default profile should exist");

        assert_eq!(profile.id, SCENE_WORKSPACE_PROFILE_ID);
        assert_eq!(profile.label, "Scene");
        assert!(
            profile
                .default_surfaces
                .iter()
                .any(|surface| surface.stable_surface_key().as_str() == "runenwerk.scene.viewport")
        );
        assert!(profile.default_modes.contains(&EDIT_MODE_ID));
        assert!(profile.default_modes.contains(&PREVIEW_MODE_ID));
        assert!(profile.default_modes.contains(&SIMULATE_MODE_ID));
        assert!(profile.default_modes.contains(&PLAY_MODE_ID));
        assert!(profile.document_kind_filters.contains(&DocumentKind::Scene));
    }

    #[test]
    fn default_profiles_store_stable_keys_primary() {
        let registry = default_workspace_profile_registry();

        for profile in registry.profiles() {
            assert!(
                profile
                    .default_surfaces
                    .iter()
                    .all(|surface| !surface.stable_surface_key().as_str().is_empty()),
                "{} profile should store stable surface keys as default surface authority",
                profile.label
            );
        }
    }

    #[test]
    fn default_profiles_store_panel_kind_with_stable_defaults() {
        let registry = default_workspace_profile_registry();

        for profile in registry.profiles() {
            assert!(
                profile
                    .default_surfaces
                    .iter()
                    .all(|surface| !surface.stable_surface_key().as_str().is_empty()),
                "{} profile should retain stable surface defaults",
                profile.label
            );
        }
    }

    #[test]
    fn runtime_debug_profile_reaches_inspector_by_stable_key_without_legacy_kind() {
        let registry = default_workspace_profile_registry();
        let profile = registry
            .profile(RUNTIME_DEBUG_WORKSPACE_PROFILE_ID)
            .expect("runtime debug profile should exist");
        let inspector = profile
            .default_surfaces
            .iter()
            .find(|surface| {
                surface.stable_surface_key().as_str() == TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY
            })
            .expect("runtime debug profile should include the registry inspector");

        assert_eq!(inspector.panel_kind(), PanelKind::Diagnostics);
    }

    #[test]
    fn legacy_profile_constructor_preserves_existing_surface_order() {
        let profile = WorkspaceProfile::new_legacy(
            GRAPH_WORKSPACE_PROFILE_ID,
            "Graph",
            WorkspaceLayoutTemplate::ToolWorkspace,
            vec![
                ToolSurfaceKind::AssetBrowser,
                ToolSurfaceKind::GraphCanvas,
                ToolSurfaceKind::Diagnostics,
                ToolSurfaceKind::Console,
            ],
            vec![EDIT_MODE_ID],
            vec![DocumentKind::Graph],
        )
        .expect("legacy profile fixture should map stable keys");

        assert_eq!(
            profile
                .default_surfaces
                .iter()
                .map(|surface| surface.stable_surface_key().as_str())
                .collect::<Vec<_>>(),
            vec![
                "runenwerk.assets.browser",
                "runenwerk.graph.canvas",
                "runenwerk.diagnostics.diagnostics",
                "runenwerk.editor.console",
            ]
        );
    }

    #[test]
    fn stable_key_profile_builder_preserves_layout_shape() {
        let registry = default_workspace_profile_registry();
        let profile = registry
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");
        let mut stable_allocator = WorkspaceIdentityAllocator::new();
        let stable_workspace_id = stable_allocator.allocate_workspace_id();

        let stable_workspace =
            profile.build_default_workspace_state(stable_workspace_id, &mut stable_allocator);

        let mut actual = workspace_surface_order(&stable_workspace);
        let mut expected = profile
            .default_surfaces
            .iter()
            .map(|surface| surface.stable_surface_key().as_str().to_string())
            .collect::<Vec<_>>();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn stable_key_profile_builder_populates_tool_surface_state_authority() {
        let registry = default_workspace_profile_registry();
        let profile = registry
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);

        let mut actual = workspace_surface_order(&workspace);
        let mut expected = profile
            .default_surfaces
            .iter()
            .map(|surface| surface.stable_surface_key().as_str().to_string())
            .collect::<Vec<_>>();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn default_profile_all_stable_keys_registered() {
        let profile_registry = default_workspace_profile_registry();
        let tool_suite_registry = full_saveable_registry();

        for profile in profile_registry.profiles() {
            for surface in &profile.default_surfaces {
                assert!(
                    tool_suite_registry
                        .surfaces()
                        .get(surface.stable_surface_key())
                        .is_some(),
                    "{} profile has unregistered default stable key {}",
                    profile.label,
                    surface.stable_surface_key().as_str()
                );
            }
        }
    }

    #[test]
    fn registry_free_legacy_profile_builder_is_compatibility_only() {
        let profile = WorkspaceProfile::new_legacy(
            TEXTURE_WORKSPACE_PROFILE_ID,
            "Textures",
            WorkspaceLayoutTemplate::ToolWorkspace,
            vec![
                ToolSurfaceKind::AssetBrowser,
                ToolSurfaceKind::TextureViewer,
                ToolSurfaceKind::Console,
            ],
            vec![EDIT_MODE_ID],
            vec![DocumentKind::ProceduralTexture],
        )
        .expect("legacy profile fixture should map stable keys");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);

        assert!(workspace.validate_integrity().is_ok());
        assert!(
            profile
                .default_surfaces
                .iter()
                .all(|surface| !surface.stable_surface_key().as_str().is_empty())
        );
    }

    #[test]
    fn profile_storage_no_longer_uses_tool_surface_kind_as_authority() {
        let source = include_str!("profile.rs");
        let profile_struct = source
            .split("pub struct WorkspaceProfile {")
            .nth(1)
            .and_then(|tail| tail.split("}").next())
            .expect("WorkspaceProfile struct should exist");

        assert!(!profile_struct.contains("default_tool_surfaces: Vec<ToolSurfaceKind>"));
        assert!(profile_struct.contains("default_surfaces: Vec<WorkspaceDefaultToolSurface>"));
        assert!(source.contains("new_legacy"));
        assert!(include_str!("state.rs").contains("pub panel_kind: PanelKind"));
    }

    #[test]
    fn panel_kind_remains_authoritative_in_c3() {
        let registry = default_workspace_profile_registry();
        let profile = registry
            .profile(TEXTURE_WORKSPACE_PROFILE_ID)
            .expect("texture profile should exist");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);

        assert!(
            workspace
                .panels()
                .any(|panel| panel.panel_kind == PanelKind::TextureViewer)
        );
        assert!(
            workspace
                .panels()
                .any(|panel| panel.panel_kind == PanelKind::VolumeTextureViewer)
        );
    }

    #[test]
    fn layout_profile_builds_current_workspace_without_changing_profile_identity() {
        let registry = default_workspace_profile_registry();
        let profile = registry
            .profile(SCENE_WORKSPACE_PROFILE_ID)
            .expect("scene profile should exist");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);

        assert_eq!(workspace.workspace_id(), workspace_id);
        assert!(workspace.validate_integrity().is_ok());
        assert_eq!(profile.id, SCENE_WORKSPACE_PROFILE_ID);
    }

    #[test]
    fn scene_and_modelling_profiles_have_distinct_layout_contracts() {
        let registry = default_workspace_profile_registry();
        let scene_profile = registry
            .profile(SCENE_WORKSPACE_PROFILE_ID)
            .expect("scene profile should exist");
        let modelling_profile = registry
            .profile(MODELLING_WORKSPACE_PROFILE_ID)
            .expect("modelling profile should exist");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let scene_workspace_id = allocator.allocate_workspace_id();
        let modelling_workspace_id = allocator.allocate_workspace_id();

        let scene_workspace =
            scene_profile.build_default_workspace_state(scene_workspace_id, &mut allocator);
        let modelling_workspace =
            modelling_profile.build_default_workspace_state(modelling_workspace_id, &mut allocator);

        assert_eq!(scene_profile.default_layout_template.contract_id(), "scene");
        assert_eq!(
            modelling_profile.default_layout_template.contract_id(),
            "modelling"
        );
        assert_ne!(
            scene_profile.default_layout_template.contract_id(),
            modelling_profile.default_layout_template.contract_id()
        );
        assert!(
            WorkspaceLayoutTemplate::Scene.default_graph_matches(&scene_workspace),
            "scene profile should accept the scene default graph"
        );
        assert!(
            WorkspaceLayoutTemplate::Modelling.default_graph_matches(&modelling_workspace),
            "modelling profile should accept its own default graph"
        );
        assert!(
            !WorkspaceLayoutTemplate::Modelling.default_graph_matches(&scene_workspace),
            "stale scene-derived modelling layouts must not satisfy the modelling contract"
        );
    }

    #[test]
    fn editor_design_profile_exposes_self_authoring_surfaces() {
        let registry = default_workspace_profile_registry();
        let profile = registry
            .profile(EDITOR_DESIGN_WORKSPACE_PROFILE_ID)
            .expect("editor design profile should exist");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);

        assert_eq!(profile.label, "Editor Design");
        assert!(
            profile
                .default_surfaces
                .iter()
                .any(|surface| surface.stable_surface_key().as_str()
                    == "runenwerk.editor_design.ui_canvas")
        );
        assert!(
            profile
                .document_kind_filters
                .contains(&DocumentKind::UiLayout)
        );
        assert!(workspace.validate_integrity().is_ok());
        assert!(workspace.tool_surfaces().any(|surface| {
            surface.stable_surface_key().as_str() == "runenwerk.editor_design.definition_validation"
        }));
    }

    #[test]
    fn preview_mode_is_profile_scoped_but_play_stays_scene_scoped() {
        let registry = default_workspace_profile_registry();
        let modelling_profile = registry
            .profile(MODELLING_WORKSPACE_PROFILE_ID)
            .expect("modelling profile should exist");
        let editor_design_profile = registry
            .profile(EDITOR_DESIGN_WORKSPACE_PROFILE_ID)
            .expect("editor design profile should exist");

        assert!(modelling_profile.default_modes.contains(&PREVIEW_MODE_ID));
        assert!(!modelling_profile.default_modes.contains(&PLAY_MODE_ID));
        assert!(
            editor_design_profile
                .default_modes
                .contains(&PREVIEW_MODE_ID)
        );
        assert!(
            !editor_design_profile
                .default_modes
                .contains(&SIMULATE_MODE_ID)
        );
        assert!(!editor_design_profile.default_modes.contains(&PLAY_MODE_ID));
    }

    #[test]
    fn m6_profiles_build_persistable_tool_workspace_layouts() {
        let registry = default_workspace_profile_registry();
        let mut allocator = WorkspaceIdentityAllocator::new();

        for profile in registry.profiles().filter(|profile| {
            profile.default_layout_template == WorkspaceLayoutTemplate::ToolWorkspace
        }) {
            let workspace_id = allocator.allocate_workspace_id();
            let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);

            assert!(
                workspace.validate_integrity().is_ok(),
                "{} profile should build a valid workspace",
                profile.label
            );
            assert!(
                profile.required_tool_surfaces_are_present(&workspace),
                "{} profile should mount its default M6 surfaces",
                profile.label
            );
        }
    }

    #[test]
    fn default_profile_registry_can_report_material_lab_registry_compatibility() {
        let profile_registry = default_workspace_profile_registry();
        let material_profile = profile_registry
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");
        let tool_suite_registry = material_lab_registry();

        let report = material_profile
            .validate_tool_surface_registry_compatibility(tool_suite_registry.surfaces());

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
        assert_eq!(
            report
                .unregistered_legacy_surfaces
                .iter()
                .map(|surface| surface.stable_surface_key.as_str())
                .collect::<Vec<_>>(),
            vec![
                "runenwerk.assets.browser",
                "runenwerk.texture.viewer_2d",
                "runenwerk.diagnostics.diagnostics",
                "runenwerk.editor.console",
            ]
        );
    }

    #[test]
    fn default_profile_registry_reports_unregistered_legacy_material_lab_surface() {
        let profile_registry = default_workspace_profile_registry();
        let material_profile = profile_registry
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");
        let empty_registry = ToolSuiteRegistry::new(Vec::new()).expect("empty registry is valid");

        let report = material_profile
            .validate_tool_surface_registry_compatibility(empty_registry.surfaces());

        let unregistered_keys = report
            .unregistered_legacy_surfaces
            .iter()
            .map(|surface| surface.stable_surface_key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            unregistered_keys,
            vec![
                "runenwerk.assets.browser",
                "runenwerk.material_lab.graph_canvas",
                "runenwerk.material_lab.inspector",
                "runenwerk.material_lab.preview",
                "runenwerk.texture.viewer_2d",
                "runenwerk.diagnostics.diagnostics",
                "runenwerk.editor.console",
            ]
        );
    }

    #[test]
    fn default_profiles_still_build_without_tool_surface_registry() {
        let registry = default_workspace_profile_registry();
        let mut allocator = WorkspaceIdentityAllocator::new();

        for profile in registry.profiles() {
            let workspace_id = allocator.allocate_workspace_id();
            let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);

            assert!(
                workspace.validate_integrity().is_ok(),
                "{} profile should still build without a tool-surface registry",
                profile.label
            );
        }
    }

    #[test]
    fn registry_free_default_profile_builder_still_works() {
        let registry = default_workspace_profile_registry();
        let profile = registry
            .profile(SCENE_WORKSPACE_PROFILE_ID)
            .expect("scene profile should exist");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);

        assert!(workspace.validate_integrity().is_ok());
    }

    #[test]
    fn registry_aware_default_profile_builder_preserves_stable_surface_keys() {
        let profile_registry = default_workspace_profile_registry();
        let profile = profile_registry
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");
        let tool_suite_registry = full_saveable_registry();
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = profile
            .build_default_workspace_state_with_registry(
                workspace_id,
                &mut allocator,
                tool_suite_registry.surfaces(),
            )
            .expect("full registry should build material workspace");

        assert!(
            workspace
                .tool_surfaces()
                .any(|surface| surface.stable_surface_key().as_str()
                    == "runenwerk.material_lab.graph_canvas")
        );
        assert!(
            workspace
                .tool_surfaces()
                .any(|surface| surface.stable_surface_key().as_str()
                    == "runenwerk.material_lab.inspector")
        );
        assert!(workspace.tool_surfaces().any(
            |surface| surface.stable_surface_key().as_str() == "runenwerk.material_lab.preview"
        ));
    }

    #[test]
    fn registry_aware_default_profile_builder_populates_stable_keys() {
        let profile_registry = default_workspace_profile_registry();
        let profile = profile_registry
            .profile(TEXTURE_WORKSPACE_PROFILE_ID)
            .expect("texture profile should exist");
        let tool_suite_registry = full_saveable_registry();
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = profile
            .build_default_workspace_state_with_registry(
                workspace_id,
                &mut allocator,
                tool_suite_registry.surfaces(),
            )
            .expect("full registry should build texture workspace");

        for surface in workspace.tool_surfaces() {
            let key = surface.stable_surface_key();
            assert!(
                tool_suite_registry.surfaces().get(key).is_some(),
                "stable metadata should be registered: {}",
                key.as_str()
            );
        }
    }

    #[test]
    fn registry_aware_default_profile_builder_rejects_unregistered_surface_key() {
        let profile_registry = default_workspace_profile_registry();
        let material_profile = profile_registry
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");
        let tool_suite_registry = material_lab_registry();
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let error = material_profile
            .build_default_workspace_state_with_registry(
                workspace_id,
                &mut allocator,
                tool_suite_registry.surfaces(),
            )
            .expect_err("partial registry should reject non-material profile surfaces");

        assert!(matches!(
            error,
            WorkspaceProfileRegistryBackedBuildError::UnregisteredDefaultToolSurface {
                stable_surface_key,
                ..
            } if stable_surface_key.as_str() == "runenwerk.assets.browser"
        ));
    }

    #[test]
    fn registry_aware_builder_preserves_default_profile_surface_order() {
        let profile_registry = default_workspace_profile_registry();
        let procgen_profile = profile_registry
            .profile(PROCGEN_WORKSPACE_PROFILE_ID)
            .expect("procgen profile should exist");
        let tool_suite_registry = full_saveable_registry();
        let mut legacy_allocator = WorkspaceIdentityAllocator::new();
        let legacy_workspace_id = legacy_allocator.allocate_workspace_id();
        let legacy_workspace = procgen_profile
            .build_default_workspace_state(legacy_workspace_id, &mut legacy_allocator);
        let mut registry_allocator = WorkspaceIdentityAllocator::new();
        let registry_workspace_id = registry_allocator.allocate_workspace_id();

        let registry_workspace = procgen_profile
            .build_default_workspace_state_with_registry(
                registry_workspace_id,
                &mut registry_allocator,
                tool_suite_registry.surfaces(),
            )
            .expect("registry-aware procgen profile should build");

        assert_eq!(
            workspace_surface_order(&registry_workspace),
            workspace_surface_order(&legacy_workspace)
        );
    }

    #[test]
    fn placeholder_surface_remains_explicit_diagnostics_namespace_not_implemented_domain() {
        let key = stable_key_for_tool_surface_kind(ToolSurfaceKind::Placeholder)
            .expect("placeholder should have an explicit fallback key");

        assert_eq!(key.as_str(), "runenwerk.diagnostics.placeholder");
    }

    #[test]
    fn profile_compatibility_validation_does_not_change_default_surface_order() {
        let profile_registry = default_workspace_profile_registry();
        let material_profile = profile_registry
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");
        let original_order = material_profile.default_surfaces.clone();
        let tool_suite_registry = material_lab_registry();

        let _report = material_profile
            .validate_tool_surface_registry_compatibility(tool_suite_registry.surfaces());

        assert_eq!(material_profile.default_surfaces, original_order);
    }

    fn workspace_surface_order(workspace: &WorkspaceState) -> Vec<String> {
        workspace
            .tab_stacks()
            .flat_map(|stack| stack.ordered_panels.iter())
            .filter_map(|panel_id| workspace.panel(*panel_id))
            .filter_map(|panel| panel.active_tool_surface)
            .filter_map(|surface_id| workspace.tool_surface(surface_id))
            .map(|surface| surface.stable_surface_key().as_str().to_string())
            .collect()
    }

    fn full_saveable_registry() -> ToolSuiteRegistry {
        let provider_family = ProviderFamilyId::new("runenwerk.test").unwrap();
        ToolSuiteRegistry::new(vec![EditorToolSuite {
            suite_id: ToolSuiteId::new("runenwerk.test").unwrap(),
            label: "Test".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family.clone(),
                label: "Test".to_string(),
            }],
            surfaces: saveable_tool_surface_stable_key_candidates()
                .iter()
                .map(|candidate| {
                    material_lab_surface(
                        candidate.stable_key,
                        candidate.stable_key,
                        ToolSurfaceRole::Primary,
                        provider_family.clone(),
                        ToolSurfaceRoute::ProviderOwnedLocal,
                    )
                })
                .chain(std::iter::once(material_lab_surface(
                    TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
                    "Tool Suite Registry Inspector",
                    ToolSurfaceRole::Inspector,
                    provider_family.clone(),
                    ToolSurfaceRoute::ProviderOwnedLocal,
                )))
                .collect(),
        }])
        .expect("full saveable registry fixture should be valid")
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
            capabilities: ui_surface::SurfaceCapabilitySet::new(true, true, true, false),
            session_retention: ui_surface::SessionRetentionClass::Restorable,
            creation_policy: crate::ToolSurfaceCreationPolicy::SingletonPerWorkspace,
            target_profile_compatibility: crate::ToolSurfaceTargetProfileCompatibility::AllProfiles,
            legacy_compatibility_key: None,
        }
    }
}
