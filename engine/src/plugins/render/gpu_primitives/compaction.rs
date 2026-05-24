use super::{GpuPrimitiveValidationError, U32ScanElement, validate_capacity};
use crate::plugins::render::{RenderResourceId, StorageArrayHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct U32ScatterDescriptor {
    pub label: String,
    pub source_indices: RenderResourceId,
    pub prefix_offsets: RenderResourceId,
    pub output_indices: RenderResourceId,
    pub element_count: u32,
    pub output_capacity: u32,
}

impl U32ScatterDescriptor {
    pub fn new(
        label: impl Into<String>,
        source_indices: StorageArrayHandle<U32ScanElement>,
        prefix_offsets: StorageArrayHandle<U32ScanElement>,
        output_indices: StorageArrayHandle<U32ScanElement>,
        element_count: u32,
        output_capacity: u32,
    ) -> Result<Self, GpuPrimitiveValidationError> {
        let descriptor = Self {
            label: label.into(),
            source_indices: *source_indices.id(),
            prefix_offsets: *prefix_offsets.id(),
            output_indices: *output_indices.id(),
            element_count,
            output_capacity,
        };
        descriptor.validate()?;
        validate_capacity(
            format!("{}.source_indices", descriptor.label),
            source_indices.len(),
            u64::from(element_count),
        )?;
        validate_capacity(
            format!("{}.prefix_offsets", descriptor.label),
            prefix_offsets.len(),
            u64::from(element_count),
        )?;
        validate_capacity(
            format!("{}.output_indices", descriptor.label),
            output_indices.len(),
            u64::from(output_capacity),
        )?;
        Ok(descriptor)
    }

    pub fn validate(&self) -> Result<(), GpuPrimitiveValidationError> {
        if self.label.trim().is_empty() {
            return Err(GpuPrimitiveValidationError::EmptyLabel {
                primitive: "u32_scatter",
            });
        }
        if self.element_count == 0 {
            return Err(GpuPrimitiveValidationError::ZeroElementCount {
                label: self.label.clone(),
            });
        }
        if self.source_indices == self.output_indices
            || self.prefix_offsets == self.output_indices
            || self.source_indices == self.prefix_offsets
        {
            return Err(GpuPrimitiveValidationError::AliasedInputOutput {
                label: self.label.clone(),
            });
        }
        validate_capacity(
            self.label.clone(),
            u64::from(self.output_capacity),
            u64::from(self.element_count),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::RenderFlow;

    #[test]
    fn gpu_primitives_scatter_rejects_aliased_buffers() {
        let (flow, indices) =
            RenderFlow::new("test.primitive.scatter.alias").storage_array("scatter.indices", 4);
        let (flow, offsets) = flow.storage_array("scatter.offsets", 4);
        let _flow = flow;

        assert!(matches!(
            U32ScatterDescriptor::new("scatter", indices.clone(), offsets, indices, 4, 4,),
            Err(GpuPrimitiveValidationError::AliasedInputOutput { .. })
        ));
    }

    #[test]
    fn gpu_primitives_scatter_rejects_output_capacity_drift() {
        let (flow, indices) =
            RenderFlow::new("test.primitive.scatter.capacity").storage_array("scatter.indices", 4);
        let (flow, offsets) = flow.storage_array("scatter.offsets", 4);
        let (_flow, output) = flow.storage_array("scatter.output", 3);

        assert!(matches!(
            U32ScatterDescriptor::new("scatter", indices, offsets, output, 4, 4),
            Err(GpuPrimitiveValidationError::InsufficientCapacity { .. })
        ));
    }
}
