use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Ordered structured metadata attached to a diagnostic.
///
/// Metadata is for portable machine-readable context. It must not become an
/// arbitrary object graph or JSON dumping ground.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticMetadata {
    entries: Vec<DiagnosticMetadataEntry>,
}

/// One ordered metadata key/value entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticMetadataEntry {
    key: DiagnosticMetadataKey,
    value: DiagnosticMetadataValue,
}

/// Domain-owned metadata key.
///
/// Examples:
///
/// - `expected`
/// - `actual`
/// - `field`
/// - `path`
/// - `version`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticMetadataKey {
    value: String,
}

/// Portable metadata value.
///
/// Floats are intentionally omitted from v1 to avoid unstable equality,
/// tolerance, ordering, and golden-test semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticMetadataValue {
    String(String),
    Integer(i64),
    Bool(bool),
    Id(String),
    Path(String),
}

/// Error returned when constructing an invalid metadata key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticMetadataKeyError {
    Empty,
    ContainsWhitespace,
}

impl DiagnosticMetadata {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn with_entry(mut self, entry: DiagnosticMetadataEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn push(&mut self, entry: DiagnosticMetadataEntry) {
        self.entries.push(entry);
    }

    pub fn iter(&self) -> core::slice::Iter<'_, DiagnosticMetadataEntry> {
        self.entries.iter()
    }

    pub fn entries(&self) -> &[DiagnosticMetadataEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl DiagnosticMetadataEntry {
    pub fn new(key: DiagnosticMetadataKey, value: DiagnosticMetadataValue) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &DiagnosticMetadataKey {
        &self.key
    }

    pub fn value(&self) -> &DiagnosticMetadataValue {
        &self.value
    }
}

impl DiagnosticMetadataKey {
    pub fn new(value: impl Into<String>) -> Result<Self, DiagnosticMetadataKeyError> {
        let value = value.into();
        validate_metadata_key(&value)?;

        Ok(Self { value })
    }

    pub fn from_static(value: &'static str) -> Result<Self, DiagnosticMetadataKeyError> {
        Self::new(value)
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

impl DiagnosticMetadataValue {
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn integer(value: i64) -> Self {
        Self::Integer(value)
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn id(value: impl Into<String>) -> Self {
        Self::Id(value.into())
    }

    pub fn path(value: impl Into<String>) -> Self {
        Self::Path(value.into())
    }
}

impl fmt::Display for DiagnosticMetadataKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Display for DiagnosticMetadataValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticMetadataValue::String(value) => formatter.write_str(value),
            DiagnosticMetadataValue::Integer(value) => write!(formatter, "{value}"),
            DiagnosticMetadataValue::Bool(value) => write!(formatter, "{value}"),
            DiagnosticMetadataValue::Id(value) => formatter.write_str(value),
            DiagnosticMetadataValue::Path(value) => formatter.write_str(value),
        }
    }
}

pub fn validate_metadata_key(value: &str) -> Result<(), DiagnosticMetadataKeyError> {
    if value.is_empty() {
        return Err(DiagnosticMetadataKeyError::Empty);
    }

    if value.chars().any(char::is_whitespace) {
        return Err(DiagnosticMetadataKeyError::ContainsWhitespace);
    }

    Ok(())
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticMetadataKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticMetadataKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::new(value).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid diagnostic metadata key: {error:?}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiagnosticMetadata, DiagnosticMetadataEntry, DiagnosticMetadataKey,
        DiagnosticMetadataKeyError, DiagnosticMetadataValue,
    };

    #[test]
    fn metadata_preserves_order() {
        let mut metadata = DiagnosticMetadata::new();

        metadata.push(DiagnosticMetadataEntry::new(
            DiagnosticMetadataKey::from_static("expected").unwrap(),
            DiagnosticMetadataValue::string("registered surface host"),
        ));
        metadata.push(DiagnosticMetadataEntry::new(
            DiagnosticMetadataKey::from_static("actual").unwrap(),
            DiagnosticMetadataValue::string("host id 7"),
        ));

        let entries = metadata.entries();

        assert_eq!(entries[0].key().as_str(), "expected");
        assert_eq!(entries[1].key().as_str(), "actual");
    }

    #[test]
    fn metadata_rejects_empty_key() {
        assert_eq!(
            DiagnosticMetadataKey::from_static(""),
            Err(DiagnosticMetadataKeyError::Empty)
        );
    }

    #[test]
    fn metadata_rejects_key_with_whitespace() {
        assert_eq!(
            DiagnosticMetadataKey::from_static("bad key"),
            Err(DiagnosticMetadataKeyError::ContainsWhitespace)
        );
    }

    #[test]
    fn metadata_preserves_key_value_identity() {
        let entry = DiagnosticMetadataEntry::new(
            DiagnosticMetadataKey::from_static("field").unwrap(),
            DiagnosticMetadataValue::string("parent"),
        );

        assert_eq!(entry.key().as_str(), "field");
        assert_eq!(
            entry.value(),
            &DiagnosticMetadataValue::String("parent".to_string())
        );
    }

    #[test]
    fn metadata_value_string_is_preserved() {
        let value = DiagnosticMetadataValue::string("registered surface host");

        assert_eq!(
            value,
            DiagnosticMetadataValue::String("registered surface host".to_string())
        );
    }

    #[test]
    fn metadata_value_integer_is_preserved() {
        let value = DiagnosticMetadataValue::integer(3);

        assert_eq!(value, DiagnosticMetadataValue::Integer(3));
    }

    #[test]
    fn metadata_value_bool_is_preserved() {
        let value = DiagnosticMetadataValue::bool(true);

        assert_eq!(value, DiagnosticMetadataValue::Bool(true));
    }

    #[test]
    fn metadata_value_id_is_preserved() {
        let value = DiagnosticMetadataValue::id("surface_host_7");

        assert_eq!(
            value,
            DiagnosticMetadataValue::Id("surface_host_7".to_string())
        );
    }

    #[test]
    fn metadata_value_path_is_preserved() {
        let value = DiagnosticMetadataValue::path("entities[4].parent");

        assert_eq!(
            value,
            DiagnosticMetadataValue::Path("entities[4].parent".to_string())
        );
    }
}
