//! Optional bridge from ratification reports into diagnostics.
//!
//! Diagnostics remain report artifacts. They do not decide whether a candidate
//! is accepted, rejected, committed, rolled back, shared, or reconciled.
//!
//! Domain crates keep ownership of concrete issue codes and subjects. This
//! bridge only defines how callers can map those domain-owned values into the
//! shared diagnostics vocabulary.

use diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticMessage, DiagnosticReport,
    DiagnosticSubject, Severity,
};

use crate::{RatificationIssue, RatificationReport, RatificationSeverity};

/// Maps domain-owned ratification issue identity into diagnostic identity.
///
/// This trait is deliberately caller/domain implemented. Foundation
/// ratification must not own a global code enum or know the meaning of concrete
/// domain issue families.
pub trait RatificationDiagnosticMapper<Code, Subject> {
    fn diagnostic_domain(&self) -> DiagnosticDomain;

    fn diagnostic_code(&self, code: &Code) -> DiagnosticCode;

    fn diagnostic_subject(&self, subject: &Subject) -> Option<DiagnosticSubject>;
}

/// Converts ratification severity into diagnostic severity.
///
/// The meaning stays observational: diagnostics describe issue seriousness,
/// while ratification status still decides acceptance.
pub const fn ratification_severity_to_diagnostic_severity(
    severity: RatificationSeverity,
) -> Severity {
    match severity {
        RatificationSeverity::Info => Severity::Info,
        RatificationSeverity::Warning => Severity::Warning,
        RatificationSeverity::Error => Severity::Error,
        RatificationSeverity::Fatal => Severity::Fatal,
    }
}

/// Converts one ratification issue into one diagnostic.
pub fn ratification_issue_to_diagnostic<Code, Subject, Mapper>(
    issue: &RatificationIssue<Code, Subject>,
    mapper: &Mapper,
) -> Diagnostic
where
    Mapper: RatificationDiagnosticMapper<Code, Subject>,
{
    let diagnostic = Diagnostic::new(
        ratification_severity_to_diagnostic_severity(issue.severity()),
        mapper.diagnostic_code(issue.code()),
        mapper.diagnostic_domain(),
        DiagnosticMessage::new(issue.message()),
    );

    match mapper.diagnostic_subject(issue.subject()) {
        Some(subject) => diagnostic.with_subject(subject),
        None => diagnostic,
    }
}

/// Converts a full ratification report into an ordered diagnostic report.
///
/// Emission order is preserved. No acceptance policy is inferred from the
/// produced diagnostics.
pub fn ratification_report_to_diagnostic_report<Code, Subject, Mapper>(
    report: &RatificationReport<Code, Subject>,
    mapper: &Mapper,
) -> DiagnosticReport
where
    Mapper: RatificationDiagnosticMapper<Code, Subject>,
{
    let mut diagnostics = DiagnosticReport::new();

    for issue in report.issues() {
        diagnostics.push(ratification_issue_to_diagnostic(issue, mapper));
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use diagnostics::{
        DiagnosticCode, DiagnosticDomain, DiagnosticSubject, DiagnosticSubjectId,
        DiagnosticSubjectKind, Severity,
    };

    use super::{
        RatificationDiagnosticMapper, ratification_issue_to_diagnostic,
        ratification_report_to_diagnostic_report, ratification_severity_to_diagnostic_severity,
    };
    use crate::{RatificationIssue, RatificationReport, RatificationSeverity};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestIssueCode {
        DeprecatedName,
        MissingName,
        FatalInvariant,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestSubject {
        Candidate,
        Name,
    }

    struct TestMapper;

    impl RatificationDiagnosticMapper<TestIssueCode, TestSubject> for TestMapper {
        fn diagnostic_domain(&self) -> DiagnosticDomain {
            DiagnosticDomain::from_static_unchecked("test_ratification")
        }

        fn diagnostic_code(&self, code: &TestIssueCode) -> DiagnosticCode {
            match code {
                TestIssueCode::DeprecatedName => {
                    DiagnosticCode::from_static_unchecked("test_ratification.name.deprecated")
                }
                TestIssueCode::MissingName => {
                    DiagnosticCode::from_static_unchecked("test_ratification.name.missing")
                }
                TestIssueCode::FatalInvariant => {
                    DiagnosticCode::from_static_unchecked("test_ratification.invariant.fatal")
                }
            }
        }

        fn diagnostic_subject(&self, subject: &TestSubject) -> Option<DiagnosticSubject> {
            let (kind, id) = match subject {
                TestSubject::Candidate => ("candidate", "candidate"),
                TestSubject::Name => ("candidate_field", "name"),
            };

            Some(
                DiagnosticSubject::new(DiagnosticSubjectKind::from_static_unchecked(kind))
                    .with_id(DiagnosticSubjectId::from_static_unchecked(id)),
            )
        }
    }

    #[test]
    fn severity_mapping_preserves_ratification_severity() {
        assert_eq!(
            ratification_severity_to_diagnostic_severity(RatificationSeverity::Info),
            Severity::Info
        );
        assert_eq!(
            ratification_severity_to_diagnostic_severity(RatificationSeverity::Warning),
            Severity::Warning
        );
        assert_eq!(
            ratification_severity_to_diagnostic_severity(RatificationSeverity::Error),
            Severity::Error
        );
        assert_eq!(
            ratification_severity_to_diagnostic_severity(RatificationSeverity::Fatal),
            Severity::Fatal
        );
    }

    #[test]
    fn issue_conversion_preserves_code_domain_subject_message_and_severity() {
        let issue = RatificationIssue::error(
            TestIssueCode::MissingName,
            TestSubject::Name,
            "name is required",
        );

        let diagnostic = ratification_issue_to_diagnostic(&issue, &TestMapper);

        assert_eq!(diagnostic.severity(), Severity::Error);
        assert_eq!(diagnostic.domain().as_str(), "test_ratification");
        assert_eq!(diagnostic.code().as_str(), "test_ratification.name.missing");
        assert_eq!(diagnostic.message().as_str(), "name is required");

        let subject = diagnostic.subject().expect("subject should be mapped");
        assert_eq!(subject.kind().as_str(), "candidate_field");
        assert_eq!(
            subject.id().expect("subject id should exist").as_str(),
            "name"
        );
    }

    #[test]
    fn report_conversion_preserves_issue_order() {
        let report = RatificationReport::from_issues([
            RatificationIssue::warning(
                TestIssueCode::DeprecatedName,
                TestSubject::Name,
                "name is deprecated",
            ),
            RatificationIssue::error(
                TestIssueCode::MissingName,
                TestSubject::Name,
                "name is required",
            ),
            RatificationIssue::fatal(
                TestIssueCode::FatalInvariant,
                TestSubject::Candidate,
                "candidate could not be safely evaluated",
            ),
        ]);

        let diagnostics = ratification_report_to_diagnostic_report(&report, &TestMapper);

        assert_eq!(diagnostics.len(), 3);
        assert_eq!(
            diagnostics.diagnostics()[0].code().as_str(),
            "test_ratification.name.deprecated"
        );
        assert_eq!(
            diagnostics.diagnostics()[1].code().as_str(),
            "test_ratification.name.missing"
        );
        assert_eq!(
            diagnostics.diagnostics()[2].code().as_str(),
            "test_ratification.invariant.fatal"
        );
    }

    #[test]
    fn accepted_report_converts_to_empty_diagnostic_report() {
        let report: RatificationReport<TestIssueCode, TestSubject> = RatificationReport::accepted();

        let diagnostics = ratification_report_to_diagnostic_report(&report, &TestMapper);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn diagnostic_report_does_not_replace_ratification_status() {
        let report = RatificationReport::from_issue(RatificationIssue::error(
            TestIssueCode::MissingName,
            TestSubject::Name,
            "name is required",
        ));

        let diagnostics = ratification_report_to_diagnostic_report(&report, &TestMapper);

        assert!(report.is_rejected());
        assert!(diagnostics.has_errors());
        assert!(!diagnostics.has_fatal());
    }
}
