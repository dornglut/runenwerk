//! File: domain/editor/editor_shell/src/workbench/compiler.rs
//! Purpose: Compile app-neutral Workbench manifests into validated registries.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AuthoredToolSurfaceResolution, EditorToolSuite, ProfileRef, ProviderBundle, ProviderFamilyId,
    ProviderFamilyProviderAssignment, ProviderFamilyProviderMap, SurfaceRef,
    ToolSuiteCapabilityDeclaration, ToolSuiteId, ToolSuiteRegistry, ToolSurfaceRegistry,
    ToolSurfaceStableKey, ToolSurfaceTargetProfileCompatibility, WorkspaceDefaultToolSurface,
    WorkspaceIdentityAllocator, WorkspaceProfile, WorkspaceProfileId, WorkspaceProfileRegistry,
    WorkspaceProfileRegistryBackedBuildError, WorkspaceToolSurfaceRegistryCompatibilityReport,
    resolve_authored_tool_surface_reference, workspace::WorkspaceProfileLayoutSource,
};
use editor_definition::{
    EditorWorkspaceFloatingHostDefinition, EditorWorkspaceHostDefinition,
    EditorWorkspaceLayoutDefinition, EditorWorkspacePanelTabDefinition,
};

use super::{
    compiled::CompiledWorkbenchComposition,
    diagnostics::WorkbenchCompositionCompileError,
    manifest::{WorkbenchCompositionManifest, WorkspaceProfileManifest},
};

const AUTHORED_WORKSPACE_PROFILE_ID_START: u64 = 10_000;

#[derive(Debug, Clone)]
pub struct WorkbenchCompositionCompilerInput {
    pub composition_manifest: WorkbenchCompositionManifest,
    pub tool_suite_manifests: Vec<EditorToolSuite>,
    pub capability_declarations: Vec<ToolSuiteCapabilityDeclaration>,
    pub workspace_profile_manifests: Vec<WorkspaceProfileManifest>,
    pub provider_assignments: Vec<ProviderFamilyProviderAssignment>,
}

pub fn compile_workbench_composition(
    input: WorkbenchCompositionCompilerInput,
) -> Result<CompiledWorkbenchComposition, WorkbenchCompositionCompileError> {
    let selected_suites =
        select_tool_suites(&input.composition_manifest, input.tool_suite_manifests)?;
    let selected_profiles = select_workspace_profiles(
        &input.composition_manifest,
        input.workspace_profile_manifests,
    )?;

    let tool_suite_registry = ToolSuiteRegistry::new_with_capability_declarations(
        selected_suites,
        input.capability_declarations,
    )
    .map_err(WorkbenchCompositionCompileError::ToolSuiteRegistry)?;

    validate_profile_default_surfaces(&selected_profiles, tool_suite_registry.surfaces())?;

    let provider_bundle = ProviderBundle::new(&tool_suite_registry, input.provider_assignments)
        .map_err(WorkbenchCompositionCompileError::ProviderBundle)?;
    let provider_family_provider_map = provider_bundle
        .provider_map(&tool_suite_registry)
        .map_err(WorkbenchCompositionCompileError::ProviderBundle)?;
    validate_provider_family_assignments(&tool_suite_registry, &provider_family_provider_map)?;

    let workspace_profile_registry = compile_workspace_profile_registry(
        &input.composition_manifest,
        &selected_profiles,
        tool_suite_registry.surfaces(),
    )?;

    Ok(CompiledWorkbenchComposition::new(
        input.composition_manifest.composition_ref,
        input.composition_manifest.label,
        tool_suite_registry,
        selected_profiles,
        workspace_profile_registry,
        provider_bundle,
        provider_family_provider_map,
        input.composition_manifest.host_policy,
    ))
}

fn select_tool_suites(
    composition: &WorkbenchCompositionManifest,
    suite_manifests: Vec<EditorToolSuite>,
) -> Result<Vec<EditorToolSuite>, WorkbenchCompositionCompileError> {
    let mut suites_by_id = BTreeMap::<ToolSuiteId, EditorToolSuite>::new();
    for suite in suite_manifests {
        if suites_by_id.contains_key(&suite.suite_id) {
            return Err(
                WorkbenchCompositionCompileError::DuplicateInstalledSuiteId {
                    suite_id: suite.suite_id.clone(),
                },
            );
        }
        suites_by_id.insert(suite.suite_id.clone(), suite);
    }

    let mut installed_suite_ids = BTreeSet::<ToolSuiteId>::new();
    let mut selected = Vec::with_capacity(composition.installed_suites.len());
    for suite_id in &composition.installed_suites {
        if !installed_suite_ids.insert(suite_id.clone()) {
            return Err(
                WorkbenchCompositionCompileError::DuplicateInstalledSuiteId {
                    suite_id: suite_id.clone(),
                },
            );
        }
        let suite = suites_by_id.get(suite_id).cloned().ok_or_else(|| {
            WorkbenchCompositionCompileError::UnknownInstalledSuiteId {
                suite_id: suite_id.clone(),
            }
        })?;
        selected.push(suite);
    }
    Ok(selected)
}

fn select_workspace_profiles(
    composition: &WorkbenchCompositionManifest,
    profile_manifests: Vec<WorkspaceProfileManifest>,
) -> Result<Vec<WorkspaceProfileManifest>, WorkbenchCompositionCompileError> {
    let mut profiles_by_ref = BTreeMap::<ProfileRef, WorkspaceProfileManifest>::new();
    let mut compatibility_ids = BTreeSet::<WorkspaceProfileId>::new();
    for profile in profile_manifests {
        if let Some(compatibility_id) = profile.compatibility_id
            && !compatibility_ids.insert(compatibility_id)
        {
            return Err(WorkbenchCompositionCompileError::DuplicateCompatibilityId {
                profile_id: compatibility_id,
            });
        }
        if profiles_by_ref.contains_key(&profile.profile_ref) {
            return Err(WorkbenchCompositionCompileError::DuplicateProfileRef {
                profile_ref: profile.profile_ref.clone(),
            });
        }
        profiles_by_ref.insert(profile.profile_ref.clone(), profile);
    }

    let mut selected_profile_refs = BTreeSet::<ProfileRef>::new();
    if !composition
        .profile_refs
        .contains(&composition.default_profile_ref)
    {
        return Err(
            WorkbenchCompositionCompileError::DefaultProfileRefNotIncluded {
                profile_ref: composition.default_profile_ref.clone(),
            },
        );
    }

    let mut selected = Vec::with_capacity(composition.profile_refs.len());
    for profile_ref in &composition.profile_refs {
        if !selected_profile_refs.insert(profile_ref.clone()) {
            return Err(
                WorkbenchCompositionCompileError::DuplicateCompositionProfileRef {
                    profile_ref: profile_ref.clone(),
                },
            );
        }
        let profile = profiles_by_ref.get(profile_ref).cloned().ok_or_else(|| {
            WorkbenchCompositionCompileError::UnknownCompositionProfileRef {
                profile_ref: profile_ref.clone(),
            }
        })?;
        selected.push(profile);
    }
    Ok(selected)
}

fn validate_profile_default_surfaces(
    profiles: &[WorkspaceProfileManifest],
    registry: &ToolSurfaceRegistry,
) -> Result<(), WorkbenchCompositionCompileError> {
    for profile in profiles {
        for surface_ref in &profile.default_surfaces {
            if registry.get(surface_ref.key()).is_none() {
                return Err(
                    WorkbenchCompositionCompileError::UnknownProfileDefaultSurface {
                        profile_ref: profile.profile_ref.clone(),
                        surface_ref: surface_ref.clone(),
                    },
                );
            }
        }
    }
    Ok(())
}

fn validate_provider_family_assignments(
    registry: &ToolSuiteRegistry,
    provider_map: &ProviderFamilyProviderMap,
) -> Result<(), WorkbenchCompositionCompileError> {
    let mut provider_families = BTreeSet::<ProviderFamilyId>::new();
    for suite in registry.suites() {
        for provider_family in &suite.provider_families {
            provider_families.insert(provider_family.id.clone());
        }
    }

    for provider_family_id in provider_families {
        if provider_map
            .providers_for(&provider_family_id)
            .next()
            .is_none()
        {
            return Err(
                WorkbenchCompositionCompileError::MissingProviderFamilyAssignment {
                    provider_family_id,
                },
            );
        }
    }
    Ok(())
}

fn compile_workspace_profile_registry(
    composition: &WorkbenchCompositionManifest,
    manifests: &[WorkspaceProfileManifest],
    registry: &ToolSurfaceRegistry,
) -> Result<WorkspaceProfileRegistry, WorkbenchCompositionCompileError> {
    let mut used_profile_ids = BTreeSet::<WorkspaceProfileId>::new();
    let mut next_authored_profile_id = AUTHORED_WORKSPACE_PROFILE_ID_START;
    let mut profiles = Vec::with_capacity(manifests.len());

    for manifest in manifests {
        let profile_id = match manifest.compatibility_id {
            Some(profile_id) => {
                if !used_profile_ids.insert(profile_id) {
                    return Err(WorkbenchCompositionCompileError::DuplicateCompatibilityId {
                        profile_id,
                    });
                }
                profile_id
            }
            None => {
                allocate_authored_profile_id(&mut next_authored_profile_id, &mut used_profile_ids)
            }
        };

        let default_surfaces = default_surfaces_for_manifest(manifest, registry)?;
        let profile = WorkspaceProfile::new_with_profile_ref(
            manifest.profile_ref.clone(),
            profile_id,
            manifest.label.clone(),
            manifest.layout_source.clone(),
            default_surfaces,
            manifest.default_modes.clone(),
            manifest.document_kind_filters.clone(),
        );
        validate_profile_layout(&profile, registry)?;
        profiles.push(profile);
    }

    let default_profile_id = profiles
        .iter()
        .find(|profile| profile.profile_ref == composition.default_profile_ref)
        .map(|profile| profile.id)
        .ok_or_else(
            || WorkbenchCompositionCompileError::DefaultProfileRefNotIncluded {
                profile_ref: composition.default_profile_ref.clone(),
            },
        )?;

    Ok(WorkspaceProfileRegistry::new_with_default_ref(
        composition.default_profile_ref.clone(),
        default_profile_id,
        profiles,
    ))
}

fn allocate_authored_profile_id(
    next_authored_profile_id: &mut u64,
    used_profile_ids: &mut BTreeSet<WorkspaceProfileId>,
) -> WorkspaceProfileId {
    loop {
        let profile_id = WorkspaceProfileId::try_from_raw(*next_authored_profile_id)
            .expect("authored workspace profile ids should be non-zero");
        *next_authored_profile_id += 1;
        if used_profile_ids.insert(profile_id) {
            return profile_id;
        }
    }
}

fn default_surfaces_for_manifest(
    manifest: &WorkspaceProfileManifest,
    registry: &ToolSurfaceRegistry,
) -> Result<Vec<WorkspaceDefaultToolSurface>, WorkbenchCompositionCompileError> {
    let surface_refs = if manifest.default_surfaces.is_empty() {
        authored_layout_surface_refs(manifest, registry)?
    } else {
        manifest.default_surfaces.clone()
    };

    surface_refs
        .into_iter()
        .map(|surface_ref| {
            let stable_surface_key = surface_ref.key().clone();
            let definition = registry.get(&stable_surface_key).ok_or_else(|| {
                WorkbenchCompositionCompileError::UnknownProfileDefaultSurface {
                    profile_ref: manifest.profile_ref.clone(),
                    surface_ref: surface_ref.clone(),
                }
            })?;
            Ok(WorkspaceDefaultToolSurface::new_with_panel_kind(
                stable_surface_key,
                definition.panel_kind,
            ))
        })
        .collect()
}

fn authored_layout_surface_refs(
    manifest: &WorkspaceProfileManifest,
    registry: &ToolSurfaceRegistry,
) -> Result<Vec<SurfaceRef>, WorkbenchCompositionCompileError> {
    let WorkspaceProfileLayoutSource::AuthoredLayout { layout_ref, layout } =
        &manifest.layout_source
    else {
        return Ok(Vec::new());
    };

    let mut surface_keys = BTreeSet::<ToolSurfaceStableKey>::new();
    collect_authored_layout_surface_refs(
        &manifest.profile_ref,
        layout_ref,
        layout,
        registry,
        &mut surface_keys,
    )?;
    Ok(surface_keys.into_iter().map(SurfaceRef::new).collect())
}

fn collect_authored_layout_surface_refs(
    profile_ref: &ProfileRef,
    layout_ref: &str,
    layout: &EditorWorkspaceLayoutDefinition,
    registry: &ToolSurfaceRegistry,
    surface_keys: &mut BTreeSet<ToolSurfaceStableKey>,
) -> Result<(), WorkbenchCompositionCompileError> {
    collect_authored_host_surface_refs(
        profile_ref,
        layout_ref,
        &layout.root,
        registry,
        surface_keys,
    )?;
    for floating_host in &layout.floating_hosts {
        collect_authored_floating_host_surface_refs(
            profile_ref,
            layout_ref,
            floating_host,
            registry,
            surface_keys,
        )?;
    }
    Ok(())
}

fn collect_authored_floating_host_surface_refs(
    profile_ref: &ProfileRef,
    layout_ref: &str,
    floating_host: &EditorWorkspaceFloatingHostDefinition,
    registry: &ToolSurfaceRegistry,
    surface_keys: &mut BTreeSet<ToolSurfaceStableKey>,
) -> Result<(), WorkbenchCompositionCompileError> {
    collect_authored_host_surface_refs(
        profile_ref,
        layout_ref,
        &floating_host.host,
        registry,
        surface_keys,
    )
}

fn collect_authored_host_surface_refs(
    profile_ref: &ProfileRef,
    layout_ref: &str,
    host: &EditorWorkspaceHostDefinition,
    registry: &ToolSurfaceRegistry,
    surface_keys: &mut BTreeSet<ToolSurfaceStableKey>,
) -> Result<(), WorkbenchCompositionCompileError> {
    match host {
        EditorWorkspaceHostDefinition::Split { first, second, .. } => {
            collect_authored_host_surface_refs(
                profile_ref,
                layout_ref,
                first,
                registry,
                surface_keys,
            )?;
            collect_authored_host_surface_refs(
                profile_ref,
                layout_ref,
                second,
                registry,
                surface_keys,
            )
        }
        EditorWorkspaceHostDefinition::TabStack { tabs, .. } => {
            for tab in tabs {
                surface_keys.insert(resolve_authored_tab_surface(
                    profile_ref,
                    layout_ref,
                    tab,
                    registry,
                )?);
            }
            Ok(())
        }
    }
}

fn resolve_authored_tab_surface(
    profile_ref: &ProfileRef,
    layout_ref: &str,
    tab: &EditorWorkspacePanelTabDefinition,
    registry: &ToolSurfaceRegistry,
) -> Result<ToolSurfaceStableKey, WorkbenchCompositionCompileError> {
    match resolve_authored_tool_surface_reference(&tab.tool_surface, Some(registry)) {
        AuthoredToolSurfaceResolution::RegistryBacked {
            stable_surface_key, ..
        } => Ok(stable_surface_key),
        AuthoredToolSurfaceResolution::Legacy {
            stable_surface_key, ..
        } => stable_surface_key.ok_or_else(|| {
            WorkbenchCompositionCompileError::UnmappedAuthoredLegacySurface {
                profile_ref: profile_ref.clone(),
                layout_ref: layout_ref.to_string(),
                tab_id: tab.id.clone(),
            }
        }),
        AuthoredToolSurfaceResolution::UnknownStableSurfaceKey { stable_surface_key } => Err(
            WorkbenchCompositionCompileError::UnknownAuthoredLayoutSurface {
                profile_ref: profile_ref.clone(),
                layout_ref: layout_ref.to_string(),
                tab_id: tab.id.clone(),
                stable_surface_key,
            },
        ),
        AuthoredToolSurfaceResolution::UnknownAuthoredSurface { authored_key } => Err(
            WorkbenchCompositionCompileError::UnknownAuthoredLegacySurface {
                profile_ref: profile_ref.clone(),
                layout_ref: layout_ref.to_string(),
                tab_id: tab.id.clone(),
                authored_key,
            },
        ),
    }
}

fn validate_profile_layout(
    profile: &WorkspaceProfile,
    registry: &ToolSurfaceRegistry,
) -> Result<(), WorkbenchCompositionCompileError> {
    let mut allocator = WorkspaceIdentityAllocator::new();
    let workspace_id = allocator.allocate_workspace_id();
    let workspace = match profile.build_default_workspace_state_with_registry(
        workspace_id,
        &mut allocator,
        registry,
    ) {
        Ok(workspace) => workspace,
        Err(WorkspaceProfileRegistryBackedBuildError::WorkspaceDefinitionFormation {
            error,
            ..
        }) => {
            return Err(
                WorkbenchCompositionCompileError::WorkspaceDefinitionFormation {
                    profile_ref: profile.profile_ref.clone(),
                    error: *error,
                },
            );
        }
        Err(error) => {
            return Err(WorkbenchCompositionCompileError::WorkspaceProfileRegistry(
                error,
            ));
        }
    };

    let report: WorkspaceToolSurfaceRegistryCompatibilityReport =
        workspace.validate_tool_surface_registry_compatibility(registry);
    if !report.is_fully_compatible() {
        return Err(WorkbenchCompositionCompileError::WorkspaceProfileRegistry(
            WorkspaceProfileRegistryBackedBuildError::WorkspaceCompatibility {
                profile_id: profile.id,
                report: Box::new(report),
            },
        ));
    }

    let mut surface_keys = profile
        .default_surfaces
        .iter()
        .map(|surface| surface.stable_surface_key().clone())
        .collect::<BTreeSet<_>>();
    surface_keys.extend(
        workspace
            .tool_surfaces()
            .map(|surface| surface.stable_surface_key().clone()),
    );
    for surface_key in surface_keys {
        let Some(definition) = registry.get(&surface_key) else {
            continue;
        };
        if let ToolSurfaceTargetProfileCompatibility::Profiles(profile_refs) =
            &definition.target_profile_compatibility
            && !profile_refs.contains(&profile.profile_ref)
        {
            return Err(
                WorkbenchCompositionCompileError::TargetProfileCompatibility {
                    profile_ref: profile.profile_ref.clone(),
                    surface_key,
                },
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HostCapabilityPolicy, ProviderFamilyDefinition, SuiteRef, SurfaceProviderId,
        ToolSurfaceCreationPolicy, ToolSurfaceDefinition, ToolSurfaceRole, ToolSurfaceRoute,
        WorkspaceLayoutTemplate,
    };
    use editor_core::{DocumentKind, EDIT_MODE_ID};
    use editor_definition::{
        EditorWorkspaceHostDefinition, EditorWorkspaceLayoutDefinition,
        EditorWorkspacePanelTabDefinition,
    };

    #[test]
    fn compiles_template_profile_through_manifest_path() {
        let compiled = compile_workbench_composition(input(
            vec![profile_manifest(
                "runenwerk.workspace.test",
                Some(profile_id(1)),
            )],
            vec![provider_assignment()],
        ))
        .expect("valid manifest composition should compile");

        assert_eq!(compiled.profiles().len(), 1);
        assert_eq!(
            compiled
                .workspace_profile_registry()
                .default_profile_ref()
                .as_str(),
            "runenwerk.workspace.test"
        );
    }

    #[test]
    fn rejects_duplicate_profile_refs() {
        let error = compile_workbench_composition(input(
            vec![
                profile_manifest("runenwerk.workspace.test", Some(profile_id(1))),
                profile_manifest("runenwerk.workspace.test", Some(profile_id(2))),
            ],
            vec![provider_assignment()],
        ))
        .expect_err("duplicate profile refs should reject");

        assert!(matches!(
            error,
            WorkbenchCompositionCompileError::DuplicateProfileRef { .. }
        ));
    }

    #[test]
    fn rejects_default_profile_drift() {
        let mut input = input(
            vec![profile_manifest(
                "runenwerk.workspace.test",
                Some(profile_id(1)),
            )],
            vec![provider_assignment()],
        );
        input.composition_manifest.default_profile_ref =
            ProfileRef::new("runenwerk.workspace.missing").unwrap();

        let error = compile_workbench_composition(input)
            .expect_err("default profile not included should reject");

        assert!(matches!(
            error,
            WorkbenchCompositionCompileError::DefaultProfileRefNotIncluded { .. }
        ));
    }

    #[test]
    fn rejects_registered_provider_family_without_assignment() {
        let error = compile_workbench_composition(input(
            vec![profile_manifest(
                "runenwerk.workspace.test",
                Some(profile_id(1)),
            )],
            Vec::new(),
        ))
        .expect_err("unassigned provider family should reject");

        assert!(matches!(
            error,
            WorkbenchCompositionCompileError::MissingProviderFamilyAssignment { .. }
        ));
    }

    #[test]
    fn rejects_unknown_authored_layout_surface() {
        let error = compile_workbench_composition(input(
            vec![authored_profile_manifest(
                "runenwerk.workspace.authored",
                "runenwerk.test.missing",
            )],
            vec![provider_assignment()],
        ))
        .expect_err("authored layout unknown stable key should reject");

        assert!(matches!(
            error,
            WorkbenchCompositionCompileError::UnknownAuthoredLayoutSurface { .. }
        ));
    }

    #[test]
    fn authored_custom_profile_compiles_through_same_path() {
        let compiled = compile_workbench_composition(input(
            vec![authored_profile_manifest(
                "runenwerk.workspace.authored",
                "runenwerk.test.surface",
            )],
            vec![provider_assignment()],
        ))
        .expect("authored layout should compile through same manifest path");
        let profile = compiled
            .workspace_profile_registry()
            .profile_by_ref(&ProfileRef::new("runenwerk.workspace.authored").unwrap())
            .expect("authored profile should be indexed by ref");

        assert_eq!(profile.profile_ref.as_str(), "runenwerk.workspace.authored");
        assert_eq!(profile.default_surfaces.len(), 1);
    }

    #[test]
    fn compiled_parts_preserve_composition_identity() {
        let compiled = compile_workbench_composition(input(
            vec![profile_manifest(
                "runenwerk.workspace.test",
                Some(profile_id(1)),
            )],
            vec![provider_assignment()],
        ))
        .expect("valid manifest composition should compile");

        let parts = compiled.into_parts();

        assert_eq!(parts.composition_ref.as_str(), "runenwerk.workbench.test");
        assert_eq!(parts.label, "Test Workbench");
    }

    fn input(
        profiles: Vec<WorkspaceProfileManifest>,
        provider_assignments: Vec<ProviderFamilyProviderAssignment>,
    ) -> WorkbenchCompositionCompilerInput {
        let default_profile_ref = profiles[0].profile_ref.clone();
        WorkbenchCompositionCompilerInput {
            composition_manifest: WorkbenchCompositionManifest {
                composition_ref: ProfileRef::new("runenwerk.workbench.test").unwrap(),
                label: "Test Workbench".to_string(),
                installed_suites: vec![ToolSuiteId::new("runenwerk.test").unwrap()],
                profile_refs: profiles
                    .iter()
                    .map(|profile| profile.profile_ref.clone())
                    .collect(),
                default_profile_ref,
                host_policy: HostCapabilityPolicy::allow_all(),
            },
            tool_suite_manifests: vec![suite()],
            capability_declarations: Vec::new(),
            workspace_profile_manifests: profiles,
            provider_assignments,
        }
    }

    fn suite() -> EditorToolSuite {
        let family = ProviderFamilyId::new("runenwerk.test").unwrap();
        EditorToolSuite::new(
            SuiteRef::from_stable_key("runenwerk.test").unwrap(),
            "Test",
            vec![ProviderFamilyDefinition::new(family.clone(), "Test")],
            vec![
                surface(
                    "runenwerk.test.surface",
                    crate::PanelKind::GraphCanvas,
                    family.clone(),
                ),
                surface(
                    "runenwerk.diagnostics.placeholder",
                    crate::PanelKind::Placeholder,
                    family.clone(),
                ),
                surface(
                    "runenwerk.editor.console",
                    crate::PanelKind::Console,
                    family,
                ),
            ],
        )
    }

    fn surface(
        stable_key: &str,
        panel_kind: crate::PanelKind,
        family: ProviderFamilyId,
    ) -> ToolSurfaceDefinition {
        ToolSurfaceDefinition::new(
            SurfaceRef::from_stable_key(stable_key).unwrap(),
            stable_key,
            ToolSurfaceRole::Primary,
            panel_kind,
            family,
            ToolSurfaceRoute::ProviderOwnedLocal,
            ui_surface::SurfaceCapabilitySet::new(true, true, true, false),
            ui_surface::SessionRetentionClass::Restorable,
            ToolSurfaceCreationPolicy::SingletonPerWorkspace,
        )
    }

    fn profile_manifest(
        profile_ref: &str,
        compatibility_id: Option<WorkspaceProfileId>,
    ) -> WorkspaceProfileManifest {
        WorkspaceProfileManifest {
            profile_ref: ProfileRef::new(profile_ref).unwrap(),
            compatibility_id,
            label: "Test".to_string(),
            layout_source: WorkspaceProfileLayoutSource::Template(
                WorkspaceLayoutTemplate::ToolWorkspace,
            ),
            default_surfaces: vec![SurfaceRef::from_stable_key("runenwerk.test.surface").unwrap()],
            default_modes: vec![EDIT_MODE_ID],
            document_kind_filters: vec![DocumentKind::Graph],
        }
    }

    fn authored_profile_manifest(
        profile_ref: &str,
        tool_surface: &str,
    ) -> WorkspaceProfileManifest {
        WorkspaceProfileManifest {
            profile_ref: ProfileRef::new(profile_ref).unwrap(),
            compatibility_id: None,
            label: "Authored".to_string(),
            layout_source: WorkspaceProfileLayoutSource::AuthoredLayout {
                layout_ref: "runenwerk.layout.authored".to_string(),
                layout: EditorWorkspaceLayoutDefinition {
                    id: "runenwerk.layout.authored".to_string(),
                    label: "Authored Layout".to_string(),
                    root: EditorWorkspaceHostDefinition::TabStack {
                        id: "root".to_string(),
                        tabs: vec![EditorWorkspacePanelTabDefinition {
                            id: "tab".to_string(),
                            label: "Tab".to_string(),
                            tool_surface: tool_surface.to_string(),
                        }],
                        active_tab: Some("tab".to_string()),
                    },
                    floating_hosts: Vec::new(),
                },
            },
            default_surfaces: Vec::new(),
            default_modes: vec![EDIT_MODE_ID],
            document_kind_filters: vec![DocumentKind::Graph],
        }
    }

    fn provider_assignment() -> ProviderFamilyProviderAssignment {
        ProviderFamilyProviderAssignment::new(
            ProviderFamilyId::new("runenwerk.test").unwrap(),
            SurfaceProviderId::try_from_raw(1).unwrap(),
        )
    }

    fn profile_id(raw: u64) -> WorkspaceProfileId {
        WorkspaceProfileId::try_from_raw(raw).unwrap()
    }
}
