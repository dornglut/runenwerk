//! Public logical GPU-work identity contracts.

mod work_resource_id;

pub use work_resource_id::{
    GpuWorkResourceId, GpuWorkResourceIdAllocationError, GpuWorkResourceIdAllocator,
};
