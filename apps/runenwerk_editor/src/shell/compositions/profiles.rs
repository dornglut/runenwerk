//! File: apps/runenwerk_editor/src/shell/compositions/profiles.rs
//! Purpose: Built-in cross-suite Workspace profile manifests.

use editor_core::{
    DocumentKind, EDIT_MODE_ID, ModeId, PLAY_MODE_ID, PREVIEW_MODE_ID, SIMULATE_MODE_ID,
};
use editor_definition::{
    EditorWorkspaceHostDefinition, EditorWorkspaceLayoutDefinition,
    EditorWorkspacePanelTabDefinition, EditorWorkspaceSplitAxisDefinition,
};
use editor_shell::{
    ANIMATION_WORKSPACE_PROFILE_ID, EDITOR_DESIGN_WORKSPACE_PROFILE_ID, EditorToolSuite,
    FIELD_WORLD_WORKSPACE_PROFILE_ID, GAMEPLAY_WORKSPACE_PROFILE_ID, GRAPH_WORKSPACE_PROFILE_ID,
    MATERIAL_WORKSPACE_PROFILE_ID, MODELLING_WORKSPACE_PROFILE_ID, PARTICLE_WORKSPACE_PROFILE_ID,
    PHYSICS_WORKSPACE_PROFILE_ID, PROCGEN_WORKSPACE_PROFILE_ID, PanelKind, ProfileRef,
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

pub(crate) fn full_editor_profiles(
    tool_suites: &[EditorToolSuite],
) -> Vec<WorkspaceProfileManifest> {
    FULL_EDITOR_PROFILE_SPECS
        .iter()
        .filter(|spec| full_editor_supports_profile_id(spec.id))
        .map(|spec| profile_manifest(spec, tool_suites))
        .collect()
}

pub(crate) fn full_editor_profile_refs() -> Vec<ProfileRef> {
    FULL_EDITOR_PROFILE_SPECS
        .iter()
        .filter(|spec| full_editor_supports_profile_id(spec.id))
        .map(|spec| workspace_profile_ref_for_id(spec.id))
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

pub(crate) fn material_lab_profiles(
    tool_suites: &[EditorToolSuite],
) -> Vec<WorkspaceProfileManifest> {
    MATERIAL_LAB_PROFILE_SPECS
        .iter()
        .map(|spec| profile_manifest(spec, tool_suites))
        .collect()
}

pub(crate) fn ui_designer_profiles(
    tool_suites: &[EditorToolSuite],
) -> Vec<WorkspaceProfileManifest> {
    UI_DESIGNER_PROFILE_SPECS
        .iter()
        .map(|spec| profile_manifest(spec, tool_suites))
        .collect()
}

pub(crate) fn headless_validation_profiles(
    tool_suites: &[EditorToolSuite],
) -> Vec<WorkspaceProfileManifest> {
    HEADLESS_VALIDATION_PROFILE_SPECS
        .iter()
        .map(|spec| profile_manifest(spec, tool_suites))
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
    let mut surfaces = Vec::<LayoutSurface>::new();
    for surface in tool_suites.iter().flat_map(|suite| suite.surfaces.iter()) {
        if surfaces
            .iter()
            .any(|candidate| candidate.key == surface.key.as_str())
        {
            continue;
        }
        surfaces.push(LayoutSurface {
            key: surface.key.as_str().to_owned(),
            panel_kind: surface.panel_kind,
        });
    }
    let default_surfaces = surfaces
        .iter()
        .map(|surface| {
            SurfaceRef::new(
                ToolSurfaceStableKey::new(surface.key.clone())
                    .expect("compiled custom tool-suite surface key should be valid"),
            )
        })
        .collect();
    let layout =
        tool_workspace_layout_definition("runenwerk.editor.layout.custom", "Custom", surfaces);

    vec![WorkspaceProfileManifest {
        profile_ref: custom_profile_ref(),
        compatibility_id: None,
        label: "Custom".to_string(),
        layout_source: WorkspaceProfileLayoutSource::AuthoredLayout {
            layout_ref: layout.id.clone(),
            layout,
        },
        default_surfaces,
        default_modes: vec![EDIT_MODE_ID],
        document_kind_filters: Vec::new(),
    }]
}

fn profile_manifest(
    spec: &WorkspaceProfileSpec,
    tool_suites: &[EditorToolSuite],
) -> WorkspaceProfileManifest {
    let layout = built_in_layout_definition(spec, tool_suites);
    WorkspaceProfileManifest {
        profile_ref: workspace_profile_ref_for_id(spec.id),
        compatibility_id: Some(spec.id),
        label: spec.label.to_string(),
        layout_source: WorkspaceProfileLayoutSource::AuthoredLayout {
            layout_ref: layout.id.clone(),
            layout,
        },
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct LayoutSurface {
    key: String,
    panel_kind: PanelKind,
}

fn built_in_layout_definition(
    spec: &WorkspaceProfileSpec,
    tool_suites: &[EditorToolSuite],
) -> EditorWorkspaceLayoutDefinition {
    let id = format!("runenwerk.editor.layout.profile-{}", spec.id.raw());
    match spec.default_layout_template {
        WorkspaceLayoutTemplate::Scene | WorkspaceLayoutTemplate::CurrentFixedEditor => {
            scene_layout_definition(id, spec.label)
        }
        WorkspaceLayoutTemplate::Modelling => modelling_layout_definition(id, spec.label),
        WorkspaceLayoutTemplate::EditorDesign => editor_design_layout_definition(id, spec.label),
        WorkspaceLayoutTemplate::ToolWorkspace => tool_workspace_layout_definition(
            id,
            spec.label,
            spec.default_surface_keys
                .iter()
                .map(|key| installed_layout_surface(tool_suites, key))
                .collect(),
        ),
    }
}

fn installed_layout_surface(tool_suites: &[EditorToolSuite], stable_key: &str) -> LayoutSurface {
    let surface = tool_suites
        .iter()
        .flat_map(|suite| suite.surfaces.iter())
        .find(|surface| surface.key.as_str() == stable_key)
        .unwrap_or_else(|| panic!("compiled-in profile surface must be installed: {stable_key}"));
    LayoutSurface {
        key: surface.key.as_str().to_owned(),
        panel_kind: surface.panel_kind,
    }
}

fn scene_layout_definition(
    id: impl Into<String>,
    label: impl Into<String>,
) -> EditorWorkspaceLayoutDefinition {
    EditorWorkspaceLayoutDefinition {
        id: id.into(),
        label: label.into(),
        root: split(
            "scene.root",
            EditorWorkspaceSplitAxisDefinition::Vertical,
            0.78,
            split(
                "scene.body",
                EditorWorkspaceSplitAxisDefinition::Horizontal,
                0.72,
                stack(
                    "scene.viewport",
                    &[("viewport", tool_suites::SCENE_VIEWPORT_SURFACE_KEY)],
                ),
                split(
                    "scene.sidebar",
                    EditorWorkspaceSplitAxisDefinition::Vertical,
                    0.56,
                    stack(
                        "scene.outliner",
                        &[
                            ("outliner", tool_suites::SCENE_OUTLINER_SURFACE_KEY),
                            ("entity-table", tool_suites::SCENE_ENTITY_TABLE_SURFACE_KEY),
                        ],
                    ),
                    stack(
                        "scene.inspector",
                        &[("inspector", tool_suites::SCENE_INSPECTOR_SURFACE_KEY)],
                    ),
                ),
            ),
            stack(
                "scene.console",
                &[("console", tool_suites::EDITOR_CONSOLE_SURFACE_KEY)],
            ),
        ),
        floating_hosts: Vec::new(),
    }
}

fn modelling_layout_definition(
    id: impl Into<String>,
    label: impl Into<String>,
) -> EditorWorkspaceLayoutDefinition {
    EditorWorkspaceLayoutDefinition {
        id: id.into(),
        label: label.into(),
        root: split(
            "modelling.root",
            EditorWorkspaceSplitAxisDefinition::Vertical,
            0.78,
            split(
                "modelling.body",
                EditorWorkspaceSplitAxisDefinition::Horizontal,
                0.20,
                stack(
                    "modelling.outliner",
                    &[
                        ("outliner", tool_suites::SCENE_OUTLINER_SURFACE_KEY),
                        ("entity-table", tool_suites::SCENE_ENTITY_TABLE_SURFACE_KEY),
                    ],
                ),
                split(
                    "modelling.center-right",
                    EditorWorkspaceSplitAxisDefinition::Horizontal,
                    0.76,
                    stack(
                        "modelling.viewport",
                        &[("viewport", tool_suites::SCENE_VIEWPORT_SURFACE_KEY)],
                    ),
                    stack(
                        "modelling.inspector",
                        &[("inspector", tool_suites::SCENE_INSPECTOR_SURFACE_KEY)],
                    ),
                ),
            ),
            stack(
                "modelling.console",
                &[("console", tool_suites::EDITOR_CONSOLE_SURFACE_KEY)],
            ),
        ),
        floating_hosts: Vec::new(),
    }
}

fn editor_design_layout_definition(
    id: impl Into<String>,
    label: impl Into<String>,
) -> EditorWorkspaceLayoutDefinition {
    EditorWorkspaceLayoutDefinition {
        id: id.into(),
        label: label.into(),
        root: split(
            "editor-design.root",
            EditorWorkspaceSplitAxisDefinition::Vertical,
            0.76,
            split(
                "editor-design.body",
                EditorWorkspaceSplitAxisDefinition::Horizontal,
                0.72,
                stack(
                    "editor-design.outliner",
                    &[
                        (
                            "definition-outliner",
                            "runenwerk.editor_design.definition_outliner",
                        ),
                        ("ui-hierarchy", "runenwerk.editor_design.ui_hierarchy"),
                    ],
                ),
                split(
                    "editor-design.center-right",
                    EditorWorkspaceSplitAxisDefinition::Horizontal,
                    0.68,
                    stack(
                        "editor-design.canvas",
                        &[
                            ("ui-canvas", "runenwerk.editor_design.ui_canvas"),
                            (
                                "dock-layout-preview",
                                "runenwerk.editor_design.dock_layout_preview",
                            ),
                        ],
                    ),
                    stack(
                        "editor-design.inspector",
                        &[
                            ("style-inspector", "runenwerk.editor_design.style_inspector"),
                            ("bindings", "runenwerk.editor_design.bindings"),
                        ],
                    ),
                ),
            ),
            stack(
                "editor-design.validation",
                &[
                    (
                        "definition-validation",
                        "runenwerk.editor_design.definition_validation",
                    ),
                    ("command-diff", "runenwerk.editor_design.command_diff"),
                ],
            ),
        ),
        floating_hosts: Vec::new(),
    }
}

fn tool_workspace_layout_definition(
    id: impl Into<String>,
    label: impl Into<String>,
    surfaces: Vec<LayoutSurface>,
) -> EditorWorkspaceLayoutDefinition {
    let groups = tool_workspace_surface_groups(surfaces);
    EditorWorkspaceLayoutDefinition {
        id: id.into(),
        label: label.into(),
        root: split(
            "tool.root",
            EditorWorkspaceSplitAxisDefinition::Vertical,
            0.76,
            split(
                "tool.body",
                EditorWorkspaceSplitAxisDefinition::Horizontal,
                0.22,
                stack_owned("tool.left", groups.left),
                split(
                    "tool.center-right",
                    EditorWorkspaceSplitAxisDefinition::Horizontal,
                    0.70,
                    stack_owned("tool.primary", groups.primary),
                    stack_owned("tool.right", groups.right),
                ),
            ),
            stack_owned("tool.bottom", groups.bottom),
        ),
        floating_hosts: Vec::new(),
    }
}

#[derive(Default)]
struct ToolWorkspaceSurfaceGroups {
    left: Vec<LayoutSurface>,
    primary: Vec<LayoutSurface>,
    right: Vec<LayoutSurface>,
    bottom: Vec<LayoutSurface>,
}

fn tool_workspace_surface_groups(surfaces: Vec<LayoutSurface>) -> ToolWorkspaceSurfaceGroups {
    let mut groups = ToolWorkspaceSurfaceGroups::default();
    let mut unique = Vec::<LayoutSurface>::new();
    for surface in surfaces {
        if !unique.iter().any(|candidate| candidate.key == surface.key) {
            unique.push(surface);
        }
    }
    if unique.is_empty() {
        unique.push(LayoutSurface {
            key: "runenwerk.diagnostics.placeholder".to_owned(),
            panel_kind: PanelKind::Placeholder,
        });
    }

    for surface in unique {
        match surface.panel_kind {
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

    ensure_group(
        &mut groups.primary,
        "runenwerk.diagnostics.placeholder",
        PanelKind::Placeholder,
    );
    ensure_group(
        &mut groups.right,
        "runenwerk.diagnostics.placeholder",
        PanelKind::Placeholder,
    );
    ensure_group(
        &mut groups.left,
        "runenwerk.diagnostics.placeholder",
        PanelKind::Placeholder,
    );
    ensure_group(
        &mut groups.bottom,
        tool_suites::EDITOR_CONSOLE_SURFACE_KEY,
        PanelKind::Console,
    );
    groups
}

fn ensure_group(group: &mut Vec<LayoutSurface>, key: &str, panel_kind: PanelKind) {
    if group.is_empty() {
        group.push(LayoutSurface {
            key: key.to_owned(),
            panel_kind,
        });
    }
}

fn split(
    id: impl Into<String>,
    axis: EditorWorkspaceSplitAxisDefinition,
    fraction: f32,
    first: EditorWorkspaceHostDefinition,
    second: EditorWorkspaceHostDefinition,
) -> EditorWorkspaceHostDefinition {
    EditorWorkspaceHostDefinition::Split {
        id: id.into(),
        axis,
        fraction,
        first: Box::new(first),
        second: Box::new(second),
    }
}

fn stack(id: impl Into<String>, tabs: &[(&str, &str)]) -> EditorWorkspaceHostDefinition {
    stack_owned(
        id,
        tabs.iter()
            .map(|(tab_id, surface)| {
                (
                    (*tab_id).to_owned(),
                    LayoutSurface {
                        key: (*surface).to_owned(),
                        panel_kind: PanelKind::Placeholder,
                    },
                )
            })
            .collect(),
    )
}

fn stack_owned<T>(id: impl Into<String>, tabs: Vec<T>) -> EditorWorkspaceHostDefinition
where
    T: Into<LayoutTab>,
{
    let id = id.into();
    let tabs = tabs.into_iter().map(Into::into).collect::<Vec<LayoutTab>>();
    let active_tab = tabs.first().map(|tab| tab.id.clone());
    EditorWorkspaceHostDefinition::TabStack {
        id,
        tabs: tabs
            .into_iter()
            .map(|tab| EditorWorkspacePanelTabDefinition {
                id: tab.id,
                label: tab.label,
                tool_surface: tab.surface,
            })
            .collect(),
        active_tab,
    }
}

struct LayoutTab {
    id: String,
    label: String,
    surface: String,
}

impl From<(String, LayoutSurface)> for LayoutTab {
    fn from((id, surface): (String, LayoutSurface)) -> Self {
        Self {
            label: surface.key.clone(),
            surface: surface.key,
            id,
        }
    }
}

impl From<LayoutSurface> for LayoutTab {
    fn from(surface: LayoutSurface) -> Self {
        let id = surface.key.clone();
        Self {
            label: surface.key.clone(),
            surface: surface.key,
            id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installed_suites() -> Vec<EditorToolSuite> {
        crate::shell::workbench_host::installed_tool_suites()
    }

    fn collect_surfaces(host: &EditorWorkspaceHostDefinition, output: &mut Vec<String>) {
        match host {
            EditorWorkspaceHostDefinition::Split { first, second, .. } => {
                collect_surfaces(first, output);
                collect_surfaces(second, output);
            }
            EditorWorkspaceHostDefinition::TabStack { tabs, .. } => {
                output.extend(tabs.iter().map(|tab| tab.tool_surface.clone()));
            }
        }
    }

    #[test]
    fn built_in_profiles_use_declarative_layout_definitions() {
        let suites = installed_suites();
        for manifest in full_editor_profiles(&suites)
            .into_iter()
            .chain(material_lab_profiles(&suites))
            .chain(ui_designer_profiles(&suites))
            .chain(headless_validation_profiles(&suites))
        {
            assert!(
                matches!(
                    manifest.layout_source,
                    WorkspaceProfileLayoutSource::AuthoredLayout { .. }
                ),
                "{} retained template-only layout authority",
                manifest.label
            );
        }
    }

    #[test]
    fn material_tool_layout_contains_every_declared_default_surface_once() {
        let suites = installed_suites();
        let manifest = material_lab_profiles(&suites).remove(0);
        let WorkspaceProfileLayoutSource::AuthoredLayout { layout, .. } = manifest.layout_source
        else {
            panic!("material profile should use declarative layout");
        };
        let mut actual = Vec::new();
        collect_surfaces(&layout.root, &mut actual);
        for key in MATERIAL_PROFILE_SURFACE_KEYS {
            assert_eq!(
                actual
                    .iter()
                    .filter(|candidate| candidate.as_str() == *key)
                    .count(),
                1,
                "material layout should contain {key} exactly once"
            );
        }
    }

    #[test]
    fn scene_and_editor_design_preserve_fixed_layout_surface_sets() {
        let suites = installed_suites();
        let scene = profile_manifest(&FULL_EDITOR_PROFILE_SPECS[0], &suites);
        let WorkspaceProfileLayoutSource::AuthoredLayout { layout: scene, .. } =
            scene.layout_source
        else {
            panic!("scene profile should use declarative layout");
        };
        let mut scene_surfaces = Vec::new();
        collect_surfaces(&scene.root, &mut scene_surfaces);
        assert_eq!(
            scene_surfaces,
            vec![
                "runenwerk.scene.viewport",
                "runenwerk.scene.outliner",
                "runenwerk.scene.entity_table",
                "runenwerk.scene.inspector",
                "runenwerk.editor.console",
            ]
        );

        let design = profile_manifest(&FULL_EDITOR_PROFILE_SPECS[2], &suites);
        let WorkspaceProfileLayoutSource::AuthoredLayout { layout: design, .. } =
            design.layout_source
        else {
            panic!("editor-design profile should use declarative layout");
        };
        let mut design_surfaces = Vec::new();
        collect_surfaces(&design.root, &mut design_surfaces);
        assert_eq!(
            design_surfaces,
            vec![
                "runenwerk.editor_design.definition_outliner",
                "runenwerk.editor_design.ui_hierarchy",
                "runenwerk.editor_design.ui_canvas",
                "runenwerk.editor_design.dock_layout_preview",
                "runenwerk.editor_design.style_inspector",
                "runenwerk.editor_design.bindings",
                "runenwerk.editor_design.definition_validation",
                "runenwerk.editor_design.command_diff",
            ]
        );
    }

    #[test]
    fn declarative_profiles_preserve_predecessor_workspace_structure() {
        use editor_shell::{
            PanelHostKind, WorkspaceDefaultToolSurface, WorkspaceIdentityAllocator,
            WorkspaceProfile, WorkspaceState,
        };

        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("workbench host should build");
        let suites = installed_suites();

        for spec in FULL_EDITOR_PROFILE_SPECS
            .iter()
            .filter(|spec| full_editor_supports_profile_id(spec.id))
        {
            let profile = host
                .workspace_profile(spec.id)
                .unwrap_or_else(|| panic!("{} profile should be compiled", spec.label));

            let mut declarative_ids = WorkspaceIdentityAllocator::new();
            let declarative_workspace_id = declarative_ids.allocate_workspace_id();
            let declarative = profile
                .build_default_workspace_state_with_registry(
                    declarative_workspace_id,
                    &mut declarative_ids,
                    host.tool_surface_registry(),
                )
                .unwrap_or_else(|error| {
                    panic!("{} declarative layout should form: {error}", spec.label)
                });

            let legacy_surfaces = spec
                .default_surface_keys
                .iter()
                .map(|key| {
                    let installed = installed_layout_surface(&suites, key);
                    WorkspaceDefaultToolSurface::new_with_panel_kind(
                        ToolSurfaceStableKey::new(installed.key)
                            .expect("compiled stable surface key should be valid"),
                        installed.panel_kind,
                    )
                })
                .collect();
            let legacy_profile = WorkspaceProfile::new(
                spec.id,
                spec.label,
                spec.default_layout_template,
                legacy_surfaces,
                spec.default_modes.to_vec(),
                spec.document_kind_filters.to_vec(),
            );
            let mut legacy_ids = WorkspaceIdentityAllocator::new();
            let legacy_workspace_id = legacy_ids.allocate_workspace_id();
            let legacy =
                legacy_profile.build_default_workspace_state(legacy_workspace_id, &mut legacy_ids);

            assert_eq!(
                workspace_structure_signature(&declarative),
                workspace_structure_signature(&legacy),
                "{} declarative layout changed predecessor split/group/order/active-tab structure",
                spec.label
            );
        }

        fn workspace_structure_signature(workspace: &WorkspaceState) -> String {
            fn stack_signature(
                workspace: &WorkspaceState,
                tab_stack_id: editor_shell::TabStackId,
            ) -> String {
                let stack = workspace
                    .tab_stack(tab_stack_id)
                    .expect("referenced tab stack should exist");
                let tabs = stack
                    .ordered_panels
                    .iter()
                    .map(|panel_id| {
                        let panel = workspace
                            .panel(*panel_id)
                            .expect("ordered panel should exist");
                        let stable_key = panel
                            .active_tool_surface
                            .and_then(|surface_id| workspace.tool_surface(surface_id))
                            .map(|surface| surface.stable_surface_key().as_str())
                            .unwrap_or("<none>");
                        format!("{:?}:{stable_key}", panel.panel_kind)
                    })
                    .collect::<Vec<_>>();
                let active = stack
                    .active_panel
                    .and_then(|panel_id| workspace.panel(panel_id))
                    .and_then(|panel| panel.active_tool_surface)
                    .and_then(|surface_id| workspace.tool_surface(surface_id))
                    .map(|surface| surface.stable_surface_key().as_str())
                    .unwrap_or("<none>");
                format!("stack({tabs:?};active={active})")
            }

            fn host_signature(
                workspace: &WorkspaceState,
                host_id: editor_shell::PanelHostId,
            ) -> String {
                let host = workspace.host(host_id).expect("referenced host should exist");
                match host.kind {
                    PanelHostKind::SplitHost(split) => format!(
                        "split({:?},{};{};{})",
                        split.axis,
                        split.fraction.to_bits(),
                        host_signature(workspace, split.first_child),
                        host_signature(workspace, split.second_child)
                    ),
                    PanelHostKind::TabStackHost(tab_host) => {
                        stack_signature(workspace, tab_host.tab_stack_id)
                    }
                    PanelHostKind::FloatingHostPlaceholder(floating) => {
                        let stack = floating
                            .tab_stack_id
                            .map(|tab_stack_id| stack_signature(workspace, tab_stack_id))
                            .unwrap_or_else(|| "none".to_owned());
                        format!(
                            "floating({},{},{},{};{stack})",
                            floating.bounds.x.to_bits(),
                            floating.bounds.y.to_bits(),
                            floating.bounds.width.to_bits(),
                            floating.bounds.height.to_bits()
                        )
                    }
                }
            }

            let root = host_signature(workspace, workspace.root_host_id());
            let mut floating = workspace
                .hosts()
                .filter_map(|host| match host.kind {
                    PanelHostKind::FloatingHostPlaceholder(_) => {
                        Some(host_signature(workspace, host.id))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            floating.sort();
            format!("{root}|floating={floating:?}")
        }
    }

}
