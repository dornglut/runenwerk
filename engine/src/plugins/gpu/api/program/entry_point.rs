use super::contract_diagnostics::{GpuProgramContractCause, GpuProgramContractError};
use super::interface::{GpuProgramInterfaceDescriptor, GpuShaderStage};
use core::fmt;
use core::str::FromStr;

const MAX_ENTRY_POINT_NAME_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuEntryPointName(String);

impl GpuEntryPointName {
    pub fn new(value: impl Into<String>) -> Result<Self, GpuProgramContractError> {
        let value = value.into();
        let mut chars = value.chars();
        let valid_start = chars
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic());
        let valid_rest =
            chars.all(|character| character == '_' || character.is_ascii_alphanumeric());
        if !valid_start || !valid_rest || value.len() > MAX_ENTRY_POINT_NAME_BYTES {
            return Err(GpuProgramContractError::invalid(
                "construct GPU entry-point name",
                if value.is_empty() {
                    "<empty>"
                } else {
                    value.as_str()
                },
                GpuProgramContractCause::InvalidEntryPointName,
                "provide a bounded WGSL-compatible identifier beginning with a letter or underscore",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for GpuEntryPointName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for GpuEntryPointName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for GpuEntryPointName {
    type Err = GpuProgramContractError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuEntryPointDescriptor {
    name: GpuEntryPointName,
    stage: GpuShaderStage,
    interface: GpuProgramInterfaceDescriptor,
}

impl GpuEntryPointDescriptor {
    pub fn new(
        name: GpuEntryPointName,
        stage: GpuShaderStage,
        interface: GpuProgramInterfaceDescriptor,
    ) -> Self {
        Self {
            name,
            stage,
            interface,
        }
    }

    pub fn name(&self) -> &GpuEntryPointName {
        &self.name
    }

    pub const fn stage(&self) -> GpuShaderStage {
        self.stage
    }

    pub fn interface(&self) -> &GpuProgramInterfaceDescriptor {
        &self.interface
    }
}
