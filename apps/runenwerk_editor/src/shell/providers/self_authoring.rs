use super::*;

pub(super) fn is_self_authoring_surface(kind: ToolSurfaceKind) -> bool {
    matches!(
        kind,
        ToolSurfaceKind::EditorDesignOutliner
            | ToolSurfaceKind::UiHierarchy
            | ToolSurfaceKind::UiCanvas
            | ToolSurfaceKind::StyleInspector
            | ToolSurfaceKind::Bindings
            | ToolSurfaceKind::DockLayoutPreview
            | ToolSurfaceKind::ThemeEditor
            | ToolSurfaceKind::ShortcutEditor
            | ToolSurfaceKind::MenuEditor
            | ToolSurfaceKind::DefinitionValidation
            | ToolSurfaceKind::CommandDiff
    )
}

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
        stable_keys_or_legacy_kind_support(
            request,
            EDITOR_DESIGN_SURFACE_KEYS,
            is_self_authoring_surface,
        )
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
        let (root, routes) = match surface_kind {
            ToolSurfaceKind::UiCanvas => context
                .shell_state
                .self_authoring()
                .formed_selected_preview_with_scope(
                    context.theme,
                    Some(editor_shell::surface_widget_scope_base(
                        request.tool_surface_instance_id,
                    )),
                )
                .map(|product| (product.root, SurfaceRouteTable::empty()))
                .unwrap_or_else(|| {
                    build_self_authoring_control_panel(
                        context.theme,
                        request.tool_surface_instance_id,
                        vec!["No retained preview available".to_string()],
                        Vec::new(),
                    )
                }),
            _ => build_self_authoring_control_panel(
                context.theme,
                request.tool_surface_instance_id,
                self_authoring_lines(context, surface_kind),
                self_authoring_actions(context, surface_kind),
            ),
        };

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
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ExportSelected) => {
                ShellCommand::ExportSelectedEditorDefinition
            }
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected) => {
                ShellCommand::ApplySelectedEditorDefinition
            }
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::RollbackSelected,
            ) => ShellCommand::RollbackSelectedEditorDefinition,
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

fn self_authoring_lines(
    context: &SurfaceProviderBuildContext<'_>,
    kind: ToolSurfaceKind,
) -> Vec<String> {
    let state = context.shell_state.self_authoring();
    match kind {
        ToolSurfaceKind::EditorDesignOutliner => state
            .draft_documents()
            .map(|document| {
                let marker = if Some(&document.id) == state.selected_document_id() {
                    "*"
                } else {
                    " "
                };
                format!("{marker} {} [{:?}]", document.display_name, document.kind)
            })
            .collect(),
        ToolSurfaceKind::UiHierarchy => state
            .selected_document()
            .map(|document| match &document.content {
                editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) => {
                    ui_node_hierarchy_lines(&template.root, 0, state.selected_ui_node_id())
                }
                _ => vec!["Selected definition is not a UI template".to_string()],
            })
            .unwrap_or_else(|| vec!["No definition selected".to_string()]),
        ToolSurfaceKind::StyleInspector => vec![
            "Theme tokens are editor-owned definition data".to_string(),
            "Retained preview uses the active ThemeTokens until a theme document is applied"
                .to_string(),
        ],
        ToolSurfaceKind::Bindings => state
            .selected_document()
            .map(|document| match &document.content {
                editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) => {
                    vec![
                        format!("template: {}", template.id),
                        format!("local templates: {}", template.templates.len()),
                        format!("menus: {}", template.menus.len()),
                    ]
                }
                editor_definition::EditorDefinitionDocumentContent::EditorBindings(bindings) => {
                    vec![
                        format!("toolbar: {}", bindings.toolbar.template),
                        format!("surface templates: {}", bindings.surface_templates.len()),
                    ]
                }
                _ => vec!["Selected editor definition has no UI slots".to_string()],
            })
            .unwrap_or_else(|| vec!["No definition selected".to_string()]),
        ToolSurfaceKind::DockLayoutPreview => {
            if let Some(document) = state.selected_document()
                && let editor_definition::EditorDefinitionDocumentContent::WorkspaceLayout(layout) =
                    &document.content
            {
                return vec![
                    format!("layout: {}", layout.label),
                    format!("root: {}", workspace_host_summary(&layout.root)),
                    format!("floating hosts: {}", layout.floating_hosts.len()),
                ];
            }
            let workspace = context.shell_state.workspace_state();
            vec![
                "Select an authored workspace layout to edit".to_string(),
                format!("active hosts: {}", workspace.hosts().count()),
                format!("active tab stacks: {}", workspace.tab_stacks().count()),
                format!("active panels: {}", workspace.panels().count()),
                format!(
                    "active tool surfaces: {}",
                    workspace.tool_surfaces().count()
                ),
            ]
        }
        ToolSurfaceKind::ThemeEditor => vec![
            "Theme documents validate in editor_definition".to_string(),
            "Apply keeps runtime state unchanged until explicit shell command".to_string(),
        ],
        ToolSurfaceKind::ShortcutEditor => vec![
            "Shortcut documents report duplicate chord diagnostics".to_string(),
            "Platform override execution remains app-owned".to_string(),
        ],
        ToolSurfaceKind::MenuEditor => vec![
            "Menu documents own labels, hierarchy, availability refs, and command refs".to_string(),
            "Command execution remains outside editor_definition".to_string(),
        ],
        ToolSurfaceKind::DefinitionValidation => {
            let diagnostics = state.selected_diagnostics();
            if diagnostics.is_empty() {
                return vec!["No blocking definition diagnostics".to_string()];
            }
            diagnostics
                .into_iter()
                .map(|diagnostic| {
                    format!(
                        "{:?} {}: {}",
                        diagnostic.severity, diagnostic.code, diagnostic.message
                    )
                })
                .collect()
        }
        ToolSurfaceKind::CommandDiff => state
            .build_apply_preview()
            .map(|preview| preview.summary)
            .unwrap_or_else(|| vec!["No definition selected".to_string()]),
        _ => vec!["Unsupported self-authoring surface".to_string()],
    }
}

fn self_authoring_actions(
    context: &SurfaceProviderBuildContext<'_>,
    kind: ToolSurfaceKind,
) -> Vec<(String, SurfaceLocalAction)> {
    let state = context.shell_state.self_authoring();
    match kind {
        ToolSurfaceKind::EditorDesignOutliner => {
            let mut actions = state
                .draft_documents()
                .map(|document| {
                    (
                        format!("Select {}", document.display_name),
                        SurfaceLocalAction::EditorDefinition(
                            EditorDefinitionSurfaceAction::SelectDocument {
                                document_id: document.id.as_str().to_string(),
                            },
                        ),
                    )
                })
                .collect::<Vec<_>>();
            actions.extend([
                (
                    "Duplicate".to_string(),
                    SurfaceLocalAction::EditorDefinition(
                        EditorDefinitionSurfaceAction::DuplicateSelected,
                    ),
                ),
                (
                    "Delete".to_string(),
                    SurfaceLocalAction::EditorDefinition(
                        EditorDefinitionSurfaceAction::DeleteSelected,
                    ),
                ),
                (
                    "Export".to_string(),
                    SurfaceLocalAction::EditorDefinition(
                        EditorDefinitionSurfaceAction::ExportSelected,
                    ),
                ),
            ]);
            actions
        }
        ToolSurfaceKind::UiHierarchy => selected_ui_node_actions(state),
        ToolSurfaceKind::StyleInspector => vec![(
            "Rename Draft".to_string(),
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::RenameSelected {
                display_name: "Retained draft".to_string(),
            }),
        )],
        ToolSurfaceKind::ThemeEditor => vec![
            (
                "Select Theme".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::SelectDocument {
                        document_id: "runenwerk.editor.theme.default".to_string(),
                    },
                ),
            ),
            (
                "Set Accent".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::SetThemeColor {
                        token: "accent".to_string(),
                        value: "#5f8cff".to_string(),
                    },
                ),
            ),
        ],
        ToolSurfaceKind::DefinitionValidation | ToolSurfaceKind::CommandDiff => vec![
            (
                "Apply".to_string(),
                SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected),
            ),
            (
                "Rollback".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::RollbackSelected,
                ),
            ),
        ],
        ToolSurfaceKind::DockLayoutPreview => vec![
            (
                "Select Layout".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::SelectDocument {
                        document_id: "runenwerk.editor.layout.editor_design".to_string(),
                    },
                ),
            ),
            (
                "Add Tab".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::AddWorkspaceLayoutTab {
                        label: "Authored Tab".to_string(),
                        tool_surface: "definition_validation".to_string(),
                    },
                ),
            ),
            (
                "Split Root".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::SplitWorkspaceLayoutRoot {
                        axis: "horizontal".to_string(),
                    },
                ),
            ),
            (
                "Close Last Tab".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::CloseWorkspaceLayoutLastTab,
                ),
            ),
            (
                "Apply".to_string(),
                SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected),
            ),
            (
                "Rollback".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::RollbackSelected,
                ),
            ),
        ],
        ToolSurfaceKind::Bindings
        | ToolSurfaceKind::ShortcutEditor
        | ToolSurfaceKind::MenuEditor => vec![
            (
                "Duplicate".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::DuplicateSelected,
                ),
            ),
            (
                "Apply".to_string(),
                SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected),
            ),
            (
                "Rollback".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::RollbackSelected,
                ),
            ),
        ],
        _ => Vec::new(),
    }
}

fn selected_ui_node_actions(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<(String, SurfaceLocalAction)> {
    let Some(document) = state.selected_document() else {
        return Vec::new();
    };
    let editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) =
        &document.content
    else {
        return Vec::new();
    };
    let mut actions = ui_node_selection_actions(&template.root);
    if let Some(node_id) = state.selected_ui_node_id() {
        actions.push((
            "Set Text".to_string(),
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::SetUiNodeText {
                node_id: node_id.to_string(),
                text: "Edited in self-authoring".to_string(),
            }),
        ));
    }
    actions
}

fn ui_node_selection_actions(
    node: &ui_definition::UiNodeDefinition,
) -> Vec<(String, SurfaceLocalAction)> {
    let mut actions = vec![(
        format!("Select {}", node.id()),
        SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::SelectUiNode {
            node_id: node.id().as_str().to_string(),
        }),
    )];
    for child in node.children() {
        actions.extend(ui_node_selection_actions(child));
    }
    actions
}

fn ui_node_hierarchy_lines(
    node: &ui_definition::UiNodeDefinition,
    depth: usize,
    selected_node_id: Option<&str>,
) -> Vec<String> {
    let marker = if selected_node_id == Some(node.id().as_str()) {
        "* "
    } else {
        "  "
    };
    let mut lines = vec![format!(
        "{}{}{}",
        "  ".repeat(depth),
        marker,
        node.id().as_str()
    )];
    for child in node.children() {
        lines.extend(ui_node_hierarchy_lines(child, depth + 1, selected_node_id));
    }
    lines
}

fn workspace_host_summary(host: &editor_definition::EditorWorkspaceHostDefinition) -> String {
    match host {
        editor_definition::EditorWorkspaceHostDefinition::TabStack { tabs, .. } => {
            format!("tab_stack tabs={}", tabs.len())
        }
        editor_definition::EditorWorkspaceHostDefinition::Split {
            axis,
            fraction,
            first,
            second,
            ..
        } => format!(
            "split {:?} {:.2} [{} | {}]",
            axis,
            fraction,
            workspace_host_summary(first),
            workspace_host_summary(second)
        ),
    }
}
