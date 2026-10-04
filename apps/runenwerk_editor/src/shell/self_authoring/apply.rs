//! Apply preview, review, accept/reject, applied snapshot, and rollback ownership.

use super::*;

#[derive(Debug, Clone)]
pub struct DefinitionApplyPreview {
    pub document_id: EditorDefinitionId,
    pub display_name: String,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
    pub summary: Vec<String>,
}

impl SelfAuthoringWorkspaceState {
    pub fn record_applied_workbench_composition_payload(
        &mut self,
        payload: &EditorDefinitionActivationPayload,
    ) {
        let EditorDefinitionActivationPayload::WorkbenchCompositionPackage {
            composition,
            profiles,
            layouts,
        } = payload
        else {
            return;
        };

        let documents = workbench_composition_package_documents(composition, profiles, layouts);

        for mut document in documents {
            document.lifecycle_state = EditorDefinitionLifecycleState::Applied;
            self.rollback_snapshots
                .insert(document.id.clone(), self.applied.get(&document.id).cloned());
            self.last_applied_snapshots
                .insert(document.id.clone(), document.clone());
            self.applied.insert(document.id.clone(), document);
        }
        self.record_source_change();
    }

    pub fn build_apply_preview(&self) -> Option<DefinitionApplyPreview> {
        let document = self.selected_document()?;
        if matches!(
            &document.content,
            EditorDefinitionDocumentContent::WorkbenchComposition(_)
        ) {
            return Some(self.build_workbench_composition_package_apply_preview(document));
        }
        let diagnostics = validate_editor_definition_document(document);
        let mut summary = vec![
            format!("document: {}", document.display_name),
            format!("kind: {:?}", document.kind),
        ];
        match &document.content {
            EditorDefinitionDocumentContent::UiTemplate(template) => {
                summary.push(format!("template: {}", template.id));
                summary.push(format!("child_templates: {}", template.templates.len()));
                summary.push(format!("menus: {}", template.menus.len()));
            }
            EditorDefinitionDocumentContent::EditorBindings(bindings) => {
                summary.push(format!("toolbar_template: {}", bindings.toolbar.template));
                summary.push(format!(
                    "surface_templates: {}",
                    bindings.surface_templates.len()
                ));
            }
            _ => summary.push("editor definition schema document".to_string()),
        }
        Some(DefinitionApplyPreview {
            document_id: document.id.clone(),
            display_name: document.display_name.clone(),
            diagnostics,
            summary,
        })
    }

    fn build_workbench_composition_package_apply_preview(
        &self,
        document: &EditorDefinitionDocument,
    ) -> DefinitionApplyPreview {
        match self.selected_workbench_composition_package_documents() {
            Ok(package_documents) => {
                let diagnostics = package_documents
                    .iter()
                    .flat_map(validate_editor_definition_document)
                    .collect::<Vec<_>>();
                let mut summary = vec![
                    format!("workbench composition: {}", document.display_name),
                    format!("package_documents: {}", package_documents.len()),
                ];
                summary.extend(
                    package_documents.iter().map(|package_document| {
                        format!("document: {}", package_document.id.as_str())
                    }),
                );
                DefinitionApplyPreview {
                    document_id: document.id.clone(),
                    display_name: document.display_name.clone(),
                    diagnostics,
                    summary,
                }
            }
            Err(diagnostic) => DefinitionApplyPreview {
                document_id: document.id.clone(),
                display_name: document.display_name.clone(),
                diagnostics: vec![diagnostic],
                summary: vec![
                    format!("workbench composition: {}", document.display_name),
                    "package assembly blocked".to_string(),
                ],
            },
        }
    }

    pub fn build_definition_apply_review(&self) -> Option<DefinitionApplyReview> {
        let document = self.selected_document()?;
        if matches!(
            &document.content,
            EditorDefinitionDocumentContent::WorkbenchComposition(_)
        ) {
            return Some(self.build_workbench_composition_package_apply_review_model(document));
        }
        let diagnostics = validate_editor_definition_document(document);
        let status = if editor_definition_has_blocking_diagnostics(&diagnostics) {
            DefinitionApplyReviewStatus::Blocked
        } else {
            DefinitionApplyReviewStatus::Pending
        };
        let mut proposed = document.clone();
        proposed.lifecycle_state = EditorDefinitionLifecycleState::Applied;
        let applied_before = self.applied.get(&document.id).cloned();
        Some(DefinitionApplyReview {
            id: format!("editor-lab.apply-review.{}", document.id.as_str()),
            document_id: document.id.clone(),
            display_name: document.display_name.clone(),
            status,
            draft_snapshot: document.clone(),
            applied_before: applied_before.clone(),
            proposed_applied_snapshot: proposed.clone(),
            diff_rows: definition_apply_diff_rows(applied_before.as_ref(), &proposed),
            diagnostics,
            rollback_target_available: self.rollback_snapshots.contains_key(&document.id)
                || applied_before.is_some(),
        })
    }

    fn build_workbench_composition_package_apply_review_model(
        &self,
        document: &EditorDefinitionDocument,
    ) -> DefinitionApplyReview {
        match self.selected_workbench_composition_package_documents() {
            Ok(package_documents) => {
                let mut proposed_documents = package_documents;
                for proposed in &mut proposed_documents {
                    proposed.lifecycle_state = EditorDefinitionLifecycleState::Applied;
                }
                let diagnostics = proposed_documents
                    .iter()
                    .flat_map(validate_editor_definition_document)
                    .collect::<Vec<_>>();
                let status = if editor_definition_has_blocking_diagnostics(&diagnostics) {
                    DefinitionApplyReviewStatus::Blocked
                } else {
                    DefinitionApplyReviewStatus::Pending
                };
                let mut diff_rows = Vec::new();
                for proposed in &proposed_documents {
                    for mut row in
                        definition_apply_diff_rows(self.applied.get(&proposed.id), proposed)
                    {
                        row.path = format!("{}:{}", proposed.id.as_str(), row.path);
                        diff_rows.push(row);
                    }
                }
                let proposed_primary = proposed_documents
                    .iter()
                    .find(|candidate| candidate.id == document.id)
                    .cloned()
                    .unwrap_or_else(|| {
                        let mut proposed = document.clone();
                        proposed.lifecycle_state = EditorDefinitionLifecycleState::Applied;
                        proposed
                    });
                DefinitionApplyReview {
                    id: format!("editor-lab.apply-review.{}", document.id.as_str()),
                    document_id: document.id.clone(),
                    display_name: document.display_name.clone(),
                    status,
                    draft_snapshot: document.clone(),
                    applied_before: self.applied.get(&document.id).cloned(),
                    proposed_applied_snapshot: proposed_primary,
                    diff_rows,
                    diagnostics,
                    rollback_target_available: proposed_documents.iter().any(|proposed| {
                        self.rollback_snapshots.contains_key(&proposed.id)
                            || self.applied.contains_key(&proposed.id)
                    }),
                }
            }
            Err(diagnostic) => {
                let mut proposed = document.clone();
                proposed.lifecycle_state = EditorDefinitionLifecycleState::Applied;
                DefinitionApplyReview {
                    id: format!("editor-lab.apply-review.{}", document.id.as_str()),
                    document_id: document.id.clone(),
                    display_name: document.display_name.clone(),
                    status: DefinitionApplyReviewStatus::Blocked,
                    draft_snapshot: document.clone(),
                    applied_before: self.applied.get(&document.id).cloned(),
                    proposed_applied_snapshot: proposed,
                    diff_rows: Vec::new(),
                    diagnostics: vec![diagnostic],
                    rollback_target_available: self.rollback_snapshots.contains_key(&document.id)
                        || self.applied.contains_key(&document.id),
                }
            }
        }
    }

    pub fn prepare_selected_apply_review(
        &mut self,
    ) -> Result<DefinitionApplyReview, UiDefinitionDiagnostic> {
        let review = self.build_definition_apply_review().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.apply_review.no_selection",
                "no definition document is selected",
            )
        })?;
        self.last_apply_review = Some(review.clone());
        Ok(review)
    }

    pub fn last_apply_preview(&self) -> Option<&DefinitionApplyPreview> {
        self.last_apply_preview.as_ref()
    }

    pub fn last_apply_review(&self) -> Option<&DefinitionApplyReview> {
        self.last_apply_review.as_ref()
    }

    pub fn reject_last_apply_review(
        &mut self,
    ) -> Result<DefinitionApplyReview, UiDefinitionDiagnostic> {
        let review = self
            .last_apply_review
            .clone()
            .or_else(|| self.build_definition_apply_review())
            .ok_or_else(|| {
                UiDefinitionDiagnostic::error(
                    "editor.self_authoring.apply_review.reject.no_review",
                    "no definition apply review is available to reject",
                )
            })?
            .with_status(DefinitionApplyReviewStatus::Rejected);
        self.last_apply_review = Some(review.clone());
        Ok(review)
    }

    pub fn apply_selected(&mut self) -> Result<DefinitionApplyPreview, UiDefinitionDiagnostic> {
        if matches!(
            self.selected_document().map(|document| &document.content),
            Some(EditorDefinitionDocumentContent::WorkbenchComposition(_))
        ) {
            return self.apply_selected_workbench_composition_package();
        }
        let review = self.prepare_selected_apply_review()?;
        let preview = self.build_apply_preview().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.apply.no_selection",
                "no definition document is selected",
            )
        })?;
        if review.status == DefinitionApplyReviewStatus::Blocked
            || review.has_blocking_diagnostics()
        {
            self.last_apply_preview = Some(preview.clone());
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.apply.blocked",
                "definition has blocking validation diagnostics",
            ));
        }
        let applied = review.proposed_applied_snapshot.clone();
        self.rollback_snapshots.insert(
            preview.document_id.clone(),
            self.applied.get(&preview.document_id).cloned(),
        );
        self.last_applied_snapshots
            .insert(preview.document_id.clone(), applied.clone());
        self.applied.insert(preview.document_id.clone(), applied);
        self.last_apply_preview = Some(preview.clone());
        self.last_apply_review = Some(review.with_status(DefinitionApplyReviewStatus::Accepted));
        self.record_source_change();
        Ok(preview)
    }

    fn apply_selected_workbench_composition_package(
        &mut self,
    ) -> Result<DefinitionApplyPreview, UiDefinitionDiagnostic> {
        let review = self.prepare_selected_apply_review()?;
        let preview = self.build_apply_preview().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.apply.no_selection",
                "no definition document is selected",
            )
        })?;
        if review.status == DefinitionApplyReviewStatus::Blocked
            || review.has_blocking_diagnostics()
        {
            self.last_apply_preview = Some(preview.clone());
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.apply.blocked",
                "workbench composition package has blocking validation diagnostics",
            ));
        }

        let mut applied_documents = self.selected_workbench_composition_package_documents()?;
        for document in &mut applied_documents {
            document.lifecycle_state = EditorDefinitionLifecycleState::Applied;
        }
        let diagnostics = applied_documents
            .iter()
            .flat_map(validate_editor_definition_document)
            .collect::<Vec<_>>();
        if editor_definition_has_blocking_diagnostics(&diagnostics) {
            self.last_apply_preview = Some(DefinitionApplyPreview {
                diagnostics,
                ..preview.clone()
            });
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.apply.blocked",
                "workbench composition package has blocking validation diagnostics",
            ));
        }

        let rollback_snapshots = applied_documents
            .iter()
            .map(|document| (document.id.clone(), self.applied.get(&document.id).cloned()))
            .collect::<Vec<_>>();
        for (document_id, snapshot) in rollback_snapshots {
            self.rollback_snapshots.insert(document_id, snapshot);
        }
        for document in applied_documents {
            self.last_applied_snapshots
                .insert(document.id.clone(), document.clone());
            self.applied.insert(document.id.clone(), document);
        }
        self.last_apply_preview = Some(preview.clone());
        self.last_apply_review = Some(review.with_status(DefinitionApplyReviewStatus::Accepted));
        self.record_source_change();
        Ok(preview)
    }

    pub fn rollback_selected(
        &mut self,
    ) -> Result<EditorDefinitionDocument, UiDefinitionDiagnostic> {
        if matches!(
            self.selected_document().map(|document| &document.content),
            Some(EditorDefinitionDocumentContent::WorkbenchComposition(_))
        ) {
            return self.rollback_selected_workbench_composition_package();
        }
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.rollback.no_selection",
                "no definition document is selected",
            )
        })?;
        let removed_document = self.applied.remove(&document_id).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.rollback.no_applied_snapshot",
                "selected definition has no applied snapshot",
            )
        })?;
        let Some(rollback_snapshot) = self.rollback_snapshots.remove(&document_id) else {
            self.applied
                .insert(document_id.clone(), removed_document.clone());
            let diagnostic = UiDefinitionDiagnostic::error(
                "editor.self_authoring.rollback.no_recorded_snapshot",
                "selected definition has no recorded rollback snapshot",
            );
            self.rollback_records.push(EditorLabRollbackRecord {
                id: format!("editor-lab.rollback.unavailable.{}", document_id.as_str()),
                document_id: document_id.clone(),
                display_name: removed_document.display_name.clone(),
                status: EditorLabRollbackStatus::Unavailable,
                removed_document: None,
                restored_document: None,
                diagnostics: vec![diagnostic.clone()],
            });
            return Err(diagnostic);
        };
        if let Some(mut previous) = rollback_snapshot {
            previous.lifecycle_state = EditorDefinitionLifecycleState::Applied;
            self.applied.insert(document_id.clone(), previous.clone());
            self.rollback_records.push(EditorLabRollbackRecord {
                id: format!("editor-lab.rollback.{}", document_id.as_str()),
                document_id: document_id.clone(),
                display_name: previous.display_name.clone(),
                status: EditorLabRollbackStatus::RolledBack,
                removed_document: Some(removed_document.clone()),
                restored_document: Some(previous),
                diagnostics: Vec::new(),
            });
        } else {
            self.rollback_records.push(EditorLabRollbackRecord {
                id: format!("editor-lab.rollback.{}", document_id.as_str()),
                document_id: document_id.clone(),
                display_name: removed_document.display_name.clone(),
                status: EditorLabRollbackStatus::RolledBack,
                removed_document: Some(removed_document.clone()),
                restored_document: None,
                diagnostics: Vec::new(),
            });
        }
        let mut rolled_back = removed_document;
        rolled_back.lifecycle_state = EditorDefinitionLifecycleState::RolledBack;
        self.record_source_change();
        Ok(rolled_back)
    }

    fn rollback_selected_workbench_composition_package(
        &mut self,
    ) -> Result<EditorDefinitionDocument, UiDefinitionDiagnostic> {
        let selected_document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.rollback.no_selection",
                "no definition document is selected",
            )
        })?;
        let package_documents = self.selected_workbench_composition_package_documents()?;
        let package_ids = package_documents
            .iter()
            .map(|document| document.id.clone())
            .collect::<Vec<_>>();

        for document_id in &package_ids {
            if !self.applied.contains_key(document_id) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.self_authoring.rollback.no_applied_snapshot",
                    format!(
                        "workbench composition package document `{}` has no applied snapshot",
                        document_id.as_str()
                    ),
                ));
            }
            if !self.rollback_snapshots.contains_key(document_id) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.self_authoring.rollback.no_recorded_snapshot",
                    format!(
                        "workbench composition package document `{}` has no recorded rollback snapshot",
                        document_id.as_str()
                    ),
                ));
            }
        }

        let mut selected_removed_document = None;
        for document_id in package_ids {
            let removed_document = self
                .applied
                .remove(&document_id)
                .expect("package rollback preflight checked applied snapshot");
            let rollback_snapshot = self
                .rollback_snapshots
                .remove(&document_id)
                .expect("package rollback preflight checked rollback snapshot");
            if document_id == selected_document_id {
                selected_removed_document = Some(removed_document.clone());
            }
            if let Some(mut previous) = rollback_snapshot {
                previous.lifecycle_state = EditorDefinitionLifecycleState::Applied;
                self.applied.insert(document_id.clone(), previous.clone());
                self.rollback_records.push(EditorLabRollbackRecord {
                    id: format!("editor-lab.rollback.{}", document_id.as_str()),
                    document_id,
                    display_name: previous.display_name.clone(),
                    status: EditorLabRollbackStatus::RolledBack,
                    removed_document: Some(removed_document),
                    restored_document: Some(previous),
                    diagnostics: Vec::new(),
                });
            } else {
                self.rollback_records.push(EditorLabRollbackRecord {
                    id: format!("editor-lab.rollback.{}", document_id.as_str()),
                    document_id,
                    display_name: removed_document.display_name.clone(),
                    status: EditorLabRollbackStatus::RolledBack,
                    removed_document: Some(removed_document),
                    restored_document: None,
                    diagnostics: Vec::new(),
                });
            }
        }

        let mut rolled_back = selected_removed_document.ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.rollback.package_primary_missing",
                "selected workbench composition was not part of its rollback package",
            )
        })?;
        rolled_back.lifecycle_state = EditorDefinitionLifecycleState::RolledBack;
        self.record_source_change();
        Ok(rolled_back)
    }

    pub fn reload_selected_from_last_applied(
        &mut self,
    ) -> Result<EditorDefinitionDocument, UiDefinitionDiagnostic> {
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.reload_last_applied.no_selection",
                "no definition document is selected",
            )
        })?;
        let snapshot = self
            .last_applied_snapshots
            .get(&document_id)
            .cloned()
            .ok_or_else(|| {
                UiDefinitionDiagnostic::error(
                    "editor.self_authoring.reload_last_applied.no_snapshot",
                    "selected definition has no last applied snapshot",
                )
            })?;
        self.drafts.insert(document_id.clone(), snapshot.clone());
        self.applied.insert(document_id, snapshot.clone());
        self.record_source_change();
        Ok(snapshot)
    }

    pub fn last_rollback_record(&self) -> Option<&EditorLabRollbackRecord> {
        self.rollback_records.last()
    }

    pub fn rollback_records(&self) -> &[EditorLabRollbackRecord] {
        &self.rollback_records
    }

    pub fn last_applied_document(
        &self,
        id: &EditorDefinitionId,
    ) -> Option<&EditorDefinitionDocument> {
        self.last_applied_snapshots.get(id)
    }

    pub fn selected_last_applied_document(&self) -> Option<&EditorDefinitionDocument> {
        self.selected_document_id
            .as_ref()
            .and_then(|id| self.last_applied_document(id))
    }

    pub fn applied_document(&self, id: &EditorDefinitionId) -> Option<&EditorDefinitionDocument> {
        self.applied.get(id)
    }

    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }
}

fn definition_apply_diff_rows(
    applied_before: Option<&EditorDefinitionDocument>,
    proposed: &EditorDefinitionDocument,
) -> Vec<DefinitionApplyDiffRow> {
    let mut rows = Vec::new();
    match applied_before {
        Some(before) => {
            if before.display_name != proposed.display_name {
                rows.push(DefinitionApplyDiffRow::updated(
                    DefinitionApplyDiffFamily::DocumentMetadata,
                    "document.display_name",
                    before.display_name.clone(),
                    proposed.display_name.clone(),
                    "display name changed",
                ));
            }
            if before.kind != proposed.kind {
                rows.push(DefinitionApplyDiffRow::updated(
                    DefinitionApplyDiffFamily::DocumentMetadata,
                    "document.kind",
                    format!("{:?}", before.kind),
                    format!("{:?}", proposed.kind),
                    "document kind changed",
                ));
            }
            if before.lifecycle_state != proposed.lifecycle_state {
                rows.push(DefinitionApplyDiffRow::state_changed(
                    DefinitionApplyDiffFamily::DocumentMetadata,
                    "document.lifecycle_state",
                    format!("{:?}", before.lifecycle_state),
                    format!("{:?}", proposed.lifecycle_state),
                    "document lifecycle state changed",
                ));
            }
            if before.content != proposed.content {
                definition_content_diff_rows(&before.content, &proposed.content, &mut rows);
            }
        }
        None => rows.push(DefinitionApplyDiffRow::added(
            DefinitionApplyDiffFamily::Document,
            "document",
            proposed.display_name.clone(),
            "definition will be added to applied state",
        )),
    }
    rows
}

fn definition_content_diff_rows(
    before: &EditorDefinitionDocumentContent,
    proposed: &EditorDefinitionDocumentContent,
    rows: &mut Vec<DefinitionApplyDiffRow>,
) {
    match (before, proposed) {
        (
            EditorDefinitionDocumentContent::UiTemplate(before),
            EditorDefinitionDocumentContent::UiTemplate(proposed),
        ) => ui_template_diff_rows(before, proposed, rows),
        (
            EditorDefinitionDocumentContent::WorkspaceProfile(before),
            EditorDefinitionDocumentContent::WorkspaceProfile(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::WorkspaceProfile,
            "document.content.workspace_profile",
            before,
            proposed,
            "workspace profile changed",
        ),
        (
            EditorDefinitionDocumentContent::WorkspaceLayout(before),
            EditorDefinitionDocumentContent::WorkspaceLayout(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::WorkspaceLayout,
            "document.content.workspace_layout",
            before,
            proposed,
            "workspace layout changed",
        ),
        (
            EditorDefinitionDocumentContent::WorkbenchComposition(before),
            EditorDefinitionDocumentContent::WorkbenchComposition(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::WorkbenchComposition,
            "document.content.workbench_composition",
            before,
            proposed,
            "workbench composition changed",
        ),
        (
            EditorDefinitionDocumentContent::Menu(before),
            EditorDefinitionDocumentContent::Menu(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::Menu,
            "document.content.menu",
            before,
            proposed,
            "menu definition changed",
        ),
        (
            EditorDefinitionDocumentContent::Theme(before),
            EditorDefinitionDocumentContent::Theme(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::Theme,
            "document.content.theme",
            before,
            proposed,
            "theme definition changed",
        ),
        (
            EditorDefinitionDocumentContent::Shortcuts(before),
            EditorDefinitionDocumentContent::Shortcuts(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::ShortcutSet,
            "document.content.shortcuts",
            before,
            proposed,
            "shortcut set changed",
        ),
        (
            EditorDefinitionDocumentContent::CommandBindings(before),
            EditorDefinitionDocumentContent::CommandBindings(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::CommandBindingSet,
            "document.content.command_bindings",
            before,
            proposed,
            "command binding set changed",
        ),
        (
            EditorDefinitionDocumentContent::PanelRegistry(before),
            EditorDefinitionDocumentContent::PanelRegistry(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::PanelRegistry,
            "document.content.panel_registry",
            before,
            proposed,
            "panel registry changed",
        ),
        (
            EditorDefinitionDocumentContent::ToolSurfaceRegistry(before),
            EditorDefinitionDocumentContent::ToolSurfaceRegistry(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::ToolSurfaceRegistry,
            "document.content.tool_surface_registry",
            before,
            proposed,
            "tool surface registry changed",
        ),
        (
            EditorDefinitionDocumentContent::EditorBindings(before),
            EditorDefinitionDocumentContent::EditorBindings(proposed),
        ) => push_structural_debug_row(
            rows,
            DefinitionApplyDiffFamily::EditorBindings,
            "document.content.editor_bindings",
            before,
            proposed,
            "editor bindings changed",
        ),
        _ => rows.push(DefinitionApplyDiffRow::updated(
            DefinitionApplyDiffFamily::Document,
            "document.content.kind",
            editor_definition_content_label(before),
            editor_definition_content_label(proposed),
            "document content kind changed",
        )),
    }
}

fn ui_template_diff_rows(
    before: &AuthoredUiTemplate,
    proposed: &AuthoredUiTemplate,
    rows: &mut Vec<DefinitionApplyDiffRow>,
) {
    if before.id != proposed.id {
        rows.push(DefinitionApplyDiffRow::updated(
            DefinitionApplyDiffFamily::UiTemplate,
            "document.content.ui_template.id",
            before.id.to_string(),
            proposed.id.to_string(),
            "UI template id changed",
        ));
    }
    ui_node_diff_rows(
        "document.content.ui_template.root",
        &before.root,
        &proposed.root,
        rows,
    );
    if before.templates != proposed.templates {
        rows.push(DefinitionApplyDiffRow::updated(
            DefinitionApplyDiffFamily::UiTemplate,
            "document.content.ui_template.templates",
            before.templates.len().to_string(),
            proposed.templates.len().to_string(),
            "child template collection changed",
        ));
    }
    if before.menus != proposed.menus {
        rows.push(DefinitionApplyDiffRow::updated(
            DefinitionApplyDiffFamily::UiTemplate,
            "document.content.ui_template.menus",
            before.menus.len().to_string(),
            proposed.menus.len().to_string(),
            "template menu collection changed",
        ));
    }
}

fn ui_node_diff_rows(
    path: &str,
    before: &UiNodeDefinition,
    proposed: &UiNodeDefinition,
    rows: &mut Vec<DefinitionApplyDiffRow>,
) {
    let node_path = format!("{path}.{}", proposed.id().as_str());
    if before.id() != proposed.id() {
        rows.push(DefinitionApplyDiffRow::updated(
            DefinitionApplyDiffFamily::UiTemplate,
            format!("{node_path}.id"),
            before.id().to_string(),
            proposed.id().to_string(),
            "UI node id changed",
        ));
    }
    let before_kind = ui_node_kind(before);
    let proposed_kind = ui_node_kind(proposed);
    if before_kind != proposed_kind {
        rows.push(DefinitionApplyDiffRow::updated(
            DefinitionApplyDiffFamily::UiTemplate,
            format!("{node_path}.kind"),
            before_kind,
            proposed_kind,
            "UI node kind changed",
        ));
        return;
    }

    ui_node_field_diff_rows(&node_path, before, proposed, rows);

    let before_children = before.children();
    let proposed_children = proposed.children();
    if before_children.len() != proposed_children.len() {
        rows.push(DefinitionApplyDiffRow::updated(
            DefinitionApplyDiffFamily::UiTemplate,
            format!("{node_path}.children"),
            before_children.len().to_string(),
            proposed_children.len().to_string(),
            "UI node child count changed",
        ));
        return;
    }
    for (before_child, proposed_child) in before_children.iter().zip(proposed_children) {
        ui_node_diff_rows(&node_path, before_child, proposed_child, rows);
    }
}

fn ui_node_field_diff_rows(
    node_path: &str,
    before: &UiNodeDefinition,
    proposed: &UiNodeDefinition,
    rows: &mut Vec<DefinitionApplyDiffRow>,
) {
    match (before, proposed) {
        (
            UiNodeDefinition::Stack { axis: before, .. },
            UiNodeDefinition::Stack { axis: proposed, .. },
        ) => push_value_row(
            rows,
            format!("{node_path}.axis"),
            format!("{before:?}"),
            format!("{proposed:?}"),
            "stack axis changed",
        ),
        (
            UiNodeDefinition::Split {
                axis: before_axis,
                ratio: before_ratio,
                ..
            },
            UiNodeDefinition::Split {
                axis: proposed_axis,
                ratio: proposed_ratio,
                ..
            },
        ) => {
            push_value_row(
                rows,
                format!("{node_path}.axis"),
                format!("{before_axis:?}"),
                format!("{proposed_axis:?}"),
                "split axis changed",
            );
            push_value_row(
                rows,
                format!("{node_path}.ratio"),
                before_ratio.to_string(),
                proposed_ratio.to_string(),
                "split ratio changed",
            );
        }
        (
            UiNodeDefinition::Label { label: before, .. },
            UiNodeDefinition::Label {
                label: proposed, ..
            },
        )
        | (
            UiNodeDefinition::Button { label: before, .. },
            UiNodeDefinition::Button {
                label: proposed, ..
            },
        ) => push_value_row(
            rows,
            format!("{node_path}.label"),
            ui_value_binding_text(before),
            ui_value_binding_text(proposed),
            "UI node label changed",
        ),
        (
            UiNodeDefinition::Toggle {
                label: before_label,
                checked: before_checked,
                ..
            },
            UiNodeDefinition::Toggle {
                label: proposed_label,
                checked: proposed_checked,
                ..
            },
        ) => {
            push_value_row(
                rows,
                format!("{node_path}.label"),
                ui_value_binding_text(before_label),
                ui_value_binding_text(proposed_label),
                "toggle label changed",
            );
            push_value_row(
                rows,
                format!("{node_path}.checked"),
                ui_value_binding_text(before_checked),
                ui_value_binding_text(proposed_checked),
                "toggle checked binding changed",
            );
        }
        (
            UiNodeDefinition::TextInput {
                value: before_value,
                placeholder: before_placeholder,
                ..
            },
            UiNodeDefinition::TextInput {
                value: proposed_value,
                placeholder: proposed_placeholder,
                ..
            },
        ) => {
            push_value_row(
                rows,
                format!("{node_path}.value"),
                ui_value_binding_text(before_value),
                ui_value_binding_text(proposed_value),
                "text input value changed",
            );
            push_value_row(
                rows,
                format!("{node_path}.placeholder"),
                format!("{before_placeholder:?}"),
                format!("{proposed_placeholder:?}"),
                "text input placeholder changed",
            );
        }
        (
            UiNodeDefinition::NumericInput { value: before, .. },
            UiNodeDefinition::NumericInput {
                value: proposed, ..
            },
        ) => push_value_row(
            rows,
            format!("{node_path}.value"),
            ui_value_binding_text(before),
            ui_value_binding_text(proposed),
            "numeric input value changed",
        ),
        (
            UiNodeDefinition::Repeat {
                template: before_template,
                axis: before_axis,
                ..
            },
            UiNodeDefinition::Repeat {
                template: proposed_template,
                axis: proposed_axis,
                ..
            },
        ) => {
            push_value_row(
                rows,
                format!("{node_path}.template"),
                before_template.to_string(),
                proposed_template.to_string(),
                "repeat template changed",
            );
            push_value_row(
                rows,
                format!("{node_path}.axis"),
                format!("{before_axis:?}"),
                format!("{proposed_axis:?}"),
                "repeat axis changed",
            );
        }
        (
            UiNodeDefinition::TemplateRef {
                template: before, ..
            },
            UiNodeDefinition::TemplateRef {
                template: proposed, ..
            },
        ) => push_value_row(
            rows,
            format!("{node_path}.template"),
            before.to_string(),
            proposed.to_string(),
            "template reference changed",
        ),
        _ => {
            if before != proposed && before.children() == proposed.children() {
                rows.push(DefinitionApplyDiffRow::updated(
                    DefinitionApplyDiffFamily::UiTemplate,
                    node_path,
                    format!("{before:#?}"),
                    format!("{proposed:#?}"),
                    "UI node fields changed",
                ));
            }
        }
    }
}

fn push_value_row(
    rows: &mut Vec<DefinitionApplyDiffRow>,
    path: impl Into<String>,
    before: String,
    proposed: String,
    summary: impl Into<String>,
) {
    if before != proposed {
        rows.push(DefinitionApplyDiffRow::updated(
            DefinitionApplyDiffFamily::UiTemplate,
            path,
            before,
            proposed,
            summary,
        ));
    }
}

fn push_structural_debug_row<T: std::fmt::Debug + PartialEq>(
    rows: &mut Vec<DefinitionApplyDiffRow>,
    family: DefinitionApplyDiffFamily,
    path: impl Into<String>,
    before: &T,
    proposed: &T,
    summary: impl Into<String>,
) {
    if before != proposed {
        rows.push(DefinitionApplyDiffRow::updated(
            family,
            path,
            format!("{before:#?}"),
            format!("{proposed:#?}"),
            summary,
        ));
    }
}

fn ui_value_binding_text(binding: &UiValueBinding) -> String {
    match binding {
        UiValueBinding::Static(value) => value.as_text(),
        UiValueBinding::Slot(slot) => format!("slot:{slot}"),
    }
}

fn editor_definition_content_label(content: &EditorDefinitionDocumentContent) -> &'static str {
    match content {
        EditorDefinitionDocumentContent::UiTemplate(_) => "ui_template",
        EditorDefinitionDocumentContent::WorkspaceProfile(_) => "workspace_profile",
        EditorDefinitionDocumentContent::WorkspaceLayout(_) => "workspace_layout",
        EditorDefinitionDocumentContent::WorkbenchComposition(_) => "workbench_composition",
        EditorDefinitionDocumentContent::Menu(_) => "menu",
        EditorDefinitionDocumentContent::Theme(_) => "theme",
        EditorDefinitionDocumentContent::Shortcuts(_) => "shortcuts",
        EditorDefinitionDocumentContent::CommandBindings(_) => "command_bindings",
        EditorDefinitionDocumentContent::PanelRegistry(_) => "panel_registry",
        EditorDefinitionDocumentContent::ToolSurfaceRegistry(_) => "tool_surface_registry",
        EditorDefinitionDocumentContent::EditorBindings(_) => "editor_bindings",
    }
}

fn ui_node_kind(node: &UiNodeDefinition) -> &'static str {
    match node {
        UiNodeDefinition::Panel { .. } => "panel",
        UiNodeDefinition::Row { .. } => "row",
        UiNodeDefinition::Column { .. } => "column",
        UiNodeDefinition::Stack { .. } => "stack",
        UiNodeDefinition::Scroll { .. } => "scroll",
        UiNodeDefinition::Split { .. } => "split",
        UiNodeDefinition::Spacer { .. } => "spacer",
        UiNodeDefinition::Separator { .. } => "separator",
        UiNodeDefinition::Label { .. } => "label",
        UiNodeDefinition::Control { .. } => "control",
        UiNodeDefinition::Button { .. } => "button",
        UiNodeDefinition::Toggle { .. } => "toggle",
        UiNodeDefinition::TextInput { .. } => "text_input",
        UiNodeDefinition::NumericInput { .. } => "numeric_input",
        UiNodeDefinition::Select { .. } => "select",
        UiNodeDefinition::Tabs { .. } => "tabs",
        UiNodeDefinition::Table { .. } => "table",
        UiNodeDefinition::Tree { .. } => "tree",
        UiNodeDefinition::Repeat { .. } => "repeat",
        UiNodeDefinition::TemplateRef { .. } => "template_ref",
        UiNodeDefinition::MenuSlot { .. } => "menu_slot",
        UiNodeDefinition::EmbedSlot { .. } => "embed_slot",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_and_rollback_keep_explicit_snapshots() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let selected = state
            .selected_document_id()
            .expect("selected document should exist")
            .clone();

        let preview = state
            .apply_selected()
            .expect("selected fixture should apply");

        assert_eq!(preview.document_id, selected);
        assert!(state.applied_document(&selected).is_some());

        let rolled_back = state
            .rollback_selected()
            .expect("applied fixture should rollback");

        assert_eq!(rolled_back.id, selected);
        assert_eq!(
            rolled_back.lifecycle_state,
            EditorDefinitionLifecycleState::RolledBack
        );
        assert_eq!(state.applied_count(), 0);
    }

    #[test]
    fn apply_review_reject_reload_and_rollback_are_snapshot_backed() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let selected = state
            .selected_document_id()
            .expect("selected document should exist")
            .clone();
        let selected_node = state
            .selected_ui_node_id()
            .expect("selected UI fixture should expose an editable node")
            .to_string();

        let review = state
            .prepare_selected_apply_review()
            .expect("selected fixture should build an apply review");
        assert_eq!(review.status, DefinitionApplyReviewStatus::Pending);
        assert!(!review.diff_rows.is_empty());
        assert_eq!(state.applied_count(), 0);

        let rejected = state
            .reject_last_apply_review()
            .expect("apply review should reject without mutating applied state");
        assert_eq!(rejected.status, DefinitionApplyReviewStatus::Rejected);
        assert_eq!(state.applied_count(), 0);

        state
            .apply_selected()
            .expect("selected fixture should apply through a review");
        assert_eq!(
            state
                .last_apply_review()
                .expect("apply should record a review")
                .status,
            DefinitionApplyReviewStatus::Accepted
        );
        assert!(state.applied_document(&selected).is_some());

        state
            .set_selected_ui_node_text(&selected_node, "dirty draft after apply")
            .expect("draft edit should remain possible after apply");
        state
            .reload_selected_from_last_applied()
            .expect("last applied snapshot should reload into the draft");
        let preview = state
            .formed_selected_preview(&ThemeTokens::default())
            .expect("reloaded applied snapshot should preview");
        assert!(!format!("{:?}", preview.root).contains("dirty draft after apply"));

        let rolled_back = state
            .rollback_selected()
            .expect("recorded rollback snapshot should restore previous applied state");
        assert_eq!(rolled_back.id, selected);
        assert_eq!(state.applied_count(), 0);
        assert_eq!(
            state
                .last_rollback_record()
                .expect("rollback should record a typed record")
                .status,
            EditorLabRollbackStatus::RolledBack
        );
    }

    #[test]
    fn workbench_composition_apply_and_rollback_are_package_atomic() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let composition_id = state
            .create_custom_workbench_package()
            .expect("custom workbench package should be created");

        let review = state
            .prepare_selected_apply_review()
            .expect("workbench package should build an apply review");
        assert_eq!(review.document_id, composition_id);
        assert_eq!(review.status, DefinitionApplyReviewStatus::Pending);
        assert!(
            review
                .diff_rows
                .iter()
                .any(|row| row.path.contains("runenwerk.editor.workspace.custom1"))
        );
        assert!(
            review
                .diff_rows
                .iter()
                .any(|row| row.path.contains("runenwerk.editor.layout.custom1"))
        );

        state
            .apply_selected()
            .expect("workbench package apply should be atomic");
        let package = state.export_project_package();
        let applied_ids = package
            .applied_documents
            .iter()
            .map(|document| document.id.as_str())
            .collect::<Vec<_>>();
        assert!(applied_ids.contains(&"runenwerk.editor.workbench.custom1"));
        assert!(applied_ids.contains(&"runenwerk.editor.workspace.custom1"));
        assert!(applied_ids.contains(&"runenwerk.editor.layout.custom1"));
        assert!(
            state
                .last_applied_document(&EditorDefinitionId::from("runenwerk.editor.layout.custom1"))
                .is_some()
        );

        state
            .rollback_selected()
            .expect("workbench package rollback should be atomic");
        let package = state.export_project_package();
        let applied_ids = package
            .applied_documents
            .iter()
            .map(|document| document.id.as_str())
            .collect::<Vec<_>>();
        assert!(!applied_ids.contains(&"runenwerk.editor.workbench.custom1"));
        assert!(!applied_ids.contains(&"runenwerk.editor.workspace.custom1"));
        assert!(!applied_ids.contains(&"runenwerk.editor.layout.custom1"));
    }

    #[test]
    fn workbench_composition_apply_missing_layout_preserves_applied_state() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let composition_id = state
            .create_custom_workbench_package()
            .expect("custom workbench package should be created");
        assert!(state.select_document_by_str("runenwerk.editor.layout.custom1"));
        state
            .delete_selected()
            .expect("draft layout should be removable before apply");
        assert!(state.select_document(composition_id));

        let before_package = state.export_project_package();
        let diagnostic = state
            .apply_selected()
            .expect_err("missing package layout should reject apply");
        let after_package = state.export_project_package();

        assert_eq!(diagnostic.code, "editor.self_authoring.apply.blocked");
        assert_eq!(
            before_package.applied_documents,
            after_package.applied_documents
        );
        assert_eq!(
            before_package.last_applied_documents,
            after_package.last_applied_documents
        );
        assert!(
            state
                .last_apply_review()
                .expect("blocked apply should leave review")
                .has_blocking_diagnostics()
        );
    }
}
