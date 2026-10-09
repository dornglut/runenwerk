//! Engine-owned integration boundary between Runenwerk UI source/product semantics and RunenUI.
//!
//! Ordinary mounted UI execution is owned by standalone RunenUI runtime slots.
//! Legacy evaluator/frame-publication types remain only for explicitly isolated unmigrated
//! renderer consumers until their own clean cut.

pub mod action;
pub mod app_ext;
pub mod diagnostics;
pub mod events;
pub mod host;
pub mod mount;
pub mod plugin;
pub mod presentation;
pub mod publish;
#[cfg(test)]
pub(crate) mod render_scene;
pub mod report;
pub mod resources;
mod runenui_adapter;
pub mod schedule;
pub mod screen;
pub mod source;
pub mod trace;

pub use action::*;
pub use app_ext::*;
pub use diagnostics::*;
pub use events::*;
pub use host::*;
pub use mount::*;
pub use plugin::UiPlugin;
pub use presentation::*;
pub use publish::*;
pub use report::*;
pub use resources::*;
pub use runenui_adapter::{
    UiRuntimeFontConfiguration, UiRuntimeHostRequestDisposition, UiRuntimePendingEventRequest,
    UiRuntimeSlotId,
    UiRuntimeSlotMountFailure, UiRuntimeSlotMountReport, UiRuntimeSlotOperationFailure,
    UiRuntimeSlotsResource,
};
pub use schedule::*;
pub use screen::*;
pub use source::*;
pub use trace::*;
