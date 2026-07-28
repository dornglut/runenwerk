//! Future-transferable, backend-neutral RunenGPU contracts.

mod access;
mod capability;
mod data;
mod errors;
mod graph;
mod handles;
mod resource;
mod work;
mod work_resource_id;

pub use access::*;
pub use capability::*;
pub use data::*;
pub use errors::*;
pub(crate) use errors::{GpuWorkAuthoringErrorContext, GpuWorkGraphErrorContext};
pub use graph::*;
pub use handles::*;
pub use resource::*;
pub use work::*;
pub use work_resource_id::{
    GpuWorkResourceId, GpuWorkResourceIdAllocationError, GpuWorkResourceIdAllocator,
};
