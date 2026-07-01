//! Renderer-neutral overlay / popup / layering substrate proof.
//!
//! Runtime owns overlay intent, stack, placement, focus, dismissal, report, and
//! no-bypass evidence. It does not execute product commands, mutate app state,
//! perform text editing, create app-specific modal lifecycle behavior, or own a
//! backend renderer.

use ui_controls::{
    ControlOverlayDescriptor, ControlOverlayDismissPolicy, ControlOverlayFocusPolicy,
    ControlOverlayKind, ControlOverlayLayerPreference, ControlOverlayPlacementAlignment,
    ControlOverlayPlacementPreference, ControlOverlayPlacementSide, ControlOverlayRequirement,
    ControlOverlayTrigger,
};
use ui_input::{FocusChange, Key, KeyState, NormalizedInputFact, NormalizedInputSample, PointerEventKind};
use ui_math::{UiPoint, UiRect};

use crate::WidgetId;

pub const BASE_CONTROLS_OVERLAY_LAYERING_PROOF_ID: &str = "base-controls.overlay-layering.proof";
pub const BASE_CONTROLS_EXECUTABLE_OVERLAY_LAYERING_STORY_ID: &str = "base-controls.overlay-layering.story";

#[derive(Debug, Clone, PartialEq)]
pub struct MountedOverlayLayeringFixture {
    pub fixture_id: String,
    pub viewport_rect: UiRect,
    pub controls: Vec<MountedOverlayControl>,
}

impl MountedOverlayLayeringFixture {
    pub fn new(fixture_id: impl Into<String>, viewport_rect: UiRect) -> Self {
        Self { fixture_id: fixture_id.into(), viewport_rect, controls: Vec::new() }
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
    pub fn new(widget_id: WidgetId, anchor_id: impl Into<String>, label: impl Into<String>, bounds: UiRect, descriptor: ControlOverlayDescriptor) -> Self {
        Self { widget_id, anchor_id: anchor_id.into(), label: label.into(), bounds, descriptor, enabled: true }
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
        Self { replay_id: replay_id.into(), steps: Vec::new() }
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
        Self { step_id: step_id.into(), sample, viewport_rect: None, invalidated_anchor: None }
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
pub struct OverlayLayerAssignment { pub request_id: String, pub layer_id: String, pub layer_class: String, pub ordinal: usize }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayFocusEvidence { pub request_id: String, pub focus_policy: String, pub focus_return_target: Option<String> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayPointerCaptureEvidence { pub input_sample_id: String, pub anchor_id: String, pub outside_dismissal_blocked: bool, pub reason: String }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayKeyboardNavigationEvidence { pub input_sample_id: String, pub request_id: Option<String>, pub key: String, pub navigation_intent: String, pub product_commands_executed: u32 }

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
pub struct OverlaySuppressionEvidence { pub anchor_id: Option<String>, pub request_kind: String, pub trigger: String, pub reason: String, pub input_sample_id: String, pub opened: bool, pub host_commands_executed: u32, pub product_mutations: u32 }

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayViewportEvidence { pub step_id: String, pub viewport_rect: UiRect, pub scroll_recomputed: bool, pub viewport_recomputed: bool, pub anchor_still_valid: bool, pub placement_suppressed: bool }

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
        self.host_commands_executed == 0 && self.product_mutations == 0 && self.text_edit_transactions == 0 && self.app_specific_modal_operations == 0 && self.authored_ui_edits == 0 && self.plugin_framework_operations == 0
    }
}

pub fn replay_overlay_layering(fixture: &MountedOverlayLayeringFixture, script: &OverlayLayeringScript) -> OverlayLayeringReport {
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
struct OverlayReplayState { focused: Option<WidgetId>, captured: Option<WidgetId> }

fn empty_report(fixture: &MountedOverlayLayeringFixture, script: &OverlayLayeringScript) -> OverlayLayeringReport {
    OverlayLayeringReport {
        replay_id: script.replay_id.clone(), fixture_id: fixture.fixture_id.clone(), input_steps: Vec::new(), declarations: declarations(fixture), open_intents: Vec::new(), stack_entries: Vec::new(), placement_resolutions: Vec::new(), layer_assignments: Vec::new(), focus_evidence: Vec::new(), dismissal_evidence: Vec::new(), pointer_capture_evidence: Vec::new(), keyboard_navigation_evidence: Vec::new(), suppression_evidence: Vec::new(), viewport_evidence: Vec::new(), boundary_assertions: OverlayBoundaryAssertions::default(),
    }
}

fn declarations(fixture: &MountedOverlayLayeringFixture) -> Vec<OverlayDeclarationEvidence> {
    fixture.controls.iter().flat_map(|control| control.descriptor.requirements.iter().map(move |requirement| OverlayDeclarationEvidence { source_control_id: control.widget_id, anchor_id: control.anchor_id.clone(), overlay_kind: requirement.kind.as_str().to_owned(), trigger: requirement.trigger.as_str().to_owned(), layer: requirement.layer.as_str().to_owned(), dismiss_policy: requirement.dismiss_policy.as_str().to_owned(), focus_policy: requirement.focus_policy.as_str().to_owned() })).collect()
}

fn apply_fact(fixture: &MountedOverlayLayeringFixture, step: &OverlayLayeringStep, fact: &NormalizedInputFact, state: &mut OverlayReplayState, viewport_rect: UiRect, report: &mut OverlayLayeringReport) {
    match fact {
        NormalizedInputFact::Pointer(pointer) => match pointer.kind {
            PointerEventKind::Down => {
                state.captured = fixture.target_at(pointer.position).map(|control| control.widget_id);
                if let Some(control) = fixture.target_at(pointer.position) { open_for_trigger(fixture, step, control, ControlOverlayTrigger::PointerPress, viewport_rect, report); report.pointer_capture_evidence.push(OverlayPointerCaptureEvidence { input_sample_id: step.sample.sample_id.clone(), anchor_id: control.anchor_id.clone(), outside_dismissal_blocked: true, reason: "opening-pointer-capture".to_owned() }); } else { dismiss_outside(step, pointer.position, report); }
            }
            PointerEventKind::Up => { if let Some(captured) = state.captured.take() { if let Some(control) = fixture.controls.iter().find(|control| control.widget_id == captured) { if !control.bounds.contains(pointer.position) { suppress(step, Some(control), "pointer.release_outside", report); } } } }
            PointerEventKind::Move | PointerEventKind::Enter => { if let Some(control) = fixture.target_at(pointer.position) { open_for_trigger(fixture, step, control, ControlOverlayTrigger::PointerHover, viewport_rect, report); } }
            PointerEventKind::Scroll => recompute_placements(fixture, &step.step_id, viewport_rect, report, false, true),
            PointerEventKind::Leave => {}
        },
        NormalizedInputFact::Focus(focus) => { if let FocusChange::Set(target) = focus.change { let widget = WidgetId(target.0); state.focused = Some(widget); if let Some(control) = fixture.controls.iter().find(|control| control.widget_id == widget) { open_for_trigger(fixture, step, control, ControlOverlayTrigger::Focus, viewport_rect, report); } } }
        NormalizedInputFact::Keyboard(keyboard) => { if keyboard.state == KeyState::Pressed { match keyboard.key { Key::Escape => dismiss_escape(step, report), Key::Enter | Key::Space => { if let Some(control) = state.focused.and_then(|widget| fixture.controls.iter().find(|control| control.widget_id == widget)) { open_for_trigger(fixture, step, control, ControlOverlayTrigger::KeyboardActivate, viewport_rect, report); } else { suppress(step, None, "keyboard.no_focus", report); } }, Key::Up | Key::Down | Key::Left | Key::Right => report.keyboard_navigation_evidence.push(OverlayKeyboardNavigationEvidence { input_sample_id: step.sample.sample_id.clone(), request_id: report.stack_entries.iter().rev().find(|entry| entry.closed_at_step.is_none()).map(|entry| entry.request_id.clone()), key: format!("{:?}", keyboard.key), navigation_intent: "overlay-keyboard-navigation".to_owned(), product_commands_executed: 0 }), _ => {} } } }
        NormalizedInputFact::Semantic(semantic) => { if matches!(semantic.event.action, ui_input::UiSemanticAction::Activate) { if let Some(control) = state.focused.and_then(|widget| fixture.controls.iter().find(|control| control.widget_id == widget)) { open_for_trigger(fixture, step, control, ControlOverlayTrigger::SemanticAction, viewport_rect, report); } } }
        NormalizedInputFact::TextIntent(_) => {}
    }
}

fn open_for_trigger(fixture: &MountedOverlayLayeringFixture, step: &OverlayLayeringStep, control: &MountedOverlayControl, trigger: ControlOverlayTrigger, viewport_rect: UiRect, report: &mut OverlayLayeringReport) {
    let Some(requirement) = control.descriptor.requirements.iter().find(|requirement| requirement.trigger == trigger) else { suppress(step, Some(control), "trigger.not_declared", report); return; };
    if !control.enabled { suppress(step, Some(control), "anchor.disabled", report); return; }
    let request_id = format!("overlay.request.{}.{}", report.open_intents.len() + 1, requirement.kind.as_str());
    let stack_entry_id = format!("overlay.stack.{}", report.stack_entries.len() + 1);
    let parent_request_id = parent_request_id(requirement, report);
    let scope_id = parent_request_id.as_ref().map(|id| format!("scope.child-of.{id}")).unwrap_or_else(|| format!("scope.{}", requirement.content_role));
    let placement = resolve_placement(&control.anchor_id, &request_id, control.bounds, &requirement.placement, viewport_rect);
    report.boundary_assertions.overlay_open_requests += 1;
    report.boundary_assertions.overlay_opened += 1;
    report.boundary_assertions.overlay_stack_entries_opened += 1;
    report.open_intents.push(OverlayOpenIntent { input_sample_id: step.sample.sample_id.clone(), step_id: step.step_id.clone(), source_control_id: control.widget_id, anchor_id: control.anchor_id.clone(), request_id: request_id.clone(), overlay_kind: requirement.kind.as_str().to_owned(), trigger: trigger.as_str().to_owned() });
    report.layer_assignments.push(OverlayLayerAssignment { request_id: request_id.clone(), layer_id: format!("layer.{}.{}", requirement.layer.as_str(), report.layer_assignments.len() + 1), layer_class: requirement.layer.as_str().to_owned(), ordinal: layer_ordinal(requirement.layer) });
    report.placement_resolutions.push(placement.clone());
    report.stack_entries.push(OverlayStackEntry { stack_entry_id: stack_entry_id.clone(), request_id: request_id.clone(), parent_request_id, scope_id, anchor_id: control.anchor_id.clone(), kind: requirement.kind.as_str().to_owned(), layer_class: requirement.layer.as_str().to_owned(), opened_at_step: step.step_id.clone(), closed_at_step: None, hit_regions: vec![placement.resolved_rect], is_topmost_dismissible: is_dismissible(requirement.dismiss_policy) });
    report.focus_evidence.push(OverlayFocusEvidence { request_id, focus_policy: requirement.focus_policy.as_str().to_owned(), focus_return_target: focus_return_target(requirement.focus_policy, control) });
    let _ = fixture;
}

fn resolve_placement(anchor_id: &str, request_id: &str, anchor_rect: UiRect, preference: &ControlOverlayPlacementPreference, viewport_rect: UiRect) -> OverlayPlacementResolution {
    let mut resolved_rect = preferred_rect(anchor_rect, preference);
    let min_x = viewport_rect.x + preference.viewport_margin;
    let min_y = viewport_rect.y + preference.viewport_margin;
    let max_x = viewport_rect.x + viewport_rect.width - preference.viewport_margin - resolved_rect.width;
    let max_y = viewport_rect.y + viewport_rect.height - preference.viewport_margin - resolved_rect.height;
    let old = resolved_rect;
    resolved_rect.x = resolved_rect.x.clamp(min_x, max_x.max(min_x));
    resolved_rect.y = resolved_rect.y.clamp(min_y, max_y.max(min_y));
    let shifted = old != resolved_rect;
    OverlayPlacementResolution { anchor_id: anchor_id.to_owned(), request_id: request_id.to_owned(), anchor_rect, requested_side: preference.side.as_str().to_owned(), requested_alignment: preference.alignment.as_str().to_owned(), resolved_side: preference.side.as_str().to_owned(), resolved_alignment: preference.alignment.as_str().to_owned(), resolved_rect, viewport_rect, clamped: shifted, shifted, hidden_or_suppressed: false }
}

fn preferred_rect(anchor: UiRect, preference: &ControlOverlayPlacementPreference) -> UiRect {
    let width = 180.0;
    let height = 96.0;
    let x = match preference.alignment { ControlOverlayPlacementAlignment::Start => anchor.x, ControlOverlayPlacementAlignment::Center => anchor.x + (anchor.width - width) / 2.0, ControlOverlayPlacementAlignment::End => anchor.x + anchor.width - width, ControlOverlayPlacementAlignment::Stretch => anchor.x } + preference.cross_axis_offset;
    let y = match preference.side { ControlOverlayPlacementSide::Top => anchor.y - height - preference.main_axis_offset, ControlOverlayPlacementSide::Bottom => anchor.y + anchor.height + preference.main_axis_offset, ControlOverlayPlacementSide::Left | ControlOverlayPlacementSide::Right => anchor.y, ControlOverlayPlacementSide::Center => anchor.y + (anchor.height - height) / 2.0, ControlOverlayPlacementSide::Cursor => anchor.y + anchor.height };
    UiRect::new(x, y, width, height)
}

fn parent_request_id(requirement: &ControlOverlayRequirement, report: &OverlayLayeringReport) -> Option<String> {
    if requirement.layer != ControlOverlayLayerPreference::Submenu { return None; }
    report.stack_entries.iter().rev().find(|entry| entry.closed_at_step.is_none() && entry.layer_class == "menu").map(|entry| entry.request_id.clone())
}

fn layer_ordinal(layer: ControlOverlayLayerPreference) -> usize { match layer { ControlOverlayLayerPreference::AnchoredPopup => 2, ControlOverlayLayerPreference::Menu => 3, ControlOverlayLayerPreference::Submenu => 4, ControlOverlayLayerPreference::Tooltip => 5, ControlOverlayLayerPreference::FocusContainingOverlay => 6, ControlOverlayLayerPreference::DiagnosticOverlay => 7 } }
fn is_dismissible(policy: ControlOverlayDismissPolicy) -> bool { !matches!(policy, ControlOverlayDismissPolicy::None | ControlOverlayDismissPolicy::HostOwned) }
fn focus_return_target(policy: ControlOverlayFocusPolicy, control: &MountedOverlayControl) -> Option<String> { match policy { ControlOverlayFocusPolicy::ReturnToAnchor | ControlOverlayFocusPolicy::ReturnToPrevious | ControlOverlayFocusPolicy::ContainFocus => Some(control.anchor_id.clone()), _ => None } }

fn topmost_dismissible_mut(entries: &mut [OverlayStackEntry]) -> Option<&mut OverlayStackEntry> { entries.iter_mut().rev().find(|entry| entry.closed_at_step.is_none() && entry.is_topmost_dismissible) }

fn dismiss_escape(step: &OverlayLayeringStep, report: &mut OverlayLayeringReport) {
    let Some(entry) = topmost_dismissible_mut(&mut report.stack_entries) else { report.dismissal_evidence.push(OverlayDismissalEvidence { request_id: None, stack_entry_id: None, reason: "escape.no_open_overlay".to_owned(), input_sample_id: step.sample.sample_id.clone(), focus_return_target: None, outside_pointer_target: None, escape_key_seen: true, closed: false, suppressed: false }); return; };
    entry.closed_at_step = Some(step.step_id.clone());
    report.boundary_assertions.overlay_dismissed_by_escape += 1;
    report.boundary_assertions.overlay_stack_entries_closed += 1;
    report.boundary_assertions.focus_returned += 1;
    report.dismissal_evidence.push(OverlayDismissalEvidence { request_id: Some(entry.request_id.clone()), stack_entry_id: Some(entry.stack_entry_id.clone()), reason: "escape.topmost_dismissed".to_owned(), input_sample_id: step.sample.sample_id.clone(), focus_return_target: Some(entry.anchor_id.clone()), outside_pointer_target: None, escape_key_seen: true, closed: true, suppressed: false });
}

fn dismiss_outside(step: &OverlayLayeringStep, point: UiPoint, report: &mut OverlayLayeringReport) {
    if report.stack_entries.iter().rev().any(|entry| entry.closed_at_step.is_none() && entry.hit_regions.iter().any(|rect| rect.contains(point))) { report.dismissal_evidence.push(OverlayDismissalEvidence { request_id: None, stack_entry_id: None, reason: "outside-pointer.inside-active-overlay".to_owned(), input_sample_id: step.sample.sample_id.clone(), focus_return_target: None, outside_pointer_target: Some(format!("{:.1},{:.1}", point.x, point.y)), escape_key_seen: false, closed: false, suppressed: false }); return; }
    let Some(entry) = topmost_dismissible_mut(&mut report.stack_entries) else { report.dismissal_evidence.push(OverlayDismissalEvidence { request_id: None, stack_entry_id: None, reason: "outside-pointer.no_open_overlay".to_owned(), input_sample_id: step.sample.sample_id.clone(), focus_return_target: None, outside_pointer_target: Some(format!("{:.1},{:.1}", point.x, point.y)), escape_key_seen: false, closed: false, suppressed: false }); return; };
    entry.closed_at_step = Some(step.step_id.clone());
    report.boundary_assertions.overlay_dismissed_by_outside_pointer += 1;
    report.boundary_assertions.overlay_stack_entries_closed += 1;
    report.boundary_assertions.focus_returned += 1;
    report.dismissal_evidence.push(OverlayDismissalEvidence { request_id: Some(entry.request_id.clone()), stack_entry_id: Some(entry.stack_entry_id.clone()), reason: "outside-pointer.topmost_dismissed".to_owned(), input_sample_id: step.sample.sample_id.clone(), focus_return_target: Some(entry.anchor_id.clone()), outside_pointer_target: Some(format!("{:.1},{:.1}", point.x, point.y)), escape_key_seen: false, closed: true, suppressed: false });
}

fn suppress(step: &OverlayLayeringStep, control: Option<&MountedOverlayControl>, reason: impl Into<String>, report: &mut OverlayLayeringReport) {
    report.boundary_assertions.overlay_suppressed += 1;
    report.suppression_evidence.push(OverlaySuppressionEvidence { anchor_id: control.map(|control| control.anchor_id.clone()), request_kind: "overlay-open".to_owned(), trigger: "runtime".to_owned(), reason: reason.into(), input_sample_id: step.sample.sample_id.clone(), opened: false, host_commands_executed: 0, product_mutations: 0 });
}

fn recompute_placements(fixture: &MountedOverlayLayeringFixture, step_id: &str, viewport_rect: UiRect, report: &mut OverlayLayeringReport, viewport_recomputed: bool, scroll_recomputed: bool) {
    let open_entries = report.stack_entries.iter().filter(|entry| entry.closed_at_step.is_none()).cloned().collect::<Vec<_>>();
    for entry in open_entries { if let Some(control) = fixture.control_by_anchor(&entry.anchor_id) { if let Some(requirement) = control.descriptor.requirements.iter().find(|requirement| requirement.kind.as_str() == entry.kind) { report.placement_resolutions.push(resolve_placement(&entry.anchor_id, &entry.request_id, control.bounds, &requirement.placement, viewport_rect)); } } }
    if viewport_recomputed { report.boundary_assertions.placement_recomputed_after_viewport_resize += 1; }
    if scroll_recomputed { report.boundary_assertions.placement_recomputed_after_scroll += 1; }
    report.viewport_evidence.push(OverlayViewportEvidence { step_id: step_id.to_owned(), viewport_rect, scroll_recomputed, viewport_recomputed, anchor_still_valid: true, placement_suppressed: false });
}

fn invalidate_anchor(step: &OverlayLayeringStep, anchor_id: &str, report: &mut OverlayLayeringReport) {
    let mut invalidated = false;
    for entry in &mut report.stack_entries { if entry.anchor_id == anchor_id && entry.closed_at_step.is_none() { entry.closed_at_step = Some(step.step_id.clone()); invalidated = true; } }
    if invalidated { report.boundary_assertions.anchor_invalidation_suppressed += 1; report.boundary_assertions.overlay_stack_entries_closed += 1; }
    report.viewport_evidence.push(OverlayViewportEvidence { step_id: step.step_id.clone(), viewport_rect: UiRect::ZERO, scroll_recomputed: false, viewport_recomputed: false, anchor_still_valid: !invalidated, placement_suppressed: invalidated });
}
