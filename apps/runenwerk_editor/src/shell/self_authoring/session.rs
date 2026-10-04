//! Session-local operation history for UI Designer self-authoring.

use super::*;

#[derive(Debug, Clone)]
pub struct EditorLabOperationHistoryEntry {
    pub id: String,
    pub label: String,
    pub document_id: EditorDefinitionId,
    pub before: EditorDefinitionDocument,
    pub after: EditorDefinitionDocument,
    pub report: EditorLabOperationReport,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EditorLabOperationHistorySnapshot {
    pub undo_count: usize,
    pub redo_count: usize,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Clone, Default)]
pub(super) struct EditorLabOperationHistory {
    pub(super) undo: Vec<EditorLabOperationHistoryEntry>,
    pub(super) redo: Vec<EditorLabOperationHistoryEntry>,
    pub(super) next_sequence: u64,
}

impl SelfAuthoringWorkspaceState {
    pub fn next_operation_id(&self, family: &str) -> String {
        let family = family.replace([' ', ':', '/'], "_");
        format!(
            "editor-lab.{family}.{:04}",
            self.operation_history.next_sequence + 1
        )
    }

    pub fn last_operation_report(&self) -> Option<&EditorLabOperationReport> {
        self.last_operation_report.as_ref()
    }

    pub fn operation_history_snapshot(&self) -> EditorLabOperationHistorySnapshot {
        EditorLabOperationHistorySnapshot {
            undo_count: self.operation_history.undo.len(),
            redo_count: self.operation_history.redo.len(),
            can_undo: !self.operation_history.undo.is_empty(),
            can_redo: !self.operation_history.redo.is_empty(),
        }
    }

    pub fn apply_editor_lab_operation(
        &mut self,
        operation: EditorLabOperation,
    ) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
        let before = self
            .drafts
            .get(&operation.document_id)
            .cloned()
            .ok_or_else(|| {
                UiDefinitionDiagnostic::error(
                    "editor.self_authoring.operation.unresolved_document",
                    format!(
                        "definition document '{}' is not loaded",
                        operation.document_id.as_str()
                    ),
                )
            })?;
        let report = apply_editor_lab_operation(&before, &operation);
        self.last_operation_report = Some(report.clone());
        self.operation_history.next_sequence += 1;
        if report.status == EditorLabOperationStatus::Accepted {
            let after = report.document.clone();
            self.drafts
                .insert(operation.document_id.clone(), after.clone());
            self.selected_document_id = Some(operation.document_id.clone());
            self.selected_ui_node_id = selected_ui_node_after_operation(&after, &operation)
                .or_else(|| selected_ui_default_node_for_document(&after));
            self.operation_history
                .undo
                .push(EditorLabOperationHistoryEntry {
                    id: operation.id.clone(),
                    label: editor_lab_operation_label(&operation),
                    document_id: operation.document_id.clone(),
                    before,
                    after,
                    report: report.clone(),
                });
            self.operation_history.redo.clear();
            self.record_source_change();
        }
        Ok(report)
    }

    pub fn undo_editor_lab_operation(
        &mut self,
    ) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
        let entry = self.operation_history.undo.pop().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.operation.undo_unavailable",
                "no Editor Lab operation is available to undo",
            )
        })?;
        let report = operation_history_restore_report(
            format!("{}.undo", entry.id),
            entry.document_id.clone(),
            "Undo",
            &entry.after,
            entry.before.clone(),
        );
        self.restore_operation_document(entry.document_id.clone(), entry.before.clone());
        self.operation_history.redo.push(entry);
        self.last_operation_report = Some(report.clone());
        self.record_source_change();
        Ok(report)
    }

    pub fn redo_editor_lab_operation(
        &mut self,
    ) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
        let entry = self.operation_history.redo.pop().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.operation.redo_unavailable",
                "no Editor Lab operation is available to redo",
            )
        })?;
        let report = operation_history_restore_report(
            format!("{}.redo", entry.id),
            entry.document_id.clone(),
            "Redo",
            &entry.before,
            entry.after.clone(),
        );
        self.restore_operation_document(entry.document_id.clone(), entry.after.clone());
        self.operation_history.undo.push(entry);
        self.last_operation_report = Some(report.clone());
        self.record_source_change();
        Ok(report)
    }

    fn restore_operation_document(
        &mut self,
        document_id: EditorDefinitionId,
        document: EditorDefinitionDocument,
    ) {
        self.drafts.insert(document_id.clone(), document.clone());
        self.selected_document_id = Some(document_id);
        self.selected_ui_node_id = selected_ui_default_node_for_document(&document);
    }
}

pub(super) fn operation_history_document_snapshot(document: &EditorDefinitionDocument) -> String {
    ron::ser::to_string_pretty(document, PrettyConfig::default())
        .unwrap_or_else(|error| format!("snapshot serialization failed: {error}"))
}

fn selected_ui_node_after_operation(
    document: &EditorDefinitionDocument,
    operation: &EditorLabOperation,
) -> Option<String> {
    match &operation.kind {
        EditorLabOperationKind::SetUiNodeText { node_id, .. } => Some(node_id.clone()),
        EditorLabOperationKind::SetUiNodeValueSlot { node_id, .. } => Some(node_id.clone()),
        EditorLabOperationKind::SetUiNodeAvailabilityRef { node_id, .. } => Some(node_id.clone()),
        EditorLabOperationKind::UiVisualLayout(layout_operation) => match &layout_operation.kind {
            UiVisualLayoutEditKind::InsertNode { node, .. } => Some(node.id().as_str().to_string()),
            _ => Some(layout_operation.expected_node_id.as_str().to_string()),
        },
        EditorLabOperationKind::SetWorkbenchInstalledSuites { .. }
        | EditorLabOperationKind::SetWorkbenchProfileRefs { .. }
        | EditorLabOperationKind::SetWorkbenchDefaultProfileRef { .. } => None,
        _ => selected_ui_default_node_for_document(document),
    }
}

fn editor_lab_operation_label(operation: &EditorLabOperation) -> String {
    match &operation.kind {
        EditorLabOperationKind::UiVisualLayout(layout_operation) => match &layout_operation.kind {
            UiVisualLayoutEditKind::InsertNode { node, .. } => {
                format!("insert UI node {}", node.id())
            }
            _ => format!("visual layout {:?}", layout_operation.kind),
        },
        EditorLabOperationKind::SetUiNodeText { node_id, .. } => {
            format!("set UI node text {node_id}")
        }
        EditorLabOperationKind::SetUiNodeValueSlot { node_id, slot } => {
            format!("set UI node value slot {node_id}:{slot}")
        }
        EditorLabOperationKind::SetUiNodeAvailabilityRef {
            node_id,
            availability,
        } => format!("set UI node availability {node_id}:{availability}"),
        EditorLabOperationKind::RenameDocument { .. } => "rename definition".to_string(),
        EditorLabOperationKind::SetThemeColor { token, .. } => {
            format!("set theme color {token}")
        }
        EditorLabOperationKind::SetWorkbenchInstalledSuites { .. } => {
            "set workbench installed suites".to_string()
        }
        EditorLabOperationKind::SetWorkbenchProfileRefs { .. } => {
            "set workbench profile refs".to_string()
        }
        EditorLabOperationKind::SetWorkbenchDefaultProfileRef { .. } => {
            "set workbench default profile".to_string()
        }
        EditorLabOperationKind::AddWorkspaceLayoutTab { label, .. } => {
            format!("add workspace layout tab {label}")
        }
        EditorLabOperationKind::SplitWorkspaceLayoutRoot { axis } => {
            format!("split workspace layout root {axis:?}")
        }
        EditorLabOperationKind::CloseWorkspaceLayoutLastTab => {
            "close workspace layout tab".to_string()
        }
    }
}

fn operation_history_restore_report(
    operation_id: String,
    document_id: EditorDefinitionId,
    kind: &'static str,
    before: &EditorDefinitionDocument,
    document: EditorDefinitionDocument,
) -> EditorLabOperationReport {
    let diagnostics = validate_editor_definition_document(&document);
    let diff = Some(EditorLabOperationDiff {
        operation_id: operation_id.clone(),
        document_id: document_id.clone(),
        target_profile: "editor.workbench".to_string(),
        changes: vec![EditorLabOperationDiffChange {
            family: EditorLabOperationDiffFamily::EditorDocument,
            kind: kind.to_string(),
            path: "document".to_string(),
            before: Some(operation_history_document_snapshot(before)),
            after: Some(operation_history_document_snapshot(&document)),
        }],
    });
    EditorLabOperationReport {
        operation_id,
        document_id,
        status: EditorLabOperationStatus::Accepted,
        document,
        diff,
        diagnostics,
    }
}
