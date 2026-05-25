use super::*;

pub(super) struct SelfAuthoringProvider;

impl EditorSurfaceProvider for SelfAuthoringProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SELF_AUTHORING_PROVIDER_ID,
            "Self-Authoring",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_keys_support(request, EDITOR_DESIGN_SURFACE_KEYS)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let surface_kind =
            tool_surface_kind_for_stable_key(request.stable_key()).ok_or_else(|| {
                SurfaceProviderDiagnostic::new(
                    "editor.surface.unknown_self_authoring_key",
                    format!(
                        "self-authoring provider does not recognize stable surface key `{}`",
                        request.stable_key().as_str()
                    ),
                )
            })?;
        let title = self_authoring_title(surface_kind).to_string();
        let (root, routes) = build_editor_lab_surface(
            context.theme,
            request.tool_surface_instance_id,
            &editor_lab_surface_view_model(context, surface_kind),
        );

        Ok(ProviderSurfaceFrame {
            title,
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        let command = match action {
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SelectDocument { document_id },
            ) => ShellCommand::SelectEditorDefinitionDocument { document_id },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::DuplicateSelected,
            ) => ShellCommand::DuplicateSelectedEditorDefinition,
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::RenameSelected { display_name },
            ) => ShellCommand::RenameSelectedEditorDefinition { display_name },
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::DeleteSelected) => {
                ShellCommand::DeleteSelectedEditorDefinition
            }
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SaveProjectPackage,
            ) => ShellCommand::SaveEditorLabProjectPackage,
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::ReloadProjectPackage,
            ) => ShellCommand::ReloadEditorLabProjectPackage,
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ExportSelected) => {
                ShellCommand::ExportSelectedEditorDefinition
            }
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::BuildApplyReview,
            ) => ShellCommand::BuildSelectedEditorDefinitionApplyReview,
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::RejectApplyReview,
            ) => ShellCommand::RejectSelectedEditorDefinitionApplyReview,
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected) => {
                ShellCommand::ApplySelectedEditorDefinition
            }
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::RollbackSelected,
            ) => ShellCommand::RollbackSelectedEditorDefinition,
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::ReloadLastApplied,
            ) => ShellCommand::ReloadSelectedEditorDefinitionLastApplied,
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::UndoOperation) => {
                ShellCommand::UndoEditorLabOperation
            }
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::RedoOperation) => {
                ShellCommand::RedoEditorLabOperation
            }
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::SelectUiNode {
                node_id,
            }) => ShellCommand::SelectEditorDefinitionUiNode { node_id },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SetUiNodeText { node_id, text },
            ) => ShellCommand::SetSelectedEditorDefinitionUiNodeText { node_id, text },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SetThemeColor { token, value },
            ) => ShellCommand::SetSelectedEditorThemeColor { token, value },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::AddWorkspaceLayoutTab {
                    label,
                    tool_surface,
                },
            ) => ShellCommand::AddSelectedEditorWorkspaceLayoutTab {
                label,
                tool_surface,
            },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SplitWorkspaceLayoutRoot { axis },
            ) => ShellCommand::SplitSelectedEditorWorkspaceLayoutRoot { axis },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::CloseWorkspaceLayoutLastTab,
            ) => ShellCommand::CloseSelectedEditorWorkspaceLayoutLastTab,
            _ => return Ok(None),
        };
        Ok(Some(SurfaceCommandProposal::Shell(command)))
    }
}

fn self_authoring_title(kind: ToolSurfaceKind) -> &'static str {
    match kind {
        ToolSurfaceKind::EditorDesignOutliner => "Definition Outliner",
        ToolSurfaceKind::UiHierarchy => "UI Hierarchy",
        ToolSurfaceKind::UiCanvas => "UI Canvas",
        ToolSurfaceKind::StyleInspector => "Style Inspector",
        ToolSurfaceKind::Bindings => "Bindings",
        ToolSurfaceKind::DockLayoutPreview => "Dock Layout Preview",
        ToolSurfaceKind::ThemeEditor => "Theme Editor",
        ToolSurfaceKind::ShortcutEditor => "Shortcut Editor",
        ToolSurfaceKind::MenuEditor => "Menu Editor",
        ToolSurfaceKind::DefinitionValidation => "Definition Validation",
        ToolSurfaceKind::CommandDiff => "Command Diff",
        _ => "Self-Authoring",
    }
}

fn editor_lab_surface_view_model(
    context: &SurfaceProviderBuildContext<'_>,
    kind: ToolSurfaceKind,
) -> EditorLabSurfaceViewModel {
    match kind {
        ToolSurfaceKind::EditorDesignOutliner => {
            EditorLabSurfaceViewModel::DefinitionHierarchy(definition_hierarchy_view_model(context))
        }
        ToolSurfaceKind::UiHierarchy => {
            EditorLabSurfaceViewModel::Inspector(ui_hierarchy_view_model(context))
        }
        ToolSurfaceKind::UiCanvas => {
            EditorLabSurfaceViewModel::CanvasPreview(canvas_preview_view_model(context))
        }
        ToolSurfaceKind::StyleInspector => {
            EditorLabSurfaceViewModel::Inspector(style_inspector_view_model(context))
        }
        ToolSurfaceKind::Bindings => {
            EditorLabSurfaceViewModel::Review(bindings_review_view_model(context))
        }
        ToolSurfaceKind::DockLayoutPreview => {
            EditorLabSurfaceViewModel::Palette(dock_layout_palette_view_model(context))
        }
        ToolSurfaceKind::ThemeEditor => {
            EditorLabSurfaceViewModel::Inspector(theme_editor_view_model(context))
        }
        ToolSurfaceKind::ShortcutEditor => {
            EditorLabSurfaceViewModel::Review(shortcut_review_view_model(context))
        }
        ToolSurfaceKind::MenuEditor => {
            EditorLabSurfaceViewModel::Review(menu_review_view_model(context))
        }
        ToolSurfaceKind::DefinitionValidation => {
            EditorLabSurfaceViewModel::Diagnostics(diagnostics_view_model(context))
        }
        ToolSurfaceKind::CommandDiff => {
            EditorLabSurfaceViewModel::Review(command_review_view_model(context))
        }
        _ => EditorLabSurfaceViewModel::Degraded(EditorLabDegradedViewModel {
            title: "Unsupported Editor Lab Surface".to_string(),
            reason: format!("surface kind {kind:?} is not part of the Editor Lab shell"),
            details: Vec::new(),
            diagnostics: Vec::new(),
            recovery_actions: Vec::new(),
        }),
    }
}

fn definition_hierarchy_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabDefinitionHierarchyViewModel {
    let state = context.shell_state.self_authoring();
    let rows = state
        .draft_documents()
        .map(|document| EditorLabDefinitionRowViewModel {
            document_id: document.id.as_str().to_string(),
            label: document.display_name.clone(),
            kind: format!("{:?}", document.kind),
            lifecycle: format!("{:?}", document.lifecycle_state),
            selected: Some(&document.id) == state.selected_document_id(),
            source_path: Some(document.display_name.clone()),
            diagnostic_count: state.diagnostics_for_document(&document.id).len(),
            select_action: EditorDefinitionSurfaceAction::SelectDocument {
                document_id: document.id.as_str().to_string(),
            },
        })
        .collect();

    EditorLabDefinitionHierarchyViewModel {
        title: "Definition Hierarchy".to_string(),
        selected_document: selected_document_label(state),
        rows,
        actions: selected_document_actions(state),
    }
}

fn ui_hierarchy_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabInspectorViewModel {
    let state = context.shell_state.self_authoring();
    let mut fields = Vec::new();
    let mut actions = Vec::new();
    let mut diagnostics = selected_diagnostics_view_models(state);

    if let Some(document) = state.selected_document() {
        fields.push(field("Document", document.display_name.clone()));
        match &document.content {
            editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) => {
                fields.push(field("Template", template.id.as_str().to_string()));
                fields.push(field("Root", template.root.id().as_str().to_string()));
                fields.push(field(
                    "Selected Node",
                    state.selected_ui_node_id().unwrap_or("none").to_string(),
                ));
                actions = ui_node_selection_actions(&template.root, state.selected_ui_node_id());
                if let Some(node_id) = state.selected_ui_node_id()
                    && let Some(text) = ui_node_authored_text(&template.root, node_id)
                {
                    fields.push(EditorLabInspectorFieldViewModel {
                        label: "Node text".to_string(),
                        value: text.clone(),
                        text_field: Some(EditorLabTextFieldViewModel::new(
                            "Edit selected node text",
                            text,
                            "Text",
                            EditorDefinitionSurfaceAction::SetUiNodeText {
                                node_id: node_id.to_string(),
                                text: String::new(),
                            },
                        )),
                    });
                }
            }
            _ => diagnostics.push(EditorLabDiagnosticViewModel {
                severity: "Warning".to_string(),
                code: "editor.lab.ui_hierarchy.not_ui_template".to_string(),
                message: "selected definition is not a UI template".to_string(),
                path: None,
            }),
        }
    }

    EditorLabInspectorViewModel {
        title: "UI Hierarchy".to_string(),
        selected_document: selected_document_label(state),
        fields,
        actions,
        diagnostics,
    }
}

fn canvas_preview_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabCanvasPreviewViewModel {
    let state = context.shell_state.self_authoring();
    let retained_preview_available = state.formed_selected_preview(context.theme).is_some();
    let mut status_lines = Vec::new();
    let mut actions = Vec::new();

    if let Some(document) = state.selected_document() {
        status_lines.push(format!("document kind: {:?}", document.kind));
        match &document.content {
            editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) => {
                status_lines.push(format!("template: {}", template.id.as_str()));
                status_lines.push(format!(
                    "selected node: {}",
                    state.selected_ui_node_id().unwrap_or("none")
                ));
                actions.extend(ui_node_selection_actions(
                    &template.root,
                    state.selected_ui_node_id(),
                ));
                if let Some(node_id) = state.selected_ui_node_id()
                    && let Some(text) = ui_node_authored_text(&template.root, node_id)
                {
                    actions.push(EditorLabActionViewModel::enabled(
                        format!("Apply canvas text edit to {node_id}"),
                        EditorDefinitionSurfaceAction::SetUiNodeText {
                            node_id: node_id.to_string(),
                            text,
                        },
                    ));
                }
            }
            _ => {
                status_lines
                    .push("Selected definition cannot form a retained UI preview".to_string());
                actions.extend(select_ui_layout_actions(state));
            }
        }
    } else {
        status_lines.push("no definition document is selected".to_string());
        actions.extend(select_ui_layout_actions(state));
    }

    if let Some(report) = state.last_operation_report() {
        status_lines.push(format!(
            "last operation: {} {:?}",
            report.operation_id, report.status
        ));
        status_lines.push(format!(
            "last operation diagnostics: {}",
            report.diagnostics.len()
        ));
    }

    let history = state.operation_history_snapshot();
    status_lines.push(format!(
        "operation history: undo={} redo={}",
        history.undo_count, history.redo_count
    ));
    actions.extend(operation_history_actions(state));

    EditorLabCanvasPreviewViewModel {
        title: "UI Canvas".to_string(),
        selected_document: selected_document_label(state),
        retained_preview_available,
        status_lines,
        actions,
    }
}

fn style_inspector_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabInspectorViewModel {
    let state = context.shell_state.self_authoring();
    let mut fields = Vec::new();
    let actions = selected_document_actions(state);
    if let Some(document) = state.selected_document() {
        fields.push(field("Document id", document.id.as_str().to_string()));
        fields.push(field("Kind", format!("{:?}", document.kind)));
        fields.push(field(
            "Lifecycle",
            format!("{:?}", document.lifecycle_state),
        ));
        fields.push(EditorLabInspectorFieldViewModel {
            label: "Display name".to_string(),
            value: document.display_name.clone(),
            text_field: Some(EditorLabTextFieldViewModel::new(
                "Rename selected definition",
                document.display_name.clone(),
                "Display name",
                EditorDefinitionSurfaceAction::RenameSelected {
                    display_name: document.display_name.clone(),
                },
            )),
        });
        if let editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) =
            &document.content
            && let Some(node_id) = state.selected_ui_node_id()
            && let Some(text) = ui_node_authored_text(&template.root, node_id)
        {
            fields.push(EditorLabInspectorFieldViewModel {
                label: "Selected node text".to_string(),
                value: text.clone(),
                text_field: Some(EditorLabTextFieldViewModel::new(
                    "Edit selected node text",
                    text,
                    "Text",
                    EditorDefinitionSurfaceAction::SetUiNodeText {
                        node_id: node_id.to_string(),
                        text: String::new(),
                    },
                )),
            });
        }
    }
    EditorLabInspectorViewModel {
        title: "Style Inspector".to_string(),
        selected_document: selected_document_label(state),
        fields,
        actions,
        diagnostics: selected_diagnostics_view_models(state),
    }
}

fn bindings_review_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabReviewViewModel {
    let state = context.shell_state.self_authoring();
    let mut summary = Vec::new();
    if let Some(document) = state.selected_document() {
        match &document.content {
            editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) => {
                summary.push(format!("template: {}", template.id.as_str()));
                summary.push(format!("local templates: {}", template.templates.len()));
                summary.push(format!("menus: {}", template.menus.len()));
            }
            editor_definition::EditorDefinitionDocumentContent::EditorBindings(bindings) => {
                summary.push(format!("toolbar template: {}", bindings.toolbar.template));
                summary.push(format!(
                    "surface templates: {}",
                    bindings.surface_templates.len()
                ));
            }
            _ => summary.push("selected definition has no UI binding slots".to_string()),
        }
    }
    review_view_model("Bindings", state, summary)
}

fn dock_layout_palette_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabPaletteViewModel {
    let state = context.shell_state.self_authoring();
    let mut diagnostics = selected_diagnostics_view_models(state);
    let mut layout_actions = select_workspace_layout_actions(state);

    if let Some(document) = state.selected_document()
        && let editor_definition::EditorDefinitionDocumentContent::WorkspaceLayout(layout) =
            &document.content
    {
        let next_label = next_workspace_tab_label(layout);
        layout_actions.extend([
            EditorLabActionViewModel::enabled(
                format!("Add {next_label}"),
                EditorDefinitionSurfaceAction::AddWorkspaceLayoutTab {
                    label: next_label,
                    tool_surface: "definition_validation".to_string(),
                },
            ),
            EditorLabActionViewModel::enabled(
                "Split root horizontally",
                EditorDefinitionSurfaceAction::SplitWorkspaceLayoutRoot {
                    axis: "horizontal".to_string(),
                },
            ),
            EditorLabActionViewModel::enabled(
                "Close last tab",
                EditorDefinitionSurfaceAction::CloseWorkspaceLayoutLastTab,
            ),
        ]);
    } else {
        diagnostics.push(EditorLabDiagnosticViewModel {
            severity: "Warning".to_string(),
            code: "editor.lab.workspace_layout.not_selected".to_string(),
            message: "select an authored workspace layout to edit layout controls".to_string(),
            path: None,
        });
    }

    EditorLabPaletteViewModel {
        title: "Dock Layout Preview".to_string(),
        categories: vec![
            EditorLabPaletteCategoryViewModel {
                label: "Workspace layout documents".to_string(),
                items: select_workspace_layout_actions(state),
            },
            EditorLabPaletteCategoryViewModel {
                label: "Layout controls".to_string(),
                items: layout_actions,
            },
            EditorLabPaletteCategoryViewModel {
                label: "Review".to_string(),
                items: apply_review_actions(state),
            },
        ],
        diagnostics,
    }
}

fn theme_editor_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabInspectorViewModel {
    let state = context.shell_state.self_authoring();
    let mut fields = Vec::new();
    let mut actions = select_theme_actions(state);
    if let Some(document) = state.selected_document()
        && let editor_definition::EditorDefinitionDocumentContent::Theme(theme) = &document.content
    {
        fields.push(field("Theme", theme.label.clone()));
        for (token, value) in &theme.colors {
            fields.push(EditorLabInspectorFieldViewModel {
                label: format!("Color {token}"),
                value: value.clone(),
                text_field: Some(EditorLabTextFieldViewModel::new(
                    format!("Set {token}"),
                    value.clone(),
                    "#rrggbb",
                    EditorDefinitionSurfaceAction::SetThemeColor {
                        token: token.clone(),
                        value: value.clone(),
                    },
                )),
            });
        }
        actions.extend(apply_review_actions(state));
    }
    EditorLabInspectorViewModel {
        title: "Theme Editor".to_string(),
        selected_document: selected_document_label(state),
        fields,
        actions,
        diagnostics: selected_diagnostics_view_models(state),
    }
}

fn shortcut_review_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabReviewViewModel {
    let state = context.shell_state.self_authoring();
    let mut summary = Vec::new();
    if let Some(document) = state.selected_document()
        && let editor_definition::EditorDefinitionDocumentContent::Shortcuts(shortcuts) =
            &document.content
    {
        summary.push(format!("shortcut set: {}", shortcuts.label));
        summary.push(format!("shortcuts: {}", shortcuts.shortcuts.len()));
        for shortcut in &shortcuts.shortcuts {
            summary.push(format!("{} -> {}", shortcut.chord, shortcut.command));
        }
    } else {
        summary.push("select a shortcut definition to inspect shortcut routes".to_string());
    }
    review_view_model("Shortcut Editor", state, summary)
}

fn menu_review_view_model(context: &SurfaceProviderBuildContext<'_>) -> EditorLabReviewViewModel {
    let state = context.shell_state.self_authoring();
    let mut summary = Vec::new();
    if let Some(document) = state.selected_document()
        && let editor_definition::EditorDefinitionDocumentContent::Menu(menu) = &document.content
    {
        summary.push(format!("menu: {}", menu.label));
        summary.push(format!("items: {}", menu.items.len()));
        for item in &menu.items {
            summary.push(format!("{} command={:?}", item.label, item.command));
        }
    } else {
        summary.push("select a menu definition to inspect menu command routes".to_string());
    }
    review_view_model("Menu Editor", state, summary)
}

fn diagnostics_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabDiagnosticsViewModel {
    let state = context.shell_state.self_authoring();
    EditorLabDiagnosticsViewModel {
        title: "Definition Validation".to_string(),
        selected_document: selected_document_label(state),
        diagnostics: editor_lab_diagnostics_view_models(state),
        actions: apply_review_actions(state),
    }
}

fn command_review_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> EditorLabReviewViewModel {
    let state = context.shell_state.self_authoring();
    let mut summary = state
        .build_apply_preview()
        .map(|preview| preview.summary)
        .unwrap_or_else(|| vec!["No definition selected".to_string()]);
    summary.push("Preview Console".to_string());
    for line in context.app.console_lines().iter().rev().take(6).rev() {
        summary.push(format!("[{:?}] {}", line.kind, line.text));
    }
    if let Some(last_apply) = state.last_apply_preview() {
        summary.push(format!("last apply preview: {}", last_apply.display_name));
    }
    if let Some(source) = state.last_saved_project_package_source() {
        summary.push(format!("project package saved: {} bytes", source.len()));
    }
    if let Some(source) = state.last_loaded_project_package_source() {
        summary.push(format!("project package loaded: {} bytes", source.len()));
    }
    if let Some(source) = state.last_invalid_project_package_source() {
        summary.push(format!(
            "invalid project package preserved: {} bytes",
            source.len()
        ));
    }
    if let Some(review) = state.last_apply_review() {
        summary.push(format!(
            "apply review: {} {:?} diffs={} diagnostics={}",
            review.display_name,
            review.status,
            review.diff_rows.len(),
            review.diagnostics.len()
        ));
        for row in review.diff_rows.iter().take(3) {
            summary.push(format!(
                "apply diff: {:?}/{:?} {} {:?} -> {:?}: {}",
                row.family, row.kind, row.path, row.before, row.after, row.summary
            ));
        }
    }
    if let Some(report) = context.app.last_editor_definition_activation_report() {
        summary.push(format!(
            "activation report: {} {:?} preserved={}",
            report.display_name, report.status, report.previous_state_preserved
        ));
        for diagnostic in report.diagnostics.iter().take(3) {
            summary.push(format!(
                "activation diagnostic: {:?} {} {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            ));
        }
    }
    if let Some(record) = state.last_rollback_record() {
        summary.push(format!(
            "rollback: {} {:?}",
            record.display_name, record.status
        ));
    }
    let history = state.operation_history_snapshot();
    summary.push(format!(
        "Operation History: undo={} redo={}",
        history.undo_count, history.redo_count
    ));
    if let Some(report) = state.last_operation_report() {
        summary.push(format!(
            "last operation: {} {:?}",
            report.operation_id, report.status
        ));
        if let Some(diff) = &report.diff {
            for change in diff.changes.iter().take(4) {
                summary.push(format!(
                    "diff: {:?} {} {:?} -> {:?}",
                    change.family, change.path, change.before, change.after
                ));
            }
        }
        for diagnostic in report.diagnostics.iter().take(3) {
            summary.push(format!(
                "diagnostic: {:?} {} {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            ));
        }
    }
    review_view_model("Command Diff and Preview Console", state, summary)
}

fn review_view_model(
    title: impl Into<String>,
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
    summary_lines: Vec<String>,
) -> EditorLabReviewViewModel {
    let mut actions = operation_history_actions(state);
    actions.extend(apply_review_actions(state));
    EditorLabReviewViewModel {
        title: title.into(),
        selected_document: selected_document_label(state),
        summary_lines,
        actions,
        diagnostics: selected_diagnostics_view_models(state),
    }
}

fn selected_document_actions(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<EditorLabActionViewModel> {
    let has_selection = state.selected_document().is_some();
    [
        EditorLabActionViewModel::enabled(
            "Duplicate selected definition",
            EditorDefinitionSurfaceAction::DuplicateSelected,
        ),
        EditorLabActionViewModel::enabled(
            "Delete selected draft",
            EditorDefinitionSurfaceAction::DeleteSelected,
        ),
        EditorLabActionViewModel::enabled(
            "Export selected definition",
            EditorDefinitionSurfaceAction::ExportSelected,
        ),
    ]
    .into_iter()
    .map(|action| {
        if has_selection {
            action
        } else {
            action.disabled("no definition document is selected")
        }
    })
    .collect()
}

fn apply_review_actions(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<EditorLabActionViewModel> {
    let has_selection = state.selected_document().is_some();
    let mut actions = Vec::new();
    actions.push(EditorLabActionViewModel::enabled(
        "Save project package",
        EditorDefinitionSurfaceAction::SaveProjectPackage,
    ));
    actions.push(
        EditorLabActionViewModel::enabled(
            "Reload project package",
            EditorDefinitionSurfaceAction::ReloadProjectPackage,
        )
        .disabled_if(
            state.last_saved_project_package_source().is_none(),
            "no Editor Lab project package has been saved in this session",
        ),
    );
    for action in [
        EditorLabActionViewModel::enabled(
            "Build apply review",
            EditorDefinitionSurfaceAction::BuildApplyReview,
        ),
        EditorLabActionViewModel::enabled(
            "Reject apply review",
            EditorDefinitionSurfaceAction::RejectApplyReview,
        ),
        EditorLabActionViewModel::enabled(
            "Apply selected definition",
            EditorDefinitionSurfaceAction::ApplySelected,
        ),
        EditorLabActionViewModel::enabled(
            "Rollback selected definition",
            EditorDefinitionSurfaceAction::RollbackSelected,
        ),
        EditorLabActionViewModel::enabled(
            "Reload last applied",
            EditorDefinitionSurfaceAction::ReloadLastApplied,
        ),
    ] {
        actions.push(if has_selection {
            action
        } else {
            action.disabled("no definition document is selected")
        });
    }
    actions
}

fn operation_history_actions(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<EditorLabActionViewModel> {
    let history = state.operation_history_snapshot();
    [
        (
            EditorLabActionViewModel::enabled(
                "Undo Editor Lab operation",
                EditorDefinitionSurfaceAction::UndoOperation,
            ),
            history.can_undo,
            "no accepted Editor Lab operation is available to undo",
        ),
        (
            EditorLabActionViewModel::enabled(
                "Redo Editor Lab operation",
                EditorDefinitionSurfaceAction::RedoOperation,
            ),
            history.can_redo,
            "no undone Editor Lab operation is available to redo",
        ),
    ]
    .into_iter()
    .map(|(action, enabled, reason)| {
        if enabled {
            action
        } else {
            action.disabled(reason)
        }
    })
    .collect()
}

fn select_ui_layout_actions(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<EditorLabActionViewModel> {
    state
        .draft_documents()
        .filter(|document| {
            matches!(
                &document.content,
                editor_definition::EditorDefinitionDocumentContent::UiTemplate(_)
            )
        })
        .map(|document| {
            EditorLabActionViewModel::enabled(
                format!("Open preview for {}", document.display_name),
                EditorDefinitionSurfaceAction::SelectDocument {
                    document_id: document.id.as_str().to_string(),
                },
            )
            .selected(Some(&document.id) == state.selected_document_id())
        })
        .collect()
}

fn select_workspace_layout_actions(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<EditorLabActionViewModel> {
    state
        .draft_documents()
        .filter(|document| {
            matches!(
                &document.content,
                editor_definition::EditorDefinitionDocumentContent::WorkspaceLayout(_)
            )
        })
        .map(|document| {
            EditorLabActionViewModel::enabled(
                format!("Select layout {}", document.display_name),
                EditorDefinitionSurfaceAction::SelectDocument {
                    document_id: document.id.as_str().to_string(),
                },
            )
            .selected(Some(&document.id) == state.selected_document_id())
        })
        .collect()
}

fn select_theme_actions(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<EditorLabActionViewModel> {
    state
        .draft_documents()
        .filter(|document| {
            matches!(
                &document.content,
                editor_definition::EditorDefinitionDocumentContent::Theme(_)
            )
        })
        .map(|document| {
            EditorLabActionViewModel::enabled(
                format!("Select theme {}", document.display_name),
                EditorDefinitionSurfaceAction::SelectDocument {
                    document_id: document.id.as_str().to_string(),
                },
            )
            .selected(Some(&document.id) == state.selected_document_id())
        })
        .collect()
}

fn ui_node_selection_actions(
    node: &ui_definition::UiNodeDefinition,
    selected_node_id: Option<&str>,
) -> Vec<EditorLabActionViewModel> {
    let selected = selected_node_id == Some(node.id().as_str());
    let mut actions = vec![
        EditorLabActionViewModel::enabled(
            format!("Select UI node {}", node.id().as_str()),
            EditorDefinitionSurfaceAction::SelectUiNode {
                node_id: node.id().as_str().to_string(),
            },
        )
        .selected(selected),
    ];
    for child in node.children() {
        actions.extend(ui_node_selection_actions(child, selected_node_id));
    }
    actions
}

fn selected_diagnostics_view_models(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<EditorLabDiagnosticViewModel> {
    state
        .selected_diagnostics()
        .into_iter()
        .map(|diagnostic| EditorLabDiagnosticViewModel {
            severity: format!("{:?}", diagnostic.severity),
            code: diagnostic.code,
            message: diagnostic.message,
            path: diagnostic.path.map(|path| format!("{path:?}")),
        })
        .collect()
}

fn editor_lab_diagnostics_view_models(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<EditorLabDiagnosticViewModel> {
    let mut diagnostics = selected_diagnostics_view_models(state);
    if let Some(report) = state.last_operation_report() {
        diagnostics.extend(report.diagnostics.iter().map(|diagnostic| {
            EditorLabDiagnosticViewModel {
                severity: format!("{:?}", diagnostic.severity),
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
                path: diagnostic.path.as_ref().map(|path| format!("{path:?}")),
            }
        }));
    }
    diagnostics
}

fn selected_document_label(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Option<String> {
    state
        .selected_document()
        .map(|document| format!("{} ({})", document.display_name, document.id.as_str()))
}

fn field(label: impl Into<String>, value: impl Into<String>) -> EditorLabInspectorFieldViewModel {
    EditorLabInspectorFieldViewModel {
        label: label.into(),
        value: value.into(),
        text_field: None,
    }
}

fn next_workspace_tab_label(layout: &editor_definition::EditorWorkspaceLayoutDefinition) -> String {
    format!(
        "{} Review {}",
        layout.label,
        first_tab_stack_len(&layout.root).unwrap_or(0) + 1
    )
}

fn first_tab_stack_len(host: &editor_definition::EditorWorkspaceHostDefinition) -> Option<usize> {
    match host {
        editor_definition::EditorWorkspaceHostDefinition::TabStack { tabs, .. } => Some(tabs.len()),
        editor_definition::EditorWorkspaceHostDefinition::Split { first, second, .. } => {
            first_tab_stack_len(first).or_else(|| first_tab_stack_len(second))
        }
    }
}

fn ui_node_authored_text(node: &ui_definition::UiNodeDefinition, node_id: &str) -> Option<String> {
    if node.id().as_str() == node_id {
        return match node {
            ui_definition::UiNodeDefinition::Label { label, .. }
            | ui_definition::UiNodeDefinition::Button { label, .. }
            | ui_definition::UiNodeDefinition::Toggle { label, .. } => {
                Some(ui_value_binding_text(label))
            }
            ui_definition::UiNodeDefinition::TextInput { value, .. } => {
                Some(ui_value_binding_text(value))
            }
            _ => None,
        };
    }
    node.children()
        .iter()
        .find_map(|child| ui_node_authored_text(child, node_id))
}

fn ui_value_binding_text(binding: &ui_definition::UiValueBinding) -> String {
    match binding {
        ui_definition::UiValueBinding::Static(value) => value.as_text(),
        ui_definition::UiValueBinding::Slot(slot) => format!("slot:{slot:?}"),
    }
}
