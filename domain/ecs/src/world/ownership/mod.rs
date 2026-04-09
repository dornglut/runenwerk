mod model;
mod registry;
mod routing;

pub use model::{
    OwnerId, OwnerRole, OwnerState, OwnershipTarget, OwnershipTransferRecord,
    ResourceOwnerKey, ResourceOwnershipDescriptor,
};
pub(super) use registry::OwnershipRegistry;
