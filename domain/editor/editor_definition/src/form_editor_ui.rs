//! Editor-facing formation helpers that remain command-execution neutral.

use crate::{EditorAvailabilityDescriptor, EditorToolbarBinding};
use ui_definition::{
    UiAvailability, UiCollectionItem, UiDefinitionContext, UiRouteSlotId, UiValue,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorToolbarFormationInput {
    pub active_workspace_id: String,
    pub can_undo: bool,
    pub can_redo: bool,
}

pub fn populate_toolbar_context(
    binding: &EditorToolbarBinding,
    input: &EditorToolbarFormationInput,
    context: &mut UiDefinitionContext,
) {
    if let Some(catalog) = &binding.workspace_catalog {
        context.collections.insert(
            catalog.collection_slot.clone(),
            catalog
                .entries
                .iter()
                .map(|entry| {
                    let mut item = UiCollectionItem::new(entry.id.clone(), entry.label.clone());
                    item.selected = entry.id == input.active_workspace_id;
                    item
                })
                .collect(),
        );
        context.selections.insert(
            catalog.active_selection_slot.clone(),
            input.active_workspace_id.clone(),
        );
    }

    for availability in &binding.availability {
        let resolved = match &availability.descriptor {
            EditorAvailabilityDescriptor::Always => UiAvailability::Available,
            EditorAvailabilityDescriptor::RequiresActiveDocument => UiAvailability::Available,
            EditorAvailabilityDescriptor::CanUndo => {
                bool_availability(input.can_undo, "nothing to undo")
            }
            EditorAvailabilityDescriptor::CanRedo => {
                bool_availability(input.can_redo, "nothing to redo")
            }
            EditorAvailabilityDescriptor::StaticDisabled { reason } => UiAvailability::Disabled {
                reason: reason.clone(),
            },
        };
        context
            .availability
            .insert(availability.availability.clone(), resolved);
    }

    context.values.insert(
        "editor.toolbar.title".into(),
        UiValue::Text("Runenwerk".to_string()),
    );
}

pub fn route_slot_id(value: impl Into<String>) -> UiRouteSlotId {
    UiRouteSlotId::new(value)
}

fn bool_availability(value: bool, reason: &str) -> UiAvailability {
    if value {
        UiAvailability::Available
    } else {
        UiAvailability::Disabled {
            reason: reason.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EditorAvailabilityBinding, EditorAvailabilityDescriptor, EditorToolbarBinding};
    use ui_theme::ThemeTokens;

    #[test]
    fn toolbar_context_resolves_can_undo_availability() {
        let binding = EditorToolbarBinding {
            template: "toolbar".into(),
            workspace_catalog: None,
            routes: Vec::new(),
            availability: vec![EditorAvailabilityBinding {
                availability: "can_undo".into(),
                descriptor: EditorAvailabilityDescriptor::CanUndo,
            }],
            menus: Vec::new(),
            menu_items: Vec::new(),
        };
        let mut context = UiDefinitionContext::new(ThemeTokens::default());
        populate_toolbar_context(
            &binding,
            &EditorToolbarFormationInput {
                active_workspace_id: "scene".to_string(),
                can_undo: false,
                can_redo: false,
            },
            &mut context,
        );
        assert!(matches!(
            context.availability.get(&"can_undo".into()),
            Some(UiAvailability::Disabled { .. })
        ));
    }
}
