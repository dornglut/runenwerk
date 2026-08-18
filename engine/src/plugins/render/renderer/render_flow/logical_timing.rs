use super::*;
use crate::plugins::gpu::{
    GpuBufferHandle, GpuBufferUsage, GpuMemoryIntent, GpuQueryKind, GpuQueryRange,
    GpuQuerySetDescriptor, GpuQuerySetHandle, GpuResourceLifetime, GpuWorkResourceIdAllocator,
};
use crate::plugins::render::renderer::resource_descriptors::{buffer_descriptor, owned_common};

const QUERY_SIZE_BYTES: u64 = 8;

/// Logical timestamp resources and occurrence-local query ranges prepared before G3 work.
///
/// This value contains only backend-neutral RunenGPU handles. Physical query/buffer realization
/// remains in `GpuPassTimingFrame`, and resolve/readback operation construction remains part of the
/// late G5A operation projection.
#[derive(Debug, Clone)]
pub(super) struct LogicalGpuPassTiming {
    query_set: GpuQuerySetHandle,
    resolve_buffer: GpuBufferHandle,
    readback_buffer: GpuBufferHandle,
    query_capacity: u32,
}

impl LogicalGpuPassTiming {
    pub(super) fn new(timestampable_occurrences: usize) -> Result<Option<Self>> {
        if timestampable_occurrences == 0 {
            return Ok(None);
        }
        let query_capacity = timestampable_occurrences
            .checked_mul(2)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| anyhow::anyhow!("render GPU timestamp query capacity exceeds u32"))?;
        let byte_len = u64::from(query_capacity)
            .checked_mul(QUERY_SIZE_BYTES)
            .ok_or_else(|| anyhow::anyhow!("render GPU timestamp buffer size overflow"))?;

        let mut allocator = GpuWorkResourceIdAllocator::new();
        let query_set = allocator.allocate_query_set_handle(GpuQuerySetDescriptor::new(
            owned_common(
                "render.flow.timestamps",
                GpuResourceLifetime::Transient,
                GpuMemoryIntent::Device,
            )?,
            GpuQueryKind::Timestamp,
            query_capacity,
        )?)?;
        let resolve_buffer = allocator.allocate_buffer_handle(buffer_descriptor(
            "render.flow.timestamp_resolve",
            byte_len,
            [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
        )?)?;
        let readback_buffer = allocator.allocate_buffer_handle(buffer_descriptor(
            "render.flow.timestamp_readback",
            byte_len,
            [GpuBufferUsage::CopyDestination, GpuBufferUsage::Readback],
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Readback,
        )?)?;

        Ok(Some(Self {
            query_set,
            resolve_buffer,
            readback_buffer,
            query_capacity,
        }))
    }

    pub(super) fn query_set(&self) -> &GpuQuerySetHandle {
        &self.query_set
    }

    pub(super) fn resolve_buffer(&self) -> &GpuBufferHandle {
        &self.resolve_buffer
    }

    pub(super) fn readback_buffer(&self) -> &GpuBufferHandle {
        &self.readback_buffer
    }

    pub(super) const fn query_capacity(&self) -> u32 {
        self.query_capacity
    }

    pub(super) fn range_for_occurrence(&self, ordinal: usize) -> Result<GpuPassTimestampIndices> {
        let begin = ordinal
            .checked_mul(2)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| anyhow::anyhow!("render GPU timestamp occurrence index exceeds u32"))?;
        let end = begin
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("render GPU timestamp query index overflow"))?;
        if end >= self.query_capacity {
            anyhow::bail!(
                "render GPU timestamp occurrence {ordinal} exceeds query capacity {}",
                self.query_capacity
            );
        }
        Ok(GpuPassTimestampIndices { begin, end })
    }

    pub(super) fn query_range(&self) -> Result<GpuQueryRange> {
        Ok(GpuQueryRange::new(
            self.query_set(),
            0,
            self.query_capacity,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_timing_uses_two_queries_per_actual_occurrence() {
        let timing = LogicalGpuPassTiming::new(3)
            .expect("logical timing should construct")
            .expect("nonzero occurrence count should allocate timing");

        assert_eq!(timing.query_capacity(), 6);
        assert_eq!(
            timing.range_for_occurrence(0).unwrap(),
            GpuPassTimestampIndices { begin: 0, end: 1 }
        );
        assert_eq!(
            timing.range_for_occurrence(2).unwrap(),
            GpuPassTimestampIndices { begin: 4, end: 5 }
        );
        assert_eq!(timing.query_range().unwrap().count(), 6);
    }

    #[test]
    fn logical_timing_omits_zero_occurrence_resources() {
        assert!(LogicalGpuPassTiming::new(0).unwrap().is_none());
    }
}
