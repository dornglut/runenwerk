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
