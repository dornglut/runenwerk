//! Future-transferable, backend-neutral RunenGPU contracts.

mod access;
mod capability;
mod context;
mod copy_compatibility;
mod data;
mod dispatch;
mod errors;
mod execution;
mod graph;
mod handles;
mod operation;
mod pipeline_realization;
mod program;
mod readback_id;
mod realization;
mod render_execution;
mod render_pass;
mod render_pass_usage;
mod resource;
mod surface;
mod surface_acquisition;
mod transfer;
mod work;
mod work_resource_id;

pub use access::*;
pub use capability::*;
pub use context::*;
pub use copy_compatibility::*;
pub use data::*;
pub use dispatch::*;
pub use errors::*;
pub(crate) use errors::{GpuWorkAuthoringErrorContext, GpuWorkGraphErrorContext};
pub use execution::*;
pub use graph::*;
pub(crate) use graph::GpuPreparedInitialContent;
pub use handles::*;
pub use operation::{GpuRenderOperation, GpuWorkNodeKind, GpuWorkOperation};
pub use pipeline_realization::*;
pub use program::*;
pub use readback_id::*;
pub use realization::*;
pub use render_execution::*;
pub use render_pass::*;
pub use resource::*;
pub use surface::*;
pub use surface_acquisition::*;
pub use transfer::*;
pub use work::{
    GpuBufferRegion, GpuBufferTextureLayout, GpuClearOperation, GpuColorAttachmentLoad,
    GpuColorClearValue, GpuComputeOperation, GpuCopyExtent, GpuCopyOperation,
    GpuDepthAttachmentLoad, GpuDepthClearValue, GpuDispatchSize, GpuDrawIntent, GpuDrawRange,
    GpuMultisampleResolveTarget, GpuPresentOperation, GpuQueryResolveOperation,
    GpuRenderColorAttachment, GpuRenderDepthStencilAttachment, GpuTextureCopyRegion,
    GpuTextureOrigin, GpuTimestampWrites,
};
pub use work_resource_id::{
    GpuWorkResourceId, GpuWorkResourceIdAllocationError, GpuWorkResourceIdAllocator,
};
