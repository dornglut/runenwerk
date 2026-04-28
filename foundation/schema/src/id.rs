#[cfg(feature = "alloc")]
use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SchemaId {
    Static(&'static str),
    #[cfg(feature = "alloc")]
    Owned(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaIdError {
    Empty,
    ContainsWhitespace,
    InvalidCharacter,
}

impl SchemaId {
    #[cfg(feature = "alloc")]
    pub fn new(value: impl Into<String>) -> Result<Self, SchemaIdError> {
        let value = value.into();
        validate_schema_id(&value)?;

        Ok(Self::Owned(value))
    }

    pub fn from_static(value: &'static str) -> Result<Self, SchemaIdError> {
        validate_schema_id(value)?;

        Ok(Self::Static(value))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(value) => value,
            #[cfg(feature = "alloc")]
            Self::Owned(value) => value.as_str(),
        }
    }
}

pub fn validate_schema_id(value: &str) -> Result<(), SchemaIdError> {
    if value.is_empty() {
        return Err(SchemaIdError::Empty);
    }

    if value.chars().any(char::is_whitespace) {
        return Err(SchemaIdError::ContainsWhitespace);
    }

    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(SchemaIdError::InvalidCharacter);
    }

    Ok(())
}

impl fmt::Display for SchemaId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for SchemaId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SchemaId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::new(value)
            .map_err(|error| serde::de::Error::custom(format_args!("invalid schema id: {error:?}")))
    }
}
