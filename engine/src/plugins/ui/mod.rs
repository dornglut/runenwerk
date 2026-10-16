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
pub mod native_input;
pub mod native_transaction;
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
mod typed_app_host;

pub use action::*;
pub use app_ext::*;
pub use diagnostics::*;
pub use events::*;
pub use host::*;
pub use mount::*;
pub use native_input::*;
pub use native_transaction::transact_ui_slot;
pub use plugin::UiPlugin;
pub use presentation::*;
pub use publish::*;
pub use report::*;
pub use resources::*;
pub use runenui_adapter::{
    UiRuntimeFontConfiguration, UiRuntimeHostRequestDisposition,
    UiRuntimeNativeKeyboardAdmissionError, UiRuntimeNativeKeyboardEvidenceFailure,
    UiRuntimeNativePointerAdmissionError, UiRuntimeNativePointerEvidenceFailure,
    UiRuntimePendingEventRequest, UiRuntimeSlotId, UiRuntimeSlotMountFailure,
    UiRuntimeSlotMountReport, UiRuntimeSlotOperationFailure, UiRuntimeSlotsResource,
    native_keyboard_finality, native_no_focused_keyboard_claim_witness, native_pointer_finality,
};
pub(crate) use runenui_adapter::{
    UiRuntimeNativeCompositionAdmissionError, UiRuntimeNativeTextAdmissionError,
    native_composition_finality, native_original_keyboard_release_nonclaim,
    native_original_keyboard_repeat_nonclaim, native_text_finality,
};
pub use schedule::*;
pub use screen::*;
pub use source::*;
pub use trace::*;
pub use typed_app_host::UiAppObservation;
