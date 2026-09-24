//! Optional bridge from command vocabulary issues into diagnostics.
//!
//! Diagnostics remain reporting projections. Constructor and well-formedness
//! errors remain typed control-flow errors, and no diagnostic produced here
//! decides command acceptance, routing, execution, mutation, or ratification.

use alloc::format;

use diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticMessage, DiagnosticReport,
    DiagnosticSubject, DiagnosticSubjectId, DiagnosticSubjectKind, Severity,
};

use crate::{
    CommandContractIdError, CommandContractVersionError, CommandDescriptorError, CommandIssue,
    CommandIssueCode, CommandIssueSubject, CommandMetadataError, CommandProposalError,
    CommandProposalIdError, CommandSchemaRefError, FOUNDATION_COMMANDS_DOMAIN,
};

pub fn command_issue_to_diagnostic(issue: &CommandIssue) -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        command_issue_code_to_diagnostic_code(issue.code()),
        command_diagnostic_domain(),
        DiagnosticMessage::new(issue.message()),
    )
    .with_subject(command_issue_subject_to_diagnostic_subject(issue.subject()))
}

pub fn command_issues_to_diagnostic_report<'a>(
    issues: impl IntoIterator<Item = &'a CommandIssue>,
) -> DiagnosticReport {
    let mut report = DiagnosticReport::new();

    for issue in issues {
        report.push(command_issue_to_diagnostic(issue));
    }

    report
}

pub fn command_contract_id_error_to_diagnostic(error: &CommandContractIdError) -> Diagnostic {
    let (code, message) = match error {
        CommandContractIdError::Empty => (
            DiagnosticCode::from_static_unchecked("foundation.commands.contract_id.empty"),
            "Command contract id must not be empty.",
        ),
        CommandContractIdError::ContainsWhitespace => (
            DiagnosticCode::from_static_unchecked("foundation.commands.contract_id.whitespace"),
            "Command contract id must not contain whitespace.",
        ),
        CommandContractIdError::InvalidCharacter => (
            DiagnosticCode::from_static_unchecked(
                "foundation.commands.contract_id.invalid_character",
            ),
            "Command contract id contains an unsupported character.",
        ),
    };

    diagnostic(code, CommandIssueSubject::ContractId, message)
}

pub fn command_contract_version_error_to_diagnostic(
    error: &CommandContractVersionError,
) -> Diagnostic {
    match error {
        CommandContractVersionError::Zero => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.commands.contract_version.zero"),
            CommandIssueSubject::ContractVersion,
            "Command contract version must start at 1.",
        ),
    }
}

pub fn command_schema_ref_error_to_diagnostic(error: &CommandSchemaRefError) -> Diagnostic {
    match error {
        CommandSchemaRefError::InvalidVersion => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.commands.schema_ref.invalid_version"),
            CommandIssueSubject::SchemaRef,
            "Command schema reference must use a valid non-zero schema version.",
        ),
    }
}

pub fn command_metadata_error_to_diagnostic(error: &CommandMetadataError) -> Diagnostic {
    match error {
        CommandMetadataError::EmptyKey => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.commands.metadata.key.empty"),
            CommandIssueSubject::Metadata,
            "Command metadata key must not be empty.",
        ),
        CommandMetadataError::DuplicateKey(key) => diagnostic_owned(
            DiagnosticCode::from_static_unchecked("foundation.commands.metadata.key.duplicate"),
            CommandIssueSubject::Metadata,
            format!("Command metadata key '{key}' is duplicated."),
        ),
    }
}

pub fn command_descriptor_error_to_diagnostic(error: &CommandDescriptorError) -> Diagnostic {
    match error {
        CommandDescriptorError::Metadata(error) => command_metadata_error_to_diagnostic(error),
    }
}

pub fn command_proposal_id_error_to_diagnostic(error: &CommandProposalIdError) -> Diagnostic {
    match error {
        CommandProposalIdError::Empty => diagnostic(
            DiagnosticCode::from_static_unchecked("foundation.commands.proposal_id.empty"),
            CommandIssueSubject::ProposalId,
            "Command proposal id must not be empty.",
        ),
    }
}

pub fn command_proposal_error_to_diagnostic(error: &CommandProposalError) -> Diagnostic {
    match error {
        CommandProposalError::Metadata(error) => command_metadata_error_to_diagnostic(error),
    }
}

fn command_issue_code_to_diagnostic_code(code: CommandIssueCode) -> DiagnosticCode {
    match code {
        CommandIssueCode::EmptyContractId => {
            DiagnosticCode::from_static_unchecked("foundation.commands.contract_id.empty")
        }
        CommandIssueCode::InvalidContractId => {
            DiagnosticCode::from_static_unchecked("foundation.commands.contract_id.invalid")
        }
        CommandIssueCode::ZeroContractVersion => {
            DiagnosticCode::from_static_unchecked("foundation.commands.contract_version.zero")
        }
        CommandIssueCode::InvalidSchemaRef => {
            DiagnosticCode::from_static_unchecked("foundation.commands.schema_ref.invalid")
        }
        CommandIssueCode::EmptyMetadataKey => {
            DiagnosticCode::from_static_unchecked("foundation.commands.metadata.key.empty")
        }
        CommandIssueCode::DuplicateMetadataKey => {
            DiagnosticCode::from_static_unchecked("foundation.commands.metadata.key.duplicate")
        }
        CommandIssueCode::InvalidDescriptor => {
            DiagnosticCode::from_static_unchecked("foundation.commands.descriptor.invalid")
        }
        CommandIssueCode::InvalidProposal => {
            DiagnosticCode::from_static_unchecked("foundation.commands.proposal.invalid")
        }
    }
}

fn command_issue_subject_to_diagnostic_subject(subject: &CommandIssueSubject) -> DiagnosticSubject {
    let (kind, id) = match subject {
        CommandIssueSubject::ContractId => ("command_contract_id", "contract_id"),
        CommandIssueSubject::ContractVersion => ("command_contract_version", "contract_version"),
        CommandIssueSubject::ContractRef => ("command_contract_ref", "contract_ref"),
        CommandIssueSubject::SchemaRef => ("command_schema_ref", "schema_ref"),
        CommandIssueSubject::Descriptor => ("command_descriptor", "descriptor"),
        CommandIssueSubject::Proposal => ("command_proposal", "proposal"),
        CommandIssueSubject::ProposalId => ("command_proposal_id", "proposal_id"),
        CommandIssueSubject::Metadata => ("command_metadata", "metadata"),
    };

    DiagnosticSubject::new(DiagnosticSubjectKind::from_static_unchecked(kind))
        .with_id(DiagnosticSubjectId::from_static_unchecked(id))
}

fn command_diagnostic_domain() -> DiagnosticDomain {
    DiagnosticDomain::from_static_unchecked(FOUNDATION_COMMANDS_DOMAIN)
}

fn diagnostic(
    code: DiagnosticCode,
    subject: CommandIssueSubject,
    message: &'static str,
) -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        code,
        command_diagnostic_domain(),
        DiagnosticMessage::from_static(message),
    )
    .with_subject(command_issue_subject_to_diagnostic_subject(&subject))
}

fn diagnostic_owned(
    code: DiagnosticCode,
    subject: CommandIssueSubject,
    message: impl Into<alloc::string::String>,
) -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        code,
        command_diagnostic_domain(),
        DiagnosticMessage::new(message),
    )
    .with_subject(command_issue_subject_to_diagnostic_subject(&subject))
}

#[cfg(test)]
mod tests {
    use diagnostics::Severity;

    use super::{
        command_issue_to_diagnostic, command_issues_to_diagnostic_report,
        command_metadata_error_to_diagnostic,
    };
    use crate::{CommandIssue, CommandIssueCode, CommandIssueSubject, CommandMetadataError};

    #[test]
    fn command_issue_maps_to_expected_diagnostic_code() {
        let issue = CommandIssue::new(
            CommandIssueCode::InvalidContractId,
            CommandIssueSubject::ContractId,
            "command contract id is invalid",
        );

        let diagnostic = command_issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.domain().as_str(), "foundation.commands");
        assert_eq!(
            diagnostic.code().as_str(),
            "foundation.commands.contract_id.invalid"
        );
        assert_eq!(
            diagnostic.message().as_str(),
            "command contract id is invalid"
        );
        assert_eq!(diagnostic.severity(), Severity::Error);
    }

    #[test]
    fn command_issue_maps_to_useful_diagnostic_subject() {
        let issue = CommandIssue::new(
            CommandIssueCode::InvalidSchemaRef,
            CommandIssueSubject::SchemaRef,
            "schema ref is invalid",
        );

        let diagnostic = command_issue_to_diagnostic(&issue);
        let subject = diagnostic.subject().expect("subject should be projected");

        assert_eq!(subject.kind().as_str(), "command_schema_ref");
        assert_eq!(
            subject.id().expect("subject id should exist").as_str(),
            "schema_ref"
        );
    }

    #[test]
    fn multiple_command_issues_preserve_order() {
        let issues = [
            CommandIssue::new(
                CommandIssueCode::InvalidContractId,
                CommandIssueSubject::ContractId,
                "id is invalid",
            ),
            CommandIssue::new(
                CommandIssueCode::DuplicateMetadataKey,
                CommandIssueSubject::Metadata,
                "metadata is duplicated",
            ),
            CommandIssue::new(
                CommandIssueCode::InvalidProposal,
                CommandIssueSubject::Proposal,
                "proposal is invalid",
            ),
        ];

        let diagnostics = command_issues_to_diagnostic_report(&issues);

        assert_eq!(diagnostics.len(), 3);
        assert_eq!(
            diagnostics.diagnostics()[0].code().as_str(),
            "foundation.commands.contract_id.invalid"
        );
        assert_eq!(
            diagnostics.diagnostics()[1].code().as_str(),
            "foundation.commands.metadata.key.duplicate"
        );
        assert_eq!(
            diagnostics.diagnostics()[2].code().as_str(),
            "foundation.commands.proposal.invalid"
        );
    }

    #[test]
    fn metadata_error_maps_to_diagnostic_code() {
        let diagnostic =
            command_metadata_error_to_diagnostic(&CommandMetadataError::DuplicateKey("key".into()));

        assert_eq!(
            diagnostic.code().as_str(),
            "foundation.commands.metadata.key.duplicate"
        );
        assert_eq!(diagnostic.severity(), Severity::Error);
    }

    #[test]
    fn diagnostics_bridge_does_not_define_acceptance_or_execution_semantics() {
        let issues = [CommandIssue::new(
            CommandIssueCode::InvalidDescriptor,
            CommandIssueSubject::Descriptor,
            "descriptor is invalid",
        )];

        let diagnostics = command_issues_to_diagnostic_report(&issues);

        assert!(diagnostics.has_errors());
        assert_eq!(diagnostics.highest_severity(), Some(Severity::Error));
    }
}
