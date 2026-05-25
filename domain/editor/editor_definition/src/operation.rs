//! Editor Lab operation contracts and reducers for authored editor definitions.

use crate::{
    EditorDefinitionDocument, EditorDefinitionDocumentContent, EditorDefinitionId,
    EditorWorkspaceHostDefinition, EditorWorkspaceLayoutDefinition,
    EditorWorkspacePanelTabDefinition, EditorWorkspaceSplitAxisDefinition,
    editor_definition_has_blocking_diagnostics, validate_editor_definition_document,
};
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize};
use ui_definition::{
    AuthoredUiNodePath, UiDefinitionDiagnostic, UiDefinitionDiagnosticSeverity, UiNodeDefinition,
    UiValueBinding, UiVisualLayoutActivationMode, UiVisualLayoutDiffChange,
    UiVisualLayoutDiffChangeKind, UiVisualLayoutEditContext, UiVisualLayoutOperation,
    apply_visual_layout_operation,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorLabOperation {
    pub id: String,
    pub document_id: EditorDefinitionId,
    pub target_profile: String,
    pub kind: EditorLabOperationKind,
    #[serde(default)]
    pub preview_only: bool,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditorLabOperationKind {
    UiVisualLayout(Box<UiVisualLayoutOperation>),
    SetUiNodeText {
        node_id: String,
        text: String,
    },
    RenameDocument {
        display_name: String,
    },
    SetThemeColor {
        token: String,
        value: String,
    },
    AddWorkspaceLayoutTab {
        label: String,
        tool_surface: String,
    },
    SplitWorkspaceLayoutRoot {
        axis: EditorWorkspaceSplitAxisDefinition,
    },
    CloseWorkspaceLayoutLastTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLabOperationStatus {
    Accepted,
    Rejected,
    PreviewOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLabOperationDiffFamily {
    UiVisualLayout,
    UiAuthoredValue,
    EditorDocument,
    EditorTheme,
    EditorWorkspaceLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabOperationDiffChange {
    pub family: EditorLabOperationDiffFamily,
    pub kind: String,
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLabOperationDiff {
    pub operation_id: String,
    pub document_id: EditorDefinitionId,
    pub target_profile: String,
    pub changes: Vec<EditorLabOperationDiffChange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorLabOperationReport {
    pub operation_id: String,
    pub document_id: EditorDefinitionId,
    pub status: EditorLabOperationStatus,
    pub document: EditorDefinitionDocument,
    pub diff: Option<EditorLabOperationDiff>,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
}

impl EditorLabOperationReport {
    pub fn accepted(&self) -> bool {
        self.status == EditorLabOperationStatus::Accepted
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == UiDefinitionDiagnosticSeverity::Error)
    }
}

pub fn apply_editor_lab_operation(
    document: &EditorDefinitionDocument,
    operation: &EditorLabOperation,
) -> EditorLabOperationReport {
    let original_document = document.clone();
    if document.id != operation.document_id {
        return rejected(
            document.clone(),
            operation,
            UiDefinitionDiagnostic::error(
                "editor.lab.operation.document_mismatch",
                format!(
                    "operation targets '{}' but selected document is '{}'",
                    operation.document_id.as_str(),
                    document.id.as_str()
                ),
            ),
        );
    }

    let result = match &operation.kind {
        EditorLabOperationKind::UiVisualLayout(layout_operation) => {
            apply_ui_visual_layout_operation(document.clone(), operation, layout_operation)
        }
        EditorLabOperationKind::SetUiNodeText { node_id, text } => {
            apply_ui_node_text_operation(document.clone(), operation, node_id, text)
        }
        EditorLabOperationKind::RenameDocument { display_name } => {
            apply_rename_operation(document.clone(), operation, display_name)
        }
        EditorLabOperationKind::SetThemeColor { token, value } => {
            apply_theme_color_operation(document.clone(), operation, token, value)
        }
        EditorLabOperationKind::AddWorkspaceLayoutTab {
            label,
            tool_surface,
        } => apply_add_workspace_tab_operation(document.clone(), operation, label, tool_surface),
        EditorLabOperationKind::SplitWorkspaceLayoutRoot { axis } => {
            apply_split_workspace_root_operation(document.clone(), operation, *axis)
        }
        EditorLabOperationKind::CloseWorkspaceLayoutLastTab => {
            apply_close_workspace_last_tab_operation(document.clone(), operation)
        }
    };

    match result {
        Ok(mut report) => {
            if report.status == EditorLabOperationStatus::Rejected {
                report.document = original_document;
            }
            report
        }
        Err(diagnostic) => rejected(document.clone(), operation, diagnostic),
    }
}

fn apply_ui_visual_layout_operation(
    mut document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    layout_operation: &UiVisualLayoutOperation,
) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
    let EditorDefinitionDocumentContent::UiTemplate(template) = &document.content else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.lab.operation.ui_visual_layout.not_ui_template",
            "UI visual layout operations require a UI template document",
        ));
    };

    let context = UiVisualLayoutEditContext::with_supported_target_profiles([layout_operation
        .target_profile
        .clone()]);
    let report = apply_visual_layout_operation(
        template.clone(),
        layout_operation,
        UiVisualLayoutActivationMode::Activate,
        &context,
    );
    if report.has_errors() {
        let diagnostics = report
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.as_definition_diagnostic())
            .collect();
        return Ok(report_with_diagnostics(
            document,
            operation,
            EditorLabOperationStatus::Rejected,
            None,
            diagnostics,
        ));
    }

    document.content = EditorDefinitionDocumentContent::UiTemplate(report.template);
    let diff = report.diff.map(|diff| EditorLabOperationDiff {
        operation_id: operation.id.clone(),
        document_id: document.id.clone(),
        target_profile: operation.target_profile.clone(),
        changes: diff
            .changes
            .into_iter()
            .map(ui_visual_layout_diff_change)
            .collect(),
    });
    finalize_accepted(document, operation, diff)
}

fn apply_ui_node_text_operation(
    mut document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    node_id: &str,
    text: &str,
) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
    let EditorDefinitionDocumentContent::UiTemplate(template) = &mut document.content else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.lab.operation.ui_text.not_ui_template",
            "UI node text operations require a UI template document",
        ));
    };
    let path = ui_node_path(&template.root, node_id).ok_or_else(|| {
        UiDefinitionDiagnostic::error(
            "editor.lab.operation.ui_text.target_missing",
            format!("UI node '{node_id}' is not present in the selected definition"),
        )
    })?;
    let before = ui_node_authored_text(&template.root, node_id).ok_or_else(|| {
        UiDefinitionDiagnostic::error(
            "editor.lab.operation.ui_text.unsupported_target",
            format!("UI node '{node_id}' does not expose an authored text value"),
        )
    })?;
    set_ui_node_text(&mut template.root, node_id, text.to_string()).ok_or_else(|| {
        UiDefinitionDiagnostic::error(
            "editor.lab.operation.ui_text.unsupported_target",
            format!("UI node '{node_id}' does not expose an authored text value"),
        )
    })?;
    let diff = single_change_diff(
        operation,
        &document.id,
        EditorLabOperationDiffFamily::UiAuthoredValue,
        "Update",
        path.as_str().to_string(),
        Some(before),
        Some(text.to_string()),
    );
    finalize_accepted(document, operation, Some(diff))
}

fn apply_rename_operation(
    mut document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    display_name: &str,
) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
    if display_name.trim().is_empty() {
        return Err(UiDefinitionDiagnostic::error(
            "editor.lab.operation.rename.empty",
            "definition display name must not be empty",
        ));
    }
    let before = document.display_name.clone();
    document.display_name = display_name.to_string();
    let diff = single_change_diff(
        operation,
        &document.id,
        EditorLabOperationDiffFamily::EditorDocument,
        "Update",
        "display_name",
        Some(before),
        Some(display_name.to_string()),
    );
    finalize_accepted(document, operation, Some(diff))
}

fn apply_theme_color_operation(
    mut document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    token: &str,
    value: &str,
) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
    let EditorDefinitionDocumentContent::Theme(theme) = &mut document.content else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.lab.operation.theme.not_theme",
            "theme color operations require a theme document",
        ));
    };
    let before = theme.colors.get(token).cloned();
    theme.colors.insert(token.to_string(), value.to_string());
    let diff = single_change_diff(
        operation,
        &document.id,
        EditorLabOperationDiffFamily::EditorTheme,
        "Update",
        format!("colors.{token}"),
        before,
        Some(value.to_string()),
    );
    finalize_accepted(document, operation, Some(diff))
}

fn apply_add_workspace_tab_operation(
    mut document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    label: &str,
    tool_surface: &str,
) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
    let layout = workspace_layout_mut(&mut document, "add_tab")?;
    let host = first_tab_stack_mut(&mut layout.root).ok_or_else(|| {
        UiDefinitionDiagnostic::error(
            "editor.lab.operation.workspace.no_tab_stack",
            "selected workspace layout has no authored tab stack",
        )
    })?;
    let next_index = host.tabs.len() + 1;
    let tab_id = format!("authored-tab-{next_index}");
    let before = canonical_text(&host.tabs, "workspace.tabs.before")?;
    host.tabs.push(EditorWorkspacePanelTabDefinition {
        id: tab_id.clone(),
        label: label.to_string(),
        tool_surface: tool_surface.to_string(),
    });
    *host.active_tab = Some(tab_id.clone());
    let after = canonical_text(&host.tabs, "workspace.tabs.after")?;
    let diff = single_change_diff(
        operation,
        &document.id,
        EditorLabOperationDiffFamily::EditorWorkspaceLayout,
        "Insert",
        format!("workspace.tab_stack.{tab_id}"),
        Some(before),
        Some(after),
    );
    finalize_accepted(document, operation, Some(diff))
}

fn apply_split_workspace_root_operation(
    mut document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    axis: EditorWorkspaceSplitAxisDefinition,
) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
    let layout = workspace_layout_mut(&mut document, "split_root")?;
    if matches!(layout.root, EditorWorkspaceHostDefinition::Split { .. }) {
        return Err(UiDefinitionDiagnostic::error(
            "editor.lab.operation.workspace.already_split",
            "selected workspace layout root is already split",
        ));
    }
    let before = canonical_text(&layout.root, "workspace.root.before")?;
    let first = std::mem::replace(
        &mut layout.root,
        EditorWorkspaceHostDefinition::TabStack {
            id: "editor-design-empty".to_string(),
            tabs: Vec::new(),
            active_tab: None,
        },
    );
    layout.root = EditorWorkspaceHostDefinition::Split {
        id: "editor-design-root-split".to_string(),
        axis,
        fraction: 0.55,
        first: Box::new(first),
        second: Box::new(EditorWorkspaceHostDefinition::TabStack {
            id: "editor-design-secondary".to_string(),
            tabs: vec![EditorWorkspacePanelTabDefinition {
                id: "validation".to_string(),
                label: "Validation".to_string(),
                tool_surface: "definition_validation".to_string(),
            }],
            active_tab: Some("validation".to_string()),
        }),
    };
    let after = canonical_text(&layout.root, "workspace.root.after")?;
    let diff = single_change_diff(
        operation,
        &document.id,
        EditorLabOperationDiffFamily::EditorWorkspaceLayout,
        "Wrap",
        "workspace.root",
        Some(before),
        Some(after),
    );
    finalize_accepted(document, operation, Some(diff))
}

fn apply_close_workspace_last_tab_operation(
    mut document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
    let layout = workspace_layout_mut(&mut document, "close_last_tab")?;
    let host = first_tab_stack_mut(&mut layout.root).ok_or_else(|| {
        UiDefinitionDiagnostic::error(
            "editor.lab.operation.workspace.no_tab_stack",
            "selected workspace layout has no authored tab stack",
        )
    })?;
    if host.tabs.len() <= 1 {
        return Err(UiDefinitionDiagnostic::error(
            "editor.lab.operation.workspace.last_tab_guard",
            "authored tab stack must keep at least one tab",
        ));
    }
    let before = canonical_text(&host.tabs, "workspace.tabs.before")?;
    let removed = host.tabs.pop().expect("tab length was checked");
    *host.active_tab = host.tabs.last().map(|tab| tab.id.clone());
    let after = canonical_text(&host.tabs, "workspace.tabs.after")?;
    let diff = single_change_diff(
        operation,
        &document.id,
        EditorLabOperationDiffFamily::EditorWorkspaceLayout,
        "Remove",
        format!("workspace.tab_stack.{}", removed.id),
        Some(before),
        Some(after),
    );
    finalize_accepted(document, operation, Some(diff))
}

fn finalize_accepted(
    document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    diff: Option<EditorLabOperationDiff>,
) -> Result<EditorLabOperationReport, UiDefinitionDiagnostic> {
    let diagnostics = validate_editor_definition_document(&document);
    if editor_definition_has_blocking_diagnostics(&diagnostics) {
        return Ok(report_with_diagnostics(
            document,
            operation,
            EditorLabOperationStatus::Rejected,
            diff,
            diagnostics,
        ));
    }
    Ok(report_with_diagnostics(
        document,
        operation,
        if operation.preview_only {
            EditorLabOperationStatus::PreviewOnly
        } else {
            EditorLabOperationStatus::Accepted
        },
        diff,
        diagnostics,
    ))
}

fn rejected(
    document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    diagnostic: UiDefinitionDiagnostic,
) -> EditorLabOperationReport {
    report_with_diagnostics(
        document,
        operation,
        EditorLabOperationStatus::Rejected,
        None,
        vec![diagnostic],
    )
}

fn report_with_diagnostics(
    document: EditorDefinitionDocument,
    operation: &EditorLabOperation,
    status: EditorLabOperationStatus,
    diff: Option<EditorLabOperationDiff>,
    diagnostics: Vec<UiDefinitionDiagnostic>,
) -> EditorLabOperationReport {
    EditorLabOperationReport {
        operation_id: operation.id.clone(),
        document_id: operation.document_id.clone(),
        status,
        document,
        diff,
        diagnostics,
    }
}

fn single_change_diff(
    operation: &EditorLabOperation,
    document_id: &EditorDefinitionId,
    family: EditorLabOperationDiffFamily,
    kind: impl Into<String>,
    path: impl Into<String>,
    before: Option<String>,
    after: Option<String>,
) -> EditorLabOperationDiff {
    EditorLabOperationDiff {
        operation_id: operation.id.clone(),
        document_id: document_id.clone(),
        target_profile: operation.target_profile.clone(),
        changes: vec![EditorLabOperationDiffChange {
            family,
            kind: kind.into(),
            path: path.into(),
            before,
            after,
        }],
    }
}

fn ui_visual_layout_diff_change(change: UiVisualLayoutDiffChange) -> EditorLabOperationDiffChange {
    EditorLabOperationDiffChange {
        family: EditorLabOperationDiffFamily::UiVisualLayout,
        kind: ui_visual_layout_diff_kind_label(change.kind).to_string(),
        path: change.path.as_str().to_string(),
        before: change.before,
        after: change.after,
    }
}

fn ui_visual_layout_diff_kind_label(kind: UiVisualLayoutDiffChangeKind) -> &'static str {
    match kind {
        UiVisualLayoutDiffChangeKind::Insert => "Insert",
        UiVisualLayoutDiffChangeKind::Remove => "Remove",
        UiVisualLayoutDiffChangeKind::Move => "Move",
        UiVisualLayoutDiffChangeKind::Reorder => "Reorder",
        UiVisualLayoutDiffChangeKind::Update => "Update",
        UiVisualLayoutDiffChangeKind::Wrap => "Wrap",
        UiVisualLayoutDiffChangeKind::Unwrap => "Unwrap",
        UiVisualLayoutDiffChangeKind::ReplaceTemplate => "ReplaceTemplate",
    }
}

fn workspace_layout_mut<'a>(
    document: &'a mut EditorDefinitionDocument,
    operation: &str,
) -> Result<&'a mut EditorWorkspaceLayoutDefinition, UiDefinitionDiagnostic> {
    let EditorDefinitionDocumentContent::WorkspaceLayout(layout) = &mut document.content else {
        return Err(UiDefinitionDiagnostic::error(
            format!("editor.lab.operation.workspace.{operation}.not_layout"),
            "workspace layout operations require a workspace layout document",
        ));
    };
    Ok(layout)
}

struct AuthoredTabStackMut<'a> {
    tabs: &'a mut Vec<EditorWorkspacePanelTabDefinition>,
    active_tab: &'a mut Option<String>,
}

fn first_tab_stack_mut(
    host: &mut EditorWorkspaceHostDefinition,
) -> Option<AuthoredTabStackMut<'_>> {
    match host {
        EditorWorkspaceHostDefinition::TabStack {
            tabs, active_tab, ..
        } => Some(AuthoredTabStackMut { tabs, active_tab }),
        EditorWorkspaceHostDefinition::Split { first, second, .. } => {
            first_tab_stack_mut(first).or_else(|| first_tab_stack_mut(second))
        }
    }
}

fn ui_node_authored_text(node: &UiNodeDefinition, node_id: &str) -> Option<String> {
    if node.id().as_str() == node_id {
        return match node {
            UiNodeDefinition::Label { label, .. }
            | UiNodeDefinition::Button { label, .. }
            | UiNodeDefinition::Toggle { label, .. } => Some(ui_value_binding_text(label)),
            UiNodeDefinition::TextInput { value, .. } => Some(ui_value_binding_text(value)),
            _ => None,
        };
    }
    node.children()
        .iter()
        .find_map(|child| ui_node_authored_text(child, node_id))
}

fn set_ui_node_text(node: &mut UiNodeDefinition, node_id: &str, text: String) -> Option<()> {
    if node.id().as_str() == node_id {
        return match node {
            UiNodeDefinition::Label { label, .. }
            | UiNodeDefinition::Button { label, .. }
            | UiNodeDefinition::Toggle { label, .. } => {
                *label = UiValueBinding::static_text(text);
                Some(())
            }
            UiNodeDefinition::TextInput { value, .. } => {
                *value = UiValueBinding::static_text(text);
                Some(())
            }
            _ => None,
        };
    }

    for child in node.children_mut()? {
        if set_ui_node_text(child, node_id, text.clone()).is_some() {
            return Some(());
        }
    }
    None
}

fn ui_node_path(node: &UiNodeDefinition, node_id: &str) -> Option<AuthoredUiNodePath> {
    ui_node_path_segments(node, node_id, Vec::new())
        .map(|segments| AuthoredUiNodePath(segments.join("/")))
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

fn ui_value_binding_text(binding: &UiValueBinding) -> String {
    match binding {
        UiValueBinding::Static(value) => value.as_text(),
        UiValueBinding::Slot(slot) => format!("slot:{slot:?}"),
    }
}

fn canonical_text<T: Serialize + ?Sized>(
    value: &T,
    path: &str,
) -> Result<String, UiDefinitionDiagnostic> {
    to_string_pretty(value, PrettyConfig::default()).map_err(|error| {
        UiDefinitionDiagnostic::error(
            "editor.lab.operation.diff.non_deterministic",
            format!("failed to serialize deterministic diff text at {path}: {error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EditorDefinitionDocumentKind, EditorThemeDefinition, EditorWorkspaceHostDefinition,
    };
    use std::collections::BTreeMap;
    use ui_definition::{
        AuthoredUiTemplate, UiAxisDefinition, UiNodeDefinition, UiTemplateId, UiValueBinding,
        UiVisualLayoutEditKind,
    };

    #[test]
    fn theme_operation_produces_deterministic_diff() {
        let document = theme_document();
        let operation = EditorLabOperation {
            id: "theme.accent".to_string(),
            document_id: document.id.clone(),
            target_profile: "editor.workbench".to_string(),
            kind: EditorLabOperationKind::SetThemeColor {
                token: "accent".to_string(),
                value: "#3366ff".to_string(),
            },
            preview_only: false,
            source: None,
        };

        let first = apply_editor_lab_operation(&document, &operation);
        let second = apply_editor_lab_operation(&document, &operation);

        assert!(first.accepted(), "{:?}", first.diagnostics);
        assert_eq!(first.diff, second.diff);
        assert_eq!(
            first
                .diff
                .as_ref()
                .and_then(|diff| diff.changes.first())
                .and_then(|change| change.after.as_deref()),
            Some("#3366ff")
        );
    }

    #[test]
    fn invalid_theme_operation_is_rejected_without_replacing_document() {
        let document = theme_document();
        let operation = EditorLabOperation {
            id: "theme.invalid".to_string(),
            document_id: document.id.clone(),
            target_profile: "editor.workbench".to_string(),
            kind: EditorLabOperationKind::SetThemeColor {
                token: "accent".to_string(),
                value: "not-a-color".to_string(),
            },
            preview_only: false,
            source: None,
        };

        let report = apply_editor_lab_operation(&document, &operation);

        assert_eq!(report.status, EditorLabOperationStatus::Rejected);
        assert!(report.has_errors());
        assert_eq!(report.document, document);
    }

    #[test]
    fn visual_layout_operation_reuses_ui_definition_diff_path() {
        let document = ui_template_document();
        let operation = EditorLabOperation {
            id: "layout.axis".to_string(),
            document_id: document.id.clone(),
            target_profile: "editor.workbench".to_string(),
            kind: EditorLabOperationKind::UiVisualLayout(Box::new(UiVisualLayoutOperation {
                id: "axis.stack".into(),
                source_document: UiTemplateId::from("test.template"),
                target_path: AuthoredUiNodePath("root/stack".to_string()),
                expected_node_id: "stack".into(),
                target_profile: "editor.workbench".into(),
                kind: UiVisualLayoutEditKind::ChangeStackAxis {
                    axis: UiAxisDefinition::Horizontal,
                },
                source_location: None,
                preview_only: false,
            })),
            preview_only: false,
            source: None,
        };

        let report = apply_editor_lab_operation(&document, &operation);

        assert!(report.accepted(), "{:?}", report.diagnostics);
        let change = report
            .diff
            .as_ref()
            .and_then(|diff| diff.changes.first())
            .expect("visual layout operation should produce a diff change");
        assert_eq!(change.family, EditorLabOperationDiffFamily::UiVisualLayout);
        assert_eq!(change.kind, "Update");
        assert_eq!(change.path, "root/stack");
    }

    #[test]
    fn workspace_operation_rejects_last_tab_removal() {
        let document = workspace_document();
        let operation = EditorLabOperation {
            id: "workspace.close".to_string(),
            document_id: document.id.clone(),
            target_profile: "editor.workbench".to_string(),
            kind: EditorLabOperationKind::CloseWorkspaceLayoutLastTab,
            preview_only: false,
            source: None,
        };

        let report = apply_editor_lab_operation(&document, &operation);

        assert_eq!(report.status, EditorLabOperationStatus::Rejected);
        assert!(
            report.diagnostics.iter().any(
                |diagnostic| diagnostic.code == "editor.lab.operation.workspace.last_tab_guard"
            )
        );
    }

    fn theme_document() -> EditorDefinitionDocument {
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("theme"),
            "Theme",
            EditorDefinitionDocumentKind::Theme,
            EditorDefinitionDocumentContent::Theme(EditorThemeDefinition {
                id: "theme".to_string(),
                label: "Theme".to_string(),
                colors: BTreeMap::from([("accent".to_string(), "#111111".to_string())]),
                spacing: BTreeMap::new(),
                typography: BTreeMap::new(),
                radius: BTreeMap::new(),
            }),
        )
    }

    fn workspace_document() -> EditorDefinitionDocument {
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("workspace"),
            "Workspace",
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceLayout(EditorWorkspaceLayoutDefinition {
                id: "workspace".to_string(),
                label: "Workspace".to_string(),
                root: EditorWorkspaceHostDefinition::TabStack {
                    id: "root".to_string(),
                    tabs: vec![EditorWorkspacePanelTabDefinition {
                        id: "only".to_string(),
                        label: "Only".to_string(),
                        tool_surface: "definition_validation".to_string(),
                    }],
                    active_tab: Some("only".to_string()),
                },
                floating_hosts: Vec::new(),
            }),
        )
    }

    fn ui_template_document() -> EditorDefinitionDocument {
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("test.template"),
            "Template",
            EditorDefinitionDocumentKind::UiLayout,
            EditorDefinitionDocumentContent::UiTemplate(AuthoredUiTemplate {
                id: "test.template".into(),
                root: UiNodeDefinition::Column {
                    id: "root".into(),
                    children: vec![UiNodeDefinition::Stack {
                        id: "stack".into(),
                        axis: UiAxisDefinition::Vertical,
                        children: vec![UiNodeDefinition::Label {
                            id: "child".into(),
                            label: UiValueBinding::static_text("Child"),
                            availability: None,
                        }],
                    }],
                },
                templates: Vec::new(),
                menus: Vec::new(),
            }),
        )
    }
}
