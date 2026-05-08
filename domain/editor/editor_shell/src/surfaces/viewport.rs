//! File: domain/editor/editor_shell/src/surfaces/viewport.rs
//! Purpose: Viewport surface workflow contracts.

use editor_viewport::{ExpressionProductId, ViewportDebugStage, ViewportId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportSurfaceAction {
    SelectProduct {
        viewport_id: ViewportId,
        product_id: ExpressionProductId,
        enabled: bool,
    },
    ToggleDetails,
    ToggleStatistics,
    ToggleOptionsMenu,
    ResetCamera {
        viewport_id: ViewportId,
    },
    SetDebugStage {
        viewport_id: ViewportId,
        debug_stage: ViewportDebugStage,
    },
    SetRootBackgroundOpaque {
        viewport_id: ViewportId,
        enabled: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportSessionMutation {
    ToggleDetails,
    ToggleStatistics,
    ToggleOptionsMenu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportDomainMutation {
    SelectProduct {
        viewport_id: ViewportId,
        product_id: ExpressionProductId,
    },
    ResetCamera {
        viewport_id: ViewportId,
    },
    SetDebugStage {
        viewport_id: ViewportId,
        debug_stage: ViewportDebugStage,
    },
    SetRootBackgroundOpaque {
        viewport_id: ViewportId,
        enabled: bool,
    },
}
