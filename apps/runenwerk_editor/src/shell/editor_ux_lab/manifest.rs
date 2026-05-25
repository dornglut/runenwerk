//! Editor UX Lab evidence manifest validation.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::shell::{EditorLabEvidenceArtifact, EditorLabEvidenceArtifactKind};
use editor_shell::{
    EditorUxDesignSystemEvidence, EditorUxGraphCanvasEvidence, EditorUxProductPatternEvidence,
    EditorUxRegisteredSurfaceEvidence, EditorUxScenarioCatalog, EditorUxScenarioKind,
    EditorUxWorkbenchEvidence, ToolSurfaceReadiness,
};
use ui_definition::UiDefinitionDiagnostic;

use super::editor_ux_scenario_catalog;

pub const EDITOR_UX_EVIDENCE_MANIFEST_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorUxEvidenceRunStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUxEvidenceRun {
    pub scenario_id: String,
    pub status: EditorUxEvidenceRunStatus,
    pub visible_widget_count: usize,
    pub scan_issues: Vec<String>,
    #[serde(default)]
    pub design_system_evidence: Vec<EditorUxDesignSystemEvidenceRun>,
    #[serde(default)]
    pub graph_canvas_evidence: Vec<EditorUxGraphCanvasEvidenceRun>,
    #[serde(default)]
    pub product_pattern_evidence: Vec<EditorUxProductPatternEvidenceRun>,
    #[serde(default)]
    pub registered_surface_evidence: Vec<EditorUxRegisteredSurfaceEvidenceRun>,
    #[serde(default)]
    pub workbench_evidence: Vec<EditorUxWorkbenchEvidenceRun>,
    pub artifacts: Vec<EditorLabEvidenceArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUxDesignSystemEvidenceRun {
    pub target_profile: String,
    pub recipe_id: String,
    pub token_ids: Vec<String>,
    pub state_variants: Vec<String>,
}

impl EditorUxDesignSystemEvidenceRun {
    pub fn from_scenario_evidence(evidence: &EditorUxDesignSystemEvidence) -> Self {
        Self {
            target_profile: evidence.target_profile.as_str().to_string(),
            recipe_id: evidence.recipe_id.as_str().to_string(),
            token_ids: evidence
                .token_ids
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            state_variants: evidence
                .state_variants
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUxGraphCanvasEvidenceRun {
    pub target_profile: String,
    pub graph_family: String,
    pub interaction_kinds: Vec<String>,
    pub route_kinds: Vec<String>,
    pub readiness_decisions: Vec<String>,
    pub native_evidence_checks: Vec<String>,
}

impl EditorUxGraphCanvasEvidenceRun {
    pub fn from_scenario_evidence(evidence: &EditorUxGraphCanvasEvidence) -> Self {
        Self {
            target_profile: evidence.target_profile.to_string(),
            graph_family: evidence.graph_family.to_string(),
            interaction_kinds: evidence
                .interaction_kinds
                .iter()
                .map(|interaction| (*interaction).to_string())
                .collect(),
            route_kinds: evidence
                .route_kinds
                .iter()
                .map(|route| (*route).to_string())
                .collect(),
            readiness_decisions: evidence
                .readiness_decisions
                .iter()
                .map(|decision| (*decision).to_string())
                .collect(),
            native_evidence_checks: evidence
                .native_evidence_checks
                .iter()
                .map(|check| (*check).to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUxProductPatternEvidenceRun {
    pub target_profile: String,
    pub pattern_kinds: Vec<String>,
    pub state_kinds: Vec<String>,
    pub interaction_kinds: Vec<String>,
    pub route_kinds: Vec<String>,
    pub native_evidence_checks: Vec<String>,
}

impl EditorUxProductPatternEvidenceRun {
    pub fn from_scenario_evidence(evidence: &EditorUxProductPatternEvidence) -> Self {
        Self {
            target_profile: evidence.target_profile.to_string(),
            pattern_kinds: evidence
                .pattern_kinds
                .iter()
                .map(|pattern| (*pattern).to_string())
                .collect(),
            state_kinds: evidence
                .state_kinds
                .iter()
                .map(|state| (*state).to_string())
                .collect(),
            interaction_kinds: evidence
                .interaction_kinds
                .iter()
                .map(|interaction| (*interaction).to_string())
                .collect(),
            route_kinds: evidence
                .route_kinds
                .iter()
                .map(|route| (*route).to_string())
                .collect(),
            native_evidence_checks: evidence
                .native_evidence_checks
                .iter()
                .map(|check| (*check).to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUxRegisteredSurfaceEvidenceRun {
    pub target_profile: String,
    pub surface_definition_id: u64,
    pub semantic_key: String,
    pub display_name: String,
    pub readiness: String,
    pub visible_in_product: bool,
    pub required_artifact_kinds: Vec<String>,
    pub required_state_kinds: Vec<String>,
    pub required_route_kinds: Vec<String>,
    pub provider_native_checks: Vec<String>,
    pub readiness_reason: String,
}

impl EditorUxRegisteredSurfaceEvidenceRun {
    pub fn from_scenario_evidence(evidence: &EditorUxRegisteredSurfaceEvidence) -> Self {
        Self {
            target_profile: evidence.target_profile.to_string(),
            surface_definition_id: evidence.surface_definition_id.raw(),
            semantic_key: evidence.semantic_key.to_string(),
            display_name: evidence.display_name.to_string(),
            readiness: readiness_name(evidence.readiness).to_string(),
            visible_in_product: evidence.visible_in_product,
            required_artifact_kinds: evidence
                .required_artifact_kinds
                .iter()
                .map(|kind| (*kind).to_string())
                .collect(),
            required_state_kinds: evidence
                .required_state_kinds
                .iter()
                .map(|state| (*state).to_string())
                .collect(),
            required_route_kinds: evidence
                .required_route_kinds
                .iter()
                .map(|route| (*route).to_string())
                .collect(),
            provider_native_checks: evidence
                .provider_native_checks
                .iter()
                .map(|check| (*check).to_string())
                .collect(),
            readiness_reason: evidence.readiness_reason.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUxWorkbenchEvidenceRun {
    pub target_profile: String,
    pub pane_kinds: Vec<String>,
    pub route_kinds: Vec<String>,
    pub readiness_checks: Vec<String>,
    pub legacy_self_authoring_bypass: bool,
}

impl EditorUxWorkbenchEvidenceRun {
    pub fn from_scenario_evidence(evidence: &EditorUxWorkbenchEvidence) -> Self {
        Self {
            target_profile: evidence.target_profile.to_string(),
            pane_kinds: evidence
                .pane_kinds
                .iter()
                .map(|pane| (*pane).to_string())
                .collect(),
            route_kinds: evidence
                .route_kinds
                .iter()
                .map(|route| (*route).to_string())
                .collect(),
            readiness_checks: evidence
                .readiness_checks
                .iter()
                .map(|check| (*check).to_string())
                .collect(),
            legacy_self_authoring_bypass: evidence.legacy_self_authoring_bypass,
        }
    }
}

impl EditorUxEvidenceRun {
    pub fn passed(
        scenario_id: impl Into<String>,
        visible_widget_count: usize,
        artifacts: Vec<EditorLabEvidenceArtifact>,
    ) -> Self {
        Self {
            scenario_id: scenario_id.into(),
            status: EditorUxEvidenceRunStatus::Passed,
            visible_widget_count,
            scan_issues: Vec::new(),
            design_system_evidence: Vec::new(),
            graph_canvas_evidence: Vec::new(),
            product_pattern_evidence: Vec::new(),
            registered_surface_evidence: Vec::new(),
            workbench_evidence: Vec::new(),
            artifacts,
        }
    }

    pub fn failed(
        scenario_id: impl Into<String>,
        visible_widget_count: usize,
        scan_issues: Vec<String>,
        artifacts: Vec<EditorLabEvidenceArtifact>,
    ) -> Self {
        Self {
            scenario_id: scenario_id.into(),
            status: EditorUxEvidenceRunStatus::Failed,
            visible_widget_count,
            scan_issues,
            design_system_evidence: Vec::new(),
            graph_canvas_evidence: Vec::new(),
            product_pattern_evidence: Vec::new(),
            registered_surface_evidence: Vec::new(),
            workbench_evidence: Vec::new(),
            artifacts,
        }
    }

    pub fn with_design_system_evidence(
        mut self,
        evidence: EditorUxDesignSystemEvidenceRun,
    ) -> Self {
        self.design_system_evidence.push(evidence);
        self
    }

    pub fn with_graph_canvas_evidence(mut self, evidence: EditorUxGraphCanvasEvidenceRun) -> Self {
        self.graph_canvas_evidence.push(evidence);
        self
    }

    pub fn with_product_pattern_evidence(
        mut self,
        evidence: EditorUxProductPatternEvidenceRun,
    ) -> Self {
        self.product_pattern_evidence.push(evidence);
        self
    }

    pub fn with_registered_surface_evidence(
        mut self,
        evidence: EditorUxRegisteredSurfaceEvidenceRun,
    ) -> Self {
        self.registered_surface_evidence.push(evidence);
        self
    }

    pub fn with_workbench_evidence(mut self, evidence: EditorUxWorkbenchEvidenceRun) -> Self {
        self.workbench_evidence.push(evidence);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUxEvidenceManifest {
    pub manifest_version: u32,
    pub generated_by: String,
    pub required_scenario_ids: Vec<String>,
    pub runs: Vec<EditorUxEvidenceRun>,
}

impl EditorUxEvidenceManifest {
    pub fn current(
        generated_by: impl Into<String>,
        catalog: &EditorUxScenarioCatalog,
        runs: Vec<EditorUxEvidenceRun>,
    ) -> Self {
        Self {
            manifest_version: EDITOR_UX_EVIDENCE_MANIFEST_VERSION,
            generated_by: generated_by.into(),
            required_scenario_ids: catalog
                .scenarios()
                .map(|scenario| scenario.id.as_str().to_string())
                .collect(),
            runs,
        }
    }

    pub fn validate(
        &self,
        catalog: &EditorUxScenarioCatalog,
    ) -> Result<(), UiDefinitionDiagnostic> {
        if self.manifest_version != EDITOR_UX_EVIDENCE_MANIFEST_VERSION {
            return Err(UiDefinitionDiagnostic::error(
                "editor.ux.evidence.manifest.unsupported_version",
                format!(
                    "unsupported Editor UX evidence manifest version {}",
                    self.manifest_version
                ),
            ));
        }

        let scenarios_by_id = catalog
            .scenarios()
            .map(|scenario| (scenario.id.as_str(), scenario))
            .collect::<BTreeMap<_, _>>();
        let required = self
            .required_scenario_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let expected = scenarios_by_id.keys().copied().collect::<BTreeSet<_>>();
        if required != expected {
            return Err(UiDefinitionDiagnostic::error(
                "editor.ux.evidence.manifest.scenario_catalog_mismatch",
                "manifest required scenario IDs do not match the Editor UX scenario catalog",
            ));
        }

        let mut seen = BTreeSet::new();
        for run in &self.runs {
            let scenario = scenarios_by_id
                .get(run.scenario_id.as_str())
                .ok_or_else(|| {
                    UiDefinitionDiagnostic::error(
                        "editor.ux.evidence.manifest.unknown_scenario",
                        format!(
                            "evidence run '{}' is not part of the scenario catalog",
                            run.scenario_id
                        ),
                    )
                })?;
            if !seen.insert(run.scenario_id.as_str()) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.duplicate_scenario",
                    format!(
                        "evidence scenario '{}' was recorded more than once",
                        run.scenario_id
                    ),
                ));
            }
            if run.status == EditorUxEvidenceRunStatus::Failed || !run.scan_issues.is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.failed_scenario",
                    format!(
                        "evidence scenario '{}' has visible-widget scan failures",
                        run.scenario_id
                    ),
                ));
            }
            if run.visible_widget_count == 0 {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.empty_retained_ui",
                    format!(
                        "evidence scenario '{}' produced no visible widgets",
                        run.scenario_id
                    ),
                ));
            }
            if !run
                .artifacts
                .iter()
                .any(|artifact| artifact.kind == EditorLabEvidenceArtifactKind::RetainedUiDebug)
            {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.missing_retained_ui_artifact",
                    format!(
                        "evidence scenario '{}' lacks retained UI proof",
                        run.scenario_id
                    ),
                ));
            }
            if scenario.readiness == ToolSurfaceReadiness::Product
                && !run.artifacts.iter().any(|artifact| {
                    matches!(
                        artifact.kind,
                        EditorLabEvidenceArtifactKind::NativeScreenshot
                            | EditorLabEvidenceArtifactKind::PlatformImpossibleReport
                    )
                })
            {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.missing_native_or_platform_report",
                    format!(
                        "product scenario '{}' lacks native screenshot or platform-impossible proof",
                        run.scenario_id
                    ),
                ));
            }
            if let Some(expected) = &scenario.design_system_evidence {
                validate_design_system_evidence(run, expected)?;
            } else if !run.design_system_evidence.is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.unexpected_design_system_evidence",
                    format!(
                        "evidence scenario '{}' reports design-system evidence but the scenario has no migrated contract",
                        run.scenario_id
                    ),
                ));
            }
            if let Some(expected) = &scenario.graph_canvas_evidence {
                validate_graph_canvas_evidence(run, expected)?;
            } else if !run.graph_canvas_evidence.is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.unexpected_graph_canvas_evidence",
                    format!(
                        "evidence scenario '{}' reports graph canvas evidence but the scenario has no graph product contract",
                        run.scenario_id
                    ),
                ));
            }
            if let Some(expected) = &scenario.product_pattern_evidence {
                validate_product_pattern_evidence(run, expected)?;
            } else if !run.product_pattern_evidence.is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.unexpected_product_pattern_evidence",
                    format!(
                        "evidence scenario '{}' reports product pattern evidence but the scenario has no product pattern contract",
                        run.scenario_id
                    ),
                ));
            }
            if let Some(expected) = &scenario.registered_surface_evidence {
                validate_registered_surface_evidence(run, expected)?;
            } else if matches!(scenario.kind, EditorUxScenarioKind::RegisteredSurface(_)) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.missing_registered_surface_contract",
                    format!(
                        "registered surface scenario '{}' has no surface evidence contract",
                        run.scenario_id
                    ),
                ));
            } else if !run.registered_surface_evidence.is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.unexpected_registered_surface_evidence",
                    format!(
                        "evidence scenario '{}' reports registered-surface evidence but the scenario has no registered surface contract",
                        run.scenario_id
                    ),
                ));
            }
            if let Some(expected) = &scenario.workbench_evidence {
                validate_workbench_evidence(run, expected)?;
            } else if !run.workbench_evidence.is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.unexpected_workbench_evidence",
                    format!(
                        "evidence scenario '{}' reports workbench evidence but the scenario has no standalone workbench contract",
                        run.scenario_id
                    ),
                ));
            }
        }

        for scenario_id in &self.required_scenario_ids {
            if !seen.contains(scenario_id.as_str()) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.ux.evidence.manifest.missing_scenario",
                    format!(
                        "required evidence scenario '{}' was not recorded",
                        scenario_id
                    ),
                ));
            }
        }

        Ok(())
    }
}

fn validate_graph_canvas_evidence(
    run: &EditorUxEvidenceRun,
    expected: &EditorUxGraphCanvasEvidence,
) -> Result<(), UiDefinitionDiagnostic> {
    let required_artifacts = [
        EditorLabEvidenceArtifactKind::GraphCanvasReport,
        EditorLabEvidenceArtifactKind::FocusTraversalReport,
        EditorLabEvidenceArtifactKind::AccessibilityReport,
        EditorLabEvidenceArtifactKind::DiagnosticsSnapshot,
        EditorLabEvidenceArtifactKind::TimingReport,
    ];
    for artifact_kind in required_artifacts {
        if !run
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == artifact_kind)
        {
            return Err(UiDefinitionDiagnostic::error(
                "editor.ux.evidence.manifest.missing_graph_canvas_artifact",
                format!(
                    "graph canvas scenario '{}' lacks required {:?} artifact",
                    run.scenario_id, artifact_kind
                ),
            ));
        }
    }

    let Some(actual) = run.graph_canvas_evidence.iter().find(|evidence| {
        evidence.target_profile == expected.target_profile
            && evidence.graph_family == expected.graph_family
    }) else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.missing_graph_canvas_evidence",
            format!(
                "graph canvas scenario '{}' lacks typed graph evidence",
                run.scenario_id
            ),
        ));
    };

    let expected_interactions = expected
        .interaction_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_routes = expected
        .route_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_decisions = expected
        .readiness_decisions
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_checks = expected
        .native_evidence_checks
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual_interactions = actual
        .interaction_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_routes = actual
        .route_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_decisions = actual
        .readiness_decisions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_checks = actual
        .native_evidence_checks
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    if !actual_interactions.is_superset(&expected_interactions)
        || !actual_routes.is_superset(&expected_routes)
        || !actual_decisions.is_superset(&expected_decisions)
        || !actual_checks.is_superset(&expected_checks)
    {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.incomplete_graph_canvas_evidence",
            format!(
                "graph canvas scenario '{}' has incomplete interaction/route/readiness/native evidence",
                run.scenario_id
            ),
        ));
    }

    Ok(())
}

fn validate_product_pattern_evidence(
    run: &EditorUxEvidenceRun,
    expected: &EditorUxProductPatternEvidence,
) -> Result<(), UiDefinitionDiagnostic> {
    let required_artifacts = [
        EditorLabEvidenceArtifactKind::ProductPatternReport,
        EditorLabEvidenceArtifactKind::FocusTraversalReport,
        EditorLabEvidenceArtifactKind::AccessibilityReport,
        EditorLabEvidenceArtifactKind::DiagnosticsSnapshot,
        EditorLabEvidenceArtifactKind::TimingReport,
    ];
    for artifact_kind in required_artifacts {
        if !run
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == artifact_kind)
        {
            return Err(UiDefinitionDiagnostic::error(
                "editor.ux.evidence.manifest.missing_product_pattern_artifact",
                format!(
                    "product pattern scenario '{}' lacks required {:?} artifact",
                    run.scenario_id, artifact_kind
                ),
            ));
        }
    }

    let Some(actual) = run
        .product_pattern_evidence
        .iter()
        .find(|evidence| evidence.target_profile == expected.target_profile)
    else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.missing_product_pattern_evidence",
            format!(
                "product pattern scenario '{}' lacks typed product pattern evidence",
                run.scenario_id
            ),
        ));
    };

    let expected_patterns = expected
        .pattern_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_states = expected
        .state_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_interactions = expected
        .interaction_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_routes = expected
        .route_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_checks = expected
        .native_evidence_checks
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual_patterns = actual
        .pattern_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_states = actual
        .state_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_interactions = actual
        .interaction_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_routes = actual
        .route_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_checks = actual
        .native_evidence_checks
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    if !actual_patterns.is_superset(&expected_patterns)
        || !actual_states.is_superset(&expected_states)
        || !actual_interactions.is_superset(&expected_interactions)
        || !actual_routes.is_superset(&expected_routes)
        || !actual_checks.is_superset(&expected_checks)
    {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.incomplete_product_pattern_evidence",
            format!(
                "product pattern scenario '{}' has incomplete pattern/state/interaction/route/native evidence",
                run.scenario_id
            ),
        ));
    }

    Ok(())
}

fn validate_registered_surface_evidence(
    run: &EditorUxEvidenceRun,
    expected: &EditorUxRegisteredSurfaceEvidence,
) -> Result<(), UiDefinitionDiagnostic> {
    let actual_artifacts = run
        .artifacts
        .iter()
        .map(|artifact| format!("{:?}", artifact.kind))
        .collect::<BTreeSet<_>>();
    for artifact_kind in &expected.required_artifact_kinds {
        if !actual_artifacts.contains(*artifact_kind) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.ux.evidence.manifest.missing_registered_surface_artifact",
                format!(
                    "registered surface scenario '{}' lacks required {} artifact",
                    run.scenario_id, artifact_kind
                ),
            ));
        }
    }

    let Some(actual) = run.registered_surface_evidence.iter().find(|evidence| {
        evidence.target_profile == expected.target_profile
            && evidence.surface_definition_id == expected.surface_definition_id.raw()
            && evidence.semantic_key == expected.semantic_key
    }) else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.missing_registered_surface_evidence",
            format!(
                "registered surface scenario '{}' lacks typed registered-surface evidence",
                run.scenario_id
            ),
        ));
    };

    if actual.display_name != expected.display_name
        || actual.readiness != readiness_name(expected.readiness)
        || actual.visible_in_product != expected.visible_in_product
        || actual.readiness_reason != expected.readiness_reason
    {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.registered_surface_identity_mismatch",
            format!(
                "registered surface scenario '{}' has mismatched identity/readiness evidence",
                run.scenario_id
            ),
        ));
    }

    let expected_artifacts = expected
        .required_artifact_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_states = expected
        .required_state_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_routes = expected
        .required_route_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_checks = expected
        .provider_native_checks
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual_required_artifacts = actual
        .required_artifact_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_states = actual
        .required_state_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_routes = actual
        .required_route_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_checks = actual
        .provider_native_checks
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    if !actual_required_artifacts.is_superset(&expected_artifacts)
        || !actual_states.is_superset(&expected_states)
        || !actual_routes.is_superset(&expected_routes)
        || !actual_checks.is_superset(&expected_checks)
    {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.incomplete_registered_surface_evidence",
            format!(
                "registered surface scenario '{}' has incomplete artifact/state/route/provider evidence",
                run.scenario_id
            ),
        ));
    }

    Ok(())
}

fn validate_workbench_evidence(
    run: &EditorUxEvidenceRun,
    expected: &EditorUxWorkbenchEvidence,
) -> Result<(), UiDefinitionDiagnostic> {
    let required_artifacts = [
        EditorLabEvidenceArtifactKind::WorkbenchReport,
        EditorLabEvidenceArtifactKind::FocusTraversalReport,
        EditorLabEvidenceArtifactKind::AccessibilityReport,
        EditorLabEvidenceArtifactKind::DiagnosticsSnapshot,
        EditorLabEvidenceArtifactKind::TimingReport,
    ];
    for artifact_kind in required_artifacts {
        if !run
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == artifact_kind)
        {
            return Err(UiDefinitionDiagnostic::error(
                "editor.ux.evidence.manifest.missing_workbench_artifact",
                format!(
                    "standalone workbench scenario '{}' lacks required {:?} artifact",
                    run.scenario_id, artifact_kind
                ),
            ));
        }
    }

    let Some(actual) = run
        .workbench_evidence
        .iter()
        .find(|evidence| evidence.target_profile == expected.target_profile)
    else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.missing_workbench_evidence",
            format!(
                "standalone workbench scenario '{}' lacks typed workbench evidence",
                run.scenario_id
            ),
        ));
    };

    let expected_panes = expected.pane_kinds.iter().copied().collect::<BTreeSet<_>>();
    let expected_routes = expected
        .route_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let expected_checks = expected
        .readiness_checks
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual_panes = actual
        .pane_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_routes = actual
        .route_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_checks = actual
        .readiness_checks
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    if !actual_panes.is_superset(&expected_panes)
        || !actual_routes.is_superset(&expected_routes)
        || !actual_checks.is_superset(&expected_checks)
        || actual.legacy_self_authoring_bypass != expected.legacy_self_authoring_bypass
    {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.incomplete_workbench_evidence",
            format!(
                "standalone workbench scenario '{}' has incomplete pane/route/readiness evidence",
                run.scenario_id
            ),
        ));
    }

    Ok(())
}

fn readiness_name(readiness: ToolSurfaceReadiness) -> &'static str {
    match readiness {
        ToolSurfaceReadiness::Product => "product",
        ToolSurfaceReadiness::FallbackOnly => "fallback_only",
        ToolSurfaceReadiness::Diagnostic => "diagnostic",
        ToolSurfaceReadiness::HiddenUntilProductized => "hidden_until_productized",
    }
}

fn validate_design_system_evidence(
    run: &EditorUxEvidenceRun,
    expected: &EditorUxDesignSystemEvidence,
) -> Result<(), UiDefinitionDiagnostic> {
    if !run
        .artifacts
        .iter()
        .any(|artifact| artifact.kind == EditorLabEvidenceArtifactKind::DesignSystemReport)
    {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.missing_design_system_artifact",
            format!(
                "migrated scenario '{}' lacks design-system evidence artifact",
                run.scenario_id
            ),
        ));
    }

    let expected_tokens = expected
        .token_ids
        .iter()
        .map(|id| id.as_str())
        .collect::<BTreeSet<_>>();
    let expected_states = expected
        .state_variants
        .iter()
        .map(|id| id.as_str())
        .collect::<BTreeSet<_>>();
    let expected_recipe = expected.recipe_id.as_str();
    let expected_target = expected.target_profile.as_str();
    let Some(actual) = run.design_system_evidence.iter().find(|evidence| {
        evidence.recipe_id == expected_recipe && evidence.target_profile == expected_target
    }) else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.missing_design_system_evidence",
            format!(
                "migrated scenario '{}' lacks token/recipe/state evidence",
                run.scenario_id
            ),
        ));
    };

    let actual_tokens = actual
        .token_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual_states = actual
        .state_variants
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    if !actual_tokens.is_superset(&expected_tokens) || !actual_states.is_superset(&expected_states)
    {
        return Err(UiDefinitionDiagnostic::error(
            "editor.ux.evidence.manifest.incomplete_design_system_evidence",
            format!(
                "migrated scenario '{}' has incomplete token/recipe/state evidence",
                run.scenario_id
            ),
        ));
    }

    Ok(())
}

pub fn validate_editor_ux_manifest(
    manifest: &EditorUxEvidenceManifest,
) -> Result<(), UiDefinitionDiagnostic> {
    let catalog = editor_ux_scenario_catalog();
    manifest.validate(&catalog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_rejects_product_scenario_without_native_or_platform_report() {
        let catalog = editor_ux_scenario_catalog();
        let scenario = catalog
            .scenarios()
            .find(|scenario| scenario.readiness == ToolSurfaceReadiness::Product)
            .expect("product scenario should exist");
        let run = EditorUxEvidenceRun::passed(
            scenario.id.as_str(),
            1,
            vec![EditorLabEvidenceArtifact::new(
                EditorLabEvidenceArtifactKind::RetainedUiDebug,
                "artifacts/scenario.txt",
                64,
                "retained UI",
            )],
        );
        let manifest = EditorUxEvidenceManifest {
            manifest_version: EDITOR_UX_EVIDENCE_MANIFEST_VERSION,
            generated_by: "test".to_string(),
            required_scenario_ids: vec![scenario.id.as_str().to_string()],
            runs: vec![run],
        };
        let single_scenario_catalog = EditorUxScenarioCatalog::new(vec![scenario.clone()])
            .expect("single scenario catalog should validate construction");

        let error = manifest
            .validate(&single_scenario_catalog)
            .expect_err("product scenario should require native or platform-impossible proof");
        assert_eq!(
            error.code,
            "editor.ux.evidence.manifest.missing_native_or_platform_report"
        );
    }

    #[test]
    fn manifest_rejects_migrated_scenario_without_design_system_report() {
        let catalog = editor_ux_scenario_catalog();
        let scenario = catalog
            .scenarios()
            .find(|scenario| scenario.design_system_evidence.is_some())
            .expect("migrated scenario should exist");
        let run = EditorUxEvidenceRun::passed(
            scenario.id.as_str(),
            1,
            vec![
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::RetainedUiDebug,
                    "artifacts/scenario.txt",
                    64,
                    "retained UI",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::PlatformImpossibleReport,
                    "artifacts/native-platform.ron",
                    64,
                    "native platform report",
                ),
            ],
        );
        let manifest = EditorUxEvidenceManifest {
            manifest_version: EDITOR_UX_EVIDENCE_MANIFEST_VERSION,
            generated_by: "test".to_string(),
            required_scenario_ids: vec![scenario.id.as_str().to_string()],
            runs: vec![run],
        };
        let single_scenario_catalog = EditorUxScenarioCatalog::new(vec![scenario.clone()])
            .expect("single scenario catalog should validate construction");

        let error = manifest
            .validate(&single_scenario_catalog)
            .expect_err("migrated scenario should require design-system evidence");
        assert_eq!(
            error.code,
            "editor.ux.evidence.manifest.missing_design_system_artifact"
        );
    }

    #[test]
    fn manifest_rejects_workbench_scenario_without_workbench_report() {
        let catalog = editor_ux_scenario_catalog();
        let scenario = catalog
            .scenarios()
            .find(|scenario| scenario.workbench_evidence.is_some())
            .expect("workbench scenario should exist");
        let run = EditorUxEvidenceRun::passed(
            scenario.id.as_str(),
            1,
            vec![
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::RetainedUiDebug,
                    "artifacts/scenario.txt",
                    64,
                    "retained UI",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::PlatformImpossibleReport,
                    "artifacts/native-platform.ron",
                    64,
                    "native platform report",
                ),
            ],
        );
        let manifest = EditorUxEvidenceManifest {
            manifest_version: EDITOR_UX_EVIDENCE_MANIFEST_VERSION,
            generated_by: "test".to_string(),
            required_scenario_ids: vec![scenario.id.as_str().to_string()],
            runs: vec![run],
        };
        let single_scenario_catalog = EditorUxScenarioCatalog::new(vec![scenario.clone()])
            .expect("single scenario catalog should validate construction");

        let error = manifest
            .validate(&single_scenario_catalog)
            .expect_err("workbench scenario should require workbench evidence artifacts");
        assert_eq!(
            error.code,
            "editor.ux.evidence.manifest.missing_workbench_artifact"
        );
    }

    #[test]
    fn manifest_rejects_graph_scenario_without_graph_canvas_report() {
        let catalog = editor_ux_scenario_catalog();
        let scenario = catalog
            .scenarios()
            .find(|scenario| scenario.graph_canvas_evidence.is_some())
            .expect("graph canvas scenario should exist");
        let run = EditorUxEvidenceRun::passed(
            scenario.id.as_str(),
            1,
            vec![
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::RetainedUiDebug,
                    "artifacts/scenario.txt",
                    64,
                    "retained UI",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::PlatformImpossibleReport,
                    "artifacts/native-platform.ron",
                    64,
                    "native platform report",
                ),
            ],
        );
        let manifest = EditorUxEvidenceManifest {
            manifest_version: EDITOR_UX_EVIDENCE_MANIFEST_VERSION,
            generated_by: "test".to_string(),
            required_scenario_ids: vec![scenario.id.as_str().to_string()],
            runs: vec![run],
        };
        let single_scenario_catalog = EditorUxScenarioCatalog::new(vec![scenario.clone()])
            .expect("single scenario catalog should validate construction");

        let error = manifest
            .validate(&single_scenario_catalog)
            .expect_err("graph canvas scenario should require graph evidence artifacts");
        assert_eq!(
            error.code,
            "editor.ux.evidence.manifest.missing_graph_canvas_artifact"
        );
    }

    #[test]
    fn manifest_rejects_product_pattern_scenario_without_pattern_report() {
        let catalog = editor_ux_scenario_catalog();
        let scenario = catalog
            .scenarios()
            .find(|scenario| scenario.product_pattern_evidence.is_some())
            .expect("product pattern scenario should exist");
        let run = EditorUxEvidenceRun::passed(
            scenario.id.as_str(),
            1,
            vec![
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::RetainedUiDebug,
                    "artifacts/scenario.txt",
                    64,
                    "retained UI",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::PlatformImpossibleReport,
                    "artifacts/native-platform.ron",
                    64,
                    "native platform report",
                ),
            ],
        );
        let manifest = EditorUxEvidenceManifest {
            manifest_version: EDITOR_UX_EVIDENCE_MANIFEST_VERSION,
            generated_by: "test".to_string(),
            required_scenario_ids: vec![scenario.id.as_str().to_string()],
            runs: vec![run],
        };
        let single_scenario_catalog = EditorUxScenarioCatalog::new(vec![scenario.clone()])
            .expect("single scenario catalog should validate construction");

        let error = manifest
            .validate(&single_scenario_catalog)
            .expect_err("product pattern scenario should require pattern evidence artifacts");
        assert_eq!(
            error.code,
            "editor.ux.evidence.manifest.missing_product_pattern_artifact"
        );
    }

    #[test]
    fn manifest_rejects_registered_surface_scenario_without_surface_evidence() {
        let catalog = editor_ux_scenario_catalog();
        let scenario = catalog
            .scenarios()
            .find(|scenario| {
                matches!(scenario.kind, EditorUxScenarioKind::RegisteredSurface(_))
                    && scenario.readiness == ToolSurfaceReadiness::HiddenUntilProductized
            })
            .expect("hidden registered surface scenario should exist");
        let run = EditorUxEvidenceRun::passed(
            scenario.id.as_str(),
            1,
            vec![
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::RetainedUiDebug,
                    "artifacts/scenario.txt",
                    64,
                    "retained UI",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::SurfaceReadinessReport,
                    "artifacts/surface-readiness.ron",
                    64,
                    "surface readiness",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::UnsupportedCheckReport,
                    "artifacts/hidden-surface.ron",
                    64,
                    "hidden surface proof",
                ),
            ],
        );
        let manifest = EditorUxEvidenceManifest {
            manifest_version: EDITOR_UX_EVIDENCE_MANIFEST_VERSION,
            generated_by: "test".to_string(),
            required_scenario_ids: vec![scenario.id.as_str().to_string()],
            runs: vec![run],
        };
        let single_scenario_catalog = EditorUxScenarioCatalog::new(vec![scenario.clone()])
            .expect("single scenario catalog should validate construction");

        let error = manifest
            .validate(&single_scenario_catalog)
            .expect_err("registered surface scenario should require typed surface evidence");
        assert_eq!(
            error.code,
            "editor.ux.evidence.manifest.missing_registered_surface_evidence"
        );
    }

    #[test]
    fn manifest_rejects_registered_surface_scenario_without_surface_report() {
        let catalog = editor_ux_scenario_catalog();
        let scenario = catalog
            .scenarios()
            .find(|scenario| {
                matches!(scenario.kind, EditorUxScenarioKind::RegisteredSurface(_))
                    && scenario.readiness == ToolSurfaceReadiness::Product
                    && scenario.graph_canvas_evidence.is_none()
            })
            .expect("product registered surface scenario should exist");
        let surface_evidence = scenario
            .registered_surface_evidence
            .as_ref()
            .expect("registered surface scenario should have evidence");
        let run = EditorUxEvidenceRun::passed(
            scenario.id.as_str(),
            1,
            vec![
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::RetainedUiDebug,
                    "artifacts/scenario.txt",
                    64,
                    "retained UI",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::PlatformImpossibleReport,
                    "artifacts/native-platform.ron",
                    64,
                    "native platform report",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::FocusTraversalReport,
                    "artifacts/focus.ron",
                    64,
                    "focus traversal",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::AccessibilityReport,
                    "artifacts/accessibility.ron",
                    64,
                    "accessibility",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::DiagnosticsSnapshot,
                    "artifacts/diagnostics.ron",
                    64,
                    "diagnostics",
                ),
                EditorLabEvidenceArtifact::new(
                    EditorLabEvidenceArtifactKind::TimingReport,
                    "artifacts/timing.ron",
                    64,
                    "timing",
                ),
            ],
        )
        .with_registered_surface_evidence(
            EditorUxRegisteredSurfaceEvidenceRun::from_scenario_evidence(surface_evidence),
        );
        let manifest = EditorUxEvidenceManifest {
            manifest_version: EDITOR_UX_EVIDENCE_MANIFEST_VERSION,
            generated_by: "test".to_string(),
            required_scenario_ids: vec![scenario.id.as_str().to_string()],
            runs: vec![run],
        };
        let single_scenario_catalog = EditorUxScenarioCatalog::new(vec![scenario.clone()])
            .expect("single scenario catalog should validate construction");

        let error = manifest
            .validate(&single_scenario_catalog)
            .expect_err("registered surface scenario should require surface readiness artifact");
        assert_eq!(
            error.code,
            "editor.ux.evidence.manifest.missing_registered_surface_artifact"
        );
    }
}
