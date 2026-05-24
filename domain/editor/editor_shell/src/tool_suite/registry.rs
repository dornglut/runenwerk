//! File: domain/editor/editor_shell/src/tool_suite/registry.rs
//! Purpose: Validation and lookup for installed editor tool suites.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use super::{
    EditorToolSuite, HostCapabilityPolicy, ProductCapabilityKey, ProfileRef,
    ProviderFamilyDefinition, ProviderFamilyId, SurfaceRef, ToolServiceKey,
    ToolSuiteCapabilityDeclaration, ToolSuiteId, ToolSuiteProfileDefinition, ToolSurfaceDefinition,
    ToolSurfaceStableKey,
};
use crate::SurfaceProviderId;
use ui_surface::SurfaceCapability;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSuiteRegistry {
    suites: Vec<EditorToolSuite>,
    capability_declarations: Vec<ToolSuiteCapabilityDeclaration>,
    capability_declaration_indices_by_suite: BTreeMap<ToolSuiteId, usize>,
    surfaces: ToolSurfaceRegistry,
}

impl ToolSuiteRegistry {
    pub fn new(suites: Vec<EditorToolSuite>) -> Result<Self, ToolSuiteRegistryError> {
        Self::new_with_capability_declarations(suites, Vec::new())
    }

    pub fn new_with_capability_declarations(
        suites: Vec<EditorToolSuite>,
        capability_declarations: Vec<ToolSuiteCapabilityDeclaration>,
    ) -> Result<Self, ToolSuiteRegistryError> {
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

        let capability_declaration_indices_by_suite =
            validate_capability_declarations(&suite_ids, &capability_declarations)?;

        let mut surface_keys = BTreeMap::<ToolSurfaceStableKey, ToolSuiteId>::new();
        let mut ordered_surfaces = Vec::new();

        for suite in &suites {
            let declared_provider_families = suite
                .provider_families
                .iter()
                .map(|definition| definition.id.clone())
                .collect::<Vec<_>>();

            for surface in &suite.surfaces {
                if surface.label.trim().is_empty() {
                    return Err(ToolSuiteRegistryError::EmptyToolSurfaceLabel {
                        suite_id: suite.suite_id.clone(),
                        surface_key: surface.key.clone(),
                    });
                }

                if !surface.capabilities.allows(SurfaceCapability::Observe) {
                    return Err(
                        ToolSuiteRegistryError::MissingToolSurfaceObserveCapability {
                            suite_id: suite.suite_id.clone(),
                            surface_key: surface.key.clone(),
                        },
                    );
                }

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
            capability_declarations,
            capability_declaration_indices_by_suite,
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

    pub fn capability_declarations(&self) -> &[ToolSuiteCapabilityDeclaration] {
        &self.capability_declarations
    }

    pub fn capability_declaration(
        &self,
        suite_id: &ToolSuiteId,
    ) -> Option<&ToolSuiteCapabilityDeclaration> {
        self.capability_declaration_indices_by_suite
            .get(suite_id)
            .and_then(|index| self.capability_declarations.get(*index))
    }
}

fn validate_capability_declarations(
    suite_ids: &BTreeMap<ToolSuiteId, usize>,
    declarations: &[ToolSuiteCapabilityDeclaration],
) -> Result<BTreeMap<ToolSuiteId, usize>, ToolSuiteRegistryError> {
    let mut indices_by_suite = BTreeMap::<ToolSuiteId, usize>::new();

    for (index, declaration) in declarations.iter().enumerate() {
        let suite_id = declaration.suite_ref.id().clone();
        if !suite_ids.contains_key(&suite_id) {
            return Err(ToolSuiteRegistryError::UnknownCapabilityDeclarationSuite { suite_id });
        }

        if indices_by_suite.insert(suite_id.clone(), index).is_some() {
            return Err(ToolSuiteRegistryError::DuplicateCapabilityDeclarationSuite { suite_id });
        }

        let mut product_needs = BTreeSet::<ProductCapabilityKey>::new();
        for need in &declaration.product_needs {
            if need.label.trim().is_empty() {
                return Err(ToolSuiteRegistryError::EmptyProductCapabilityNeedLabel {
                    suite_id: suite_id.clone(),
                    product_capability_key: need.key.clone(),
                });
            }
            if !product_needs.insert(need.key.clone()) {
                return Err(ToolSuiteRegistryError::DuplicateProductCapabilityNeed {
                    suite_id: suite_id.clone(),
                    product_capability_key: need.key.clone(),
                });
            }
        }

        let mut service_needs = BTreeSet::<ToolServiceKey>::new();
        for need in &declaration.service_needs {
            if need.label.trim().is_empty() {
                return Err(ToolSuiteRegistryError::EmptyToolServiceNeedLabel {
                    suite_id: suite_id.clone(),
                    service_key: need.key.clone(),
                });
            }
            if !service_needs.insert(need.key.clone()) {
                return Err(ToolSuiteRegistryError::DuplicateToolServiceNeed {
                    suite_id: suite_id.clone(),
                    service_key: need.key.clone(),
                });
            }
        }
    }

    Ok(indices_by_suite)
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
    EmptyToolSurfaceLabel {
        suite_id: ToolSuiteId,
        surface_key: ToolSurfaceStableKey,
    },
    MissingToolSurfaceObserveCapability {
        suite_id: ToolSuiteId,
        surface_key: ToolSurfaceStableKey,
    },
    UnknownCapabilityDeclarationSuite {
        suite_id: ToolSuiteId,
    },
    DuplicateCapabilityDeclarationSuite {
        suite_id: ToolSuiteId,
    },
    DuplicateProductCapabilityNeed {
        suite_id: ToolSuiteId,
        product_capability_key: ProductCapabilityKey,
    },
    EmptyProductCapabilityNeedLabel {
        suite_id: ToolSuiteId,
        product_capability_key: ProductCapabilityKey,
    },
    DuplicateToolServiceNeed {
        suite_id: ToolSuiteId,
        service_key: ToolServiceKey,
    },
    EmptyToolServiceNeedLabel {
        suite_id: ToolSuiteId,
        service_key: ToolServiceKey,
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
            Self::EmptyToolSurfaceLabel {
                suite_id,
                surface_key,
            } => write!(
                f,
                "surface `{surface_key}` in suite `{suite_id}` has an empty label"
            ),
            Self::MissingToolSurfaceObserveCapability {
                suite_id,
                surface_key,
            } => write!(
                f,
                "surface `{surface_key}` in suite `{suite_id}` does not declare observe capability"
            ),
            Self::UnknownCapabilityDeclarationSuite { suite_id } => write!(
                f,
                "capability declaration references unknown suite `{suite_id}`"
            ),
            Self::DuplicateCapabilityDeclarationSuite { suite_id } => {
                write!(f, "duplicate capability declaration for suite `{suite_id}`")
            }
            Self::DuplicateProductCapabilityNeed {
                suite_id,
                product_capability_key,
            } => write!(
                f,
                "duplicate product capability need `{product_capability_key}` in suite `{suite_id}`"
            ),
            Self::EmptyProductCapabilityNeedLabel {
                suite_id,
                product_capability_key,
            } => write!(
                f,
                "product capability need `{product_capability_key}` in suite `{suite_id}` has an empty label"
            ),
            Self::DuplicateToolServiceNeed {
                suite_id,
                service_key,
            } => write!(
                f,
                "duplicate tool service need `{service_key}` in suite `{suite_id}`"
            ),
            Self::EmptyToolServiceNeedLabel {
                suite_id,
                service_key,
            } => write!(
                f,
                "tool service need `{service_key}` in suite `{suite_id}` has an empty label"
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
    capability_declarations: Vec<ToolSuiteCapabilityDeclaration>,
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

    pub fn with_capability_declarations(
        mut self,
        declarations: Vec<ToolSuiteCapabilityDeclaration>,
    ) -> Self {
        self.capability_declarations = declarations;
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
        let tool_suite_registry = ToolSuiteRegistry::new_with_capability_declarations(
            self.suites,
            self.capability_declarations,
        )
        .map_err(WorkbenchCompositionBuildError::ToolSuiteRegistry)?;
        validate_profiles(&tool_suite_registry, &self.profiles)?;
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

fn validate_profiles(
    registry: &ToolSuiteRegistry,
    profiles: &[ToolSuiteProfileDefinition],
) -> Result<(), WorkbenchCompositionBuildError> {
    let mut profile_refs = BTreeMap::<ProfileRef, usize>::new();

    for (index, profile) in profiles.iter().enumerate() {
        if profile_refs
            .insert(profile.profile_ref.clone(), index)
            .is_some()
        {
            return Err(WorkbenchCompositionBuildError::DuplicateProfileRef {
                profile_ref: profile.profile_ref.clone(),
            });
        }

        for surface_ref in &profile.default_surfaces {
            if registry.surfaces().get(surface_ref.key()).is_none() {
                return Err(
                    WorkbenchCompositionBuildError::UnknownProfileDefaultSurface {
                        profile_ref: profile.profile_ref.clone(),
                        surface_ref: surface_ref.clone(),
                    },
                );
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum WorkbenchCompositionBuildError {
    ToolSuiteRegistry(ToolSuiteRegistryError),
    ProviderBundle(ProviderBundleError),
    DuplicateProfileRef {
        profile_ref: ProfileRef,
    },
    UnknownProfileDefaultSurface {
        profile_ref: ProfileRef,
        surface_ref: SurfaceRef,
    },
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
            Self::DuplicateProfileRef { profile_ref } => {
                write!(f, "duplicate Workbench profile ref: {profile_ref}")
            }
            Self::UnknownProfileDefaultSurface {
                profile_ref,
                surface_ref,
            } => write!(
                f,
                "Workbench profile `{profile_ref}` references unknown default surface `{surface_ref}`"
            ),
        }
    }
}

impl std::error::Error for WorkbenchCompositionBuildError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ProductCapabilityNeed, ProviderFamilyDefinition, SuiteRef, SurfaceProviderId,
        ToolServiceNeed, ToolSurfaceCreationPolicy, ToolSurfaceRole, ToolSurfaceRoute,
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
    fn empty_surface_label_is_rejected() {
        let mut suite = suite("runenwerk.material_lab", ["graph_canvas"]);
        suite.surfaces[0].label = " ".to_string();

        let error =
            ToolSuiteRegistry::new(vec![suite]).expect_err("empty surface label should reject");

        assert!(matches!(
            error,
            ToolSuiteRegistryError::EmptyToolSurfaceLabel { .. }
        ));
    }

    #[test]
    fn surface_without_observe_capability_is_rejected() {
        let mut suite = suite("runenwerk.material_lab", ["graph_canvas"]);
        suite.surfaces[0].capabilities = ui_surface::SurfaceCapabilitySet::default();

        let error = ToolSuiteRegistry::new(vec![suite])
            .expect_err("surfaces without observe capability should reject");

        assert!(matches!(
            error,
            ToolSuiteRegistryError::MissingToolSurfaceObserveCapability { .. }
        ));
    }

    #[test]
    fn product_and_service_capability_declarations_are_registered_by_suite() {
        let suite_id = ToolSuiteId::new("runenwerk.material_lab").unwrap();
        let product_need = ProductCapabilityNeed::new(
            ProductCapabilityKey::new("runenwerk.material.preview_product").unwrap(),
            "Material preview product",
        );
        let service_need = ToolServiceNeed::new(
            ToolServiceKey::new("runenwerk.material.preview_builder").unwrap(),
            "Material preview builder",
        );
        let declaration = ToolSuiteCapabilityDeclaration::new(
            SuiteRef::new(suite_id.clone()),
            vec![product_need.clone()],
            vec![service_need.clone()],
        );

        let registry = ToolSuiteRegistry::new_with_capability_declarations(
            vec![suite("runenwerk.material_lab", ["graph_canvas"])],
            vec![declaration],
        )
        .expect("valid capability declarations should register");

        let registered = registry
            .capability_declaration(&suite_id)
            .expect("suite declaration should be indexed by suite id");
        assert_eq!(registered.product_needs, vec![product_need]);
        assert_eq!(registered.service_needs, vec![service_need]);
        assert_eq!(registry.capability_declarations().len(), 1);
    }

    #[test]
    fn capability_declaration_rejects_unknown_suite() {
        let declaration = ToolSuiteCapabilityDeclaration::new(
            SuiteRef::from_stable_key("runenwerk.unknown").unwrap(),
            vec![ProductCapabilityNeed::new(
                ProductCapabilityKey::new("runenwerk.material.preview_product").unwrap(),
                "Material preview product",
            )],
            Vec::new(),
        );

        let error = ToolSuiteRegistry::new_with_capability_declarations(
            vec![suite("runenwerk.material_lab", ["graph_canvas"])],
            vec![declaration],
        )
        .expect_err("unknown suite declarations should be rejected");

        assert!(matches!(
            error,
            ToolSuiteRegistryError::UnknownCapabilityDeclarationSuite { .. }
        ));
    }

    #[test]
    fn capability_declaration_rejects_duplicate_product_and_service_needs() {
        let product_key = ProductCapabilityKey::new("runenwerk.material.preview_product").unwrap();
        let service_key = ToolServiceKey::new("runenwerk.material.preview_builder").unwrap();
        let product_error = ToolSuiteRegistry::new_with_capability_declarations(
            vec![suite("runenwerk.material_lab", ["graph_canvas"])],
            vec![ToolSuiteCapabilityDeclaration::new(
                SuiteRef::from_stable_key("runenwerk.material_lab").unwrap(),
                vec![
                    ProductCapabilityNeed::new(product_key.clone(), "Material preview product"),
                    ProductCapabilityNeed::new(product_key.clone(), "Material preview product"),
                ],
                Vec::new(),
            )],
        )
        .expect_err("duplicate product needs should be rejected");
        let service_error = ToolSuiteRegistry::new_with_capability_declarations(
            vec![suite("runenwerk.material_lab", ["graph_canvas"])],
            vec![ToolSuiteCapabilityDeclaration::new(
                SuiteRef::from_stable_key("runenwerk.material_lab").unwrap(),
                Vec::new(),
                vec![
                    ToolServiceNeed::new(service_key.clone(), "Material preview builder"),
                    ToolServiceNeed::new(service_key.clone(), "Material preview builder"),
                ],
            )],
        )
        .expect_err("duplicate service needs should be rejected");

        assert!(matches!(
            product_error,
            ToolSuiteRegistryError::DuplicateProductCapabilityNeed { .. }
        ));
        assert!(matches!(
            service_error,
            ToolSuiteRegistryError::DuplicateToolServiceNeed { .. }
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
        let material_suite_id = ToolSuiteId::new("runenwerk.material_lab").unwrap();
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
        let product_need = ProductCapabilityNeed::new(
            ProductCapabilityKey::new("runenwerk.material.preview_product").unwrap(),
            "Material preview product",
        );

        let composition = WorkbenchCompositionBuilder::new()
            .with_suites(vec![suite("runenwerk.material_lab", ["graph_canvas"])])
            .with_capability_declarations(vec![ToolSuiteCapabilityDeclaration::new(
                SuiteRef::new(material_suite_id.clone()),
                vec![product_need.clone()],
                Vec::new(),
            )])
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
        assert_eq!(
            composition
                .tool_suite_registry()
                .capability_declaration(&material_suite_id)
                .map(|declaration| declaration.product_needs.as_slice()),
            Some([product_need].as_slice())
        );
        assert!(
            composition
                .tool_suite_registry()
                .surfaces()
                .get(&ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap())
                .is_some()
        );
    }

    #[test]
    fn composition_builder_rejects_duplicate_profile_refs() {
        let profile_ref = super::super::ProfileRef::new("runenwerk.material_lab.default").unwrap();
        let profile = ToolSuiteProfileDefinition::new(
            profile_ref.clone(),
            "Material Lab",
            vec![
                super::super::SurfaceRef::from_stable_key("runenwerk.material_lab.graph_canvas")
                    .unwrap(),
            ],
        );

        let error = WorkbenchCompositionBuilder::new()
            .with_suites(vec![suite("runenwerk.material_lab", ["graph_canvas"])])
            .with_profiles(vec![profile.clone(), profile])
            .with_provider_assignments(vec![ProviderFamilyProviderAssignment::new(
                ProviderFamilyId::new("runenwerk.material_lab").unwrap(),
                provider_id(7),
            )])
            .build()
            .expect_err("duplicate profile refs should be rejected");

        assert!(matches!(
            error,
            WorkbenchCompositionBuildError::DuplicateProfileRef {
                profile_ref: duplicate
            } if duplicate == profile_ref
        ));
    }

    #[test]
    fn composition_builder_rejects_unknown_profile_default_surface() {
        let profile_ref = super::super::ProfileRef::new("runenwerk.material_lab.default").unwrap();
        let surface_ref =
            super::super::SurfaceRef::from_stable_key("runenwerk.material_lab.preview").unwrap();
        let profile = ToolSuiteProfileDefinition::new(
            profile_ref.clone(),
            "Material Lab",
            vec![surface_ref.clone()],
        );

        let error = WorkbenchCompositionBuilder::new()
            .with_suites(vec![suite("runenwerk.material_lab", ["graph_canvas"])])
            .with_profiles(vec![profile])
            .with_provider_assignments(vec![ProviderFamilyProviderAssignment::new(
                ProviderFamilyId::new("runenwerk.material_lab").unwrap(),
                provider_id(7),
            )])
            .build()
            .expect_err("unknown profile surface should be rejected");

        assert!(matches!(
            error,
            WorkbenchCompositionBuildError::UnknownProfileDefaultSurface {
                profile_ref: unknown_profile,
                surface_ref: unknown_surface
            } if unknown_profile == profile_ref && unknown_surface == surface_ref
        ));
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
        EditorToolSuite::new(
            super::super::SuiteRef::from_stable_key(suite_id).unwrap(),
            suite_id.to_string(),
            vec![ProviderFamilyDefinition::new(
                provider_family.clone(),
                provider_family_id.to_string(),
            )],
            surface_names
                .into_iter()
                .map(|surface_name| {
                    surface(
                        format!("{suite_id}.{surface_name}").as_str(),
                        provider_family.as_str(),
                    )
                })
                .collect(),
        )
    }

    fn suite_with_surface_key(
        suite_id: &str,
        surface_key: &str,
        referenced_provider_family_id: &str,
    ) -> EditorToolSuite {
        EditorToolSuite::new(
            super::super::SuiteRef::from_stable_key(suite_id).unwrap(),
            suite_id.to_string(),
            vec![ProviderFamilyDefinition::new(
                ProviderFamilyId::new(suite_id).unwrap(),
                suite_id.to_string(),
            )],
            vec![surface(surface_key, referenced_provider_family_id)],
        )
    }

    fn surface(key: &str, provider_family_id: &str) -> ToolSurfaceDefinition {
        ToolSurfaceDefinition::new(
            super::super::SurfaceRef::from_stable_key(key).unwrap(),
            key.to_string(),
            ToolSurfaceRole::Primary,
            crate::PanelKind::GraphCanvas,
            ProviderFamilyId::new(provider_family_id).unwrap(),
            ToolSurfaceRoute::ProviderOwnedLocal,
            ui_surface::SurfaceCapabilitySet::new(true, true, true, false),
            ui_surface::SessionRetentionClass::Restorable,
            ToolSurfaceCreationPolicy::SingletonPerWorkspace,
        )
    }

    fn provider_id(raw: u64) -> SurfaceProviderId {
        SurfaceProviderId::try_from_raw(raw).unwrap()
    }
}
