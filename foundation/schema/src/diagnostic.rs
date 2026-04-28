//! Optional bridge from schema-definition issues into diagnostics.
//!
//! Diagnostics remain reporting projections. Constructor and well-formedness
//! errors remain typed control-flow errors, and no diagnostic produced here
//! decides domain acceptance, command execution, mutation, or ratification.

use alloc::format;

use diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticMessage, DiagnosticReport,
    DiagnosticSubject, DiagnosticSubjectId, DiagnosticSubjectKind, Severity,
};

use crate::{
    FOUNDATION_SCHEMA_DOMAIN, SchemaConstraintError, SchemaDescriptorError, SchemaFieldError,
    SchemaIdError, SchemaIssue, SchemaIssueCode, SchemaIssueSubject, SchemaMetadataError,
    SchemaPathError, SchemaShapeError, SchemaValueError, SchemaVersionError,
};

pub fn schema_issue_to_diagnostic(issue: &SchemaIssue) -> Diagnostic {
    Diagnostic::new(
        schema_issue_severity_to_diagnostic_severity(issue.severity_rank()),
        schema_issue_code_to_diagnostic_code(issue.code()),
        schema_diagnostic_domain(),
        DiagnosticMessage::new(issue.message()),
    )
    .with_subject(schema_issue_subject_to_diagnostic_subject(issue.subject()))
}

pub fn schema_issues_to_diagnostic_report<'a>(
    issues: impl IntoIterator<Item = &'a SchemaIssue>,
) -> DiagnosticReport {
    let mut report = DiagnosticReport::new();

    for issue in issues {
        report.push(schema_issue_to_diagnostic(issue));
    }

    report
}

pub fn schema_id_error_to_diagnostic(error: &SchemaIdError) -> Diagnostic {
    let (code, message) = match error {
        SchemaIdError::Empty => (
            DiagnosticCode::from_static_unchecked("foundation.schema.id.empty"),
            "Schema id must not be empty.",
        ),
        SchemaIdError::ContainsWhitespace => (
            DiagnosticCode::from_static_unchecked("foundation.schema.id.whitespace"),
            "Schema id must not contain whitespace.",
        ),
        SchemaIdError::InvalidCharacter => (
            DiagnosticCode::from_static_unchecked("foundation.schema.id.invalid_character"),
            "Schema id contains an unsupported character.",
        ),
    };

    diagnostic(code, SchemaIssueSubject::Id, message)
}

pub fn schema_version_error_to_diagnostic(error: &SchemaVersionError) -> Diagnostic {
    match error {
        SchemaVersionError::Zero => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.version.zero"),
            SchemaIssueSubject::Version,
            "Schema version must start at 1.",
        ),
    }
}

pub fn schema_path_error_to_diagnostic(error: &SchemaPathError) -> Diagnostic {
    match error {
        SchemaPathError::EmptySegment => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.path.empty_segment"),
            SchemaIssueSubject::Path,
            "Schema path segment must not be empty.",
        ),
    }
}

pub fn schema_value_error_to_diagnostic(error: &SchemaValueError) -> Diagnostic {
    match error {
        SchemaValueError::NonFiniteFloat => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.value.non_finite_float"),
            SchemaIssueSubject::Value,
            "Schema value float must be finite.",
        ),
        SchemaValueError::EmptyKey => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.value.key.empty"),
            SchemaIssueSubject::Value,
            "Schema value key must not be empty.",
        ),
        SchemaValueError::EmptyEnumSymbol => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.value.enum_symbol.empty"),
            SchemaIssueSubject::Value,
            "Schema enum symbol must not be empty.",
        ),
        SchemaValueError::EmptyOpaqueKind => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.value.opaque_kind.empty"),
            SchemaIssueSubject::Value,
            "Schema opaque value kind must not be empty.",
        ),
        SchemaValueError::DuplicateKey(key) => diagnostic_owned(
            DiagnosticCode::from_static_unchecked("foundation.schema.value.key.duplicate"),
            SchemaIssueSubject::Value,
            format!("Schema value key '{key}' is duplicated."),
        ),
    }
}

pub fn schema_field_error_to_diagnostic(error: &SchemaFieldError) -> Diagnostic {
    match error {
        SchemaFieldError::EmptyName => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.field.name.empty"),
            SchemaIssueSubject::Field,
            "Schema field name must not be empty.",
        ),
        SchemaFieldError::Metadata(error) => schema_metadata_error_to_diagnostic(error),
    }
}

pub fn schema_constraint_error_to_diagnostic(error: &SchemaConstraintError) -> Diagnostic {
    match error {
        SchemaConstraintError::NonFiniteNumber => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.constraint.number.non_finite"),
            SchemaIssueSubject::Constraint,
            "Schema numeric constraint value must be finite.",
        ),
        SchemaConstraintError::MinGreaterThanMax => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.constraint.range.invalid"),
            SchemaIssueSubject::Constraint,
            "Schema numeric range minimum must not exceed maximum.",
        ),
        SchemaConstraintError::EmptyPatternName => diagnostic(
            DiagnosticCode::from_static_unchecked(
                "foundation.schema.constraint.pattern_name.empty",
            ),
            SchemaIssueSubject::Constraint,
            "Schema string pattern hint name must not be empty.",
        ),
        SchemaConstraintError::EmptyEnumOptions => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.constraint.enum.empty"),
            SchemaIssueSubject::Constraint,
            "Schema enum options must not be empty.",
        ),
        SchemaConstraintError::EmptyEnumOption => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.constraint.enum_option.empty"),
            SchemaIssueSubject::Constraint,
            "Schema enum option must not be empty.",
        ),
        SchemaConstraintError::DuplicateEnumOption(option) => diagnostic_owned(
            DiagnosticCode::from_static_unchecked(
                "foundation.schema.constraint.enum_option.duplicate",
            ),
            SchemaIssueSubject::Constraint,
            format!("Schema enum option '{option}' is duplicated."),
        ),
        SchemaConstraintError::EmptyUnitLabel => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.constraint.unit_label.empty"),
            SchemaIssueSubject::Constraint,
            "Schema display unit label must not be empty.",
        ),
    }
}

pub fn schema_descriptor_error_to_diagnostic(error: &SchemaDescriptorError) -> Diagnostic {
    match error {
        SchemaDescriptorError::Metadata(error) => schema_metadata_error_to_diagnostic(error),
    }
}

pub fn schema_metadata_error_to_diagnostic(error: &SchemaMetadataError) -> Diagnostic {
    match error {
        SchemaMetadataError::EmptyKey => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.metadata.key.empty"),
            SchemaIssueSubject::Metadata,
            "Schema metadata key must not be empty.",
        ),
        SchemaMetadataError::DuplicateKey(key) => diagnostic_owned(
            DiagnosticCode::from_static_unchecked("foundation.schema.metadata.key.duplicate"),
            SchemaIssueSubject::Metadata,
            format!("Schema metadata key '{key}' is duplicated."),
        ),
    }
}

pub fn schema_shape_error_to_diagnostic(error: &SchemaShapeError) -> Diagnostic {
    match error {
        SchemaShapeError::EmptyEnumOptions => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.shape.enum.empty"),
            SchemaIssueSubject::Schema,
            "Schema enum shape options must not be empty.",
        ),
        SchemaShapeError::EmptyEnumOption => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.shape.enum_option.empty"),
            SchemaIssueSubject::Schema,
            "Schema enum shape option must not be empty.",
        ),
        SchemaShapeError::DuplicateEnumOption(option) => diagnostic_owned(
            DiagnosticCode::from_static_unchecked("foundation.schema.shape.enum_option.duplicate"),
            SchemaIssueSubject::Schema,
            format!("Schema enum shape option '{option}' is duplicated."),
        ),
        SchemaShapeError::DuplicateFieldName(field) => diagnostic_owned(
            DiagnosticCode::from_static_unchecked("foundation.schema.shape.field.duplicate"),
            SchemaIssueSubject::Field,
            format!("Schema object field '{field}' is duplicated."),
        ),
        SchemaShapeError::EmptyOpaqueKind => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.schema.shape.opaque_kind.empty"),
            SchemaIssueSubject::Schema,
            "Schema opaque shape kind must not be empty.",
        ),
    }
}

fn schema_issue_code_to_diagnostic_code(code: SchemaIssueCode) -> DiagnosticCode {
    match code {
        SchemaIssueCode::InvalidId => {
            DiagnosticCode::from_static_unchecked("foundation.schema.id.invalid")
        }
        SchemaIssueCode::InvalidVersion => {
            DiagnosticCode::from_static_unchecked("foundation.schema.version.invalid")
        }
        SchemaIssueCode::InvalidPath => {
            DiagnosticCode::from_static_unchecked("foundation.schema.path.invalid")
        }
        SchemaIssueCode::InvalidValue => {
            DiagnosticCode::from_static_unchecked("foundation.schema.value.invalid")
        }
        SchemaIssueCode::InvalidField => {
            DiagnosticCode::from_static_unchecked("foundation.schema.field.invalid")
        }
        SchemaIssueCode::InvalidConstraint => {
            DiagnosticCode::from_static_unchecked("foundation.schema.constraint.invalid")
        }
        SchemaIssueCode::InvalidDescriptor => {
            DiagnosticCode::from_static_unchecked("foundation.schema.descriptor.invalid")
        }
        SchemaIssueCode::InvalidMetadata => {
            DiagnosticCode::from_static_unchecked("foundation.schema.metadata.invalid")
        }
        SchemaIssueCode::DuplicateKey => {
            DiagnosticCode::from_static_unchecked("foundation.schema.key.duplicate")
        }
        SchemaIssueCode::DuplicateField => {
            DiagnosticCode::from_static_unchecked("foundation.schema.field.duplicate")
        }
    }
}

fn schema_issue_subject_to_diagnostic_subject(subject: &SchemaIssueSubject) -> DiagnosticSubject {
    let (kind, id) = match subject {
        SchemaIssueSubject::Schema => ("schema", "schema"),
        SchemaIssueSubject::Id => ("schema_id", "id"),
        SchemaIssueSubject::Version => ("schema_version", "version"),
        SchemaIssueSubject::Path => ("schema_path", "path"),
        SchemaIssueSubject::Value => ("schema_value", "value"),
        SchemaIssueSubject::Field => ("schema_field", "field"),
        SchemaIssueSubject::Constraint => ("schema_constraint", "constraint"),
        SchemaIssueSubject::Descriptor => ("schema_descriptor", "descriptor"),
        SchemaIssueSubject::Metadata => ("schema_metadata", "metadata"),
    };

    DiagnosticSubject::new(DiagnosticSubjectKind::from_static_unchecked(kind))
        .with_id(DiagnosticSubjectId::from_static_unchecked(id))
}

fn schema_issue_severity_to_diagnostic_severity(severity_rank: u8) -> Severity {
    match severity_rank {
        0 => Severity::Info,
        1 => Severity::Warning,
        2 => Severity::Error,
        _ => Severity::Fatal,
    }
}

fn schema_diagnostic_domain() -> DiagnosticDomain {
    DiagnosticDomain::from_static_unchecked(FOUNDATION_SCHEMA_DOMAIN)
}

fn diagnostic(
    code: DiagnosticCode,
    subject: SchemaIssueSubject,
    message: &'static str,
) -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        code,
        schema_diagnostic_domain(),
        DiagnosticMessage::from_static(message),
    )
    .with_subject(schema_issue_subject_to_diagnostic_subject(&subject))
}

fn diagnostic_owned(
    code: DiagnosticCode,
    subject: SchemaIssueSubject,
    message: impl Into<alloc::string::String>,
) -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        code,
        schema_diagnostic_domain(),
        DiagnosticMessage::new(message),
    )
    .with_subject(schema_issue_subject_to_diagnostic_subject(&subject))
}

#[cfg(test)]
mod tests {
    use diagnostics::Severity;

    use super::{schema_issue_to_diagnostic, schema_issues_to_diagnostic_report};
    use crate::{SchemaIssue, SchemaIssueCode, SchemaIssueSubject};

    #[test]
    fn schema_issue_maps_to_expected_diagnostic_code() {
        let issue = SchemaIssue::new(
            SchemaIssueCode::InvalidId,
            SchemaIssueSubject::Id,
            "schema id is invalid",
            2,
        );

        let diagnostic = schema_issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.domain().as_str(), "foundation.schema");
        assert_eq!(diagnostic.code().as_str(), "foundation.schema.id.invalid");
        assert_eq!(diagnostic.message().as_str(), "schema id is invalid");
        assert_eq!(diagnostic.severity(), Severity::Error);
    }

    #[test]
    fn schema_issue_maps_to_useful_diagnostic_subject() {
        let issue = SchemaIssue::new(
            SchemaIssueCode::InvalidConstraint,
            SchemaIssueSubject::Constraint,
            "constraint is invalid",
            1,
        );

        let diagnostic = schema_issue_to_diagnostic(&issue);
        let subject = diagnostic.subject().expect("subject should be projected");

        assert_eq!(subject.kind().as_str(), "schema_constraint");
        assert_eq!(
            subject.id().expect("subject id should exist").as_str(),
            "constraint"
        );
        assert_eq!(diagnostic.severity(), Severity::Warning);
    }

    #[test]
    fn multiple_schema_issues_preserve_order() {
        let issues = [
            SchemaIssue::new(
                SchemaIssueCode::InvalidId,
                SchemaIssueSubject::Id,
                "id is invalid",
                2,
            ),
            SchemaIssue::new(
                SchemaIssueCode::DuplicateField,
                SchemaIssueSubject::Field,
                "field is duplicated",
                2,
            ),
            SchemaIssue::new(
                SchemaIssueCode::InvalidMetadata,
                SchemaIssueSubject::Metadata,
                "metadata is invalid",
                1,
            ),
        ];

        let diagnostics = schema_issues_to_diagnostic_report(&issues);

        assert_eq!(diagnostics.len(), 3);
        assert_eq!(
            diagnostics.diagnostics()[0].code().as_str(),
            "foundation.schema.id.invalid"
        );
        assert_eq!(
            diagnostics.diagnostics()[1].code().as_str(),
            "foundation.schema.field.duplicate"
        );
        assert_eq!(
            diagnostics.diagnostics()[2].code().as_str(),
            "foundation.schema.metadata.invalid"
        );
    }

    #[test]
    fn diagnostics_bridge_does_not_define_acceptance_or_ratification_semantics() {
        let issues = [SchemaIssue::new(
            SchemaIssueCode::InvalidDescriptor,
            SchemaIssueSubject::Descriptor,
            "descriptor is invalid",
            3,
        )];

        let diagnostics = schema_issues_to_diagnostic_report(&issues);

        assert!(diagnostics.has_fatal());
        assert_eq!(diagnostics.highest_severity(), Some(Severity::Fatal));
    }
}
