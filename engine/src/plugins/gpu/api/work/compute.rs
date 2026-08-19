use super::super::{
    GpuComputePipelineDescriptor, GpuDispatchIntent, GpuQueryAccess, GpuQueryAccessKind,
    GpuRuntimeBindingSet, GpuWorkOperationCause, GpuWorkOperationError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuDispatchSize {
    x: u32,
    y: u32,
    z: u32,
}

impl GpuDispatchSize {
    pub const fn new(x: u32, y: u32, z: u32) -> Result<Self, GpuWorkOperationError> {
        Ok(Self { x, y, z })
    }

    pub const fn x(self) -> u32 {
        self.x
    }
    pub const fn y(self) -> u32 {
        self.y
    }
    pub const fn z(self) -> u32 {
        self.z
    }
    pub const fn as_array(self) -> [u32; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GpuComputeOperation {
    pipeline: GpuComputePipelineDescriptor,
    bindings: GpuRuntimeBindingSet,
    dispatch: GpuDispatchIntent,
    timestamp_writes: Vec<GpuQueryAccess>,
}

impl GpuComputeOperation {
    pub fn new(
        pipeline: GpuComputePipelineDescriptor,
        bindings: GpuRuntimeBindingSet,
        dispatch: GpuDispatchIntent,
    ) -> Result<Self, GpuWorkOperationError> {
        if pipeline.layout() != bindings.layout() {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU compute operation",
                "pipeline bindings",
                None,
                GpuWorkOperationCause::OperationAccessContradiction,
                "use a runtime binding set constructed for the exact compute pipeline layout",
            ));
        }
        Ok(Self {
            pipeline,
            bindings,
            dispatch,
            timestamp_writes: Vec::new(),
        })
    }

    pub fn with_timestamp_writes(
        mut self,
        timestamp_writes: impl IntoIterator<Item = GpuQueryAccess>,
    ) -> Result<Self, GpuWorkOperationError> {
        let timestamp_writes = timestamp_writes.into_iter().collect::<Vec<_>>();
        if timestamp_writes
            .iter()
            .any(|access| access.kind() != GpuQueryAccessKind::WriteTimestamp)
        {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU compute operation",
                "timestamp writes",
                timestamp_writes
                    .first()
                    .map(GpuQueryAccess::resource_identity),
                GpuWorkOperationCause::OperationAccessContradiction,
                "provide only WriteTimestamp query accesses as compute-side timestamp writes",
            ));
        }
        self.timestamp_writes = timestamp_writes;
        Ok(self)
    }

    pub fn pipeline(&self) -> &GpuComputePipelineDescriptor {
        &self.pipeline
    }

    pub fn bindings(&self) -> &GpuRuntimeBindingSet {
        &self.bindings
    }

    pub fn dispatch(&self) -> &GpuDispatchIntent {
        &self.dispatch
    }

    pub fn timestamp_writes(&self) -> &[GpuQueryAccess] {
        &self.timestamp_writes
    }
}
