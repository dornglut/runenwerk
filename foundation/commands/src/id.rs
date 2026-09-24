#[cfg(feature = "alloc")]
use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommandContractId {
    Static(&'static str),
    #[cfg(feature = "alloc")]
    Owned(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandContractIdError {
    Empty,
    ContainsWhitespace,
    InvalidCharacter,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandContractRef {
    id: CommandContractId,
    version: crate::CommandContractVersion,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandProposalId {
    value: String,
}

#[cfg(not(feature = "alloc"))]
pub type CommandProposalId = CommandContractId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandProposalIdError {
    Empty,
}

impl CommandContractId {
    #[cfg(feature = "alloc")]
    pub fn new(value: impl Into<String>) -> Result<Self, CommandContractIdError> {
        let value = value.into();
        validate_command_contract_id(&value)?;

        Ok(Self::Owned(value))
    }

    pub fn from_static(value: &'static str) -> Result<Self, CommandContractIdError> {
        validate_command_contract_id(value)?;

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

impl CommandContractRef {
    pub fn new(id: CommandContractId, version: crate::CommandContractVersion) -> Self {
        Self { id, version }
    }

    pub fn id(&self) -> &CommandContractId {
        &self.id
    }

    pub fn version(&self) -> crate::CommandContractVersion {
        self.version
    }
}

#[cfg(feature = "alloc")]
impl CommandProposalId {
    pub fn new(value: impl Into<String>) -> Result<Self, CommandProposalIdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(CommandProposalIdError::Empty);
        }

        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

pub fn validate_command_contract_id(value: &str) -> Result<(), CommandContractIdError> {
    if value.is_empty() {
        return Err(CommandContractIdError::Empty);
    }

    if value.chars().any(char::is_whitespace) {
        return Err(CommandContractIdError::ContainsWhitespace);
    }

    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(CommandContractIdError::InvalidCharacter);
    }

    Ok(())
}

impl fmt::Display for CommandContractId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for CommandContractId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CommandContractId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::new(value).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid command contract id: {error:?}"))
        })
    }
}

#[cfg(all(feature = "serde", feature = "alloc"))]
impl serde::Serialize for CommandProposalId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<'de> serde::Deserialize<'de> for CommandProposalId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::new(value).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid command proposal id: {error:?}"))
        })
    }
}
