//! File: domain/ui/ui_controls/src/interaction.rs
//! Crate: ui_controls

use serde::{Deserialize, Serialize};

use crate::package::ids::ControlKindId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlInteractionState {
    Enabled,
    Disabled,
    ReadOnly,
    Hovered,
    Pressed,
    Active,
    Focused,
    FocusVisible,
    Captured,
    Suppressed,
}

impl ControlInteractionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::ReadOnly => "read-only",
            Self::Hovered => "hovered",
            Self::Pressed => "pressed",
            Self::Active => "active",
            Self::Focused => "focused",
            Self::FocusVisible => "focus-visible",
            Self::Captured => "captured",
            Self::Suppressed => "suppressed",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlInteractionStateSet {
    #[serde(default)]
    pub states: Vec<ControlInteractionState>,
}

impl ControlInteractionStateSet {
    pub fn new(states: impl IntoIterator<Item = ControlInteractionState>) -> Self {
        let mut states = states.into_iter().collect::<Vec<_>>();
        states.sort();
        states.dedup();
        Self { states }
    }

    pub fn contains(&self, state: ControlInteractionState) -> bool {
        self.states.contains(&state)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlInteractionTrigger {
    PointerHover,
    PointerPress,
    PointerCancel,
    Focus,
    KeyboardActivate,
    KeyboardNavigate,
    SemanticAction,
    TextIntent,
}

impl ControlInteractionTrigger {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PointerHover => "pointer-hover",
            Self::PointerPress => "pointer-press",
            Self::PointerCancel => "pointer-cancel",
            Self::Focus => "focus",
            Self::KeyboardActivate => "keyboard-activate",
            Self::KeyboardNavigate => "keyboard-navigate",
            Self::SemanticAction => "semantic-action",
            Self::TextIntent => "text-intent",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlInteractionOutcome {
    ActivationRequested,
    ActionIntent,
    ActiveItemIntent,
    NodeIntent,
    CellOrRowIntent,
    TextIntentSeen,
    InspectionRequested,
    OpenRequested,
}

impl ControlInteractionOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationRequested => "activation-requested",
            Self::ActionIntent => "action-intent",
            Self::ActiveItemIntent => "active-item-intent",
            Self::NodeIntent => "node-intent",
            Self::CellOrRowIntent => "cell-or-row-intent",
            Self::TextIntentSeen => "text-intent-seen",
            Self::InspectionRequested => "inspection-requested",
            Self::OpenRequested => "open-requested",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlInteractionRequirement {
    pub trigger: ControlInteractionTrigger,
    #[serde(default)]
    pub outcomes: Vec<ControlInteractionOutcome>,
    #[serde(default)]
    pub requires_focus: bool,
    #[serde(default = "default_suppresses_when_disabled")]
    pub suppresses_when_disabled: bool,
}

impl ControlInteractionRequirement {
    pub fn new(trigger: ControlInteractionTrigger) -> Self {
        Self {
            trigger,
            outcomes: Vec::new(),
            requires_focus: false,
            suppresses_when_disabled: true,
        }
    }

    pub fn with_outcome(mut self, outcome: ControlInteractionOutcome) -> Self {
        self.outcomes.push(outcome);
        self.outcomes.sort();
        self.outcomes.dedup();
        self
    }

    pub fn requiring_focus(mut self) -> Self {
        self.requires_focus = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlInteractionDescriptor {
    pub control_kind_id: ControlKindId,
    pub states: ControlInteractionStateSet,
    #[serde(default)]
    pub requirements: Vec<ControlInteractionRequirement>,
    #[serde(default)]
    pub text_intent_probe: bool,
}

impl ControlInteractionDescriptor {
    pub fn new(control_kind_id: ControlKindId) -> Self {
        Self {
            control_kind_id,
            states: ControlInteractionStateSet::new([
                ControlInteractionState::Enabled,
                ControlInteractionState::Disabled,
                ControlInteractionState::Hovered,
                ControlInteractionState::Pressed,
                ControlInteractionState::Active,
                ControlInteractionState::Focused,
                ControlInteractionState::FocusVisible,
                ControlInteractionState::Captured,
                ControlInteractionState::Suppressed,
            ]),
            requirements: Vec::new(),
            text_intent_probe: false,
        }
    }

    pub fn with_states(
        mut self,
        states: impl IntoIterator<Item = ControlInteractionState>,
    ) -> Self {
        self.states = ControlInteractionStateSet::new(states);
        self
    }

    pub fn with_requirement(mut self, requirement: ControlInteractionRequirement) -> Self {
        self.requirements.push(requirement);
        self.requirements
            .sort_by_key(|requirement| requirement.trigger);
        self.requirements
            .dedup_by_key(|requirement| requirement.trigger);
        self
    }

    pub fn with_text_intent_probe(mut self, text_intent_probe: bool) -> Self {
        self.text_intent_probe = text_intent_probe;
        self
    }

    pub fn summary(&self) -> ControlInteractionSupportSummary {
        ControlInteractionSupportSummary::from_descriptor(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlInteractionSupportSummary {
    pub control_kind_id: ControlKindId,
    pub states: Vec<String>,
    pub triggers: Vec<String>,
    pub outcomes: Vec<String>,
    pub requires_focus: bool,
    pub text_intent_probe: bool,
    pub has_runtime_behavior: bool,
    pub executes_host_commands: bool,
    pub mutates_product_state: bool,
}

impl ControlInteractionSupportSummary {
    pub fn from_descriptor(descriptor: &ControlInteractionDescriptor) -> Self {
        let mut triggers = descriptor
            .requirements
            .iter()
            .map(|requirement| requirement.trigger.as_str().to_owned())
            .collect::<Vec<_>>();
        let mut outcomes = descriptor
            .requirements
            .iter()
            .flat_map(|requirement| requirement.outcomes.iter())
            .map(|outcome| outcome.as_str().to_owned())
            .collect::<Vec<_>>();

        triggers.sort();
        triggers.dedup();
        outcomes.sort();
        outcomes.dedup();

        Self {
            control_kind_id: descriptor.control_kind_id.clone(),
            states: descriptor
                .states
                .states
                .iter()
                .map(|state| state.as_str().to_owned())
                .collect(),
            triggers,
            outcomes,
            requires_focus: descriptor
                .requirements
                .iter()
                .any(|requirement| requirement.requires_focus),
            text_intent_probe: descriptor.text_intent_probe,
            has_runtime_behavior: false,
            executes_host_commands: false,
            mutates_product_state: false,
        }
    }

    pub fn inspection_facts(&self) -> Vec<ControlInteractionInspectionFact> {
        vec![
            ControlInteractionInspectionFact::new("states", self.states.join(",")),
            ControlInteractionInspectionFact::new("triggers", self.triggers.join(",")),
            ControlInteractionInspectionFact::new("outcomes", self.outcomes.join(",")),
            ControlInteractionInspectionFact::new(
                "requires_focus",
                bool_string(self.requires_focus),
            ),
            ControlInteractionInspectionFact::new(
                "text_intent_probe",
                bool_string(self.text_intent_probe),
            ),
            ControlInteractionInspectionFact::new(
                "has_runtime_behavior",
                bool_string(self.has_runtime_behavior),
            ),
            ControlInteractionInspectionFact::new(
                "executes_host_commands",
                bool_string(self.executes_host_commands),
            ),
            ControlInteractionInspectionFact::new(
                "mutates_product_state",
                bool_string(self.mutates_product_state),
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlInteractionInspectionFact {
    pub key: String,
    pub value: String,
}

impl ControlInteractionInspectionFact {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

fn default_suppresses_when_disabled() -> bool {
    true
}

fn bool_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
