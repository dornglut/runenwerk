//! File: domain/editor/editor_shell/src/surfaces/viewport.rs
//! Purpose: Viewport surface workflow contracts.

use editor_viewport::{ExpressionProductId, ViewportDebugStage, ViewportId};
use ui_math::UiPoint;

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
    ToggleToolsMenu,
    ActivateSelectTool,
    ActivateTranslateTool,
    ActivateRotateTool,
    ActivateScaleTool,
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

#[derive(Debug, Clone, PartialEq)]
pub enum ViewportSessionMutation {
    ToggleDetails,
    ToggleStatistics,
    ToggleOptionsMenu,
    ToggleToolsMenu,
    OpenToolRadialMenu {
        viewport_id: ViewportId,
        anchor_position: UiPoint,
        opened_by_tab_hold: bool,
    },
    CloseToolRadialMenu,
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
