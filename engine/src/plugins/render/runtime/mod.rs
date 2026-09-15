pub mod debug_eval;
pub mod dynamic_targets;
pub mod dynamic_texture_uploads;
mod frame_diagnostics;
pub mod frame_prepare;
pub mod frame_submit;

use runen_ecs::SystemSet;

pub use dynamic_targets::*;
pub use dynamic_texture_uploads::*;

pub(crate) use frame_diagnostics::{
    CompletedRenderFrameDiagnostics, RenderFrameDiagnosticsSnapshot,
    RenderFrameDiagnosticsTransactionState,
};
pub(crate) use frame_prepare::frame_render_prepare_system;
pub(crate) use frame_submit::frame_render_submit_system;

#[derive(Debug, Copy, Clone, PartialEq, Eq, SystemSet)]
pub enum RenderRuntimeSet {
    GpuResidency,
    FramePrepare,
}
