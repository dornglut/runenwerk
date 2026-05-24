use crate::plugins::render::{GpuStorage, RenderResourceId, StorageArrayHandle};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, GpuStorage)]
pub struct U32ScanElement {
    pub value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixScanMode {
    Exclusive,
    Inclusive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct U32PrefixScanDescriptor {
    pub label: String,
    pub input: RenderResourceId,
    pub output: RenderResourceId,
    pub total_count: u32,
    pub mode: PrefixScanMode,
}

impl U32PrefixScanDescriptor {
    pub fn new<I, O>(
        label: impl Into<String>,
        input: StorageArrayHandle<I>,
        output: StorageArrayHandle<O>,
        total_count: u32,
        mode: PrefixScanMode,
    ) -> Result<Self, GpuPrimitiveValidationError> {
        let descriptor = Self {
            label: label.into(),
            input: *input.id(),
            output: *output.id(),
            total_count,
            mode,
        };
        descriptor.validate()?;
        validate_capacity(
            format!("{}.input", descriptor.label),
            input.len(),
            u64::from(total_count),
        )?;
        validate_capacity(
            format!("{}.output", descriptor.label),
            output.len(),
            u64::from(total_count),
        )?;
        Ok(descriptor)
    }

    pub fn validate(&self) -> Result<(), GpuPrimitiveValidationError> {
        if self.label.trim().is_empty() {
            return Err(GpuPrimitiveValidationError::EmptyLabel {
                primitive: "u32_prefix_scan",
            });
        }
        if self.total_count == 0 {
            return Err(GpuPrimitiveValidationError::ZeroElementCount {
                label: self.label.clone(),
            });
        }
        if self.input == self.output {
            return Err(GpuPrimitiveValidationError::AliasedInputOutput {
                label: self.label.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GpuPrimitiveValidationError {
    #[error("{primitive} primitive label must not be empty")]
    EmptyLabel { primitive: &'static str },

    #[error("gpu primitive '{label}' must process at least one element")]
    ZeroElementCount { label: String },

    #[error("gpu primitive '{label}' requires distinct input and output buffers")]
    AliasedInputOutput { label: String },

    #[error(
        "gpu primitive '{label}' declares capacity {capacity}, but required count is {required_count}"
    )]
    InsufficientCapacity {
        label: String,
        capacity: u64,
        required_count: u64,
    },

    #[error("gpu primitive plan '{label}' must contain at least one step")]
    EmptyExecutionPlan { label: String },
}

pub fn validate_capacity(
    label: impl Into<String>,
    capacity: u64,
    required_count: u64,
) -> Result<(), GpuPrimitiveValidationError> {
    let label = label.into();
    if capacity < required_count {
        return Err(GpuPrimitiveValidationError::InsufficientCapacity {
            label,
            capacity,
            required_count,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::RenderFlow;

    #[test]
    fn gpu_primitives_prefix_scan_uses_real_storage_lengths() {
        let (flow, input) =
            RenderFlow::new("test.primitive.scan").storage_array::<U32ScanElement>("scan.input", 4);
        let (_flow, output) = flow.storage_array::<U32ScanElement>("scan.output", 3);

        assert!(matches!(
            U32PrefixScanDescriptor::new("scan", input, output, 4, PrefixScanMode::Exclusive,),
            Err(GpuPrimitiveValidationError::InsufficientCapacity { .. })
        ));
    }

    #[test]
    fn gpu_primitives_prefix_scan_rejects_aliased_output() {
        let (flow, input) = RenderFlow::new("test.primitive.scan.alias")
            .storage_array::<U32ScanElement>("scan.input", 4);
        let _flow = flow;

        assert!(matches!(
            U32PrefixScanDescriptor::new(
                "scan",
                input.clone(),
                input,
                4,
                PrefixScanMode::Exclusive,
            ),
            Err(GpuPrimitiveValidationError::AliasedInputOutput { .. })
        ));
    }
}
