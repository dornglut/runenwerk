//! Operation helper ownership for UI Designer self-authoring.

use super::*;

pub(super) fn operation_history_document_snapshot(document: &EditorDefinitionDocument) -> String {
    ron::ser::to_string_pretty(document, PrettyConfig::default())
        .unwrap_or_else(|error| format!("snapshot serialization failed: {error}"))
}

pub(super) fn first_text_editable_ui_node_id(
    node: &ui_definition::UiNodeDefinition,
) -> Option<String> {
    match node {
        ui_definition::UiNodeDefinition::Label { id, .. }
        | ui_definition::UiNodeDefinition::Button { id, .. }
        | ui_definition::UiNodeDefinition::Toggle { id, .. }
        | ui_definition::UiNodeDefinition::TextInput { id, .. } => {
            return Some(id.as_str().to_string());
        }
        _ => {}
    }

    node.children()
        .iter()
        .find_map(first_text_editable_ui_node_id)
}
