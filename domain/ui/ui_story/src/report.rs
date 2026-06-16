//! Unified report contracts returned by UI story runs.

use serde::{Deserialize, Serialize};

use crate::manifest::{UiStoryExpectedVerdict, UiStoryId, UiStoryManifest};

pub const DIAGNOSTIC_EXPECTED_FAILURE_PASSED: &str = "ui.story.verdict.expected_failure_passed";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiStoryStageKind {
    Manifest,
    SourceLoad,
    SourceParse,
    DefinitionValidation,
    DefinitionNormalization,
    SchemaValidation,
    ControlPackage,
    ProgramFormation,
    Compiler,
    RuntimeArtifact,
    RuntimeView,
    Binding,
    HostRoutes,
    Layout,
    Style,
    Text,
    Accessibility,
    Interaction,
    RenderPrimitives,
    RenderData,
    StaticMount,
    PreviewFrame,
    MountEligibility,
    Verdict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryStageStatus {
    Passed,
    Failed,
    Skipped,
    MissingProof,
}

impl UiStoryStageStatus {
    pub fn blocks_verdict(self) -> bool {
        matches!(self, Self::Failed | Self::MissingProof)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryDiagnostic {
    pub code: String,
    pub message: String,
    pub stage: UiStoryStageKind,
    pub severity: UiStoryDiagnosticSeverity,
}

impl UiStoryDiagnostic {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        stage: UiStoryStageKind,
        severity: UiStoryDiagnosticSeverity,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            stage,
            severity,
        }
    }

    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        stage: UiStoryStageKind,
    ) -> Self {
        Self::new(code, message, stage, UiStoryDiagnosticSeverity::Error)
    }

    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        stage: UiStoryStageKind,
    ) -> Self {
        Self::new(code, message, stage, UiStoryDiagnosticSeverity::Info)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryStageReport {
    pub stage: UiStoryStageKind,
    pub status: UiStoryStageStatus,
    #[serde(default)]
    pub diagnostics: Vec<UiStoryDiagnostic>,
    #[serde(default)]
    pub elapsed_micros: Option<u64>,
}

impl UiStoryStageReport {
    pub fn passed(stage: UiStoryStageKind) -> Self {
        Self {
            stage,
            status: UiStoryStageStatus::Passed,
            diagnostics: Vec::new(),
            elapsed_micros: None,
        }
    }

    pub fn failed(stage: UiStoryStageKind, diagnostics: Vec<UiStoryDiagnostic>) -> Self {
        Self {
            stage,
            status: UiStoryStageStatus::Failed,
            diagnostics,
            elapsed_micros: None,
        }
    }

    pub fn missing_proof(stage: UiStoryStageKind, diagnostic: UiStoryDiagnostic) -> Self {
        Self {
            stage,
            status: UiStoryStageStatus::MissingProof,
            diagnostics: vec![diagnostic],
            elapsed_micros: None,
        }
    }

    pub fn blocks_verdict(&self) -> bool {
        self.status.blocks_verdict()
            || self
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == UiStoryDiagnosticSeverity::Error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryVerdictStatus {
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryVerdict {
    pub status: UiStoryVerdictStatus,
    #[serde(default)]
    pub first_failing_stage: Option<UiStoryStageKind>,
}

impl UiStoryVerdict {
    pub fn passed() -> Self {
        Self {
            status: UiStoryVerdictStatus::Passed,
            first_failing_stage: None,
        }
    }

    pub fn failed(stage: UiStoryStageKind) -> Self {
        Self {
            status: UiStoryVerdictStatus::Failed,
            first_failing_stage: Some(stage),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiStoryRunReport {
    #[serde(default)]
    pub manifest: Option<UiStoryManifest>,
    pub story_id: UiStoryId,
    pub title: String,
    pub expected_verdict: UiStoryExpectedVerdict,
    pub stages: Vec<UiStoryStageReport>,
    pub diagnostics: Vec<UiStoryDiagnostic>,
    pub verdict: UiStoryVerdict,
}

impl UiStoryRunReport {
    pub fn from_manifest_and_stages(
        manifest: UiStoryManifest,
        stages: Vec<UiStoryStageReport>,
    ) -> Self {
        let story_id = manifest.story_id.clone();
        let title = manifest.title.clone();
        let expected_verdict = manifest.expected.verdict.clone();
        Self::from_parts(Some(manifest), story_id, title, expected_verdict, stages)
    }

    pub fn unknown_story(story_id: UiStoryId, diagnostic: UiStoryDiagnostic) -> Self {
        Self::from_parts(
            None,
            story_id,
            "Unknown story".to_owned(),
            UiStoryExpectedVerdict::Pass,
            vec![UiStoryStageReport::failed(
                UiStoryStageKind::Manifest,
                vec![diagnostic],
            )],
        )
    }

    pub fn passed(&self) -> bool {
        self.verdict.status == UiStoryVerdictStatus::Passed
    }

    pub fn stage(&self, stage: UiStoryStageKind) -> Option<&UiStoryStageReport> {
        self.stages.iter().find(|report| report.stage == stage)
    }

    fn from_parts(
        manifest: Option<UiStoryManifest>,
        story_id: UiStoryId,
        title: String,
        expected_verdict: UiStoryExpectedVerdict,
        stages: Vec<UiStoryStageReport>,
    ) -> Self {
        let mut diagnostics = stages
            .iter()
            .flat_map(|stage| stage.diagnostics.iter().cloned())
            .collect::<Vec<_>>();
        let first_failing_stage = stages
            .iter()
            .find(|stage| stage.blocks_verdict())
            .map(|stage| stage.stage);
        let verdict = match (expected_verdict.clone(), first_failing_stage) {
            (UiStoryExpectedVerdict::Pass, Some(stage)) => UiStoryVerdict::failed(stage),
            (UiStoryExpectedVerdict::Pass, None) => UiStoryVerdict::passed(),
            (UiStoryExpectedVerdict::Fail, Some(_stage)) => UiStoryVerdict::passed(),
            (UiStoryExpectedVerdict::Fail, None) => {
                diagnostics.push(UiStoryDiagnostic::error(
                    DIAGNOSTIC_EXPECTED_FAILURE_PASSED,
                    "story was expected to fail but no failing stage was reported",
                    UiStoryStageKind::Verdict,
                ));
                UiStoryVerdict::failed(UiStoryStageKind::Verdict)
            }
        };

        Self {
            manifest,
            story_id,
            title,
            expected_verdict,
            stages,
            diagnostics,
            verdict,
        }
    }
}
