//! Recipe insertion lifecycle ownership for UI Designer self-authoring.

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
        | UiNodeDefinition::Control { id, .. }
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

impl SelfAuthoringWorkspaceState {
    pub fn insert_selected_ui_recipe(
        &mut self,
        library: &UiRecipeLibrary,
        recipe_id: UiRecipeId,
        target_profile: UiRecipeTargetProfileId,
    ) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.recipe.no_selection",
                "no definition document is selected for recipe insertion",
            )
        })?;
        let before = self.drafts.get(&document_id).cloned().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.recipe.unresolved_document",
                "selected definition document is not loaded",
            )
        })?;
        let operation_id = self.next_operation_id("recipe_insert");
        let target_profile_label = target_profile.as_str().to_string();

        if !matches!(
            &before.content,
            EditorDefinitionDocumentContent::UiTemplate(_)
        ) {
            return self.reject_recipe_insert(
                operation_id,
                document_id,
                target_profile_label,
                before,
                vec![UiDefinitionDiagnostic::error(
                    "editor.self_authoring.recipe.not_ui_template",
                    "selected definition is not a UI template",
                )],
            );
        }

        let expansion = expand_ui_recipe(
            library,
            &UiRecipeExpansionRequest::activate(recipe_id.clone(), target_profile),
        );
        if expansion.has_errors() {
            return self.reject_recipe_insert(
                operation_id,
                document_id,
                target_profile_label,
                before,
                expansion.as_definition_diagnostics(),
            );
        }
        let Some(mut inserted_root) = expansion.root else {
            return self.reject_recipe_insert(
                operation_id,
                document_id,
                target_profile_label,
                before,
                vec![UiDefinitionDiagnostic::error(
                    "editor.self_authoring.recipe.empty_expansion",
                    format!(
                        "recipe '{}' did not produce an insertable root node",
                        recipe_id
                    ),
                )],
            );
        };

        namespace_ui_recipe_node_ids(
            &mut inserted_root,
            &recipe_id,
            self.operation_history.next_sequence + 1,
        );
        let inserted_node_id = inserted_root.id().as_str().to_string();

        let EditorDefinitionDocumentContent::UiTemplate(template) = &before.content else {
            unreachable!("non-UI template documents were rejected before recipe expansion");
        };

        let parent_id = self
            .selected_ui_node_id
            .as_deref()
            .filter(|node_id| ui_node_accepts_children(&template.root, node_id))
            .unwrap_or_else(|| template.root.id().as_str())
            .to_string();
        let Some(parent_path) = ui_node_path(&template.root, &parent_id) else {
            return self.reject_recipe_insert(
                operation_id,
                document_id,
                target_profile_label,
                before,
                vec![UiDefinitionDiagnostic::error(
                    "editor.self_authoring.recipe.parent_unresolved",
                    format!("recipe insertion parent '{parent_id}' is not present"),
                )],
            );
        };
        let Some(insert_index) = ui_node_child_count(&template.root, &parent_id) else {
            return self.reject_recipe_insert(
                operation_id,
                document_id,
                target_profile_label,
                before,
                vec![UiDefinitionDiagnostic::error(
                    "editor.self_authoring.recipe.parent_not_container",
                    format!("recipe insertion parent '{parent_id}' cannot contain children"),
                )],
            );
        };

        let operation = EditorLabOperation {
            id: operation_id.clone(),
            document_id,
            target_profile: target_profile_label.clone(),
            kind: EditorLabOperationKind::UiVisualLayout(Box::new(UiVisualLayoutOperation {
                id: AuthoredId::new(operation_id_from_recipe(&recipe_id, &inserted_node_id)),
                source_document: template.id.clone(),
                target_path: AuthoredUiNodePath(parent_path),
                expected_node_id: AuthoredId::new(parent_id),
                target_profile: AuthoredId::new(target_profile_label),
                kind: UiVisualLayoutEditKind::InsertNode {
                    index: insert_index,
                    node: inserted_root,
                },
                source_location: None,
                preview_only: false,
            })),
            preview_only: false,
            source: Some(format!("recipe:{}", recipe_id.as_str())),
        };
        let report = self.apply_editor_lab_operation(operation)?;
        if report.status == EditorLabOperationStatus::Rejected {
            return Err(report.diagnostics.first().cloned().unwrap_or_else(|| {
                UiDefinitionDiagnostic::error(
                    "editor.self_authoring.recipe.rejected",
                    "recipe insertion operation was rejected",
                )
            }));
        }
        self.selected_ui_node_id = Some(inserted_node_id);
        Ok(report)
    }

    fn reject_recipe_insert(
        &mut self,
        operation_id: String,
        document_id: EditorDefinitionId,
        _target_profile: String,
        document: EditorDefinitionDocument,
        diagnostics: Vec<UiDefinitionDiagnostic>,
    ) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
        let first_diagnostic = diagnostics.first().cloned().unwrap_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.recipe.rejected",
                "recipe insertion was rejected",
            )
        });
        let report = EditorLabOperationReport {
            operation_id,
            document_id,
            status: EditorLabOperationStatus::Rejected,
            document,
            diff: None,
            diagnostics,
        };
        self.last_operation_report = Some(report);
        Err(first_diagnostic)
    }
}

fn ui_node_accepts_children(node: &UiNodeDefinition, node_id: &str) -> bool {
    if node.id().as_str() == node_id {
        return matches!(
            node,
            UiNodeDefinition::Panel { .. }
                | UiNodeDefinition::Row { .. }
                | UiNodeDefinition::Column { .. }
                | UiNodeDefinition::Stack { .. }
                | UiNodeDefinition::Scroll { .. }
                | UiNodeDefinition::Split { .. }
        );
    }
    node.children()
        .iter()
        .any(|child| ui_node_accepts_children(child, node_id))
}

fn ui_node_path(node: &UiNodeDefinition, node_id: &str) -> Option<String> {
    ui_node_path_segments(node, node_id, Vec::new()).map(|segments| segments.join("/"))
}

fn ui_node_path_segments(
    node: &UiNodeDefinition,
    node_id: &str,
    mut ancestors: Vec<String>,
) -> Option<Vec<String>> {
    ancestors.push(node.id().as_str().to_string());
    if node.id().as_str() == node_id {
        return Some(ancestors);
    }
    for child in node.children() {
        if let Some(path) = ui_node_path_segments(child, node_id, ancestors.clone()) {
            return Some(path);
        }
    }
    None
}

fn ui_node_child_count(node: &UiNodeDefinition, node_id: &str) -> Option<usize> {
    if node.id().as_str() == node_id {
        return Some(node.children().len());
    }
    node.children()
        .iter()
        .find_map(|child| ui_node_child_count(child, node_id))
}
