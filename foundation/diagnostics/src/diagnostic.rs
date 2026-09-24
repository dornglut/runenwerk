use alloc::vec::Vec;
use core::fmt;

use crate::{
    DiagnosticCode, DiagnosticDomain, DiagnosticLocation, DiagnosticMessage, DiagnosticMetadata,
    DiagnosticMetadataEntry, DiagnosticNote, DiagnosticRelated, DiagnosticSubject, Severity,
};

/// Core diagnostic observation artifact.
///
/// A diagnostic describes an observed issue, warning, fatal condition, or
/// relevant fact. It does not execute commands, mutate state, ratify state, or
/// decide acceptance policy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    severity: Severity,
    code: DiagnosticCode,
    domain: DiagnosticDomain,
    message: DiagnosticMessage,
    subject: Option<DiagnosticSubject>,
    location: Option<DiagnosticLocation>,
    notes: Vec<DiagnosticNote>,
    metadata: DiagnosticMetadata,
    related: Vec<DiagnosticRelated>,
}

impl Diagnostic {
    /// Creates a diagnostic with required core fields.
    ///
    /// Required:
    ///
    /// - severity
    /// - stable code
    /// - owning domain
    /// - human-readable message
    pub fn new(
        severity: Severity,
        code: DiagnosticCode,
        domain: DiagnosticDomain,
        message: DiagnosticMessage,
    ) -> Self {
        Self {
            severity,
            code,
            domain,
            message,
            subject: None,
            location: None,
            notes: Vec::new(),
            metadata: DiagnosticMetadata::new(),
            related: Vec::new(),
        }
    }

    pub fn with_subject(mut self, subject: DiagnosticSubject) -> Self {
        self.subject = Some(subject);
        self
    }

    pub fn with_location(mut self, location: DiagnosticLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_note(mut self, note: DiagnosticNote) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_metadata(mut self, entry: DiagnosticMetadataEntry) -> Self {
        self.metadata.push(entry);
        self
    }

    pub fn with_related(mut self, related: DiagnosticRelated) -> Self {
        self.related.push(related);
        self
    }

    pub fn push_note(&mut self, note: DiagnosticNote) {
        self.notes.push(note);
    }

    pub fn push_metadata(&mut self, entry: DiagnosticMetadataEntry) {
        self.metadata.push(entry);
    }

    pub fn push_related(&mut self, related: DiagnosticRelated) {
        self.related.push(related);
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn code(&self) -> &DiagnosticCode {
        &self.code
    }

    pub fn domain(&self) -> &DiagnosticDomain {
        &self.domain
    }

    pub fn message(&self) -> &DiagnosticMessage {
        &self.message
    }

    pub fn subject(&self) -> Option<&DiagnosticSubject> {
        self.subject.as_ref()
    }

    pub fn location(&self) -> Option<&DiagnosticLocation> {
        self.location.as_ref()
    }

    pub fn notes(&self) -> &[DiagnosticNote] {
        &self.notes
    }

    pub fn metadata(&self) -> &DiagnosticMetadata {
        &self.metadata
    }

    pub fn related(&self) -> &[DiagnosticRelated] {
        &self.related
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "[{:?}] {}: {}",
            self.severity, self.code, self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Diagnostic;
    use crate::{
        DiagnosticCode, DiagnosticDomain, DiagnosticLocation, DiagnosticMessage,
        DiagnosticMetadataEntry, DiagnosticMetadataKey, DiagnosticMetadataValue, DiagnosticNote,
        DiagnosticRelated, DiagnosticSubject, DiagnosticSubjectId, DiagnosticSubjectKind, Severity,
    };

    fn sample_diagnostic() -> Diagnostic {
        Diagnostic::new(
            Severity::Error,
            DiagnosticCode::from_static("ui_surface.mount.unknown_host").unwrap(),
            DiagnosticDomain::from_static("ui_surface").unwrap(),
            DiagnosticMessage::from_static("Unknown surface host."),
        )
    }

    #[test]
    fn diagnostic_requires_core_fields() {
        let diagnostic = sample_diagnostic();

        assert_eq!(diagnostic.severity(), Severity::Error);
        assert_eq!(diagnostic.code().as_str(), "ui_surface.mount.unknown_host");
        assert_eq!(diagnostic.domain().as_str(), "ui_surface");
        assert_eq!(diagnostic.message().as_str(), "Unknown surface host.");
    }

    #[test]
    fn diagnostic_preserves_severity_code_domain_message() {
        let diagnostic = Diagnostic::new(
            Severity::Warning,
            DiagnosticCode::from_static("editor_shell.route.stale_projection_epoch").unwrap(),
            DiagnosticDomain::from_static("editor_shell").unwrap(),
            DiagnosticMessage::from_static("Route was built from a stale projection epoch."),
        );

        assert_eq!(diagnostic.severity(), Severity::Warning);
        assert_eq!(
            diagnostic.code().as_str(),
            "editor_shell.route.stale_projection_epoch"
        );
        assert_eq!(diagnostic.domain().as_str(), "editor_shell");
        assert_eq!(
            diagnostic.message().as_str(),
            "Route was built from a stale projection epoch."
        );
    }

    #[test]
    fn diagnostic_preserves_subject() {
        let diagnostic = sample_diagnostic().with_subject(
            DiagnosticSubject::new(DiagnosticSubjectKind::from_static("surface_host").unwrap())
                .with_id(DiagnosticSubjectId::from_static("main_dock").unwrap())
                .with_label(DiagnosticMessage::from_static("Main Dock")),
        );

        let subject = diagnostic.subject().unwrap();

        assert_eq!(subject.kind().as_str(), "surface_host");
        assert_eq!(subject.id().unwrap().as_str(), "main_dock");
        assert_eq!(subject.label().unwrap().as_str(), "Main Dock");
    }

    #[test]
    fn diagnostic_preserves_location() {
        let diagnostic = sample_diagnostic().with_location(
            DiagnosticLocation::logical_path_static("workspace.tool_surfaces[2]").unwrap(),
        );

        assert_eq!(
            diagnostic.location().unwrap().to_string(),
            "workspace.tool_surfaces[2]"
        );
    }

    #[test]
    fn diagnostic_preserves_notes() {
        let diagnostic = sample_diagnostic()
            .with_note(DiagnosticNote::from_static(
                "Register the host before mounting surfaces.",
            ))
            .with_note(DiagnosticNote::from_static(
                "The mount request was not applied.",
            ));

        assert_eq!(diagnostic.notes().len(), 2);
        assert_eq!(
            diagnostic.notes()[0].as_str(),
            "Register the host before mounting surfaces."
        );
        assert_eq!(
            diagnostic.notes()[1].as_str(),
            "The mount request was not applied."
        );
    }

    #[test]
    fn diagnostic_preserves_metadata() {
        let diagnostic = sample_diagnostic()
            .with_metadata(DiagnosticMetadataEntry::new(
                DiagnosticMetadataKey::from_static("expected").unwrap(),
                DiagnosticMetadataValue::string("registered surface host"),
            ))
            .with_metadata(DiagnosticMetadataEntry::new(
                DiagnosticMetadataKey::from_static("actual").unwrap(),
                DiagnosticMetadataValue::id("main_dock"),
            ));

        let entries = diagnostic.metadata().entries();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key().as_str(), "expected");
        assert_eq!(
            entries[0].value(),
            &DiagnosticMetadataValue::String("registered surface host".to_string())
        );
        assert_eq!(entries[1].key().as_str(), "actual");
        assert_eq!(
            entries[1].value(),
            &DiagnosticMetadataValue::Id("main_dock".to_string())
        );
    }

    #[test]
    fn diagnostic_preserves_related() {
        let diagnostic = sample_diagnostic().with_related(DiagnosticRelated::new(
            DiagnosticCode::from_static("ui_surface.mount.unknown_definition").unwrap(),
            DiagnosticDomain::from_static("ui_surface").unwrap(),
        ));

        assert_eq!(diagnostic.related().len(), 1);
        assert_eq!(
            diagnostic.related()[0].code().as_str(),
            "ui_surface.mount.unknown_definition"
        );
        assert_eq!(diagnostic.related()[0].domain().as_str(), "ui_surface");
    }

    #[test]
    fn diagnostic_debug_output_is_available() {
        let diagnostic = sample_diagnostic();

        let output = format!("{diagnostic:?}");

        assert!(output.contains("Diagnostic"));
        assert!(output.contains("Unknown surface host."));
    }

    #[test]
    fn diagnostic_display_output_is_not_identity() {
        let diagnostic = sample_diagnostic();

        let output = diagnostic.to_string();

        assert!(output.contains("ui_surface.mount.unknown_host"));
        assert!(output.contains("Unknown surface host."));
    }

    #[test]
    fn diagnostic_mutation_helpers_preserve_order() {
        let mut diagnostic = sample_diagnostic();

        diagnostic.push_note(DiagnosticNote::from_static("First note."));
        diagnostic.push_note(DiagnosticNote::from_static("Second note."));

        diagnostic.push_related(DiagnosticRelated::new(
            DiagnosticCode::from_static("ui_surface.intent.unsupported").unwrap(),
            DiagnosticDomain::from_static("ui_surface").unwrap(),
        ));

        assert_eq!(diagnostic.notes()[0].as_str(), "First note.");
        assert_eq!(diagnostic.notes()[1].as_str(), "Second note.");
        assert_eq!(
            diagnostic.related()[0].code().as_str(),
            "ui_surface.intent.unsupported"
        );
    }
}
