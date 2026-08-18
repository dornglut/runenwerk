//! Future-transferable, backend-neutral RunenGPU contracts.

mod access;
mod capability;
mod context;
mod data;
mod dispatch;
mod errors;
mod graph;
mod handles;
mod pipeline_realization;
mod program;
mod realization;
mod render_execution;
mod render_pass;
mod render_pass_usage;
mod resource;
mod work;
mod work_resource_id;

pub use access::*;
pub use capability::*;
pub use context::*;
pub use data::*;
pub use dispatch::*;
pub use errors::*;
pub(crate) use errors::{GpuWorkAuthoringErrorContext, GpuWorkGraphErrorContext};
pub use graph::*;
pub use handles::*;
pub use pipeline_realization::*;
pub use program::*;
pub use realization::*;
pub use render_execution::*;
pub use render_pass::*;
pub use resource::*;
pub use work::*;
pub use work_resource_id::{
    GpuWorkResourceId, GpuWorkResourceIdAllocationError, GpuWorkResourceIdAllocator,
};
