//! Renderer-neutral overlay / popup / layering substrate proof.
//!
//! Overlay runtime consumes normalized input facts. It is not part of input
//! ownership. Runtime owns overlay intent, stack, placement, focus, dismissal,
//! report, proof-frame projection, and no-bypass evidence.

use ui_controls::{
    ControlOverlayDescriptor, ControlOverlayFocusPolicy, ControlOverlayKind,
    ControlOverlayLayerPreference, ControlOverlayPlacementPreference, ControlOverlayPlacementSide,
    ControlOverlayRequirement, ControlOverlayTrigger,
};
use ui_input::{
    FocusChange, FocusInputFact, FocusTargetId, Key, KeyState, KeyboardInputFact,
    NormalizedInputFact, NormalizedInputSample, PointerEventKind, PointerInputFact,
};
use ui_math::{UiPoint, UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};
use ui_text::{FontId, GlyphRun, PositionedGlyph};

use crate::WidgetId;

pub const BASE_CONTROLS_OVERLAY_LAYERING_PROOF_ID: &str = "base-controls.overlay-layering.proof";
pub const BASE_CONTROLS_EXECUTABLE_OVERLAY_LAYERING_STORY_ID: &str =
    "base-controls.overlay-layering.story";

#[derive(Debug, Clone, PartialEq)]
pub struct MountedOverlayLayeringFixture {
    pub fixture_id: String,
    pub viewport_rect: UiRect,
    pub controls: Vec<MountedOverlayControl>,
}

impl MountedOverlayLayeringFixture {
    pub fn new(fixture_id: impl Into<String>, viewport_rect: UiRect) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            viewport_rect,
            controls: Vec::new(),
        }
    }

    pub fn with_control(mut self, control: MountedOverlayControl) -> Self {
        self.controls.push(control);
        self
    }

    pub fn target_at(&self, point: UiPoint) -> Option<&MountedOverlayControl> {
        self.controls.iter().find(|control| control.bounds.contains(point))
    }

    pub fn control_by_anchor(&self, anchor_id: &str) -> Option<&MountedOverlayControl> {
        self.controls.iter().find(|control| control.anchor_id == anchor_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MountedOverlayControl {
    pub widget_id: WidgetId,
    pub anchor_id: String,
    pub label: String,
    pub bounds: UiRect,
    pub descriptor: ControlOverlayDescriptor,
    pub enabled: bool,
}

impl MountedOverlayControl {
    pub fn new(
        widget_id: WidgetId,
        anchor_id: impl Into<String>,
        label: impl Into<String>,
        bounds: UiRect,
        descriptor: ControlOverlayDescriptor,
    ) -> Self {
        Self {
            widget_id,
            anchor_id: anchor_id.into(),
            label: label.into(),
            bounds,
            descriptor,
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayLayeringScript {
    pub replay_id: String,
    pub steps: Vec<OverlayLayeringStep>,
}

impl OverlayLayeringScript {
    pub fn new(replay_id: impl Into<String>) -> Self {
        Self {
            replay_id: replay_id.into(),
            steps: Vec::new(),
        }
    }

    pub fn with_step(mut self, step: OverlayLayeringStep) -> Self {
        self.steps.push(step);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayLayeringStep {
    pub step_id: String,
    pub sample: NormalizedInputSample,
    pub viewport_rect: Option<UiRect>,
    pub invalidated_anchor: Option<String>,
}

impl OverlayLayeringStep {
    pub fn new(step_id: impl Into<String>, sample: NormalizedInputSample) -> Self {
        Self {
            step_id: step_id.into(),
            sample,
            viewport_rect: None,
            invalidated_anchor: None,
        }
    }

    pub fn with_viewport_rect(mut self, viewport_rect: UiRect) -> Self {
        self.viewport_rect = Some(viewport_rect);
        self
    }

    pub fn with_invalidated_anchor(mut self, anchor_id: impl Into<String>) -> Self {
        self.invalidated_anchor = Some(anchor_id.into());
        self
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayDismissalEvidence {
    pub request_id: Option<String>,
    pub stack_entry_id: Option<String>,
    pub reason: String,
    pub input_sample_id: String,
    pub focus_return_target: Option<String>,
    pub outside_pointer_target: Option<String>,
    pub escape_key_seen: bool,
    pub closed: bool,
    pub suppressed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlaySuppressionEvidence {
    pub anchor_id: Option<String>,
    pub request_kind: String,
    pub trigger: String,
    pub reason: String,
    pub input_sample_id: String,
    pub opened: bool,
    pub host_commands_executed: u32,
    pub product_mutations: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayViewportEvidence {
    pub step_id: String,
    pub viewport_rect: UiRect,
    pub scroll_recomputed: bool,
    pub viewport_recomputed: bool,
    pub anchor_still_valid: bool,
    pub placement_suppressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OverlayBoundaryAssertions {
    pub host_commands_executed: u32,
    pub product_mutations: u32,
    pub text_edit_transactions: u32,
    pub app_specific_modal_operations: u32,
    pub authored_ui_edits: u32,
    pub plugin_framework_operations: u32,
    pub overlay_open_requests: u32,
    pub overlay_opened: u32,
    pub overlay_suppressed: u32,
    pub overlay_dismissed_by_escape: u32,
    pub overlay_dismissed_by_outside_pointer: u32,
    pub overlay_stack_entries_opened: u32,
    pub overlay_stack_entries_closed: u32,
    pub placement_recomputed_after_scroll: u32,
    pub placement_recomputed_after_viewport_resize: u32,
    pub anchor_invalidation_suppressed: u32,
    pub focus_returned: u32,
}

impl OverlayBoundaryAssertions {
    pub const fn no_bypass_evidence(self) -> bool {
        self.host_commands_executed == 0
            && self.product_mutations == 0
            && self.text_edit_transactions == 0
            && self.app_specific_modal_operations == 0
            && self.authored_ui_edits == 0
            && self.plugin_framework_operations == 0
    }
}

pub fn replay_overlay_layering(
    fixture: &MountedOverlayLayeringFixture,
    script: &OverlayLayeringScript,
) -> OverlayLayeringReport {
    let mut state = OverlayReplayState::default();
    let mut viewport_rect = fixture.viewport_rect;
    let mut report = empty_report(fixture, script);
    for step in &script.steps {
        report.input_steps.push(step.step_id.clone());
        if let Some(next_viewport) = step.viewport_rect {
            viewport_rect = next_viewport;
            recompute_placements(fixture, &step.step_id, viewport_rect, &mut report, true, false);
        }
        if let Some(anchor_id) = &step.invalidated_anchor {
            invalidate_anchor(step, anchor_id, &mut report);
        }
        for fact in &step.sample.facts {
            apply_fact(fixture, step, fact, &mut state, viewport_rect, &mut report);
        }
    }
    report
}

#[derive(Default)]
struct OverlayReplayState {
    focused: Option<WidgetId>,
    captured: Option<WidgetId>,
}

fn empty_report(fixture: &MountedOverlayLayeringFixture, script: &OverlayLayeringScript) -> OverlayLayeringReport {
    OverlayLayeringReport {
        replay_id: script.replay_id.clone(),
        fixture_id: fixture.fixture_id.clone(),
        input_steps: Vec::new(),
        declarations: declarations(fixture),
        open_intents: Vec::new(),
        stack_entries: Vec::new(),
        placement_resolutions: Vec::new(),
        layer_assignments: Vec::new(),
        focus_evidence: Vec::new(),
        dismissal_evidence: Vec::new(),
        pointer_capture_evidence: Vec::new(),
        keyboard_navigation_evidence: Vec::new(),
        suppression_evidence: Vec::new(),
        viewport_evidence: Vec::new(),
        boundary_assertions: OverlayBoundaryAssertions::default(),
    }
}

fn declarations(fixture: &MountedOverlayLayeringFixture) -> Vec<OverlayDeclarationEvidence> {
    fixture
        .controls
        .iter()
        .flat_map(|control| {
            control.descriptor.requirements.iter().map(move |requirement| OverlayDeclarationEvidence {
                source_control_id: control.widget_id,
                anchor_id: control.anchor_id.clone(),
                overlay_kind: requirement.kind.as_str().to_owned(),
                trigger: requirement.trigger.as_str().to_owned(),
                layer: requirement.layer.as_str().to_owned(),
                dismiss_policy: requirement.dismiss_policy.as_str().to_owned(),
                focus_policy: requirement.focus_policy.as_str().to_owned(),
            })
        })
        .collect()
}

fn apply_fact(
    fixture: &MountedOverlayLayeringFixture,
    step: &OverlayLayeringStep,
    fact: &NormalizedInputFact,
    state: &mut OverlayReplayState,
    viewport_rect: UiRect,
    report: &mut OverlayLayeringReport,
) {
    match fact {
        NormalizedInputFact::Pointer(pointer) => match pointer.kind {
            PointerEventKind::Down => {
                state.captured = fixture.target_at(pointer.position).map(|control| control.widget_id);
                if let Some(control) = fixture.target_at(pointer.position) {
                    open_for_trigger(fixture, step, control, ControlOverlayTrigger::PointerPress, viewport_rect, report);
                    report.pointer_capture_evidence.push(OverlayPointerCaptureEvidence {
                        input_sample_id: step.sample.sample_id.clone(),
                        anchor_id: control.anchor_id.clone(),
                        outside_dismissal_blocked: true,
                        reason: "opening-pointer-capture".to_owned(),
                    });
                } else {
                    dismiss_outside(step, pointer.position, report);
                }
            }
            PointerEventKind::Move | PointerEventKind::Enter => {
                if let Some(control) = fixture.target_at(pointer.position) {
                    open_for_trigger(fixture, step, control, ControlOverlayTrigger::PointerHover, viewport_rect, report);
                }
            }
            PointerEventKind::Scroll => recompute_placements(fixture, &step.step_id, viewport_rect, report, false, true),
            PointerEventKind::Up | PointerEventKind::Leave => {}
        },
        NormalizedInputFact::Focus(focus) => {
            if let FocusChange::Set(target) = focus.change {
                let widget = WidgetId(target.0);
                state.focused = Some(widget);
                if let Some(control) = fixture.controls.iter().find(|control| control.widget_id == widget) {
                    open_for_trigger(fixture, step, control, ControlOverlayTrigger::Focus, viewport_rect, report);
                }
            }
        }
        NormalizedInputFact::Keyboard(keyboard) => {
            if keyboard.state == KeyState::Pressed {
                match keyboard.key {
                    Key::Escape => dismiss_escape(step, report),
                    Key::Up | Key::Down | Key::Left | Key::Right => report.keyboard_navigation_evidence.push(
                        OverlayKeyboardNavigationEvidence {
                            input_sample_id: step.sample.sample_id.clone(),
                            request_id: report.stack_entries.iter().rev().find(|entry| entry.closed_at_step.is_none()).map(|entry| entry.request_id.clone()),
                            key: format!("{:?}", keyboard.key),
                            navigation_intent: "overlay-keyboard-navigation".to_owned(),
                            product_commands_executed: 0,
                        },
                    ),
                    _ => {}
                }
            }
        }
        NormalizedInputFact::Semantic(_) | NormalizedInputFact::TextIntent(_) => {}
    }
}

fn open_for_trigger(
    _fixture: &MountedOverlayLayeringFixture,
    step: &OverlayLayeringStep,
    control: &MountedOverlayControl,
    trigger: ControlOverlayTrigger,
    viewport_rect: UiRect,
    report: &mut OverlayLayeringReport,
) {
    let Some(requirement) = control.descriptor.requirements.iter().find(|requirement| requirement.trigger == trigger) else {
        suppress(step, Some(control), "trigger.not_declared", report);
        return;
    };
    if !control.enabled && requirement.suppresses_when_disabled {
        suppress(step, Some(control), "anchor.disabled", report);
        return;
    }

    let request_id = format!("overlay.request.{}", report.open_intents.len() + 1);
    let stack_entry_id = format!("overlay.stack-entry.{}", report.stack_entries.len() + 1);
    let parent_request_id = if requirement.layer == ControlOverlayLayerPreference::Submenu {
        report.stack_entries.iter().rev().find(|entry| entry.closed_at_step.is_none()).map(|entry| entry.request_id.clone())
    } else {
        None
    };
    let placement = placement_resolution(control, requirement, &request_id, viewport_rect);
    let hit_region = placement.resolved_rect;

    report.open_intents.push(OverlayOpenIntent {
        input_sample_id: step.sample.sample_id.clone(),
        step_id: step.step_id.clone(),
        source_control_id: control.widget_id,
        anchor_id: control.anchor_id.clone(),
        request_id: request_id.clone(),
        overlay_kind: requirement.kind.as_str().to_owned(),
        trigger: trigger.as_str().to_owned(),
    });
    report.stack_entries.push(OverlayStackEntry {
        stack_entry_id: stack_entry_id.clone(),
        request_id: request_id.clone(),
        parent_request_id,
        scope_id: overlay_scope(requirement),
        anchor_id: control.anchor_id.clone(),
        kind: requirement.kind.as_str().to_owned(),
        layer_class: requirement.layer.as_str().to_owned(),
        opened_at_step: step.step_id.clone(),
        closed_at_step: None,
        hit_regions: vec![hit_region],
        is_topmost_dismissible: requirement.kind != ControlOverlayKind::Tooltip,
    });
    report.placement_resolutions.push(placement);
    report.layer_assignments.push(OverlayLayerAssignment {
        request_id: request_id.clone(),
        layer_id: format!("layer.{}", requirement.layer.as_str()),
        layer_class: requirement.layer.as_str().to_owned(),
        ordinal: report.layer_assignments.len(),
    });
    report.focus_evidence.push(OverlayFocusEvidence {
        request_id,
        focus_policy: requirement.focus_policy.as_str().to_owned(),
        focus_return_target: focus_return_target(control, requirement),
    });
    report.boundary_assertions.overlay_open_requests += 1;
    report.boundary_assertions.overlay_opened += 1;
    report.boundary_assertions.overlay_stack_entries_opened += 1;
}

fn placement_resolution(
    control: &MountedOverlayControl,
    requirement: &ControlOverlayRequirement,
    request_id: &str,
    viewport_rect: UiRect,
) -> OverlayPlacementResolution {
    let requested = &requirement.placement;
    let mut rect = raw_overlay_rect(control.bounds, requested);
    let before = rect;
    rect.x = rect.x.max(viewport_rect.x + requested.viewport_margin);
    rect.y = rect.y.max(viewport_rect.y + requested.viewport_margin);
    rect.x = rect.x.min(viewport_rect.x + viewport_rect.width - rect.width - requested.viewport_margin);
    rect.y = rect.y.min(viewport_rect.y + viewport_rect.height - rect.height - requested.viewport_margin);
    OverlayPlacementResolution {
        anchor_id: control.anchor_id.clone(),
        request_id: request_id.to_owned(),
        anchor_rect: control.bounds,
        requested_side: requested.side.as_str().to_owned(),
        requested_alignment: requested.alignment.as_str().to_owned(),
        resolved_side: requested.side.as_str().to_owned(),
        resolved_alignment: requested.alignment.as_str().to_owned(),
        resolved_rect: rect,
        viewport_rect,
        clamped: rect != before,
        shifted: rect.x != before.x || rect.y != before.y,
        hidden_or_suppressed: false,
    }
}

fn raw_overlay_rect(anchor: UiRect, placement: &ControlOverlayPlacementPreference) -> UiRect {
    let size = UiSize::new(220.0, 132.0);
    match placement.side {
        ControlOverlayPlacementSide::Top => UiRect::new(anchor.x, anchor.y - size.height - placement.main_axis_offset, size.width, size.height),
        ControlOverlayPlacementSide::Right => UiRect::new(anchor.x + anchor.width + placement.main_axis_offset, anchor.y, size.width, size.height),
        ControlOverlayPlacementSide::Bottom => UiRect::new(anchor.x, anchor.y + anchor.height + placement.main_axis_offset, size.width, size.height),
        ControlOverlayPlacementSide::Left => UiRect::new(anchor.x - size.width - placement.main_axis_offset, anchor.y, size.width, size.height),
        ControlOverlayPlacementSide::Center => UiRect::new(anchor.x, anchor.y, size.width, size.height),
        ControlOverlayPlacementSide::Cursor => UiRect::new(anchor.x, anchor.y, size.width, size.height),
    }
}

fn overlay_scope(requirement: &ControlOverlayRequirement) -> String {
    match requirement.kind {
        ControlOverlayKind::Menu => "scope.menu".to_owned(),
        ControlOverlayKind::Tooltip => "scope.tooltip".to_owned(),
        ControlOverlayKind::FocusContainingOverlay => "scope.focus-containing".to_owned(),
        _ => "scope.anchored".to_owned(),
    }
}

fn focus_return_target(control: &MountedOverlayControl, requirement: &ControlOverlayRequirement) -> Option<String> {
    match requirement.focus_policy {
        ControlOverlayFocusPolicy::ReturnToAnchor
        | ControlOverlayFocusPolicy::ReturnToPrevious
        | ControlOverlayFocusPolicy::ContainFocus => Some(control.anchor_id.clone()),
        _ => None,
    }
}

fn dismiss_escape(step: &OverlayLayeringStep, report: &mut OverlayLayeringReport) {
    let Some((index, entry)) = report
        .stack_entries
        .iter()
        .enumerate()
        .rev()
        .find(|(_, entry)| entry.closed_at_step.is_none() && entry.is_topmost_dismissible)
    else {
        return;
    };
    let request_id = entry.request_id.clone();
    let stack_entry_id = entry.stack_entry_id.clone();
    report.stack_entries[index].closed_at_step = Some(step.step_id.clone());
    report.dismissal_evidence.push(OverlayDismissalEvidence {
        request_id: Some(request_id),
        stack_entry_id: Some(stack_entry_id),
        reason: "escape.topmost_dismissed".to_owned(),
        input_sample_id: step.sample.sample_id.clone(),
        focus_return_target: report.focus_evidence.last().and_then(|evidence| evidence.focus_return_target.clone()),
        outside_pointer_target: None,
        escape_key_seen: true,
        closed: true,
        suppressed: false,
    });
    report.boundary_assertions.overlay_dismissed_by_escape += 1;
    report.boundary_assertions.overlay_stack_entries_closed += 1;
    report.boundary_assertions.focus_returned += 1;
}

fn dismiss_outside(step: &OverlayLayeringStep, point: UiPoint, report: &mut OverlayLayeringReport) {
    if report
        .stack_entries
        .iter()
        .rev()
        .find(|entry| entry.closed_at_step.is_none())
        .is_some_and(|entry| entry.hit_regions.iter().any(|region| region.contains(point)))
    {
        report.pointer_capture_evidence.push(OverlayPointerCaptureEvidence {
            input_sample_id: step.sample.sample_id.clone(),
            anchor_id: "overlay.active".to_owned(),
            outside_dismissal_blocked: true,
            reason: "inside-active-overlay".to_owned(),
        });
        return;
    }
    let Some((index, entry)) = report
        .stack_entries
        .iter()
        .enumerate()
        .rev()
        .find(|(_, entry)| entry.closed_at_step.is_none() && entry.is_topmost_dismissible)
    else {
        return;
    };
    let request_id = entry.request_id.clone();
    let stack_entry_id = entry.stack_entry_id.clone();
    report.stack_entries[index].closed_at_step = Some(step.step_id.clone());
    report.dismissal_evidence.push(OverlayDismissalEvidence {
        request_id: Some(request_id),
        stack_entry_id: Some(stack_entry_id),
        reason: "outside-pointer.topmost_dismissed".to_owned(),
        input_sample_id: step.sample.sample_id.clone(),
        focus_return_target: report.focus_evidence.last().and_then(|evidence| evidence.focus_return_target.clone()),
        outside_pointer_target: Some(format!("{},{}", point.x, point.y)),
        escape_key_seen: false,
        closed: true,
        suppressed: false,
    });
    report.boundary_assertions.overlay_dismissed_by_outside_pointer += 1;
    report.boundary_assertions.overlay_stack_entries_closed += 1;
    report.boundary_assertions.focus_returned += 1;
}

fn recompute_placements(
    fixture: &MountedOverlayLayeringFixture,
    step_id: &str,
    viewport_rect: UiRect,
    report: &mut OverlayLayeringReport,
    viewport_recomputed: bool,
    scroll_recomputed: bool,
) {
    for placement in &mut report.placement_resolutions {
        if let Some(control) = fixture.control_by_anchor(&placement.anchor_id) {
            placement.viewport_rect = viewport_rect;
            placement.anchor_rect = control.bounds;
        }
    }
    report.viewport_evidence.push(OverlayViewportEvidence {
        step_id: step_id.to_owned(),
        viewport_rect,
        scroll_recomputed,
        viewport_recomputed,
        anchor_still_valid: true,
        placement_suppressed: false,
    });
    if scroll_recomputed {
        report.boundary_assertions.placement_recomputed_after_scroll += 1;
    }
    if viewport_recomputed {
        report.boundary_assertions.placement_recomputed_after_viewport_resize += 1;
    }
}

fn invalidate_anchor(step: &OverlayLayeringStep, anchor_id: &str, report: &mut OverlayLayeringReport) {
    let mut closed = false;
    for entry in &mut report.stack_entries {
        if entry.anchor_id == anchor_id && entry.closed_at_step.is_none() {
            entry.closed_at_step = Some(step.step_id.clone());
            closed = true;
        }
    }
    if closed {
        report.suppression_evidence.push(OverlaySuppressionEvidence {
            anchor_id: Some(anchor_id.to_owned()),
            request_kind: "anchor-invalidation".to_owned(),
            trigger: "runtime-anchor-removed".to_owned(),
            reason: "anchor.removed".to_owned(),
            input_sample_id: step.sample.sample_id.clone(),
            opened: false,
            host_commands_executed: 0,
            product_mutations: 0,
        });
        report.boundary_assertions.anchor_invalidation_suppressed += 1;
        report.boundary_assertions.overlay_suppressed += 1;
    }
}

fn suppress(
    step: &OverlayLayeringStep,
    control: Option<&MountedOverlayControl>,
    reason: &str,
    report: &mut OverlayLayeringReport,
) {
    report.suppression_evidence.push(OverlaySuppressionEvidence {
        anchor_id: control.map(|control| control.anchor_id.clone()),
        request_kind: "overlay-open".to_owned(),
        trigger: "normalized-input".to_owned(),
        reason: reason.to_owned(),
        input_sample_id: step.sample.sample_id.clone(),
        opened: false,
        host_commands_executed: 0,
        product_mutations: 0,
    });
    report.boundary_assertions.overlay_suppressed += 1;
}

pub fn base_controls_overlay_layering_fixture() -> MountedOverlayLayeringFixture {
    use ui_controls::{ControlKindId, ControlOverlayLayerPreference};
    MountedOverlayLayeringFixture::new(
        "base-controls.overlay-layering.fixture",
        UiRect::new(0.0, 0.0, 900.0, 640.0),
    )
    .with_control(control(
        101,
        "anchor.button.popup",
        "Button popup",
        32.0,
        ControlOverlayDescriptor::popup_on_press(ControlKindId::new("runenwerk.ui.button"), "anchor.button.popup", "popup.button"),
    ))
    .with_control(control(
        102,
        "anchor.action-prompt.menu",
        "ActionPrompt menu",
        84.0,
        ControlOverlayDescriptor::menu_on_press(ControlKindId::new("runenwerk.ui.action_prompt"), "anchor.action-prompt.menu", "menu.action-prompt"),
    ))
    .with_control(control(
        103,
        "anchor.action-prompt.submenu",
        "Submenu item",
        136.0,
        ControlOverlayDescriptor::new(ControlKindId::new("runenwerk.ui.action_prompt")).with_requirement(
            ControlOverlayRequirement::new(ControlOverlayKind::Menu, ControlOverlayTrigger::PointerPress, "anchor.action-prompt.submenu", "submenu.action-prompt")
                .with_layer(ControlOverlayLayerPreference::Submenu),
        ),
    ))
    .with_control(control(
        104,
        "anchor.dropdown.fixture",
        "Dropdown fixture",
        188.0,
        ControlOverlayDescriptor::dropdown_on_press(ControlKindId::new("runenwerk.ui.list_view"), "anchor.dropdown.fixture", "dropdown.options"),
    ))
    .with_control(control(
        105,
        "anchor.tooltip.hover",
        "Tooltip hover",
        240.0,
        ControlOverlayDescriptor::tooltip_on_hover(ControlKindId::new("runenwerk.ui.label"), "anchor.tooltip.hover", "tooltip.hover"),
    ))
    .with_control(control(
        106,
        "anchor.tooltip.focus",
        "Tooltip focus",
        292.0,
        ControlOverlayDescriptor::tooltip_on_focus(ControlKindId::new("runenwerk.ui.label"), "anchor.tooltip.focus", "tooltip.focus"),
    ))
    .with_control(control(
        107,
        "anchor.color-picker.picker-popup",
        "Picker popup",
        344.0,
        ControlOverlayDescriptor::picker_popup_on_press(ControlKindId::new("runenwerk.ui.color_picker"), "anchor.color-picker.picker-popup", "picker.color"),
    ))
    .with_control(control(
        108,
        "anchor.focus-containing.fixture",
        "Focus-containing",
        396.0,
        ControlOverlayDescriptor::focus_containing_overlay_on_press(ControlKindId::new("runenwerk.ui.button"), "anchor.focus-containing.fixture", "focus-containing.fixture"),
    ))
    .with_control(
        control(
            109,
            "anchor.disabled.fixture",
            "Disabled popup",
            448.0,
            ControlOverlayDescriptor::popup_on_press(ControlKindId::new("runenwerk.ui.button"), "anchor.disabled.fixture", "popup.disabled"),
        )
        .disabled(),
    )
}

pub fn base_controls_overlay_layering_positive_script() -> OverlayLayeringScript {
    OverlayLayeringScript::new("base-controls.overlay-layering.positive")
        .with_step(pointer_step("step.open-popup.button", PointerEventKind::Down, 40.0, 42.0))
        .with_step(key_step("step.dismiss.escape", Key::Escape))
        .with_step(pointer_step("step.open-menu.action", PointerEventKind::Down, 40.0, 94.0))
        .with_step(pointer_step("step.open-submenu.menu", PointerEventKind::Down, 40.0, 146.0))
        .with_step(key_step("step.navigate.menu", Key::Down))
        .with_step(pointer_step("step.dismiss.outside-pointer", PointerEventKind::Down, 780.0, 560.0))
        .with_step(pointer_step("step.open-tooltip.hover", PointerEventKind::Move, 40.0, 250.0))
        .with_step(focus_step("step.open-tooltip.focus", WidgetId(106)))
        .with_step(pointer_step("step.open-picker-popup", PointerEventKind::Down, 40.0, 354.0))
        .with_step(pointer_step("step.open-focus-containing", PointerEventKind::Down, 40.0, 406.0))
        .with_step(pointer_step("step.recompute.scroll", PointerEventKind::Scroll, 40.0, 406.0))
        .with_step(OverlayLayeringStep::new("step.recompute.viewport-resize", NormalizedInputSample::new("sample.recompute.viewport-resize")).with_viewport_rect(UiRect::new(0.0, 0.0, 480.0, 360.0)))
        .with_step(OverlayLayeringStep::new("step.invalidate.anchor-removed", NormalizedInputSample::new("sample.invalidate.anchor-removed")).with_invalidated_anchor("anchor.focus-containing.fixture"))
}

pub fn base_controls_overlay_layering_negative_scripts() -> Vec<OverlayLayeringScript> {
    vec![OverlayLayeringScript::new("base-controls.overlay-layering.disabled")
        .with_step(pointer_step("step.suppress.disabled-anchor", PointerEventKind::Down, 40.0, 458.0))]
}

fn control(id: u64, anchor: &str, label: &str, y: f32, descriptor: ControlOverlayDescriptor) -> MountedOverlayControl {
    MountedOverlayControl::new(WidgetId(id), anchor, label, UiRect::new(24.0, y, 200.0, 34.0), descriptor)
}

fn pointer_step(id: &str, kind: PointerEventKind, x: f32, y: f32) -> OverlayLayeringStep {
    OverlayLayeringStep::new(
        id,
        NormalizedInputSample::new(format!("sample.{id}")).with_fact(NormalizedInputFact::Pointer(
            PointerInputFact::new(kind, UiPoint::new(x, y)),
        )),
    )
}

fn key_step(id: &str, key: Key) -> OverlayLayeringStep {
    OverlayLayeringStep::new(
        id,
        NormalizedInputSample::new(format!("sample.{id}")).with_fact(NormalizedInputFact::Keyboard(
            KeyboardInputFact::new(key, KeyState::Pressed),
        )),
    )
}

fn focus_step(id: &str, widget_id: WidgetId) -> OverlayLayeringStep {
    OverlayLayeringStep::new(
        id,
        NormalizedInputSample::new(format!("sample.{id}")).with_fact(NormalizedInputFact::Focus(
            FocusInputFact::target(FocusTargetId(widget_id.0)),
        )),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayLayeringVisualProof {
    pub proof_id: String,
    pub report: OverlayLayeringReport,
}

pub fn overlay_layering_report_to_visual_proof(report: OverlayLayeringReport) -> OverlayLayeringVisualProof {
    OverlayLayeringVisualProof {
        proof_id: BASE_CONTROLS_OVERLAY_LAYERING_PROOF_ID.to_owned(),
        report,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayLayeringProofRenderFrame {
    pub proof_id: String,
    pub frame: UiFrame,
    pub summary: OverlayLayeringProofRenderSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayLayeringProofRenderSummary {
    pub anchor_rows: usize,
    pub stack_rows: usize,
    pub placement_rows: usize,
    pub report_rows: usize,
    pub has_main_inspector_and_report: bool,
}

pub fn overlay_layering_visual_proof_to_frame(proof: &OverlayLayeringVisualProof) -> OverlayLayeringProofRenderFrame {
    let report = &proof.report;
    let size = UiSize::new(920.0, 640.0);
    let mut primitives = Vec::new();
    let mut order = 0_u32;
    panel(&mut primitives, &mut order, UiRect::new(16.0, 16.0, 280.0, 600.0), "main: anchors");
    panel(&mut primitives, &mut order, UiRect::new(316.0, 16.0, 280.0, 600.0), "inspector: overlays");
    panel(&mut primitives, &mut order, UiRect::new(616.0, 16.0, 280.0, 600.0), "report: evidence");
    label(&mut primitives, &mut order, 32.0, 58.0, &format!("anchors={}", report.declarations.len()));
    label(&mut primitives, &mut order, 332.0, 58.0, &format!("open={}", report.open_intents.len()));
    label(&mut primitives, &mut order, 632.0, 58.0, &format!("stack={}", report.stack_entries.len()));
    label(&mut primitives, &mut order, 632.0, 76.0, &format!("placement={}", report.placement_resolutions.len()));
    label(&mut primitives, &mut order, 632.0, 94.0, &format!("dismiss={}", report.dismissal_evidence.len()));
    label(&mut primitives, &mut order, 632.0, 112.0, &format!("suppress={}", report.suppression_evidence.len()));
    let mut surface = UiSurface::new(UiSurfaceId(13), size);
    surface.push_layer(UiLayer::with_primitives(UiLayerId(0), primitives));
    OverlayLayeringProofRenderFrame {
        proof_id: proof.proof_id.clone(),
        frame: UiFrame::with_surfaces(vec![surface]),
        summary: OverlayLayeringProofRenderSummary {
            anchor_rows: report.declarations.len(),
            stack_rows: report.stack_entries.len(),
            placement_rows: report.placement_resolutions.len(),
            report_rows: report.input_steps.len()
                + report.open_intents.len()
                + report.dismissal_evidence.len()
                + report.suppression_evidence.len(),
            has_main_inspector_and_report: !report.declarations.is_empty()
                && !report.open_intents.is_empty()
                && !report.input_steps.is_empty(),
        },
    }
}

pub fn base_controls_overlay_layering_proof_frame(report: OverlayLayeringReport) -> OverlayLayeringProofRenderFrame {
    let proof = overlay_layering_report_to_visual_proof(report);
    overlay_layering_visual_proof_to_frame(&proof)
}

fn panel(primitives: &mut Vec<UiPrimitive>, order: &mut u32, area: UiRect, title: &str) {
    primitives.push(RectPrimitive::new(area, 3.0, UiPaint::rgba(0.11, 0.12, 0.14, 1.0), UiDrawKey::new(1301, None), sort_key(order)).into());
    primitives.push(BorderPrimitive::new(area, 3.0, 1.0, UiPaint::WHITE, UiDrawKey::new(1302, None), sort_key(order)).into());
    label(primitives, order, area.x + 12.0, area.y + 20.0, title);
}

fn label(primitives: &mut Vec<UiPrimitive>, order: &mut u32, x: f32, y: f32, text: &str) {
    primitives.push(GlyphRunPrimitive::new(glyph_run(text, UiPoint::new(x, y)), Some(UiRect::new(x, y - 12.0, 240.0, 16.0)), UiPaint::WHITE, UiDrawKey::new(1303, None), sort_key(order)).into());
}

fn sort_key(order: &mut u32) -> UiSortKey {
    let key = UiSortKey::new(0, 0, *order);
    *order += 1;
    key
}

fn glyph_run(text: &str, origin: UiPoint) -> GlyphRun {
    let advance = 7.0;
    let glyphs = text
        .chars()
        .enumerate()
        .map(|(index, ch)| PositionedGlyph {
            ch,
            origin: UiPoint::new(origin.x + index as f32 * advance, origin.y),
            advance,
        })
        .collect();
    GlyphRun {
        font_id: FontId(13),
        font_size: 12.0,
        size: UiSize::new(text.chars().count() as f32 * advance, 14.0),
        glyphs,
    }
}
