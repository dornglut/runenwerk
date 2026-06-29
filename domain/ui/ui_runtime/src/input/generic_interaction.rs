//! Generic reusable interaction proof for `ui_runtime`.
//!
//! This module resolves normalized `ui_input` facts against mounted controls
//! that carry package-backed `ui_controls` interaction descriptors. It forms
//! replay reports and visible proof models that later adapters can project into
//! static render evidence.
//!
//! It deliberately does not own OS/window input collection, app/editor/game
//! command execution, product mutation, overlay/layering behavior, or full text
//! editing.

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
    /// Stable fixture/story id used by replay reports and proof adapters.
    pub mounted_story_id: String,

    /// Mounted controls with package-backed interaction descriptors.
    pub controls: Vec<MountedInteractionControl>,
}

impl MountedInteractionFixture {
    /// Creates an empty deterministic interaction fixture.
    pub fn new(mounted_story_id: impl Into<String>) -> Self {
        Self {
            mounted_story_id: mounted_story_id.into(),
            controls: Vec::new(),
        }
    }

    /// Adds one mounted control to the fixture.
    pub fn with_control(mut self, control: MountedInteractionControl) -> Self {
        self.controls.push(control);
        self
    }

    /// Builds a fixture from compiled package interaction descriptors.
    ///
    /// The compiled package is the authority for interaction declarations. This
    /// panics when a placement references a control kind without a package-level
    /// interaction descriptor so replay proofs cannot silently use fake data.
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
    /// Widget id assigned to the mounted proof control.
    pub widget_id: WidgetId,

    /// Control kind id that must resolve through the compiled package.
    pub control_kind_id: String,

    /// Human-readable label rendered by proof adapters.
    pub label: String,

    /// Renderer-neutral hit-test bounds for deterministic replay.
    pub bounds: UiRect,

    /// Whether reusable runtime interaction may target this control.
    pub enabled: bool,

    /// Whether focus traversal and explicit focus may target this control.
    pub focusable: bool,

    /// Whether text intent is observed as read-only probe evidence.
    pub read_only: bool,
}

impl MountedInteractionPlacement {
    /// Creates an enabled, focusable placement for a package-backed control.
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

    /// Marks the placement disabled for suppression proof cases.
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Marks the placement inert so focus validation can reject it.
    pub fn inert(mut self) -> Self {
        self.focusable = false;
        self
    }

    /// Marks the placement read-only for text-intent probe evidence.
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }
}

/// Mounted control plus the package-owned interaction descriptor copy.
#[derive(Debug, Clone, PartialEq)]
pub struct MountedInteractionControl {
    /// Widget id assigned to the mounted proof control.
    pub widget_id: WidgetId,

    /// Package control kind id copied from the interaction descriptor.
    pub control_kind_id: String,

    /// Human-readable proof label.
    pub label: String,

    /// Renderer-neutral bounds used by deterministic pointer hit testing.
    pub bounds: UiRect,

    /// Package-backed reusable interaction declaration for this control.
    pub descriptor: ControlInteractionDescriptor,

    /// Whether replay may form interaction for this control.
    pub enabled: bool,

    /// Whether focus replay may resolve this control.
    pub focusable: bool,

    /// Whether text intent is observed without edit ownership.
    pub read_only: bool,
}

impl MountedInteractionControl {
    /// Creates a mounted control from a package-backed interaction descriptor.
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

    /// Marks the mounted control disabled for suppression proof cases.
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Marks the mounted control inert so focus validation can reject it.
    pub fn inert(mut self) -> Self {
        self.focusable = false;
        self
    }

    /// Marks the mounted control read-only for text-intent probe evidence.
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }
}

/// Deterministic sequence of normalized input samples for interaction replay.
#[derive(Debug, Clone, PartialEq)]
pub struct InteractionReplayScript {
    /// Stable replay id recorded by the formation report.
    pub replay_id: String,

    /// Ordered normalized input samples resolved by replay.
    pub steps: Vec<InteractionReplayStep>,
}

impl InteractionReplayScript {
    /// Creates an empty deterministic replay script.
    pub fn new(replay_id: impl Into<String>) -> Self {
        Self {
            replay_id: replay_id.into(),
            steps: Vec::new(),
        }
    }

    /// Appends one replay step.
    pub fn with_step(mut self, step: InteractionReplayStep) -> Self {
        self.steps.push(step);
        self
    }
}

/// One replay step and the normalized input sample formed by `ui_input`.
#[derive(Debug, Clone, PartialEq)]
pub struct InteractionReplayStep {
    /// Stable step id used by runtime report rows.
    pub step_id: String,

    /// Normalized input sample to resolve for this step.
    pub sample: NormalizedInputSample,
}

impl InteractionReplayStep {
    /// Creates one replay step from a normalized input sample.
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
    /// Replay id copied from the script.
    pub replay_id: String,

    /// Fixture/story id copied from the mounted fixture.
    pub mounted_story_id: String,

    /// Descriptor facts proving replay used package-backed declarations.
    pub control_descriptors: Vec<RuntimeControlDescriptorFact>,

    /// Ordered input step ids processed by replay.
    pub input_steps: Vec<String>,

    /// Pointer/target resolution rows.
    pub target_resolution: Vec<InteractionTargetResolution>,

    /// Focus resolution rows.
    pub focus_resolution: Vec<InteractionFocusResolution>,

    /// Stateful interaction transitions emitted by replay.
    pub state_transitions: Vec<InteractionStateTransition>,

    /// Reusable runtime facts observed during replay.
    pub runtime_facts: Vec<RuntimeInteractionFact>,

    /// Reusable runtime events observed during replay.
    pub runtime_events: Vec<RuntimeControlInteractionEvent>,

    /// Semantic outcomes emitted only from declared requirements.
    pub semantic_outcomes: Vec<RuntimeInteractionOutcome>,

    /// Suppressed interaction evidence for negative cases.
    pub suppressed_events: Vec<RuntimeSuppressedInteraction>,

    /// No-target evidence for unmatched input.
    pub no_target_events: Vec<RuntimeNoTargetInteraction>,

    /// Boundary counters proving no host/product/overlay/text-edit bypass.
    pub boundary_assertions: InteractionBoundaryAssertions,
}

/// Descriptor fact captured for one mounted control during replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeControlDescriptorFact {
    /// Mounted widget id for the descriptor fact.
    pub widget_id: WidgetId,

    /// Package control kind id for the mounted control.
    pub control_kind_id: String,

    /// Declared reusable states copied from the package descriptor.
    pub interaction_states: Vec<String>,

    /// Declared reusable triggers copied from the package descriptor.
    pub interaction_triggers: Vec<String>,

    /// Declared semantic outcomes copied from the package descriptor.
    pub interaction_outcomes: Vec<String>,
}

/// Target resolution evidence for one input step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionTargetResolution {
    /// Replay step id that produced this resolution.
    pub step_id: String,

    /// Resolved target, when a target existed.
    pub target: Option<WidgetId>,

    /// Stable resolution reason.
    pub reason: String,
}

/// Focus resolution evidence for one input step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionFocusResolution {
    /// Replay step id that produced this resolution.
    pub step_id: String,

    /// Focused widget after resolution.
    pub focused: Option<WidgetId>,

    /// Whether focus-visible is active after resolution.
    pub focus_visible: bool,

    /// Stable focus resolution reason.
    pub reason: String,
}

/// Stateful interaction transition emitted by replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionStateTransition {
    /// Replay step id that produced the transition.
    pub step_id: String,

    /// Target widget for the transition.
    pub target: WidgetId,

    /// Stable state name.
    pub state: String,

    /// Whether the state became active or inactive.
    pub active: bool,
}

/// Runtime fact formed from normalized input and descriptor requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInteractionFact {
    /// Target widget for the fact.
    pub target: WidgetId,

    /// Stable fact name.
    pub fact: String,
}

/// Runtime event formed from normalized input and descriptor requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeControlInteractionEvent {
    /// Replay step id that produced the event.
    pub step_id: String,

    /// Target widget for the event.
    pub target: WidgetId,

    /// Stable event name.
    pub event: String,
}

/// Semantic reusable outcome emitted from a declared requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInteractionOutcome {
    /// Replay step id that produced the outcome.
    pub step_id: String,

    /// Target widget for the outcome.
    pub target: WidgetId,

    /// Stable semantic outcome name.
    pub outcome: String,
}

/// Suppressed reusable interaction evidence for one negative case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSuppressedInteraction {
    /// Replay step id that produced the suppression.
    pub step_id: String,

    /// Target widget whose interaction was suppressed.
    pub target: WidgetId,

    /// Stable suppression reason.
    pub reason: String,
}

/// No-target reusable interaction evidence for one unmatched input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeNoTargetInteraction {
    /// Replay step id that produced the no-target result.
    pub step_id: String,

    /// Stable no-target reason.
    pub reason: String,
}

/// Boundary counters for no-bypass evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InteractionBoundaryAssertions {
    /// Host command executions observed during replay.
    pub host_commands_executed: u32,

    /// Product/app/editor/game state mutations observed during replay.
    pub product_mutations: u32,

    /// Overlay, popup, dropdown, tooltip, or layering events observed.
    pub overlay_events: u32,

    /// Full text-editing transactions observed during replay.
    pub text_edit_transactions: u32,
}

impl InteractionBoundaryAssertions {
    /// Returns true when replay produced no host/product/overlay/text-edit bypass.
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
    /// Stable proof identifier.
    pub proof_id: String,

    /// Main visible proof view containing mounted controls.
    pub main_view: InteractionVisualMainView,

    /// Inspector visible proof view for the selected control.
    pub inspector_view: InteractionInspectorView,

    /// Report/event visible proof view.
    pub report_view: InteractionReportView,
}

impl InteractionVisualProof {
    /// Projects a descriptor-backed interaction replay into visible proof data.
    ///
    /// The proof contains three renderer-neutral views: main, inspector, and
    /// report/event. It is proof evidence rather than product UI.
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
    /// Fixture/story id shown by the main proof view.
    pub mounted_story_id: String,

    /// Mounted controls and their observed/current visual state evidence.
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

    /// Finds one visible proof control by widget id.
    pub fn control(&self, widget_id: WidgetId) -> Option<&InteractionVisualControl> {
        self.controls
            .iter()
            .find(|control| control.widget_id == widget_id)
    }
}

/// One visible mounted control in the Phase 12 proof model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionVisualControl {
    /// Mounted widget id represented by this proof control.
    pub widget_id: WidgetId,

    /// Package control kind id represented by this proof control.
    pub control_kind_id: String,

    /// Human-readable label rendered by proof adapters.
    pub label: String,

    /// Visual evidence observed at any point during replay.
    pub observed_markers: Vec<InteractionVisualMarker>,

    /// Final reusable state after replay transitions are folded in order.
    pub current_states: Vec<InteractionVisibleState>,
}

impl InteractionVisualControl {
    fn from_report(
        control: &MountedInteractionControl,
        report: &InteractionFormationReport,
    ) -> Self {
        let mut observed_markers = vec![InteractionVisualMarker::new(
            InteractionVisibleState::Normal,
        )];
        if !control.enabled {
            push_marker(&mut observed_markers, InteractionVisibleState::Disabled);
        }
        if control.read_only {
            push_marker(&mut observed_markers, InteractionVisibleState::ReadOnly);
        }
        for transition in report
            .state_transitions
            .iter()
            .filter(|transition| transition.target == control.widget_id)
        {
            if let Some(state) = visible_state_from_transition(&transition.state) {
                push_marker(&mut observed_markers, state);
            }
        }
        for fact in report
            .runtime_facts
            .iter()
            .filter(|fact| fact.target == control.widget_id)
        {
            if let Some(state) = visible_state_from_fact(&fact.fact) {
                push_marker(&mut observed_markers, state);
            }
        }
        for outcome in report
            .semantic_outcomes
            .iter()
            .filter(|outcome| outcome.target == control.widget_id)
        {
            if let Some(state) = visible_state_from_outcome(&outcome.outcome) {
                push_marker(&mut observed_markers, state);
            }
        }
        if report
            .suppressed_events
            .iter()
            .any(|event| event.target == control.widget_id)
        {
            push_marker(&mut observed_markers, InteractionVisibleState::Suppressed);
        }

        Self {
            widget_id: control.widget_id,
            control_kind_id: control.control_kind_id.clone(),
            label: control.label.clone(),
            observed_markers,
            current_states: folded_current_states(control, report),
        }
    }

    /// Returns true when the replay observed a marker for this control.
    pub fn has_marker(&self, state: InteractionVisibleState) -> bool {
        self.observed_markers
            .iter()
            .any(|marker| marker.state == state)
    }

    /// Returns true when the folded final state contains this state.
    pub fn has_current_state(&self, state: InteractionVisibleState) -> bool {
        self.current_states.contains(&state)
    }
}

/// A named visible marker, such as hovered, pressed, or text-intent probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionVisualMarker {
    /// Stable visible state represented by this marker.
    pub state: InteractionVisibleState,

    /// Human-readable marker label.
    pub label: String,
}

impl InteractionVisualMarker {
    /// Creates a marker with the state's stable label.
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
    /// Normal baseline proof marker.
    Normal,

    /// Pointer hover was observed.
    Hovered,

    /// Pointer press was observed.
    Pressed,

    /// Pointer capture was observed.
    Captured,

    /// Focus was observed.
    Focused,

    /// Keyboard-visible focus was observed.
    FocusVisible,

    /// Active/navigation state was observed.
    Active,

    /// Disabled state or suppression target was observed.
    Disabled,

    /// Read-only state was observed.
    ReadOnly,

    /// Suppressed interaction evidence was observed.
    Suppressed,

    /// Activation was requested by a declared reusable requirement.
    ActivationRequested,

    /// Action intent was emitted by a declared reusable requirement.
    ActionIntent,

    /// List active-item intent was emitted.
    ListActiveItemIntent,

    /// Tree node intent was emitted.
    TreeNodeIntent,

    /// Table cell or row intent was emitted.
    TableCellOrRowIntent,

    /// Text intent was observed as a probe, without editing.
    TextIntentProbe,

    /// Read-only text intent was observed as a probe, without editing.
    ReadOnlyTextIntentProbe,
}

impl InteractionVisibleState {
    /// Returns the stable report/proof label for this visible state.
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
    /// Selected widget shown by the inspector proof view.
    pub selected_widget: Option<WidgetId>,

    /// Selected control kind id.
    pub control_kind_id: Option<String>,

    /// Declared trigger and outcome requirements for the selected control.
    pub declared_requirements: Vec<String>,

    /// States observed during replay for the selected control.
    pub observed_reusable_interaction_states: Vec<InteractionVisibleState>,

    /// Final state set after active/inactive transitions are folded.
    pub current_reusable_interaction_state_set: Vec<InteractionVisibleState>,

    /// Whether the descriptor declares text-intent probe support.
    pub text_intent_probe: bool,

    /// Whether the mounted selected control is read-only.
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
                observed_reusable_interaction_states: Vec::new(),
                current_reusable_interaction_state_set: Vec::new(),
                text_intent_probe: false,
                read_only: false,
            };
        };
        let mut observed_state_set = Vec::new();
        for transition in report
            .state_transitions
            .iter()
            .filter(|transition| transition.target == control.widget_id)
        {
            if let Some(state) = visible_state_from_transition(&transition.state)
                && !observed_state_set.contains(&state)
            {
                observed_state_set.push(state);
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
            observed_reusable_interaction_states: observed_state_set,
            current_reusable_interaction_state_set: folded_current_states(control, report),
            text_intent_probe: control.descriptor.text_intent_probe,
            read_only: control.read_only,
        }
    }
}

/// Report/event pane data for the Phase 12 visual proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionReportView {
    /// Ordered replay step ids.
    pub replay_steps: Vec<String>,

    /// Target resolution rows formatted for renderer-neutral proof output.
    pub target_resolution: Vec<String>,

    /// Focus resolution rows formatted for renderer-neutral proof output.
    pub focus_resolution: Vec<String>,

    /// State transition rows formatted for renderer-neutral proof output.
    pub state_transitions: Vec<String>,

    /// Runtime fact rows formatted for renderer-neutral proof output.
    pub runtime_facts: Vec<String>,

    /// Runtime event rows formatted for renderer-neutral proof output.
    pub runtime_events: Vec<String>,

    /// Semantic outcome rows formatted for renderer-neutral proof output.
    pub semantic_outcomes: Vec<String>,

    /// Suppressed event rows formatted for renderer-neutral proof output.
    pub suppressed_events: Vec<String>,

    /// No-target rows formatted for renderer-neutral proof output.
    pub no_target_events: Vec<String>,

    /// Boundary counters proving no bypass occurred.
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
    /// Semantic visible proof model carried by this proof frame.
    pub proof: InteractionVisualProof,
}

impl InteractionProofFrame {
    /// Wraps a semantic visible proof in a reusable proof frame.
    pub fn new(proof: InteractionVisualProof) -> Self {
        Self { proof }
    }
}

fn push_marker(markers: &mut Vec<InteractionVisualMarker>, state: InteractionVisibleState) {
    if !markers.iter().any(|marker| marker.state == state) {
        markers.push(InteractionVisualMarker::new(state));
    }
}

fn folded_current_states(
    control: &MountedInteractionControl,
    report: &InteractionFormationReport,
) -> Vec<InteractionVisibleState> {
    let mut states = Vec::new();
    if !control.enabled {
        push_state(&mut states, InteractionVisibleState::Disabled);
    }
    if control.read_only {
        push_state(&mut states, InteractionVisibleState::ReadOnly);
    }

    for transition in report
        .state_transitions
        .iter()
        .filter(|transition| transition.target == control.widget_id)
    {
        let Some(state) = visible_state_from_transition(&transition.state) else {
            continue;
        };
        if transition.active {
            push_state(&mut states, state);
        } else {
            remove_state(&mut states, state);
        }
    }
    states
}

fn push_state(states: &mut Vec<InteractionVisibleState>, state: InteractionVisibleState) {
    if !states.contains(&state) {
        states.push(state);
    }
}

fn remove_state(states: &mut Vec<InteractionVisibleState>, state: InteractionVisibleState) {
    states.retain(|existing| *existing != state);
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

/// Replays normalized input facts against mounted descriptor-backed controls.
///
/// Replay emits reusable facts, events, outcomes, negative evidence, and
/// boundary counters only. It does not execute host commands, mutate product
/// state, open overlays, or perform text editing.
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
                set_focus_state(fixture, step, state, report, Some(control.widget_id), false);
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
        let next = traverse_focus(fixture, state.focused, direction);
        set_focus_state(fixture, step, state, report, next, focus.focus_visible);
        reason = if next.is_some() {
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
                        set_focus_state(
                            fixture,
                            step,
                            state,
                            report,
                            Some(control.widget_id),
                            focus.focus_visible,
                        );
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
                set_focus_state(fixture, step, state, report, None, false);
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
}

fn set_focus_state(
    fixture: &MountedInteractionFixture,
    step: &InteractionReplayStep,
    state: &mut ReplayState,
    report: &mut InteractionFormationReport,
    next: Option<WidgetId>,
    next_focus_visible: bool,
) {
    let previous = state.focused;
    let previous_focus_visible = state.focus_visible;

    if previous != next {
        if let Some(previous) = previous {
            transition(report, step, previous, "focused", false);
            if previous_focus_visible {
                transition(report, step, previous, "focus-visible", false);
            }
        }
        state.focused = next;
        state.focus_visible = next_focus_visible;
        if let Some(next) = next
            && let Some(control) = fixture.control(next)
            && requirement_for(control, ControlInteractionTrigger::Focus).is_some()
        {
            transition(report, step, next, "focused", true);
            if next_focus_visible {
                transition(report, step, next, "focus-visible", true);
            }
        }
        return;
    }

    state.focus_visible = next_focus_visible;
    if previous_focus_visible != next_focus_visible
        && let Some(current) = next
    {
        transition(report, step, current, "focus-visible", next_focus_visible);
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
