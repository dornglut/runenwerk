//! Machine-readable proof contracts for UI story reports.

use serde::{Deserialize, Serialize};

use crate::report::{
    UiStoryDiagnostic, UiStoryDiagnosticSeverity, UiStoryStageKind, UiStoryStageReport,
};

pub const UI_STORY_PROOF_CONTRACT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiStoryProofProducerId(String);

impl UiStoryProofProducerId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn for_stage(stage: UiStoryStageKind) -> Self {
        Self(format!("ui_story.stage.{}", proof_stage_key(stage)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiStoryProofKey(String);

impl UiStoryProofKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn for_stage(stage: UiStoryStageKind) -> Self {
        Self(proof_stage_key(stage).to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryProofSubject {
    Story,
    Source,
    Program,
    RuntimeArtifact,
    RuntimeView,
    RenderPrimitives,
    RenderData,
    StaticMount,
    PreviewFrame,
    MountEligibility,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryProofRequirement {
    pub producer: UiStoryProofProducerId,
    pub proof_key: UiStoryProofKey,
    pub stage: UiStoryStageKind,
    pub subject: UiStoryProofSubject,
    pub required: bool,
}

impl UiStoryProofRequirement {
    pub fn required_stage(stage: UiStoryStageKind, subject: UiStoryProofSubject) -> Self {
        Self {
            producer: UiStoryProofProducerId::for_stage(stage),
            proof_key: UiStoryProofKey::for_stage(stage),
            stage,
            subject,
            required: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryProofContract {
    pub version: u32,
    #[serde(default)]
    pub requirements: Vec<UiStoryProofRequirement>,
}

impl UiStoryProofContract {
    pub fn new(requirements: impl Into<Vec<UiStoryProofRequirement>>) -> Self {
        Self {
            version: UI_STORY_PROOF_CONTRACT_VERSION,
            requirements: requirements.into(),
        }
    }

    pub fn required_stages(&self) -> impl Iterator<Item = UiStoryStageKind> + '_ {
        self.requirements
            .iter()
            .filter(|requirement| requirement.required)
            .map(|requirement| requirement.stage)
    }
}

impl Default for UiStoryProofContract {
    fn default() -> Self {
        Self {
            version: UI_STORY_PROOF_CONTRACT_VERSION,
            requirements: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryProofEvidence {
    pub producer: UiStoryProofProducerId,
    pub proof_key: UiStoryProofKey,
    pub stage: UiStoryStageKind,
    pub subject: UiStoryProofSubject,
    #[serde(default)]
    pub diagnostics: Vec<UiStoryDiagnostic>,
    #[serde(default)]
    pub elapsed_micros: Option<u64>,
}

impl UiStoryProofEvidence {
    pub fn from_stage_report(report: &UiStoryStageReport) -> Self {
        Self {
            producer: UiStoryProofProducerId::for_stage(report.stage),
            proof_key: UiStoryProofKey::for_stage(report.stage),
            stage: report.stage,
            subject: proof_subject(report.stage),
            diagnostics: report.diagnostics.clone(),
            elapsed_micros: report.elapsed_micros,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryProofDiagnosticExpectation {
    pub producer: UiStoryProofProducerId,
    pub proof_key: UiStoryProofKey,
    pub stage: UiStoryStageKind,
    pub code: String,
    pub severity: UiStoryDiagnosticSeverity,
}

impl UiStoryProofDiagnosticExpectation {
    pub fn for_stage_error(stage: UiStoryStageKind, code: impl Into<String>) -> Self {
        Self {
            producer: UiStoryProofProducerId::for_stage(stage),
            proof_key: UiStoryProofKey::for_stage(stage),
            stage,
            code: code.into(),
            severity: UiStoryDiagnosticSeverity::Error,
        }
    }

    pub fn matches_diagnostic(&self, diagnostic: &UiStoryDiagnostic) -> bool {
        self.stage == diagnostic.stage
            && self.producer == UiStoryProofProducerId::for_stage(diagnostic.stage)
            && self.proof_key == UiStoryProofKey::for_stage(diagnostic.stage)
            && self.code == diagnostic.code
            && self.severity == diagnostic.severity
    }
}

fn proof_subject(stage: UiStoryStageKind) -> UiStoryProofSubject {
    match stage {
        UiStoryStageKind::Manifest => UiStoryProofSubject::Story,
        UiStoryStageKind::SourceLoad | UiStoryStageKind::SourceParse => UiStoryProofSubject::Source,
        UiStoryStageKind::DefinitionValidation
        | UiStoryStageKind::DefinitionNormalization
        | UiStoryStageKind::SchemaValidation
        | UiStoryStageKind::ControlPackage
        | UiStoryStageKind::ProgramFormation => UiStoryProofSubject::Program,
        UiStoryStageKind::Compiler | UiStoryStageKind::RuntimeArtifact => {
            UiStoryProofSubject::RuntimeArtifact
        }
        UiStoryStageKind::RuntimeView
        | UiStoryStageKind::Binding
        | UiStoryStageKind::HostRoutes
        | UiStoryStageKind::Layout
        | UiStoryStageKind::Style
        | UiStoryStageKind::Text
        | UiStoryStageKind::Accessibility
        | UiStoryStageKind::Interaction => UiStoryProofSubject::RuntimeView,
        UiStoryStageKind::RenderPrimitives => UiStoryProofSubject::RenderPrimitives,
        UiStoryStageKind::RenderData => UiStoryProofSubject::RenderData,
        UiStoryStageKind::StaticMount => UiStoryProofSubject::StaticMount,
        UiStoryStageKind::PreviewFrame => UiStoryProofSubject::PreviewFrame,
        UiStoryStageKind::MountEligibility | UiStoryStageKind::Verdict => {
            UiStoryProofSubject::MountEligibility
        }
    }
}

fn proof_stage_key(stage: UiStoryStageKind) -> &'static str {
    match stage {
        UiStoryStageKind::Manifest => "manifest",
        UiStoryStageKind::SourceLoad => "source_load",
        UiStoryStageKind::SourceParse => "source_parse",
        UiStoryStageKind::DefinitionValidation => "definition_validation",
        UiStoryStageKind::DefinitionNormalization => "definition_normalization",
        UiStoryStageKind::SchemaValidation => "schema_validation",
        UiStoryStageKind::ControlPackage => "control_package",
        UiStoryStageKind::ProgramFormation => "program_formation",
        UiStoryStageKind::Compiler => "compiler",
        UiStoryStageKind::RuntimeArtifact => "runtime_artifact",
        UiStoryStageKind::RuntimeView => "runtime_view",
        UiStoryStageKind::Binding => "binding",
        UiStoryStageKind::HostRoutes => "host_routes",
        UiStoryStageKind::Layout => "layout",
        UiStoryStageKind::Style => "style",
        UiStoryStageKind::Text => "text",
        UiStoryStageKind::Accessibility => "accessibility",
        UiStoryStageKind::Interaction => "interaction",
        UiStoryStageKind::RenderPrimitives => "render_primitives",
        UiStoryStageKind::RenderData => "render_data",
        UiStoryStageKind::StaticMount => "static_mount",
        UiStoryStageKind::PreviewFrame => "preview_frame",
        UiStoryStageKind::MountEligibility => "mount_eligibility",
        UiStoryStageKind::Verdict => "verdict",
    }
}

#[cfg(test)]
mod tests {
    use crate::report::{UiStoryDiagnostic, UiStoryDiagnosticSeverity, UiStoryStageKind};

    use super::*;

    #[test]
    fn proof_contract_exposes_stage_requirements() {
        let contract = UiStoryProofContract::new([
            UiStoryProofRequirement::required_stage(
                UiStoryStageKind::SourceLoad,
                UiStoryProofSubject::Source,
            ),
            UiStoryProofRequirement::required_stage(
                UiStoryStageKind::StaticMount,
                UiStoryProofSubject::StaticMount,
            ),
        ]);

        let stages = contract.required_stages().collect::<Vec<_>>();

        assert_eq!(
            stages,
            vec![UiStoryStageKind::SourceLoad, UiStoryStageKind::StaticMount]
        );
    }

    #[test]
    fn proof_diagnostic_expectation_matches_stage_producer_key_code_and_severity() {
        let expectation = UiStoryProofDiagnosticExpectation::for_stage_error(
            UiStoryStageKind::SourceLoad,
            "ui_gallery.story.source.read_failed",
        );

        assert!(expectation.matches_diagnostic(&UiStoryDiagnostic::new(
            "ui_gallery.story.source.read_failed",
            "fixture is absent",
            UiStoryStageKind::SourceLoad,
            UiStoryDiagnosticSeverity::Error,
        )));
        assert!(!expectation.matches_diagnostic(&UiStoryDiagnostic::new(
            "ui_gallery.story.source.read_failed",
            "fixture is absent",
            UiStoryStageKind::SourceLoad,
            UiStoryDiagnosticSeverity::Warning,
        )));
    }
}
