//! File: domain/ui/ui_controls/src/base_control/lowering/interaction.rs
//! Crate: ui_controls

use crate::{
    ControlInteractionDescriptor, ControlInteractionOutcome, ControlInteractionRequirement,
    ControlInteractionState, ControlInteractionTrigger, ControlKindId,
};

use super::super::{ControlDef, ControlPreset};

pub(crate) fn lower_interaction(
    def: &ControlDef,
    kind_id: ControlKindId,
) -> ControlInteractionDescriptor {
    match def.preset() {
        ControlPreset::Label => ControlInteractionDescriptor::new(kind_id)
            .with_states([
                ControlInteractionState::Enabled,
                ControlInteractionState::Disabled,
            ])
            .with_requirement(
                ControlInteractionRequirement::new(ControlInteractionTrigger::SemanticAction)
                    .with_outcome(ControlInteractionOutcome::InspectionRequested),
            ),
        ControlPreset::Button => activatable(kind_id),
        ControlPreset::ActionPrompt => action_prompt(kind_id),
        ControlPreset::InspectorField => focusable(kind_id)
            .with_text_intent_probe(true)
            .with_requirement(
                ControlInteractionRequirement::new(ControlInteractionTrigger::TextIntent)
                    .requiring_focus()
                    .with_outcome(ControlInteractionOutcome::TextIntentSeen),
            ),
        ControlPreset::ColorPicker => focusable(kind_id).with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::KeyboardActivate)
                .requiring_focus()
                .with_outcome(ControlInteractionOutcome::OpenRequested),
        ),
        ControlPreset::ListView => focusable(kind_id).with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::KeyboardNavigate)
                .requiring_focus()
                .with_outcome(ControlInteractionOutcome::ActiveItemIntent),
        ),
        ControlPreset::TreeView => focusable(kind_id).with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::KeyboardNavigate)
                .requiring_focus()
                .with_outcome(ControlInteractionOutcome::NodeIntent),
        ),
        ControlPreset::TableView => focusable(kind_id).with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::KeyboardNavigate)
                .requiring_focus()
                .with_outcome(ControlInteractionOutcome::CellOrRowIntent),
        ),
    }
}

fn activatable(kind_id: ControlKindId) -> ControlInteractionDescriptor {
    focusable(kind_id)
        .with_requirement(ControlInteractionRequirement::new(
            ControlInteractionTrigger::PointerHover,
        ))
        .with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::PointerPress)
                .with_outcome(ControlInteractionOutcome::ActivationRequested),
        )
        .with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::KeyboardActivate)
                .requiring_focus()
                .with_outcome(ControlInteractionOutcome::ActivationRequested),
        )
        .with_requirement(ControlInteractionRequirement::new(
            ControlInteractionTrigger::PointerCancel,
        ))
}

fn action_prompt(kind_id: ControlKindId) -> ControlInteractionDescriptor {
    focusable(kind_id)
        .with_requirement(ControlInteractionRequirement::new(
            ControlInteractionTrigger::PointerHover,
        ))
        .with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::PointerPress)
                .with_outcome(ControlInteractionOutcome::ActionIntent),
        )
        .with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::KeyboardActivate)
                .requiring_focus()
                .with_outcome(ControlInteractionOutcome::ActionIntent),
        )
        .with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::SemanticAction)
                .with_outcome(ControlInteractionOutcome::ActionIntent),
        )
}

fn focusable(kind_id: ControlKindId) -> ControlInteractionDescriptor {
    ControlInteractionDescriptor::new(kind_id).with_requirement(ControlInteractionRequirement::new(
        ControlInteractionTrigger::Focus,
    ))
}
