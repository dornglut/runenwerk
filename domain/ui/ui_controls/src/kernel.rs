//! File: domain/ui/ui_controls/src/kernel.rs
//! Crate: ui_controls

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlKernelId(String);

impl ControlKernelId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control kernel IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlKernelContractError> {
        let value = value.into();
        validate_kernel_id(&value, "kernel")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ControlKernelVersion(u32);

impl ControlKernelVersion {
    pub const fn new(value: u32) -> Self {
        assert!(value > 0);
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlKernelKind {
    Layout,
    Interaction,
    Visual,
    Accessibility,
    Inspection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlKernelDescriptor {
    pub kernel_id: ControlKernelId,
    pub version: ControlKernelVersion,
    pub kind: ControlKernelKind,
    pub deterministic: bool,
}

impl ControlKernelDescriptor {
    pub fn new(kernel_id: ControlKernelId, kind: ControlKernelKind) -> Self {
        Self {
            kernel_id,
            version: ControlKernelVersion::new(1),
            kind,
            deterministic: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlKernelSet {
    pub layout: ControlKernelId,
    pub interaction: ControlKernelId,
    pub visual: ControlKernelId,
    pub accessibility: ControlKernelId,
    pub inspection: ControlKernelId,
}

impl ControlKernelSet {
    pub fn new(
        layout: ControlKernelId,
        interaction: ControlKernelId,
        visual: ControlKernelId,
        accessibility: ControlKernelId,
        inspection: ControlKernelId,
    ) -> Self {
        Self {
            layout,
            interaction,
            visual,
            accessibility,
            inspection,
        }
    }

    pub fn ids(&self) -> [&ControlKernelId; 5] {
        [
            &self.layout,
            &self.interaction,
            &self.visual,
            &self.accessibility,
            &self.inspection,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlKernelContractError {
    EmptyId { kind: &'static str },
    UnnamespacedId { kind: &'static str, value: String },
    InvalidIdCharacter { kind: &'static str, value: String },
}

impl fmt::Display for ControlKernelContractError {
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

impl std::error::Error for ControlKernelContractError {}

fn validate_kernel_id(value: &str, kind: &'static str) -> Result<(), ControlKernelContractError> {
    if value.is_empty() {
        return Err(ControlKernelContractError::EmptyId { kind });
    }
    if !value.contains('.') {
        return Err(ControlKernelContractError::UnnamespacedId {
            kind,
            value: value.to_owned(),
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(ControlKernelContractError::InvalidIdCharacter {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}
