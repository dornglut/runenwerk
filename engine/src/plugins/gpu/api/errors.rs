use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuCapabilityRequirementCause {
    ConflictingStrength,
    AmbiguousPreferredFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuCapabilityRequirementError {
    Invalid {
        operation: &'static str,
        label: String,
        cause: GpuCapabilityRequirementCause,
        correction: &'static str,
    },
}

impl fmt::Display for GpuCapabilityRequirementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Invalid {
            operation,
            label,
            cause,
            correction,
        } = self;
        write!(
            f,
            "cannot {operation} '{label}': {cause:?}; correction: {correction}"
        )
    }
}

impl std::error::Error for GpuCapabilityRequirementError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuCapabilityAdmissionCause {
    RequiredUnavailable,
    RequiredNotEnabled,
    DisabledEnabled,
    EnabledUnavailable,
    InvalidLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuCapabilityAdmissionError {
    Rejected {
        operation: &'static str,
        label: String,
        cause: GpuCapabilityAdmissionCause,
        correction: &'static str,
    },
}

impl fmt::Display for GpuCapabilityAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Rejected {
            operation,
            label,
            cause,
            correction,
        } = self;
        write!(
            f,
            "cannot {operation} '{label}': {cause:?}; correction: {correction}"
        )
    }
}

impl std::error::Error for GpuCapabilityAdmissionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuResourceDescriptorCause {
    EmptyLabel,
    ZeroSize,
    ArithmeticOverflow,
    EmptyUsage,
    InvalidOwnership,
    InvalidReconstruction,
    InvalidMemoryIntent,
    InvalidInitialization,
    InitializationLengthMismatch,
    InvalidExtent,
    InvalidMipCount,
    InvalidSampleCount,
    InvalidFormatUsage,
    InvalidAspect,
    InvalidRowLayout,
    InsufficientTextureData,
    ParentLeaseMismatch,
    SubresourceOutOfBounds,
    IncompatibleViewFormat,
    IncompatibleViewDimension,
    InvalidLodRange,
    InvalidQueryCount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuResourceDescriptorError {
    Invalid {
        operation: &'static str,
        label: String,
        cause: GpuResourceDescriptorCause,
        correction: &'static str,
    },
}

impl GpuResourceDescriptorError {
    pub(crate) fn invalid(
        operation: &'static str,
        label: impl Into<String>,
        cause: GpuResourceDescriptorCause,
        correction: &'static str,
    ) -> Self {
        Self::Invalid {
            operation,
            label: label.into(),
            cause,
            correction,
        }
    }

    pub fn cause(&self) -> GpuResourceDescriptorCause {
        match self {
            Self::Invalid { cause, .. } => *cause,
        }
    }
}

impl fmt::Display for GpuResourceDescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Invalid {
            operation,
            label,
            cause,
            correction,
        } = self;
        write!(
            f,
            "cannot {operation} '{label}': {cause:?}; correction: {correction}"
        )
    }
}

impl std::error::Error for GpuResourceDescriptorError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuHandleCause {
    WrongKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuHandleError {
    Invalid {
        operation: &'static str,
        label: String,
        cause: GpuHandleCause,
        correction: &'static str,
    },
}

impl fmt::Display for GpuHandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Invalid {
            operation,
            label,
            cause,
            correction,
        } = self;
        write!(
            f,
            "cannot {operation} '{label}': {cause:?}; correction: {correction}"
        )
    }
}

impl std::error::Error for GpuHandleError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuDataPreparationCause {
    ZeroLength,
    InvalidAlignment,
    InvalidStride,
    InvalidElementCount,
    ArithmeticOverflow,
    LengthMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuDataPreparationError {
    Invalid {
        operation: &'static str,
        label: String,
        cause: GpuDataPreparationCause,
        correction: &'static str,
    },
}

impl GpuDataPreparationError {
    pub(crate) fn invalid(
        operation: &'static str,
        label: impl Into<String>,
        cause: GpuDataPreparationCause,
        correction: &'static str,
    ) -> Self {
        Self::Invalid {
            operation,
            label: label.into(),
            cause,
            correction,
        }
    }
}

impl fmt::Display for GpuDataPreparationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Invalid {
            operation,
            label,
            cause,
            correction,
        } = self;
        write!(
            f,
            "cannot {operation} '{label}': {cause:?}; correction: {correction}"
        )
    }
}

impl std::error::Error for GpuDataPreparationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuReadbackDecodeCause {
    InvalidLength,
    InvalidFormat,
    DecoderRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuReadbackDecodeError {
    Invalid {
        operation: &'static str,
        label: String,
        cause: GpuReadbackDecodeCause,
        correction: &'static str,
    },
}

impl fmt::Display for GpuReadbackDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Invalid {
            operation,
            label,
            cause,
            correction,
        } = self;
        write!(
            f,
            "cannot {operation} '{label}': {cause:?}; correction: {correction}"
        )
    }
}

impl std::error::Error for GpuReadbackDecodeError {}
