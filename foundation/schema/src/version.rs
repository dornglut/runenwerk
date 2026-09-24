use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaVersion {
    value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaVersionError {
    Zero,
}

impl SchemaVersion {
    pub fn new(value: u32) -> Result<Self, SchemaVersionError> {
        if value == 0 {
            return Err(SchemaVersionError::Zero);
        }

        Ok(Self { value })
    }

    pub fn value(self) -> u32 {
        self.value
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.value)
    }
}
