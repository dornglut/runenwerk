//! File: domain/ui/ui_runtime/src/input/generic_interaction.rs
//! Crate: ui_runtime

use ui_controls::{ControlInteractionDescriptor, ControlInteractionOutcome};
use ui_input::{
    FocusDirection, Key, KeyState, NormalizedInputFact, NormalizedInputSample, PointerEventKind,
};
use ui_math::{UiPoint, UiRect};

use crate::WidgetId;

#[derive(Debug, Clone, PartialEq)]
pub struct MountedInteractionFixture {
    pub mounted_story_id: String,
    pub controls: Vec<MountedInteractionControl>,
}

impl MountedInteractionFixture {
    pub fn new(mounted_story_id: impl Into<String>) -> Self {
        Self {
            mounted_story_id: mounted_story_id.into(),
            controls: Vec::new(),
        }
    }

    pub fn with_control(mut self, control: MountedInteractionControl) -> Self {
        self.controls.push(control);
        self
    }

    fn target_at(&self, point: UiPoint) -> Option<&MountedInteractionControl> {
        self.controls
            .iter()
            .find(|control| control.bounds.contains(point))
    }

    fn focusable(&self) -> impl Iterator<Item = &MountedInteractionControl> {
        self.controls
            .iter()
            .filter(|control| control.enabled && control.focusable)
    }

    fn control(&self, widget_id: WidgetId) -> Option<&MountedInteractionControl> {
        self.controls
            .iter()
            .find(|control| control.widget_id == widget_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MountedInteractionControl {
    pub widget_id: WidgetId,
    pub control_kind_id: String,
    pub label: String,
    pub bounds: UiRect,
    pub descriptor: ControlInteractionDescriptor,
    pub enabled: bool,
    pub focusable: bool,
    pub read_only: bool,
}

impl MountedInteractionControl {
    pub fn new(
        widget_id: WidgetId,
        label: impl Into<String>,
        bounds: UiRect,
        descriptor: ControlInteractionDescriptor,
    ) -> Self {
        Self {
            widget_id,
            control_kind_id: descriptor.control_kind_id.as_str().to_owned(),
            label: label.into(),
            bounds,
            descriptor,
            enabled: true,
            focusable: true,
            read_only: false,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn inert(mut self) -> Self {
        self.focusable = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InteractionReplayScript {
    pub replay_id: String,
    pub steps: Vec<InteractionReplayStep>,
}

impl InteractionReplayScript {
    pub fn new(replay_id: impl Into<String>) -> Self {
        Self {
            replay_id: replay_id.into(),
            steps: Vec::new(),
        }
    }

    pub fn with_step(mut self, step: InteractionReplayStep) -> Self {
        self.steps.push(step);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InteractionReplayStep {
    pub step_id: String,
    pub sample: NormalizedInputSample,
}

impl InteractionReplayStep {
    pub fn new(step_id: impl Into<String>, sample: NormalizedInputSample) -> Self {
        Self {
            step_id: step_id.into(),
            sample,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InteractionFormationReport {
    pub replay_id: String,
    pub mounted_story_id: String,
    pub control_descriptors: Vec<RuntimeControlDescriptorFact>,
    pub input_steps: Vec<String>,
    pub target_resolution: Vec<InteractionTargetResolution>,
    pub focus_resolution: Vec<InteractionFocusResolution>,
    pub state_transitions: Vec<InteractionStateTransition>,
    pub runtime_facts: Vec<RuntimeInteractionFact>,
    pub runtime_events: Vec<RuntimeControlInteractionEvent>,
    pub semantic_outcomes: Vec<RuntimeInteractionOutcome>,
    pub suppressed_events: Vec<RuntimeSuppressedInteraction>,
    pub no_target_events: Vec<RuntimeNoTargetInteraction>,
    pub boundary_assertions: InteractionBoundaryAssertions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeControlDescriptorFact {
    pub widget_id: WidgetId,
    pub control_kind_id: String,
    pub interaction_outcomes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionTargetResolution {
    pub step_id: String,
    pub target: Option<WidgetId>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionFocusResolution {
    pub step_id: String,
    pub focused: Option<WidgetId>,
    pub focus_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionStateTransition {
    pub step_id: String,
    pub target: WidgetId,
    pub state: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInteractionFact {
    pub target: WidgetId,
    pub fact: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeControlInteractionEvent {
    pub step_id: String,
    pub target: WidgetId,
    pub event: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInteractionOutcome {
    pub step_id: String,
    pub target: WidgetId,
    pub outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSuppressedInteraction {
    pub step_id: String,
    pub target: WidgetId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeNoTargetInteraction {
    pub step_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InteractionBoundaryAssertions {
    pub host_commands_executed: u32,
    pub product_mutations: u32,
    pub overlay_events: u32,
    pub text_edit_transactions: u32,
}

impl InteractionBoundaryAssertions {
    pub const fn no_bypass_evidence(self) -> bool {
        self.host_commands_executed == 0
            && self.product_mutations == 0
            && self.overlay_events == 0
            && self.text_edit_transactions == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ReplayState {
    hovered: Option<WidgetId>,
    pressed: Option<WidgetId>,
    focused: Option<WidgetId>,
    focus_visible: bool,
}

pub fn replay_interactions(
    fixture: &MountedInteractionFixture,
    script: &InteractionReplayScript,
) -> InteractionFormationReport {
    let mut state = ReplayState::default();
    let mut report = InteractionFormationReport {
        replay_id: script.replay_id.clone(),
        mounted_story_id: fixture.mounted_story_id.clone(),
        control_descriptors: fixture
            .controls
            .iter()
            .map(|control| RuntimeControlDescriptorFact {
                widget_id: control.widget_id,
                control_kind_id: control.control_kind_id.clone(),
                interaction_outcomes: control.descriptor.summary().outcomes,
            })
            .collect(),
        input_steps: Vec::new(),
        target_resolution: Vec::new(),
        focus_resolution: Vec::new(),
        state_transitions: Vec::new(),
        runtime_facts: Vec::new(),
        runtime_events: Vec::new(),
        semantic_outcomes: Vec::new(),
        suppressed_events: Vec::new(),
        no_target_events: Vec::new(),
        boundary_assertions: InteractionBoundaryAssertions::default(),
    };

    for step in &script.steps {
        report.input_steps.push(step.step_id.clone());
        for fact in &step.sample.facts {
            apply_fact(fixture, step, fact, &mut state, &mut report);
        }
    }

    report
}

fn apply_fact(
    fixture: &MountedInteractionFixture,
    step: &InteractionReplayStep,
    fact: &NormalizedInputFact,
    state: &mut ReplayState,
    report: &mut InteractionFormationReport,
) {
    match fact {
        NormalizedInputFact::Pointer(pointer) => {
            let target = fixture.target_at(pointer.position);
            push_target_resolution(
                report,
                step,
                target.map(|control| control.widget_id),
                "pointer",
            );
            let Some(control) = target else {
                report.no_target_events.push(RuntimeNoTargetInteraction {
                    step_id: step.step_id.clone(),
                    reason: "pointer.no_target".to_owned(),
                });
                return;
            };
            if !control.enabled {
                suppress(report, step, control, "control.disabled");
                return;
            }
            match pointer.kind {
                PointerEventKind::Move | PointerEventKind::Enter => {
                    state.hovered = Some(control.widget_id);
                    transition(report, step, control.widget_id, "hovered", true);
                    event(report, step, control.widget_id, "pointer_hover");
                }
                PointerEventKind::Down => {
                    state.pressed = Some(control.widget_id);
                    state.focused = Some(control.widget_id);
                    state.focus_visible = false;
                    transition(report, step, control.widget_id, "pressed", true);
                    transition(report, step, control.widget_id, "focused", true);
                    event(report, step, control.widget_id, "pointer_press");
                }
                PointerEventKind::Up => {
                    let pressed = state.pressed.take();
                    transition(report, step, control.widget_id, "pressed", false);
                    event(report, step, control.widget_id, "pointer_release");
                    if pressed == Some(control.widget_id) {
                        outcome(
                            report,
                            step,
                            control.widget_id,
                            ControlInteractionOutcome::ActivationRequested,
                        );
                    } else {
                        suppress(report, step, control, "pointer.release_without_capture");
                    }
                }
                PointerEventKind::Leave => {
                    state.hovered = None;
                    transition(report, step, control.widget_id, "hovered", false);
                }
                PointerEventKind::Scroll => {
                    event(report, step, control.widget_id, "pointer_scroll_fact");
                }
            }
        }
        NormalizedInputFact::Focus(focus) => {
            if let Some(direction) = focus.direction {
                state.focused = traverse_focus(fixture, state.focused, direction);
                state.focus_visible = focus.focus_visible;
            } else {
                match focus.change {
                    ui_input::FocusChange::None => {}
                    ui_input::FocusChange::Set(target) => {
                        state.focused = Some(WidgetId(target.0));
                        state.focus_visible = focus.focus_visible;
                    }
                    ui_input::FocusChange::Clear => {
                        state.focused = None;
                        state.focus_visible = false;
                    }
                }
            }
            report.focus_resolution.push(InteractionFocusResolution {
                step_id: step.step_id.clone(),
                focused: state.focused,
                focus_visible: state.focus_visible,
            });
            if let Some(target) = state.focused {
                transition(report, step, target, "focused", true);
                if state.focus_visible {
                    transition(report, step, target, "focus-visible", true);
                }
            }
        }
        NormalizedInputFact::Keyboard(keyboard) => {
            let Some(target) = state.focused.and_then(|id| fixture.control(id)) else {
                report.no_target_events.push(RuntimeNoTargetInteraction {
                    step_id: step.step_id.clone(),
                    reason: "keyboard.no_focus".to_owned(),
                });
                return;
            };
            if !target.enabled {
                suppress(report, step, target, "control.disabled");
                return;
            }
            event(report, step, target.widget_id, "keyboard_fact");
            match (&keyboard.key, keyboard.state) {
                (Key::Enter | Key::Space, KeyState::Pressed) => outcome(
                    report,
                    step,
                    target.widget_id,
                    ControlInteractionOutcome::ActivationRequested,
                ),
                (Key::Up | Key::Down | Key::Left | Key::Right, KeyState::Pressed) => {
                    let summary = target.descriptor.summary();
                    let mapped = if summary.outcomes.iter().any(|value| value == "node-intent") {
                        ControlInteractionOutcome::NodeIntent
                    } else if summary
                        .outcomes
                        .iter()
                        .any(|value| value == "cell-or-row-intent")
                    {
                        ControlInteractionOutcome::CellOrRowIntent
                    } else {
                        ControlInteractionOutcome::ActiveItemIntent
                    };
                    outcome(report, step, target.widget_id, mapped);
                    transition(report, step, target.widget_id, "active", true);
                }
                _ => {}
            }
        }
        NormalizedInputFact::Semantic(semantic) => {
            let Some(target) = state.focused.and_then(|id| fixture.control(id)) else {
                report.no_target_events.push(RuntimeNoTargetInteraction {
                    step_id: step.step_id.clone(),
                    reason: "semantic.no_focus".to_owned(),
                });
                return;
            };
            if !target.enabled {
                suppress(report, step, target, "control.disabled");
                return;
            }
            event(report, step, target.widget_id, "semantic_fact");
            if matches!(semantic.event.action, ui_input::UiSemanticAction::Activate) {
                outcome(
                    report,
                    step,
                    target.widget_id,
                    ControlInteractionOutcome::ActivationRequested,
                );
            }
        }
        NormalizedInputFact::TextIntent(_) => {
            let Some(target) = state.focused.and_then(|id| fixture.control(id)) else {
                report.no_target_events.push(RuntimeNoTargetInteraction {
                    step_id: step.step_id.clone(),
                    reason: "text_intent.no_focus".to_owned(),
                });
                return;
            };
            if target.descriptor.text_intent_probe {
                event(report, step, target.widget_id, "text_intent_fact");
                outcome(
                    report,
                    step,
                    target.widget_id,
                    ControlInteractionOutcome::TextIntentSeen,
                );
            } else {
                suppress(report, step, target, "text_intent.not_declared");
            }
        }
    }
}

fn traverse_focus(
    fixture: &MountedInteractionFixture,
    current: Option<WidgetId>,
    direction: FocusDirection,
) -> Option<WidgetId> {
    let focusable = fixture
        .focusable()
        .map(|control| control.widget_id)
        .collect::<Vec<_>>();
    if focusable.is_empty() {
        return None;
    }
    let current_index =
        current.and_then(|id| focusable.iter().position(|candidate| *candidate == id));
    match direction {
        FocusDirection::Previous | FocusDirection::Left | FocusDirection::Up => current_index
            .map(|index| {
                if index == 0 {
                    focusable.len() - 1
                } else {
                    index - 1
                }
            })
            .or(Some(0))
            .map(|index| focusable[index]),
        FocusDirection::Next | FocusDirection::Right | FocusDirection::Down => current_index
            .map(|index| (index + 1) % focusable.len())
            .or(Some(0))
            .map(|index| focusable[index]),
    }
}

fn push_target_resolution(
    report: &mut InteractionFormationReport,
    step: &InteractionReplayStep,
    target: Option<WidgetId>,
    reason: &str,
) {
    report.target_resolution.push(InteractionTargetResolution {
        step_id: step.step_id.clone(),
        target,
        reason: reason.to_owned(),
    });
}

fn transition(
    report: &mut InteractionFormationReport,
    step: &InteractionReplayStep,
    target: WidgetId,
    state: &str,
    active: bool,
) {
    report.state_transitions.push(InteractionStateTransition {
        step_id: step.step_id.clone(),
        target,
        state: state.to_owned(),
        active,
    });
    if active {
        report.runtime_facts.push(RuntimeInteractionFact {
            target,
            fact: state.to_owned(),
        });
    }
}

fn event(
    report: &mut InteractionFormationReport,
    step: &InteractionReplayStep,
    target: WidgetId,
    event: &str,
) {
    report.runtime_events.push(RuntimeControlInteractionEvent {
        step_id: step.step_id.clone(),
        target,
        event: event.to_owned(),
    });
}

fn outcome(
    report: &mut InteractionFormationReport,
    step: &InteractionReplayStep,
    target: WidgetId,
    outcome_value: ControlInteractionOutcome,
) {
    report.semantic_outcomes.push(RuntimeInteractionOutcome {
        step_id: step.step_id.clone(),
        target,
        outcome: outcome_value.as_str().to_owned(),
    });
}

fn suppress(
    report: &mut InteractionFormationReport,
    step: &InteractionReplayStep,
    control: &MountedInteractionControl,
    reason: &str,
) {
    transition(report, step, control.widget_id, "suppressed", true);
    report.suppressed_events.push(RuntimeSuppressedInteraction {
        step_id: step.step_id.clone(),
        target: control.widget_id,
        reason: reason.to_owned(),
    });
}
