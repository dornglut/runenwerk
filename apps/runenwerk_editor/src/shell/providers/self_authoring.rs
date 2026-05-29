use super::*;

pub(super) struct SelfAuthoringProvider;

impl EditorSurfaceProvider for SelfAuthoringProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SELF_AUTHORING_PROVIDER_ID,
            "UI Designer Workbench",
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
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::InsertRecipe {
                recipe_id,
            }) => ShellCommand::InsertSelectedEditorDefinitionRecipe { recipe_id },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SetRecipeCatalogFilter { query },
            ) => ShellCommand::SetEditorDefinitionRecipeCatalogFilter { query },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::CaptureScenarioEvidence,
            ) => ShellCommand::CaptureUiDesignerScenarioEvidence,
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
        ToolSurfaceKind::UiCanvas => "UI Designer Workbench",
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
        ToolSurfaceKind::UiCanvas => EditorLabSurfaceViewModel::UiDesignerWorkbench(
            ui_designer_workbench_view_model(context),
        ),
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

fn ui_designer_workbench_view_model(
    context: &SurfaceProviderBuildContext<'_>,
) -> UiDesignerWorkbenchViewModel {
    let state = context.shell_state.self_authoring();
    let panes = vec![
        workbench_pane_from_canvas(canvas_preview_view_model(context)),
        workbench_pane_from_inspector(
            UiDesignerWorkbenchPaneKind::Hierarchy,
            ui_hierarchy_view_model(context),
        ),
        workbench_pane_from_inspector(
            UiDesignerWorkbenchPaneKind::Inspector,
            style_inspector_view_model(context),
        ),
        workbench_pane_from_inspector(
            UiDesignerWorkbenchPaneKind::Properties,
            theme_editor_view_model(context),
        ),
        token_recipe_preview_pane(state),
        workbench_pane_from_review(
            UiDesignerWorkbenchPaneKind::BindingPreview,
            bindings_review_view_model(context),
        ),
        scenario_matrix_pane(),
        readiness_pane(),
        workbench_pane_from_diagnostics(diagnostics_view_model(context)),
        native_evidence_pane(context),
    ];

    UiDesignerWorkbenchViewModel::new(
        "Standalone UI Designer Workbench",
        UI_DESIGNER_WORKBENCH_TARGET_PROFILE,
        panes,
    )
    .with_selected_document(selected_document_label(state))
    .with_readiness(workbench_readiness(context))
    .with_actions(
        operation_history_actions(state)
            .into_iter()
            .chain(apply_review_actions(state)),
    )
}

fn workbench_pane_from_canvas(
    model: EditorLabCanvasPreviewViewModel,
) -> UiDesignerWorkbenchPaneViewModel {
    let mut summary_lines = Vec::new();
    summary_lines.push(if model.retained_preview_available {
        "retained preview available".to_string()
    } else {
        "retained preview unavailable".to_string()
    });
    if let Some(selected) = model.selected_document {
        summary_lines.push(format!("document: {selected}"));
    }
    summary_lines.extend(model.status_lines);
    UiDesignerWorkbenchPaneViewModel::new(UiDesignerWorkbenchPaneKind::Canvas, model.title)
        .with_summary_lines(summary_lines)
        .with_actions(model.actions)
}

fn workbench_pane_from_inspector(
    kind: UiDesignerWorkbenchPaneKind,
    model: EditorLabInspectorViewModel,
) -> UiDesignerWorkbenchPaneViewModel {
    let mut summary_lines = Vec::new();
    if let Some(selected) = model.selected_document {
        summary_lines.push(format!("document: {selected}"));
    }
    summary_lines.extend(
        model
            .fields
            .into_iter()
            .map(|field| format!("{}: {}", field.label, field.value)),
    );
    UiDesignerWorkbenchPaneViewModel::new(kind, model.title)
        .with_summary_lines(summary_lines)
        .with_actions(model.actions)
        .with_diagnostics(model.diagnostics)
}

fn workbench_pane_from_review(
    kind: UiDesignerWorkbenchPaneKind,
    model: EditorLabReviewViewModel,
) -> UiDesignerWorkbenchPaneViewModel {
    let mut summary_lines = Vec::new();
    if let Some(selected) = model.selected_document {
        summary_lines.push(format!("document: {selected}"));
    }
    summary_lines.extend(model.summary_lines);
    UiDesignerWorkbenchPaneViewModel::new(kind, model.title)
        .with_summary_lines(summary_lines)
        .with_actions(model.actions)
        .with_diagnostics(model.diagnostics)
}

fn workbench_pane_from_diagnostics(
    model: EditorLabDiagnosticsViewModel,
) -> UiDesignerWorkbenchPaneViewModel {
    let mut summary_lines = Vec::new();
    if let Some(selected) = model.selected_document {
        summary_lines.push(format!("document: {selected}"));
    }
    summary_lines.push(format!("blocking diagnostics: {}", model.diagnostics.len()));
    UiDesignerWorkbenchPaneViewModel::new(UiDesignerWorkbenchPaneKind::Diagnostics, model.title)
        .with_summary_lines(summary_lines)
        .with_actions(model.actions)
        .with_diagnostics(model.diagnostics)
}

fn token_recipe_preview_pane(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> UiDesignerWorkbenchPaneViewModel {
    let library = editor_shell::editor_design_system_recipe_library();
    let catalog_items = component_catalog_items(state, &library);
    let enabled_insertions = catalog_items
        .iter()
        .filter(|item| item.action.enabled)
        .count();
    let mut summary_lines = Vec::new();
    if let Some(source_version) = ui_designer_source_version_label(state) {
        summary_lines.push(format!("source version: {source_version}"));
    }
    summary_lines.push("catalog target profile: editor.workbench".to_string());
    summary_lines.push(format!(
        "recipe declarations: {} enabled insertions: {}",
        library.declarations.len(),
        enabled_insertions
    ));
    if let Some(document) = state.selected_document() {
        summary_lines.push(format!("document kind: {:?}", document.kind));
        match &document.content {
            editor_definition::EditorDefinitionDocumentContent::Theme(theme) => {
                summary_lines.push(format!("theme: {}", theme.label));
                summary_lines.push(format!("tokens: {}", theme.colors.len()));
            }
            editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) => {
                summary_lines.push(format!("template: {}", template.id.as_str()));
                summary_lines
                    .push("recipe catalog source: editor.product.design_system".to_string());
                summary_lines.push("target profile: editor.workbench".to_string());
            }
            _ => summary_lines.push(
                "select a UI template or theme definition to inspect token/recipe coverage"
                    .to_string(),
            ),
        }
    } else {
        summary_lines.push("no definition document is selected".to_string());
    }
    UiDesignerWorkbenchPaneViewModel::new(
        UiDesignerWorkbenchPaneKind::TokenRecipePreview,
        "Component Catalog",
    )
    .with_summary_lines(summary_lines)
    .with_filter_field(EditorLabTextFieldViewModel::new(
        "Filter component catalog",
        state.recipe_catalog_filter().to_string(),
        "Filter recipes by name, category, target, token, or state",
        EditorDefinitionSurfaceAction::SetRecipeCatalogFilter {
            query: state.recipe_catalog_filter().to_string(),
        },
    ))
    .with_catalog_items(catalog_items)
}

fn component_catalog_items(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
    library: &ui_definition::UiRecipeLibrary,
) -> Vec<EditorLabCatalogItemViewModel> {
    let selected_template = state.selected_document().is_some_and(|document| {
        matches!(
            &document.content,
            editor_definition::EditorDefinitionDocumentContent::UiTemplate(_)
        )
    });
    let filter = state.recipe_catalog_filter().trim().to_lowercase();
    library
        .declarations
        .iter()
        .map(|declaration| recipe_catalog_item(declaration, selected_template))
        .filter(|item| catalog_item_matches_filter(item, &filter))
        .collect()
}

fn recipe_catalog_item(
    declaration: &ui_definition::UiRecipeDeclaration,
    selected_template: bool,
) -> EditorLabCatalogItemViewModel {
    let target_profile =
        ui_definition::UiRecipeTargetProfileId::from(UI_DESIGNER_WORKBENCH_TARGET_PROFILE);
    let expansion = ui_definition::expand_ui_recipe(
        &ui_definition::UiRecipeLibrary {
            declarations: vec![declaration.clone()],
        },
        &ui_definition::UiRecipeExpansionRequest::activate(
            declaration.id.clone(),
            target_profile.clone(),
        ),
    );
    let mut action = EditorLabActionViewModel::enabled(
        format!("Insert {}", declaration.label),
        EditorDefinitionSurfaceAction::InsertRecipe {
            recipe_id: declaration.id.as_str().to_string(),
        },
    );
    if !selected_template {
        action = action.disabled("requires a selected UI template document");
    } else if expansion.has_errors() {
        action = action.disabled(recipe_catalog_disabled_reason(
            &expansion.as_definition_diagnostics(),
        ));
    }

    let readiness = recipe_readiness(declaration, &target_profile, expansion.has_errors());
    let searchable_text = format!(
        "{} {} {} {} {} {} {} {}",
        declaration.id.as_str(),
        declaration.label,
        declaration.category,
        target_profile.as_str(),
        declaration.source_package.as_str(),
        recipe_target_compatibility(declaration, &target_profile),
        recipe_token_requirements(declaration).join(" "),
        declaration
            .state_variants
            .iter()
            .map(|state| state.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );

    EditorLabCatalogItemViewModel::new(
        EditorLabCatalogItemDetails {
            recipe_id: declaration.id.as_str().to_string(),
            label: declaration.label.clone(),
            category: declaration.category.clone(),
            target_profile: target_profile.as_str().to_string(),
            target_compatibility: recipe_target_compatibility(declaration, &target_profile),
            source_package: declaration.source_package.as_str().to_string(),
            slot_compatibility: recipe_slot_compatibility(declaration),
        },
        action,
    )
    .with_required_token_families(recipe_token_requirements(declaration))
    .with_supported_states(
        declaration
            .state_variants
            .iter()
            .map(|state| state.as_str().to_string()),
    )
    .with_accessibility_requirements(recipe_accessibility_requirements(declaration))
    .with_readiness(readiness)
    .with_searchable_text(searchable_text)
}

fn catalog_item_matches_filter(item: &EditorLabCatalogItemViewModel, filter: &str) -> bool {
    filter.is_empty() || item.searchable_text.contains(filter)
}

fn recipe_target_compatibility(
    declaration: &ui_definition::UiRecipeDeclaration,
    target_profile: &ui_definition::UiRecipeTargetProfileId,
) -> String {
    if declaration.target_profiles.is_empty() {
        return "compatible: all target profiles".to_string();
    }
    if declaration.target_profiles.contains(target_profile) {
        return format!("compatible: {}", target_profile.as_str());
    }
    format!(
        "incompatible: requires {}",
        declaration
            .target_profiles
            .iter()
            .map(|profile| profile.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn recipe_slot_compatibility(declaration: &ui_definition::UiRecipeDeclaration) -> String {
    if declaration.slots.is_empty() {
        return "direct insert into selected container".to_string();
    }
    declaration
        .slots
        .iter()
        .map(|slot| format!("{} accepts {:?}", slot.id.as_str(), slot.accepted_kinds))
        .collect::<Vec<_>>()
        .join("; ")
}

fn recipe_token_requirements(declaration: &ui_definition::UiRecipeDeclaration) -> Vec<String> {
    declaration
        .token_requirements
        .iter()
        .map(|requirement| {
            requirement
                .token
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| format!("{:?}", requirement.family))
        })
        .collect()
}

fn recipe_accessibility_requirements(
    declaration: &ui_definition::UiRecipeDeclaration,
) -> Vec<String> {
    let Some(accessibility) = &declaration.accessibility else {
        return Vec::new();
    };
    let mut requirements = vec![format!("role:{:?}", accessibility.role)];
    requirements.extend(accessibility.required_semantics.iter().cloned());
    requirements
}

fn recipe_readiness(
    declaration: &ui_definition::UiRecipeDeclaration,
    target_profile: &ui_definition::UiRecipeTargetProfileId,
    has_errors: bool,
) -> ToolSurfaceReadiness {
    if !declaration.target_profiles.is_empty()
        && !declaration.target_profiles.contains(target_profile)
    {
        ToolSurfaceReadiness::HiddenUntilProductized
    } else if declaration.preview_only || has_errors {
        ToolSurfaceReadiness::Diagnostic
    } else {
        ToolSurfaceReadiness::Product
    }
}

fn recipe_catalog_disabled_reason(diagnostics: &[ui_definition::UiDefinitionDiagnostic]) -> String {
    diagnostics
        .first()
        .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
        .unwrap_or_else(|| "recipe expansion was rejected".to_string())
}

fn scenario_matrix_pane() -> UiDesignerWorkbenchPaneViewModel {
    UiDesignerWorkbenchPaneViewModel::new(
        UiDesignerWorkbenchPaneKind::ScenarioMatrix,
        "Scenario Matrix",
    )
    .with_summary_lines([
        "density: comfortable, compact".to_string(),
        "viewport: narrow, standard, wide".to_string(),
        "input: pointer, keyboard".to_string(),
        "states: default, focused, disabled, overflow".to_string(),
    ])
}

fn readiness_pane() -> UiDesignerWorkbenchPaneViewModel {
    UiDesignerWorkbenchPaneViewModel::new(UiDesignerWorkbenchPaneKind::Readiness, "Readiness")
        .with_summary_lines([
            "surface readiness: product for standalone workbench scenario".to_string(),
            "visible-widget scan: required".to_string(),
            "native proof: screenshot or platform-impossible report".to_string(),
        ])
}

fn native_evidence_pane(
    context: &SurfaceProviderBuildContext<'_>,
) -> UiDesignerWorkbenchPaneViewModel {
    let mut summary_lines = vec![
        "retained UI debug artifact: required".to_string(),
        "focus traversal report: required".to_string(),
        "accessibility report: required".to_string(),
        "timing report: required".to_string(),
    ];

    for packet in context
        .shell_state
        .self_authoring()
        .last_scenario_evidence_packets()
    {
        let source_freshness = if context
            .shell_state
            .self_authoring()
            .selected_source_revision()
            .as_ref()
            == Some(packet.source_revision())
        {
            "Fresh"
        } else {
            "Stale"
        };
        summary_lines.push(format!(
            "pm005 evidence packet: {} kind={} target={} document={} source={} capture={:?} freshness={}",
            packet.scenario_id(),
            if packet.is_runtime_product() { "runtime-product" } else { "descriptor-compatibility" },
            packet.target_profile(),
            packet.document_id(),
            packet.source_version(),
            packet.capture_mode(),
            source_freshness
        ));
        summary_lines.push(format!(
            "pm005 diagnostics snapshot: {} diagnostics",
            packet.diagnostics().len()
        ));
        for binding in packet.fixture_bindings() {
            summary_lines.push(format!(
                "pm005 fixture binding: {} {} read_only={} compatibility={:?}",
                binding.fixture_id, binding.binding_id, binding.read_only, binding.compatibility
            ));
        }
        for intent in packet.intent_descriptors() {
            summary_lines.push(format!(
                "pm005 intent descriptor: {} validated={} runtime_command={}",
                intent.intent_id, intent.validated, intent.executes_runtime_command
            ));
        }
        for baseline in packet.performance_baselines() {
            summary_lines.push(format!(
                "pm005 baseline: {:?} {}us samples={} provenance={:?}",
                baseline.kind, baseline.elapsed_micros, baseline.sample_count, baseline.provenance
            ));
        }
        for unsupported in packet.unsupported_checks() {
            summary_lines.push(format!(
                "pm005 unsupported check: {} - {}",
                unsupported.check, unsupported.reason
            ));
        }
    }

    UiDesignerWorkbenchPaneViewModel::new(
        UiDesignerWorkbenchPaneKind::NativeEvidencePreview,
        "Native Evidence",
    )
    .with_summary_lines(summary_lines)
    .with_actions([EditorLabActionViewModel::enabled(
        "Capture scenario evidence",
        EditorDefinitionSurfaceAction::CaptureScenarioEvidence,
    )])
}

fn workbench_readiness(
    context: &SurfaceProviderBuildContext<'_>,
) -> Vec<UiDesignerWorkbenchReadinessViewModel> {
    let state = context.shell_state.self_authoring();
    let selected_template = state.selected_document().is_some_and(|document| {
        matches!(
            &document.content,
            editor_definition::EditorDefinitionDocumentContent::UiTemplate(_)
        )
    });
    let selected_node = state.selected_ui_node_id().is_some();
    let history = state.operation_history_snapshot();
    let has_operation_diff = state
        .last_operation_report()
        .and_then(|report| report.diff.as_ref())
        .is_some_and(|diff| !diff.changes.is_empty());
    let has_apply_review = state.last_apply_review().is_some();
    let has_last_applied_snapshot = state.selected_last_applied_document().is_some();
    let has_rollback_record = state.last_rollback_record().is_some();
    let scenario_packets = state.last_scenario_evidence_packets();
    let current_source_revision = state.selected_source_revision();
    let has_editor_workbench_runtime_packet = scenario_packets.iter().any(|packet| {
        packet.target_profile() == "editor.workbench"
            && packet.validate_runtime_product_evidence().is_ok()
            && current_source_revision.as_ref() == Some(packet.source_revision())
    });
    let has_pm005_baselines = scenario_packets.iter().any(|packet| {
        packet.target_profile() == "editor.workbench"
            && packet.validate_runtime_product_evidence().is_ok()
            && packet.performance_baselines().len()
                >= crate::shell::editor_lab_evidence::UI_DESIGNER_SCENARIO_BASELINE_KINDS.len()
            && packet.performance_baselines().iter().all(|baseline| {
                baseline.provenance
                    == crate::shell::editor_lab_evidence::EditorLabMeasurementProvenance::ProductPath
            })
    });
    let has_game_runtime_descriptor_packet = scenario_packets.iter().any(|packet| {
        packet.target_profile() == "game.runtime"
            && !packet.is_runtime_product()
            && packet.validate_scenario_evidence().is_ok()
            && current_source_revision.as_ref() == Some(packet.source_revision())
            && !packet.fixture_bindings().is_empty()
            && packet
                .fixture_bindings()
                .iter()
                .all(|binding| binding.read_only)
            && !packet.intent_descriptors().is_empty()
            && packet
                .intent_descriptors()
                .iter()
                .all(|intent| intent.validated && !intent.executes_runtime_command)
    });
    let recipe_catalog_items =
        component_catalog_items(state, &editor_shell::editor_design_system_recipe_library());
    let has_recipe_insertion = selected_template
        && recipe_catalog_items.iter().any(|item| {
            matches!(
                &item.action.action,
                EditorDefinitionSurfaceAction::InsertRecipe { .. }
            ) && item.action.enabled
        });

    vec![
        UiDesignerWorkbenchReadinessViewModel::new(
            "typed_pane_chain",
            UiDesignerWorkbenchReadinessStatus::Passed,
            "canvas, hierarchy, inspector, properties, preview, scenario, diagnostics, and evidence panes are composed as one workbench",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "operation_intent_roundtrip",
            if selected_template && selected_node {
                UiDesignerWorkbenchReadinessStatus::Passed
            } else {
                UiDesignerWorkbenchReadinessStatus::Warning
            },
            "normal edits use EditorDefinition route actions and EditorLabOperation reducers",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "ux_lab_native_evidence",
            UiDesignerWorkbenchReadinessStatus::Passed,
            "PM-EDITOR-UX-004 UX Lab manifest requires workbench, accessibility, focus, timing, and native proof",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "legacy_self_authoring_bypass",
            UiDesignerWorkbenchReadinessStatus::Passed,
            "UI canvas stable key renders the standalone workbench instead of legacy text/action control panels",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "pm003_recipe_catalog_insertion",
            if has_recipe_insertion {
                UiDesignerWorkbenchReadinessStatus::Passed
            } else {
                UiDesignerWorkbenchReadinessStatus::Warning
            },
            "component catalog rows are projected from UiRecipeDeclaration metadata and route compatible rows through InsertRecipe",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "pm004_product_catalog_rows",
            UiDesignerWorkbenchReadinessStatus::Passed,
            "component catalog rows expose target compatibility, source package, slots, tokens, states, accessibility, readiness, and disabled reasons",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "pm004_source_version_selection_parity",
            if selected_template && selected_node {
                UiDesignerWorkbenchReadinessStatus::Passed
            } else {
                UiDesignerWorkbenchReadinessStatus::Warning
            },
            "hierarchy, canvas, inspector, diagnostics, and diff/review panes share the selected authored id and source version label",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "pm005_operation_diff_history",
            if has_operation_diff && (history.can_undo || history.can_redo) {
                UiDesignerWorkbenchReadinessStatus::Passed
            } else {
                UiDesignerWorkbenchReadinessStatus::Warning
            },
            "typed operations produce deterministic diffs and session undo/redo history without entering source truth",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "pm005_apply_reload_rollback",
            if has_apply_review && has_last_applied_snapshot && has_rollback_record {
                UiDesignerWorkbenchReadinessStatus::Passed
            } else {
                UiDesignerWorkbenchReadinessStatus::Warning
            },
            "apply review, last-applied reload, and rollback expose typed recovery state for the selected authored package",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "pm005_runtime_product_evidence_packet",
            if has_editor_workbench_runtime_packet {
                UiDesignerWorkbenchReadinessStatus::Passed
            } else {
                UiDesignerWorkbenchReadinessStatus::Warning
            },
            "editor.workbench scenario evidence must be a fresh runtime product packet with source revision, product artifacts, and product-path provenance",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "pm005_performance_baselines",
            if has_pm005_baselines {
                UiDesignerWorkbenchReadinessStatus::Passed
            } else {
                UiDesignerWorkbenchReadinessStatus::Warning
            },
            "explicit runtime capture records resize, canvas interaction, catalog, diagnostics, and frame-build baselines through product-path measurements",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "pm005_game_runtime_descriptor_only",
            if has_game_runtime_descriptor_packet {
                UiDesignerWorkbenchReadinessStatus::Passed
            } else {
                UiDesignerWorkbenchReadinessStatus::Warning
            },
            "game.runtime evidence remains descriptor compatibility only until PT-GAME-RUNTIME-UI provides a real runtime product path",
        ),
    ]
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
        if let Some(source_version) = ui_designer_source_version_label(state) {
            fields.push(field("Source version", source_version));
        }
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
        if let Some(source_version) = ui_designer_source_version_label(state) {
            status_lines.push(format!("source version: {source_version}"));
        }
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
        if let Some(diff) = &report.diff {
            for change in diff.changes.iter().take(3) {
                status_lines.push(format!(
                    "operation diff: {:?} {} {:?} -> {:?}",
                    change.family, change.path, change.before, change.after
                ));
            }
        }
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
    if let Some(review) = state.last_apply_review() {
        status_lines.push(format!(
            "apply review: {} {:?} diffs={} diagnostics={}",
            review.display_name,
            review.status,
            review.diff_rows.len(),
            review.diagnostics.len()
        ));
        for row in review.diff_rows.iter().take(2) {
            status_lines.push(format!(
                "apply diff: {:?}/{:?} {} {:?} -> {:?}: {}",
                row.family, row.kind, row.path, row.before, row.after, row.summary
            ));
        }
    }
    if let Some(snapshot) = state.selected_last_applied_document() {
        status_lines.push(format!(
            "last applied snapshot: {} {:?}",
            snapshot.display_name, snapshot.lifecycle_state
        ));
    }
    if let Some(record) = state.last_rollback_record() {
        status_lines.push(format!(
            "rollback: {} {:?}",
            record.display_name, record.status
        ));
    }
    for packet in state.last_scenario_evidence_packets() {
        let source_freshness =
            if state.selected_source_revision().as_ref() == Some(packet.source_revision()) {
                "Fresh"
            } else {
                "Stale"
            };
        status_lines.push(format!(
            "evidence packet: {} kind={} target={} document={} source={} capture={:?} freshness={}",
            packet.scenario_id(),
            if packet.is_runtime_product() {
                "runtime-product"
            } else {
                "descriptor-compatibility"
            },
            packet.target_profile(),
            packet.document_id(),
            packet.source_version(),
            packet.capture_mode(),
            source_freshness
        ));
        status_lines.push(format!(
            "evidence diagnostics: {} artifacts={} unsupported={}",
            packet.diagnostics().len(),
            packet.artifacts().len(),
            packet.unsupported_checks().len()
        ));
        for artifact in packet.artifacts().iter().take(2) {
            status_lines.push(format!(
                "evidence artifact: {:?} {} bytes={} provenance={:?}",
                artifact.kind, artifact.path, artifact.bytes, artifact.provenance
            ));
        }
        for unsupported in packet.unsupported_checks().iter().take(2) {
            status_lines.push(format!(
                "unsupported check: {} - {}",
                unsupported.check, unsupported.reason
            ));
        }
        for binding in packet.fixture_bindings().iter().take(2) {
            status_lines.push(format!(
                "fixture binding: {} {} read_only={} compatibility={:?}",
                binding.fixture_id, binding.binding_id, binding.read_only, binding.compatibility
            ));
        }
        for intent in packet.intent_descriptors().iter().take(2) {
            status_lines.push(format!(
                "intent descriptor: {} validated={} runtime_command={}",
                intent.intent_id, intent.validated, intent.executes_runtime_command
            ));
        }
        for baseline in packet.performance_baselines().iter().take(5) {
            status_lines.push(format!(
                "baseline: {:?} {}us samples={} provenance={:?}",
                baseline.kind, baseline.elapsed_micros, baseline.sample_count, baseline.provenance
            ));
        }
    }
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
        if let Some(source_version) = ui_designer_source_version_label(state) {
            fields.push(field("Source version", source_version));
        }
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
    if let Some(snapshot) = state.selected_last_applied_document() {
        summary.push(format!(
            "last applied snapshot: {} {:?}",
            snapshot.display_name, snapshot.lifecycle_state
        ));
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
    mut summary_lines: Vec<String>,
) -> EditorLabReviewViewModel {
    if let Some(source_version) = ui_designer_source_version_label(state) {
        summary_lines.insert(0, format!("source version: {source_version}"));
    }
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

fn ui_designer_source_version_label(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Option<String> {
    state.selected_source_version_label()
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
