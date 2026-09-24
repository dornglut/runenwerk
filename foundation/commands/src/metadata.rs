use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandMetadata {
    entries: Vec<CommandMetadataEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandMetadataEntry {
    key: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandMetadataError {
    EmptyKey,
    DuplicateKey(String),
}

impl CommandMetadata {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn from_entries(
        entries: impl IntoIterator<Item = CommandMetadataEntry>,
    ) -> Result<Self, CommandMetadataError> {
        let mut metadata = Self::new();

        for entry in entries {
            metadata.push(entry)?;
        }

        Ok(metadata)
    }

    pub fn push(&mut self, entry: CommandMetadataEntry) -> Result<(), CommandMetadataError> {
        if self
            .entries
            .iter()
            .any(|existing| existing.key == entry.key)
        {
            return Err(CommandMetadataError::DuplicateKey(entry.key));
        }

        self.entries.push(entry);
        Ok(())
    }

    pub fn entries(&self) -> &[CommandMetadataEntry] {
        &self.entries
    }

    pub fn iter(&self) -> core::slice::Iter<'_, CommandMetadataEntry> {
        self.entries.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl CommandMetadataEntry {
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, CommandMetadataError> {
        let key = key.into();
        validate_metadata_key(&key)?;

        Ok(Self {
            key,
            value: value.into(),
        })
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

fn validate_metadata_key(value: &str) -> Result<(), CommandMetadataError> {
    if value.is_empty() {
        return Err(CommandMetadataError::EmptyKey);
    }

    Ok(())
}
