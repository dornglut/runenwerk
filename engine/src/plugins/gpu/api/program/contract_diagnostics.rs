use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuProgramContractCause {
    InvalidEntryPointName,
    EntryPointMissing,
    DuplicateEntryPoint,
    CanonicalWgslInvalid,
    InvalidBindingKey,
    EmptyStageVisibility,
    BindingDeclarationInvalid,
    BindingRefinementInvalid,
    DuplicateBindingKey,
    ProgramInterfaceMismatch,
    BindGroupLayoutInvalid,
    DuplicateBindGroupLayout,
    RuntimeBindingIncompatible,
    StageIoSignatureInvalid,
    PipelineStageIoMismatch,
    InvalidSpecializationKey,
    InvalidSpecializationValue,
    DuplicateSpecializationKey,
    SpecializationUnknownMissingOrTypeMismatch,
    SpecializationRequirementConflict,
    SpecializationOverridesUnsupported,
    PipelineDescriptorInvalid,
    VertexInputStateInvalid,
    RenderAttachmentStateInvalid,
    RenderPrimitiveStateInvalid,
    RenderMultisampleStateInvalid,
    RenderPipelineStateInvalid,
    InvalidDiagnosticMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuProgramContractError {
    operation: &'static str,
    label: String,
    cause: GpuProgramContractCause,
    correction: &'static str,
}

impl GpuProgramContractError {
    pub(crate) fn invalid(
        operation: &'static str,
        label: impl Into<String>,
        cause: GpuProgramContractCause,
        correction: &'static str,
    ) -> Self {
        Self {
            operation,
            label: label.into(),
            cause,
            correction,
        }
    }

    pub const fn cause(&self) -> GpuProgramContractCause {
        self.cause
    }

    pub const fn operation(&self) -> &'static str {
        self.operation
    }

    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub const fn correction(&self) -> &'static str {
        self.correction
    }
}

impl fmt::Display for GpuProgramContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot {} '{}': {:?}; correction: {}",
            self.operation, self.label, self.cause, self.correction
        )
    }
}

impl std::error::Error for GpuProgramContractError {}
