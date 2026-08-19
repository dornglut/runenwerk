use super::super::{
    GpuAccessCause, GpuBufferAccess, GpuBufferAccessKind, GpuBufferHandle, GpuBufferRange,
    GpuQueryAccess, GpuQueryAccessKind, GpuQueryKind, GpuQueryRange, GpuQuerySetHandle,
    GpuWorkOperationCause, GpuWorkOperationError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuQueryResolveOperation {
    source: GpuQuerySetHandle,
    source_range: GpuQueryRange,
    destination: GpuBufferHandle,
    destination_offset: u64,
    destination_range: GpuBufferRange,
    source_access: GpuQueryAccess,
    destination_access: GpuBufferAccess,
}

impl GpuQueryResolveOperation {
    pub fn new(
        source: &GpuQuerySetHandle,
        source_range: GpuQueryRange,
        destination: &GpuBufferHandle,
        destination_offset: u64,
    ) -> Result<Self, GpuWorkOperationError> {
        if source.descriptor().kind() != GpuQueryKind::Timestamp {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU query resolve operation",
                source.descriptor().common().label().as_str(),
                Some(source.diagnostic_identity()),
                GpuWorkOperationCause::InvalidQueryResolution,
                "use a timestamp query set for the current G3 resolve operation",
            ));
        }
        let byte_len = u64::from(source_range.count())
            .checked_mul(8)
            .ok_or_else(|| {
                GpuWorkOperationError::invalid(
                    "construct GPU query resolve operation",
                    source.descriptor().common().label().as_str(),
                    Some(source.diagnostic_identity()),
                    GpuWorkOperationCause::QueryDestinationOverflow,
                    "reduce the query count",
                )
            })?;
        let destination_range = GpuBufferRange::new(destination, destination_offset, byte_len)
            .map_err(|source| {
                let cause = match source.cause() {
                    GpuAccessCause::ArithmeticOverflow => GpuWorkOperationCause::QueryDestinationOverflow,
                    _ => GpuWorkOperationCause::QueryDestinationOutOfBounds,
                };
                GpuWorkOperationError::from_access(
                    "construct GPU query resolve destination",
                    destination.descriptor().common().label().as_str(),
                    cause,
                    "keep count-times-eight bytes at the destination offset inside the buffer",
                    source,
                )
            })?;
        let source_access =
            GpuQueryAccess::new(source, source_range, GpuQueryAccessKind::ResolveSource).map_err(
                |source| {
                    GpuWorkOperationError::from_access(
                        "construct GPU query resolve source",
                        "query resolve",
                        GpuWorkOperationCause::InvalidQueryRange,
                        "provide a checked query range",
                        source,
                    )
                },
            )?;
        let destination_access = GpuBufferAccess::new(
            destination,
            destination_range,
            GpuBufferAccessKind::QueryResolveDestination,
        )
        .map_err(|source| {
            GpuWorkOperationError::from_access(
                "construct GPU query resolve destination",
                destination.descriptor().common().label().as_str(),
                GpuWorkOperationCause::InvalidQueryResolution,
                "declare QueryResolve usage on the destination buffer",
                source,
            )
        })?;
        Ok(Self {
            source: source.clone(),
            source_range,
            destination: destination.clone(),
            destination_offset,
            destination_range,
            source_access,
            destination_access,
        })
    }

    pub fn source(&self) -> &GpuQuerySetHandle {
        &self.source
    }
    pub const fn source_range(&self) -> GpuQueryRange {
        self.source_range
    }
    pub fn destination(&self) -> &GpuBufferHandle {
        &self.destination
    }
    pub const fn destination_offset(&self) -> u64 {
        self.destination_offset
    }
    pub const fn destination_range(&self) -> GpuBufferRange {
        self.destination_range
    }
    pub fn source_access(&self) -> &GpuQueryAccess {
        &self.source_access
    }
    pub fn destination_access(&self) -> &GpuBufferAccess {
        &self.destination_access
    }
}
