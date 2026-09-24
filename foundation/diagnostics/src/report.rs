use alloc::vec::Vec;

use crate::{Diagnostic, DiagnosticSink, Severity};

/// Ordered diagnostic report.
///
/// A report preserves diagnostic emission order and provides deterministic
/// aggregation helpers. It does not decide acceptance, rejection, rollback,
/// commit, visibility, or command execution policy.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticReport {
    diagnostics: Vec<Diagnostic>,
}

/// Count summary grouped by severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticSeverityCounts {
    info: usize,
    warning: usize,
    error: usize,
    fatal: usize,
}

impl DiagnosticReport {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn from_diagnostic(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: alloc::vec![diagnostic],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Diagnostic> {
        self.diagnostics.iter()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.push(diagnostic);
        self
    }

    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn merge(&mut self, other: DiagnosticReport) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn merged(mut self, other: DiagnosticReport) -> Self {
        self.merge(other);
        self
    }

    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity() == Severity::Warning)
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity() == Severity::Error)
    }

    pub fn has_fatal(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity() == Severity::Fatal)
    }

    pub fn highest_severity(&self) -> Option<Severity> {
        self.diagnostics.iter().map(Diagnostic::severity).max()
    }

    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity() == severity)
            .count()
    }

    pub fn severity_counts(&self) -> DiagnosticSeverityCounts {
        let mut counts = DiagnosticSeverityCounts::new();

        for diagnostic in &self.diagnostics {
            counts.increment(diagnostic.severity());
        }

        counts
    }
}

impl DiagnosticSink for DiagnosticReport {
    fn emit(&mut self, diagnostic: Diagnostic) {
        self.push(diagnostic);
    }
}

impl IntoIterator for DiagnosticReport {
    type Item = Diagnostic;
    type IntoIter = alloc::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a DiagnosticReport {
    type Item = &'a Diagnostic;
    type IntoIter = core::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl DiagnosticSeverityCounts {
    pub fn new() -> Self {
        Self {
            info: 0,
            warning: 0,
            error: 0,
            fatal: 0,
        }
    }

    pub fn info(self) -> usize {
        self.info
    }

    pub fn warning(self) -> usize {
        self.warning
    }

    pub fn error(self) -> usize {
        self.error
    }

    pub fn fatal(self) -> usize {
        self.fatal
    }

    pub fn get(self, severity: Severity) -> usize {
        match severity {
            Severity::Info => self.info,
            Severity::Warning => self.warning,
            Severity::Error => self.error,
            Severity::Fatal => self.fatal,
        }
    }

    fn increment(&mut self, severity: Severity) {
        match severity {
            Severity::Info => self.info += 1,
            Severity::Warning => self.warning += 1,
            Severity::Error => self.error += 1,
            Severity::Fatal => self.fatal += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticReport;
    use crate::{
        Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticMessage, DiagnosticSink, Severity,
    };

    fn diagnostic(severity: Severity, code: &'static str) -> Diagnostic {
        Diagnostic::new(
            severity,
            DiagnosticCode::from_static(code).unwrap(),
            DiagnosticDomain::from_static("ui_surface").unwrap(),
            DiagnosticMessage::from_static("Test diagnostic."),
        )
    }

    #[test]
    fn sink_collects_in_emission_order() {
        let mut report = DiagnosticReport::new();

        report.emit(diagnostic(
            Severity::Warning,
            "ui_surface.capability.missing",
        ));
        report.emit(diagnostic(Severity::Error, "ui_surface.mount.unknown_host"));

        assert_eq!(report.len(), 2);
        assert_eq!(
            report.diagnostics()[0].code().as_str(),
            "ui_surface.capability.missing"
        );
        assert_eq!(
            report.diagnostics()[1].code().as_str(),
            "ui_surface.mount.unknown_host"
        );
    }

    #[test]
    fn report_push_preserves_order() {
        let mut report = DiagnosticReport::new();

        report.push(diagnostic(Severity::Info, "ui_surface.intent.observed"));
        report.push(diagnostic(
            Severity::Warning,
            "ui_surface.intent.unsupported",
        ));

        assert_eq!(
            report.diagnostics()[0].code().as_str(),
            "ui_surface.intent.observed"
        );
        assert_eq!(
            report.diagnostics()[1].code().as_str(),
            "ui_surface.intent.unsupported"
        );
    }

    #[test]
    fn report_extend_preserves_order() {
        let mut report = DiagnosticReport::new();

        report.extend([
            diagnostic(Severity::Info, "ui_surface.mount.observed"),
            diagnostic(Severity::Error, "ui_surface.mount.unknown_definition"),
        ]);

        assert_eq!(report.len(), 2);
        assert_eq!(
            report.diagnostics()[0].code().as_str(),
            "ui_surface.mount.observed"
        );
        assert_eq!(
            report.diagnostics()[1].code().as_str(),
            "ui_surface.mount.unknown_definition"
        );
    }

    #[test]
    fn report_merge_preserves_left_to_right_order() {
        let mut left = DiagnosticReport::new();
        left.push(diagnostic(
            Severity::Warning,
            "ui_surface.capability.missing",
        ));

        let mut right = DiagnosticReport::new();
        right.push(diagnostic(
            Severity::Fatal,
            "ui_surface.mount.fatal_dispatch",
        ));

        left.merge(right);

        assert_eq!(left.len(), 2);
        assert_eq!(
            left.diagnostics()[0].code().as_str(),
            "ui_surface.capability.missing"
        );
        assert_eq!(
            left.diagnostics()[1].code().as_str(),
            "ui_surface.mount.fatal_dispatch"
        );
    }

    #[test]
    fn report_counts_by_severity() {
        let report = DiagnosticReport::new()
            .with_diagnostic(diagnostic(Severity::Info, "ui_surface.info.one"))
            .with_diagnostic(diagnostic(Severity::Warning, "ui_surface.warning.one"))
            .with_diagnostic(diagnostic(Severity::Warning, "ui_surface.warning.two"))
            .with_diagnostic(diagnostic(Severity::Error, "ui_surface.error.one"))
            .with_diagnostic(diagnostic(Severity::Fatal, "ui_surface.fatal.one"));

        assert_eq!(report.count_by_severity(Severity::Info), 1);
        assert_eq!(report.count_by_severity(Severity::Warning), 2);
        assert_eq!(report.count_by_severity(Severity::Error), 1);
        assert_eq!(report.count_by_severity(Severity::Fatal), 1);

        let counts = report.severity_counts();

        assert_eq!(counts.info(), 1);
        assert_eq!(counts.warning(), 2);
        assert_eq!(counts.error(), 1);
        assert_eq!(counts.fatal(), 1);
    }

    #[test]
    fn report_highest_severity_is_stable() {
        let report = DiagnosticReport::new()
            .with_diagnostic(diagnostic(Severity::Info, "ui_surface.info.one"))
            .with_diagnostic(diagnostic(Severity::Error, "ui_surface.error.one"))
            .with_diagnostic(diagnostic(Severity::Warning, "ui_surface.warning.one"));

        assert_eq!(report.highest_severity(), Some(Severity::Error));
    }

    #[test]
    fn empty_report_has_no_highest_severity() {
        let report = DiagnosticReport::new();

        assert_eq!(report.highest_severity(), None);
    }

    #[test]
    fn report_has_warning_error_fatal_helpers_work() {
        let report = DiagnosticReport::new()
            .with_diagnostic(diagnostic(Severity::Warning, "ui_surface.warning.one"))
            .with_diagnostic(diagnostic(Severity::Error, "ui_surface.error.one"))
            .with_diagnostic(diagnostic(Severity::Fatal, "ui_surface.fatal.one"));

        assert!(report.has_warnings());
        assert!(report.has_errors());
        assert!(report.has_fatal());
    }

    #[test]
    fn report_helpers_do_not_define_acceptance() {
        let report = DiagnosticReport::new()
            .with_diagnostic(diagnostic(Severity::Error, "ui_surface.error.one"));

        assert!(report.has_errors());
        assert_eq!(report.highest_severity(), Some(Severity::Error));

        // No API here says accepted/rejected. That policy belongs to ratifiers
        // and owning domains.
    }

    #[test]
    fn report_iterates_in_order() {
        let report = DiagnosticReport::new()
            .with_diagnostic(diagnostic(Severity::Info, "ui_surface.info.one"))
            .with_diagnostic(diagnostic(Severity::Info, "ui_surface.info.two"));

        let codes = report
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<alloc::vec::Vec<_>>();

        assert_eq!(
            codes,
            alloc::vec!["ui_surface.info.one", "ui_surface.info.two"]
        );
    }
}
