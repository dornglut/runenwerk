//! File: domain/editor/editor_shell/src/tool_suite/registry.rs
//! Purpose: Validation and lookup for installed editor tool suites.

use std::{collections::BTreeMap, fmt};

use super::{
    EditorToolSuite, HostCapabilityPolicy, ProviderFamilyDefinition, ProviderFamilyId, ToolSuiteId,
    ToolSuiteProfileDefinition, ToolSurfaceDefinition, ToolSurfaceStableKey,
};
use crate::SurfaceProviderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSuiteRegistry {
    suites: Vec<EditorToolSuite>,
    surfaces: ToolSurfaceRegistry,
}

impl ToolSuiteRegistry {
    pub fn new(suites: Vec<EditorToolSuite>) -> Result<Self, ToolSuiteRegistryError> {
        let mut suite_ids = BTreeMap::<ToolSuiteId, usize>::new();
        let mut provider_family_ids = BTreeMap::<ProviderFamilyId, ToolSuiteId>::new();

        for suite in &suites {
            if suite_ids
                .insert(suite.suite_id.clone(), suite_ids.len())
                .is_some()
            {
                return Err(ToolSuiteRegistryError::DuplicateToolSuiteId {
                    suite_id: suite.suite_id.clone(),
                });
            }

            for provider_family in &suite.provider_families {
                if let Some(owner_suite_id) =
                    provider_family_ids.insert(provider_family.id.clone(), suite.suite_id.clone())
                {
                    return Err(ToolSuiteRegistryError::DuplicateProviderFamilyId {
                        provider_family_id: provider_family.id.clone(),
                        first_suite_id: owner_suite_id,
                        duplicate_suite_id: suite.suite_id.clone(),
                    });
                }
            }
        }

        let mut surface_keys = BTreeMap::<ToolSurfaceStableKey, ToolSuiteId>::new();
        let mut ordered_surfaces = Vec::new();

        for suite in &suites {
            let declared_provider_families = suite
                .provider_families
                .iter()
                .map(|definition| definition.id.clone())
                .collect::<Vec<_>>();

            for surface in &suite.surfaces {
                if !declared_provider_families.contains(&surface.provider_family) {
                    return Err(ToolSuiteRegistryError::InvalidProviderFamilyReference {
                        suite_id: suite.suite_id.clone(),
                        surface_key: surface.key.clone(),
                        provider_family_id: surface.provider_family.clone(),
                    });
                }

                if let Some(owner_suite_id) =
                    surface_keys.insert(surface.key.clone(), suite.suite_id.clone())
                {
                    return Err(ToolSuiteRegistryError::DuplicateToolSurfaceStableKey {
                        surface_key: surface.key.clone(),
                        first_suite_id: owner_suite_id,
                        duplicate_suite_id: suite.suite_id.clone(),
                    });
                }

                ordered_surfaces.push(surface.clone());
            }
        }

        Ok(Self {
            suites,
            surfaces: ToolSurfaceRegistry::new(ordered_surfaces),
        })
    }

    pub fn suites(&self) -> &[EditorToolSuite] {
        &self.suites
    }

    pub fn surfaces(&self) -> &ToolSurfaceRegistry {
        &self.surfaces
    }

    pub fn provider_family(&self, id: &ProviderFamilyId) -> Option<&ProviderFamilyDefinition> {
        self.suites
            .iter()
            .flat_map(|suite| suite.provider_families.iter())
            .find(|provider_family| provider_family.id == *id)
    }

    pub fn has_provider_family(&self, id: &ProviderFamilyId) -> bool {
        self.provider_family(id).is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSurfaceRegistry {
    surfaces: Vec<ToolSurfaceDefinition>,
    index_by_key: BTreeMap<ToolSurfaceStableKey, usize>,
}

impl ToolSurfaceRegistry {
    fn new(surfaces: Vec<ToolSurfaceDefinition>) -> Self {
        let index_by_key = surfaces
            .iter()
            .enumerate()
            .map(|(index, surface)| (surface.key.clone(), index))
            .collect();

        Self {
            surfaces,
            index_by_key,
        }
    }

    pub fn get(&self, key: &ToolSurfaceStableKey) -> Option<&ToolSurfaceDefinition> {
        self.index_by_key
            .get(key)
            .and_then(|index| self.surfaces.get(*index))
    }

    pub fn resolve(&self, key: &ToolSurfaceStableKey) -> ToolSurfaceResolution<'_> {
        match self.get(key) {
            Some(definition) => ToolSurfaceResolution::Resolved(definition),
            None => ToolSurfaceResolution::UnknownKey { key: key.clone() },
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ToolSurfaceDefinition> {
        self.surfaces.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolSuiteRegistryError {
    DuplicateToolSuiteId {
        suite_id: ToolSuiteId,
    },
    DuplicateToolSurfaceStableKey {
        surface_key: ToolSurfaceStableKey,
        first_suite_id: ToolSuiteId,
        duplicate_suite_id: ToolSuiteId,
    },
    DuplicateProviderFamilyId {
        provider_family_id: ProviderFamilyId,
        first_suite_id: ToolSuiteId,
        duplicate_suite_id: ToolSuiteId,
    },
    InvalidProviderFamilyReference {
        suite_id: ToolSuiteId,
        surface_key: ToolSurfaceStableKey,
        provider_family_id: ProviderFamilyId,
    },
}

impl fmt::Display for ToolSuiteRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateToolSuiteId { suite_id } => {
                write!(f, "duplicate tool suite id: {suite_id}")
            }
            Self::DuplicateToolSurfaceStableKey {
                surface_key,
                first_suite_id,
                duplicate_suite_id,
            } => write!(
                f,
                "duplicate tool surface stable key `{surface_key}` in suites `{first_suite_id}` and `{duplicate_suite_id}`"
            ),
            Self::DuplicateProviderFamilyId {
                provider_family_id,
                first_suite_id,
                duplicate_suite_id,
            } => write!(
                f,
                "duplicate provider family id `{provider_family_id}` in suites `{first_suite_id}` and `{duplicate_suite_id}`"
            ),
            Self::InvalidProviderFamilyReference {
                suite_id,
                surface_key,
                provider_family_id,
            } => write!(
                f,
                "surface `{surface_key}` in suite `{suite_id}` references undeclared provider family `{provider_family_id}`"
            ),
        }
    }
}

impl std::error::Error for ToolSuiteRegistryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolSurfaceResolution<'a> {
    Resolved(&'a ToolSurfaceDefinition),
    UnknownKey { key: ToolSurfaceStableKey },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFamilyProviderMap {
    assignments: Vec<ProviderFamilyProviderAssignment>,
    assignment_indices_by_family: BTreeMap<ProviderFamilyId, Vec<usize>>,
}

impl ProviderFamilyProviderMap {
    pub fn new(
        registry: &ToolSuiteRegistry,
        assignments: Vec<ProviderFamilyProviderAssignment>,
    ) -> Result<Self, ProviderFamilyProviderMapError> {
        let mut provider_assignments = BTreeMap::<SurfaceProviderId, ProviderFamilyId>::new();
        let mut assignment_indices_by_family = BTreeMap::<ProviderFamilyId, Vec<usize>>::new();

        for (index, assignment) in assignments.iter().enumerate() {
            if !registry.has_provider_family(&assignment.provider_family_id) {
                return Err(ProviderFamilyProviderMapError::UnknownProviderFamily {
                    provider_family_id: assignment.provider_family_id.clone(),
                    provider_id: assignment.provider_id,
                });
            }

            if let Some(first_provider_family_id) = provider_assignments.insert(
                assignment.provider_id,
                assignment.provider_family_id.clone(),
            ) {
                return Err(
                    ProviderFamilyProviderMapError::DuplicateProviderAssignment {
                        provider_id: assignment.provider_id,
                        first_provider_family_id,
                        duplicate_provider_family_id: assignment.provider_family_id.clone(),
                    },
                );
            }

            assignment_indices_by_family
                .entry(assignment.provider_family_id.clone())
                .or_default()
                .push(index);
        }

        Ok(Self {
            assignments,
            assignment_indices_by_family,
        })
    }

    pub fn assignments(&self) -> &[ProviderFamilyProviderAssignment] {
        &self.assignments
    }

    pub fn providers_for<'a>(
        &'a self,
        provider_family_id: &ProviderFamilyId,
    ) -> impl Iterator<Item = SurfaceProviderId> + 'a {
        self.assignment_indices_by_family
            .get(provider_family_id)
            .into_iter()
            .flat_map(|indices| indices.iter())
            .map(|index| self.assignments[*index].provider_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFamilyProviderAssignment {
    pub provider_family_id: ProviderFamilyId,
    pub provider_id: SurfaceProviderId,
}

impl ProviderFamilyProviderAssignment {
    pub fn new(provider_family_id: ProviderFamilyId, provider_id: SurfaceProviderId) -> Self {
        Self {
            provider_family_id,
            provider_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderFamilyProviderMapError {
    UnknownProviderFamily {
        provider_family_id: ProviderFamilyId,
        provider_id: SurfaceProviderId,
    },
    DuplicateProviderAssignment {
        provider_id: SurfaceProviderId,
        first_provider_family_id: ProviderFamilyId,
        duplicate_provider_family_id: ProviderFamilyId,
    },
}

impl fmt::Display for ProviderFamilyProviderMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProviderFamily {
                provider_family_id,
                provider_id,
            } => write!(
                f,
                "surface provider `{provider_id}` references unknown provider family `{provider_family_id}`"
            ),
            Self::DuplicateProviderAssignment {
                provider_id,
                first_provider_family_id,
                duplicate_provider_family_id,
            } => write!(
                f,
                "surface provider `{provider_id}` is assigned to provider families `{first_provider_family_id}` and `{duplicate_provider_family_id}`"
            ),
        }
    }
}

impl std::error::Error for ProviderFamilyProviderMapError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBundle {
    assignments: Vec<ProviderFamilyProviderAssignment>,
}

impl ProviderBundle {
    pub fn new(
        registry: &ToolSuiteRegistry,
        assignments: Vec<ProviderFamilyProviderAssignment>,
    ) -> Result<Self, ProviderBundleError> {
        ProviderFamilyProviderMap::new(registry, assignments.clone())
            .map_err(ProviderBundleError::ProviderMap)?;
        Ok(Self { assignments })
    }

    pub fn assignments(&self) -> &[ProviderFamilyProviderAssignment] {
        &self.assignments
    }

    pub fn provider_map(
        &self,
        registry: &ToolSuiteRegistry,
    ) -> Result<ProviderFamilyProviderMap, ProviderBundleError> {
        ProviderFamilyProviderMap::new(registry, self.assignments.clone())
            .map_err(ProviderBundleError::ProviderMap)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderBundleError {
    ProviderMap(ProviderFamilyProviderMapError),
}

impl fmt::Display for ProviderBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderMap(error) => write!(f, "invalid provider bundle: {error}"),
        }
    }
}

impl std::error::Error for ProviderBundleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchComposition {
    tool_suite_registry: ToolSuiteRegistry,
    profiles: Vec<ToolSuiteProfileDefinition>,
    provider_bundle: ProviderBundle,
    host_policy: HostCapabilityPolicy,
}

impl WorkbenchComposition {
    pub fn tool_suite_registry(&self) -> &ToolSuiteRegistry {
        &self.tool_suite_registry
    }

    pub fn profiles(&self) -> &[ToolSuiteProfileDefinition] {
        &self.profiles
    }

    pub fn provider_bundle(&self) -> &ProviderBundle {
        &self.provider_bundle
    }

    pub fn host_policy(&self) -> &HostCapabilityPolicy {
        &self.host_policy
    }

    pub fn into_parts(
        self,
    ) -> (
        ToolSuiteRegistry,
        Vec<ToolSuiteProfileDefinition>,
        ProviderBundle,
        HostCapabilityPolicy,
    ) {
        (
            self.tool_suite_registry,
            self.profiles,
            self.provider_bundle,
            self.host_policy,
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorkbenchCompositionBuilder {
    suites: Vec<EditorToolSuite>,
    profiles: Vec<ToolSuiteProfileDefinition>,
    provider_assignments: Vec<ProviderFamilyProviderAssignment>,
    host_policy: HostCapabilityPolicy,
}

impl WorkbenchCompositionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_suites(mut self, suites: Vec<EditorToolSuite>) -> Self {
        self.suites = suites;
        self
    }

    pub fn with_profiles(mut self, profiles: Vec<ToolSuiteProfileDefinition>) -> Self {
        self.profiles = profiles;
        self
    }

    pub fn with_provider_assignments(
        mut self,
        assignments: Vec<ProviderFamilyProviderAssignment>,
    ) -> Self {
        self.provider_assignments = assignments;
        self
    }

    pub fn with_host_policy(mut self, policy: HostCapabilityPolicy) -> Self {
        self.host_policy = policy;
        self
    }

    pub fn build(self) -> Result<WorkbenchComposition, WorkbenchCompositionBuildError> {
        let tool_suite_registry = ToolSuiteRegistry::new(self.suites)
            .map_err(WorkbenchCompositionBuildError::ToolSuiteRegistry)?;
        let provider_bundle = ProviderBundle::new(&tool_suite_registry, self.provider_assignments)
            .map_err(WorkbenchCompositionBuildError::ProviderBundle)?;

        Ok(WorkbenchComposition {
            tool_suite_registry,
            profiles: self.profiles,
            provider_bundle,
            host_policy: self.host_policy,
        })
    }
}

#[derive(Debug)]
pub enum WorkbenchCompositionBuildError {
    ToolSuiteRegistry(ToolSuiteRegistryError),
    ProviderBundle(ProviderBundleError),
}

impl fmt::Display for WorkbenchCompositionBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolSuiteRegistry(error) => {
                write!(f, "failed to build Workbench tool-suite registry: {error}")
            }
            Self::ProviderBundle(error) => {
                write!(f, "failed to build Workbench provider bundle: {error}")
            }
        }
    }
}

impl std::error::Error for WorkbenchCompositionBuildError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ProviderFamilyDefinition, SurfaceProviderId, ToolSurfacePersistence, ToolSurfaceRole,
        ToolSurfaceRoute,
    };

    #[test]
    fn duplicate_suite_id_is_rejected() {
        let suite = suite("runenwerk.material_lab", ["graph_canvas"]);

        let error = ToolSuiteRegistry::new(vec![suite.clone(), suite])
            .expect_err("duplicate suite id should be rejected");

        assert!(matches!(
            error,
            ToolSuiteRegistryError::DuplicateToolSuiteId { .. }
        ));
    }

    #[test]
    fn duplicate_stable_surface_key_is_rejected() {
        let first = suite("runenwerk.material_lab", ["graph_canvas"]);
        let second = suite_with_surface_key(
            "runenwerk.material_texture",
            "runenwerk.material_lab.graph_canvas",
            "runenwerk.material_texture",
        );

        let error = ToolSuiteRegistry::new(vec![first, second])
            .expect_err("duplicate surface key should be rejected");

        assert!(matches!(
            error,
            ToolSuiteRegistryError::DuplicateToolSurfaceStableKey { .. }
        ));
    }

    #[test]
    fn duplicate_provider_family_id_is_rejected() {
        let first = suite("runenwerk.material_lab", ["graph_canvas"]);
        let second = suite_with_provider_family(
            "runenwerk.material_texture",
            "runenwerk.material_lab",
            ["texture_viewer"],
        );

        let error = ToolSuiteRegistry::new(vec![first, second])
            .expect_err("duplicate provider family id should be rejected");

        assert!(matches!(
            error,
            ToolSuiteRegistryError::DuplicateProviderFamilyId { .. }
        ));
    }

    #[test]
    fn invalid_provider_family_reference_is_rejected() {
        let suite = suite_with_surface_key(
            "runenwerk.material_lab",
            "runenwerk.material_lab.graph_canvas",
            "runenwerk.missing_family",
        );

        let error =
            ToolSuiteRegistry::new(vec![suite]).expect_err("invalid provider family reference");

        assert!(matches!(
            error,
            ToolSuiteRegistryError::InvalidProviderFamilyReference { .. }
        ));
    }

    #[test]
    fn unknown_stable_surface_key_resolves_fail_closed() {
        let registry =
            ToolSuiteRegistry::new(vec![suite("runenwerk.material_lab", ["graph_canvas"])])
                .expect("valid registry");
        let unknown = ToolSurfaceStableKey::new("runenwerk.material_lab.preview").unwrap();

        let resolution = registry.surfaces().resolve(&unknown);

        assert_eq!(
            resolution,
            ToolSurfaceResolution::UnknownKey { key: unknown }
        );
    }

    #[test]
    fn registry_iteration_preserves_installed_suite_and_surface_order() {
        let first = suite(
            "runenwerk.material_lab",
            ["graph_canvas", "inspector", "preview"],
        );
        let second = suite(
            "runenwerk.texture",
            ["texture_viewer", "volume_texture_viewer"],
        );

        let registry = ToolSuiteRegistry::new(vec![first, second]).expect("valid registry");

        let suite_order = registry
            .suites()
            .iter()
            .map(|suite| suite.suite_id.as_str())
            .collect::<Vec<_>>();
        let surface_order = registry
            .surfaces()
            .iter()
            .map(|surface| surface.key.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            suite_order,
            vec!["runenwerk.material_lab", "runenwerk.texture"]
        );
        assert_eq!(
            surface_order,
            vec![
                "runenwerk.material_lab.graph_canvas",
                "runenwerk.material_lab.inspector",
                "runenwerk.material_lab.preview",
                "runenwerk.texture.texture_viewer",
                "runenwerk.texture.volume_texture_viewer",
            ]
        );
    }

    #[test]
    fn provider_family_provider_map_rejects_unknown_provider_family() {
        let registry =
            ToolSuiteRegistry::new(vec![suite("runenwerk.material_lab", ["graph_canvas"])])
                .expect("valid registry");
        let unknown_family = ProviderFamilyId::new("runenwerk.unknown").unwrap();

        let error = ProviderFamilyProviderMap::new(
            &registry,
            vec![ProviderFamilyProviderAssignment::new(
                unknown_family.clone(),
                provider_id(1),
            )],
        )
        .expect_err("unknown provider family should be rejected");

        assert_eq!(
            error,
            ProviderFamilyProviderMapError::UnknownProviderFamily {
                provider_family_id: unknown_family,
                provider_id: provider_id(1),
            }
        );
    }

    #[test]
    fn provider_family_provider_map_preserves_provider_order() {
        let registry =
            ToolSuiteRegistry::new(vec![suite("runenwerk.material_lab", ["graph_canvas"])])
                .expect("valid registry");
        let family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();

        let map = ProviderFamilyProviderMap::new(
            &registry,
            vec![
                ProviderFamilyProviderAssignment::new(family.clone(), provider_id(2)),
                ProviderFamilyProviderAssignment::new(family.clone(), provider_id(1)),
                ProviderFamilyProviderAssignment::new(family.clone(), provider_id(3)),
            ],
        )
        .expect("valid provider map");

        assert_eq!(
            map.providers_for(&family).collect::<Vec<_>>(),
            vec![provider_id(2), provider_id(1), provider_id(3)]
        );
        assert_eq!(
            map.assignments()
                .iter()
                .map(|assignment| assignment.provider_id)
                .collect::<Vec<_>>(),
            vec![provider_id(2), provider_id(1), provider_id(3)]
        );
    }

    #[test]
    fn provider_family_provider_map_rejects_duplicate_provider_assignment() {
        let registry = ToolSuiteRegistry::new(vec![
            suite("runenwerk.material_lab", ["graph_canvas"]),
            suite("runenwerk.scene", ["viewport"]),
        ])
        .expect("valid registry");
        let first_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        let second_family = ProviderFamilyId::new("runenwerk.scene").unwrap();

        let error = ProviderFamilyProviderMap::new(
            &registry,
            vec![
                ProviderFamilyProviderAssignment::new(first_family.clone(), provider_id(1)),
                ProviderFamilyProviderAssignment::new(second_family.clone(), provider_id(1)),
            ],
        )
        .expect_err("duplicate provider assignment should be rejected");

        assert_eq!(
            error,
            ProviderFamilyProviderMapError::DuplicateProviderAssignment {
                provider_id: provider_id(1),
                first_provider_family_id: first_family,
                duplicate_provider_family_id: second_family,
            }
        );
    }

    #[test]
    fn provider_bundle_rejects_duplicate_providers() {
        let registry = ToolSuiteRegistry::new(vec![
            suite("runenwerk.material_lab", ["graph_canvas"]),
            suite("runenwerk.scene", ["viewport"]),
        ])
        .expect("valid registry");
        let first_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        let second_family = ProviderFamilyId::new("runenwerk.scene").unwrap();

        let error = ProviderBundle::new(
            &registry,
            vec![
                ProviderFamilyProviderAssignment::new(first_family, provider_id(1)),
                ProviderFamilyProviderAssignment::new(second_family, provider_id(1)),
            ],
        )
        .expect_err("duplicate providers should be rejected");

        assert!(matches!(
            error,
            ProviderBundleError::ProviderMap(
                ProviderFamilyProviderMapError::DuplicateProviderAssignment { .. }
            )
        ));
    }

    #[test]
    fn provider_bundle_rejects_unknown_families() {
        let registry =
            ToolSuiteRegistry::new(vec![suite("runenwerk.material_lab", ["graph_canvas"])])
                .expect("valid registry");
        let unknown_family = ProviderFamilyId::new("runenwerk.unknown").unwrap();

        let error = ProviderBundle::new(
            &registry,
            vec![ProviderFamilyProviderAssignment::new(
                unknown_family,
                provider_id(1),
            )],
        )
        .expect_err("unknown families should be rejected");

        assert!(matches!(
            error,
            ProviderBundleError::ProviderMap(
                ProviderFamilyProviderMapError::UnknownProviderFamily { .. }
            )
        ));
    }

    #[test]
    fn composition_builder_installs_suites_profiles_provider_bundle_and_policy() {
        let material_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        let profile = ToolSuiteProfileDefinition::new(
            super::super::ProfileRef::new("runenwerk.material_lab.default").unwrap(),
            "Material Lab",
            vec![
                super::super::SurfaceRef::from_stable_key("runenwerk.material_lab.graph_canvas")
                    .unwrap(),
            ],
        );
        let command =
            super::super::CommandCapabilityKey::new("runenwerk.material_graph.connect_edge")
                .unwrap();

        let composition = WorkbenchCompositionBuilder::new()
            .with_suites(vec![suite("runenwerk.material_lab", ["graph_canvas"])])
            .with_profiles(vec![profile.clone()])
            .with_provider_assignments(vec![ProviderFamilyProviderAssignment::new(
                material_family,
                provider_id(7),
            )])
            .with_host_policy(HostCapabilityPolicy::deny_all().allow_command(command.clone()))
            .build()
            .expect("composition should build");

        assert_eq!(composition.profiles(), &[profile]);
        assert!(composition.host_policy().allows_command(&command));
        assert_eq!(composition.provider_bundle().assignments().len(), 1);
        assert!(
            composition
                .tool_suite_registry()
                .surfaces()
                .get(&ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap())
                .is_some()
        );
    }

    fn suite<const N: usize>(suite_id: &str, surface_names: [&str; N]) -> EditorToolSuite {
        suite_with_provider_family(suite_id, suite_id, surface_names)
    }

    fn suite_with_provider_family<const N: usize>(
        suite_id: &str,
        provider_family_id: &str,
        surface_names: [&str; N],
    ) -> EditorToolSuite {
        let provider_family = ProviderFamilyId::new(provider_family_id).unwrap();
        EditorToolSuite {
            suite_id: ToolSuiteId::new(suite_id).unwrap(),
            label: suite_id.to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family.clone(),
                label: provider_family_id.to_string(),
            }],
            surfaces: surface_names
                .into_iter()
                .map(|surface_name| {
                    surface(
                        format!("{suite_id}.{surface_name}").as_str(),
                        provider_family.as_str(),
                    )
                })
                .collect(),
        }
    }

    fn suite_with_surface_key(
        suite_id: &str,
        surface_key: &str,
        referenced_provider_family_id: &str,
    ) -> EditorToolSuite {
        EditorToolSuite {
            suite_id: ToolSuiteId::new(suite_id).unwrap(),
            label: suite_id.to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: ProviderFamilyId::new(suite_id).unwrap(),
                label: suite_id.to_string(),
            }],
            surfaces: vec![surface(surface_key, referenced_provider_family_id)],
        }
    }

    fn surface(key: &str, provider_family_id: &str) -> ToolSurfaceDefinition {
        ToolSurfaceDefinition {
            key: ToolSurfaceStableKey::new(key).unwrap(),
            label: key.to_string(),
            role: ToolSurfaceRole::Primary,
            panel_kind: crate::PanelKind::GraphCanvas,
            provider_family: ProviderFamilyId::new(provider_family_id).unwrap(),
            route: ToolSurfaceRoute::ProviderOwnedLocal,
            persistence: ToolSurfacePersistence::StableKey,
        }
    }

    fn provider_id(raw: u64) -> SurfaceProviderId {
        SurfaceProviderId::try_from_raw(raw).unwrap()
    }
}
