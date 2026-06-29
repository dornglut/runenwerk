//! File: domain/ui/ui_runtime/src/input/generic_interaction.rs
//! Crate: ui_runtime

use ui_controls::{
    CompiledControlPackage, ControlInteractionDescriptor, ControlInteractionOutcome,
    ControlInteractionRequirement, ControlInteractionTrigger,
};
use ui_input::{
    FocusChange, FocusDirection, Key, KeyState, NormalizedInputFact, NormalizedInputSample,
    PointerEventKind, PointerInputFact,
};
use ui_math::{UiPoint, UiRect};

use crate::WidgetId;

/// Mounted, renderer-neutral interaction fixture used by deterministic replay.
///
/// The fixture binds package-backed control descriptors to bounds and local
/// enabled/focusable/read-only flags. It deliberately does not execute
/// app/editor/game commands, mutate product state, create overlays, or own text
/// editing.
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

    pub fn from_compiled_controls(
        mounted_story_id: impl Into<String>,
        compiled: &CompiledControlPackage,
        placements: impl IntoIterator<Item = MountedInteractionPlacement>,
    ) -> Self {
        let mut fixture = Self::new(mounted_story_id);
        for placement in placements {
            let kind_id = ui_controls::ControlKindId::new(placement.control_kind_id.clone());
            let descriptor = compiled
                .package
                .interaction_descriptor(&kind_id)
                .cloned()
                .unwrap_or_else(|| {
                    panic!(
                        "missing package interaction descriptor for {}",
                        placement.control_kind_id
                    )
                });
            let mut control = MountedInteractionControl::new(
                placement.widget_id,
                placement.label,
                placement.bounds,
                descriptor,
            );
            control.enabled = placement.enabled;
            control.focusable = placement.focusable;
            control.read_only = placement.read_only;
            fixture = fixture.with_control(control);
        }
        fixture
    }

    fn target_at(&self, point: UiPoint) -> Option<&MountedInteractionControl> {
        self.controls
            .iter()
            .find(|control| control.bounds.contains(point))
    }

    fn focusable(&self) -> impl Iterator<Item = &MountedInteractionControl> {
        self.controls.iter().filter(|control| {
            control.enabled
                && control.focusable
                && control
                    .descriptor
                    .requirements
                    .iter()
                    .any(|requirement| requirement.trigger == ControlInteractionTrigger::Focus)
        })
    }

    fn control(&self, widget_id: WidgetId) -> Option<&MountedInteractionControl> {
        self.controls
            .iter()
            .find(|control| control.widget_id == widget_id)
    }
}

/// Placement data that binds a package-backed control descriptor to bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct MountedInteractionPlacement {
    pub widget_id: WidgetId,
    pub control_kind_id: String,
    pub label: String,
    pub bounds: UiRect,
    pub enabled: bool,
    pub focusable: bool,
    pub read_only: bool,
}

impl MountedInteractionPlacement {
    pub fn new(
        widget_id: WidgetId,
        control_kind_id: impl Into<String>,
        label: impl Into<String>,
        bounds: UiRect,
    ) -> Self {
        Self {
            widget_id,
            control_kind_id: control_kind_id.into(),
            label: label.into(),
            bounds,
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

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }
}

/// Mounted control plus the package-owned interaction descriptor copy.
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

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }
}

/// Deterministic sequence of normalized input samples for interaction replay.
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

/// One replay step and the normalized input sample formed by `ui_input`.
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

/// Auditable report produced by resolving normalized input against a fixture.
///
/// The report separates target/focus resolution, runtime facts/events,
/// semantic outcomes, negative evidence, and no-bypass boundary counters.
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
    pub interaction_states: Vec<String>,
    pub interaction_triggers: Vec<String>,
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
    pub reason: String,
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

/// Renderer-neutral visible proof for Phase 12 generic interaction.
///
/// This model is not a product UI and does not add a story/gallery framework.
/// It is the static proof surface that an existing gallery or static mount path
/// can render: a main view, inspector view, and report/event view formed from
/// the same descriptor-backed replay report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionVisualProof {
    pub proof_id: String,
    pub main_view: InteractionVisualMainView,
    pub inspector_view: InteractionInspectorView,
    pub report_view: InteractionReportView,
}

impl InteractionVisualProof {
    pub fn from_fixture_report(
        proof_id: impl Into<String>,
        fixture: &MountedInteractionFixture,
        report: &InteractionFormationReport,
        selected_widget: WidgetId,
    ) -> Self {
        let selected_control = fixture.control(selected_widget);
        Self {
            proof_id: proof_id.into(),
            main_view: InteractionVisualMainView::from_fixture_report(fixture, report),
            inspector_view: InteractionInspectorView::from_fixture_report(selected_control, report),
            report_view: InteractionReportView::from_report(report),
        }
    }
}

/// Main-view controls and markers that a static story can render visibly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionVisualMainView {
    pub mounted_story_id: String,
    pub controls: Vec<InteractionVisualControl>,
}

impl InteractionVisualMainView {
    fn from_fixture_report(
        fixture: &MountedInteractionFixture,
        report: &InteractionFormationReport,
    ) -> Self {
        Self {
            mounted_story_id: fixture.mounted_story_id.clone(),
            controls: fixture
                .controls
                .iter()
                .map(|control| InteractionVisualControl::from_report(control, report))
                .collect(),
        }
    }

    pub fn control(&self, widget_id: WidgetId) -> Option<&InteractionVisualControl> {
        self.controls
            .iter()
            .find(|control| control.widget_id == widget_id)
    }
}

/// One visible mounted control in the Phase 12 proof model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionVisualControl {
    pub widget_id: WidgetId,
    pub control_kind_id: String,
    pub label: String,
    pub markers: Vec<InteractionVisualMarker>,
}

impl InteractionVisualControl {
    fn from_report(
        control: &MountedInteractionControl,
        report: &InteractionFormationReport,
    ) -> Self {
        let mut markers = vec![InteractionVisualMarker::new(
            InteractionVisibleState::Normal,
        )];
        if !control.enabled {
            push_marker(&mut markers, InteractionVisibleState::Disabled);
        }
        if control.read_only {
            push_marker(&mut markers, InteractionVisibleState::ReadOnly);
        }
        for transition in report
            .state_transitions
            .iter()
            .filter(|transition| transition.target == control.widget_id && transition.active)
        {
            if let Some(state) = visible_state_from_transition(&transition.state) {
                push_marker(&mut markers, state);
            }
        }
        for fact in report
            .runtime_facts
            .iter()
            .filter(|fact| fact.target == control.widget_id)
        {
            if let Some(state) = visible_state_from_fact(&fact.fact) {
                push_marker(&mut markers, state);
            }
        }
        for outcome in report
            .semantic_outcomes
            .iter()
            .filter(|outcome| outcome.target == control.widget_id)
        {
            if let Some(state) = visible_state_from_outcome(&outcome.outcome) {
                push_marker(&mut markers, state);
            }
        }
        if report
            .suppressed_events
            .iter()
            .any(|event| event.target == control.widget_id)
        {
            push_marker(&mut markers, InteractionVisibleState::Suppressed);
        }

        Self {
            widget_id: control.widget_id,
            control_kind_id: control.control_kind_id.clone(),
            label: control.label.clone(),
            markers,
        }
    }

    pub fn has_marker(&self, state: InteractionVisibleState) -> bool {
        self.markers.iter().any(|marker| marker.state == state)
    }
}

/// A named visible marker, such as hovered, pressed, or text-intent probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionVisualMarker {
    pub state: InteractionVisibleState,
    pub label: String,
}

impl InteractionVisualMarker {
    pub fn new(state: InteractionVisibleState) -> Self {
        Self {
            label: state.as_str().to_owned(),
            state,
        }
    }
}

/// Visible proof states used by the renderer-neutral Phase 12 proof surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InteractionVisibleState {
    Normal,
    Hovered,
    Pressed,
    Captured,
    Focused,
    FocusVisible,
    Active,
    Disabled,
    ReadOnly,
    Suppressed,
    ActivationRequested,
    ActionIntent,
    ListActiveItemIntent,
    TreeNodeIntent,
    TableCellOrRowIntent,
    TextIntentProbe,
    ReadOnlyTextIntentProbe,
}

impl InteractionVisibleState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Hovered => "hovered",
            Self::Pressed => "pressed",
            Self::Captured => "captured",
            Self::Focused => "focused",
            Self::FocusVisible => "focus-visible",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::ReadOnly => "read-only",
            Self::Suppressed => "suppressed",
            Self::ActivationRequested => "activation-requested",
            Self::ActionIntent => "action-intent",
            Self::ListActiveItemIntent => "list-active-item-intent",
            Self::TreeNodeIntent => "tree-node-intent",
            Self::TableCellOrRowIntent => "table-cell-or-row-intent",
            Self::TextIntentProbe => "text-intent-probe",
            Self::ReadOnlyTextIntentProbe => "read-only-text-intent-probe",
        }
    }
}

/// Inspector pane data for the selected proof control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionInspectorView {
    pub selected_widget: Option<WidgetId>,
    pub control_kind_id: Option<String>,
    pub declared_requirements: Vec<String>,
    pub current_reusable_interaction_state_set: Vec<InteractionVisibleState>,
    pub text_intent_probe: bool,
    pub read_only: bool,
}

impl InteractionInspectorView {
    fn from_fixture_report(
        control: Option<&MountedInteractionControl>,
        report: &InteractionFormationReport,
    ) -> Self {
        let Some(control) = control else {
            return Self {
                selected_widget: None,
                control_kind_id: None,
                declared_requirements: Vec::new(),
                current_reusable_interaction_state_set: Vec::new(),
                text_intent_probe: false,
                read_only: false,
            };
        };
        let mut state_set = Vec::new();
        for transition in report
            .state_transitions
            .iter()
            .filter(|transition| transition.target == control.widget_id && transition.active)
        {
            if let Some(state) = visible_state_from_transition(&transition.state)
                && !state_set.contains(&state)
            {
                state_set.push(state);
            }
        }

        Self {
            selected_widget: Some(control.widget_id),
            control_kind_id: Some(control.control_kind_id.clone()),
            declared_requirements: control
                .descriptor
                .requirements
                .iter()
                .map(|requirement| {
                    let outcomes = requirement
                        .outcomes
                        .iter()
                        .map(|outcome| outcome.as_str())
                        .collect::<Vec<_>>()
                        .join("|");
                    format!("{} -> {}", requirement.trigger.as_str(), outcomes)
                })
                .collect(),
            current_reusable_interaction_state_set: state_set,
            text_intent_probe: control.descriptor.text_intent_probe,
            read_only: control.read_only,
        }
    }
}

/// Report/event pane data for the Phase 12 visual proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionReportView {
    pub replay_steps: Vec<String>,
    pub target_resolution: Vec<String>,
    pub focus_resolution: Vec<String>,
    pub state_transitions: Vec<String>,
    pub runtime_facts: Vec<String>,
    pub runtime_events: Vec<String>,
    pub semantic_outcomes: Vec<String>,
    pub suppressed_events: Vec<String>,
    pub no_target_events: Vec<String>,
    pub boundary_assertions: InteractionBoundaryAssertions,
}

impl InteractionReportView {
    fn from_report(report: &InteractionFormationReport) -> Self {
        Self {
            replay_steps: report.input_steps.clone(),
            target_resolution: report
                .target_resolution
                .iter()
                .map(|resolution| {
                    format!(
                        "{}:{:?}:{}",
                        resolution.step_id, resolution.target, resolution.reason
                    )
                })
                .collect(),
            focus_resolution: report
                .focus_resolution
                .iter()
                .map(|resolution| {
                    format!(
                        "{}:{:?}:{}:{}",
                        resolution.step_id,
                        resolution.focused,
                        resolution.focus_visible,
                        resolution.reason
                    )
                })
                .collect(),
            state_transitions: report
                .state_transitions
                .iter()
                .map(|transition| {
                    format!(
                        "{}:{:?}:{}:{}",
                        transition.step_id, transition.target, transition.state, transition.active
                    )
                })
                .collect(),
            runtime_facts: report
                .runtime_facts
                .iter()
                .map(|fact| format!("{:?}:{}", fact.target, fact.fact))
                .collect(),
            runtime_events: report
                .runtime_events
                .iter()
                .map(|event| format!("{}:{:?}:{}", event.step_id, event.target, event.event))
                .collect(),
            semantic_outcomes: report
                .semantic_outcomes
                .iter()
                .map(|outcome| {
                    format!(
                        "{}:{:?}:{}",
                        outcome.step_id, outcome.target, outcome.outcome
                    )
                })
                .collect(),
            suppressed_events: report
                .suppressed_events
                .iter()
                .map(|event| format!("{}:{:?}:{}", event.step_id, event.target, event.reason))
                .collect(),
            no_target_events: report
                .no_target_events
                .iter()
                .map(|event| format!("{}:{}", event.step_id, event.reason))
                .collect(),
            boundary_assertions: report.boundary_assertions,
        }
    }
}

/// Complete proof frame that can be mounted by a renderer or inspected in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionProofFrame {
    pub proof: InteractionVisualProof,
}

impl InteractionProofFrame {
    pub fn new(proof: InteractionVisualProof) -> Self {
        Self { proof }
    }
}

fn push_marker(markers: &mut Vec<InteractionVisualMarker>, state: InteractionVisibleState) {
    if !markers.iter().any(|marker| marker.state == state) {
        markers.push(InteractionVisualMarker::new(state));
    }
}

fn visible_state_from_transition(state: &str) -> Option<InteractionVisibleState> {
    match state {
        "hovered" => Some(InteractionVisibleState::Hovered),
        "pressed" => Some(InteractionVisibleState::Pressed),
        "captured" => Some(InteractionVisibleState::Captured),
        "focused" => Some(InteractionVisibleState::Focused),
        "focus-visible" => Some(InteractionVisibleState::FocusVisible),
        "active" => Some(InteractionVisibleState::Active),
        "read-only" => Some(InteractionVisibleState::ReadOnly),
        "suppressed" => Some(InteractionVisibleState::Suppressed),
        _ => None,
    }
}

fn visible_state_from_fact(fact: &str) -> Option<InteractionVisibleState> {
    match fact {
        "text-intent-probe" => Some(InteractionVisibleState::TextIntentProbe),
        "text-intent-read-only-probe" => Some(InteractionVisibleState::ReadOnlyTextIntentProbe),
        _ => visible_state_from_transition(fact),
    }
}

fn visible_state_from_outcome(outcome: &str) -> Option<InteractionVisibleState> {
    match outcome {
        "activation-requested" => Some(InteractionVisibleState::ActivationRequested),
        "action-intent" => Some(InteractionVisibleState::ActionIntent),
        "active-item-intent" => Some(InteractionVisibleState::ListActiveItemIntent),
        "node-intent" => Some(InteractionVisibleState::TreeNodeIntent),
        "cell-or-row-intent" => Some(InteractionVisibleState::TableCellOrRowIntent),
        "text-intent-seen" => Some(InteractionVisibleState::TextIntentProbe),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ReplayState {
    hovered: Option<WidgetId>,
    pressed: Option<WidgetId>,
    captured: Option<WidgetId>,
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
                interaction_states: control.descriptor.summary().states,
                interaction_triggers: control.descriptor.summary().triggers,
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
            apply_pointer_fact(fixture, step, pointer, state, report);
        }
        NormalizedInputFact::Focus(focus) => {
            apply_focus_fact(fixture, step, focus, state, report);
        }
        NormalizedInputFact::Keyboard(keyboard) => {
            let Some(target) = state.focused.and_then(|id| fixture.control(id)) else {
                report.no_target_events.push(RuntimeNoTargetInteraction {
                    step_id: step.step_id.clone(),
                    reason: "keyboard.no_focus".to_owned(),
                });
                return;
            };
            let trigger = match (&keyboard.key, keyboard.state) {
                (Key::Enter | Key::Space, KeyState::Pressed) => {
                    Some(ControlInteractionTrigger::KeyboardActivate)
                }
                (Key::Up | Key::Down | Key::Left | Key::Right, KeyState::Pressed) => {
                    Some(ControlInteractionTrigger::KeyboardNavigate)
                }
                _ => None,
            };
            let Some(trigger) = trigger else {
                return;
            };
            if !target.enabled {
                suppress_disabled_for_trigger(report, step, target, trigger);
                return;
            }
            let Some(requirement) = requirement_for(target, trigger) else {
                suppress_if_declared(report, step, target, "trigger.not_declared");
                return;
            };
            if requirement.requires_focus && state.focused != Some(target.widget_id) {
                suppress_if_declared(report, step, target, "focus.required");
                return;
            }
            event(report, step, target.widget_id, trigger.as_str());
            emit_declared_outcomes(report, step, target, requirement);
            if trigger == ControlInteractionTrigger::KeyboardNavigate {
                transition(report, step, target.widget_id, "active", true);
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
                suppress_disabled_for_trigger(
                    report,
                    step,
                    target,
                    ControlInteractionTrigger::SemanticAction,
                );
                return;
            }
            let Some(requirement) =
                requirement_for(target, ControlInteractionTrigger::SemanticAction)
            else {
                suppress_if_declared(report, step, target, "trigger.not_declared");
                return;
            };
            if requirement.requires_focus && state.focused != Some(target.widget_id) {
                suppress_if_declared(report, step, target, "focus.required");
                return;
            }
            if matches!(semantic.event.action, ui_input::UiSemanticAction::Activate) {
                event(
                    report,
                    step,
                    target.widget_id,
                    ControlInteractionTrigger::SemanticAction.as_str(),
                );
                emit_declared_outcomes(report, step, target, requirement);
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
            if !target.enabled {
                suppress_disabled_for_trigger(
                    report,
                    step,
                    target,
                    ControlInteractionTrigger::TextIntent,
                );
                return;
            }
            if !target.descriptor.text_intent_probe {
                suppress_if_declared(report, step, target, "text_intent.not_declared");
                return;
            }
            let Some(requirement) = requirement_for(target, ControlInteractionTrigger::TextIntent)
            else {
                suppress_if_declared(report, step, target, "trigger.not_declared");
                return;
            };
            if requirement.requires_focus && state.focused != Some(target.widget_id) {
                suppress_if_declared(report, step, target, "focus.required");
                return;
            }
            if target.read_only {
                transition(report, step, target.widget_id, "read-only", true);
                runtime_fact(report, target.widget_id, "text-intent-read-only-probe");
                event(
                    report,
                    step,
                    target.widget_id,
                    ControlInteractionTrigger::TextIntent.as_str(),
                );
                event(
                    report,
                    step,
                    target.widget_id,
                    "text_intent_read_only_probe",
                );
            } else {
                runtime_fact(report, target.widget_id, "text-intent-probe");
                event(
                    report,
                    step,
                    target.widget_id,
                    ControlInteractionTrigger::TextIntent.as_str(),
                );
            }
            emit_declared_outcomes(report, step, target, requirement);
        }
    }
}

fn apply_pointer_fact(
    fixture: &MountedInteractionFixture,
    step: &InteractionReplayStep,
    pointer: &PointerInputFact,
    state: &mut ReplayState,
    report: &mut InteractionFormationReport,
) {
    match pointer.kind {
        PointerEventKind::Move | PointerEventKind::Enter | PointerEventKind::Scroll => {
            let target = fixture.target_at(pointer.position);
            push_target_resolution(
                report,
                step,
                target.map(|control| control.widget_id),
                target
                    .map(|_| "pointer.target_resolved")
                    .unwrap_or("pointer.no_target"),
            );
            let Some(control) = target else {
                no_target(report, step, "pointer.no_target");
                return;
            };
            let trigger = pointer_trigger(pointer.kind);
            if !control.enabled {
                suppress_disabled_for_trigger(report, step, control, trigger);
                return;
            }
            let Some(requirement) = requirement_for(control, trigger) else {
                suppress_if_declared(report, step, control, "trigger.not_declared");
                return;
            };
            if pointer.kind == PointerEventKind::Scroll {
                event(report, step, control.widget_id, "pointer_scroll_fact");
                emit_declared_outcomes(report, step, control, requirement);
                return;
            }
            state.hovered = Some(control.widget_id);
            transition(report, step, control.widget_id, "hovered", true);
            event(report, step, control.widget_id, trigger.as_str());
            emit_declared_outcomes(report, step, control, requirement);
        }
        PointerEventKind::Down => {
            let target = fixture.target_at(pointer.position);
            push_target_resolution(
                report,
                step,
                target.map(|control| control.widget_id),
                target
                    .map(|_| "pointer.target_resolved")
                    .unwrap_or("pointer.no_target"),
            );
            let Some(control) = target else {
                no_target(report, step, "pointer.no_target");
                return;
            };
            if !control.enabled {
                suppress_disabled_for_trigger(
                    report,
                    step,
                    control,
                    ControlInteractionTrigger::PointerPress,
                );
                return;
            }
            if requirement_for(control, ControlInteractionTrigger::PointerPress).is_none() {
                suppress_if_declared(report, step, control, "trigger.not_declared");
                return;
            }

            state.pressed = Some(control.widget_id);
            state.captured = Some(control.widget_id);
            if control.focusable
                && requirement_for(control, ControlInteractionTrigger::Focus).is_some()
            {
                state.focused = Some(control.widget_id);
                state.focus_visible = false;
                transition(report, step, control.widget_id, "focused", true);
            }
            transition(report, step, control.widget_id, "pressed", true);
            transition(report, step, control.widget_id, "captured", true);
            event(
                report,
                step,
                control.widget_id,
                ControlInteractionTrigger::PointerPress.as_str(),
            );
        }
        PointerEventKind::Up => {
            let hit_target = fixture.target_at(pointer.position);
            let pressed = state.pressed.take();
            let captured = state.captured.take();
            push_target_resolution(
                report,
                step,
                hit_target.map(|control| control.widget_id),
                match (captured, hit_target.map(|control| control.widget_id)) {
                    (Some(captured), Some(hit)) if captured == hit => "pointer.release_inside",
                    (Some(_), _) => "pointer.release_outside",
                    (None, Some(_)) => "pointer.release_without_capture",
                    (None, None) => "pointer.no_target",
                },
            );

            let Some(pressed_id) = pressed else {
                if let Some(control) = hit_target {
                    suppress_if_declared(report, step, control, "pointer.release_without_capture");
                } else {
                    no_target(report, step, "pointer.no_target");
                }
                return;
            };
            let Some(control) = fixture.control(pressed_id) else {
                no_target(report, step, "pointer.capture_missing");
                return;
            };
            transition(report, step, control.widget_id, "pressed", false);
            transition(report, step, control.widget_id, "captured", false);

            if !control.enabled {
                suppress_disabled_for_trigger(
                    report,
                    step,
                    control,
                    ControlInteractionTrigger::PointerCancel,
                );
                return;
            }
            if requirement_for(control, ControlInteractionTrigger::PointerRelease).is_some() {
                event(
                    report,
                    step,
                    control.widget_id,
                    ControlInteractionTrigger::PointerRelease.as_str(),
                );
            } else {
                suppress_if_declared(report, step, control, "trigger.not_declared");
                return;
            }

            if hit_target.map(|target| target.widget_id) == Some(pressed_id) {
                let Some(requirement) =
                    requirement_for(control, ControlInteractionTrigger::PointerActivate)
                else {
                    suppress_if_declared(report, step, control, "trigger.not_declared");
                    return;
                };
                event(
                    report,
                    step,
                    control.widget_id,
                    ControlInteractionTrigger::PointerActivate.as_str(),
                );
                emit_declared_outcomes(report, step, control, requirement);
            } else {
                if requirement_for(control, ControlInteractionTrigger::PointerCancel).is_some() {
                    event(
                        report,
                        step,
                        control.widget_id,
                        ControlInteractionTrigger::PointerCancel.as_str(),
                    );
                }
                suppress_if_declared(report, step, control, "pointer.release_outside");
            }
        }
        PointerEventKind::Leave => {
            let target = fixture
                .target_at(pointer.position)
                .or_else(|| state.hovered.and_then(|id| fixture.control(id)))
                .or_else(|| state.captured.and_then(|id| fixture.control(id)));
            push_target_resolution(
                report,
                step,
                target.map(|control| control.widget_id),
                target
                    .map(|_| "pointer.leave_target_resolved")
                    .unwrap_or("pointer.no_target"),
            );
            let Some(control) = target else {
                state.hovered = None;
                no_target(report, step, "pointer.no_target");
                return;
            };
            state.hovered = None;
            transition(report, step, control.widget_id, "hovered", false);
            if state.pressed == Some(control.widget_id) {
                event(
                    report,
                    step,
                    control.widget_id,
                    "pointer_leave_kept_capture",
                );
            }
        }
    }
}

fn apply_focus_fact(
    fixture: &MountedInteractionFixture,
    step: &InteractionReplayStep,
    focus: &ui_input::FocusInputFact,
    state: &mut ReplayState,
    report: &mut InteractionFormationReport,
) {
    let mut reason = "focus.no_change";
    if let Some(direction) = focus.direction {
        state.focused = traverse_focus(fixture, state.focused, direction);
        state.focus_visible = focus.focus_visible;
        reason = if state.focused.is_some() {
            "focus.target_resolved"
        } else {
            "focus.target_missing"
        };
    } else {
        match focus.change {
            FocusChange::None => {}
            FocusChange::Set(target) => {
                let requested = WidgetId(target.0);
                match validate_focus_target(fixture, requested) {
                    Ok(control) => {
                        state.focused = Some(control.widget_id);
                        state.focus_visible = focus.focus_visible;
                        reason = "focus.target_resolved";
                    }
                    Err(FocusTargetRejection::Missing) => {
                        no_target(report, step, "focus.target_missing");
                        reason = "focus.target_missing";
                    }
                    Err(FocusTargetRejection::Disabled(control)) => {
                        suppress_if_declared(report, step, control, "focus.target_disabled");
                        reason = "focus.target_disabled";
                    }
                    Err(FocusTargetRejection::NotFocusable(control)) => {
                        suppress_if_declared(report, step, control, "focus.target_not_focusable");
                        reason = "focus.target_not_focusable";
                    }
                    Err(FocusTargetRejection::DoesNotDeclareFocus(control)) => {
                        suppress_if_declared(
                            report,
                            step,
                            control,
                            "focus.target_does_not_declare_focus",
                        );
                        reason = "focus.target_does_not_declare_focus";
                    }
                }
            }
            FocusChange::Clear => {
                state.focused = None;
                state.focus_visible = false;
                reason = "focus.cleared";
            }
        }
    }
    report.focus_resolution.push(InteractionFocusResolution {
        step_id: step.step_id.clone(),
        focused: state.focused,
        focus_visible: state.focus_visible,
        reason: reason.to_owned(),
    });
    if let Some(target) = state.focused {
        if let Some(control) = fixture.control(target)
            && requirement_for(control, ControlInteractionTrigger::Focus).is_some()
        {
            transition(report, step, target, "focused", true);
            if state.focus_visible {
                transition(report, step, target, "focus-visible", true);
            }
        }
    }
}

enum FocusTargetRejection<'a> {
    Missing,
    Disabled(&'a MountedInteractionControl),
    NotFocusable(&'a MountedInteractionControl),
    DoesNotDeclareFocus(&'a MountedInteractionControl),
}

fn validate_focus_target(
    fixture: &MountedInteractionFixture,
    target: WidgetId,
) -> Result<&MountedInteractionControl, FocusTargetRejection<'_>> {
    let Some(control) = fixture.control(target) else {
        return Err(FocusTargetRejection::Missing);
    };
    if !control.enabled {
        return Err(FocusTargetRejection::Disabled(control));
    }
    if !control.focusable {
        return Err(FocusTargetRejection::NotFocusable(control));
    }
    if requirement_for(control, ControlInteractionTrigger::Focus).is_none() {
        return Err(FocusTargetRejection::DoesNotDeclareFocus(control));
    }
    Ok(control)
}

fn pointer_trigger(kind: PointerEventKind) -> ControlInteractionTrigger {
    match kind {
        PointerEventKind::Move | PointerEventKind::Enter | PointerEventKind::Leave => {
            ControlInteractionTrigger::PointerHover
        }
        PointerEventKind::Down => ControlInteractionTrigger::PointerPress,
        PointerEventKind::Up => ControlInteractionTrigger::PointerRelease,
        PointerEventKind::Scroll => ControlInteractionTrigger::PointerHover,
    }
}

fn requirement_for(
    control: &MountedInteractionControl,
    trigger: ControlInteractionTrigger,
) -> Option<&ControlInteractionRequirement> {
    control
        .descriptor
        .requirements
        .iter()
        .find(|requirement| requirement.trigger == trigger)
}

fn emit_declared_outcomes(
    report: &mut InteractionFormationReport,
    step: &InteractionReplayStep,
    control: &MountedInteractionControl,
    requirement: &ControlInteractionRequirement,
) {
    for declared_outcome in &requirement.outcomes {
        outcome(report, step, control.widget_id, *declared_outcome);
    }
}

fn suppress_disabled_for_trigger(
    report: &mut InteractionFormationReport,
    step: &InteractionReplayStep,
    control: &MountedInteractionControl,
    trigger: ControlInteractionTrigger,
) {
    let Some(requirement) = requirement_for(control, trigger) else {
        suppress_if_declared(report, step, control, "trigger.not_declared");
        return;
    };
    if requirement.suppresses_when_disabled {
        suppress_if_declared(report, step, control, "control.disabled");
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

fn no_target(report: &mut InteractionFormationReport, step: &InteractionReplayStep, reason: &str) {
    report.no_target_events.push(RuntimeNoTargetInteraction {
        step_id: step.step_id.clone(),
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

fn runtime_fact(report: &mut InteractionFormationReport, target: WidgetId, fact: &str) {
    report.runtime_facts.push(RuntimeInteractionFact {
        target,
        fact: fact.to_owned(),
    });
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

fn suppress_if_declared(
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
