use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaPath {
    segments: Vec<SchemaPathSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SchemaPathSegment {
    Field(String),
    Index(usize),
    Key(String),
    Variant(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaPathError {
    EmptySegment,
}

impl SchemaPath {
    pub fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn from_segments(
        segments: impl IntoIterator<Item = SchemaPathSegment>,
    ) -> Result<Self, SchemaPathError> {
        let segments = segments.into_iter().collect();

        Ok(Self { segments })
    }

    pub fn with_segment(mut self, segment: SchemaPathSegment) -> Self {
        self.segments.push(segment);
        self
    }

    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn segments(&self) -> &[SchemaPathSegment] {
        &self.segments
    }
}

impl SchemaPathSegment {
    pub fn field(name: impl Into<String>) -> Result<Self, SchemaPathError> {
        let name = name.into();
        validate_segment_name(&name)?;

        Ok(Self::Field(name))
    }

    pub fn index(index: usize) -> Self {
        Self::Index(index)
    }

    pub fn key(key: impl Into<String>) -> Result<Self, SchemaPathError> {
        let key = key.into();
        validate_segment_name(&key)?;

        Ok(Self::Key(key))
    }

    pub fn variant(name: impl Into<String>) -> Result<Self, SchemaPathError> {
        let name = name.into();
        validate_segment_name(&name)?;

        Ok(Self::Variant(name))
    }

    pub fn as_field(&self) -> Option<&str> {
        match self {
            Self::Field(name) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<usize> {
        match self {
            Self::Index(index) => Some(*index),
            _ => None,
        }
    }
}

fn validate_segment_name(value: &str) -> Result<(), SchemaPathError> {
    if value.is_empty() {
        return Err(SchemaPathError::EmptySegment);
    }

    Ok(())
}
