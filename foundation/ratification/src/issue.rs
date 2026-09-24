use alloc::string::{String, ToString};

use crate::RatificationSeverity;

/// One domain-owned ratification issue.
///
/// `Code` and `Subject` are generic because concrete meaning belongs to the
/// owning domain. Foundation provides the shared shape only.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RatificationIssue<Code, Subject> {
    code: Code,
    subject: Subject,
    severity: RatificationSeverity,
    message: String,
}

impl<Code, Subject> RatificationIssue<Code, Subject> {
    pub fn new(
        code: Code,
        subject: Subject,
        severity: RatificationSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            subject,
            severity,
            message: message.into(),
        }
    }

    pub fn info(code: Code, subject: Subject, message: impl Into<String>) -> Self {
        Self::new(code, subject, RatificationSeverity::Info, message)
    }

    pub fn warning(code: Code, subject: Subject, message: impl Into<String>) -> Self {
        Self::new(code, subject, RatificationSeverity::Warning, message)
    }

    pub fn error(code: Code, subject: Subject, message: impl Into<String>) -> Self {
        Self::new(code, subject, RatificationSeverity::Error, message)
    }

    pub fn fatal(code: Code, subject: Subject, message: impl Into<String>) -> Self {
        Self::new(code, subject, RatificationSeverity::Fatal, message)
    }

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn subject(&self) -> &Subject {
        &self.subject
    }

    pub fn severity(&self) -> RatificationSeverity {
        self.severity
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    pub fn is_blocking(&self) -> bool {
        self.severity.is_blocking()
    }

    pub fn into_parts(self) -> (Code, Subject, RatificationSeverity, String) {
        (self.code, self.subject, self.severity, self.message)
    }
}

impl<Code, Subject> RatificationIssue<Code, Subject>
where
    Code: Copy,
    Subject: Copy,
{
    pub fn with_message(&self, message: impl ToString) -> Self {
        Self {
            code: self.code,
            subject: self.subject,
            severity: self.severity,
            message: message.to_string(),
        }
    }
}
