use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaMetadata {
    entries: Vec<SchemaMetadataEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaMetadataEntry {
    key: String,
    value: SchemaMetadataValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SchemaMetadataValue {
    String(String),
    Integer(i64),
    UnsignedInteger(u64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaMetadataError {
    EmptyKey,
    DuplicateKey(String),
}

impl SchemaMetadata {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn from_entries(
        entries: impl IntoIterator<Item = SchemaMetadataEntry>,
    ) -> Result<Self, SchemaMetadataError> {
        let mut metadata = Self::new();

        for entry in entries {
            metadata.push(entry)?;
        }

        Ok(metadata)
    }

    pub fn push(&mut self, entry: SchemaMetadataEntry) -> Result<(), SchemaMetadataError> {
        if self
            .entries
            .iter()
            .any(|existing| existing.key == entry.key)
        {
            return Err(SchemaMetadataError::DuplicateKey(entry.key));
        }

        self.entries.push(entry);
        Ok(())
    }

    pub fn entries(&self) -> &[SchemaMetadataEntry] {
        &self.entries
    }

    pub fn iter(&self) -> core::slice::Iter<'_, SchemaMetadataEntry> {
        self.entries.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl SchemaMetadataEntry {
    pub fn new(
        key: impl Into<String>,
        value: SchemaMetadataValue,
    ) -> Result<Self, SchemaMetadataError> {
        let key = key.into();
        validate_metadata_key(&key)?;

        Ok(Self { key, value })
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn value(&self) -> &SchemaMetadataValue {
        &self.value
    }
}

impl SchemaMetadataValue {
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn integer(value: i64) -> Self {
        Self::Integer(value)
    }

    pub fn unsigned_integer(value: u64) -> Self {
        Self::UnsignedInteger(value)
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }
}

fn validate_metadata_key(value: &str) -> Result<(), SchemaMetadataError> {
    if value.is_empty() {
        return Err(SchemaMetadataError::EmptyKey);
    }

    Ok(())
}
