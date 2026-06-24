use ui_runtime_view::{ButtonRuntimeViewDiagnosticSeverity, UiRuntimeViewDiagnosticSeverity};
use ui_story::{
    UiStoryDiagnostic, UiStoryDiagnosticSeverity, UiStoryDiagnosticSubject, UiStoryWorkflowReportV2,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub(super) fn append_story_report_diagnostics(
    report: &UiStoryWorkflowReportV2,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
    blocks_gallery: bool,
) {
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
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Error => UiStoryDiagnosticSeverity::Error,
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

pub(super) fn button_gallery_severity(
    severity: ButtonRuntimeViewDiagnosticSeverity,
) -> UiGalleryDiagnosticSeverity {
    match severity {
        ButtonRuntimeViewDiagnosticSeverity::Info => UiGalleryDiagnosticSeverity::Info,
        ButtonRuntimeViewDiagnosticSeverity::Warning => UiGalleryDiagnosticSeverity::Warning,
        ButtonRuntimeViewDiagnosticSeverity::Error => UiGalleryDiagnosticSeverity::Error,
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
