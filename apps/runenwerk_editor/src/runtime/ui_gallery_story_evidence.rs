//! Editor-owned UI Story V2 evidence vocabulary for the checked-in gallery.
//!
//! This module maps editor/gallery proof producers to the domain-owned workflow
//! nodes they prove. The editor still owns filesystem, parser, compiler, render,
//! static-mount, and preview-frame behavior; `ui_story` only receives evidence.

use ui_headless_render_data::UiHeadlessRenderDataReport;
use ui_render_primitives::UiRenderPrimitiveReport;
use ui_static_mount::UiStaticMountReport;
use ui_story::{
    NODE_COMPILER, NODE_PREVIEW_FRAME, NODE_PROGRAM_FORMATION, NODE_RENDER_DATA,
    NODE_RENDER_PRIMITIVES, NODE_RUNTIME_VIEW, NODE_SOURCE_LOAD, NODE_SOURCE_PARSE,
    NODE_STATIC_MOUNT, UiStoryDiagnostic, UiStoryDiagnosticOrigin, UiStoryDiagnosticSeverity,
    UiStoryDiagnosticSubject, UiStoryEvidence, UiStoryEvidenceProducerId, UiStoryWorkflowNodeId,
};

pub const SOURCE_LOAD: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_SOURCE_LOAD,
    "runenwerk_editor.ui_gallery.source_loader",
    "ui.gallery.source_load",
);
pub const SOURCE_PARSE: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_SOURCE_PARSE,
    "runenwerk_editor.ui_gallery.source_parser",
    "ui.gallery.source_parse",
);
pub const PROGRAM_FORMATION: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_PROGRAM_FORMATION,
    "runenwerk_editor.ui_gallery.program_formation",
    "ui.gallery.program_formation",
);
pub const COMPILER: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_COMPILER,
    "runenwerk_editor.ui_gallery.compiler",
    "ui.gallery.compiler",
);
pub const RUNTIME_VIEW: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_RUNTIME_VIEW,
    "runenwerk_editor.ui_gallery.runtime_view",
    "ui.gallery.runtime_view",
);
pub const RENDER_PRIMITIVES: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_RENDER_PRIMITIVES,
    "runenwerk_editor.ui_gallery.render_primitives",
    "ui.gallery.render_primitives",
);
pub const RENDER_DATA: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_RENDER_DATA,
    "runenwerk_editor.ui_gallery.render_data",
    "ui.gallery.render_data",
);
pub const STATIC_MOUNT: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_STATIC_MOUNT,
    "runenwerk_editor.ui_gallery.static_mount",
    "ui.gallery.static_mount",
);
pub const PREVIEW_FRAME: UiGalleryStoryEvidenceSpec = UiGalleryStoryEvidenceSpec::new(
    NODE_PREVIEW_FRAME,
    "runenwerk_editor.ui_gallery.preview_frame",
    "ui.gallery.preview_frame",
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiGalleryStoryEvidenceSpec {
    pub node_id: &'static str,
    pub producer_id: &'static str,
    pub evidence_key: &'static str,
}

impl UiGalleryStoryEvidenceSpec {
    pub const fn new(
        node_id: &'static str,
        producer_id: &'static str,
        evidence_key: &'static str,
    ) -> Self {
        Self {
            node_id,
            producer_id,
            evidence_key,
        }
    }

    pub fn passed(self) -> UiStoryEvidence {
        UiStoryEvidence::passed(self.node_id, self.producer_id, self.evidence_key)
    }

    pub fn failed(self, diagnostics: Vec<UiStoryDiagnostic>) -> UiStoryEvidence {
        UiStoryEvidence::failed(
            self.node_id,
            self.producer_id,
            self.evidence_key,
            diagnostics,
        )
    }

    pub fn result(self, passed: bool, diagnostics: Vec<UiStoryDiagnostic>) -> UiStoryEvidence {
        UiStoryEvidence::from_result(
            self.node_id,
            self.producer_id,
            self.evidence_key,
            passed,
            diagnostics,
        )
    }

    pub fn diagnostic(
        self,
        code: impl Into<String>,
        message: impl Into<String>,
        severity: UiStoryDiagnosticSeverity,
    ) -> UiStoryDiagnostic {
        UiStoryDiagnostic::new(
            code,
            severity,
            UiStoryDiagnosticOrigin::ExternalProducer(UiStoryEvidenceProducerId::new(
                self.producer_id,
            )),
            UiStoryDiagnosticSubject::WorkflowNode(UiStoryWorkflowNodeId::new(self.node_id)),
            message,
        )
        .with_context("producer_id", self.producer_id)
    }
}

pub fn render_primitive_report(report: &UiRenderPrimitiveReport) -> UiStoryEvidence {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            RENDER_PRIMITIVES.diagnostic(
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                render_primitive_severity(diagnostic.severity),
            )
        })
        .collect::<Vec<_>>();
    RENDER_PRIMITIVES.result(report.passed(), diagnostics)
}

pub fn render_data_report(report: &UiHeadlessRenderDataReport) -> UiStoryEvidence {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            RENDER_DATA.diagnostic(
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                render_data_severity(diagnostic.severity),
            )
        })
        .collect::<Vec<_>>();
    RENDER_DATA.result(report.passed(), diagnostics)
}

pub fn static_mount_report(report: &UiStaticMountReport) -> UiStoryEvidence {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            STATIC_MOUNT.diagnostic(
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                static_mount_severity(diagnostic.severity),
            )
        })
        .collect::<Vec<_>>();
    STATIC_MOUNT.result(report.passed(), diagnostics)
}

fn render_primitive_severity(
    severity: ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Info => {
            UiStoryDiagnosticSeverity::Info
        }
        ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Error => {
            UiStoryDiagnosticSeverity::Error
        }
    }
}

fn render_data_severity(
    severity: ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Info => {
            UiStoryDiagnosticSeverity::Info
        }
        ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Error => {
            UiStoryDiagnosticSeverity::Error
        }
    }
}

fn static_mount_severity(
    severity: ui_static_mount::UiStaticMountDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ui_static_mount::UiStaticMountDiagnosticSeverity::Info => UiStoryDiagnosticSeverity::Info,
        ui_static_mount::UiStaticMountDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_static_mount::UiStaticMountDiagnosticSeverity::Error => UiStoryDiagnosticSeverity::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_evidence_specs_preserve_node_producer_and_key_contracts() {
        let evidence = SOURCE_LOAD.passed();

        assert_eq!(evidence.workflow_node_id.as_str(), NODE_SOURCE_LOAD);
        assert_eq!(
            evidence.producer_id.as_str(),
            "runenwerk_editor.ui_gallery.source_loader"
        );
        assert_eq!(evidence.evidence_key.as_str(), "ui.gallery.source_load");
    }

    #[test]
    fn gallery_diagnostic_is_anchored_to_workflow_node_and_producer() {
        let diagnostic = PREVIEW_FRAME.diagnostic(
            "ui_gallery.story.preview_frame.missing",
            "static mount did not produce a mounted preview frame",
            UiStoryDiagnosticSeverity::Error,
        );

        assert_eq!(
            diagnostic.code.as_str(),
            "ui_gallery.story.preview_frame.missing"
        );
        assert_eq!(diagnostic.severity, UiStoryDiagnosticSeverity::Error);
        assert!(matches!(
            &diagnostic.subject,
            UiStoryDiagnosticSubject::WorkflowNode(node) if node.as_str() == NODE_PREVIEW_FRAME
        ));
        assert!(matches!(
            &diagnostic.origin,
            UiStoryDiagnosticOrigin::ExternalProducer(producer)
                if producer.as_str() == "runenwerk_editor.ui_gallery.preview_frame"
        ));
    }
}
