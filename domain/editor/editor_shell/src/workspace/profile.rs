//! File: domain/editor/editor_shell/src/workspace/profile.rs
//! Purpose: Workspace profile contracts for task-focused editor layout presets.

use editor_core::{DocumentKind, EDIT_MODE_ID, ModeId};
use id_macros::id;

use crate::{ToolSurfaceKind, WorkspaceId, WorkspaceIdentityAllocator, WorkspaceState};

#[id]
pub struct WorkspaceProfileId;

pub const SCENE_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(1);
pub const MODELLING_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(2);
pub const EDITOR_DESIGN_WORKSPACE_PROFILE_ID: WorkspaceProfileId = workspace_profile_id(3);
pub const LAYOUT_WORKSPACE_PROFILE_ID: WorkspaceProfileId = SCENE_WORKSPACE_PROFILE_ID;

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
    CurrentFixedEditor,
}

impl WorkspaceLayoutTemplate {
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProfile {
    pub id: WorkspaceProfileId,
    pub label: String,
    pub default_layout_template: WorkspaceLayoutTemplate,
    pub default_tool_surfaces: Vec<ToolSurfaceKind>,
    pub default_modes: Vec<ModeId>,
    pub document_kind_filters: Vec<DocumentKind>,
}

impl WorkspaceProfile {
    pub fn new(
        id: WorkspaceProfileId,
        label: impl Into<String>,
        default_layout_template: WorkspaceLayoutTemplate,
        default_tool_surfaces: Vec<ToolSurfaceKind>,
        default_modes: Vec<ModeId>,
        document_kind_filters: Vec<DocumentKind>,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            default_layout_template,
            default_tool_surfaces,
            default_modes,
            document_kind_filters,
        }
    }

    pub fn build_default_workspace_state(
        &self,
        workspace_id: WorkspaceId,
        allocator: &mut WorkspaceIdentityAllocator,
    ) -> WorkspaceState {
        self.default_layout_template
            .build_workspace_state(workspace_id, allocator)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProfileRegistry {
    default_profile_id: WorkspaceProfileId,
    profiles: Vec<WorkspaceProfile>,
}

impl WorkspaceProfileRegistry {
    pub fn new(default_profile_id: WorkspaceProfileId, profiles: Vec<WorkspaceProfile>) -> Self {
        Self {
            default_profile_id,
            profiles,
        }
    }

    pub fn default_profile_id(&self) -> WorkspaceProfileId {
        self.default_profile_id
    }

    pub fn default_profile(&self) -> Option<&WorkspaceProfile> {
        self.profile(self.default_profile_id)
    }

    pub fn profile(&self, profile_id: WorkspaceProfileId) -> Option<&WorkspaceProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.id == profile_id)
    }

    pub fn profiles(&self) -> impl Iterator<Item = &WorkspaceProfile> {
        self.profiles.iter()
    }
}

pub fn default_workspace_profile_registry() -> WorkspaceProfileRegistry {
    WorkspaceProfileRegistry::new(
        SCENE_WORKSPACE_PROFILE_ID,
        vec![
            WorkspaceProfile::new(
                SCENE_WORKSPACE_PROFILE_ID,
                "Scene",
                WorkspaceLayoutTemplate::Scene,
                vec![
                    ToolSurfaceKind::Viewport,
                    ToolSurfaceKind::Outliner,
                    ToolSurfaceKind::Inspector,
                    ToolSurfaceKind::Console,
                ],
                vec![EDIT_MODE_ID],
                vec![DocumentKind::Scene],
            ),
            WorkspaceProfile::new(
                MODELLING_WORKSPACE_PROFILE_ID,
                "Modelling",
                WorkspaceLayoutTemplate::Modelling,
                vec![
                    ToolSurfaceKind::Viewport,
                    ToolSurfaceKind::Outliner,
                    ToolSurfaceKind::Inspector,
                    ToolSurfaceKind::Console,
                ],
                vec![EDIT_MODE_ID],
                vec![DocumentKind::Scene, DocumentKind::SdfBrushLayer],
            ),
            WorkspaceProfile::new(
                EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                "Editor Design",
                WorkspaceLayoutTemplate::EditorDesign,
                vec![
                    ToolSurfaceKind::EditorDesignOutliner,
                    ToolSurfaceKind::UiHierarchy,
                    ToolSurfaceKind::UiCanvas,
                    ToolSurfaceKind::StyleInspector,
                    ToolSurfaceKind::Bindings,
                    ToolSurfaceKind::DockLayoutPreview,
                    ToolSurfaceKind::ThemeEditor,
                    ToolSurfaceKind::ShortcutEditor,
                    ToolSurfaceKind::MenuEditor,
                    ToolSurfaceKind::DefinitionValidation,
                    ToolSurfaceKind::CommandDiff,
                ],
                vec![EDIT_MODE_ID],
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
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
                .default_tool_surfaces
                .contains(&ToolSurfaceKind::Viewport)
        );
        assert!(profile.default_modes.contains(&EDIT_MODE_ID));
        assert!(profile.document_kind_filters.contains(&DocumentKind::Scene));
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
                .default_tool_surfaces
                .contains(&ToolSurfaceKind::UiCanvas)
        );
        assert!(
            profile
                .document_kind_filters
                .contains(&DocumentKind::UiLayout)
        );
        assert!(workspace.validate_integrity().is_ok());
        assert!(
            workspace
                .tool_surfaces()
                .any(|surface| surface.tool_surface_kind == ToolSurfaceKind::DefinitionValidation)
        );
    }
}
