//! File: domain/editor/editor_scene/src/sdf_authoring/ratification.rs
//! Purpose: Semantic ratification for authored SDF operation documents.

use crate::{SceneTransform, SdfBooleanIntent, SdfOperationDocument, SdfOperationEntryId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfOperationIssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfOperationIssueCode {
    EmptyDocument,
    EmptyLayerStableName,
    DuplicateLayerStableName,
    EmptyOperationName,
    NonFiniteTransform,
    UnsupportedBooleanIntent,
    MissingSmoothRadius,
    InvalidSmoothRadius,
    GraphStructuralError,
    MissingGraphOutput,
    MissingGraphNodeSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdfOperationIssueSubject {
    Document,
    Layer(u64),
    Operation(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfOperationIssue {
    pub code: SdfOperationIssueCode,
    pub severity: SdfOperationIssueSeverity,
    pub subject: SdfOperationIssueSubject,
    pub message: String,
}

impl SdfOperationIssue {
    pub fn error(
        code: SdfOperationIssueCode,
        subject: SdfOperationIssueSubject,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: SdfOperationIssueSeverity::Error,
            subject,
            message: message.into(),
        }
    }

    pub fn warning(
        code: SdfOperationIssueCode,
        subject: SdfOperationIssueSubject,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: SdfOperationIssueSeverity::Warning,
            subject,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SdfOperationRatificationReport {
    pub issues: Vec<SdfOperationIssue>,
}

impl SdfOperationRatificationReport {
    pub fn has_blocking_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == SdfOperationIssueSeverity::Error)
    }
}

pub struct SdfOperationRatifier;

impl SdfOperationRatifier {
    pub fn ratify(&self, document: &SdfOperationDocument) -> SdfOperationRatificationReport {
        let mut report = SdfOperationRatificationReport::default();
        if document.layers().is_empty() {
            report.issues.push(SdfOperationIssue::warning(
                SdfOperationIssueCode::EmptyDocument,
                SdfOperationIssueSubject::Document,
                "SDF operation document has no layers",
            ));
        }
        for duplicate in document.duplicate_layer_names() {
            report.issues.push(SdfOperationIssue::error(
                SdfOperationIssueCode::DuplicateLayerStableName,
                SdfOperationIssueSubject::Document,
                format!("duplicate SDF layer stable name: {duplicate}"),
            ));
        }
        for layer in document.layers() {
            if layer.metadata.stable_name.trim().is_empty() {
                report.issues.push(SdfOperationIssue::error(
                    SdfOperationIssueCode::EmptyLayerStableName,
                    SdfOperationIssueSubject::Layer(layer.id.raw()),
                    "SDF layer stable name must not be empty",
                ));
            }
            for operation in &layer.operations {
                if operation.display_name.trim().is_empty() {
                    report.issues.push(SdfOperationIssue::error(
                        SdfOperationIssueCode::EmptyOperationName,
                        SdfOperationIssueSubject::Operation(operation.id.raw()),
                        "SDF operation display name must not be empty",
                    ));
                }
                if !transform_is_finite(operation.primitive.transform) {
                    report.issues.push(SdfOperationIssue::error(
                        SdfOperationIssueCode::NonFiniteTransform,
                        SdfOperationIssueSubject::Operation(operation.id.raw()),
                        "SDF operation transform must contain finite values",
                    ));
                }
                if operation.enabled && layer.metadata.enabled {
                    if unsupported_boolean_intent(operation.primitive.boolean) {
                        report.issues.push(unsupported_boolean_issue(operation.id));
                    }
                    if operation.primitive.boolean.requires_smooth_radius() {
                        match operation.primitive.smooth_radius_meters {
                            Some(radius) if radius.is_finite() && radius > 0.0 => {}
                            Some(_) => report.issues.push(SdfOperationIssue::error(
                                SdfOperationIssueCode::InvalidSmoothRadius,
                                SdfOperationIssueSubject::Operation(operation.id.raw()),
                                "smooth SDF boolean radius must be finite and positive",
                            )),
                            None => report.issues.push(SdfOperationIssue::error(
                                SdfOperationIssueCode::MissingSmoothRadius,
                                SdfOperationIssueSubject::Operation(operation.id.raw()),
                                "smooth SDF boolean radius must be authored explicitly",
                            )),
                        }
                    }
                }
            }
        }
        report
    }
}

pub fn ratify_sdf_operation_document(
    document: &SdfOperationDocument,
) -> SdfOperationRatificationReport {
    SdfOperationRatifier.ratify(document)
}

pub fn unsupported_boolean_intent(intent: SdfBooleanIntent) -> bool {
    match intent {
        SdfBooleanIntent::Add
        | SdfBooleanIntent::Subtract
        | SdfBooleanIntent::Intersect
        | SdfBooleanIntent::SmoothAdd
        | SdfBooleanIntent::SmoothSubtract
        | SdfBooleanIntent::SmoothIntersect => false,
    }
}

pub fn unsupported_boolean_issue(operation_id: SdfOperationEntryId) -> SdfOperationIssue {
    SdfOperationIssue::error(
        SdfOperationIssueCode::UnsupportedBooleanIntent,
        SdfOperationIssueSubject::Operation(operation_id.raw()),
        "SDF operation boolean intent is authored but cannot lower to world_ops yet",
    )
}

fn transform_is_finite(transform: SceneTransform) -> bool {
    [
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
        transform.rotation.w,
        transform.scale.x,
        transform.scale.y,
        transform.scale.z,
    ]
    .into_iter()
    .all(f32::is_finite)
}
