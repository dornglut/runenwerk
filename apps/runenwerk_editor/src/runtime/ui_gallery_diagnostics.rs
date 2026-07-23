use std::collections::BTreeSet;

use ui_runtime_view::{ButtonRuntimeViewDiagnosticSeverity, UiRuntimeViewDiagnosticSeverity};
use ui_story::{
    UiStoryDiagnostic, UiStoryDiagnosticSeverity, UiStoryDiagnosticSubject, UiStoryOutcomeV2,
    UiStoryWorkflowReportV2,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiGalleryDiagnostic {
    pub stage: UiGalleryStage,
    pub story_id: Option<String>,
    pub code: String,
    pub message: String,
    pub severity: UiGalleryDiagnosticSeverity,
    pub source_map_index: Option<u32>,
    pub blocks_gallery: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiGalleryStage {
    Story,
    WorkflowNode(String),
    RenderPrimitives,
    RenderData,
    StaticMount,
}

impl UiGalleryStage {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Story => "story",
            Self::WorkflowNode(node_id) => node_id.as_str(),
            Self::RenderPrimitives => "render_primitives",
            Self::RenderData => "render_data",
            Self::StaticMount => "static_mount",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiGalleryDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl UiGalleryDiagnosticSeverity {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UiGalleryDiagnosticKey {
    stage: String,
    story_id: Option<String>,
    code: String,
    message: String,
    severity: UiGalleryDiagnosticSeverity,
    source_map_index: Option<u32>,
    blocks_gallery: bool,
}

impl From<&UiGalleryDiagnostic> for UiGalleryDiagnosticKey {
    fn from(diagnostic: &UiGalleryDiagnostic) -> Self {
        Self {
            stage: diagnostic.stage.as_str().to_owned(),
            story_id: diagnostic.story_id.clone(),
            code: diagnostic.code.clone(),
            message: diagnostic.message.clone(),
            severity: diagnostic.severity,
            source_map_index: diagnostic.source_map_index,
            blocks_gallery: diagnostic.blocks_gallery,
        }
    }
}

pub(super) fn append_interactive_story_report_diagnostics(
    report: &UiStoryWorkflowReportV2,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
    blocks_gallery: bool,
) {
    if !story_report_diagnostics_surface_in_interactive_gallery(report) {
        return;
    }

    diagnostics.extend(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| UiGalleryDiagnostic {
                stage: gallery_stage_for_story_diagnostic(diagnostic),
                story_id: Some(report.story_id.as_str().to_owned()),
                code: diagnostic.code.as_str().to_owned(),
                message: diagnostic.message.clone(),
                severity: story_severity(diagnostic.severity),
                source_map_index: None,
                blocks_gallery,
            }),
    );
}

fn story_report_diagnostics_surface_in_interactive_gallery(
    report: &UiStoryWorkflowReportV2,
) -> bool {
    report.outcome() != UiStoryOutcomeV2::ExpectedFailureMatched
}

pub(super) fn dedupe_gallery_diagnostics(diagnostics: &mut Vec<UiGalleryDiagnostic>) {
    let mut seen = BTreeSet::new();
    diagnostics.retain(|diagnostic| seen.insert(UiGalleryDiagnosticKey::from(diagnostic)));
}

fn gallery_stage_for_story_diagnostic(diagnostic: &UiStoryDiagnostic) -> UiGalleryStage {
    match &diagnostic.subject {
        UiStoryDiagnosticSubject::WorkflowNode(node_id) => {
            UiGalleryStage::WorkflowNode(node_id.as_str().to_owned())
        }
        _ => UiGalleryStage::Story,
    }
}

pub(super) fn runtime_artifact_severity(
    severity: ui_artifacts::UiRuntimeArtifactDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Info => UiStoryDiagnosticSeverity::Info,
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Error => {
            UiStoryDiagnosticSeverity::Error
        }
    }
}

pub(super) fn runtime_view_severity(
    severity: UiRuntimeViewDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        UiRuntimeViewDiagnosticSeverity::Info => UiStoryDiagnosticSeverity::Info,
        UiRuntimeViewDiagnosticSeverity::Warning => UiStoryDiagnosticSeverity::Warning,
        UiRuntimeViewDiagnosticSeverity::Error => UiStoryDiagnosticSeverity::Error,
    }
}

pub(super) fn button_runtime_severity(
    severity: ButtonRuntimeViewDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ButtonRuntimeViewDiagnosticSeverity::Info => UiStoryDiagnosticSeverity::Info,
        ButtonRuntimeViewDiagnosticSeverity::Warning => UiStoryDiagnosticSeverity::Warning,
        ButtonRuntimeViewDiagnosticSeverity::Error => UiStoryDiagnosticSeverity::Error,
    }
}

fn story_severity(severity: UiStoryDiagnosticSeverity) -> UiGalleryDiagnosticSeverity {
    match severity {
        UiStoryDiagnosticSeverity::Info => UiGalleryDiagnosticSeverity::Info,
        UiStoryDiagnosticSeverity::Warning => UiGalleryDiagnosticSeverity::Warning,
        UiStoryDiagnosticSeverity::Error | UiStoryDiagnosticSeverity::Fatal => {
            UiGalleryDiagnosticSeverity::Error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_story::{UiStoryId, UiStoryWorkflowReportV2};

    #[test]
    fn gallery_diagnostic_projection_deduplicates_exact_duplicates() {
        let diagnostic = UiGalleryDiagnostic {
            stage: UiGalleryStage::WorkflowNode("runtime_view".to_owned()),
            story_id: Some("ui.gallery.button.selected".to_owned()),
            code: "ui.runtime_view.button.selected_binding_missing_host_value".to_owned(),
            message: "duplicate".to_owned(),
            severity: UiGalleryDiagnosticSeverity::Warning,
            source_map_index: None,
            blocks_gallery: false,
        };
        let mut diagnostics = vec![diagnostic.clone(), diagnostic];

        dedupe_gallery_diagnostics(&mut diagnostics);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn expected_failure_reports_do_not_surface_as_interactive_gallery_diagnostics() {
        let report = UiStoryWorkflowReportV2 {
            story_id: UiStoryId::new("ui.gallery.button.missing_source"),
            workflow_graph: None,
            node_reports: Vec::new(),
            diagnostics: Vec::new(),
            outcome: UiStoryOutcomeV2::ExpectedFailureMatched,
        };

        assert!(!story_report_diagnostics_surface_in_interactive_gallery(
            &report
        ));
    }
}
