use crate::{DiagnosticCode, DiagnosticDomain, DiagnosticMessage, DiagnosticSubject};

/// Lightweight related diagnostic reference.
///
/// Related diagnostics intentionally do not recursively contain full
/// diagnostics in v1. This keeps reports deterministic, cycle-safe, and easy to
/// serialize later.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticRelated {
    code: DiagnosticCode,
    domain: DiagnosticDomain,
    subject: Option<DiagnosticSubject>,
    message: Option<DiagnosticMessage>,
}

impl DiagnosticRelated {
    pub fn new(code: DiagnosticCode, domain: DiagnosticDomain) -> Self {
        Self {
            code,
            domain,
            subject: None,
            message: None,
        }
    }

    pub fn with_subject(mut self, subject: DiagnosticSubject) -> Self {
        self.subject = Some(subject);
        self
    }

    pub fn with_message(mut self, message: DiagnosticMessage) -> Self {
        self.message = Some(message);
        self
    }

    pub fn code(&self) -> &DiagnosticCode {
        &self.code
    }

    pub fn domain(&self) -> &DiagnosticDomain {
        &self.domain
    }

    pub fn subject(&self) -> Option<&DiagnosticSubject> {
        self.subject.as_ref()
    }

    pub fn message(&self) -> Option<&DiagnosticMessage> {
        self.message.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticRelated;
    use crate::{
        DiagnosticCode, DiagnosticDomain, DiagnosticMessage, DiagnosticSubject,
        DiagnosticSubjectId, DiagnosticSubjectKind,
    };

    #[test]
    fn related_preserves_code_domain_subject() {
        let related = DiagnosticRelated::new(
            DiagnosticCode::from_static("ui_surface.mount.unknown_host").unwrap(),
            DiagnosticDomain::from_static("ui_surface").unwrap(),
        )
        .with_subject(
            DiagnosticSubject::new(DiagnosticSubjectKind::from_static("surface_host").unwrap())
                .with_id(DiagnosticSubjectId::from_static("main_dock").unwrap()),
        );

        assert_eq!(related.code().as_str(), "ui_surface.mount.unknown_host");
        assert_eq!(related.domain().as_str(), "ui_surface");
        assert_eq!(
            related.subject().unwrap().id().unwrap().as_str(),
            "main_dock"
        );
    }

    #[test]
    fn related_preserves_message() {
        let related = DiagnosticRelated::new(
            DiagnosticCode::from_static("asset.decode.unsupported_format").unwrap(),
            DiagnosticDomain::from_static("asset").unwrap(),
        )
        .with_message(DiagnosticMessage::from_static("Unsupported asset format."));

        assert_eq!(
            related.message().unwrap().as_str(),
            "Unsupported asset format."
        );
    }

    #[test]
    fn related_does_not_require_recursive_diagnostic() {
        let related = DiagnosticRelated::new(
            DiagnosticCode::from_static("scene.persistence.missing_parent").unwrap(),
            DiagnosticDomain::from_static("scene").unwrap(),
        );

        assert_eq!(related.code().as_str(), "scene.persistence.missing_parent");
        assert!(related.subject().is_none());
        assert!(related.message().is_none());
    }
}
