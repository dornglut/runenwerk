//! App-owned Editor Lab preview scenario and runtime evidence contracts.

pub mod game_runtime;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabEvidenceArtifactProvenance {
    ProductPath,
    #[default]
    DescriptorCompatibility,
    UnsupportedCheck,
    ImportedManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabEvidenceArtifact {
    pub kind: EditorLabEvidenceArtifactKind,
    pub path: String,
    pub bytes: usize,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub provenance: EditorLabEvidenceArtifactProvenance,
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
            digest: String::new(),
            provenance: EditorLabEvidenceArtifactProvenance::DescriptorCompatibility,
            description: description.into(),
        }
    }

    pub fn from_content(
        kind: EditorLabEvidenceArtifactKind,
        path: impl Into<String>,
        content: &[u8],
        provenance: EditorLabEvidenceArtifactProvenance,
        description: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            path: path.into(),
            bytes: content.len(),
            digest: format!("blake3:{}", blake3::hash(content).to_hex()),
            provenance,
            description: description.into(),
        }
    }

    pub fn with_digest(
        mut self,
        digest: impl Into<String>,
        provenance: EditorLabEvidenceArtifactProvenance,
    ) -> Self {
        self.digest = digest.into();
        self.provenance = provenance;
        self
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
    DesignSystemReport,
    GraphCanvasReport,
    ProductPatternReport,
    SurfaceReadinessReport,
    WorkbenchReport,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabEvidenceFreshness {
    Fresh,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabEvidenceCaptureMode {
    ExplicitCommand,
    AutomaticFrame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabPerformanceBaselineKind {
    Resize,
    CanvasInteraction,
    CatalogProjection,
    DiagnosticsProjection,
    FrameBuild,
}

pub const UI_DESIGNER_SCENARIO_BASELINE_KINDS: [EditorLabPerformanceBaselineKind; 5] = [
    EditorLabPerformanceBaselineKind::Resize,
    EditorLabPerformanceBaselineKind::CanvasInteraction,
    EditorLabPerformanceBaselineKind::CatalogProjection,
    EditorLabPerformanceBaselineKind::DiagnosticsProjection,
    EditorLabPerformanceBaselineKind::FrameBuild,
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabMeasurementProvenance {
    ProductPath,
    DescriptorProjection,
    #[default]
    StatusSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabPerformanceBaseline {
    pub kind: EditorLabPerformanceBaselineKind,
    pub elapsed_micros: u64,
    pub sample_count: usize,
    #[serde(default)]
    pub provenance: EditorLabMeasurementProvenance,
    pub description: String,
}

impl EditorLabPerformanceBaseline {
    pub fn new(
        kind: EditorLabPerformanceBaselineKind,
        elapsed_micros: u64,
        sample_count: usize,
        description: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            elapsed_micros,
            sample_count,
            provenance: EditorLabMeasurementProvenance::StatusSummary,
            description: description.into(),
        }
    }

    pub fn product_path(
        kind: EditorLabPerformanceBaselineKind,
        elapsed_micros: u64,
        sample_count: usize,
        description: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            elapsed_micros,
            sample_count,
            provenance: EditorLabMeasurementProvenance::ProductPath,
            description: description.into(),
        }
    }

    pub fn descriptor_projection(
        kind: EditorLabPerformanceBaselineKind,
        elapsed_micros: u64,
        sample_count: usize,
        description: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            elapsed_micros,
            sample_count,
            provenance: EditorLabMeasurementProvenance::DescriptorProjection,
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorLabDescriptorCompatibility {
    Compatible,
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabReadOnlyFixtureBindingDescriptor {
    pub fixture_id: String,
    pub binding_id: String,
    pub target_profile: String,
    pub compatibility: EditorLabDescriptorCompatibility,
    pub read_only: bool,
    pub source: String,
}

impl EditorLabReadOnlyFixtureBindingDescriptor {
    pub fn new(
        fixture_id: impl Into<String>,
        binding_id: impl Into<String>,
        target_profile: impl Into<String>,
        compatibility: EditorLabDescriptorCompatibility,
        source: impl Into<String>,
    ) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            binding_id: binding_id.into(),
            target_profile: target_profile.into(),
            compatibility,
            read_only: true,
            source: source.into(),
        }
    }

    pub fn mutable_for_test(mut self) -> Self {
        self.read_only = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabValidatedIntentDescriptor {
    pub intent_id: String,
    pub target_profile: String,
    pub command_descriptor: String,
    pub validated: bool,
    pub executes_runtime_command: bool,
}

impl EditorLabValidatedIntentDescriptor {
    pub fn new(
        intent_id: impl Into<String>,
        target_profile: impl Into<String>,
        command_descriptor: impl Into<String>,
    ) -> Self {
        Self {
            intent_id: intent_id.into(),
            target_profile: target_profile.into(),
            command_descriptor: command_descriptor.into(),
            validated: true,
            executes_runtime_command: false,
        }
    }

    pub fn unvalidated_for_test(mut self) -> Self {
        self.validated = false;
        self
    }

    pub fn runtime_command_for_test(mut self) -> Self {
        self.executes_runtime_command = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabSourceRevision {
    pub document_id: String,
    pub schema_version: u32,
    pub content_hash: String,
    pub session_epoch: u64,
}

impl EditorLabSourceRevision {
    pub fn new(
        document_id: impl Into<String>,
        schema_version: u32,
        content_hash: impl Into<String>,
        session_epoch: u64,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            schema_version,
            content_hash: content_hash.into(),
            session_epoch,
        }
    }

    pub fn display_label(&self) -> String {
        format!(
            "{}@schema{}:{}:epoch{}",
            self.document_id, self.schema_version, self.content_hash, self.session_epoch
        )
    }

    pub fn validate(&self) -> Result<(), UiDefinitionDiagnostic> {
        if self.document_id.trim().is_empty()
            || self.content_hash.trim().is_empty()
            || !self.content_hash.starts_with("blake3:")
        {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.source_revision.invalid",
                "source revision requires a document id and blake3 content hash",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabScenarioEvidenceIdentity {
    pub package_id: String,
    pub document_id: String,
    pub source_revision: EditorLabSourceRevision,
    pub target_profile: String,
    pub scenario_id: String,
    pub capture_mode: EditorLabEvidenceCaptureMode,
    pub freshness: EditorLabEvidenceFreshness,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
}

impl EditorLabScenarioEvidenceIdentity {
    pub fn new(
        package_id: impl Into<String>,
        document_id: impl Into<String>,
        source_revision: EditorLabSourceRevision,
        target_profile: impl Into<String>,
        scenario_id: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            document_id: document_id.into(),
            source_revision,
            target_profile: target_profile.into(),
            scenario_id: scenario_id.into(),
            capture_mode: EditorLabEvidenceCaptureMode::ExplicitCommand,
            freshness: EditorLabEvidenceFreshness::Fresh,
            diagnostics: Vec::new(),
        }
    }

    pub fn source_version(&self) -> String {
        self.source_revision.display_label()
    }

    pub fn with_capture_mode(mut self, capture_mode: EditorLabEvidenceCaptureMode) -> Self {
        self.capture_mode = capture_mode;
        self
    }

    pub fn with_freshness(mut self, freshness: EditorLabEvidenceFreshness) -> Self {
        self.freshness = freshness;
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: Vec<UiDefinitionDiagnostic>) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    fn validate_common(&self) -> Result<(), UiDefinitionDiagnostic> {
        for (field, value) in [
            ("package_id", self.package_id.as_str()),
            ("document_id", self.document_id.as_str()),
            ("target_profile", self.target_profile.as_str()),
            ("scenario_id", self.scenario_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.missing_identity",
                    format!("scenario evidence packet is missing {field}"),
                ));
            }
        }
        self.source_revision.validate()?;

        if self.document_id != self.source_revision.document_id {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.source_document_mismatch",
                "scenario evidence source revision document does not match packet document",
            ));
        }

        if self.capture_mode != EditorLabEvidenceCaptureMode::ExplicitCommand {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.implicit_capture",
                "scenario evidence capture must be explicit and must not run automatically every frame",
            ));
        }

        if self.freshness != EditorLabEvidenceFreshness::Fresh {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.stale_packet",
                "scenario evidence packet is stale",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabDescriptorCompatibilityEvidencePacket {
    pub identity: EditorLabScenarioEvidenceIdentity,
    pub artifacts: Vec<EditorLabEvidenceArtifact>,
    pub unsupported_checks: Vec<EditorLabUnsupportedCheckDiagnostic>,
    #[serde(default)]
    pub fixture_bindings: Vec<EditorLabReadOnlyFixtureBindingDescriptor>,
    #[serde(default)]
    pub intent_descriptors: Vec<EditorLabValidatedIntentDescriptor>,
}

impl EditorLabDescriptorCompatibilityEvidencePacket {
    pub fn new(
        package_id: impl Into<String>,
        document_id: impl Into<String>,
        source_revision: EditorLabSourceRevision,
        target_profile: impl Into<String>,
        scenario_id: impl Into<String>,
    ) -> Self {
        Self {
            identity: EditorLabScenarioEvidenceIdentity::new(
                package_id,
                document_id,
                source_revision,
                target_profile,
                scenario_id,
            ),
            artifacts: Vec::new(),
            unsupported_checks: Vec::new(),
            fixture_bindings: Vec::new(),
            intent_descriptors: Vec::new(),
        }
    }

    pub fn with_capture_mode(mut self, capture_mode: EditorLabEvidenceCaptureMode) -> Self {
        self.identity.capture_mode = capture_mode;
        self
    }

    pub fn with_freshness(mut self, freshness: EditorLabEvidenceFreshness) -> Self {
        self.identity.freshness = freshness;
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: Vec<UiDefinitionDiagnostic>) -> Self {
        self.identity.diagnostics = diagnostics;
        self
    }

    pub fn with_artifacts(mut self, artifacts: Vec<EditorLabEvidenceArtifact>) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn with_unsupported_checks(
        mut self,
        unsupported_checks: Vec<EditorLabUnsupportedCheckDiagnostic>,
    ) -> Self {
        self.unsupported_checks = unsupported_checks;
        self
    }

    pub fn with_fixture_bindings(
        mut self,
        fixture_bindings: Vec<EditorLabReadOnlyFixtureBindingDescriptor>,
    ) -> Self {
        self.fixture_bindings = fixture_bindings;
        self
    }

    pub fn with_intent_descriptors(
        mut self,
        intent_descriptors: Vec<EditorLabValidatedIntentDescriptor>,
    ) -> Self {
        self.intent_descriptors = intent_descriptors;
        self
    }

    pub fn validate_descriptor_compatibility(&self) -> Result<(), UiDefinitionDiagnostic> {
        self.identity.validate_common()?;

        if self.artifacts.is_empty() && self.unsupported_checks.is_empty() {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.missing_artifact_or_reason",
                "scenario evidence packet needs at least one artifact reference or typed unsupported reason",
            ));
        }

        for artifact in &self.artifacts {
            if artifact.path.trim().is_empty() || artifact.description.trim().is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.invalid_artifact",
                    "scenario evidence artifact is missing path or description",
                ));
            }
            if artifact.provenance == EditorLabEvidenceArtifactProvenance::ProductPath {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.descriptor.product_artifact",
                    "descriptor compatibility evidence must not claim product-path artifacts",
                ));
            }
        }

        for unsupported in &self.unsupported_checks {
            if unsupported.check.trim().is_empty() || unsupported.reason.trim().is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.invalid_unsupported_reason",
                    "scenario evidence unsupported check reason is incomplete",
                ));
            }
        }

        if self.fixture_bindings.is_empty() {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.missing_fixture_binding",
                "scenario evidence packet requires at least one read-only fixture or binding descriptor",
            ));
        }
        for binding in &self.fixture_bindings {
            for (field, value) in [
                ("fixture_id", binding.fixture_id.as_str()),
                ("binding_id", binding.binding_id.as_str()),
                ("target_profile", binding.target_profile.as_str()),
                ("source", binding.source.as_str()),
            ] {
                if value.trim().is_empty() {
                    return Err(UiDefinitionDiagnostic::error(
                        "editor.lab.evidence.scenario.invalid_fixture_binding",
                        format!("scenario evidence fixture binding is missing {field}"),
                    ));
                }
            }
            if binding.target_profile != self.identity.target_profile {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.fixture_binding_target_mismatch",
                    "scenario evidence fixture binding target profile does not match the packet",
                ));
            }
            if !binding.read_only {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.mutable_fixture_binding",
                    "scenario evidence fixture and binding descriptors must be read-only",
                ));
            }
        }

        if self.intent_descriptors.is_empty() {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.missing_intent_descriptor",
                "scenario evidence packet requires at least one validated intent descriptor",
            ));
        }
        for intent in &self.intent_descriptors {
            for (field, value) in [
                ("intent_id", intent.intent_id.as_str()),
                ("target_profile", intent.target_profile.as_str()),
                ("command_descriptor", intent.command_descriptor.as_str()),
            ] {
                if value.trim().is_empty() {
                    return Err(UiDefinitionDiagnostic::error(
                        "editor.lab.evidence.scenario.invalid_intent_descriptor",
                        format!("scenario evidence intent descriptor is missing {field}"),
                    ));
                }
            }
            if intent.target_profile != self.identity.target_profile {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.intent_target_mismatch",
                    "scenario evidence intent descriptor target profile does not match the packet",
                ));
            }
            if !intent.validated {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.unvalidated_intent",
                    "scenario evidence intent descriptors must be validated proposals",
                ));
            }
            if intent.executes_runtime_command {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.runtime_command_execution",
                    "scenario evidence intent descriptors must not execute game-runtime commands",
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabRuntimeProductEvidencePacket {
    pub identity: EditorLabScenarioEvidenceIdentity,
    pub artifacts: Vec<EditorLabEvidenceArtifact>,
    pub unsupported_checks: Vec<EditorLabUnsupportedCheckDiagnostic>,
    pub performance_baselines: Vec<EditorLabPerformanceBaseline>,
}

impl EditorLabRuntimeProductEvidencePacket {
    pub fn new(
        package_id: impl Into<String>,
        document_id: impl Into<String>,
        source_revision: EditorLabSourceRevision,
        target_profile: impl Into<String>,
        scenario_id: impl Into<String>,
    ) -> Self {
        Self {
            identity: EditorLabScenarioEvidenceIdentity::new(
                package_id,
                document_id,
                source_revision,
                target_profile,
                scenario_id,
            ),
            artifacts: Vec::new(),
            unsupported_checks: Vec::new(),
            performance_baselines: Vec::new(),
        }
    }

    pub fn with_capture_mode(mut self, capture_mode: EditorLabEvidenceCaptureMode) -> Self {
        self.identity.capture_mode = capture_mode;
        self
    }

    pub fn with_freshness(mut self, freshness: EditorLabEvidenceFreshness) -> Self {
        self.identity.freshness = freshness;
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: Vec<UiDefinitionDiagnostic>) -> Self {
        self.identity.diagnostics = diagnostics;
        self
    }

    pub fn with_artifacts(mut self, artifacts: Vec<EditorLabEvidenceArtifact>) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn with_unsupported_checks(
        mut self,
        unsupported_checks: Vec<EditorLabUnsupportedCheckDiagnostic>,
    ) -> Self {
        self.unsupported_checks = unsupported_checks;
        self
    }

    pub fn with_performance_baselines(
        mut self,
        performance_baselines: Vec<EditorLabPerformanceBaseline>,
    ) -> Self {
        self.performance_baselines = performance_baselines;
        self
    }

    pub fn validate_runtime_product_evidence(&self) -> Result<(), UiDefinitionDiagnostic> {
        self.identity.validate_common()?;

        if self.artifacts.is_empty() {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.runtime.missing_artifact",
                "runtime product evidence requires at least one product-path artifact",
            ));
        }

        for artifact in &self.artifacts {
            if artifact.path.trim().is_empty()
                || artifact.description.trim().is_empty()
                || artifact.bytes == 0
                || !artifact.digest.starts_with("blake3:")
            {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.runtime.invalid_artifact",
                    "runtime product evidence artifact requires path, description, bytes, and blake3 digest",
                ));
            }
            if artifact.path.starts_with("memory://")
                || artifact.provenance != EditorLabEvidenceArtifactProvenance::ProductPath
            {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.runtime.non_product_artifact",
                    "runtime product evidence requires product-path artifact provenance",
                ));
            }
        }

        let mut seen = BTreeSet::new();
        for baseline in &self.performance_baselines {
            if !seen.insert(baseline.kind) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.duplicate_baseline",
                    format!(
                        "scenario evidence baseline {:?} is recorded more than once",
                        baseline.kind
                    ),
                ));
            }
            if baseline.sample_count == 0 || baseline.description.trim().is_empty() {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.scenario.invalid_baseline",
                    format!(
                        "scenario evidence baseline {:?} is missing samples or description",
                        baseline.kind
                    ),
                ));
            }
            if baseline.provenance != EditorLabMeasurementProvenance::ProductPath {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.lab.evidence.runtime.non_product_baseline",
                    format!(
                        "runtime evidence baseline {:?} was not measured through a product path",
                        baseline.kind
                    ),
                ));
            }
        }

        let required = UI_DESIGNER_SCENARIO_BASELINE_KINDS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if !required.is_subset(&seen) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.missing_baseline",
                "scenario evidence packet is missing one or more required performance baselines",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLabScenarioEvidencePacket {
    DescriptorCompatibility(EditorLabDescriptorCompatibilityEvidencePacket),
    RuntimeProduct(EditorLabRuntimeProductEvidencePacket),
}

impl EditorLabScenarioEvidencePacket {
    pub fn descriptor(packet: EditorLabDescriptorCompatibilityEvidencePacket) -> Self {
        Self::DescriptorCompatibility(packet)
    }

    pub fn runtime(packet: EditorLabRuntimeProductEvidencePacket) -> Self {
        Self::RuntimeProduct(packet)
    }

    pub fn identity(&self) -> &EditorLabScenarioEvidenceIdentity {
        match self {
            Self::DescriptorCompatibility(packet) => &packet.identity,
            Self::RuntimeProduct(packet) => &packet.identity,
        }
    }

    pub fn document_id(&self) -> &str {
        &self.identity().document_id
    }

    pub fn source_revision(&self) -> &EditorLabSourceRevision {
        &self.identity().source_revision
    }

    pub fn source_version(&self) -> String {
        self.identity().source_version()
    }

    pub fn target_profile(&self) -> &str {
        &self.identity().target_profile
    }

    pub fn scenario_id(&self) -> &str {
        &self.identity().scenario_id
    }

    pub fn capture_mode(&self) -> EditorLabEvidenceCaptureMode {
        self.identity().capture_mode
    }

    pub fn diagnostics(&self) -> &[UiDefinitionDiagnostic] {
        &self.identity().diagnostics
    }

    pub fn artifacts(&self) -> &[EditorLabEvidenceArtifact] {
        match self {
            Self::DescriptorCompatibility(packet) => &packet.artifacts,
            Self::RuntimeProduct(packet) => &packet.artifacts,
        }
    }

    pub fn unsupported_checks(&self) -> &[EditorLabUnsupportedCheckDiagnostic] {
        match self {
            Self::DescriptorCompatibility(packet) => &packet.unsupported_checks,
            Self::RuntimeProduct(packet) => &packet.unsupported_checks,
        }
    }

    pub fn fixture_bindings(&self) -> &[EditorLabReadOnlyFixtureBindingDescriptor] {
        match self {
            Self::DescriptorCompatibility(packet) => &packet.fixture_bindings,
            Self::RuntimeProduct(_) => &[],
        }
    }

    pub fn intent_descriptors(&self) -> &[EditorLabValidatedIntentDescriptor] {
        match self {
            Self::DescriptorCompatibility(packet) => &packet.intent_descriptors,
            Self::RuntimeProduct(_) => &[],
        }
    }

    pub fn performance_baselines(&self) -> &[EditorLabPerformanceBaseline] {
        match self {
            Self::DescriptorCompatibility(_) => &[],
            Self::RuntimeProduct(packet) => &packet.performance_baselines,
        }
    }

    pub fn is_runtime_product(&self) -> bool {
        matches!(self, Self::RuntimeProduct(_))
    }

    pub fn validate_scenario_evidence(&self) -> Result<(), UiDefinitionDiagnostic> {
        match self {
            Self::DescriptorCompatibility(packet) => packet.validate_descriptor_compatibility(),
            Self::RuntimeProduct(packet) => packet.validate_runtime_product_evidence(),
        }
    }

    pub fn validate_runtime_product_evidence(&self) -> Result<(), UiDefinitionDiagnostic> {
        match self {
            Self::RuntimeProduct(packet) => packet.validate_runtime_product_evidence(),
            Self::DescriptorCompatibility(_) => Err(UiDefinitionDiagnostic::error(
                "editor.lab.evidence.runtime.descriptor_only",
                "descriptor compatibility evidence cannot satisfy runtime product readiness",
            )),
        }
    }
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

    #[test]
    fn scenario_evidence_splits_descriptor_and_runtime_packets() {
        let revision = EditorLabSourceRevision::new(
            "runenwerk.editor.toolbar",
            1,
            "blake3:0123456789abcdef",
            7,
        );
        let descriptor = EditorLabScenarioEvidencePacket::descriptor(
            EditorLabDescriptorCompatibilityEvidencePacket::new(
                "runenwerk.editor.ui_designer_workbench.v1",
                "runenwerk.editor.toolbar",
                revision.clone(),
                "game.runtime",
                "ui-designer.v1-closure.pm005.game-runtime",
            )
            .with_artifacts(vec![EditorLabEvidenceArtifact::from_content(
                EditorLabEvidenceArtifactKind::UnsupportedCheckReport,
                "evidence://ui-designer/v1-closure/pm005/game-runtime/unsupported",
                b"typed unsupported check report",
                EditorLabEvidenceArtifactProvenance::UnsupportedCheck,
                "typed unsupported check report",
            )])
            .with_unsupported_checks(vec![EditorLabUnsupportedCheckDiagnostic::new(
                "concrete game HUD runtime",
                "PT-GAME-RUNTIME-UI owns concrete game HUD behavior",
            )])
            .with_fixture_bindings(vec![EditorLabReadOnlyFixtureBindingDescriptor::new(
                "fixture.game-runtime.safe-area",
                "binding.game-runtime.hud-data",
                "game.runtime",
                EditorLabDescriptorCompatibility::Compatible,
                "read-only game.runtime fixture descriptor",
            )])
            .with_intent_descriptors(vec![EditorLabValidatedIntentDescriptor::new(
                "intent.game-runtime.open-hud-preview",
                "game.runtime",
                "validated descriptor only; no runtime command is executed",
            )]),
        );

        descriptor
            .validate_scenario_evidence()
            .expect("descriptor compatibility packet should validate");
        let error = descriptor
            .validate_runtime_product_evidence()
            .expect_err("descriptor packets must not satisfy runtime product evidence");
        assert_eq!(error.code, "editor.lab.evidence.runtime.descriptor_only");

        let runtime = EditorLabScenarioEvidencePacket::runtime(
            EditorLabRuntimeProductEvidencePacket::new(
                "runenwerk.editor.ui_designer_workbench.v1",
                "runenwerk.editor.toolbar",
                revision,
                "editor.workbench",
                "ui-designer.v1-closure.pm005.editor-workbench",
            )
            .with_artifacts(vec![EditorLabEvidenceArtifact::from_content(
                EditorLabEvidenceArtifactKind::RetainedUiDebug,
                "evidence://ui-designer/runtime/retained-ui-debug",
                b"retained ui debug",
                EditorLabEvidenceArtifactProvenance::ProductPath,
                "retained UI Designer surface evidence",
            )])
            .with_performance_baselines(
                UI_DESIGNER_SCENARIO_BASELINE_KINDS
                    .iter()
                    .copied()
                    .map(|kind| {
                        EditorLabPerformanceBaseline::product_path(kind, 1, 1, "test baseline")
                    })
                    .collect(),
            ),
        );

        runtime
            .validate_runtime_product_evidence()
            .expect("runtime product packet should validate");
    }

    #[test]
    fn scenario_evidence_packet_rejects_implicit_stale_or_incomplete_capture() {
        let revision = EditorLabSourceRevision::new(
            "runenwerk.editor.toolbar",
            1,
            "blake3:0123456789abcdef",
            7,
        );
        let incomplete = EditorLabScenarioEvidencePacket::runtime(
            EditorLabRuntimeProductEvidencePacket::new(
                "runenwerk.editor.ui_designer_workbench.v1",
                "runenwerk.editor.toolbar",
                revision.clone(),
                "editor.workbench",
                "ui-designer.v1-closure.pm005.editor-workbench",
            )
            .with_capture_mode(EditorLabEvidenceCaptureMode::AutomaticFrame)
            .with_freshness(EditorLabEvidenceFreshness::Stale),
        );

        let error = incomplete
            .validate_scenario_evidence()
            .expect_err("implicit stale packet should fail before artifacts are considered");
        assert_eq!(error.code, "editor.lab.evidence.scenario.implicit_capture");

        let missing_baselines = EditorLabScenarioEvidencePacket::runtime(
            EditorLabRuntimeProductEvidencePacket::new(
                "runenwerk.editor.ui_designer_workbench.v1",
                "runenwerk.editor.toolbar",
                revision,
                "editor.workbench",
                "ui-designer.v1-closure.pm005.editor-workbench",
            )
            .with_artifacts(vec![EditorLabEvidenceArtifact::from_content(
                EditorLabEvidenceArtifactKind::RetainedUiDebug,
                "evidence://ui-designer/runtime/retained-ui-debug",
                b"retained ui debug",
                EditorLabEvidenceArtifactProvenance::ProductPath,
                "retained UI Designer surface evidence",
            )]),
        );

        let error = missing_baselines
            .validate_scenario_evidence()
            .expect_err("packet without all baseline kinds should fail");
        assert_eq!(error.code, "editor.lab.evidence.scenario.missing_baseline");
    }

    #[test]
    fn scenario_evidence_packet_rejects_mutable_fixture_or_runtime_intent() {
        let packet = EditorLabScenarioEvidencePacket::descriptor(
            EditorLabDescriptorCompatibilityEvidencePacket::new(
                "runenwerk.editor.ui_designer_workbench.v1",
                "runenwerk.editor.toolbar",
                EditorLabSourceRevision::new(
                    "runenwerk.editor.toolbar",
                    1,
                    "blake3:0123456789abcdef",
                    7,
                ),
                "game.runtime",
                "ui-designer.v1-closure.pm005.game-runtime",
            )
            .with_artifacts(vec![EditorLabEvidenceArtifact::from_content(
                EditorLabEvidenceArtifactKind::UnsupportedCheckReport,
                "evidence://ui-designer/v1-closure/pm005/game-runtime/unsupported",
                b"typed unsupported check report",
                EditorLabEvidenceArtifactProvenance::UnsupportedCheck,
                "typed unsupported check report",
            )])
            .with_fixture_bindings(vec![
                EditorLabReadOnlyFixtureBindingDescriptor::new(
                    "fixture.game-runtime.safe-area",
                    "binding.game-runtime.hud-data",
                    "game.runtime",
                    EditorLabDescriptorCompatibility::Compatible,
                    "read-only game.runtime fixture descriptor",
                )
                .mutable_for_test(),
            ])
            .with_intent_descriptors(vec![
                EditorLabValidatedIntentDescriptor::new(
                    "intent.game-runtime.open-hud-preview",
                    "game.runtime",
                    "validated descriptor only",
                )
                .runtime_command_for_test(),
            ]),
        );

        let error = packet
            .validate_scenario_evidence()
            .expect_err("mutable fixture descriptors should fail before runtime intents");
        assert_eq!(
            error.code,
            "editor.lab.evidence.scenario.mutable_fixture_binding"
        );
    }
}
