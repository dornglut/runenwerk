//! File: domain/ui/ui_controls/src/diagnostics.rs
//! Crate: ui_controls

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlDiagnosticId(String);

impl ControlDiagnosticId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control diagnostic IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlDiagnosticContractError> {
        let value = value.into();
        validate_diagnostic_id(&value, "diagnostic")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlDiagnosticSeverity {
    Info,
    Warning,
    #[default]
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlDiagnosticDescriptor {
    pub diagnostic_id: ControlDiagnosticId,
    pub severity: ControlDiagnosticSeverity,
    pub message_template: String,
}

impl ControlDiagnosticDescriptor {
    pub fn new(diagnostic_id: ControlDiagnosticId, message_template: impl Into<String>) -> Self {
        Self {
            diagnostic_id,
            severity: ControlDiagnosticSeverity::Error,
            message_template: message_template.into(),
        }
    }

    pub fn with_severity(mut self, severity: ControlDiagnosticSeverity) -> Self {
        self.severity = severity;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlDiagnosticRecord {
    pub diagnostic_id: ControlDiagnosticId,
    pub control_kind_id: String,
    pub message: String,
}

impl ControlDiagnosticRecord {
    pub fn new(
        diagnostic_id: ControlDiagnosticId,
        control_kind_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            diagnostic_id,
            control_kind_id: control_kind_id.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlDiagnosticContractError {
    EmptyId { kind: &'static str },
    UnnamespacedId { kind: &'static str, value: String },
    InvalidIdCharacter { kind: &'static str, value: String },
}

impl fmt::Display for ControlDiagnosticContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId { kind } => write!(formatter, "empty control {kind} id"),
            Self::UnnamespacedId { kind, value } => {
                write!(formatter, "control {kind} id {value} is not namespaced")
            }
            Self::InvalidIdCharacter { kind, value } => write!(
                formatter,
                "control {kind} id {value} contains an invalid character"
            ),
        }
    }
}

impl std::error::Error for ControlDiagnosticContractError {}

fn validate_diagnostic_id(
    value: &str,
    kind: &'static str,
) -> Result<(), ControlDiagnosticContractError> {
    if value.is_empty() {
        return Err(ControlDiagnosticContractError::EmptyId { kind });
    }
    if !value.contains('.') {
        return Err(ControlDiagnosticContractError::UnnamespacedId {
            kind,
            value: value.to_owned(),
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(ControlDiagnosticContractError::InvalidIdCharacter {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}
