use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

use crate::DiagnosticMessage;

/// Structured diagnostic subject.
///
/// A subject identifies the thing a diagnostic is about without requiring tools
/// to parse the human-readable message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticSubject {
    kind: DiagnosticSubjectKind,
    id: Option<DiagnosticSubjectId>,
    label: Option<DiagnosticMessage>,
}

/// Structured subject kind.
///
/// Examples:
///
/// - `surface_instance`
/// - `surface_host`
/// - `entity`
/// - `scene_file`
/// - `render_resource`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticSubjectKind {
    value: SubjectNameStorage,
}

/// Structured subject identifier.
///
/// Identifiers are domain-owned and portable. They are not parsed by this crate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticSubjectId {
    value: SubjectNameStorage,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SubjectNameStorage {
    Static(&'static str),

    #[cfg(feature = "alloc")]
    Owned(String),
}

/// Error returned when constructing an invalid diagnostic subject name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSubjectNameError {
    Empty,
    ContainsWhitespace,
}

impl DiagnosticSubject {
    pub fn new(kind: DiagnosticSubjectKind) -> Self {
        Self {
            kind,
            id: None,
            label: None,
        }
    }

    pub fn with_id(mut self, id: DiagnosticSubjectId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_label(mut self, label: DiagnosticMessage) -> Self {
        self.label = Some(label);
        self
    }

    pub fn kind(&self) -> &DiagnosticSubjectKind {
        &self.kind
    }

    pub fn id(&self) -> Option<&DiagnosticSubjectId> {
        self.id.as_ref()
    }

    pub fn label(&self) -> Option<&DiagnosticMessage> {
        self.label.as_ref()
    }
}

impl DiagnosticSubjectKind {
    pub const fn from_static(value: &'static str) -> Result<Self, DiagnosticSubjectNameError> {
        match validate_subject_name(value) {
            Ok(()) => Ok(Self {
                value: SubjectNameStorage::Static(value),
            }),
            Err(error) => Err(error),
        }
    }

    pub const fn from_static_unchecked(value: &'static str) -> Self {
        Self {
            value: SubjectNameStorage::Static(value),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn new(value: impl Into<String>) -> Result<Self, DiagnosticSubjectNameError> {
        let value = value.into();
        validate_subject_name(&value)?;
        Ok(Self {
            value: SubjectNameStorage::Owned(value),
        })
    }

    pub fn as_str(&self) -> &str {
        match &self.value {
            SubjectNameStorage::Static(value) => value,
            #[cfg(feature = "alloc")]
            SubjectNameStorage::Owned(value) => value.as_str(),
        }
    }
}

impl DiagnosticSubjectId {
    pub const fn from_static(value: &'static str) -> Result<Self, DiagnosticSubjectNameError> {
        match validate_subject_name(value) {
            Ok(()) => Ok(Self {
                value: SubjectNameStorage::Static(value),
            }),
            Err(error) => Err(error),
        }
    }

    pub const fn from_static_unchecked(value: &'static str) -> Self {
        Self {
            value: SubjectNameStorage::Static(value),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn new(value: impl Into<String>) -> Result<Self, DiagnosticSubjectNameError> {
        let value = value.into();
        validate_subject_name(&value)?;
        Ok(Self {
            value: SubjectNameStorage::Owned(value),
        })
    }

    pub fn as_str(&self) -> &str {
        match &self.value {
            SubjectNameStorage::Static(value) => value,
            #[cfg(feature = "alloc")]
            SubjectNameStorage::Owned(value) => value.as_str(),
        }
    }
}

impl fmt::Display for DiagnosticSubjectKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Display for DiagnosticSubjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub const fn validate_subject_name(value: &str) -> Result<(), DiagnosticSubjectNameError> {
    let bytes = value.as_bytes();

    if bytes.is_empty() {
        return Err(DiagnosticSubjectNameError::Empty);
    }

    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];

        if byte == b' '
            || byte == b'\t'
            || byte == b'\n'
            || byte == b'\r'
            || byte == b'\x0C'
            || byte == b'\x0B'
        {
            return Err(DiagnosticSubjectNameError::ContainsWhitespace);
        }

        index += 1;
    }

    Ok(())
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticSubjectKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticSubjectKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::new(value).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid diagnostic subject kind: {error:?}"))
        })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticSubjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticSubjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::new(value).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid diagnostic subject id: {error:?}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiagnosticSubject, DiagnosticSubjectId, DiagnosticSubjectKind, DiagnosticSubjectNameError,
    };
    use crate::DiagnosticMessage;

    #[test]
    fn subject_preserves_kind_id_and_label() {
        let subject =
            DiagnosticSubject::new(DiagnosticSubjectKind::from_static("surface_instance").unwrap())
                .with_id(DiagnosticSubjectId::from_static("42").unwrap())
                .with_label(DiagnosticMessage::from_static("Inspector"));

        assert_eq!(subject.kind().as_str(), "surface_instance");
        assert_eq!(subject.id().unwrap().as_str(), "42");
        assert_eq!(subject.label().unwrap().as_str(), "Inspector");
    }

    #[test]
    fn subject_without_id_is_allowed() {
        let subject =
            DiagnosticSubject::new(DiagnosticSubjectKind::from_static("surface_host").unwrap());

        assert_eq!(subject.kind().as_str(), "surface_host");
        assert!(subject.id().is_none());
    }

    #[test]
    fn empty_subject_kind_is_rejected() {
        assert_eq!(
            DiagnosticSubjectKind::from_static(""),
            Err(DiagnosticSubjectNameError::Empty)
        );
    }

    #[test]
    fn empty_subject_id_is_rejected() {
        assert_eq!(
            DiagnosticSubjectId::from_static(""),
            Err(DiagnosticSubjectNameError::Empty)
        );
    }

    #[test]
    fn label_is_not_identity() {
        let subject = DiagnosticSubject::new(DiagnosticSubjectKind::from_static("entity").unwrap())
            .with_id(DiagnosticSubjectId::from_static("12").unwrap())
            .with_label(DiagnosticMessage::from_static("Player"));

        assert_eq!(subject.id().unwrap().as_str(), "12");
        assert_eq!(subject.label().unwrap().as_str(), "Player");
    }
}
