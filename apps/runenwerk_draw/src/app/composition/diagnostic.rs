//! Draw-owned composition diagnostics.

use std::collections::BTreeMap;

use ui_composition::{CompositionDefinitionId, MountedUnitId, PresentationTargetId, RegionId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DrawingCompositionDiagnosticCode {
    DefinitionInvalid,
    ExtensionSchemaUnsupported,
    ExtensionCoreMismatch,
    ExtensionCoverageMismatch,
    ExtensionRoleDuplicate,
    ExtensionContentProfileMismatch,
    ExtensionNonCanonical,
    LinkedBundleInvalid,
    ContentUnitUnknown,
    ContentUnavailable,
    ContentFallbackExhausted,
    ProjectionTargetInvalid,
    ProjectionRootMissing,
    ProjectionRegionMissing,
    ProjectionRegionCycle,
    ProjectionMountedUnitBoundsMissing,
}

impl DrawingCompositionDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DefinitionInvalid => "draw_composition.definition.invalid",
            Self::ExtensionSchemaUnsupported => "draw_composition.extension.schema_unsupported",
            Self::ExtensionCoreMismatch => "draw_composition.extension.core_mismatch",
            Self::ExtensionCoverageMismatch => "draw_composition.extension.coverage_mismatch",
            Self::ExtensionRoleDuplicate => "draw_composition.extension.role_duplicate",
            Self::ExtensionContentProfileMismatch => {
                "draw_composition.extension.content_profile_mismatch"
            }
            Self::ExtensionNonCanonical => "draw_composition.extension.non_canonical",
            Self::LinkedBundleInvalid => "draw_composition.extension.linked_bundle_invalid",
            Self::ContentUnitUnknown => "draw_composition.content.unit_unknown",
            Self::ContentUnavailable => "draw_composition.content.unavailable",
            Self::ContentFallbackExhausted => "draw_composition.content.fallback_exhausted",
            Self::ProjectionTargetInvalid => "draw_composition.projection.target_invalid",
            Self::ProjectionRootMissing => "draw_composition.projection.root_missing",
            Self::ProjectionRegionMissing => "draw_composition.projection.region_missing",
            Self::ProjectionRegionCycle => "draw_composition.projection.region_cycle",
            Self::ProjectionMountedUnitBoundsMissing => {
                "draw_composition.projection.mounted_unit_bounds_missing"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DrawingCompositionDiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DrawingCompositionDiagnosticStage {
    Definition,
    Extension,
    Content,
    Projection,
    Runtime,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DrawingCompositionDiagnosticSubject {
    Layout(CompositionDefinitionId),
    Target(PresentationTargetId),
    Region(RegionId),
    MountedUnit(MountedUnitId),
    General(String),
}

impl DrawingCompositionDiagnosticSubject {
    fn kind(&self) -> &'static str {
        match self {
            Self::Layout(_) => "layout",
            Self::Target(_) => "target",
            Self::Region(_) => "region",
            Self::MountedUnit(_) => "mounted_unit",
            Self::General(_) => "draw_composition",
        }
    }

    fn canonical_id(&self) -> String {
        match self {
            Self::Layout(value) => value.to_string(),
            Self::Target(value) => value.to_string(),
            Self::Region(value) => value.to_string(),
            Self::MountedUnit(value) => value.to_string(),
            Self::General(value) => value.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingCompositionDiagnosticRecord {
    code: DrawingCompositionDiagnosticCode,
    severity: DrawingCompositionDiagnosticSeverity,
    stage: DrawingCompositionDiagnosticStage,
    subject: DrawingCompositionDiagnosticSubject,
    message: String,
    context: BTreeMap<String, String>,
}

impl DrawingCompositionDiagnosticRecord {
    pub fn new(
        code: DrawingCompositionDiagnosticCode,
        severity: DrawingCompositionDiagnosticSeverity,
        stage: DrawingCompositionDiagnosticStage,
        subject: DrawingCompositionDiagnosticSubject,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            stage,
            subject,
            message: message.into(),
            context: BTreeMap::new(),
        }
    }

    pub fn error(
        code: DrawingCompositionDiagnosticCode,
        stage: DrawingCompositionDiagnosticStage,
        subject: DrawingCompositionDiagnosticSubject,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            code,
            DrawingCompositionDiagnosticSeverity::Error,
            stage,
            subject,
            message,
        )
    }

    pub fn warning(
        code: DrawingCompositionDiagnosticCode,
        stage: DrawingCompositionDiagnosticStage,
        subject: DrawingCompositionDiagnosticSubject,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            code,
            DrawingCompositionDiagnosticSeverity::Warning,
            stage,
            subject,
            message,
        )
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub const fn code(&self) -> DrawingCompositionDiagnosticCode {
        self.code
    }

    pub const fn severity(&self) -> DrawingCompositionDiagnosticSeverity {
        self.severity
    }

    pub const fn stage(&self) -> DrawingCompositionDiagnosticStage {
        self.stage
    }

    pub fn subject(&self) -> &DrawingCompositionDiagnosticSubject {
        &self.subject
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn context(&self) -> &BTreeMap<String, String> {
        &self.context
    }

    fn sort_key(
        &self,
    ) -> (
        DrawingCompositionDiagnosticStage,
        DrawingCompositionDiagnosticSeverity,
        &'static str,
        &'static str,
        String,
    ) {
        (
            self.stage,
            self.severity,
            self.code.as_str(),
            self.subject.kind(),
            self.subject.canonical_id(),
        )
    }
}

impl PartialOrd for DrawingCompositionDiagnosticRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DrawingCompositionDiagnosticRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingCompositionRejection {
    diagnostics: Vec<DrawingCompositionDiagnosticRecord>,
}

impl DrawingCompositionRejection {
    pub fn new(mut diagnostics: Vec<DrawingCompositionDiagnosticRecord>) -> Self {
        diagnostics.sort();
        diagnostics.dedup();
        Self { diagnostics }
    }

    pub fn single(diagnostic: DrawingCompositionDiagnosticRecord) -> Self {
        Self::new(vec![diagnostic])
    }

    pub fn diagnostics(&self) -> &[DrawingCompositionDiagnosticRecord] {
        &self.diagnostics
    }
}

impl std::fmt::Display for DrawingCompositionRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Draw composition rejected with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl std::error::Error for DrawingCompositionRejection {}
