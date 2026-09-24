#[cfg(feature = "alloc")]
use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SchemaIssueCode {
    InvalidId,
    InvalidVersion,
    InvalidPath,
    InvalidValue,
    InvalidField,
    InvalidConstraint,
    InvalidDescriptor,
    InvalidMetadata,
    DuplicateKey,
    DuplicateField,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SchemaIssueSubject {
    Schema,
    Id,
    Version,
    Path,
    Value,
    Field,
    Constraint,
    Descriptor,
    Metadata,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaIssue {
    code: SchemaIssueCode,
    subject: SchemaIssueSubject,
    message: String,
    severity_rank: u8,
}

#[cfg(not(feature = "alloc"))]
pub type SchemaIssue = SchemaIssueCode;

#[cfg(feature = "alloc")]
impl SchemaIssue {
    pub fn new(
        code: SchemaIssueCode,
        subject: SchemaIssueSubject,
        message: impl Into<String>,
        severity_rank: u8,
    ) -> Self {
        Self {
            code,
            subject,
            message: message.into(),
            severity_rank,
        }
    }

    pub fn code(&self) -> SchemaIssueCode {
        self.code
    }

    pub fn subject(&self) -> &SchemaIssueSubject {
        &self.subject
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    pub fn severity_rank(&self) -> u8 {
        self.severity_rank
    }
}
