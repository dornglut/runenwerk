use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

/// Stable machine-readable diagnostic code.
///
/// Expected format:
///
/// ```text
/// <domain>.<area>.<condition>
/// ```
///
/// Examples:
///
/// ```text
/// ui_surface.mount.unknown_host
/// editor_shell.route.stale_projection_epoch
/// foundation.diagnostics.code.invalid_format
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCode {
    value: CodeStorage,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CodeStorage {
    Static(&'static str),

    #[cfg(feature = "alloc")]
    Owned(String),
}

/// Error returned when constructing an invalid diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCodeError {
    Empty,
    ContainsWhitespace,
    MissingSeparator,
    EmptySegment,
    SentenceLike,
}

impl DiagnosticCode {
    /// Creates a diagnostic code from a static string.
    ///
    /// Use this for normal domain-owned diagnostic constants.
    pub const fn from_static(value: &'static str) -> Result<Self, DiagnosticCodeError> {
        match validate_code(value) {
            Ok(()) => Ok(Self {
                value: CodeStorage::Static(value),
            }),
            Err(error) => Err(error),
        }
    }

    /// Creates a diagnostic code from a static string without validation.
    ///
    /// This should only be used for constants whose format has been reviewed.
    pub const fn from_static_unchecked(value: &'static str) -> Self {
        Self {
            value: CodeStorage::Static(value),
        }
    }

    /// Creates an owned diagnostic code.
    ///
    /// Owned codes are primarily for deserialization, adapters, external input,
    /// and tests where dynamic construction is unavoidable.
    #[cfg(feature = "alloc")]
    pub fn new(value: impl Into<String>) -> Result<Self, DiagnosticCodeError> {
        let value = value.into();
        validate_code(&value)?;
        Ok(Self {
            value: CodeStorage::Owned(value),
        })
    }

    pub fn as_str(&self) -> &str {
        match &self.value {
            CodeStorage::Static(value) => value,
            #[cfg(feature = "alloc")]
            CodeStorage::Owned(value) => value.as_str(),
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub const fn validate_code(value: &str) -> Result<(), DiagnosticCodeError> {
    let bytes = value.as_bytes();

    if bytes.is_empty() {
        return Err(DiagnosticCodeError::Empty);
    }

    let mut index = 0;
    let mut has_separator = false;
    let mut segment_len = 0;

    while index < bytes.len() {
        let byte = bytes[index];

        if byte == b'.' {
            if segment_len == 0 {
                return Err(DiagnosticCodeError::EmptySegment);
            }

            has_separator = true;
            segment_len = 0;
            index += 1;
            continue;
        }

        if byte == b' '
            || byte == b'\t'
            || byte == b'\n'
            || byte == b'\r'
            || byte == b'\x0C'
            || byte == b'\x0B'
        {
            return Err(DiagnosticCodeError::ContainsWhitespace);
        }

        if byte == b'!' || byte == b'?' {
            return Err(DiagnosticCodeError::SentenceLike);
        }

        segment_len += 1;
        index += 1;
    }

    if segment_len == 0 {
        return Err(DiagnosticCodeError::EmptySegment);
    }

    if !has_separator {
        return Err(DiagnosticCodeError::MissingSeparator);
    }

    Ok(())
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::new(value).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid diagnostic code: {error:?}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticCode, DiagnosticCodeError};

    #[test]
    fn valid_static_code_is_preserved() {
        let code = DiagnosticCode::from_static("ui_surface.mount.unknown_host").unwrap();

        assert_eq!(code.as_str(), "ui_surface.mount.unknown_host");
    }

    #[test]
    fn empty_code_is_rejected() {
        assert_eq!(
            DiagnosticCode::from_static(""),
            Err(DiagnosticCodeError::Empty)
        );
    }

    #[test]
    fn code_with_whitespace_is_rejected() {
        assert_eq!(
            DiagnosticCode::from_static("ui_surface.mount unknown_host"),
            Err(DiagnosticCodeError::ContainsWhitespace)
        );
    }

    #[test]
    fn code_with_sentence_text_is_rejected() {
        assert_eq!(
            DiagnosticCode::from_static("ui_surface.mount.unknown_host?"),
            Err(DiagnosticCodeError::SentenceLike)
        );
    }

    #[test]
    fn code_without_separator_is_rejected() {
        assert_eq!(
            DiagnosticCode::from_static("ui_surface"),
            Err(DiagnosticCodeError::MissingSeparator)
        );
    }

    #[test]
    fn code_with_empty_segment_is_rejected() {
        assert_eq!(
            DiagnosticCode::from_static("ui_surface..unknown_host"),
            Err(DiagnosticCodeError::EmptySegment)
        );
    }
}
