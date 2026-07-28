use super::super::GpuWorkResourceId;
use super::identity::GpuPreparedWorkNodeId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuDependencyReason {
    ReadAfterWrite { resource: GpuWorkResourceId },
    WriteAfterRead { resource: GpuWorkResourceId },
    WriteAfterWrite { resource: GpuWorkResourceId },
    ExplicitNonData { reason: String },
}

impl GpuDependencyReason {
    pub const fn resource(&self) -> Option<GpuWorkResourceId> {
        match self {
            Self::ReadAfterWrite { resource }
            | Self::WriteAfterRead { resource }
            | Self::WriteAfterWrite { resource } => Some(*resource),
            Self::ExplicitNonData { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuWorkDependency {
    pub(super) before: GpuPreparedWorkNodeId,
    pub(super) after: GpuPreparedWorkNodeId,
    pub(super) reasons: Vec<GpuDependencyReason>,
}

impl GpuWorkDependency {
    pub const fn before(&self) -> GpuPreparedWorkNodeId {
        self.before
    }

    pub const fn after(&self) -> GpuPreparedWorkNodeId {
        self.after
    }

    pub fn reasons(&self) -> &[GpuDependencyReason] {
        &self.reasons
    }
}
