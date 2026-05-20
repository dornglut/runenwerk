//! File: apps/runenwerk_editor/src/shell/workbench_host.rs
//! Purpose: Compiled-in workbench composition boundary for tool suites and surface providers.

use std::{collections::BTreeSet, fmt, sync::Arc};

use editor_core::{
    DocumentKind, EDIT_MODE_ID, ModeId, PLAY_MODE_ID, PREVIEW_MODE_ID, SIMULATE_MODE_ID,
};
use editor_shell::{
    ANIMATION_WORKSPACE_PROFILE_ID, EDITOR_DESIGN_WORKSPACE_PROFILE_ID, EditorToolSuite,
    FIELD_WORLD_WORKSPACE_PROFILE_ID, GAMEPLAY_WORKSPACE_PROFILE_ID, GRAPH_WORKSPACE_PROFILE_ID,
    HostCapabilityPolicy, MATERIAL_WORKSPACE_PROFILE_ID, MODELLING_WORKSPACE_PROFILE_ID,
    PARTICLE_WORKSPACE_PROFILE_ID, PHYSICS_WORKSPACE_PROFILE_ID, PROCGEN_WORKSPACE_PROFILE_ID,
    ProfileRef, ProviderBundle, ProviderBundleError, ProviderFamilyProviderAssignment,
    ProviderFamilyProviderMap, ProviderFamilyProviderMapError, RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
    SCENE_WORKSPACE_PROFILE_ID, SIMULATION_WORKSPACE_PROFILE_ID, SurfaceProviderId, SurfaceRef,
    TEXTURE_WORKSPACE_PROFILE_ID, ToolSuiteProfileDefinition, ToolSuiteRegistry,
    ToolSuiteRegistryError, ToolSurfaceRegistry, WorkbenchCompositionBuildError,
    WorkbenchCompositionBuilder, WorkspaceLayoutTemplate, WorkspaceProfile, WorkspaceProfileId,
    WorkspaceProfileRegistry, WorkspaceProfileRegistryBackedBuildError,
};

use crate::material_lab::material_lab_tool_suite;

use super::{
    EditorSurfaceProviderRegistry, SurfaceProviderRegistryError,
    runenwerk_provider_family_assignments,
    tool_suites::{self, runenwerk_shell_tool_suites},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunenwerkWorkbenchComposition {
    FullEditor,
    MaterialLab,
    HeadlessValidation,
    Constrained,
    Custom,
}

pub struct RunenwerkWorkbenchHost {
    composition: RunenwerkWorkbenchComposition,
    tool_suite_registry: ToolSuiteRegistry,
    profiles: Vec<ToolSuiteProfileDefinition>,
    workspace_profile_registry: WorkspaceProfileRegistry,
    provider_bundle: ProviderBundle,
    provider_family_provider_map: ProviderFamilyProviderMap,
    host_capability_policy: HostCapabilityPolicy,
    provider_registry: Arc<EditorSurfaceProviderRegistry>,
}

impl RunenwerkWorkbenchHost {
    pub fn new() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::full_editor()
    }

    pub fn full_editor() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition(RunenwerkWorkbenchComposition::FullEditor)
    }

    pub fn material_lab() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition(RunenwerkWorkbenchComposition::MaterialLab)
    }

    pub fn headless_validation() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition(RunenwerkWorkbenchComposition::HeadlessValidation)
    }

    pub fn constrained() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition(RunenwerkWorkbenchComposition::Constrained)
    }

    fn from_composition(
        composition: RunenwerkWorkbenchComposition,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        let (tool_suites, provider_registry) = match composition {
            RunenwerkWorkbenchComposition::FullEditor => (
                installed_tool_suites(),
                EditorSurfaceProviderRegistry::runenwerk_default(),
            ),
            RunenwerkWorkbenchComposition::MaterialLab => (
                material_lab_workbench_tool_suites(),
                EditorSurfaceProviderRegistry::runenwerk_material_lab_workbench(),
            ),
            RunenwerkWorkbenchComposition::HeadlessValidation => (
                headless_validation_tool_suites(),
                EditorSurfaceProviderRegistry::runenwerk_default(),
            ),
            RunenwerkWorkbenchComposition::Constrained => (
                installed_tool_suites(),
                EditorSurfaceProviderRegistry::runenwerk_default(),
            ),
            RunenwerkWorkbenchComposition::Custom => unreachable!(
                "custom workbench compositions are built from explicit suites and providers"
            ),
        };
        Self::from_composition_tool_suites_and_provider_registry(
            composition,
            tool_suites,
            provider_registry,
        )
    }

    #[cfg(test)]
    pub(crate) fn from_tool_suites_and_provider_registry(
        tool_suites: Vec<EditorToolSuite>,
        provider_registry: EditorSurfaceProviderRegistry,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition_tool_suites_and_provider_registry(
            RunenwerkWorkbenchComposition::Custom,
            tool_suites,
            provider_registry,
        )
    }

    fn from_composition_tool_suites_and_provider_registry(
        composition: RunenwerkWorkbenchComposition,
        tool_suites: Vec<EditorToolSuite>,
        provider_registry: EditorSurfaceProviderRegistry,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        let provider_family_assignments = provider_family_assignments_for_tool_suites(&tool_suites);
        let host_capability_policy = host_capability_policy_for_composition(composition);
        Self::from_composition_tool_suites_provider_registry_and_provider_family_assignments(
            composition,
            tool_suites,
            provider_registry,
            provider_family_assignments,
            host_capability_policy,
        )
    }

    pub(crate) fn from_tool_suites_provider_registry_and_provider_family_assignments(
        tool_suites: Vec<EditorToolSuite>,
        provider_registry: EditorSurfaceProviderRegistry,
        provider_family_assignments: Vec<ProviderFamilyProviderAssignment>,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition_tool_suites_provider_registry_and_provider_family_assignments(
            RunenwerkWorkbenchComposition::Custom,
            tool_suites,
            provider_registry,
            provider_family_assignments,
            HostCapabilityPolicy::allow_all(),
        )
    }

    #[cfg(test)]
    pub(crate) fn with_host_capability_policy(
        mut self,
        host_capability_policy: HostCapabilityPolicy,
    ) -> Self {
        self.host_capability_policy = host_capability_policy;
        self
    }

    fn from_composition_tool_suites_provider_registry_and_provider_family_assignments(
        composition: RunenwerkWorkbenchComposition,
        tool_suites: Vec<EditorToolSuite>,
        provider_registry: EditorSurfaceProviderRegistry,
        provider_family_assignments: Vec<ProviderFamilyProviderAssignment>,
        host_capability_policy: HostCapabilityPolicy,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        for assignment in &provider_family_assignments {
            if !provider_registry.has_provider_id(assignment.provider_id) {
                return Err(RunenwerkWorkbenchHostError::UnknownProviderId {
                    provider_family_id: assignment.provider_family_id.clone(),
                    provider_id: assignment.provider_id,
                });
            }
        }
        let profiles = tool_suite_profile_definitions_for_composition(composition);
        let composition_model = WorkbenchCompositionBuilder::new()
            .with_suites(tool_suites)
            .with_profiles(profiles)
            .with_provider_assignments(provider_family_assignments)
            .with_host_policy(host_capability_policy)
            .build()?;
        let (tool_suite_registry, profiles, provider_bundle, host_capability_policy) =
            composition_model.into_parts();
        let workspace_profile_registry = workspace_profile_registry_for_composition(
            composition,
            &profiles,
            tool_suite_registry.surfaces(),
        )?;
        let provider_family_provider_map = provider_bundle.provider_map(&tool_suite_registry)?;
        Ok(Self {
            composition,
            tool_suite_registry,
            profiles,
            workspace_profile_registry,
            provider_bundle,
            provider_family_provider_map,
            host_capability_policy,
            provider_registry: Arc::new(provider_registry),
        })
    }

    pub fn composition(&self) -> RunenwerkWorkbenchComposition {
        self.composition
    }

    pub fn tool_suite_registry(&self) -> &ToolSuiteRegistry {
        &self.tool_suite_registry
    }

    pub fn tool_surface_registry(&self) -> &ToolSurfaceRegistry {
        self.tool_suite_registry.surfaces()
    }

    pub fn profiles(&self) -> &[ToolSuiteProfileDefinition] {
        &self.profiles
    }

    pub fn workspace_profile_registry(&self) -> &WorkspaceProfileRegistry {
        &self.workspace_profile_registry
    }

    pub fn workspace_profile(&self, profile_id: WorkspaceProfileId) -> Option<&WorkspaceProfile> {
        self.workspace_profile_registry.profile(profile_id)
    }

    pub fn default_workspace_profile(&self) -> Option<&WorkspaceProfile> {
        self.workspace_profile_registry.default_profile()
    }

    pub fn default_workspace_profile_id(&self) -> WorkspaceProfileId {
        self.workspace_profile_registry.default_profile_id()
    }

    pub fn provider_family_provider_map(&self) -> &ProviderFamilyProviderMap {
        &self.provider_family_provider_map
    }

    pub fn provider_bundle(&self) -> &ProviderBundle {
        &self.provider_bundle
    }

    pub fn host_capability_policy(&self) -> &HostCapabilityPolicy {
        &self.host_capability_policy
    }

    pub fn provider_registry(&self) -> &EditorSurfaceProviderRegistry {
        &self.provider_registry
    }

    pub fn provider_registry_handle(&self) -> Arc<EditorSurfaceProviderRegistry> {
        Arc::clone(&self.provider_registry)
    }
}

fn installed_tool_suites() -> Vec<EditorToolSuite> {
    let mut suites = runenwerk_shell_tool_suites();
    suites.push(material_lab_tool_suite());
    suites
}

fn material_lab_workbench_tool_suites() -> Vec<EditorToolSuite> {
    vec![
        tool_suites::core_tool_suite::editor_core_tool_suite(),
        tool_suites::asset_tool_suite::asset_tool_suite(),
        tool_suites::diagnostics_tool_suite::diagnostics_tool_suite(),
        tool_suites::texture_tool_suite::texture_tool_suite(),
        material_lab_tool_suite(),
    ]
}

fn headless_validation_tool_suites() -> Vec<EditorToolSuite> {
    vec![
        tool_suites::core_tool_suite::editor_core_tool_suite(),
        tool_suites::asset_tool_suite::asset_tool_suite(),
        tool_suites::diagnostics_tool_suite::diagnostics_tool_suite(),
    ]
}

fn host_capability_policy_for_composition(
    composition: RunenwerkWorkbenchComposition,
) -> HostCapabilityPolicy {
    match composition {
        RunenwerkWorkbenchComposition::FullEditor
        | RunenwerkWorkbenchComposition::MaterialLab
        | RunenwerkWorkbenchComposition::Custom => HostCapabilityPolicy::allow_all(),
        RunenwerkWorkbenchComposition::HeadlessValidation
        | RunenwerkWorkbenchComposition::Constrained => HostCapabilityPolicy::deny_all(),
    }
}

fn provider_family_assignments_for_tool_suites(
    tool_suites: &[EditorToolSuite],
) -> Vec<ProviderFamilyProviderAssignment> {
    let installed_provider_families = tool_suites
        .iter()
        .flat_map(|suite| suite.provider_families.iter())
        .map(|provider_family| provider_family.id.as_str().to_string())
        .collect::<BTreeSet<_>>();

    runenwerk_provider_family_assignments()
        .into_iter()
        .filter(|assignment| {
            installed_provider_families.contains(assignment.provider_family_id.as_str())
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct CompiledWorkspaceProfileSpec {
    id: WorkspaceProfileId,
    label: &'static str,
    default_layout_template: WorkspaceLayoutTemplate,
    default_surface_keys: &'static [&'static str],
    default_modes: &'static [ModeId],
    document_kind_filters: &'static [DocumentKind],
}

const FULL_EDITOR_PROFILE_SPECS: &[CompiledWorkspaceProfileSpec] = &[
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
        id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
        label: "Editor Design",
        default_layout_template: WorkspaceLayoutTemplate::EditorDesign,
        default_surface_keys: &[
            "runenwerk.editor_design.definition_outliner",
            "runenwerk.editor_design.ui_hierarchy",
            "runenwerk.editor_design.ui_canvas",
            "runenwerk.editor_design.style_inspector",
            "runenwerk.editor_design.bindings",
            "runenwerk.editor_design.dock_layout_preview",
            "runenwerk.editor_design.theme_editor",
            "runenwerk.editor_design.shortcut_editor",
            "runenwerk.editor_design.menu_editor",
            "runenwerk.editor_design.definition_validation",
            "runenwerk.editor_design.command_diff",
        ],
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
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
    CompiledWorkspaceProfileSpec {
        id: RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
        label: "Runtime Debug",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: RUNTIME_DEBUG_PROFILE_SURFACE_KEYS,
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::RuntimeDebug, DocumentKind::Scene],
    },
    CompiledWorkspaceProfileSpec {
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

const MATERIAL_PROFILE_SURFACE_KEYS: &[&str] = &[
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

const MATERIAL_LAB_PROFILE_SPECS: &[CompiledWorkspaceProfileSpec] =
    &[CompiledWorkspaceProfileSpec {
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

const HEADLESS_VALIDATION_PROFILE_SPECS: &[CompiledWorkspaceProfileSpec] =
    &[CompiledWorkspaceProfileSpec {
        id: RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
        label: "Runtime Debug",
        default_layout_template: WorkspaceLayoutTemplate::ToolWorkspace,
        default_surface_keys: RUNTIME_DEBUG_PROFILE_SURFACE_KEYS,
        default_modes: &[EDIT_MODE_ID, PREVIEW_MODE_ID],
        document_kind_filters: &[DocumentKind::RuntimeDebug, DocumentKind::Scene],
    }];

fn tool_suite_profile_definitions_for_composition(
    composition: RunenwerkWorkbenchComposition,
) -> Vec<ToolSuiteProfileDefinition> {
    workspace_profile_specs_for_composition(composition)
        .iter()
        .map(|spec| tool_suite_profile_definition(spec))
        .collect()
}

fn tool_suite_profile_definition(
    spec: &CompiledWorkspaceProfileSpec,
) -> ToolSuiteProfileDefinition {
    ToolSuiteProfileDefinition::new(
        workspace_profile_ref(spec.id),
        spec.label,
        spec.default_surface_keys
            .iter()
            .map(|stable_key| {
                SurfaceRef::new(
                    editor_shell::ToolSurfaceStableKey::new(*stable_key)
                        .expect("compiled-in workspace profile surface key should be valid"),
                )
            })
            .collect(),
    )
}

fn workspace_profile_registry_for_composition(
    composition: RunenwerkWorkbenchComposition,
    profile_definitions: &[ToolSuiteProfileDefinition],
    tool_surface_registry: &ToolSurfaceRegistry,
) -> Result<WorkspaceProfileRegistry, WorkspaceProfileRegistryBackedBuildError> {
    let specs = workspace_profile_specs_for_composition(composition);
    let profiles = profile_definitions
        .iter()
        .filter_map(|definition| {
            let spec = specs
                .iter()
                .find(|spec| workspace_profile_ref(spec.id) == definition.profile_ref)?;
            Some(WorkspaceProfile::from_tool_suite_profile_definition(
                spec.id,
                spec.default_layout_template,
                definition,
                spec.default_modes.to_vec(),
                spec.document_kind_filters.to_vec(),
                tool_surface_registry,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(WorkspaceProfileRegistry::new(
        default_workspace_profile_id_for_composition(composition),
        profiles,
    ))
}

fn default_workspace_profile_id_for_composition(
    composition: RunenwerkWorkbenchComposition,
) -> WorkspaceProfileId {
    match composition {
        RunenwerkWorkbenchComposition::FullEditor
        | RunenwerkWorkbenchComposition::Constrained
        | RunenwerkWorkbenchComposition::Custom => SCENE_WORKSPACE_PROFILE_ID,
        RunenwerkWorkbenchComposition::MaterialLab => MATERIAL_WORKSPACE_PROFILE_ID,
        RunenwerkWorkbenchComposition::HeadlessValidation => RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
    }
}

fn workspace_profile_specs_for_composition(
    composition: RunenwerkWorkbenchComposition,
) -> &'static [CompiledWorkspaceProfileSpec] {
    match composition {
        RunenwerkWorkbenchComposition::FullEditor | RunenwerkWorkbenchComposition::Constrained => {
            FULL_EDITOR_PROFILE_SPECS
        }
        RunenwerkWorkbenchComposition::MaterialLab => MATERIAL_LAB_PROFILE_SPECS,
        RunenwerkWorkbenchComposition::HeadlessValidation => HEADLESS_VALIDATION_PROFILE_SPECS,
        RunenwerkWorkbenchComposition::Custom => &[],
    }
}

fn workspace_profile_ref(profile_id: WorkspaceProfileId) -> ProfileRef {
    let stable_key = if profile_id == SCENE_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.scene"
    } else if profile_id == MODELLING_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.modelling"
    } else if profile_id == EDITOR_DESIGN_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.editor_design"
    } else if profile_id == FIELD_WORLD_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.field_world"
    } else if profile_id == MATERIAL_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.materials"
    } else if profile_id == TEXTURE_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.textures"
    } else if profile_id == PROCGEN_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.procgen"
    } else if profile_id == GAMEPLAY_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.gameplay"
    } else if profile_id == PARTICLE_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.particles"
    } else if profile_id == PHYSICS_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.physics"
    } else if profile_id == ANIMATION_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.animation"
    } else if profile_id == SIMULATION_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.simulation"
    } else if profile_id == RUNTIME_DEBUG_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.runtime_debug"
    } else if profile_id == GRAPH_WORKSPACE_PROFILE_ID {
        "runenwerk.workspace.graph"
    } else {
        panic!(
            "compiled-in workspace profile {} must have a typed profile ref",
            profile_id.raw()
        );
    };

    ProfileRef::new(stable_key).expect("compiled-in workspace profile ref should be valid")
}

#[derive(Debug)]
pub enum RunenwerkWorkbenchHostError {
    ToolSuiteRegistry(ToolSuiteRegistryError),
    SurfaceProviderRegistry(SurfaceProviderRegistryError),
    ProviderFamilyProviderMap(ProviderFamilyProviderMapError),
    ProviderBundle(ProviderBundleError),
    WorkbenchComposition(WorkbenchCompositionBuildError),
    WorkspaceProfileRegistry(WorkspaceProfileRegistryBackedBuildError),
    UnknownProviderId {
        provider_family_id: editor_shell::ProviderFamilyId,
        provider_id: SurfaceProviderId,
    },
}

impl fmt::Display for RunenwerkWorkbenchHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolSuiteRegistry(error) => {
                write!(f, "failed to build tool suite registry: {error}")
            }
            Self::SurfaceProviderRegistry(error) => {
                write!(f, "failed to build surface provider registry: {error:?}")
            }
            Self::ProviderFamilyProviderMap(error) => {
                write!(f, "failed to build provider family provider map: {error}")
            }
            Self::ProviderBundle(error) => {
                write!(f, "failed to build provider bundle: {error}")
            }
            Self::WorkbenchComposition(error) => {
                write!(f, "failed to build Workbench composition: {error}")
            }
            Self::WorkspaceProfileRegistry(error) => {
                write!(f, "failed to build workspace profile registry: {error}")
            }
            Self::UnknownProviderId {
                provider_family_id,
                provider_id,
            } => write!(
                f,
                "provider family `{provider_family_id}` references unknown provider `{provider_id}`"
            ),
        }
    }
}

impl std::error::Error for RunenwerkWorkbenchHostError {}

impl From<ToolSuiteRegistryError> for RunenwerkWorkbenchHostError {
    fn from(error: ToolSuiteRegistryError) -> Self {
        Self::ToolSuiteRegistry(error)
    }
}

impl From<SurfaceProviderRegistryError> for RunenwerkWorkbenchHostError {
    fn from(error: SurfaceProviderRegistryError) -> Self {
        Self::SurfaceProviderRegistry(error)
    }
}

impl From<ProviderFamilyProviderMapError> for RunenwerkWorkbenchHostError {
    fn from(error: ProviderFamilyProviderMapError) -> Self {
        Self::ProviderFamilyProviderMap(error)
    }
}

impl From<ProviderBundleError> for RunenwerkWorkbenchHostError {
    fn from(error: ProviderBundleError) -> Self {
        Self::ProviderBundle(error)
    }
}

impl From<WorkbenchCompositionBuildError> for RunenwerkWorkbenchHostError {
    fn from(error: WorkbenchCompositionBuildError) -> Self {
        match error {
            WorkbenchCompositionBuildError::ToolSuiteRegistry(error) => {
                Self::ToolSuiteRegistry(error)
            }
            WorkbenchCompositionBuildError::ProviderBundle(error) => Self::ProviderBundle(error),
            WorkbenchCompositionBuildError::DuplicateProfileRef { .. }
            | WorkbenchCompositionBuildError::UnknownProfileDefaultSurface { .. } => {
                Self::WorkbenchComposition(error)
            }
        }
    }
}

impl From<WorkspaceProfileRegistryBackedBuildError> for RunenwerkWorkbenchHostError {
    fn from(error: WorkspaceProfileRegistryBackedBuildError) -> Self {
        Self::WorkspaceProfileRegistry(error)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use editor_core::DocumentKind;
    use editor_shell::{
        CommandCapabilityKey, MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
        MATERIAL_WORKSPACE_PROFILE_ID, PanelInstanceId, ProviderFamilyId,
        ProviderFamilyProviderAssignment, RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
        SurfaceDocumentContext, SurfaceProviderAvailability, SurfaceProviderId,
        SurfaceProviderRequest, TabStackId, ToolSuiteRegistryError, ToolSurfaceInstanceId,
        ToolSurfaceKind, ToolSurfacePersistence, ToolSurfaceRoute, WorkspaceState,
        saveable_tool_surface_stable_key_candidates, tool_surface_capability_set,
        tool_surface_kind_for_stable_key,
    };
    use ui_theme::ThemeTokens;

    use crate::{
        editor_app::RunenwerkEditorApp,
        material_lab::material_lab_tool_suite,
        shell::{
            RunenwerkEditorShellState, SurfaceProviderBuildContext, SurfaceSessionState,
            mounted_surface_requests, mounted_surface_requests_with_registry,
        },
    };

    use super::*;

    #[test]
    fn workbench_host_builds_with_material_lab_suite_metadata() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let suite = host
            .tool_suite_registry()
            .suites()
            .iter()
            .find(|suite| suite.suite_id.as_str() == "runenwerk.material_lab")
            .expect("Material Lab suite metadata should be installed");

        assert_eq!(suite.provider_families.len(), 1);
        assert_eq!(suite.surfaces.len(), 3);
    }

    #[test]
    fn full_editor_workbench_exposes_validated_typed_profiles() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let profile_refs = host
            .profiles()
            .iter()
            .map(|profile| profile.profile_ref.as_str())
            .collect::<BTreeSet<_>>();

        assert!(profile_refs.contains("runenwerk.workspace.scene"));
        assert!(profile_refs.contains("runenwerk.workspace.materials"));
        assert!(profile_refs.contains("runenwerk.workspace.runtime_debug"));
        for profile in host.profiles() {
            for surface_ref in &profile.default_surfaces {
                assert!(
                    host.tool_surface_registry()
                        .get(surface_ref.key())
                        .is_some(),
                    "{} profile should reference registered surface {}",
                    profile.profile_ref,
                    surface_ref
                );
            }
        }
    }

    #[test]
    fn material_lab_workbench_exposes_material_typed_profile_only() {
        let host = RunenwerkWorkbenchHost::material_lab().expect("host should build");
        let profiles = host.profiles();

        assert_eq!(profiles.len(), 1);
        assert_eq!(
            profiles[0].profile_ref.as_str(),
            "runenwerk.workspace.materials"
        );
        assert_eq!(
            profiles[0]
                .default_surfaces
                .iter()
                .map(|surface| surface.key().as_str())
                .collect::<BTreeSet<_>>(),
            [
                "runenwerk.assets.browser",
                "runenwerk.diagnostics.diagnostics",
                "runenwerk.editor.console",
                "runenwerk.material_lab.graph_canvas",
                "runenwerk.material_lab.inspector",
                "runenwerk.material_lab.preview",
                "runenwerk.texture.viewer_2d",
            ]
            .into_iter()
            .collect::<BTreeSet<_>>()
        );
    }

    #[test]
    fn explicit_workbench_presets_are_composition_data() {
        let full_editor = RunenwerkWorkbenchHost::full_editor().expect("host should build");
        let material_lab = RunenwerkWorkbenchHost::material_lab().expect("host should build");
        let headless_validation =
            RunenwerkWorkbenchHost::headless_validation().expect("host should build");
        let constrained = RunenwerkWorkbenchHost::constrained().expect("host should build");

        assert_eq!(
            full_editor.composition(),
            RunenwerkWorkbenchComposition::FullEditor
        );
        assert_eq!(
            material_lab.composition(),
            RunenwerkWorkbenchComposition::MaterialLab
        );
        assert_eq!(
            headless_validation.composition(),
            RunenwerkWorkbenchComposition::HeadlessValidation
        );
        assert_eq!(
            constrained.composition(),
            RunenwerkWorkbenchComposition::Constrained
        );
        assert_eq!(
            suite_ids(&headless_validation),
            vec![
                "runenwerk.editor",
                "runenwerk.assets",
                "runenwerk.diagnostics",
            ]
        );
        assert_eq!(
            provider_family_ids(&headless_validation),
            [
                "runenwerk.assets",
                "runenwerk.diagnostics",
                "runenwerk.editor"
            ]
            .into_iter()
            .collect::<BTreeSet<_>>()
        );
        assert_eq!(headless_validation.profiles().len(), 1);
        assert_eq!(
            headless_validation.default_workspace_profile_id(),
            RUNTIME_DEBUG_WORKSPACE_PROFILE_ID
        );
        assert_eq!(suite_ids(&constrained), suite_ids(&full_editor));
        assert_eq!(constrained.profiles().len(), full_editor.profiles().len());
        assert_eq!(
            constrained.provider_bundle().assignments().len(),
            full_editor.provider_bundle().assignments().len()
        );

        let mutation_capability =
            CommandCapabilityKey::new("runenwerk.surface.session_mutation").unwrap();
        assert!(
            full_editor
                .host_capability_policy()
                .allows_command(&mutation_capability)
        );
        assert!(
            !constrained
                .host_capability_policy()
                .allows_command(&mutation_capability)
        );
        assert!(
            !headless_validation
                .host_capability_policy()
                .allows_command(&mutation_capability)
        );
    }

    #[test]
    fn workbench_host_exposes_tool_surface_registry() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let key =
            editor_shell::ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap();
        let surface = host
            .tool_surface_registry()
            .get(&key)
            .expect("Material graph canvas metadata should be registered");

        assert_eq!(surface.label, "Material Graph");
        assert_eq!(surface.route, ToolSurfaceRoute::ProviderOwnedGraphCanvas);
    }

    #[test]
    fn material_lab_surfaces_are_registered_in_workbench_host() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let expected = [
            (
                "runenwerk.material_lab.graph_canvas",
                "Material Graph",
                ToolSurfaceRoute::ProviderOwnedGraphCanvas,
            ),
            (
                "runenwerk.material_lab.inspector",
                "Material Inspector",
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            (
                "runenwerk.material_lab.preview",
                "Material Preview",
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
        ];

        for (stable_key, label, route) in expected {
            let key = editor_shell::ToolSurfaceStableKey::new(stable_key).unwrap();
            let surface = host
                .tool_surface_registry()
                .get(&key)
                .expect("Material Lab surface should be registered");

            assert_eq!(surface.label, label);
            assert_eq!(surface.route, route);
            assert_eq!(surface.provider_family.as_str(), "runenwerk.material_lab");
        }
    }

    #[test]
    fn inspector_surface_is_registered_in_workbench_host_surface_registry() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let key = editor_shell::ToolSurfaceStableKey::new(
            crate::shell::tool_suites::TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
        )
        .unwrap();
        let surface = host
            .tool_surface_registry()
            .get(&key)
            .expect("Tool Suite Registry Inspector should be registered");

        assert_eq!(surface.label, "Tool Suite Registry Inspector");
        assert_eq!(surface.provider_family.as_str(), "runenwerk.diagnostics");
        assert_eq!(surface.route, ToolSurfaceRoute::ProviderOwnedLocal);
    }

    #[test]
    fn workbench_host_tool_suite_registry_covers_all_saveable_tool_surfaces() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");

        for candidate in saveable_tool_surface_stable_key_candidates() {
            let key = editor_shell::ToolSurfaceStableKey::new(candidate.stable_key).unwrap();
            assert!(
                host.tool_surface_registry().get(&key).is_some(),
                "missing registered stable key for {:?}: {}",
                candidate.kind,
                candidate.stable_key
            );
        }

        let fallback_key =
            editor_shell::ToolSurfaceStableKey::new("runenwerk.diagnostics.placeholder").unwrap();
        assert!(
            host.tool_surface_registry().get(&fallback_key).is_some(),
            "placeholder fallback key should be registered"
        );
    }

    #[test]
    fn no_duplicate_stable_keys_across_installed_suites() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let mut keys = BTreeSet::new();

        for surface in host.tool_surface_registry().iter() {
            assert!(keys.insert(surface.key.as_str()));
        }
    }

    #[test]
    fn no_duplicate_provider_families_across_installed_suites() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let mut provider_families = BTreeSet::new();

        for suite in host.tool_suite_registry().suites() {
            for provider_family in &suite.provider_families {
                assert!(provider_families.insert(provider_family.id.as_str()));
            }
        }
    }

    #[test]
    fn legacy_reverse_mapping_matches_registered_stable_keys() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");

        for candidate in saveable_tool_surface_stable_key_candidates() {
            let key = editor_shell::ToolSurfaceStableKey::new(candidate.stable_key).unwrap();
            assert!(
                host.tool_surface_registry().get(&key).is_some(),
                "candidate should be registered: {}",
                candidate.stable_key
            );
            assert_eq!(tool_surface_kind_for_stable_key(&key), Some(candidate.kind));
        }
    }

    #[test]
    fn material_lab_suite_still_registered_with_provider_owned_graph_route() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let key =
            editor_shell::ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap();

        let definition = host
            .tool_surface_registry()
            .get(&key)
            .expect("material graph canvas should be registered");

        assert_eq!(definition.route, ToolSurfaceRoute::ProviderOwnedGraphCanvas);
    }

    #[test]
    fn workbench_host_builds_existing_provider_registry() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = material_graph_request();

        let frame = host.provider_registry().resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &SurfaceSessionState::default(),
        );

        assert_eq!(frame.provider_id, Some(surface_provider_id(12)));
        assert_eq!(frame.title, "Material Graph Canvas");
    }

    #[test]
    fn workbench_host_builds_provider_family_provider_map() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let scene_family = ProviderFamilyId::new("runenwerk.scene").unwrap();

        let scene_providers = host
            .provider_family_provider_map()
            .providers_for(&scene_family)
            .collect::<Vec<_>>();

        assert_eq!(
            scene_providers,
            vec![
                surface_provider_id(1),
                surface_provider_id(2),
                surface_provider_id(3),
                surface_provider_id(4),
            ]
        );
    }

    #[test]
    fn workbench_host_provider_family_map_rejects_unknown_provider_id() {
        let material_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        let unknown_provider_id = SurfaceProviderId::try_from_raw(999).unwrap();
        let error = match RunenwerkWorkbenchHost::from_tool_suites_provider_registry_and_provider_family_assignments(
            vec![material_lab_tool_suite()],
            EditorSurfaceProviderRegistry::runenwerk_default(),
            vec![ProviderFamilyProviderAssignment::new(
                material_family.clone(),
                unknown_provider_id,
            )],
        ) {
            Ok(_) => panic!("unknown provider ids should be rejected"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            RunenwerkWorkbenchHostError::UnknownProviderId {
                provider_family_id,
                provider_id,
            } if provider_family_id == material_family && provider_id == unknown_provider_id
        ));
    }

    #[test]
    fn material_lab_provider_family_maps_to_material_providers() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let material_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();

        let providers = host
            .provider_family_provider_map()
            .providers_for(&material_family)
            .collect::<Vec<_>>();

        assert_eq!(
            providers,
            vec![
                surface_provider_id(12),
                surface_provider_id(13),
                surface_provider_id(14),
            ]
        );
    }

    #[test]
    fn material_lab_provider_family_maps_to_three_material_providers() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let material_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();

        let providers = host
            .provider_family_provider_map()
            .providers_for(&material_family)
            .collect::<Vec<_>>();

        assert_eq!(providers.len(), 3);
        assert_eq!(
            providers,
            vec![
                surface_provider_id(12),
                surface_provider_id(13),
                surface_provider_id(14),
            ]
        );
    }

    #[test]
    fn placeholder_future_suite_families_do_not_claim_unimplemented_providers() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");

        for family in [
            "runenwerk.graph",
            "runenwerk.gameplay",
            "runenwerk.particle",
            "runenwerk.physics",
            "runenwerk.animation",
            "runenwerk.simulation",
        ] {
            let family = ProviderFamilyId::new(family).unwrap();
            assert_eq!(
                host.provider_family_provider_map()
                    .providers_for(&family)
                    .count(),
                0,
                "{} should remain metadata-only in Phase 9B",
                family.as_str()
            );
        }
    }

    #[test]
    fn provider_family_map_preserves_provider_order() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let field_family = ProviderFamilyId::new("runenwerk.field_world").unwrap();

        let providers = host
            .provider_family_provider_map()
            .providers_for(&field_family)
            .collect::<Vec<_>>();

        assert_eq!(
            providers,
            vec![
                surface_provider_id(9),
                surface_provider_id(10),
                surface_provider_id(17),
                surface_provider_id(18),
            ]
        );
    }

    #[test]
    fn inspector_provider_is_assigned_to_diagnostics_provider_family() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let diagnostics_family = ProviderFamilyId::new("runenwerk.diagnostics").unwrap();

        let providers = host
            .provider_family_provider_map()
            .providers_for(&diagnostics_family)
            .collect::<Vec<_>>();

        assert_eq!(
            providers,
            vec![surface_provider_id(11), surface_provider_id(21)]
        );
    }

    #[test]
    fn workbench_host_does_not_change_default_provider_registry_behavior() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let default_registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = material_graph_request();
        let session = SurfaceSessionState::default();

        let hosted_frame = host.provider_registry().resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &session,
        );
        let default_frame = default_registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &session,
        );

        assert_eq!(hosted_frame.availability, default_frame.availability);
        assert_eq!(hosted_frame.provider_id, default_frame.provider_id);
        assert_eq!(hosted_frame.title, default_frame.title);
    }

    #[test]
    fn workbench_host_rejects_invalid_duplicate_suite_fixture() {
        let error = match RunenwerkWorkbenchHost::from_tool_suites_and_provider_registry(
            vec![material_lab_tool_suite(), material_lab_tool_suite()],
            EditorSurfaceProviderRegistry::runenwerk_default(),
        ) {
            Ok(_) => panic!("duplicate suites should be rejected"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            RunenwerkWorkbenchHostError::ToolSuiteRegistry(
                ToolSuiteRegistryError::DuplicateToolSuiteId { .. }
            )
        ));
    }

    #[test]
    fn material_lab_suite_remains_metadata_only_not_startup_surface_authority() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let shell_state = RunenwerkEditorShellState::new();
        let legacy_requests =
            mounted_surface_requests(&shell_state, SurfaceDocumentContext::NoActiveDocument);
        let hosted_metadata_requests = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::NoActiveDocument,
            Some(host.tool_surface_registry()),
        );

        assert_eq!(legacy_requests.len(), hosted_metadata_requests.len());
        assert!(
            legacy_requests
                .iter()
                .all(|request| request.provider_family_id.is_none()
                    && request.surface_route.is_none())
        );
        assert!(
            hosted_metadata_requests
                .iter()
                .all(|request| !request.matches_stable_key("runenwerk.material_lab.graph_canvas"))
        );
    }

    #[test]
    fn default_app_uses_workbench_host_provider_registry_boundary() {
        let app = RunenwerkEditorApp::new();

        assert!(
            app.workbench_host()
                .tool_suite_registry()
                .suites()
                .iter()
                .any(|suite| suite.suite_id.as_str() == "runenwerk.material_lab")
        );
        assert!(std::sync::Arc::ptr_eq(
            &app.workbench_host().provider_registry_handle(),
            &app.surface_provider_registry_handle()
        ));
    }

    #[test]
    fn material_lab_workbench_installs_material_support_composition() {
        let host = RunenwerkWorkbenchHost::material_lab().expect("host should build");
        let suite_ids = host
            .tool_suite_registry()
            .suites()
            .iter()
            .map(|suite| suite.suite_id.as_str())
            .collect::<Vec<_>>();
        let provider_ids = host
            .provider_registry()
            .provider_ids()
            .collect::<BTreeSet<_>>();
        let expected_provider_ids = [5, 7, 8, 11, 12, 13, 14, 15, 16, 21]
            .into_iter()
            .map(surface_provider_id)
            .collect::<BTreeSet<_>>();

        assert_eq!(
            host.composition(),
            RunenwerkWorkbenchComposition::MaterialLab
        );
        assert_eq!(
            suite_ids,
            vec![
                "runenwerk.editor",
                "runenwerk.assets",
                "runenwerk.diagnostics",
                "runenwerk.texture",
                "runenwerk.material_lab",
            ]
        );
        assert_eq!(provider_ids, expected_provider_ids);

        let scene_family = ProviderFamilyId::new("runenwerk.scene").unwrap();
        let material_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        assert_eq!(
            host.provider_family_provider_map()
                .providers_for(&scene_family)
                .count(),
            0
        );
        assert_eq!(
            host.provider_family_provider_map()
                .providers_for(&material_family)
                .collect::<Vec<_>>(),
            vec![
                surface_provider_id(12),
                surface_provider_id(13),
                surface_provider_id(14),
            ]
        );
    }

    #[test]
    fn material_lab_workbench_bootstraps_material_workspace_profile() {
        let host = RunenwerkWorkbenchHost::material_lab().expect("host should build");
        let shell_state =
            RunenwerkEditorShellState::new_for_workspace_profile_with_workspace_profile_registry_and_tool_surface_registry(
                MATERIAL_WORKSPACE_PROFILE_ID,
                host.workspace_profile_registry(),
                host.tool_surface_registry(),
            )
            .expect("Material Lab workbench should build material workspace");
        let mounted_keys = shell_state
            .workspace_state()
            .tool_surfaces()
            .map(|surface| surface.stable_surface_key().as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            shell_state.active_workspace_profile_id(),
            MATERIAL_WORKSPACE_PROFILE_ID
        );
        assert_eq!(
            shell_state.open_workspace_profile_ids(),
            &[MATERIAL_WORKSPACE_PROFILE_ID]
        );
        assert_eq!(
            mounted_keys,
            [
                "runenwerk.assets.browser",
                "runenwerk.diagnostics.diagnostics",
                "runenwerk.editor.console",
                "runenwerk.material_lab.graph_canvas",
                "runenwerk.material_lab.inspector",
                "runenwerk.material_lab.preview",
                "runenwerk.texture.viewer_2d",
            ]
            .into_iter()
            .collect::<BTreeSet<_>>()
        );
        assert!(
            shell_state
                .workspace_state()
                .validate_tool_surface_registry_compatibility(host.tool_surface_registry())
                .is_fully_compatible()
        );
    }

    #[test]
    fn material_lab_full_editor_and_standalone_resolve_mounted_material_workspace_surfaces() {
        assert_material_profile_requests_resolve_from_host(
            "full editor",
            RunenwerkEditorApp::new(),
            RunenwerkWorkbenchComposition::FullEditor,
        );
        assert_material_profile_requests_resolve_from_host(
            "standalone Material Lab",
            RunenwerkEditorApp::new_material_lab_workbench(),
            RunenwerkWorkbenchComposition::MaterialLab,
        );
    }

    #[test]
    fn material_lab_workbench_preview_keeps_first_visible_path_sdf_first() {
        let app = RunenwerkEditorApp::new_material_lab_workbench();
        let host = app.workbench_host();
        let shell_state =
            RunenwerkEditorShellState::new_for_workspace_profile_with_workspace_profile_registry_and_tool_surface_registry(
                MATERIAL_WORKSPACE_PROFILE_ID,
                host.workspace_profile_registry(),
                host.tool_surface_registry(),
            )
            .expect("Material Lab workbench should build material workspace");
        let request = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(77),
                document_kind: DocumentKind::MaterialGraph,
            },
            Some(host.tool_surface_registry()),
        )
        .into_iter()
        .find(|request| request.stable_surface_key.as_str() == "runenwerk.material_lab.preview")
        .expect("Material Lab preview surface should be mounted");
        let theme = ThemeTokens::default();
        let frame = host
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &SurfaceSessionState::default(),
                Some(host.provider_family_provider_map()),
            );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        let text = format!("{:?}", frame.artifact.root);
        assert!(text.contains("scene material binding: SDF primitives use scene material slots"));
        assert!(!text.contains("model/mesh preview"));
    }

    #[test]
    fn default_app_builds_workspace_with_workbench_host_registry() {
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new_with_workspace_profile_registry_and_tool_surface_registry(
            app.workbench_host().workspace_profile_registry(),
            app.workbench_host().tool_surface_registry(),
        )
        .expect("default app workspace should build with hosted registry");

        let report = shell_state
            .workspace_state()
            .validate_tool_surface_registry_compatibility(
                app.workbench_host().tool_surface_registry(),
            );
        assert!(report.is_fully_compatible());
    }

    #[test]
    fn default_app_profiles_build_from_stable_key_profile_data() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let material_profile = host
            .workspace_profile_registry()
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");
        let mut allocator = editor_shell::WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let workspace = material_profile
            .build_default_workspace_state_with_registry(
                workspace_id,
                &mut allocator,
                host.tool_surface_registry(),
            )
            .expect("profile should build through hosted registry");

        let profile_keys = material_profile
            .default_surfaces
            .iter()
            .map(|surface| surface.stable_surface_key().as_str())
            .collect::<BTreeSet<_>>();
        assert!(
            workspace
                .tool_surfaces()
                .all(|surface| profile_keys.contains(surface.stable_surface_key().as_str()))
        );
    }

    #[test]
    fn material_lab_profile_layout_order_is_stable_key_primary() {
        let host = RunenwerkWorkbenchHost::material_lab().expect("host should build");
        let material_profile = host
            .workspace_profile_registry()
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should exist");

        let keys = material_profile
            .default_surfaces
            .iter()
            .map(|surface| surface.stable_surface_key().as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            keys,
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
    fn default_workspace_fallback_uses_stable_key_profile_data() {
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new_with_workspace_profile_registry_and_tool_surface_registry(
            app.workbench_host().workspace_profile_registry(),
            app.workbench_host().tool_surface_registry(),
        )
        .expect("default workspace fallback should build with hosted registry");

        assert!(
            shell_state
                .workspace_state()
                .tool_surfaces()
                .all(|surface| app
                    .workbench_host()
                    .tool_surface_registry()
                    .get(surface.stable_surface_key())
                    .is_some())
        );
    }

    #[test]
    fn workbench_host_registry_covers_all_default_profiles() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let profile_registry = host.workspace_profile_registry();

        for profile in profile_registry.profiles() {
            assert!(
                profile
                    .validate_tool_surface_registry_compatibility(host.tool_surface_registry())
                    .unregistered_legacy_surfaces
                    .is_empty(),
                "{} profile should not reference unregistered stable keys",
                profile.label
            );
            assert!(
                profile
                    .validate_tool_surface_registry_compatibility(host.tool_surface_registry())
                    .unmapped_legacy_surfaces
                    .is_empty(),
                "{} profile should not reference unmapped surfaces",
                profile.label
            );
        }
    }

    #[test]
    fn workbench_host_registry_covers_all_default_profile_stable_keys() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let profile_registry = host.workspace_profile_registry();

        for profile in profile_registry.profiles() {
            for surface in &profile.default_surfaces {
                assert!(
                    host.tool_surface_registry()
                        .get(surface.stable_surface_key())
                        .is_some(),
                    "{} profile stable key should be registered: {}",
                    profile.label,
                    surface.stable_surface_key().as_str()
                );
            }
        }
    }

    #[test]
    fn tool_suite_registry_inspector_is_reachable_by_stable_key_profile_or_layout() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let profile = host
            .workspace_profile_registry()
            .profile(RUNTIME_DEBUG_WORKSPACE_PROFILE_ID)
            .expect("runtime debug profile should exist");
        let mut allocator = editor_shell::WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = profile
            .build_default_workspace_state_with_registry(
                workspace_id,
                &mut allocator,
                host.tool_surface_registry(),
            )
            .expect("runtime debug profile should build through hosted registry");
        let inspector_key = editor_shell::ToolSurfaceStableKey::new(
            crate::shell::tool_suites::TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
        )
        .unwrap();

        let inspector_surface = workspace
            .tool_surfaces()
            .find(|surface| surface.stable_surface_key() == &inspector_key)
            .expect("runtime debug profile should mount the inspector by stable key");

        assert_eq!(inspector_surface.stable_surface_key(), &inspector_key);
        assert!(workspace.validate_integrity().is_ok());
    }

    #[test]
    fn inspector_provider_resolves_from_stable_key_only_profile_surface() {
        let app = RunenwerkEditorApp::new();
        let host = app.workbench_host();
        let profile = host
            .workspace_profile_registry()
            .profile(RUNTIME_DEBUG_WORKSPACE_PROFILE_ID)
            .expect("runtime debug profile should exist");
        let mut allocator = editor_shell::WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = profile
            .build_default_workspace_state_with_registry(
                workspace_id,
                &mut allocator,
                host.tool_surface_registry(),
            )
            .expect("runtime debug profile should build through hosted registry");
        let mut shell_state = RunenwerkEditorShellState::new_with_workspace_profile_registry_and_tool_surface_registry(
            host.workspace_profile_registry(),
            host.tool_surface_registry(),
        )
        .expect("shell state should build through hosted registry");
        shell_state.set_active_workspace_profile_id(RUNTIME_DEBUG_WORKSPACE_PROFILE_ID);
        shell_state.replace_workspace_state(workspace);
        let request = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::NoActiveDocument,
            Some(host.tool_surface_registry()),
        )
        .into_iter()
        .find(|request| {
            request.stable_surface_key.as_str()
                == crate::shell::tool_suites::TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY
        })
        .expect("inspector mounted surface request should be projected by stable key");
        let theme = ThemeTokens::default();

        assert_eq!(
            request.provider_family_id.as_ref().map(|id| id.as_str()),
            Some("runenwerk.diagnostics")
        );

        let frame = host
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &SurfaceSessionState::default(),
                Some(host.provider_family_provider_map()),
            );

        assert_eq!(
            frame.availability,
            editor_shell::SurfaceProviderAvailability::Available
        );
        assert_eq!(frame.provider_id, Some(surface_provider_id(21)));
        assert_eq!(frame.stable_surface_key, request.stable_surface_key);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn inspector_v5_round_trip_preserves_stable_key_without_legacy_kind() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let profile = host
            .workspace_profile_registry()
            .profile(RUNTIME_DEBUG_WORKSPACE_PROFILE_ID)
            .expect("runtime debug profile should exist");
        let mut allocator = editor_shell::WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = profile
            .build_default_workspace_state_with_registry(
                workspace_id,
                &mut allocator,
                host.tool_surface_registry(),
            )
            .expect("runtime debug profile should build through hosted registry");

        let persisted = workspace
            .to_persisted_v5()
            .expect("runtime debug profile workspace should convert to V5");
        let inspector_persisted = persisted
            .tool_surfaces
            .iter()
            .find(|surface| {
                surface.stable_surface_key
                    == crate::shell::tool_suites::TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY
            })
            .expect("V5 layout should persist the inspector stable key");

        assert_eq!(inspector_persisted.legacy_tool_surface_kind, None);

        let restored =
            WorkspaceState::from_persisted_v5(persisted, Some(host.tool_surface_registry()))
                .expect("V5 layout should reload stable-key-native inspector with registry");

        let inspector_key = editor_shell::ToolSurfaceStableKey::new(
            crate::shell::tool_suites::TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
        )
        .unwrap();
        let restored_surface = restored
            .tool_surfaces()
            .find(|surface| surface.stable_surface_key() == &inspector_key)
            .expect("restored V5 workspace should retain inspector stable key");

        assert_eq!(restored_surface.stable_surface_key(), &inspector_key);
    }

    #[test]
    fn future_placeholder_suites_are_metadata_only_not_domain_implementation() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        for key in [
            "runenwerk.gameplay.graph_canvas",
            "runenwerk.particle.graph_canvas",
            "runenwerk.physics.authoring",
            "runenwerk.animation.graph_canvas",
            "runenwerk.simulation.preview",
        ] {
            let key = editor_shell::ToolSurfaceStableKey::new(key).unwrap();
            let definition = host
                .tool_surface_registry()
                .get(&key)
                .expect("future-facing current placeholder surface should have metadata");
            assert_ne!(definition.route, ToolSurfaceRoute::ProviderOwnedGraphCanvas);
        }
    }

    fn assert_material_profile_requests_resolve_from_host(
        label: &str,
        app: RunenwerkEditorApp,
        expected_composition: RunenwerkWorkbenchComposition,
    ) {
        let host = app.workbench_host();
        let shell_state =
            RunenwerkEditorShellState::new_for_workspace_profile_with_workspace_profile_registry_and_tool_surface_registry(
                MATERIAL_WORKSPACE_PROFILE_ID,
                host.workspace_profile_registry(),
                host.tool_surface_registry(),
            )
            .expect("Material workspace profile should build from the host registries");
        let document_context = SurfaceDocumentContext::Resolved {
            document_id: editor_core::DocumentId(77),
            document_kind: DocumentKind::MaterialGraph,
        };
        let requests = mounted_surface_requests_with_registry(
            &shell_state,
            document_context,
            Some(host.tool_surface_registry()),
        );
        let mounted_keys = requests
            .iter()
            .map(|request| request.stable_surface_key.as_str())
            .collect::<BTreeSet<_>>();
        let expected_keys = MATERIAL_PROFILE_SURFACE_KEYS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let theme = ThemeTokens::default();
        let context = context(&app, &shell_state, &theme);

        assert_eq!(host.composition(), expected_composition, "{label}");
        assert_eq!(
            mounted_keys, expected_keys,
            "{label} should mount the complete Material Lab profile surface set"
        );

        for request in &requests {
            let (expected_provider_family, expected_provider_id, expected_route) =
                expected_material_profile_provider_case(request.stable_surface_key.as_str());
            let definition = host
                .tool_surface_registry()
                .get(&request.stable_surface_key)
                .expect("mounted Material Lab profile surface should be registry-backed");

            assert_eq!(
                definition.persistence,
                ToolSurfacePersistence::StableKey,
                "{label}: {} should be stable-key persisted",
                request.stable_surface_key.as_str()
            );
            assert_eq!(
                request
                    .provider_family_id
                    .as_ref()
                    .map(ProviderFamilyId::as_str),
                Some(expected_provider_family),
                "{label}: {} should carry hosted provider-family metadata",
                request.stable_surface_key.as_str()
            );
            assert_eq!(
                request.surface_route,
                Some(expected_route),
                "{label}: {} should carry hosted route metadata",
                request.stable_surface_key.as_str()
            );

            let frame = host
                .provider_registry()
                .resolve_frame_with_provider_family_map(
                    &context,
                    request,
                    &SurfaceSessionState::default(),
                    Some(host.provider_family_provider_map()),
                );

            assert_eq!(
                frame.availability,
                SurfaceProviderAvailability::Available,
                "{label}: {} should resolve through the active host provider-family map",
                request.stable_surface_key.as_str()
            );
            assert_eq!(
                frame.provider_id,
                Some(expected_provider_id),
                "{label}: {} should select the expected provider",
                request.stable_surface_key.as_str()
            );
            assert_eq!(
                frame.stable_surface_key, request.stable_surface_key,
                "{label}: resolved frame should preserve the stable surface key"
            );
        }
    }

    fn expected_material_profile_provider_case(
        stable_key: &str,
    ) -> (&'static str, SurfaceProviderId, ToolSurfaceRoute) {
        match stable_key {
            "runenwerk.assets.browser" => (
                "runenwerk.assets",
                surface_provider_id(7),
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            "runenwerk.material_lab.graph_canvas" => (
                "runenwerk.material_lab",
                surface_provider_id(12),
                ToolSurfaceRoute::ProviderOwnedGraphCanvas,
            ),
            "runenwerk.material_lab.inspector" => (
                "runenwerk.material_lab",
                surface_provider_id(13),
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            "runenwerk.material_lab.preview" => (
                "runenwerk.material_lab",
                surface_provider_id(14),
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            "runenwerk.texture.viewer_2d" => (
                "runenwerk.texture",
                surface_provider_id(15),
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            "runenwerk.diagnostics.diagnostics" => (
                "runenwerk.diagnostics",
                surface_provider_id(11),
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            "runenwerk.editor.console" => (
                "runenwerk.editor",
                surface_provider_id(5),
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            _ => panic!("unexpected Material Lab profile surface key `{stable_key}`"),
        }
    }

    fn material_graph_request() -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(6),
                document_kind: DocumentKind::MaterialGraph,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(50).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(50).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(50).unwrap(),
            stable_surface_key: editor_shell::ToolSurfaceStableKey::new(
                "runenwerk.material_lab.graph_canvas",
            )
            .unwrap(),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            capabilities: tool_surface_capability_set(ToolSurfaceKind::MaterialGraphCanvas),
        }
    }

    fn context<'a>(
        app: &'a RunenwerkEditorApp,
        shell_state: &'a RunenwerkEditorShellState,
        theme: &'a ThemeTokens,
    ) -> SurfaceProviderBuildContext<'a> {
        SurfaceProviderBuildContext {
            app,
            shell_state,
            theme,
            frame_metrics: None,
            viewport_observations: None,
            tool_surface_bindings: None,
            viewport_instances: None,
        }
    }

    const fn surface_provider_id(raw: u64) -> SurfaceProviderId {
        match SurfaceProviderId::try_from_raw(raw) {
            Ok(id) => id,
            Err(_) => panic!("surface provider ids must be non-zero"),
        }
    }

    fn suite_ids(host: &RunenwerkWorkbenchHost) -> Vec<&str> {
        host.tool_suite_registry()
            .suites()
            .iter()
            .map(|suite| suite.suite_id.as_str())
            .collect()
    }

    fn provider_family_ids(host: &RunenwerkWorkbenchHost) -> BTreeSet<&str> {
        host.provider_bundle()
            .assignments()
            .iter()
            .map(|assignment| assignment.provider_family_id.as_str())
            .collect()
    }
}
