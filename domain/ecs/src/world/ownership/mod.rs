mod model;
mod registry;
mod routing;

pub use model::{
    ControllerId, ControllerRole, OwnerState, OwnershipTarget, OwnershipTransferRecord,
    ResourceOwnerKey, ResourceOwnershipDescriptor,
};
pub(super) use registry::OwnershipRegistry;
