//! CLI-oriented formatting for UI story run reports.

use serde::{Deserialize, Serialize};

use crate::report::{
    UiStoryDiagnosticSeverity, UiStoryRunReport, UiStoryStageKind, UiStoryStageStatus,
    UiStoryVerdictStatus,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryCliReport {
    pub stories: Vec<UiStoryCliStorySummary>,
}

impl UiStoryCliReport {
    pub fn from_reports<'a>(reports: impl IntoIterator<Item = &'a UiStoryRunReport>) -> Self {
        Self {
            stories: reports
                .into_iter()
                .map(UiStoryCliStorySummary::from_report)
                .collect(),
        }
    }

    pub fn passed(&self) -> bool {
        self.stories.iter().all(|story| story.passed)
    }

    pub fn render_text(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("ui stories: {}", self.stories.len()));
        for story in &self.stories {
            lines.push(format!(
                "- {} [{}] expected={} first_failure={}",
                story.story_id,
                if story.passed { "passed" } else { "failed" },
                story.expected_verdict,
                story.first_failing_stage.as_deref().unwrap_or("none")
            ));
            for stage in &story.stages {
                lines.push(format!("  stage {}: {}", stage.stage, stage.status));
            }
            for diagnostic in &story.diagnostics {
                lines.push(format!(
                    "  diagnostic {} {}: {}",
                    diagnostic.severity, diagnostic.code, diagnostic.message
                ));
            }
        }
        lines.join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryCliStorySummary {
    pub story_id: String,
    pub title: String,
    pub expected_verdict: String,
    pub passed: bool,
    pub first_failing_stage: Option<String>,
    pub stages: Vec<UiStoryCliStageSummary>,
    pub diagnostics: Vec<UiStoryCliDiagnosticSummary>,
}

impl UiStoryCliStorySummary {
    pub fn from_report(report: &UiStoryRunReport) -> Self {
        Self {
            story_id: report.story_id.as_str().to_owned(),
            title: report.title.clone(),
            expected_verdict: format!("{:?}", report.expected_verdict),
            passed: report.verdict.status == UiStoryVerdictStatus::Passed,
            first_failing_stage: report
                .verdict
                .first_failing_stage
                .map(stage_label)
                .map(str::to_owned),
            stages: report
                .stages
                .iter()
                .map(|stage| UiStoryCliStageSummary {
                    stage: stage_label(stage.stage).to_owned(),
                    status: stage_status_label(stage.status).to_owned(),
                })
                .collect(),
            diagnostics: report
                .diagnostics
                .iter()
                .map(|diagnostic| UiStoryCliDiagnosticSummary {
                    stage: stage_label(diagnostic.stage).to_owned(),
                    severity: diagnostic_severity_label(diagnostic.severity).to_owned(),
                    code: diagnostic.code.clone(),
                    message: diagnostic.message.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryCliStageSummary {
    pub stage: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryCliDiagnosticSummary {
    pub stage: String,
    pub severity: String,
    pub code: String,
    pub message: String,
}

pub fn stage_label(stage: UiStoryStageKind) -> &'static str {
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

pub fn stage_status_label(status: UiStoryStageStatus) -> &'static str {
    match status {
        UiStoryStageStatus::Passed => "passed",
        UiStoryStageStatus::Failed => "failed",
        UiStoryStageStatus::Skipped => "skipped",
        UiStoryStageStatus::MissingProof => "missing_proof",
    }
}

pub fn diagnostic_severity_label(severity: UiStoryDiagnosticSeverity) -> &'static str {
    match severity {
        UiStoryDiagnosticSeverity::Info => "info",
        UiStoryDiagnosticSeverity::Warning => "warning",
        UiStoryDiagnosticSeverity::Error => "error",
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gallery::checked_in_gallery_registry,
        report::{UiStoryStageKind, UiStoryStageReport},
        runner::UiStoryRunner,
    };

    use super::*;

    #[test]
    fn cli_report_renders_stage_and_verdict_summary() {
        let registry = checked_in_gallery_registry();
        let runner = UiStoryRunner::new(&registry);
        let report = runner.run_story_with_stage_reports(
            &registry.run_request("ui.gallery.button.basic"),
            [
                UiStoryStageReport::passed(UiStoryStageKind::SourceLoad),
                UiStoryStageReport::passed(UiStoryStageKind::SourceParse),
                UiStoryStageReport::passed(UiStoryStageKind::ProgramFormation),
                UiStoryStageReport::passed(UiStoryStageKind::Compiler),
                UiStoryStageReport::passed(UiStoryStageKind::RuntimeView),
                UiStoryStageReport::passed(UiStoryStageKind::RenderPrimitives),
                UiStoryStageReport::passed(UiStoryStageKind::RenderData),
                UiStoryStageReport::passed(UiStoryStageKind::StaticMount),
                UiStoryStageReport::passed(UiStoryStageKind::PreviewFrame),
            ],
        );

        let cli_report = UiStoryCliReport::from_reports([&report]);
        let rendered = cli_report.render_text();

        assert!(cli_report.passed());
        assert!(rendered.contains("ui.gallery.button.basic [passed]"));
        assert!(rendered.contains("stage runtime_view: passed"));
        assert!(rendered.contains("stage static_mount: passed"));
    }
}
