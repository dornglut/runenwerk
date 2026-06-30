//! File: apps/runenwerk_editor/src/shell/workbench_host.rs
//! Purpose: Compiled-in workbench composition boundary for tool suites and surface providers.

use std::{collections::BTreeSet, fmt, sync::Arc};

use editor_core::{DocumentId, DocumentKind};
use editor_shell::{
    EditorToolSuite, HostCapabilityPolicy, PanelInstanceId, ProfileRef, ProviderBundle,
    ProviderBundleError, ProviderFamilyId, ProviderFamilyProviderAssignment,
    ProviderFamilyProviderMap, ProviderFamilyProviderMapError, SurfaceDocumentContext,
    SurfaceProviderDiagnostic, SurfaceProviderId, SurfaceProviderRequest, TabStackId,
    ToolSuiteRegistry, ToolSuiteRegistryError, ToolSurfaceDefinition, ToolSurfaceInstanceId,
    ToolSurfaceRegistry, ToolSurfaceStableKey, WorkbenchCompositionCompileError,
    WorkbenchCompositionCompilerInput, WorkbenchCompositionManifest, WorkspaceProfile,
    WorkspaceProfileId, WorkspaceProfileManifest, WorkspaceProfileRegistry,
    WorkspaceProfileRegistryBackedBuildError, compile_workbench_composition,
};

use crate::material_lab::material_lab_tool_suite;

use super::{
    EditorSurfaceProviderRegistry, SurfaceProviderRegistryError, compositions,
    runenwerk_provider_family_assignments, tool_suites,
};

const PROVIDER_VALIDATION_INSTANCE_RAW_ID: u64 = 1;
const UI_LAB_PROVIDER_BRIDGE_RAW_ID: u64 = 11;
const UI_LAB_PROVIDER_FAMILY_ID: &str = "runenwerk.ui_lab";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunenwerkWorkbenchComposition {
    FullEditor,
    MaterialLab,
    UiDesigner,
    HeadlessValidation,
    Constrained,
    Custom,
}

pub struct RunenwerkWorkbenchHost {
    composition: RunenwerkWorkbenchComposition,
    composition_ref: ProfileRef,
    label: String,
    tool_suite_registry: ToolSuiteRegistry,
    profiles: Vec<WorkspaceProfileManifest>,
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

    pub fn ui_designer() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition(RunenwerkWorkbenchComposition::UiDesigner)
    }

    pub fn headless_validation() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition(RunenwerkWorkbenchComposition::HeadlessValidation)
    }

    pub fn constrained() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_composition(RunenwerkWorkbenchComposition::Constrained)
    }

    pub(crate) fn authored(
        composition_manifest: WorkbenchCompositionManifest,
        workspace_profile_manifests: Vec<WorkspaceProfileManifest>,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        let tool_suites = installed_tool_suites();
        let provider_registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let provider_family_assignments =
            provider_family_assignments_for_composition(&composition_manifest, &tool_suites);
        Self::from_manifest_tool_suites_provider_registry_and_provider_family_assignments(
            RunenwerkWorkbenchComposition::Custom,
            composition_manifest,
            tool_suites,
            provider_registry,
            provider_family_assignments,
            workspace_profile_manifests,
        )
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
            RunenwerkWorkbenchComposition::UiDesigner => (
                ui_designer_workbench_tool_suites(),
                EditorSurfaceProviderRegistry::runenwerk_ui_designer_workbench(),
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
        let host_capability_policy = compositions::host_policy_for(composition);
        Self::from_composition_tool_suites_provider_registry_and_provider_family_assignments(
            composition,
            tool_suites,
            provider_registry,
            provider_family_assignments,
            host_capability_policy,
        )
    }

    #[cfg(test)]
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
        let installed_suite_ids = tool_suites
            .iter()
            .map(|suite| suite.suite_id.clone())
            .collect::<Vec<_>>();
        let mut composition_manifest =
            compositions::composition_manifest_for(composition, installed_suite_ids);
        composition_manifest.host_policy = host_capability_policy;
        let profiles = workspace_profile_manifests_for_composition(composition, &tool_suites);
        Self::from_manifest_tool_suites_provider_registry_and_provider_family_assignments(
            composition,
            composition_manifest,
            tool_suites,
            provider_registry,
            provider_family_assignments,
            profiles,
        )
    }

    fn from_manifest_tool_suites_provider_registry_and_provider_family_assignments(
        composition: RunenwerkWorkbenchComposition,
        composition_manifest: WorkbenchCompositionManifest,
        tool_suites: Vec<EditorToolSuite>,
        provider_registry: EditorSurfaceProviderRegistry,
        provider_family_assignments: Vec<ProviderFamilyProviderAssignment>,
        workspace_profile_manifests: Vec<WorkspaceProfileManifest>,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        for assignment in &provider_family_assignments {
            if !provider_registry.has_provider_id(assignment.provider_id) {
                return Err(RunenwerkWorkbenchHostError::UnknownProviderId {
                    provider_family_id: assignment.provider_family_id.clone(),
                    provider_id: assignment.provider_id,
                });
            }
        }
        let compiled = compile_workbench_composition(WorkbenchCompositionCompilerInput {
            composition_manifest,
            tool_suite_manifests: tool_suites,
            capability_declarations: Vec::new(),
            workspace_profile_manifests,
            provider_assignments: provider_family_assignments,
        })?;
        validate_provider_registry_support(&compiled, &provider_registry)?;
        let parts = compiled.into_parts();
        Ok(Self {
            composition,
            composition_ref: parts.composition_ref,
            label: parts.label,
            tool_suite_registry: parts.tool_suite_registry,
            profiles: parts.profiles,
            workspace_profile_registry: parts.workspace_profile_registry,
            provider_bundle: parts.provider_bundle,
            provider_family_provider_map: parts.provider_family_provider_map,
            host_capability_policy: parts.host_policy,
            provider_registry: Arc::new(provider_registry),
        })
    }

    pub fn composition(&self) -> RunenwerkWorkbenchComposition {
        self.composition
    }

    pub fn composition_ref(&self) -> &ProfileRef {
        &self.composition_ref
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn tool_suite_registry(&self) -> &ToolSuiteRegistry {
        &self.tool_suite_registry
    }

    pub fn tool_surface_registry(&self) -> &ToolSurfaceRegistry {
        self.tool_suite_registry.surfaces()
    }

    pub fn profiles(&self) -> &[WorkspaceProfileManifest] {
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

    pub fn default_workspace_profile_ref(&self) -> &ProfileRef {
        self.workspace_profile_registry.default_profile_ref()
    }

    pub fn workspace_profile_by_ref(&self, profile_ref: &ProfileRef) -> Option<&WorkspaceProfile> {
        self.workspace_profile_registry.profile_by_ref(profile_ref)
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
    vec![
        tool_suites::core_tool_suite::scene_tool_suite(),
        tool_suites::core_tool_suite::editor_core_tool_suite(),
        tool_suites::editor_design_tool_suite::editor_design_tool_suite(),
        tool_suites::asset_tool_suite::asset_tool_suite(),
        tool_suites::field_sdf_tool_suite::field_sdf_tool_suite(),
        tool_suites::diagnostics_tool_suite::diagnostics_tool_suite(),
        tool_suites::ui_lab_tool_suite::ui_lab_tool_suite(),
        tool_suites::texture_tool_suite::texture_tool_suite(),
        tool_suites::procgen_tool_suite::procgen_tool_suite(),
        material_lab_tool_suite(),
    ]
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

fn ui_designer_workbench_tool_suites() -> Vec<EditorToolSuite> {
    vec![
        tool_suites::core_tool_suite::editor_core_tool_suite(),
        tool_suites::editor_design_tool_suite::editor_design_tool_suite(),
        tool_suites::ui_lab_tool_suite::ui_lab_tool_suite(),
    ]
}

fn headless_validation_tool_suites() -> Vec<EditorToolSuite> {
    vec![
        tool_suites::core_tool_suite::editor_core_tool_suite(),
        tool_suites::asset_tool_suite::asset_tool_suite(),
        tool_suites::diagnostics_tool_suite::diagnostics_tool_suite(),
    ]
}

fn provider_family_assignments_for_tool_suites(
    tool_suites: &[EditorToolSuite],
) -> Vec<ProviderFamilyProviderAssignment> {
    let installed_provider_families = tool_suites
        .iter()
        .flat_map(|suite| suite.provider_families.iter())
        .map(|provider_family| provider_family.id.as_str().to_string())
        .collect::<BTreeSet<_>>();

    let mut assignments = runenwerk_provider_family_assignments()
        .into_iter()
        .filter(|assignment| {
            installed_provider_families.contains(assignment.provider_family_id.as_str())
        })
        .collect::<Vec<_>>();

    if installed_provider_families.contains(UI_LAB_PROVIDER_FAMILY_ID) {
        assignments.push(ProviderFamilyProviderAssignment::new(
            ProviderFamilyId::new(UI_LAB_PROVIDER_FAMILY_ID)
                .expect("compiled-in UI Lab provider family should be valid"),
            SurfaceProviderId::try_from_raw(UI_LAB_PROVIDER_BRIDGE_RAW_ID)
                .expect("UI Lab provider bridge id should be non-zero"),
        ));
    }

    assignments
}

fn provider_family_assignments_for_composition(
    composition: &WorkbenchCompositionManifest,
    tool_suites: &[EditorToolSuite],
) -> Vec<ProviderFamilyProviderAssignment> {
    let selected_suite_ids = composition
        .installed_suites
        .iter()
        .map(|suite_id| suite_id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let selected_tool_suites = tool_suites
        .iter()
        .filter(|suite| selected_suite_ids.contains(suite.suite_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    provider_family_assignments_for_tool_suites(&selected_tool_suites)
}

fn workspace_profile_manifests_for_composition(
    composition: RunenwerkWorkbenchComposition,
    tool_suites: &[EditorToolSuite],
) -> Vec<WorkspaceProfileManifest> {
    match composition {
        RunenwerkWorkbenchComposition::FullEditor | RunenwerkWorkbenchComposition::Constrained => {
            compositions::profiles::full_editor_profiles()
        }
        RunenwerkWorkbenchComposition::MaterialLab => {
            compositions::profiles::material_lab_profiles()
        }
        RunenwerkWorkbenchComposition::UiDesigner => compositions::profiles::ui_designer_profiles(),
        RunenwerkWorkbenchComposition::HeadlessValidation => {
            compositions::profiles::headless_validation_profiles()
        }
        RunenwerkWorkbenchComposition::Custom => {
            compositions::profiles::custom_profiles_for_tool_suites(tool_suites)
        }
    }
}

fn validate_provider_registry_support(
    compiled: &editor_shell::CompiledWorkbenchComposition,
    provider_registry: &EditorSurfaceProviderRegistry,
) -> Result<(), RunenwerkWorkbenchHostError> {
    for surface in compiled.tool_surface_registry().iter() {
        let assigned_providers = compiled
            .provider_family_provider_map()
            .providers_for(&surface.provider_family)
            .collect::<Vec<_>>();
        if assigned_providers.is_empty() {
            return Err(RunenwerkWorkbenchHostError::MissingProviderAssignment {
                provider_family_id: surface.provider_family.clone(),
            });
        }
        let mut supports_surface = false;
        for document_context in
            provider_validation_document_contexts(compiled.workspace_profile_registry())
        {
            let request = provider_validation_request(
                surface,
                compiled.workspace_profile_registry().default_profile_id(),
                document_context,
            )?;
            if provider_registry
                .assigned_provider_supports_request(
                    &request,
                    compiled.provider_family_provider_map(),
                )
                .unwrap_or(false)
            {
                supports_surface = true;
                break;
            }
        }
        if !supports_surface {
            return Err(RunenwerkWorkbenchHostError::UnsupportedSurfaceProvider {
                surface_key: surface.key.clone(),
                provider_family_id: surface.provider_family.clone(),
            });
        }
    }

    Ok(())
}

fn provider_validation_request(
    surface: &ToolSurfaceDefinition,
    workspace_profile_id: WorkspaceProfileId,
    document_context: SurfaceDocumentContext,
) -> Result<SurfaceProviderRequest, RunenwerkWorkbenchHostError> {
    let panel_instance_id = PanelInstanceId::try_from_raw(PROVIDER_VALIDATION_INSTANCE_RAW_ID)
        .map_err(
            |_| RunenwerkWorkbenchHostError::InvalidProviderValidationRequestId {
                field: "panel_instance_id",
                raw: PROVIDER_VALIDATION_INSTANCE_RAW_ID,
            },
        )?;
    let tab_stack_id =
        TabStackId::try_from_raw(PROVIDER_VALIDATION_INSTANCE_RAW_ID).map_err(|_| {
            RunenwerkWorkbenchHostError::InvalidProviderValidationRequestId {
                field: "tab_stack_id",
                raw: PROVIDER_VALIDATION_INSTANCE_RAW_ID,
            }
        })?;
    let tool_surface_instance_id =
        ToolSurfaceInstanceId::try_from_raw(PROVIDER_VALIDATION_INSTANCE_RAW_ID).map_err(|_| {
            RunenwerkWorkbenchHostError::InvalidProviderValidationRequestId {
                field: "tool_surface_instance_id",
                raw: PROVIDER_VALIDATION_INSTANCE_RAW_ID,
            }
        })?;
    let mounted_unit_id =
        ui_composition::MountedUnitId::try_from_raw(PROVIDER_VALIDATION_INSTANCE_RAW_ID).map_err(
            |_| RunenwerkWorkbenchHostError::InvalidProviderValidationRequestId {
                field: "mounted_unit_id",
                raw: PROVIDER_VALIDATION_INSTANCE_RAW_ID,
            },
        )?;

    Ok(SurfaceProviderRequest {
        mounted_unit_id,
        unavailable_content_policy: ui_composition::UnavailableContentPolicy::ShowFallback,
        workspace_profile_id,
        document_context,
        panel_instance_id,
        tab_stack_id,
        tool_surface_instance_id,
        stable_surface_key: surface.key.clone(),
        provider_family_id: Some(surface.provider_family.clone()),
        surface_route: Some(surface.route),
        surface_definition_id: editor_shell::PLACEHOLDER_SURFACE_DEFINITION_ID,
        capabilities: surface.capabilities,
    })
}

fn provider_validation_document_contexts(
    workspace_profile_registry: &WorkspaceProfileRegistry,
) -> Vec<SurfaceDocumentContext> {
    let mut document_kinds = Vec::<DocumentKind>::new();
    for profile in workspace_profile_registry.profiles() {
        for document_kind in &profile.document_kind_filters {
            if !document_kinds.contains(document_kind) {
                document_kinds.push(document_kind.clone());
            }
        }
    }

    let mut contexts = vec![SurfaceDocumentContext::NoActiveDocument];
    contexts.extend(document_kinds.into_iter().enumerate().map(|(index, kind)| {
        SurfaceDocumentContext::Resolved {
            document_id: DocumentId(index as u64 + 1),
            document_kind: kind,
        }
    }));
    contexts
}

#[derive(Debug)]
pub enum RunenwerkWorkbenchHostError {
    ToolSuiteRegistry(ToolSuiteRegistryError),
    SurfaceProviderRegistry(SurfaceProviderRegistryError),
    ProviderFamilyProviderMap(ProviderFamilyProviderMapError),
    ProviderBundle(ProviderBundleError),
    WorkbenchComposition(WorkbenchCompositionCompileError),
    WorkspaceProfileRegistry(WorkspaceProfileRegistryBackedBuildError),
    UnknownProviderId {
        provider_family_id: ProviderFamilyId,
        provider_id: SurfaceProviderId,
    },
    MissingProviderAssignment {
        provider_family_id: ProviderFamilyId,
    },
    UnsupportedSurfaceProvider {
        provider_family_id: ProviderFamilyId,
        surface_key: ToolSurfaceStableKey,
    },
    InvalidProviderValidationRequestId {
        field: &'static str,
        raw: u64,
    },
    SurfaceProviderSupport(SurfaceProviderDiagnostic),
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
            Self::MissingProviderAssignment { provider_family_id } => write!(
                f,
                "provider family `{provider_family_id}` has no concrete provider assignment"
            ),
            Self::UnsupportedSurfaceProvider {
                provider_family_id,
                surface_key,
            } => write!(
                f,
                "provider family `{provider_family_id}` has no assigned provider that supports surface `{surface_key}`"
            ),
            Self::InvalidProviderValidationRequestId { field, raw } => write!(
                f,
                "invalid synthetic provider validation request id `{raw}` for `{field}`"
            ),
            Self::SurfaceProviderSupport(diagnostic) => write!(
                f,
                "failed to validate provider support: {}: {}",
                diagnostic.code, diagnostic.message
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

impl From<WorkbenchCompositionCompileError> for RunenwerkWorkbenchHostError {
    fn from(error: WorkbenchCompositionCompileError) -> Self {
        Self::WorkbenchComposition(error)
    }
}

impl From<WorkspaceProfileRegistryBackedBuildError> for RunenwerkWorkbenchHostError {
    fn from(error: WorkspaceProfileRegistryBackedBuildError) -> Self {
        Self::WorkspaceProfileRegistry(error)
    }
}
