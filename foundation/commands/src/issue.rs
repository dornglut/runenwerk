#[cfg(feature = "alloc")]
use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandIssueCode {
    EmptyContractId,
    InvalidContractId,
    ZeroContractVersion,
    InvalidSchemaRef,
    EmptyMetadataKey,
    DuplicateMetadataKey,
    InvalidDescriptor,
    InvalidProposal,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandIssueSubject {
    ContractId,
    ContractVersion,
    ContractRef,
    SchemaRef,
    Descriptor,
    Proposal,
    ProposalId,
    Metadata,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandIssue {
    code: CommandIssueCode,
    subject: CommandIssueSubject,
    message: String,
}

#[cfg(not(feature = "alloc"))]
pub type CommandIssue = CommandIssueCode;

#[cfg(feature = "alloc")]
impl CommandIssue {
    pub fn new(
        code: CommandIssueCode,
        subject: CommandIssueSubject,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            subject,
            message: message.into(),
        }
    }

    pub fn code(&self) -> CommandIssueCode {
        self.code
    }

    pub fn subject(&self) -> &CommandIssueSubject {
        &self.subject
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}
