//! File: apps/runenwerk_editor/src/shell/compositions/profiles.rs
//! Purpose: Built-in cross-suite Workspace profile manifests.

use editor_core::{
    DocumentKind, EDIT_MODE_ID, ModeId, PLAY_MODE_ID, PREVIEW_MODE_ID, SIMULATE_MODE_ID,
};
use editor_shell::{
    ANIMATION_WORKSPACE_PROFILE_ID, EDITOR_DESIGN_WORKSPACE_PROFILE_ID, EditorToolSuite,
    FIELD_WORLD_WORKSPACE_PROFILE_ID, GAMEPLAY_WORKSPACE_PROFILE_ID, GRAPH_WORKSPACE_PROFILE_ID,
    MATERIAL_WORKSPACE_PROFILE_ID, MODELLING_WORKSPACE_PROFILE_ID, PARTICLE_WORKSPACE_PROFILE_ID,
    PHYSICS_WORKSPACE_PROFILE_ID, PROCGEN_WORKSPACE_PROFILE_ID, ProfileRef,
    RUNTIME_DEBUG_WORKSPACE_PROFILE_ID, SCENE_WORKSPACE_PROFILE_ID,
    SIMULATION_WORKSPACE_PROFILE_ID, SurfaceRef, TEXTURE_WORKSPACE_PROFILE_ID,
    ToolSurfaceStableKey, WorkspaceLayoutTemplate, WorkspaceProfileId,
    WorkspaceProfileLayoutSource, WorkspaceProfileManifest, workspace_profile_ref_for_id,
};

use crate::shell::tool_suites;

#[derive(Debug, Clone, Copy)]
struct WorkspaceProfileSpec {
    id: WorkspaceProfileId,
    label: &'static str,
    default_layout_template: WorkspaceLayoutTemplate,
    default_surface_keys: &'static [&'static str],
    default_modes: &'static [ModeId],
    document_kind_filters: &'static [DocumentKind],
}

pub(crate) const MATERIAL_PROFILE_SURFACE_KEYS: &[&str] = &[
    "runenwerk.assets.browser",
    "runenwerk.material_lab.graph_canvas",
    "runenwerk.material_lab.inspector",
    "runenwerk.material_lab.preview",
    "runenwerk.texture.viewer_2d",
    "runenwerk.diagnostics.diagnostics",
    "runenwerk.editor.console",
];

const RUNTIME_DEBUG_PROFILE_SURFACE_KEYS: &[&str] = &[
    "runenwerk.assets.browser",
    "runenwerk.diagnostics.runtime_debug",
    "runenwerk.diagnostics.diagnostics",
    "runenwerk.diagnostics.tool_suite_registry_inspector",
    "runenwerk.editor.console",
];

const FULL_EDITOR_PROFILE_SPECS: &[WorkspaceProfileSpec] = &[
    WorkspaceProfileSpec {
        id: SCENE_WORKSPACE_PROFILE_ID,
        label: "Scene",
        default_layout_template: WorkspaceLayoutTemplate::Scene,
        default_surface_keys: &[
            "runenwerk.scene.viewport",
            "runenwerk.scene.outliner",
            "runenwerk.scene.inspector",
            "runenwerk.editor.console",
        ],
        default_modes: &[
            EDIT_MODE_ID,
            PREVIEW_MODE_ID,
            SIMULATE_MODE_ID,
            PLAY_MODE_ID,
        ],
        document_kind_filters: &[DocumentKind::Scene],
    },
    WorkspaceProfileSpec {
        id: MODELLING_WORKSPACE_PROFILE_ID,
        label: "Modelling",
        default_layout_template: WorkspaceLayoutTemplate::Modelling,
        default_surface_keys: &[
            "runenwerk.scene.viewport",
            "runenwerk.scene.outliner",
            "runenwerk.scene.inspector",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::Scene, DocumentKind::SdfBrushLayer],
    },
    WorkspaceProfileSpec {
        id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
        label: "Editor Design",
        default_layout_template: WorkspaceLayoutTemplate::EditorDesign,
        default_surface_keys: tool_suites::EDITOR_DESIGN_SURFACE_KEYS,
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[
            DocumentKind::UiLayout,
            DocumentKind::WorkspaceDefinition,
            DocumentKind::Theme,
            DocumentKind::Shortcut,
            DocumentKind::Menu,
            DocumentKind::CommandBinding,
            DocumentKind::PanelRegistry,
            DocumentKind::ToolSurfaceDefinition,
        ],
    },
    WorkspaceProfileSpec {
        id: FIELD_WORLD_WORKSPACE_PROFILE_ID,
        label: "Field World",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.field_world.layer_stack",
            "runenwerk.field_world.sdf_graph_canvas",
            "runenwerk.field_world.product_viewer",
            "runenwerk.field_world.sdf_brush_browser",
            "runenwerk.diagnostics.diagnostics",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[
            DocumentKind::Scene,
            DocumentKind::SdfGraph,
            DocumentKind::SdfBrushLayer,
            DocumentKind::FieldWorldDefinition,
            DocumentKind::FieldProductPreview,
        ],
    },
    WorkspaceProfileSpec {
        id: MATERIAL_WORKSPACE_PROFILE_ID,
        label: "Materials",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: MATERIAL_PROFILE_SURFACE_KEYS,
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[
            DocumentKind::Scene,
            DocumentKind::MaterialGraph,
            DocumentKind::Material,
        ],
    },
    WorkspaceProfileSpec {
        id: TEXTURE_WORKSPACE_PROFILE_ID,
        label: "Textures",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.texture.viewer_2d",
            "runenwerk.texture.viewer_3d",
            "runenwerk.diagnostics.diagnostics",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::ProceduralTexture, DocumentKind::VolumeTexture],
    },
    WorkspaceProfileSpec {
        id: PROCGEN_WORKSPACE_PROFILE_ID,
        label: "Procedural Generation",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.procgen.graph_canvas",
            "runenwerk.procgen.preview",
            "runenwerk.diagnostics.diagnostics",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::ProceduralGenerationGraph],
    },
    WorkspaceProfileSpec {
        id: GAMEPLAY_WORKSPACE_PROFILE_ID,
        label: "Gameplay Graph",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.gameplay.graph_canvas",
            "runenwerk.gameplay.compiler_diagnostics",
            "runenwerk.diagnostics.runtime_debug",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[
            DocumentKind::GameplayGraph,
            DocumentKind::GameplayRuleTrigger,
            DocumentKind::Ability,
            DocumentKind::Quest,
        ],
    },
    WorkspaceProfileSpec {
        id: PARTICLE_WORKSPACE_PROFILE_ID,
        label: "Particles",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.particle.graph_canvas",
            "runenwerk.particle.preview",
            "runenwerk.diagnostics.diagnostics",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::ParticleGraph, DocumentKind::ParticleEmitter],
    },
    WorkspaceProfileSpec {
        id: PHYSICS_WORKSPACE_PROFILE_ID,
        label: "Physics",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.physics.authoring",
            "runenwerk.physics.debug",
            "runenwerk.diagnostics.runtime_debug",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::PhysicsScene, DocumentKind::PhysicsConfig],
    },
    WorkspaceProfileSpec {
        id: ANIMATION_WORKSPACE_PROFILE_ID,
        label: "Animation",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.animation.timeline",
            "runenwerk.animation.curve_editor",
            "runenwerk.animation.graph_canvas",
            "runenwerk.diagnostics.diagnostics",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[
            DocumentKind::AnimationClip,
            DocumentKind::AnimationGraph,
            DocumentKind::Timeline,
        ],
    },
    WorkspaceProfileSpec {
        id: SIMULATION_WORKSPACE_PROFILE_ID,
        label: "Simulation Processes",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.simulation.preview",
            "runenwerk.simulation.diagnostics",
            "runenwerk.diagnostics.runtime_debug",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[
            DocumentKind::FieldWorldDefinition,
            DocumentKind::FieldProductPreview,
            DocumentKind::RuntimeDebug,
        ],
    },
    WorkspaceProfileSpec {
        id: RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
        label: "Runtime Debug",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: RUNTIME_DEBUG_PROFILE_SURFACE_KEYS,
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::RuntimeDebug, DocumentKind::Scene],
    },
    WorkspaceProfileSpec {
        id: GRAPH_WORKSPACE_PROFILE_ID,
        label: "Graph",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: &[
            "runenwerk.assets.browser",
            "runenwerk.graph.canvas",
            "runenwerk.diagnostics.diagnostics",
            "runenwerk.editor.console",
        ],
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::Graph],
    },
];

const MATERIAL_LAB_PROFILE_SPECS: &[WorkspaceProfileSpec] = &[WorkspaceProfileSpec {
    id: MATERIAL_WORKSPACE_PROFILE_ID,
    label: "Materials",
    default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
    default_surface_keys: MATERIAL_PROFILE_SURFACE_KEYS,
    default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
    document_kind_filters: &[
        DocumentKind::Scene,
        DocumentKind::MaterialGraph,
        DocumentKind::Material,
    ],
}];

const UI_DESIGNER_PROFILE_SPECS: &[WorkspaceProfileSpec] = &[WorkspaceProfileSpec {
    id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
    label: "UI Designer",
    default_layout_template: WorkspaceLayoutTemplate::EditorDesign,
    default_surface_keys: tool_suites::EDITOR_DESIGN_SURFACE_KEYS,
    default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
    document_kind_filters: &[
        DocumentKind::UiLayout,
        DocumentKind::WorkspaceDefinition,
        DocumentKind::Theme,
        DocumentKind::Shortcut,
        DocumentKind::Menu,
        DocumentKind::CommandBinding,
        DocumentKind::PanelRegistry,
        DocumentKind::ToolSurfaceDefinition,
    ],
}];

const HEADLESS_VALIDATION_PROFILE_SPECS: &[WorkspaceProfileSpec] = &[WorkspaceProfileSpec {
    id: RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
    label: "Runtime Debug",
    default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
    default_surface_keys: RUNTIME_DEBUG_PROFILE_SURFACE_KEYS,
    default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
    document_kind_filters: &[DocumentKind::RuntimeDebug, DocumentKind::Scene],
}];

pub(crate) fn full_editor_profiles() -> Vec<WorkspaceProfileManifest> {
    FULL_EDITOR_PROFILE_SPECS
        .iter()
        .filter(|spec| full_editor_supports_profile_id(spec.id))
        .map(profile_manifest)
        .collect()
}

fn full_editor_supports_profile_id(profile_id: WorkspaceProfileId) -> bool {
    profile_id == SCENE_WORKSPACE_PROFILE_ID
        || profile_id == MODELLING_WORKSPACE_PROFILE_ID
        || profile_id == EDITOR_DESIGN_WORKSPACE_PROFILE_ID
        || profile_id == FIELD_WORLD_WORKSPACE_PROFILE_ID
        || profile_id == MATERIAL_WORKSPACE_PROFILE_ID
        || profile_id == TEXTURE_WORKSPACE_PROFILE_ID
        || profile_id == PROCGEN_WORKSPACE_PROFILE_ID
        || profile_id == RUNTIME_DEBUG_WORKSPACE_PROFILE_ID
}

pub(crate) fn material_lab_profiles() -> Vec<WorkspaceProfileManifest> {
    MATERIAL_LAB_PROFILE_SPECS
        .iter()
        .map(profile_manifest)
        .collect()
}

pub(crate) fn ui_designer_profiles() -> Vec<WorkspaceProfileManifest> {
    UI_DESIGNER_PROFILE_SPECS
        .iter()
        .map(profile_manifest)
        .collect()
}

pub(crate) fn headless_validation_profiles() -> Vec<WorkspaceProfileManifest> {
    HEADLESS_VALIDATION_PROFILE_SPECS
        .iter()
        .map(profile_manifest)
        .collect()
}

pub(crate) fn scene_profile_ref() -> ProfileRef {
    workspace_profile_ref_for_id(SCENE_WORKSPACE_PROFILE_ID)
}

pub(crate) fn material_profile_ref() -> ProfileRef {
    workspace_profile_ref_for_id(MATERIAL_WORKSPACE_PROFILE_ID)
}

pub(crate) fn ui_designer_profile_ref() -> ProfileRef {
    workspace_profile_ref_for_id(EDITOR_DESIGN_WORKSPACE_PROFILE_ID)
}

pub(crate) fn runtime_debug_profile_ref() -> ProfileRef {
    workspace_profile_ref_for_id(RUNTIME_DEBUG_WORKSPACE_PROFILE_ID)
}

pub(crate) fn custom_profile_ref() -> ProfileRef {
    ProfileRef::new("runenwerk.workspace.custom")
        .expect("compiled-in custom workspace profile ref should be valid")
}

pub(crate) fn custom_profiles_for_tool_suites(
    tool_suites: &[EditorToolSuite],
) -> Vec<WorkspaceProfileManifest> {
    let default_surfaces = tool_suites
        .iter()
        .flat_map(|suite| suite.surfaces.iter())
        .map(|surface| SurfaceRef::new(surface.key.clone()))
        .collect();

    vec![WorkspaceProfileManifest {
        profile_ref: custom_profile_ref(),
        compatibility_id: None,
        label: "Custom".to_string(),
        layout_source: WorkspaceProfileLayoutSource::Template(
            WorkspaceLayoutTemplate::ToolWorkspace,
        ),
        default_surfaces,
        default_modes: vec![EDIT_MODE_ID],
        document_kind_filters: Vec::new(),
    }]
}

fn profile_manifest(spec: &WorkspaceProfileSpec) -> WorkspaceProfileManifest {
    WorkspaceProfileManifest {
        profile_ref: workspace_profile_ref_for_id(spec.id),
        compatibility_id: Some(spec.id),
        label: spec.label.to_string(),
        layout_source: WorkspaceProfileLayoutSource::Template(spec.default_layout_template),
        default_surfaces: spec
            .default_surface_keys
            .iter()
            .map(|stable_key| {
                SurfaceRef::new(
                    ToolSurfaceStableKey::new(*stable_key)
                        .expect("compiled-in workspace profile surface key should be valid"),
                )
            })
            .collect(),
        default_modes: spec.default_modes.to_vec(),
        document_kind_filters: spec.document_kind_filters.to_vec(),
    }
}
