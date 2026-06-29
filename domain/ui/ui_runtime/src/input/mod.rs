//! File: domain/ui/ui_runtime/src/input/mod.rs
//! Purpose: Runtime-facing input dispatch contracts.

pub mod dispatch;
pub mod generic_interaction;
pub mod generic_interaction_fixture;
pub mod generic_interaction_visual_frame;
pub mod hit_test;
pub mod interaction_result;
pub mod outcome;
pub mod pointer;

pub use dispatch::UiInputDispatchResult;
pub use generic_interaction::*;
pub use generic_interaction_fixture::*;
pub use generic_interaction_visual_frame::*;
pub use hit_test::hit_test_widget;
pub use interaction_result::{UiInteraction, UiInteractionResults};
pub use outcome::{UiInputOutcome, UiInvalidation};
pub use pointer::dispatch_pointer_event;
