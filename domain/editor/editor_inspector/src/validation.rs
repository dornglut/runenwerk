//! File: domain/editor/editor_inspector/src/validation.rs
//! Purpose: Validation messages surfaced by inspector adapters and field editing.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationMessage {
    pub severity: ValidationSeverity,
    pub text: String,
}

impl ValidationMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Info,
            text: text.into(),
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            text: text.into(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            text: text.into(),
        }
    }
}
