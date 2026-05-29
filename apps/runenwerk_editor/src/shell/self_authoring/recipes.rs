//! Recipe helper ownership for UI Designer self-authoring.

use super::*;

pub(super) fn operation_id_from_recipe(recipe_id: &UiRecipeId, inserted_node_id: &str) -> String {
    format!("recipe.{}.{}", recipe_id.as_str(), inserted_node_id)
}

pub(super) fn namespace_ui_recipe_node_ids(
    node: &mut UiNodeDefinition,
    recipe_id: &UiRecipeId,
    sequence: u64,
) {
    let original_id = node.id().as_str().to_string();
    set_ui_node_id(
        node,
        AuthoredId::new(format!(
            "recipe.{sequence:04}.{}.{}",
            recipe_id.as_str(),
            original_id
        )),
    );
    if let Some(children) = node.children_mut() {
        for child in children {
            namespace_ui_recipe_node_ids(child, recipe_id, sequence);
        }
    }
}

fn set_ui_node_id(node: &mut UiNodeDefinition, replacement: AuthoredId) {
    match node {
        UiNodeDefinition::Panel { id, .. }
        | UiNodeDefinition::Row { id, .. }
        | UiNodeDefinition::Column { id, .. }
        | UiNodeDefinition::Stack { id, .. }
        | UiNodeDefinition::Scroll { id, .. }
        | UiNodeDefinition::Split { id, .. }
        | UiNodeDefinition::Spacer { id }
        | UiNodeDefinition::Separator { id, .. }
        | UiNodeDefinition::Label { id, .. }
        | UiNodeDefinition::Button { id, .. }
        | UiNodeDefinition::Toggle { id, .. }
        | UiNodeDefinition::TextInput { id, .. }
        | UiNodeDefinition::NumericInput { id, .. }
        | UiNodeDefinition::Select { id, .. }
        | UiNodeDefinition::Tabs { id, .. }
        | UiNodeDefinition::Table { id, .. }
        | UiNodeDefinition::Tree { id, .. }
        | UiNodeDefinition::Repeat { id, .. }
        | UiNodeDefinition::TemplateRef { id, .. }
        | UiNodeDefinition::MenuSlot { id, .. }
        | UiNodeDefinition::EmbedSlot { id, .. } => *id = replacement,
    }
}
