//! App-owned Editor Lab preview scenario and runtime evidence contracts.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use ui_definition::{UiDefinitionDiagnostic, UiDefinitionDiagnosticSeverity};

pub const EDITOR_LAB_EVIDENCE_MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabPreviewScenario {
    pub id: String,
    pub label: String,
    pub state_family: EditorLabScenarioStateFamily,
    pub target_profile: String,
    pub expected_runtime_path: String,
    pub capture_requirement: EditorLabCaptureRequirement,
    pub accessibility_required: bool,
    pub performance_required: bool,
}

impl EditorLabPreviewScenario {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        state_family: EditorLabScenarioStateFamily,
        expected_runtime_path: impl Into<String>,
        capture_requirement: EditorLabCaptureRequirement,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state_family,
            target_profile: "runenwerk.editor.workspace.editor_design".to_string(),
            expected_runtime_path: expected_runtime_path.into(),
            capture_requirement,
            accessibility_required: matches!(
                state_family,
                EditorLabScenarioStateFamily::Accessibility
            ),
            performance_required: matches!(state_family, EditorLabScenarioStateFamily::Performance),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabScenarioStateFamily {
    Success,
    Warning,
    Error,
    Reload,
    Apply,
    Rollback,
    DegradedProvider,
    Accessibility,
    Performance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLabCaptureRequirement {
    NativeScreenshotPreferred,
    RetainedVisualRequired,
    DiagnosticsOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLabEvidenceRunStatus {
    Passed,
    PassedWithUnsupportedChecks,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabEvidenceArtifact {
    pub kind: EditorLabEvidenceArtifactKind,
    pub path: String,
    pub bytes: usize,
    pub description: String,
}

impl EditorLabEvidenceArtifact {
    pub fn new(
        kind: EditorLabEvidenceArtifactKind,
        path: impl Into<String>,
        bytes: usize,
        description: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            path: path.into(),
            bytes,
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabEvidenceArtifactKind {
    RetainedUiDebug,
    NativeScreenshot,
    GpuVisualDiff,
    FocusTraversalReport,
    ContrastSampleReport,
    TimingReport,
    ProviderSnapshot,
    DiagnosticsSnapshot,
    ActivationReport,
    ProjectPackage,
    RollbackReport,
    AccessibilityReport,
    PerformanceReport,
    PlatformImpossibleReport,
    UnsupportedCheckReport,
    EvidenceManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabEvidenceCapability {
    RetainedVisualTruth,
    NativeScreenshotCapture,
    GpuVisualDiff,
    NativeFocusTraversal,
    PixelContrastSampling,
    RuntimeTimingCapture,
    NativeScreenshotTiming,
    GpuVisualDiffTiming,
    DiagnosticsSnapshot,
    DegradedProviderSurface,
    ReloadWithoutActivation,
    ApplyActivation,
    RollbackRecovery,
    FailurePreservation,
}

pub const PM_UI_LAB_PERF_002_EVIDENCE_CAPABILITIES: [EditorLabEvidenceCapability; 14] = [
    EditorLabEvidenceCapability::RetainedVisualTruth,
    EditorLabEvidenceCapability::NativeScreenshotCapture,
    EditorLabEvidenceCapability::GpuVisualDiff,
    EditorLabEvidenceCapability::NativeFocusTraversal,
    EditorLabEvidenceCapability::PixelContrastSampling,
    EditorLabEvidenceCapability::RuntimeTimingCapture,
    EditorLabEvidenceCapability::NativeScreenshotTiming,
    EditorLabEvidenceCapability::GpuVisualDiffTiming,
    EditorLabEvidenceCapability::DiagnosticsSnapshot,
    EditorLabEvidenceCapability::DegradedProviderSurface,
    EditorLabEvidenceCapability::ReloadWithoutActivation,
    EditorLabEvidenceCapability::ApplyActivation,
    EditorLabEvidenceCapability::RollbackRecovery,
    EditorLabEvidenceCapability::FailurePreservation,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLabEvidenceCapabilitySupportStatus {
    Supported,
    PlatformImpossible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabEvidenceCapabilityProbe {
    pub capability: EditorLabEvidenceCapability,
    pub backend: String,
    pub environment: String,
    pub support_status: EditorLabEvidenceCapabilitySupportStatus,
    pub reason: String,
}

impl EditorLabEvidenceCapabilityProbe {
    pub fn supported(
        capability: EditorLabEvidenceCapability,
        backend: impl Into<String>,
        environment: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            capability,
            backend: backend.into(),
            environment: environment.into(),
            support_status: EditorLabEvidenceCapabilitySupportStatus::Supported,
            reason: reason.into(),
        }
    }

    pub fn platform_impossible(
        capability: EditorLabEvidenceCapability,
        backend: impl Into<String>,
        environment: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            capability,
            backend: backend.into(),
            environment: environment.into(),
            support_status: EditorLabEvidenceCapabilitySupportStatus::PlatformImpossible,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLabEvidenceCapabilityResultStatus {
    Captured,
    PlatformImpossible,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabEvidenceCapabilityResult {
    pub capability: EditorLabEvidenceCapability,
    pub status: EditorLabEvidenceCapabilityResultStatus,
    pub probe: EditorLabEvidenceCapabilityProbe,
    pub artifacts: Vec<EditorLabEvidenceArtifact>,
    pub reproduction_command: String,
    pub diagnostic: String,
}

impl EditorLabEvidenceCapabilityResult {
    pub fn captured(
        capability: EditorLabEvidenceCapability,
        probe: EditorLabEvidenceCapabilityProbe,
        artifacts: Vec<EditorLabEvidenceArtifact>,
        reproduction_command: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            capability,
            status: EditorLabEvidenceCapabilityResultStatus::Captured,
            probe,
            artifacts,
            reproduction_command: reproduction_command.into(),
            diagnostic: diagnostic.into(),
        }
    }

    pub fn platform_impossible(
        capability: EditorLabEvidenceCapability,
        probe: EditorLabEvidenceCapabilityProbe,
        artifacts: Vec<EditorLabEvidenceArtifact>,
        reproduction_command: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            capability,
            status: EditorLabEvidenceCapabilityResultStatus::PlatformImpossible,
            probe,
            artifacts,
            reproduction_command: reproduction_command.into(),
            diagnostic: diagnostic.into(),
        }
    }

    pub fn failed(
        capability: EditorLabEvidenceCapability,
        probe: EditorLabEvidenceCapabilityProbe,
        artifacts: Vec<EditorLabEvidenceArtifact>,
        reproduction_command: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            capability,
            status: EditorLabEvidenceCapabilityResultStatus::Failed,
            probe,
            artifacts,
            reproduction_command: reproduction_command.into(),
            diagnostic: diagnostic.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabUnsupportedCheckDiagnostic {
    pub check: String,
    pub reason: String,
}

impl EditorLabUnsupportedCheckDiagnostic {
    pub fn new(check: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            check: check.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabAccessibilitySnapshot {
    pub scenario_id: String,
    pub labelled_controls: usize,
    pub disabled_reason_controls: usize,
    pub focusable_routes: usize,
    pub unsupported_checks: Vec<EditorLabUnsupportedCheckDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabPerformanceSnapshot {
    pub scenario_id: String,
    pub setup_micros: u64,
    pub retained_surface_micros: u64,
    pub artifact_count: usize,
    pub artifact_bytes: usize,
    pub unsupported_checks: Vec<EditorLabUnsupportedCheckDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabEvidenceRun {
    pub scenario_id: String,
    pub state_family: EditorLabScenarioStateFamily,
    pub status: EditorLabEvidenceRunStatus,
    pub runtime_path: String,
    pub provider_state: String,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
    pub accessibility: Option<EditorLabAccessibilitySnapshot>,
    pub performance: Option<EditorLabPerformanceSnapshot>,
    pub artifacts: Vec<EditorLabEvidenceArtifact>,
    pub unsupported_checks: Vec<EditorLabUnsupportedCheckDiagnostic>,
    #[serde(default)]
    pub capability_results: Vec<EditorLabEvidenceCapabilityResult>,
}

impl EditorLabEvidenceRun {
    pub fn passed(
        scenario: &EditorLabPreviewScenario,
        runtime_path: impl Into<String>,
        provider_state: impl Into<String>,
        artifacts: Vec<EditorLabEvidenceArtifact>,
    ) -> Self {
        Self {
            scenario_id: scenario.id.clone(),
            state_family: scenario.state_family,
            status: EditorLabEvidenceRunStatus::Passed,
            runtime_path: runtime_path.into(),
            provider_state: provider_state.into(),
            diagnostics: Vec::new(),
            accessibility: None,
            performance: None,
            artifacts,
            unsupported_checks: Vec::new(),
            capability_results: Vec::new(),
        }
    }

    pub fn with_diagnostics(mut self, diagnostics: Vec<UiDefinitionDiagnostic>) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    pub fn with_accessibility(mut self, accessibility: EditorLabAccessibilitySnapshot) -> Self {
        self.accessibility = Some(accessibility);
        self
    }

    pub fn with_performance(mut self, performance: EditorLabPerformanceSnapshot) -> Self {
        self.performance = Some(performance);
        self
    }

    pub fn with_unsupported_checks(
        mut self,
        unsupported_checks: Vec<EditorLabUnsupportedCheckDiagnostic>,
    ) -> Self {
        if !unsupported_checks.is_empty() {
            self.status = EditorLabEvidenceRunStatus::PassedWithUnsupportedChecks;
        }
        self.unsupported_checks = unsupported_checks;
        self
    }

    pub fn with_capability_results(
        mut self,
        capability_results: Vec<EditorLabEvidenceCapabilityResult>,
    ) -> Self {
        if capability_results
            .iter()
            .any(|result| result.status == EditorLabEvidenceCapabilityResultStatus::Failed)
        {
            self.status = EditorLabEvidenceRunStatus::Failed;
        }
        self.capability_results = capability_results;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabEvidenceManifest {
    pub manifest_version: u32,
    pub generated_by: String,
    pub target_profile: String,
    pub required_scenarios: Vec<EditorLabPreviewScenario>,
    pub runs: Vec<EditorLabEvidenceRun>,
}

impl EditorLabEvidenceManifest {
    pub fn current(
        generated_by: impl Into<String>,
        required_scenarios: Vec<EditorLabPreviewScenario>,
        runs: Vec<EditorLabEvidenceRun>,
    ) -> Self {
        Self {
            manifest_version: EDITOR_LAB_EVIDENCE_MANIFEST_VERSION,
            generated_by: generated_by.into(),
            target_profile: "runenwerk.editor.workspace.editor_design".to_string(),
            required_scenarios,
            runs,
        }
    }

    pub fn validate(&self) -> Result<(), UiDefinitionDiagnostic> {
        if self.manifest_version != EDITOR_LAB_EVIDENCE_MANIFEST_VERSION {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.manifest.unsupported_version",
                format!(
                    "unsupported Editor Lab evidence manifest version {}",
                    self.manifest_version
                ),
            ));
        }

        let required_by_id = self
            .required_scenarios
            .iter()
            .map(|scenario| (scenario.id.as_str(), scenario))
            .collect::<BTreeMap<_, _>>();
        let mut seen = BTreeSet::new();
        for run in &self.runs {
            let scenario = required_by_id
                .get(run.scenario_id.as_str())
                .ok_or_else(|| {
                    UiDefinitionDiagnostic::error(
                        "editor.lab.evidence.manifest.unknown_scenario",
                        format!(
                            "evidence run '{}' is not part of the required scenario catalog",
                            run.scenario_id
                        ),
                    )
                })?;
            if !seen.insert(run.scenario_id.clone()) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.duplicate_scenario",
                    format!(
                        "evidence scenario '{}' was recorded more than once",
                        run.scenario_id
                    ),
                ));
            }
            if run.state_family != scenario.state_family {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.family_mismatch",
                    format!(
                        "evidence scenario '{}' recorded {:?}, expected {:?}",
                        run.scenario_id, run.state_family, scenario.state_family
                    ),
                ));
            }
            if run.status == EditorLabEvidenceRunStatus::Failed {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.failed_scenario",
                    format!("evidence scenario '{}' failed", run.scenario_id),
                ));
            }
            if run.artifacts.is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.missing_artifact",
                    format!("evidence scenario '{}' has no artifacts", run.scenario_id),
                ));
            }
            if scenario.capture_requirement == EditorLabCaptureRequirement::RetainedVisualRequired
                && !run
                    .artifacts
                    .iter()
                    .any(|artifact| artifact.kind == EditorLabEvidenceArtifactKind::RetainedUiDebug)
            {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.missing_retained_visual",
                    format!(
                        "evidence scenario '{}' requires a retained visual artifact",
                        run.scenario_id
                    ),
                ));
            }
            if scenario.accessibility_required && run.accessibility.is_none() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.missing_accessibility",
                    format!(
                        "evidence scenario '{}' requires an accessibility snapshot",
                        run.scenario_id
                    ),
                ));
            }
            if scenario.performance_required && run.performance.is_none() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.missing_performance",
                    format!(
                        "evidence scenario '{}' requires a performance snapshot",
                        run.scenario_id
                    ),
                ));
            }
        }

        for scenario in &self.required_scenarios {
            if !seen.contains(&scenario.id) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.missing_scenario",
                    format!(
                        "required evidence scenario '{}' was not recorded",
                        scenario.id
                    ),
                ));
            }
        }
        Ok(())
    }

    pub fn diagnostics_snapshot(&self) -> Vec<UiDefinitionDiagnostic> {
        self.runs
            .iter()
            .flat_map(|run| run.diagnostics.iter().cloned())
            .collect()
    }

    pub fn unsupported_checks(&self) -> Vec<EditorLabUnsupportedCheckDiagnostic> {
        self.runs
            .iter()
            .flat_map(|run| run.unsupported_checks.iter().cloned())
            .collect()
    }

    pub fn validate_no_gap_capabilities(
        &self,
        required_capabilities: &[EditorLabEvidenceCapability],
    ) -> Result<(), UiDefinitionDiagnostic> {
        self.validate()?;

        let required = required_capabilities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if required.is_empty() {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.manifest.no_gap.empty_requirement",
                "no no-gap evidence capabilities were required",
            ));
        }

        let mut satisfied = BTreeSet::new();
        for run in &self.runs {
            for result in &run.capability_results {
                Self::validate_no_gap_capability_result(&run.scenario_id, result)?;
                if required.contains(&result.capability)
                    && matches!(
                        result.status,
                        EditorLabEvidenceCapabilityResultStatus::Captured
                            | EditorLabEvidenceCapabilityResultStatus::PlatformImpossible
                    )
                {
                    satisfied.insert(result.capability);
                }
            }
        }

        for capability in required {
            if !satisfied.contains(&capability) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.no_gap.missing_capability",
                    format!("no-gap evidence capability '{capability:?}' was not recorded"),
                ));
            }
        }
        Ok(())
    }

    fn validate_no_gap_capability_result(
        scenario_id: &str,
        result: &EditorLabEvidenceCapabilityResult,
    ) -> Result<(), UiDefinitionDiagnostic> {
        if result.probe.capability != result.capability {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.manifest.no_gap.probe_mismatch",
                format!(
                    "scenario '{scenario_id}' records {:?} with a {:?} probe",
                    result.capability, result.probe.capability
                ),
            ));
        }
        if result.probe.backend.trim().is_empty()
            || result.probe.environment.trim().is_empty()
            || result.probe.reason.trim().is_empty()
            || result.reproduction_command.trim().is_empty()
            || result.diagnostic.trim().is_empty()
        {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.manifest.no_gap.missing_probe_metadata",
                format!(
                    "scenario '{scenario_id}' result {:?} is missing typed probe metadata",
                    result.capability
                ),
            ));
        }
        if result.artifacts.is_empty() {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.manifest.no_gap.missing_result_artifact",
                format!(
                    "scenario '{scenario_id}' result {:?} has no evidence artifact",
                    result.capability
                ),
            ));
        }

        match result.status {
            EditorLabEvidenceCapabilityResultStatus::Captured => {
                if result.probe.support_status
                    != EditorLabEvidenceCapabilitySupportStatus::Supported
                {
                    return Err(UiDefinitionDiagnostic::error(
                        "editor.lab.evidence.manifest.no_gap.status_mismatch",
                        format!(
                            "scenario '{scenario_id}' captured {:?} with a non-supported probe",
                            result.capability
                        ),
                    ));
                }
                if result.artifacts.iter().all(|artifact| {
                    matches!(
                        artifact.kind,
                        EditorLabEvidenceArtifactKind::UnsupportedCheckReport
                            | EditorLabEvidenceArtifactKind::PlatformImpossibleReport
                            | EditorLabEvidenceArtifactKind::EvidenceManifest
                    )
                }) {
                    return Err(UiDefinitionDiagnostic::error(
                        "editor.lab.evidence.manifest.no_gap.descriptor_only_capture",
                        format!(
                            "scenario '{scenario_id}' captured {:?} without a concrete runtime artifact",
                            result.capability
                        ),
                    ));
                }
            }
            EditorLabEvidenceCapabilityResultStatus::PlatformImpossible => {
                if result.probe.support_status
                    != EditorLabEvidenceCapabilitySupportStatus::PlatformImpossible
                {
                    return Err(UiDefinitionDiagnostic::error(
                        "editor.lab.evidence.manifest.no_gap.status_mismatch",
                        format!(
                            "scenario '{scenario_id}' marked {:?} platform-impossible with a supported probe",
                            result.capability
                        ),
                    ));
                }
                if !result.artifacts.iter().any(|artifact| {
                    artifact.kind == EditorLabEvidenceArtifactKind::PlatformImpossibleReport
                }) {
                    return Err(UiDefinitionDiagnostic::error(
                        "editor.lab.evidence.manifest.no_gap.missing_platform_impossible_report",
                        format!(
                            "scenario '{scenario_id}' platform-impossible {:?} lacks a typed report artifact",
                            result.capability
                        ),
                    ));
                }
            }
            EditorLabEvidenceCapabilityResultStatus::Failed => {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.manifest.no_gap.failed_capability",
                    format!(
                        "scenario '{scenario_id}' result {:?} failed",
                        result.capability
                    ),
                ));
            }
        }

        Ok(())
    }
}

pub fn editor_lab_preview_scenarios() -> Vec<EditorLabPreviewScenario> {
    use EditorLabCaptureRequirement::{
        DiagnosticsOnly, NativeScreenshotPreferred, RetainedVisualRequired,
    };
    use EditorLabScenarioStateFamily::{
        Accessibility, Apply, DegradedProvider, Error, Performance, Reload, Rollback, Success,
        Warning,
    };

    vec![
        EditorLabPreviewScenario::new(
            "editor-lab.success",
            "Editor Lab surfaces mount with retained visual output",
            Success,
            "SwitchWorkspaceProfile -> build_editor_shell_frame_model",
            NativeScreenshotPreferred,
        ),
        EditorLabPreviewScenario::new(
            "editor-lab.warning",
            "Preview console warning is visible in the app-hosted command surface",
            Warning,
            "RunenwerkEditorApp::append_console_warning -> command_diff surface",
            RetainedVisualRequired,
        ),
        EditorLabPreviewScenario::new(
            "editor-lab.error",
            "Invalid project package preserves input and typed diagnostics",
            Error,
            "SelfAuthoringWorkspaceState::load_project_package_from_ron",
            DiagnosticsOnly,
        ),
        EditorLabPreviewScenario::new(
            "editor-lab.reload",
            "Saved project package reloads without live activation",
            Reload,
            "SaveEditorLabProjectPackage -> ReloadEditorLabProjectPackage",
            RetainedVisualRequired,
        ),
        EditorLabPreviewScenario::new(
            "editor-lab.apply",
            "Accepted apply produces review and runtime activation report",
            Apply,
            "BuildSelectedEditorDefinitionApplyReview -> ApplySelectedEditorDefinition",
            RetainedVisualRequired,
        ),
        EditorLabPreviewScenario::new(
            "editor-lab.rollback",
            "Snapshot-backed rollback records a typed rollback report",
            Rollback,
            "RollbackSelectedEditorDefinition",
            RetainedVisualRequired,
        ),
        EditorLabPreviewScenario::new(
            "editor-lab.degraded-provider",
            "Non-previewable selection renders typed degraded provider surface",
            DegradedProvider,
            "Select theme document -> ui_canvas degraded Editor Lab surface",
            RetainedVisualRequired,
        ),
        EditorLabPreviewScenario::new(
            "editor-lab.accessibility",
            "Editor Lab controls expose labels, routes, and disabled reasons",
            Accessibility,
            "build_editor_shell_frame_model route and retained text inspection",
            RetainedVisualRequired,
        ),
        EditorLabPreviewScenario::new(
            "editor-lab.performance",
            "Scenario setup and retained-surface formation timing is recorded",
            Performance,
            "std::time::Instant around app-hosted frame formation",
            RetainedVisualRequired,
        ),
    ]
}

pub fn evidence_warning(
    code: impl Into<String>,
    message: impl Into<String>,
) -> UiDefinitionDiagnostic {
    UiDefinitionDiagnostic {
        severity: UiDefinitionDiagnosticSeverity::Warning,
        code: code.into(),
        message: message.into(),
        path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_scenario_catalog_has_required_state_families() {
        let scenarios = editor_lab_preview_scenarios();
        let families = scenarios
            .iter()
            .map(|scenario| scenario.state_family)
            .collect::<BTreeSet<_>>();

        assert_eq!(scenarios.len(), 9);
        assert!(families.contains(&EditorLabScenarioStateFamily::Success));
        assert!(families.contains(&EditorLabScenarioStateFamily::Warning));
        assert!(families.contains(&EditorLabScenarioStateFamily::Error));
        assert!(families.contains(&EditorLabScenarioStateFamily::Reload));
        assert!(families.contains(&EditorLabScenarioStateFamily::Apply));
        assert!(families.contains(&EditorLabScenarioStateFamily::Rollback));
        assert!(families.contains(&EditorLabScenarioStateFamily::DegradedProvider));
        assert!(families.contains(&EditorLabScenarioStateFamily::Accessibility));
        assert!(families.contains(&EditorLabScenarioStateFamily::Performance));
    }

    #[test]
    fn manifest_validation_rejects_descriptor_only_runs() {
        let scenarios = editor_lab_preview_scenarios();
        let run = EditorLabEvidenceRun::passed(
            &scenarios[0],
            "descriptor only",
            "not executed",
            Vec::new(),
        );
        let manifest = EditorLabEvidenceManifest::current("test", scenarios, vec![run]);

        let error = manifest
            .validate()
            .expect_err("descriptor-only evidence should be rejected");
        assert_eq!(error.code, "editor.lab.evidence.manifest.missing_artifact");
    }

    #[test]
    fn no_gap_validation_accepts_captured_capability_evidence() {
        let scenario = editor_lab_preview_scenarios()
            .into_iter()
            .find(|scenario| scenario.id == "editor-lab.success")
            .expect("success scenario should exist");
        let artifact = EditorLabEvidenceArtifact::new(
            EditorLabEvidenceArtifactKind::RetainedUiDebug,
            "artifacts/success-retained-surface-debug.txt",
            128,
            "retained visual truth artifact",
        );
        let run = EditorLabEvidenceRun::passed(
            &scenario,
            "build_editor_shell_frame_model",
            "provider surfaces mounted",
            vec![artifact.clone()],
        )
        .with_capability_results(vec![EditorLabEvidenceCapabilityResult::captured(
            EditorLabEvidenceCapability::RetainedVisualTruth,
            EditorLabEvidenceCapabilityProbe::supported(
                EditorLabEvidenceCapability::RetainedVisualTruth,
                "retained-ui-test",
                "cargo test",
                "retained surface text and provider frames were captured",
            ),
            vec![artifact],
            "cargo test -p runenwerk_editor editor_lab_evidence",
            "retained visual evidence was captured by the app-owned evidence harness",
        )]);
        let manifest = EditorLabEvidenceManifest::current("test", vec![scenario], vec![run]);

        manifest
            .validate_no_gap_capabilities(&[EditorLabEvidenceCapability::RetainedVisualTruth])
            .expect("captured no-gap capability evidence should validate");
    }

    #[test]
    fn no_gap_validation_rejects_platform_impossible_without_probe_metadata() {
        let scenario = editor_lab_preview_scenarios()
            .into_iter()
            .find(|scenario| scenario.id == "editor-lab.accessibility")
            .expect("accessibility scenario should exist");
        let retained_artifact = EditorLabEvidenceArtifact::new(
            EditorLabEvidenceArtifactKind::RetainedUiDebug,
            "artifacts/accessibility-retained-surface-debug.txt",
            128,
            "retained accessibility surface",
        );
        let platform_report = EditorLabEvidenceArtifact::new(
            EditorLabEvidenceArtifactKind::PlatformImpossibleReport,
            "artifacts/platform-impossible-results.ron",
            128,
            "typed platform-impossible report",
        );
        let run = EditorLabEvidenceRun::passed(
            &scenario,
            "build_editor_shell_frame_model route and retained text inspection",
            "labels, routes, and disabled reasons inspected",
            vec![retained_artifact, platform_report.clone()],
        )
        .with_accessibility(EditorLabAccessibilitySnapshot {
            scenario_id: "editor-lab.accessibility".to_string(),
            labelled_controls: 1,
            disabled_reason_controls: 1,
            focusable_routes: 1,
            unsupported_checks: Vec::new(),
        })
        .with_capability_results(vec![EditorLabEvidenceCapabilityResult {
            capability: EditorLabEvidenceCapability::NativeFocusTraversal,
            status: EditorLabEvidenceCapabilityResultStatus::PlatformImpossible,
            probe: EditorLabEvidenceCapabilityProbe {
                capability: EditorLabEvidenceCapability::NativeFocusTraversal,
                backend: String::new(),
                environment: String::new(),
                support_status: EditorLabEvidenceCapabilitySupportStatus::PlatformImpossible,
                reason: String::new(),
            },
            artifacts: vec![platform_report],
            reproduction_command: "cargo test -p runenwerk_editor editor_lab_evidence".to_string(),
            diagnostic: "native focus traversal is not available".to_string(),
        }]);
        let manifest = EditorLabEvidenceManifest::current("test", vec![scenario], vec![run]);

        let error = manifest
            .validate_no_gap_capabilities(&[EditorLabEvidenceCapability::NativeFocusTraversal])
            .expect_err("platform-impossible evidence requires typed probe metadata");
        assert_eq!(
            error.code,
            "editor.lab.evidence.manifest.no_gap.missing_probe_metadata"
        );
    }
}
