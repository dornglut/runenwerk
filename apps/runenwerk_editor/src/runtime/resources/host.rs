use editor_definition::{
    EditorDefinitionDocument, EditorDefinitionDocumentContent, EditorDefinitionDocumentKind,
    EditorDefinitionId, EditorWorkbenchCompositionDefinition, EditorWorkspaceLayoutDefinition,
    EditorWorkspaceProfileDefinition, editor_definition_has_blocking_diagnostics,
    validate_editor_definition_document,
};
use editor_shell::{
    EDITOR_DESIGN_WORKSPACE_PROFILE_ID, MATERIAL_WORKSPACE_PROFILE_ID,
    workbench_composition_manifest_from_definition,
    workspace_profile_manifests_from_authored_documents,
};
use ui_definition::UiDefinitionDiagnostic;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::material_lab::ensure_default_material_source_document;
use crate::shell::{
    EditorDefinitionActivation, EditorDefinitionActivationPayload,
    EditorDefinitionActivationReport, EditorDefinitionActivationStatus,
    PendingEditorDefinitionActivation, RunenwerkEditorShellState, RunenwerkWorkbenchHost,
    activate_editor_definition_document, is_known_editor_command_key, validate_editor_shortcuts,
};

const SHELL_READABILITY_BUMP: f32 = 1.15;
const SHELL_SCALE_MIN: f32 = 1.0;
const SHELL_SCALE_MAX: f32 = 3.0;

#[derive(runen_ecs::Component, runen_ecs::Resource)]
pub struct EditorHostResource {
    pub app: RunenwerkEditorApp,
    pub shell_state: RunenwerkEditorShellState,
    pub theme: ThemeTokens,
}

impl Default for EditorHostResource {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorHostResource {
    pub fn new() -> Self {
        let mut app = RunenwerkEditorApp::new();
        ensure_default_material_source_document(&mut app);
        let shell_state = RunenwerkEditorShellState::new_with_workspace_profile_registry_and_tool_surface_registry(
            app.workbench_host().workspace_profile_registry(),
            app.workbench_host().tool_surface_registry(),
        )
        .expect("default shell workspace should be compatible with workbench registry");
        Self {
            app,
            shell_state,
            theme: ThemeTokens::default(),
        }
    }

    pub fn material_lab_workbench() -> Self {
        let mut app = RunenwerkEditorApp::new_material_lab_workbench();
        ensure_default_material_source_document(&mut app);
        app.runtime_mut()
            .activate_default_material_graph_document()
            .expect("Material Lab workbench should open a default material graph document");
        let shell_state =
            RunenwerkEditorShellState::new_for_workspace_profile_with_workspace_profile_registry_and_tool_surface_registry(
                MATERIAL_WORKSPACE_PROFILE_ID,
                app.workbench_host().workspace_profile_registry(),
                app.workbench_host().tool_surface_registry(),
            )
            .expect("Material Lab shell workspace should be compatible with workbench registry");
        Self {
            app,
            shell_state,
            theme: ThemeTokens::default(),
        }
    }

    pub fn ui_designer_workbench() -> Self {
        let app = RunenwerkEditorApp::new_ui_designer_workbench();
        let shell_state =
            RunenwerkEditorShellState::new_for_workspace_profile_with_workspace_profile_registry_and_tool_surface_registry(
                EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                app.workbench_host().workspace_profile_registry(),
                app.workbench_host().tool_surface_registry(),
            )
            .expect("UI Designer shell workspace should be compatible with workbench registry");
        Self {
            app,
            shell_state,
            theme: ThemeTokens::default(),
        }
    }

    pub fn apply_theme(&mut self, theme: ThemeTokens) {
        self.theme = theme;
    }

    pub fn apply_pending_editor_definition_activations(&mut self) -> usize {
        let pending = self.app.take_pending_editor_definition_activations();
        let mut activated = 0;
        for request in pending {
            match request.payload.clone() {
                EditorDefinitionActivationPayload::Document(document) => {
                    activated +=
                        self.apply_editor_definition_document_activation(request, document);
                }
                EditorDefinitionActivationPayload::WorkbenchCompositionPackage {
                    composition,
                    profiles,
                    layouts,
                } => {
                    activated += self.apply_workbench_composition_package_activation(
                        request,
                        composition,
                        profiles,
                        layouts,
                    );
                }
            }
        }
        activated
    }

    fn apply_editor_definition_document_activation(
        &mut self,
        request: PendingEditorDefinitionActivation,
        document: EditorDefinitionDocument,
    ) -> usize {
        let mut activated = 0;
        match activate_editor_definition_document(&document, &self.theme) {
            Ok(EditorDefinitionActivation::ThemeChanged(theme)) => {
                self.apply_theme(theme);
                self.app.append_console_line(format!(
                    "[editor-definition] activated live theme {}",
                    document.display_name
                ));
                self.app.record_editor_definition_activation_report(
                    EditorDefinitionActivationReport::from_request(
                        &request,
                        EditorDefinitionActivationStatus::Applied,
                        vec![format!("activated live theme {}", document.display_name)],
                        Vec::new(),
                        false,
                    ),
                );
                activated += 1;
            }
            Ok(EditorDefinitionActivation::NoLiveActivation) => {
                self.app.append_console_line(format!(
                    "[editor-definition] no live activation for {}",
                    document.display_name
                ));
                self.app.record_editor_definition_activation_report(
                    EditorDefinitionActivationReport::from_request(
                        &request,
                        EditorDefinitionActivationStatus::NoLiveActivation,
                        vec![format!("no live activation for {}", document.display_name)],
                        Vec::new(),
                        true,
                    ),
                );
            }
            Ok(EditorDefinitionActivation::UiTemplateCatalogChanged {
                template_id,
                template,
            }) => {
                self.shell_state
                    .active_editor_definitions_mut()
                    .install_template(template);
                self.app.append_console_line(format!(
                    "[editor-definition] activated UI template {template_id}"
                ));
                self.app.record_editor_definition_activation_report(
                    EditorDefinitionActivationReport::from_request(
                        &request,
                        EditorDefinitionActivationStatus::Applied,
                        vec![format!("activated UI template {template_id}")],
                        Vec::new(),
                        false,
                    ),
                );
                activated += 1;
            }
            Ok(EditorDefinitionActivation::EditorBindingsCatalogChanged(bindings)) => {
                match self
                    .shell_state
                    .active_editor_definitions_mut()
                    .install_editor_bindings(bindings)
                {
                    Ok(()) => {
                        self.app.append_console_line(
                            "[editor-definition] activated editor bindings".to_string(),
                        );
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Applied,
                                vec!["activated editor bindings".to_string()],
                                Vec::new(),
                                false,
                            ),
                        );
                        activated += 1;
                    }
                    Err(diagnostics) => {
                        for diagnostic in &diagnostics {
                            self.app.append_console_line(format!(
                                "[editor-definition] editor bindings activation blocked: {}",
                                diagnostic.message
                            ));
                        }
                        self.app
                            .preserve_failed_editor_definition_activation(request.clone());
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Failed,
                                vec!["editor bindings activation blocked".to_string()],
                                diagnostics,
                                true,
                            ),
                        );
                    }
                }
            }
            Ok(EditorDefinitionActivation::MenuCatalogChanged { menu_id, menu }) => {
                self.shell_state
                    .active_editor_definitions_mut()
                    .install_menu(menu);
                self.app
                    .append_console_line(format!("[editor-definition] activated menu {menu_id}"));
                self.app.record_editor_definition_activation_report(
                    EditorDefinitionActivationReport::from_request(
                        &request,
                        EditorDefinitionActivationStatus::Applied,
                        vec![format!("activated menu {menu_id}")],
                        Vec::new(),
                        false,
                    ),
                );
                activated += 1;
            }
            Ok(EditorDefinitionActivation::ShortcutCatalogChanged {
                shortcut_set_id,
                shortcuts,
            }) => {
                match self
                    .shell_state
                    .active_editor_definitions_mut()
                    .install_shortcuts(shortcuts, validate_editor_shortcuts)
                {
                    Ok(()) => {
                        self.app.append_console_line(format!(
                            "[editor-definition] activated shortcut set {shortcut_set_id}"
                        ));
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Applied,
                                vec![format!("activated shortcut set {shortcut_set_id}")],
                                Vec::new(),
                                false,
                            ),
                        );
                        activated += 1;
                    }
                    Err(diagnostics) => {
                        for diagnostic in &diagnostics {
                            self.app.append_console_line(format!(
                                "[editor-definition] shortcut activation blocked: {}",
                                diagnostic.message
                            ));
                        }
                        self.app
                            .preserve_failed_editor_definition_activation(request.clone());
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Failed,
                                vec!["shortcut activation blocked".to_string()],
                                diagnostics,
                                true,
                            ),
                        );
                    }
                }
            }
            Ok(EditorDefinitionActivation::CommandBindingCatalogChanged {
                command_binding_set_id,
                command_bindings,
            }) => {
                match self
                    .shell_state
                    .active_editor_definitions_mut()
                    .install_command_bindings(command_bindings, is_known_editor_command_key)
                {
                    Ok(()) => {
                        self.app.append_console_line(format!(
                                "[editor-definition] activated command binding set {command_binding_set_id}"
                            ));
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Applied,
                                vec![format!(
                                    "activated command binding set {command_binding_set_id}"
                                )],
                                Vec::new(),
                                false,
                            ),
                        );
                        activated += 1;
                    }
                    Err(diagnostics) => {
                        for diagnostic in &diagnostics {
                            self.app.append_console_line(format!(
                                "[editor-definition] command binding activation blocked: {}",
                                diagnostic.message
                            ));
                        }
                        self.app
                            .preserve_failed_editor_definition_activation(request.clone());
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Failed,
                                vec!["command binding activation blocked".to_string()],
                                diagnostics,
                                true,
                            ),
                        );
                    }
                }
            }
            Ok(EditorDefinitionActivation::PanelRegistryCatalogChanged {
                registry_id,
                registry,
            }) => {
                let composition = self.shell_state.composition_runtime().clone();
                match self
                    .shell_state
                    .active_editor_definitions_mut()
                    .install_panel_registry(registry, &composition)
                {
                    Ok(()) => {
                        self.app.append_console_line(format!(
                            "[editor-definition] activated panel registry {registry_id}"
                        ));
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Applied,
                                vec![format!("activated panel registry {registry_id}")],
                                Vec::new(),
                                false,
                            ),
                        );
                        activated += 1;
                    }
                    Err(diagnostic) => {
                        self.app.append_console_line(format!(
                            "[editor-definition] panel registry activation blocked: {}",
                            diagnostic.message
                        ));
                        self.app
                            .preserve_failed_editor_definition_activation(request.clone());
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Failed,
                                vec!["panel registry activation blocked".to_string()],
                                vec![diagnostic],
                                true,
                            ),
                        );
                    }
                }
            }
            Ok(EditorDefinitionActivation::ToolSurfaceRegistryCatalogChanged {
                registry_id,
                registry,
            }) => {
                let composition = self.shell_state.composition_runtime().clone();
                match self
                    .shell_state
                    .active_editor_definitions_mut()
                    .install_tool_surface_registry(registry, &composition)
                {
                    Ok(()) => {
                        self.app.append_console_line(format!(
                            "[editor-definition] activated tool-surface registry {registry_id}"
                        ));
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Applied,
                                vec![format!("activated tool-surface registry {registry_id}")],
                                Vec::new(),
                                false,
                            ),
                        );
                        activated += 1;
                    }
                    Err(diagnostic) => {
                        self.app.append_console_line(format!(
                            "[editor-definition] tool-surface registry activation blocked: {}",
                            diagnostic.message
                        ));
                        self.app
                            .preserve_failed_editor_definition_activation(request.clone());
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Failed,
                                vec!["tool-surface registry activation blocked".to_string()],
                                vec![diagnostic],
                                true,
                            ),
                        );
                    }
                }
            }
            Ok(EditorDefinitionActivation::WorkspaceLayoutChanged {
                workspace_id,
                layout,
            }) => {
                let result = editor_shell::form_editor_profile_composition(
                    self.shell_state.active_workspace_profile_id(),
                    &layout,
                    self.app.workbench_host().tool_surface_registry(),
                )
                .map_err(|error| format!("composition formation failed: {error:?}"))
                .and_then(|runtime| {
                    self.shell_state
                        .install_composition_runtime(runtime)
                        .map_err(|error| format!("composition install failed: {error:?}"))
                });
                match result {
                    Ok(()) => {
                        self.app.append_console_line(format!(
                            "[editor-definition] activated composition layout {workspace_id}"
                        ));
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Applied,
                                vec![format!("activated composition layout {workspace_id}")],
                                Vec::new(),
                                false,
                            ),
                        );
                        activated += 1;
                    }
                    Err(message) => {
                        let diagnostic = UiDefinitionDiagnostic::error(
                            editor_shell::EditorCompositionDiagnosticCode::LayoutActivationFailed
                                .as_str(),
                            message,
                        );
                        self.app
                            .preserve_failed_editor_definition_activation(request.clone());
                        self.app.record_editor_definition_activation_report(
                            EditorDefinitionActivationReport::from_request(
                                &request,
                                EditorDefinitionActivationStatus::Failed,
                                vec!["composition layout activation failed".to_string()],
                                vec![diagnostic],
                                true,
                            ),
                        );
                    }
                }
            }
            Err(diagnostics) => {
                for diagnostic in &diagnostics {
                    self.app.append_console_line(format!(
                        "[editor-definition] activation blocked: {}",
                        diagnostic.message
                    ));
                }
                self.app
                    .preserve_failed_editor_definition_activation(request.clone());
                self.app.record_editor_definition_activation_report(
                    EditorDefinitionActivationReport::from_request(
                        &request,
                        EditorDefinitionActivationStatus::Failed,
                        vec!["activation blocked by definition validation".to_string()],
                        diagnostics,
                        true,
                    ),
                );
            }
        }
        activated
    }

    fn apply_workbench_composition_package_activation(
        &mut self,
        request: PendingEditorDefinitionActivation,
        composition: EditorWorkbenchCompositionDefinition,
        profiles: Vec<EditorWorkspaceProfileDefinition>,
        layouts: Vec<EditorWorkspaceLayoutDefinition>,
    ) -> usize {
        let composition_id = composition.id.clone();
        let composition_label = composition.label.clone();
        let host = match authored_workbench_host_from_package(&composition, &profiles, &layouts) {
            Ok(host) => host,
            Err(diagnostic) => {
                return self.record_failed_workbench_package_activation(
                    request,
                    "workbench composition package activation blocked".to_string(),
                    diagnostic,
                );
            }
        };
        let default_profile_ref = host.default_workspace_profile_ref().clone();
        match self
            .shell_state
            .activate_workspace_profile_ref_with_registry(
                &default_profile_ref,
                host.workspace_profile_registry(),
                host.tool_surface_registry(),
            ) {
            Ok(_) => {
                self.app.replace_workbench_host(host);
                self.shell_state
                    .self_authoring_mut()
                    .record_applied_workbench_composition_payload(&request.payload);
                self.app
                    .prune_surface_sessions_for_composition(self.shell_state.composition_runtime());
                self.app.append_console_line(format!(
                    "[editor-definition] activated workbench composition package {composition_id}"
                ));
                self.app.record_editor_definition_activation_report(
                    EditorDefinitionActivationReport::from_request(
                        &request,
                        EditorDefinitionActivationStatus::Applied,
                        vec![format!(
                            "activated workbench composition package {composition_label}"
                        )],
                        Vec::new(),
                        false,
                    ),
                );
                1
            }
            Err(error) => {
                let diagnostic = UiDefinitionDiagnostic::error(
                    "editor.definition.activation.workbench_shell_state_blocked",
                    format!("workbench shell state activation blocked: {error}"),
                );
                self.record_failed_workbench_package_activation(
                    request,
                    "workbench shell state activation blocked".to_string(),
                    diagnostic,
                )
            }
        }
    }

    fn record_failed_workbench_package_activation(
        &mut self,
        request: PendingEditorDefinitionActivation,
        summary: String,
        diagnostic: UiDefinitionDiagnostic,
    ) -> usize {
        self.app.append_console_line(format!(
            "[editor-definition] {}: {}",
            summary, diagnostic.message
        ));
        self.app
            .preserve_failed_editor_definition_activation(request.clone());
        self.app.record_editor_definition_activation_report(
            EditorDefinitionActivationReport::from_request(
                &request,
                EditorDefinitionActivationStatus::Failed,
                vec![summary],
                vec![diagnostic],
                true,
            ),
        );
        0
    }
}

fn authored_workbench_host_from_package(
    composition: &EditorWorkbenchCompositionDefinition,
    profiles: &[EditorWorkspaceProfileDefinition],
    layouts: &[EditorWorkspaceLayoutDefinition],
) -> Result<RunenwerkWorkbenchHost, UiDefinitionDiagnostic> {
    let composition_document = authored_workbench_composition_document(composition);
    let diagnostics = validate_editor_definition_document(&composition_document);
    if editor_definition_has_blocking_diagnostics(&diagnostics) {
        return Err(UiDefinitionDiagnostic::error(
            "editor.definition.activation.workbench_composition_invalid",
            format!("workbench composition document is invalid: {diagnostics:?}"),
        ));
    }

    let package_documents = authored_workbench_package_documents(profiles, layouts);
    for document in &package_documents {
        let diagnostics = validate_editor_definition_document(document);
        if editor_definition_has_blocking_diagnostics(&diagnostics) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.definition.activation.workbench_package_document_invalid",
                format!(
                    "workbench package document `{}` is invalid: {diagnostics:?}",
                    document.id.as_str()
                ),
            ));
        }
    }

    let composition_manifest = workbench_composition_manifest_from_definition(composition)
        .map_err(|error| {
            UiDefinitionDiagnostic::error(
                "editor.definition.activation.workbench_composition_manifest_blocked",
                format!("workbench composition manifest conversion blocked: {error}"),
            )
        })?;
    let profile_manifests = workspace_profile_manifests_from_authored_documents(
        composition.profile_refs.iter().map(String::as_str),
        package_documents.iter(),
    )
    .map_err(|error| {
        UiDefinitionDiagnostic::error(
            "editor.definition.activation.workbench_profile_manifest_blocked",
            format!("workbench profile manifest conversion blocked: {error}"),
        )
    })?;
    RunenwerkWorkbenchHost::authored(composition_manifest, profile_manifests).map_err(|error| {
        UiDefinitionDiagnostic::error(
            "editor.definition.activation.workbench_host_blocked",
            format!("workbench host activation blocked: {error}"),
        )
    })
}

fn authored_workbench_composition_document(
    composition: &EditorWorkbenchCompositionDefinition,
) -> EditorDefinitionDocument {
    EditorDefinitionDocument::current(
        EditorDefinitionId::from(composition.id.as_str()),
        composition.label.clone(),
        EditorDefinitionDocumentKind::WorkbenchComposition,
        EditorDefinitionDocumentContent::WorkbenchComposition(composition.clone()),
    )
}

fn authored_workbench_package_documents(
    profiles: &[EditorWorkspaceProfileDefinition],
    layouts: &[EditorWorkspaceLayoutDefinition],
) -> Vec<EditorDefinitionDocument> {
    profiles
        .iter()
        .map(|profile| {
            EditorDefinitionDocument::current(
                EditorDefinitionId::from(profile.id.as_str()),
                profile.label.clone(),
                EditorDefinitionDocumentKind::WorkspaceDefinition,
                EditorDefinitionDocumentContent::WorkspaceProfile(profile.clone()),
            )
        })
        .chain(layouts.iter().map(|layout| {
            EditorDefinitionDocument::current(
                EditorDefinitionId::from(layout.id.as_str()),
                layout.label.clone(),
                EditorDefinitionDocumentKind::WorkspaceDefinition,
                EditorDefinitionDocumentContent::WorkspaceLayout(layout.clone()),
            )
        }))
        .collect()
}

pub fn effective_shell_scale(scale_factor: f64) -> f32 {
    (scale_factor as f32).clamp(SHELL_SCALE_MIN, SHELL_SCALE_MAX) * SHELL_READABILITY_BUMP
}

pub fn scaled_shell_theme(theme: &ThemeTokens, scale_factor: f64) -> ThemeTokens {
    theme.scaled_by(effective_shell_scale(scale_factor))
}
