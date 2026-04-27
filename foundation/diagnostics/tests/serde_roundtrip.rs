#![cfg(feature = "serde")]

use diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticLocation, DiagnosticMessage,
    DiagnosticMetadataEntry, DiagnosticMetadataKey, DiagnosticMetadataValue, DiagnosticNote,
    DiagnosticRelated, DiagnosticReport, DiagnosticSubject, DiagnosticSubjectId,
    DiagnosticSubjectKind, DiagnosticTextPosition, DiagnosticTextRange, Severity,
};

fn sample_report() -> DiagnosticReport {
    let start = DiagnosticTextPosition::new(1, 1).unwrap();
    let end = DiagnosticTextPosition::new(1, 5).unwrap();
    let range = DiagnosticTextRange::new(start, end).unwrap();

    let diagnostic = Diagnostic::new(
        Severity::Error,
        DiagnosticCode::from_static("ui_surface.mount.unknown_host").unwrap(),
        DiagnosticDomain::from_static("ui_surface").unwrap(),
        DiagnosticMessage::from_static("Unknown surface host."),
    )
    .with_subject(
        DiagnosticSubject::new(DiagnosticSubjectKind::from_static("surface_host").unwrap())
            .with_id(DiagnosticSubjectId::from_static("main_dock").unwrap())
            .with_label(DiagnosticMessage::from_static("Main Dock")),
    )
    .with_location(
        DiagnosticLocation::file_path_static("assets/editor.workspace", Some(range)).unwrap(),
    )
    .with_note(DiagnosticNote::from_static(
        "Register the host before mounting surfaces.",
    ))
    .with_metadata(DiagnosticMetadataEntry::new(
        DiagnosticMetadataKey::from_static("expected").unwrap(),
        DiagnosticMetadataValue::string("registered surface host"),
    ))
    .with_related(DiagnosticRelated::new(
        DiagnosticCode::from_static("ui_surface.mount.unknown_definition").unwrap(),
        DiagnosticDomain::from_static("ui_surface").unwrap(),
    ));

    DiagnosticReport::from_diagnostic(diagnostic)
}

#[test]
fn serde_round_trip_preserves_code_domain_severity_message() {
    let report = sample_report();

    let json = serde_json::to_string(&report).unwrap();
    let round_trip: DiagnosticReport = serde_json::from_str(&json).unwrap();

    let diagnostic = &round_trip.diagnostics()[0];

    assert_eq!(diagnostic.severity(), Severity::Error);
    assert_eq!(diagnostic.code().as_str(), "ui_surface.mount.unknown_host");
    assert_eq!(diagnostic.domain().as_str(), "ui_surface");
    assert_eq!(diagnostic.message().as_str(), "Unknown surface host.");
}

#[test]
fn serde_round_trip_preserves_subject() {
    let report = sample_report();

    let json = serde_json::to_string(&report).unwrap();
    let round_trip: DiagnosticReport = serde_json::from_str(&json).unwrap();

    let subject = round_trip.diagnostics()[0].subject().unwrap();

    assert_eq!(subject.kind().as_str(), "surface_host");
    assert_eq!(subject.id().unwrap().as_str(), "main_dock");
    assert_eq!(subject.label().unwrap().as_str(), "Main Dock");
}

#[test]
fn serde_round_trip_preserves_location() {
    let report = sample_report();

    let json = serde_json::to_string(&report).unwrap();
    let round_trip: DiagnosticReport = serde_json::from_str(&json).unwrap();

    assert_eq!(
        round_trip.diagnostics()[0].location().unwrap().to_string(),
        "assets/editor.workspace:1:1..1:5"
    );
}

#[test]
fn serde_round_trip_preserves_metadata() {
    let report = sample_report();

    let json = serde_json::to_string(&report).unwrap();
    let round_trip: DiagnosticReport = serde_json::from_str(&json).unwrap();

    let metadata = round_trip.diagnostics()[0].metadata();

    assert_eq!(metadata.entries()[0].key().as_str(), "expected");
    assert_eq!(
        metadata.entries()[0].value(),
        &DiagnosticMetadataValue::String("registered surface host".to_string())
    );
}

#[test]
fn serde_round_trip_preserves_related() {
    let report = sample_report();

    let json = serde_json::to_string(&report).unwrap();
    let round_trip: DiagnosticReport = serde_json::from_str(&json).unwrap();

    let related = &round_trip.diagnostics()[0].related()[0];

    assert_eq!(
        related.code().as_str(),
        "ui_surface.mount.unknown_definition"
    );
    assert_eq!(related.domain().as_str(), "ui_surface");
}

#[test]
fn serde_round_trip_preserves_report_order() {
    let first = Diagnostic::new(
        Severity::Info,
        DiagnosticCode::from_static("ui_surface.info.first").unwrap(),
        DiagnosticDomain::from_static("ui_surface").unwrap(),
        DiagnosticMessage::from_static("First."),
    );

    let second = Diagnostic::new(
        Severity::Warning,
        DiagnosticCode::from_static("ui_surface.warning.second").unwrap(),
        DiagnosticDomain::from_static("ui_surface").unwrap(),
        DiagnosticMessage::from_static("Second."),
    );

    let report = DiagnosticReport::new()
        .with_diagnostic(first)
        .with_diagnostic(second);

    let json = serde_json::to_string(&report).unwrap();
    let round_trip: DiagnosticReport = serde_json::from_str(&json).unwrap();

    assert_eq!(
        round_trip.diagnostics()[0].code().as_str(),
        "ui_surface.info.first"
    );
    assert_eq!(
        round_trip.diagnostics()[1].code().as_str(),
        "ui_surface.warning.second"
    );
}

#[test]
fn serde_rejects_invalid_code() {
    let json = r#"
    {
        "diagnostics": [
            {
                "severity": "Error",
                "code": "not_a_valid_code",
                "domain": "ui_surface",
                "message": "Invalid.",
                "subject": null,
                "location": null,
                "notes": [],
                "metadata": { "entries": [] },
                "related": []
            }
        ]
    }
    "#;

    let result = serde_json::from_str::<DiagnosticReport>(json);

    assert!(result.is_err());
}

#[test]
fn serde_rejects_invalid_text_position() {
    let json = r#"{ "line": 0, "column": 1 }"#;

    let result = serde_json::from_str::<DiagnosticTextPosition>(json);

    assert!(result.is_err());
}
