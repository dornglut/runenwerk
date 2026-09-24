use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandContractVersion {
    value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandContractVersionError {
    Zero,
}

impl CommandContractVersion {
    pub fn new(value: u32) -> Result<Self, CommandContractVersionError> {
        if value == 0 {
            return Err(CommandContractVersionError::Zero);
        }

        Ok(Self { value })
    }

    pub fn value(self) -> u32 {
        self.value
    }
}

impl fmt::Display for CommandContractVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.value)
    }
}
