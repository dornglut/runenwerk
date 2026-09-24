use std::collections::BTreeMap;

use ui_composition::CompositionDiagnosticSeverity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorCompositionDiagnosticCode {
    LegacyIdentityInvalid,
    LegacyTopologyUnsupported,
    LegacyPanelContentMissing,
    LegacyContentProfileUnsupported,
    SplitFractionInvalid,
    ExtensionCoverageMismatch,
    ExtensionIdentityInvalid,
    ExtensionSchemaUnsupported,
    ExtensionNonCanonical,
    ExtensionCoreMismatch,
    TransactionRejected,
    StalePreparedChange,
    IdentityExhausted,
    StaleProposal,
    DockTargetInvalid,
    SourceCompactionInvalid,
    StructuralEditInvalid,
    CoordinationPending,
    WindowPrimaryDetachDenied,
    WindowCreationFailed,
    WindowUnboundCloseDenied,
    WindowTargetBindingMissing,
    WindowCloseFallbackMissing,
    WindowDirtyQuitDenied,
    HistoryUnavailable,
    HistoryTargetCoordinationRequired,
    LayoutActivationFailed,
    TargetBindingMismatch,
    ContentMissing,
    ContentLoading,
    ContentSuspended,
    ContentDenied,
    ContentUnsupportedProfile,
    ContentCrashed,
}

impl EditorCompositionDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LegacyIdentityInvalid => "editor_composition.legacy.identity_invalid",
            Self::LegacyTopologyUnsupported => "editor_composition.legacy.topology_unsupported",
            Self::LegacyPanelContentMissing => "editor_composition.legacy.panel_content_missing",
            Self::LegacyContentProfileUnsupported => {
                "editor_composition.legacy.content_profile_unsupported"
            }
            Self::SplitFractionInvalid => "editor_composition.legacy.split_fraction_invalid",
            Self::ExtensionCoverageMismatch => "editor_composition.extension.coverage_mismatch",
            Self::ExtensionIdentityInvalid => "editor_composition.extension.identity_invalid",
            Self::ExtensionSchemaUnsupported => "editor_composition.extension.schema_unsupported",
            Self::ExtensionNonCanonical => "editor_composition.extension.non_canonical",
            Self::ExtensionCoreMismatch => "editor_composition.extension.core_mismatch",
            Self::TransactionRejected => "editor_composition.transaction.rejected",
            Self::StalePreparedChange => "editor_composition.transaction.stale_prepared_change",
            Self::IdentityExhausted => "editor_composition.identity.exhausted",
            Self::StaleProposal => "editor_composition.docking.stale_proposal",
            Self::DockTargetInvalid => "editor_composition.docking.target_invalid",
            Self::SourceCompactionInvalid => "editor_composition.docking.source_compaction_invalid",
            Self::StructuralEditInvalid => "editor_composition.transaction.structural_edit_invalid",
            Self::CoordinationPending => "editor_composition.coordination.pending",
            Self::WindowPrimaryDetachDenied => "editor_composition.window.primary_detach_denied",
            Self::WindowCreationFailed => "editor_composition.window.creation_failed",
            Self::WindowUnboundCloseDenied => "editor_composition.window.unbound_close_denied",
            Self::WindowTargetBindingMissing => "editor_composition.window.target_binding_missing",
            Self::WindowCloseFallbackMissing => "editor_composition.window.close_fallback_missing",
            Self::WindowDirtyQuitDenied => "editor_composition.window.dirty_quit_denied",
            Self::HistoryUnavailable => "editor_composition.history.unavailable",
            Self::HistoryTargetCoordinationRequired => {
                "editor_composition.history.target_coordination_required"
            }
            Self::LayoutActivationFailed => "editor_composition.layout.activation_failed",
            Self::TargetBindingMismatch => "editor_composition.target.binding_mismatch",
            Self::ContentMissing => "editor_composition.content.missing",
            Self::ContentLoading => "editor_composition.content.loading",
            Self::ContentSuspended => "editor_composition.content.suspended",
            Self::ContentDenied => "editor_composition.content.denied",
            Self::ContentUnsupportedProfile => "editor_composition.content.unsupported_profile",
            Self::ContentCrashed => "editor_composition.content.crashed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorCompositionDiagnosticStage {
    Import,
    Extension,
    Projection,
    Provider,
    Persistence,
    StaticGate,
    Transaction,
    Policy,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorCompositionDiagnosticSubject {
    Layout(u64),
    Region(u64),
    MountedUnit(u64),
    Target(u64),
    Transaction(u64),
    Legacy(&'static str, u64),
    Profile(String),
    General(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCompositionDiagnosticRecord {
    code: EditorCompositionDiagnosticCode,
    severity: CompositionDiagnosticSeverity,
    stage: EditorCompositionDiagnosticStage,
    subject: EditorCompositionDiagnosticSubject,
    message: String,
    context: BTreeMap<String, String>,
}

impl EditorCompositionDiagnosticRecord {
    pub fn error(
        code: EditorCompositionDiagnosticCode,
        stage: EditorCompositionDiagnosticStage,
        subject: EditorCompositionDiagnosticSubject,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: CompositionDiagnosticSeverity::Error,
            stage,
            subject,
            message: message.into(),
            context: BTreeMap::new(),
        }
    }

    pub fn warning(
        code: EditorCompositionDiagnosticCode,
        stage: EditorCompositionDiagnosticStage,
        subject: EditorCompositionDiagnosticSubject,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: CompositionDiagnosticSeverity::Warning,
            stage,
            subject,
            message: message.into(),
            context: BTreeMap::new(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub const fn code(&self) -> EditorCompositionDiagnosticCode {
        self.code
    }

    pub const fn severity(&self) -> CompositionDiagnosticSeverity {
        self.severity
    }

    pub const fn stage(&self) -> EditorCompositionDiagnosticStage {
        self.stage
    }

    pub fn subject(&self) -> &EditorCompositionDiagnosticSubject {
        &self.subject
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn context(&self) -> &BTreeMap<String, String> {
        &self.context
    }

    pub fn to_foundation_diagnostic(&self) -> diagnostics::Diagnostic {
        use diagnostics::{
            Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticMessage,
            DiagnosticMetadataEntry, DiagnosticMetadataKey, DiagnosticMetadataValue,
            DiagnosticSubject, DiagnosticSubjectId, DiagnosticSubjectKind, Severity,
        };
        let severity = match self.severity {
            CompositionDiagnosticSeverity::Info => Severity::Info,
            CompositionDiagnosticSeverity::Warning => Severity::Warning,
            CompositionDiagnosticSeverity::Error => Severity::Error,
            CompositionDiagnosticSeverity::Fatal => Severity::Fatal,
        };
        let mut subject = DiagnosticSubject::new(DiagnosticSubjectKind::from_static_unchecked(
            self.subject.kind(),
        ));
        if let Ok(id) = DiagnosticSubjectId::new(self.subject.canonical_id()) {
            subject = subject.with_id(id);
        }
        let mut diagnostic = Diagnostic::new(
            severity,
            DiagnosticCode::from_static_unchecked(self.code.as_str()),
            DiagnosticDomain::from_static_unchecked("editor_composition"),
            DiagnosticMessage::new(self.message.clone()),
        )
        .with_subject(subject);
        if let Ok(key) = DiagnosticMetadataKey::from_static("stage") {
            diagnostic.push_metadata(DiagnosticMetadataEntry::new(
                key,
                DiagnosticMetadataValue::string(format!("{:?}", self.stage).to_ascii_lowercase()),
            ));
        }
        for (key, value) in &self.context {
            if let Ok(key) = DiagnosticMetadataKey::new(key.clone()) {
                diagnostic.push_metadata(DiagnosticMetadataEntry::new(
                    key,
                    DiagnosticMetadataValue::string(value.clone()),
                ));
            }
        }
        diagnostic
    }
}

impl EditorCompositionDiagnosticSubject {
    const fn kind(&self) -> &'static str {
        match self {
            Self::Layout(_) => "composition_layout",
            Self::Region(_) => "composition_region",
            Self::MountedUnit(_) => "mounted_unit",
            Self::Target(_) => "presentation_target",
            Self::Transaction(_) => "composition_transaction",
            Self::Legacy(_, _) => "legacy_editor_layout",
            Self::Profile(_) => "editor_profile",
            Self::General(_) => "editor_composition",
        }
    }

    fn canonical_id(&self) -> String {
        match self {
            Self::Layout(raw) => format!("layout:{raw}"),
            Self::Region(raw) => format!("region:{raw}"),
            Self::MountedUnit(raw) => format!("mounted-unit:{raw}"),
            Self::Target(raw) => format!("target:{raw}"),
            Self::Transaction(raw) => format!("transaction:{raw}"),
            Self::Legacy(kind, raw) => format!("legacy-{kind}:{raw}"),
            Self::Profile(value) => format!("profile:{value}"),
            Self::General(value) => value.clone(),
        }
    }
}

impl PartialOrd for EditorCompositionDiagnosticRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EditorCompositionDiagnosticRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.stage,
            self.severity,
            self.code.as_str(),
            &self.subject,
            &self.message,
            &self.context,
        )
            .cmp(&(
                other.stage,
                other.severity,
                other.code.as_str(),
                &other.subject,
                &other.message,
                &other.context,
            ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCompositionRejection {
    diagnostics: Vec<EditorCompositionDiagnosticRecord>,
}

impl EditorCompositionRejection {
    pub fn new(mut diagnostics: Vec<EditorCompositionDiagnosticRecord>) -> Self {
        diagnostics.sort();
        diagnostics.dedup();
        Self { diagnostics }
    }

    pub fn single(diagnostic: EditorCompositionDiagnosticRecord) -> Self {
        Self::new(vec![diagnostic])
    }

    pub fn diagnostics(&self) -> &[EditorCompositionDiagnosticRecord] {
        &self.diagnostics
    }
}

impl std::fmt::Display for EditorCompositionRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "editor composition rejected with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl std::error::Error for EditorCompositionRejection {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_diagnostic_converts_to_foundation_contract_without_losing_identity() {
        let record = EditorCompositionDiagnosticRecord::error(
            EditorCompositionDiagnosticCode::ContentMissing,
            EditorCompositionDiagnosticStage::Provider,
            EditorCompositionDiagnosticSubject::MountedUnit(42),
            "Restore or remap the missing mounted content binding.",
        );

        let diagnostic = record.to_foundation_diagnostic();

        assert_eq!(
            diagnostic.code().as_str(),
            "editor_composition.content.missing"
        );
        assert_eq!(diagnostic.domain().as_str(), "editor_composition");
        assert_eq!(diagnostic.severity(), diagnostics::Severity::Error);
        assert_eq!(
            diagnostic
                .subject()
                .and_then(|subject| subject.id())
                .map(|id| id.as_str()),
            Some("mounted-unit:42")
        );
    }
}
