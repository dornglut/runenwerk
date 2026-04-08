//! File: domain/ui/ui_runtime/src/input/outcome.rs
//! Purpose: Combined low-level dispatch response and semantic interaction results.

use crate::{UiInputDispatchResult, UiInteractionResults};

#[derive(Debug, Clone, PartialEq)]
pub struct UiInputOutcome {
    pub dispatch: UiInputDispatchResult,
    pub interactions: UiInteractionResults,
}

impl UiInputOutcome {
    pub fn ignored() -> Self {
        Self {
            dispatch: UiInputDispatchResult::ignored(),
            interactions: UiInteractionResults::new(),
        }
    }
}
