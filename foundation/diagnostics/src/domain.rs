use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::String;

/// Identifies the conceptual owner of a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticDomain {
    value: DomainStorage,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum DomainStorage {
    Static(&'static str),

    #[cfg(feature = "alloc")]
    Owned(String),
}

/// Error returned when constructing an invalid diagnostic domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticDomainError {
    Empty,
    ContainsWhitespace,
}

impl DiagnosticDomain {
    /// Creates a domain from a static string.
    pub const fn from_static(value: &'static str) -> Result<Self, DiagnosticDomainError> {
        match validate_domain(value) {
            Ok(()) => Ok(Self {
                value: DomainStorage::Static(value),
            }),
            Err(error) => Err(error),
        }
    }

    /// Creates a domain from a static string without validation.
    ///
    /// This should only be used for constants whose format has been reviewed.
    pub const fn from_static_unchecked(value: &'static str) -> Self {
        Self {
            value: DomainStorage::Static(value),
        }
    }

    /// Creates an owned domain.
    #[cfg(feature = "alloc")]
    pub fn new(value: impl Into<String>) -> Result<Self, DiagnosticDomainError> {
        let value = value.into();
        validate_domain(&value)?;
        Ok(Self {
            value: DomainStorage::Owned(value),
        })
    }

    pub fn as_str(&self) -> &str {
        match &self.value {
            DomainStorage::Static(value) => value,
            #[cfg(feature = "alloc")]
            DomainStorage::Owned(value) => value.as_str(),
        }
    }
}

impl fmt::Display for DiagnosticDomain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub const fn validate_domain(value: &str) -> Result<(), DiagnosticDomainError> {
    let bytes = value.as_bytes();

    if bytes.is_empty() {
        return Err(DiagnosticDomainError::Empty);
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
            return Err(DiagnosticDomainError::ContainsWhitespace);
        }

        index += 1;
    }

    Ok(())
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticDomain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticDomain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::new(value).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid diagnostic domain: {error:?}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticDomain, DiagnosticDomainError};

    #[test]
    fn valid_static_domain_is_preserved() {
        let domain = DiagnosticDomain::from_static("ui_surface").unwrap();

        assert_eq!(domain.as_str(), "ui_surface");
    }

    #[test]
    fn empty_domain_is_rejected() {
        assert_eq!(
            DiagnosticDomain::from_static(""),
            Err(DiagnosticDomainError::Empty)
        );
    }

    #[test]
    fn domain_with_whitespace_is_rejected() {
        assert_eq!(
            DiagnosticDomain::from_static("ui surface"),
            Err(DiagnosticDomainError::ContainsWhitespace)
        );
    }
}
