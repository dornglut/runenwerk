use ratification::{
    RatificationIssue, RatificationReport, RatificationSeverity, RatificationStatus, Ratifier,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestIssueCode {
    MissingName,
    DeprecatedName,
    FatalInvariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestSubject {
    Candidate,
    Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NameCandidate {
    name: String,
}

#[test]
fn accepted_report_has_accepted_status() {
    let report: RatificationReport<TestIssueCode, TestSubject> = RatificationReport::accepted();

    assert_eq!(report.status(), RatificationStatus::Accepted);
    assert!(report.is_accepted());
    assert!(report.is_clean());
    assert!(report.is_empty());
    assert_eq!(report.highest_severity(), None);
}

#[test]
fn accepted_with_warnings_contains_warning_and_no_blocking_errors() {
    let report = RatificationReport::from_issue(RatificationIssue::warning(
        TestIssueCode::DeprecatedName,
        TestSubject::Name,
        "name is deprecated but still accepted",
    ));

    assert_eq!(report.status(), RatificationStatus::AcceptedWithWarnings);
    assert!(report.is_accepted());
    assert!(report.has_warnings());
    assert!(!report.has_blocking_issues());
    assert_eq!(
        report.highest_severity(),
        Some(RatificationSeverity::Warning)
    );
}

#[test]
fn rejected_report_has_at_least_one_blocking_issue() {
    let report = RatificationReport::from_issue(RatificationIssue::error(
        TestIssueCode::MissingName,
        TestSubject::Name,
        "name is required",
    ));

    assert_eq!(report.status(), RatificationStatus::Rejected);
    assert!(!report.is_accepted());
    assert!(report.is_rejected());
    assert!(report.has_blocking_issues());
    assert_eq!(report.highest_severity(), Some(RatificationSeverity::Error));
}

#[test]
fn fatal_report_is_distinguishable_from_rejection() {
    let report = RatificationReport::from_issue(RatificationIssue::fatal(
        TestIssueCode::FatalInvariant,
        TestSubject::Candidate,
        "candidate could not be safely evaluated",
    ));

    assert_eq!(report.status(), RatificationStatus::Fatal);
    assert!(!report.is_accepted());
    assert!(!report.is_rejected());
    assert!(report.is_fatal());
    assert!(report.has_blocking_issues());
    assert_eq!(report.highest_severity(), Some(RatificationSeverity::Fatal));
}

#[test]
fn merge_preserves_highest_severity_and_status() {
    let mut report = RatificationReport::from_issue(RatificationIssue::warning(
        TestIssueCode::DeprecatedName,
        TestSubject::Name,
        "name is deprecated",
    ));

    report.merge(RatificationReport::from_issue(RatificationIssue::error(
        TestIssueCode::MissingName,
        TestSubject::Name,
        "name is required",
    )));

    assert_eq!(report.status(), RatificationStatus::Rejected);
    assert_eq!(report.len(), 2);
    assert_eq!(report.highest_severity(), Some(RatificationSeverity::Error));
}

#[test]
fn pushing_issue_recomputes_status() {
    let mut report: RatificationReport<TestIssueCode, TestSubject> = RatificationReport::accepted();

    report.push(RatificationIssue::warning(
        TestIssueCode::DeprecatedName,
        TestSubject::Name,
        "name is deprecated",
    ));

    assert_eq!(report.status(), RatificationStatus::AcceptedWithWarnings);

    report.push(RatificationIssue::fatal(
        TestIssueCode::FatalInvariant,
        TestSubject::Candidate,
        "fatal invariant violation",
    ));

    assert_eq!(report.status(), RatificationStatus::Fatal);
}

#[test]
fn ratifier_trait_validates_an_immutable_candidate() {
    struct NameRatifier;

    impl Ratifier<NameCandidate> for NameRatifier {
        type Code = TestIssueCode;
        type Subject = TestSubject;

        fn ratify(
            &self,
            candidate: &NameCandidate,
        ) -> RatificationReport<Self::Code, Self::Subject> {
            if candidate.name.trim().is_empty() {
                return RatificationReport::from_issue(RatificationIssue::error(
                    TestIssueCode::MissingName,
                    TestSubject::Name,
                    "name is required",
                ));
            }

            RatificationReport::accepted()
        }
    }

    let ratifier = NameRatifier;
    let candidate = NameCandidate {
        name: "Player".to_string(),
    };

    let report = ratifier.ratify(&candidate);

    assert_eq!(candidate.name, "Player");
    assert_eq!(report.status(), RatificationStatus::Accepted);
}

#[cfg(feature = "serde")]
#[test]
fn report_round_trips_with_serde_feature() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    enum SerdeIssueCode {
        MissingName,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    enum SerdeSubject {
        Name,
    }

    let report = RatificationReport::from_issue(RatificationIssue::error(
        SerdeIssueCode::MissingName,
        SerdeSubject::Name,
        "name is required",
    ));

    let json = serde_json::to_string(&report).expect("report should serialize");
    let decoded: RatificationReport<SerdeIssueCode, SerdeSubject> =
        serde_json::from_str(&json).expect("report should deserialize");

    assert_eq!(decoded, report);
}
