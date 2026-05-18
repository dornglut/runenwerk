//! File: apps/runenwerk_editor/src/shell/workbench_host.rs
//! Purpose: Compiled-in workbench composition boundary for tool suites and surface providers.

use std::{collections::BTreeSet, fmt, sync::Arc};

use editor_shell::{
    EditorToolSuite, ProviderFamilyProviderAssignment, ProviderFamilyProviderMap,
    ProviderFamilyProviderMapError, SurfaceProviderId, ToolSuiteRegistry, ToolSuiteRegistryError,
    ToolSurfaceRegistry,
};

use crate::material_lab::material_lab_tool_suite;

use super::{
    runenwerk_provider_family_assignments, tool_suites::runenwerk_shell_tool_suites,
    EditorSurfaceProviderRegistry, SurfaceProviderRegistryError,
};

pub struct RunenwerkWorkbenchHost {
    tool_suite_registry: ToolSuiteRegistry,
    provider_family_provider_map: ProviderFamilyProviderMap,
    provider_registry: Arc<EditorSurfaceProviderRegistry>,
}

impl RunenwerkWorkbenchHost {
    pub fn new() -> Result<Self, RunenwerkWorkbenchHostError> {
        Self::from_tool_suites_and_provider_registry(
            installed_tool_suites(),
            EditorSurfaceProviderRegistry::runenwerk_default(),
        )
    }

    pub(crate) fn from_tool_suites_and_provider_registry(
        tool_suites: Vec<EditorToolSuite>,
        provider_registry: EditorSurfaceProviderRegistry,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        let provider_family_assignments = provider_family_assignments_for_tool_suites(&tool_suites);
        Self::from_tool_suites_provider_registry_and_provider_family_assignments(
            tool_suites,
            provider_registry,
            provider_family_assignments,
        )
    }

    pub(crate) fn from_tool_suites_provider_registry_and_provider_family_assignments(
        tool_suites: Vec<EditorToolSuite>,
        provider_registry: EditorSurfaceProviderRegistry,
        provider_family_assignments: Vec<ProviderFamilyProviderAssignment>,
    ) -> Result<Self, RunenwerkWorkbenchHostError> {
        let tool_suite_registry = ToolSuiteRegistry::new(tool_suites)?;
        for assignment in &provider_family_assignments {
            if !provider_registry.has_provider_id(assignment.provider_id) {
                return Err(RunenwerkWorkbenchHostError::UnknownProviderId {
                    provider_family_id: assignment.provider_family_id.clone(),
                    provider_id: assignment.provider_id,
                });
            }
        }
        let provider_family_provider_map =
            ProviderFamilyProviderMap::new(&tool_suite_registry, provider_family_assignments)?;
        Ok(Self {
            tool_suite_registry,
            provider_family_provider_map,
            provider_registry: Arc::new(provider_registry),
        })
    }

    pub fn tool_suite_registry(&self) -> &ToolSuiteRegistry {
        &self.tool_suite_registry
    }

    pub fn tool_surface_registry(&self) -> &ToolSurfaceRegistry {
        self.tool_suite_registry.surfaces()
    }

    pub fn provider_family_provider_map(&self) -> &ProviderFamilyProviderMap {
        &self.provider_family_provider_map
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

#[derive(Debug)]
pub enum RunenwerkWorkbenchHostError {
    ToolSuiteRegistry(ToolSuiteRegistryError),
    SurfaceProviderRegistry(SurfaceProviderRegistryError),
    ProviderFamilyProviderMap(ProviderFamilyProviderMapError),
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use editor_core::DocumentKind;
    use editor_shell::{
        default_workspace_profile_registry, saveable_tool_surface_stable_key_candidates,
        tool_surface_capability_set, tool_surface_kind_for_stable_key, PanelInstanceId,
        ProviderFamilyId, ProviderFamilyProviderAssignment, SurfaceDocumentContext,
        SurfaceProviderId, SurfaceProviderRequest, TabStackId, ToolSuiteRegistryError,
        ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceRoute,
        WorkspaceState, MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID, MATERIAL_WORKSPACE_PROFILE_ID,
        RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
    };
    use ui_theme::ThemeTokens;

    use crate::{
        editor_app::RunenwerkEditorApp,
        material_lab::material_lab_tool_suite,
        shell::{
            mounted_surface_requests, mounted_surface_requests_with_registry,
            RunenwerkEditorShellState, SurfaceProviderBuildContext, SurfaceSessionState,
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
        assert!(hosted_metadata_requests
            .iter()
            .all(|request| request.legacy_kind() != Some(ToolSurfaceKind::MaterialGraphCanvas)));
    }

    #[test]
    fn default_app_uses_workbench_host_provider_registry_boundary() {
        let app = RunenwerkEditorApp::new();

        assert!(app
            .workbench_host()
            .tool_suite_registry()
            .suites()
            .iter()
            .any(|suite| suite.suite_id.as_str() == "runenwerk.material_lab"));
        assert!(std::sync::Arc::ptr_eq(
            &app.workbench_host().provider_registry_handle(),
            &app.surface_provider_registry_handle()
        ));
    }

    #[test]
    fn default_app_builds_workspace_with_workbench_host_registry() {
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
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
        let profile_registry = default_workspace_profile_registry();
        let material_profile = profile_registry
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
        assert!(workspace
            .tool_surfaces()
            .all(|surface| profile_keys.contains(surface.stable_surface_key().as_str())));
    }

    #[test]
    fn material_lab_profile_layout_order_is_stable_key_primary() {
        let profile_registry = default_workspace_profile_registry();
        let material_profile = profile_registry
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
        let shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
            app.workbench_host().tool_surface_registry(),
        )
        .expect("default workspace fallback should build with hosted registry");

        assert!(shell_state
            .workspace_state()
            .tool_surfaces()
            .all(|surface| app
                .workbench_host()
                .tool_surface_registry()
                .get(surface.stable_surface_key())
                .is_some()));
    }

    #[test]
    fn workbench_host_registry_covers_all_default_profiles() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let profile_registry = default_workspace_profile_registry();

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
        let profile_registry = default_workspace_profile_registry();

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
        let profile_registry = default_workspace_profile_registry();
        let profile = profile_registry
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

        assert_eq!(inspector_surface.legacy_tool_surface_kind(), None);
        assert!(workspace.validate_integrity().is_ok());
    }

    #[test]
    fn inspector_provider_resolves_from_stable_key_only_profile_surface() {
        let app = RunenwerkEditorApp::new();
        let host = app.workbench_host();
        let profile_registry = default_workspace_profile_registry();
        let profile = profile_registry
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
        let mut shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
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

        assert_eq!(request.legacy_kind(), None);
        assert_eq!(
            request.provider_family_id.as_ref().map(|id| id.as_str()),
            Some("runenwerk.diagnostics")
        );

        let frame = host.provider_registry().resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &SurfaceSessionState::default(),
            Some(host.provider_family_provider_map()),
        );

        assert_eq!(frame.availability, editor_shell::SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(surface_provider_id(21)));
        assert_eq!(frame.stable_surface_key, request.stable_surface_key);
        assert_eq!(frame.surface_kind, None);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn inspector_v5_round_trip_preserves_stable_key_without_legacy_kind() {
        let host = RunenwerkWorkbenchHost::new().expect("host should build");
        let profile_registry = default_workspace_profile_registry();
        let profile = profile_registry
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

        let restored = WorkspaceState::from_persisted_v5(
            persisted,
            Some(host.tool_surface_registry()),
        )
        .expect("V5 layout should reload stable-key-native inspector with registry");

        let inspector_key = editor_shell::ToolSurfaceStableKey::new(
            crate::shell::tool_suites::TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
        )
        .unwrap();
        let restored_surface = restored
            .tool_surfaces()
            .find(|surface| surface.stable_surface_key() == &inspector_key)
            .expect("restored V5 workspace should retain inspector stable key");

        assert_eq!(restored_surface.legacy_tool_surface_kind(), None);
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
            legacy_tool_surface_kind: Some(ToolSurfaceKind::MaterialGraphCanvas),
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
}
