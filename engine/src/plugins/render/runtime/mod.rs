pub mod debug_eval;
pub mod dynamic_targets;
pub mod frame_prepare;
pub mod frame_submit;
pub mod ui_submission;

use crate::runtime::IntoSystemSetKey;
use scheduler::label::SystemSetKey;

pub use dynamic_targets::*;

pub(crate) use frame_prepare::frame_render_prepare_system;
pub(crate) use frame_submit::frame_render_submit_system;
pub(crate) use ui_submission::collect_runtime_ui_frame_submissions_system;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderRuntimeSet {
    GpuResidency,
    FramePrepare,
}

impl IntoSystemSetKey for RenderRuntimeSet {
    fn system_set_key(&self) -> SystemSetKey {
        match self {
            Self::GpuResidency => {
                SystemSetKey::of::<RenderRuntimeSet>("RenderRuntimeSet::GpuResidency")
            }
            Self::FramePrepare => {
                SystemSetKey::of::<RenderRuntimeSet>("RenderRuntimeSet::FramePrepare")
            }
        }
    }
}
