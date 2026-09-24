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

    runenwerk_provider_family_assignments()
        .into_iter()
        .filter(|assignment| {
            installed_provider_families.contains(assignment.provider_family_id.as_str())
        })
        .collect()
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

impl From<SurfaceProviderDiagnostic> for RunenwerkWorkbenchHostError {
    fn from(diagnostic: SurfaceProviderDiagnostic) -> Self {
        Self::SurfaceProviderSupport(diagnostic)
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use editor_core::DocumentKind;
    use editor_shell::{
        CommandCapabilityKey, MATERIAL_WORKSPACE_PROFILE_ID, ProviderFamilyId,
        ProviderFamilyProviderAssignment, RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
        SurfaceDocumentContext, SurfaceProviderId, SurfaceProviderRequest, ToolSurfaceKind,
        ToolSurfaceRoute, saveable_tool_surface_stable_key_candidates,
        tool_surface_kind_for_stable_key,
    };
    use ui_theme::ThemeTokens;

    use crate::{
        editor_app::RunenwerkEditorApp,
        shell::{
            RunenwerkEditorShellState, SurfaceProviderBuildContext, SurfaceSessionState,
            mounted_surface_requests_with_registry,
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
        let ui_designer = RunenwerkWorkbenchHost::ui_designer().expect("host should build");
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
            ui_designer.composition(),
            RunenwerkWorkbenchComposition::UiDesigner
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
        assert_eq!(
            suite_ids(&ui_designer),
            vec!["runenwerk.editor", "runenwerk.editor_design"]
        );
        assert_eq!(
            provider_family_ids(&ui_designer),
            ["runenwerk.editor", "runenwerk.editor_design"]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );
        assert_eq!(ui_designer.profiles().len(), 1);
        assert_eq!(
            ui_designer.default_workspace_profile_id(),
            editor_shell::EDITOR_DESIGN_WORKSPACE_PROFILE_ID
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
        assert_eq!(surface.provider_family.as_str(), "runenwerk.material_lab");
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
    fn workbench_host_tool_suite_registry_covers_all_saveable_tool_surfaces() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");

        for candidate in saveable_tool_surface_stable_key_candidates() {
            if !provider_backed_candidate_kind(candidate.kind) {
                continue;
            }
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
            if !provider_backed_candidate_kind(candidate.kind) {
                continue;
            }
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
    fn material_lab_provider_family_maps_to_three_material_providers() {
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
                "{} should remain metadata-only until it has an installed provider-backed suite",
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

        let rendered = format!("{error:?}");
        assert!(
            rendered.contains("DuplicateInstalledSuiteId"),
            "duplicate suite fixture should reject with DuplicateInstalledSuiteId, got {rendered}"
        );
    }

    #[test]
    fn material_lab_suite_remains_metadata_only_not_startup_surface_authority() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let shell_state = RunenwerkEditorShellState::new();
        let hosted_metadata_requests = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::NoActiveDocument,
            Some(host.tool_surface_registry()),
        );

        assert!(hosted_metadata_requests.iter().all(|request| {
            request.stable_surface_key.as_str() != "runenwerk.material_lab.graph_canvas"
        }));
    }

    fn material_graph_request() -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            mounted_unit_id: ui_composition::MountedUnitId::try_from_raw(50).unwrap(),
            unavailable_content_policy: ui_composition::UnavailableContentPolicy::ShowFallback,
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
            surface_definition_id: editor_shell::MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            capabilities: editor_shell::tool_surface_capability_set(
                editor_shell::ToolSurfaceKind::MaterialGraphCanvas,
            ),
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

    fn provider_backed_candidate_kind(kind: ToolSurfaceKind) -> bool {
        !matches!(
            kind,
            ToolSurfaceKind::GraphCanvas
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
                | ToolSurfaceKind::SimulationDiagnostics
        )
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
