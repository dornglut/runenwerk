use crate::plugins::gpu::GpuWorkResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAccess {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceUsageKind {
    Sampled,
    Storage,
    ColorTarget,
    DepthTarget,
    Vertex,
    Index,
    Instance,
    Indirect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceUsage {
    pub resource_id: GpuWorkResourceId,
    pub access: ResourceAccess,
    pub kind: ResourceUsageKind,
}

impl ResourceUsage {
    pub fn new(
        resource_id: impl Into<GpuWorkResourceId>,
        access: ResourceAccess,
        kind: ResourceUsageKind,
    ) -> Self {
        Self {
            resource_id: resource_id.into(),
            access,
            kind,
        }
    }
}
