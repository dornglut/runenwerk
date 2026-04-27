use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

/// Human-readable diagnostic message.
///
/// Messages are not identity. Tools must use diagnostic codes, domains,
/// subjects, locations, and metadata for machine-readable meaning.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticMessage {
    value: MessageStorage,
}

/// Human-readable supporting note.
///
/// Notes provide context. They are not identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticNote {
    value: MessageStorage,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MessageStorage {
    Static(&'static str),

    #[cfg(feature = "alloc")]
    Owned(String),
}

impl DiagnosticMessage {
    pub const fn from_static(value: &'static str) -> Self {
        Self {
            value: MessageStorage::Static(value),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: MessageStorage::Owned(value.into()),
        }
    }

    pub fn as_str(&self) -> &str {
        match &self.value {
            MessageStorage::Static(value) => value,
            #[cfg(feature = "alloc")]
            MessageStorage::Owned(value) => value.as_str(),
        }
    }
}

impl DiagnosticNote {
    pub const fn from_static(value: &'static str) -> Self {
        Self {
            value: MessageStorage::Static(value),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: MessageStorage::Owned(value.into()),
        }
    }

    pub fn as_str(&self) -> &str {
        match &self.value {
            MessageStorage::Static(value) => value,
            #[cfg(feature = "alloc")]
            MessageStorage::Owned(value) => value.as_str(),
        }
    }
}

impl fmt::Display for DiagnosticMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Display for DiagnosticNote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticNote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticNote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticMessage, DiagnosticNote};

    #[test]
    fn message_is_not_identity() {
        let message = DiagnosticMessage::from_static("Unknown surface host.");

        assert_eq!(message.as_str(), "Unknown surface host.");
    }

    #[test]
    fn note_preserves_text() {
        let note = DiagnosticNote::from_static("Register the host before mounting surfaces.");

        assert_eq!(note.as_str(), "Register the host before mounting surfaces.");
    }
}
