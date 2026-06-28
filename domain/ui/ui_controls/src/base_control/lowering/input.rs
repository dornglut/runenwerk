//! File: domain/ui/ui_controls/src/base_control/lowering/input.rs
//! Crate: ui_controls

use crate::{
    ControlInputDescriptor, ControlInputMode, ControlKeyboardRequirement, ControlKindId,
    ControlSemanticActionRequirement, ControlWheelRequirement,
};

use super::super::{ControlDef, ControlPreset};

pub(crate) fn lower_input(def: &ControlDef, kind_id: ControlKindId) -> ControlInputDescriptor {
    match def.preset() {
        ControlPreset::Label => add_semantic_actions(
            ControlInputDescriptor::new(kind_id).with_modes([ControlInputMode::SemanticAction]),
            &["inspect"],
        ),
        ControlPreset::Button => add_semantic_actions(
            focused_input(
                kind_id,
                &[
                    ControlInputMode::Pointer,
                    ControlInputMode::Keyboard,
                    ControlInputMode::SemanticAction,
                    ControlInputMode::TouchReady,
                    ControlInputMode::Controller,
                ],
            ),
            &["activate"],
        ),
        ControlPreset::InspectorField => add_semantic_actions(
            focused_input(
                kind_id,
                &[
                    ControlInputMode::Pointer,
                    ControlInputMode::Keyboard,
                    ControlInputMode::SemanticAction,
                ],
            ),
            &["commit-value"],
        ),
        ControlPreset::ColorPicker => add_semantic_actions(
            focused_input(
                kind_id,
                &[
                    ControlInputMode::Pointer,
                    ControlInputMode::Keyboard,
                    ControlInputMode::SemanticAction,
                    ControlInputMode::TouchReady,
                    ControlInputMode::Controller,
                ],
            ),
            &["preview-color", "commit-color"],
        ),
        ControlPreset::ActionPrompt => add_semantic_actions(
            focused_input(
                kind_id,
                &[
                    ControlInputMode::Pointer,
                    ControlInputMode::Keyboard,
                    ControlInputMode::SemanticAction,
                    ControlInputMode::Controller,
                ],
            ),
            &["accept", "cancel"],
        ),
        ControlPreset::ListView | ControlPreset::TreeView | ControlPreset::TableView => {
            add_semantic_actions(
                focused_input(
                    kind_id,
                    &[
                        ControlInputMode::Pointer,
                        ControlInputMode::Wheel,
                        ControlInputMode::Keyboard,
                        ControlInputMode::SemanticAction,
                        ControlInputMode::TouchReady,
                        ControlInputMode::Controller,
                    ],
                )
                .with_wheel(ControlWheelRequirement {
                    requires_scroll_delta: true,
                    requires_zoom_delta: false,
                }),
                &["select", "navigate"],
            )
        }
    }
}

fn focused_input(kind_id: ControlKindId, modes: &[ControlInputMode]) -> ControlInputDescriptor {
    ControlInputDescriptor::new(kind_id)
        .with_modes(modes.iter().copied())
        .with_keyboard(ControlKeyboardRequirement {
            requires_focus: true,
            requires_shortcuts: false,
        })
}

fn add_semantic_actions(
    mut descriptor: ControlInputDescriptor,
    values: &[&str],
) -> ControlInputDescriptor {
    for value in values {
        descriptor = descriptor.with_semantic_action(ControlSemanticActionRequirement::new(*value));
    }
    descriptor
}
