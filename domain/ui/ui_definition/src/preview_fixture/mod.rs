//! Runtime-neutral preview fixtures, scenarios, target matrices, and evidence descriptors.

use crate::{UiDefinitionDiagnosticSeverity, UiNodeId, UiSourceLocation, identity::AuthoredId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub type UiPreviewFixtureId = AuthoredId;
pub type UiPreviewScenarioId = AuthoredId;
pub type UiPreviewMatrixId = AuthoredId;
pub type UiPreviewEvidenceId = AuthoredId;
pub type UiPreviewDataPackageId = AuthoredId;
pub type UiPreviewCapabilityId = AuthoredId;
pub type UiPreviewTargetProfileId = AuthoredId;
pub type UiPreviewSourcePackageId = AuthoredId;
pub type UiPreviewDiagnosticRef = AuthoredId;
pub type UiPreviewStateRef = AuthoredId;
pub type UiPreviewStepId = AuthoredId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiPreviewDataStateKind {
    Empty,
    Loading,
    Error,
    Denied,
    Offline,
    Heavy,
    Accessibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiPreviewMatrixAxisKind {
    Platform,
    Accessibility,
    Localization,
    Input,
    Size,
    Performance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiPreviewMatrixAxis {
    pub kind: UiPreviewMatrixAxisKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiPreviewValidationMode {
    Preview,
    DryRun,
    AcceptanceEvidence,
    Activate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiPreviewActivationImpact {
    None,
    PreviewOnly,
    BlocksActivation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiPreviewDiagnosticDomain {
    UiDefinition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiPreviewScenarioStepKind {
    Intent,
    Input,
    Wait,
    AssertState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewScenarioStep {
    pub id: UiPreviewStepId,
    pub kind: UiPreviewScenarioStepKind,
    #[serde(default)]
    pub target_node: Option<UiNodeId>,
    #[serde(default)]
    pub expected_state: Option<UiPreviewStateRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewFixtureDeclaration {
    pub id: UiPreviewFixtureId,
    pub data_state: UiPreviewDataStateKind,
    #[serde(default)]
    pub target_profiles: BTreeSet<UiPreviewTargetProfileId>,
    #[serde(default)]
    pub required_data_packages: BTreeSet<UiPreviewDataPackageId>,
    #[serde(default)]
    pub required_capabilities: BTreeSet<UiPreviewCapabilityId>,
    #[serde(default)]
    pub expected_diagnostics: BTreeSet<UiPreviewDiagnosticRef>,
    #[serde(default)]
    pub expected_states: BTreeSet<UiPreviewStateRef>,
    pub source_package: UiPreviewSourcePackageId,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    pub preview_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewScenarioDeclaration {
    pub id: UiPreviewScenarioId,
    pub fixture: UiPreviewFixtureId,
    #[serde(default)]
    pub target_profiles: BTreeSet<UiPreviewTargetProfileId>,
    #[serde(default)]
    pub steps: Vec<UiPreviewScenarioStep>,
    #[serde(default)]
    pub required_capabilities: BTreeSet<UiPreviewCapabilityId>,
    #[serde(default)]
    pub expected_diagnostics: BTreeSet<UiPreviewDiagnosticRef>,
    pub source_package: UiPreviewSourcePackageId,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    pub preview_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewEvidenceDescriptor {
    pub id: UiPreviewEvidenceId,
    #[serde(default)]
    pub target_profiles: BTreeSet<UiPreviewTargetProfileId>,
    #[serde(default)]
    pub expected_diagnostics: BTreeSet<UiPreviewDiagnosticRef>,
    #[serde(default)]
    pub expected_states: BTreeSet<UiPreviewStateRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewMatrixDeclaration {
    pub id: UiPreviewMatrixId,
    #[serde(default)]
    pub fixtures: Vec<UiPreviewFixtureId>,
    #[serde(default)]
    pub scenarios: Vec<UiPreviewScenarioId>,
    #[serde(default)]
    pub target_profiles: BTreeSet<UiPreviewTargetProfileId>,
    #[serde(default)]
    pub axes: Vec<UiPreviewMatrixAxis>,
    #[serde(default)]
    pub evidence: Vec<UiPreviewEvidenceDescriptor>,
    pub validation_mode: UiPreviewValidationMode,
    pub source_package: UiPreviewSourcePackageId,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    pub preview_only: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewLibrary {
    #[serde(default)]
    pub fixtures: Vec<UiPreviewFixtureDeclaration>,
    #[serde(default)]
    pub scenarios: Vec<UiPreviewScenarioDeclaration>,
    #[serde(default)]
    pub matrices: Vec<UiPreviewMatrixDeclaration>,
    #[serde(default)]
    pub known_data_packages: BTreeSet<UiPreviewDataPackageId>,
    #[serde(default)]
    pub known_capabilities: BTreeSet<UiPreviewCapabilityId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewValidationRequest {
    pub target_profile: UiPreviewTargetProfileId,
    pub mode: UiPreviewValidationMode,
    #[serde(default)]
    pub available_data_packages: BTreeSet<UiPreviewDataPackageId>,
    #[serde(default)]
    pub denied_capabilities: BTreeSet<UiPreviewCapabilityId>,
    #[serde(default)]
    pub actual_diagnostics: BTreeSet<UiPreviewDiagnosticRef>,
}

impl UiPreviewValidationRequest {
    pub fn preview(target_profile: impl Into<UiPreviewTargetProfileId>) -> Self {
        Self {
            target_profile: target_profile.into(),
            mode: UiPreviewValidationMode::Preview,
            available_data_packages: BTreeSet::new(),
            denied_capabilities: BTreeSet::new(),
            actual_diagnostics: BTreeSet::new(),
        }
    }

    pub fn activate(target_profile: impl Into<UiPreviewTargetProfileId>) -> Self {
        Self {
            target_profile: target_profile.into(),
            mode: UiPreviewValidationMode::Activate,
            available_data_packages: BTreeSet::new(),
            denied_capabilities: BTreeSet::new(),
            actual_diagnostics: BTreeSet::new(),
        }
    }

    pub fn with_data_package(mut self, package: impl Into<UiPreviewDataPackageId>) -> Self {
        self.available_data_packages.insert(package.into());
        self
    }

    pub fn with_denied_capability(mut self, capability: impl Into<UiPreviewCapabilityId>) -> Self {
        self.denied_capabilities.insert(capability.into());
        self
    }

    pub fn with_actual_diagnostic(mut self, diagnostic: impl Into<UiPreviewDiagnosticRef>) -> Self {
        self.actual_diagnostics.insert(diagnostic.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewDiagnostic {
    pub severity: UiDefinitionDiagnosticSeverity,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub fixture: Option<UiPreviewFixtureId>,
    #[serde(default)]
    pub scenario: Option<UiPreviewScenarioId>,
    #[serde(default)]
    pub matrix: Option<UiPreviewMatrixId>,
    #[serde(default)]
    pub axis: Option<UiPreviewMatrixAxis>,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    #[serde(default)]
    pub target_profile: Option<UiPreviewTargetProfileId>,
    pub owning_domain: UiPreviewDiagnosticDomain,
    #[serde(default)]
    pub source_package: Option<UiPreviewSourcePackageId>,
    #[serde(default)]
    pub expected_diagnostics: Vec<UiPreviewDiagnosticRef>,
    #[serde(default)]
    pub actual_diagnostics: Vec<UiPreviewDiagnosticRef>,
    #[serde(default)]
    pub denied_capabilities: Vec<UiPreviewCapabilityId>,
    pub activation_impact: UiPreviewActivationImpact,
    pub suggested_fix: String,
}

impl UiPreviewDiagnostic {
    fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        suggested_fix: impl Into<String>,
    ) -> Self {
        Self {
            severity: UiDefinitionDiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            fixture: None,
            scenario: None,
            matrix: None,
            axis: None,
            source_location: None,
            target_profile: None,
            owning_domain: UiPreviewDiagnosticDomain::UiDefinition,
            source_package: None,
            expected_diagnostics: Vec::new(),
            actual_diagnostics: Vec::new(),
            denied_capabilities: Vec::new(),
            activation_impact: UiPreviewActivationImpact::BlocksActivation,
            suggested_fix: suggested_fix.into(),
        }
    }

    fn for_fixture(mut self, fixture: &UiPreviewFixtureDeclaration) -> Self {
        self.fixture = Some(fixture.id.clone());
        self.source_location = fixture.source_location.clone();
        self.source_package = Some(fixture.source_package.clone());
        self.expected_diagnostics = fixture.expected_diagnostics.iter().cloned().collect();
        self
    }

    fn for_scenario(mut self, scenario: &UiPreviewScenarioDeclaration) -> Self {
        self.scenario = Some(scenario.id.clone());
        self.fixture = Some(scenario.fixture.clone());
        self.source_location = scenario.source_location.clone();
        self.source_package = Some(scenario.source_package.clone());
        self.expected_diagnostics = scenario.expected_diagnostics.iter().cloned().collect();
        self
    }

    fn for_matrix(mut self, matrix: &UiPreviewMatrixDeclaration) -> Self {
        self.matrix = Some(matrix.id.clone());
        self.source_location = matrix.source_location.clone();
        self.source_package = Some(matrix.source_package.clone());
        self
    }

    fn with_target_profile(mut self, target_profile: UiPreviewTargetProfileId) -> Self {
        self.target_profile = Some(target_profile);
        self
    }

    fn with_axis(mut self, axis: UiPreviewMatrixAxis) -> Self {
        self.axis = Some(axis);
        self
    }

    fn with_denied_capabilities(mut self, capabilities: Vec<UiPreviewCapabilityId>) -> Self {
        self.denied_capabilities = capabilities;
        self
    }

    fn with_diagnostic_mismatch(
        mut self,
        expected: Vec<UiPreviewDiagnosticRef>,
        actual: Vec<UiPreviewDiagnosticRef>,
    ) -> Self {
        self.expected_diagnostics = expected;
        self.actual_diagnostics = actual;
        self
    }

    fn preview_only(mut self) -> Self {
        self.activation_impact = UiPreviewActivationImpact::PreviewOnly;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreviewValidationReport {
    #[serde(default)]
    pub diagnostics: Vec<UiPreviewDiagnostic>,
}

impl UiPreviewValidationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == UiDefinitionDiagnosticSeverity::Error)
    }
}

pub fn validate_preview_fixtures(
    library: &UiPreviewLibrary,
    request: &UiPreviewValidationRequest,
) -> UiPreviewValidationReport {
    let mut diagnostics = Vec::new();
    let fixtures = index_fixtures(library, request, &mut diagnostics);
    let scenarios = index_scenarios(library, request, &mut diagnostics);

    let mut validator = PreviewValidator {
        library,
        request,
        diagnostics,
    };

    validator.validate_data_state_coverage();
    validator.validate_fixtures(&fixtures);
    validator.validate_scenarios(&fixtures);
    validator.validate_matrices(&fixtures, &scenarios);

    UiPreviewValidationReport {
        diagnostics: validator.diagnostics,
    }
}

fn index_fixtures<'a>(
    library: &'a UiPreviewLibrary,
    request: &UiPreviewValidationRequest,
    diagnostics: &mut Vec<UiPreviewDiagnostic>,
) -> BTreeMap<UiPreviewFixtureId, &'a UiPreviewFixtureDeclaration> {
    let mut fixtures = BTreeMap::new();
    for fixture in &library.fixtures {
        if fixtures.insert(fixture.id.clone(), fixture).is_some() {
            diagnostics.push(
                UiPreviewDiagnostic::error(
                    "ui.preview.fixture.duplicate_id",
                    format!(
                        "Preview fixture '{}' is declared more than once.",
                        fixture.id
                    ),
                    "Keep one fixture declaration for each stable fixture id.",
                )
                .for_fixture(fixture)
                .with_target_profile(request.target_profile.clone()),
            );
        }
    }
    fixtures
}

fn index_scenarios<'a>(
    library: &'a UiPreviewLibrary,
    request: &UiPreviewValidationRequest,
    diagnostics: &mut Vec<UiPreviewDiagnostic>,
) -> BTreeMap<UiPreviewScenarioId, &'a UiPreviewScenarioDeclaration> {
    let mut scenarios = BTreeMap::new();
    for scenario in &library.scenarios {
        if scenarios.insert(scenario.id.clone(), scenario).is_some() {
            diagnostics.push(
                UiPreviewDiagnostic::error(
                    "ui.preview.scenario.duplicate_id",
                    format!(
                        "Preview scenario '{}' is declared more than once.",
                        scenario.id
                    ),
                    "Keep one scenario declaration for each stable scenario id.",
                )
                .for_scenario(scenario)
                .with_target_profile(request.target_profile.clone()),
            );
        }
    }
    scenarios
}

struct PreviewValidator<'a> {
    library: &'a UiPreviewLibrary,
    request: &'a UiPreviewValidationRequest,
    diagnostics: Vec<UiPreviewDiagnostic>,
}

impl PreviewValidator<'_> {
    fn validate_data_state_coverage(&mut self) {
        let covered: BTreeSet<_> = self
            .library
            .fixtures
            .iter()
            .map(|fixture| fixture.data_state)
            .collect();
        let required = BTreeSet::from([
            UiPreviewDataStateKind::Empty,
            UiPreviewDataStateKind::Loading,
            UiPreviewDataStateKind::Error,
            UiPreviewDataStateKind::Denied,
            UiPreviewDataStateKind::Offline,
            UiPreviewDataStateKind::Heavy,
            UiPreviewDataStateKind::Accessibility,
        ]);

        if !required.is_subset(&covered) {
            self.diagnostics.push(
                UiPreviewDiagnostic::error(
                    "ui.preview.fixture.data_state_coverage_missing",
                    "Preview fixture library does not cover all required data states.",
                    "Add empty, loading, error, denied, offline, heavy, and accessibility fixtures.",
                )
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_fixtures(
        &mut self,
        fixtures: &BTreeMap<UiPreviewFixtureId, &UiPreviewFixtureDeclaration>,
    ) {
        for fixture in fixtures.values() {
            self.validate_fixture_target_profile(fixture);
            self.validate_preview_only_fixture(fixture);
            self.validate_required_data_packages(fixture);
            self.validate_required_capabilities(fixture);
            self.validate_expected_diagnostics_for_fixture(fixture);
        }
    }

    fn validate_fixture_target_profile(&mut self, fixture: &UiPreviewFixtureDeclaration) {
        if !fixture.target_profiles.is_empty()
            && !fixture
                .target_profiles
                .contains(&self.request.target_profile)
        {
            self.diagnostics.push(
                UiPreviewDiagnostic::error(
                    "ui.preview.fixture.target_profile.unsupported",
                    format!(
                        "Fixture '{}' does not support target profile '{}'.",
                        fixture.id, self.request.target_profile
                    ),
                    "Add target-profile support to the fixture or choose a compatible fixture.",
                )
                .for_fixture(fixture)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_preview_only_fixture(&mut self, fixture: &UiPreviewFixtureDeclaration) {
        if fixture.preview_only && self.request.mode == UiPreviewValidationMode::Activate {
            self.diagnostics.push(
                UiPreviewDiagnostic::error(
                    "ui.preview.fixture.preview_only_activation",
                    format!(
                        "Fixture '{}' is preview-only and cannot activate.",
                        fixture.id
                    ),
                    "Use preview validation or remove the preview-only flag before activation.",
                )
                .for_fixture(fixture)
                .with_target_profile(self.request.target_profile.clone())
                .preview_only(),
            );
        }
    }

    fn validate_required_data_packages(&mut self, fixture: &UiPreviewFixtureDeclaration) {
        let missing: Vec<_> = fixture
            .required_data_packages
            .iter()
            .filter(|package| {
                !self.library.known_data_packages.contains(*package)
                    || !self.request.available_data_packages.contains(*package)
            })
            .cloned()
            .collect();
        if !missing.is_empty() {
            self.diagnostics.push(
                UiPreviewDiagnostic::error(
                    "ui.preview.fixture.data_package.missing",
                    format!("Fixture '{}' requires missing data packages.", fixture.id),
                    "Provide the data packages or remove them from the fixture requirement.",
                )
                .for_fixture(fixture)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_required_capabilities(&mut self, fixture: &UiPreviewFixtureDeclaration) {
        let denied: Vec<_> = fixture
            .required_capabilities
            .iter()
            .filter(|capability| {
                !self.library.known_capabilities.contains(*capability)
                    || self.request.denied_capabilities.contains(*capability)
            })
            .cloned()
            .collect();
        if !denied.is_empty() {
            self.diagnostics.push(
                UiPreviewDiagnostic::error(
                    "ui.preview.capability.denied",
                    format!("Fixture '{}' requires unavailable capabilities.", fixture.id),
                    "Grant the capability for this target profile or remove the fixture requirement.",
                )
                .for_fixture(fixture)
                .with_target_profile(self.request.target_profile.clone())
                .with_denied_capabilities(denied),
            );
        }
    }

    fn validate_expected_diagnostics_for_fixture(&mut self, fixture: &UiPreviewFixtureDeclaration) {
        self.validate_expected_diagnostics(
            &fixture.expected_diagnostics,
            UiPreviewDiagnostic::error(
                "ui.preview.expected_diagnostic.mismatch",
                format!("Fixture '{}' expected diagnostics did not match actual diagnostics.", fixture.id),
                "Update the expected diagnostics or fix the preview input that produced different diagnostics.",
            )
            .for_fixture(fixture)
            .with_target_profile(self.request.target_profile.clone()),
        );
    }

    fn validate_scenarios(
        &mut self,
        fixtures: &BTreeMap<UiPreviewFixtureId, &UiPreviewFixtureDeclaration>,
    ) {
        for scenario in &self.library.scenarios {
            if !fixtures.contains_key(&scenario.fixture) {
                self.diagnostics.push(
                    UiPreviewDiagnostic::error(
                        "ui.preview.scenario.fixture_unknown",
                        format!(
                            "Scenario '{}' references unknown fixture '{}'.",
                            scenario.id, scenario.fixture
                        ),
                        "Add the fixture declaration or update the scenario fixture reference.",
                    )
                    .for_scenario(scenario)
                    .with_target_profile(self.request.target_profile.clone()),
                );
            }

            if !scenario.target_profiles.is_empty()
                && !scenario
                    .target_profiles
                    .contains(&self.request.target_profile)
            {
                self.diagnostics.push(
                    UiPreviewDiagnostic::error(
                        "ui.preview.scenario.target_profile.unsupported",
                        format!(
                            "Scenario '{}' does not support target profile '{}'.",
                            scenario.id, self.request.target_profile
                        ),
                        "Add target-profile support to the scenario or choose a compatible scenario.",
                    )
                    .for_scenario(scenario)
                    .with_target_profile(self.request.target_profile.clone()),
                );
            }

            if scenario.preview_only && self.request.mode == UiPreviewValidationMode::Activate {
                self.diagnostics.push(
                    UiPreviewDiagnostic::error(
                        "ui.preview.scenario.preview_only_activation",
                        format!(
                            "Scenario '{}' is preview-only and cannot activate.",
                            scenario.id
                        ),
                        "Use preview validation or remove the preview-only flag before activation.",
                    )
                    .for_scenario(scenario)
                    .with_target_profile(self.request.target_profile.clone())
                    .preview_only(),
                );
            }

            self.validate_scenario_required_capabilities(scenario);

            if scenario.steps.is_empty()
                || scenario
                    .steps
                    .iter()
                    .any(|step| step.id.as_str().trim().is_empty())
            {
                self.diagnostics.push(
                    UiPreviewDiagnostic::error(
                        "ui.preview.scenario.step.invalid",
                        format!("Scenario '{}' contains invalid replay steps.", scenario.id),
                        "Add at least one replay step and ensure every step has a stable id.",
                    )
                    .for_scenario(scenario)
                    .with_target_profile(self.request.target_profile.clone()),
                );
            }

            self.validate_expected_diagnostics(
                &scenario.expected_diagnostics,
                UiPreviewDiagnostic::error(
                    "ui.preview.expected_diagnostic.mismatch",
                    format!(
                        "Scenario '{}' expected diagnostics did not match actual diagnostics.",
                        scenario.id
                    ),
                    "Update the expected diagnostics or fix the scenario input that produced different diagnostics.",
                )
                .for_scenario(scenario)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_scenario_required_capabilities(&mut self, scenario: &UiPreviewScenarioDeclaration) {
        let denied: Vec<_> = scenario
            .required_capabilities
            .iter()
            .filter(|capability| {
                !self.library.known_capabilities.contains(*capability)
                    || self.request.denied_capabilities.contains(*capability)
            })
            .cloned()
            .collect();
        if !denied.is_empty() {
            self.diagnostics.push(
                UiPreviewDiagnostic::error(
                    "ui.preview.capability.denied",
                    format!("Scenario '{}' requires unavailable capabilities.", scenario.id),
                    "Grant the capability for this target profile or remove the scenario requirement.",
                )
                .for_scenario(scenario)
                .with_target_profile(self.request.target_profile.clone())
                .with_denied_capabilities(denied),
            );
        }
    }

    fn validate_matrices(
        &mut self,
        fixtures: &BTreeMap<UiPreviewFixtureId, &UiPreviewFixtureDeclaration>,
        scenarios: &BTreeMap<UiPreviewScenarioId, &UiPreviewScenarioDeclaration>,
    ) {
        for matrix in &self.library.matrices {
            if !matrix.target_profiles.is_empty()
                && !matrix
                    .target_profiles
                    .contains(&self.request.target_profile)
            {
                self.diagnostics.push(
                    UiPreviewDiagnostic::error(
                        "ui.preview.matrix.target_profile.unsupported",
                        format!(
                            "Matrix '{}' does not support target profile '{}'.",
                            matrix.id, self.request.target_profile
                        ),
                        "Add target-profile support to the matrix or choose a compatible matrix.",
                    )
                    .for_matrix(matrix)
                    .with_target_profile(self.request.target_profile.clone()),
                );
            }

            if matrix.preview_only && self.request.mode == UiPreviewValidationMode::Activate {
                self.diagnostics.push(
                    UiPreviewDiagnostic::error(
                        "ui.preview.matrix.preview_only_activation",
                        format!(
                            "Matrix '{}' is preview-only and cannot activate.",
                            matrix.id
                        ),
                        "Use preview validation or remove the preview-only flag before activation.",
                    )
                    .for_matrix(matrix)
                    .with_target_profile(self.request.target_profile.clone())
                    .preview_only(),
                );
            }

            for fixture_id in &matrix.fixtures {
                if !fixtures.contains_key(fixture_id) {
                    self.diagnostics.push(
                        UiPreviewDiagnostic::error(
                            "ui.preview.matrix.fixture_unknown",
                            format!(
                                "Matrix '{}' references unknown fixture '{}'.",
                                matrix.id, fixture_id
                            ),
                            "Add the fixture declaration or remove it from the matrix.",
                        )
                        .for_matrix(matrix)
                        .with_target_profile(self.request.target_profile.clone()),
                    );
                }
            }

            for scenario_id in &matrix.scenarios {
                if !scenarios.contains_key(scenario_id) {
                    self.diagnostics.push(
                        UiPreviewDiagnostic::error(
                            "ui.preview.matrix.scenario_unknown",
                            format!(
                                "Matrix '{}' references unknown scenario '{}'.",
                                matrix.id, scenario_id
                            ),
                            "Add the scenario declaration or remove it from the matrix.",
                        )
                        .for_matrix(matrix)
                        .with_target_profile(self.request.target_profile.clone()),
                    );
                }
            }

            let mut axis_kinds = BTreeSet::new();
            for axis in &matrix.axes {
                if !axis_kinds.insert(axis.kind) || axis.value.trim().is_empty() {
                    self.diagnostics.push(
                        UiPreviewDiagnostic::error(
                            "ui.preview.matrix.axis.incompatible",
                            format!("Matrix '{}' contains an incompatible axis.", matrix.id),
                            "Keep one non-empty value for each matrix axis kind in this bounded contract.",
                        )
                        .for_matrix(matrix)
                        .with_target_profile(self.request.target_profile.clone())
                        .with_axis(axis.clone()),
                    );
                }
            }

            for evidence in &matrix.evidence {
                if !evidence.target_profiles.is_empty()
                    && !evidence
                        .target_profiles
                        .contains(&self.request.target_profile)
                {
                    self.diagnostics.push(
                        UiPreviewDiagnostic::error(
                            "ui.preview.evidence.target_profile.unsupported",
                            format!(
                                "Evidence descriptor '{}' does not support target profile '{}'.",
                                evidence.id, self.request.target_profile
                            ),
                            "Add target-profile support to the evidence descriptor or choose compatible evidence.",
                        )
                        .for_matrix(matrix)
                        .with_target_profile(self.request.target_profile.clone()),
                    );
                }

                self.validate_expected_diagnostics(
                    &evidence.expected_diagnostics,
                    UiPreviewDiagnostic::error(
                        "ui.preview.expected_diagnostic.mismatch",
                        format!(
                            "Evidence descriptor '{}' expected diagnostics did not match actual diagnostics.",
                            evidence.id
                        ),
                        "Update the evidence descriptor expected diagnostics or refresh the preview evidence.",
                    )
                    .for_matrix(matrix)
                    .with_target_profile(self.request.target_profile.clone()),
                );
            }
        }
    }

    fn validate_expected_diagnostics(
        &mut self,
        expected: &BTreeSet<UiPreviewDiagnosticRef>,
        diagnostic: UiPreviewDiagnostic,
    ) {
        if expected != &self.request.actual_diagnostics {
            self.diagnostics.push(diagnostic.with_diagnostic_mismatch(
                expected.iter().cloned().collect(),
                self.request.actual_diagnostics.iter().cloned().collect(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> AuthoredId {
        AuthoredId::from(value)
    }

    fn ids(values: &[&str]) -> BTreeSet<AuthoredId> {
        values.iter().copied().map(AuthoredId::from).collect()
    }

    fn profiles(values: &[&str]) -> BTreeSet<UiPreviewTargetProfileId> {
        ids(values)
    }

    fn fixture(fixture_id: &str, state: UiPreviewDataStateKind) -> UiPreviewFixtureDeclaration {
        UiPreviewFixtureDeclaration {
            id: id(fixture_id),
            data_state: state,
            target_profiles: profiles(&["editor.workbench", "game.runtime"]),
            required_data_packages: ids(&["preview.data"]),
            required_capabilities: ids(&["preview.run"]),
            expected_diagnostics: ids(&["diag.expected"]),
            expected_states: ids(&["state.ready"]),
            source_package: id("test.package"),
            source_location: None,
            preview_only: false,
        }
    }

    fn all_fixtures() -> Vec<UiPreviewFixtureDeclaration> {
        vec![
            fixture("empty", UiPreviewDataStateKind::Empty),
            fixture("loading", UiPreviewDataStateKind::Loading),
            fixture("error", UiPreviewDataStateKind::Error),
            fixture("denied", UiPreviewDataStateKind::Denied),
            fixture("offline", UiPreviewDataStateKind::Offline),
            fixture("heavy", UiPreviewDataStateKind::Heavy),
            fixture("accessibility", UiPreviewDataStateKind::Accessibility),
        ]
    }

    fn scenario() -> UiPreviewScenarioDeclaration {
        UiPreviewScenarioDeclaration {
            id: id("open-panel"),
            fixture: id("empty"),
            target_profiles: profiles(&["editor.workbench", "game.runtime"]),
            steps: vec![UiPreviewScenarioStep {
                id: id("step-open"),
                kind: UiPreviewScenarioStepKind::Intent,
                target_node: Some(id("button")),
                expected_state: Some(id("state.ready")),
            }],
            required_capabilities: ids(&["preview.run"]),
            expected_diagnostics: ids(&["diag.expected"]),
            source_package: id("test.package"),
            source_location: None,
            preview_only: false,
        }
    }

    fn matrix() -> UiPreviewMatrixDeclaration {
        UiPreviewMatrixDeclaration {
            id: id("default-matrix"),
            fixtures: vec![id("empty")],
            scenarios: vec![id("open-panel")],
            target_profiles: profiles(&["editor.workbench", "game.runtime"]),
            axes: vec![
                UiPreviewMatrixAxis {
                    kind: UiPreviewMatrixAxisKind::Platform,
                    value: "desktop".to_string(),
                },
                UiPreviewMatrixAxis {
                    kind: UiPreviewMatrixAxisKind::Input,
                    value: "keyboard".to_string(),
                },
            ],
            evidence: vec![UiPreviewEvidenceDescriptor {
                id: id("default-evidence"),
                target_profiles: profiles(&["editor.workbench", "game.runtime"]),
                expected_diagnostics: ids(&["diag.expected"]),
                expected_states: ids(&["state.ready"]),
            }],
            validation_mode: UiPreviewValidationMode::Preview,
            source_package: id("test.package"),
            source_location: None,
            preview_only: false,
        }
    }

    fn library() -> UiPreviewLibrary {
        UiPreviewLibrary {
            fixtures: all_fixtures(),
            scenarios: vec![scenario()],
            matrices: vec![matrix()],
            known_data_packages: ids(&["preview.data"]),
            known_capabilities: ids(&["preview.run"]),
        }
    }

    fn request(target: &str) -> UiPreviewValidationRequest {
        UiPreviewValidationRequest::preview(target)
            .with_data_package("preview.data")
            .with_actual_diagnostic("diag.expected")
    }

    fn codes(report: &UiPreviewValidationReport) -> BTreeSet<&str> {
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect()
    }

    #[test]
    fn preview_fixture_validates_editor_and_runtime_examples_without_runtime_ownership() {
        let editor = validate_preview_fixtures(&library(), &request("editor.workbench"));
        let runtime = validate_preview_fixtures(&library(), &request("game.runtime"));

        assert!(!editor.has_errors(), "{:?}", editor.diagnostics);
        assert!(!runtime.has_errors(), "{:?}", runtime.diagnostics);
    }

    #[test]
    fn preview_fixture_rejects_missing_data_state_coverage() {
        let mut library = library();
        library.fixtures.pop();

        let report = validate_preview_fixtures(&library, &request("editor.workbench"));

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.fixture.data_state_coverage_missing"));
    }

    #[test]
    fn preview_fixture_rejects_missing_data_package() {
        let report = validate_preview_fixtures(
            &library(),
            &UiPreviewValidationRequest::preview("editor.workbench")
                .with_actual_diagnostic("diag.expected"),
        );

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.fixture.data_package.missing"));
    }

    #[test]
    fn preview_fixture_rejects_denied_capability() {
        let report = validate_preview_fixtures(
            &library(),
            &request("editor.workbench").with_denied_capability("preview.run"),
        );

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.capability.denied"));
    }

    #[test]
    fn preview_fixture_rejects_denied_scenario_capability() {
        let mut library = library();
        library.scenarios[0].required_capabilities = ids(&["scenario.run"]);
        library.known_capabilities.insert(id("scenario.run"));

        let report = validate_preview_fixtures(
            &library,
            &request("editor.workbench").with_denied_capability("scenario.run"),
        );

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.capability.denied"));
    }

    #[test]
    fn preview_fixture_rejects_unsupported_target_profile() {
        let report = validate_preview_fixtures(&library(), &request("console.runtime"));

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.fixture.target_profile.unsupported"));
        assert!(codes(&report).contains("ui.preview.scenario.target_profile.unsupported"));
        assert!(codes(&report).contains("ui.preview.matrix.target_profile.unsupported"));
    }

    #[test]
    fn preview_fixture_rejects_invalid_scenario_steps() {
        let mut library = library();
        library.scenarios[0].steps.clear();

        let report = validate_preview_fixtures(&library, &request("editor.workbench"));

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.scenario.step.invalid"));
    }

    #[test]
    fn preview_fixture_rejects_matrix_axis_conflicts() {
        let mut library = library();
        library.matrices[0].axes.push(UiPreviewMatrixAxis {
            kind: UiPreviewMatrixAxisKind::Platform,
            value: "mobile".to_string(),
        });

        let report = validate_preview_fixtures(&library, &request("editor.workbench"));

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.matrix.axis.incompatible"));
    }

    #[test]
    fn preview_fixture_rejects_expected_diagnostic_mismatches() {
        let report = validate_preview_fixtures(
            &library(),
            &UiPreviewValidationRequest::preview("editor.workbench")
                .with_data_package("preview.data"),
        );

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.expected_diagnostic.mismatch"));
    }

    #[test]
    fn preview_fixture_rejects_unexpected_actual_diagnostics() {
        let report = validate_preview_fixtures(
            &library(),
            &request("editor.workbench").with_actual_diagnostic("diag.unexpected"),
        );

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.expected_diagnostic.mismatch"));
    }

    #[test]
    fn preview_fixture_rejects_preview_only_activation() {
        let mut library = library();
        library.fixtures[0].preview_only = true;
        library.scenarios[0].preview_only = true;
        library.matrices[0].preview_only = true;

        let report = validate_preview_fixtures(
            &library,
            &UiPreviewValidationRequest::activate("editor.workbench")
                .with_data_package("preview.data")
                .with_actual_diagnostic("diag.expected"),
        );

        assert!(report.has_errors());
        assert!(codes(&report).contains("ui.preview.fixture.preview_only_activation"));
        assert!(codes(&report).contains("ui.preview.scenario.preview_only_activation"));
        assert!(codes(&report).contains("ui.preview.matrix.preview_only_activation"));
    }
}
