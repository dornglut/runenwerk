//! Overlay report evidence types.

use ui_math::UiRect;

use crate::WidgetId;

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayLayeringReport {
    pub replay_id: String,
    pub fixture_id: String,
    pub input_steps: Vec<String>,
    pub declarations: Vec<OverlayDeclarationEvidence>,
    pub open_intents: Vec<OverlayOpenIntent>,
    pub stack_entries: Vec<OverlayStackEntry>,
    pub placement_resolutions: Vec<OverlayPlacementResolution>,
    pub layer_assignments: Vec<OverlayLayerAssignment>,
    pub focus_evidence: Vec<OverlayFocusEvidence>,
    pub dismissal_evidence: Vec<OverlayDismissalEvidence>,
    pub pointer_capture_evidence: Vec<OverlayPointerCaptureEvidence>,
    pub keyboard_navigation_evidence: Vec<OverlayKeyboardNavigationEvidence>,
    pub suppression_evidence: Vec<OverlaySuppressionEvidence>,
    pub viewport_evidence: Vec<OverlayViewportEvidence>,
    pub boundary_assertions: OverlayBoundaryAssertions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayDeclarationEvidence {
    pub source_control_id: WidgetId,
    pub anchor_id: String,
    pub overlay_kind: String,
    pub trigger: String,
    pub layer: String,
    pub dismiss_policy: String,
    pub focus_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayOpenIntent {
    pub input_sample_id: String,
    pub step_id: String,
    pub source_control_id: WidgetId,
    pub anchor_id: String,
    pub request_id: String,
    pub overlay_kind: String,
    pub trigger: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayStackEntry {
    pub stack_entry_id: String,
    pub request_id: String,
    pub parent_request_id: Option<String>,
    pub scope_id: String,
    pub anchor_id: String,
    pub kind: String,
    pub layer_class: String,
    pub opened_at_step: String,
    pub closed_at_step: Option<String>,
    pub hit_regions: Vec<UiRect>,
    pub is_topmost_dismissible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayPlacementResolution {
    pub anchor_id: String,
    pub request_id: String,
    pub anchor_rect: UiRect,
    pub requested_side: String,
    pub requested_alignment: String,
    pub resolved_side: String,
    pub resolved_alignment: String,
    pub resolved_rect: UiRect,
    pub viewport_rect: UiRect,
    pub clamped: bool,
    pub shifted: bool,
    pub hidden_or_suppressed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayLayerAssignment {
    pub request_id: String,
    pub layer_id: String,
    pub layer_class: String,
    pub ordinal: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayFocusEvidence {
    pub request_id: String,
    pub focus_policy: String,
    pub focus_return_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayPointerCaptureEvidence {
    pub input_sample_id: String,
    pub anchor_id: String,
    pub outside_dismissal_blocked: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayKeyboardNavigationEvidence {
    pub input_sample_id: String,
    pub request_id: Option<String>,
    pub key: String,
    pub navigation_intent: String,
    pub product_commands_executed: u32,
}
