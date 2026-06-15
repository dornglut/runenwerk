//! File: apps/runenwerk_editor/src/shell/self_authoring/mod.rs
//! Purpose: App-owned UI/editor definition authoring state, preview, apply, and rollback.

mod evidence;
mod operations;
mod project_io;
mod recipes;
mod session;

pub use evidence::EditorLabProductPathEvidenceCapture;
use operations::*;
pub use project_io::{DefinitionApplyPreview, EditorDefinitionExportPackage};
use recipes::*;
use session::EditorLabOperationHistory;
pub use session::{EditorLabOperationHistoryEntry, EditorLabOperationHistorySnapshot};

use anyhow::{Context, Result};
use editor_definition::{
    EditorCommandBindingDefinition, EditorCommandBindingSetDefinition, EditorDefinitionDocument,
    EditorDefinitionDocumentContent, EditorDefinitionDocumentKind, EditorDefinitionId,
    EditorDefinitionLifecycleState, EditorLabOperation, EditorLabOperationDiff,
    EditorLabOperationDiffChange, EditorLabOperationDiffFamily, EditorLabOperationKind,
    EditorLabOperationReport, EditorLabOperationStatus, EditorMenuDefinition,
    EditorMenuItemDefinition, EditorShortcutDefinition, EditorShortcutSetDefinition,
    EditorThemeDefinition, EditorTypographyTokenDefinition, EditorWorkbenchCompositionDefinition,
    EditorWorkbenchHostPolicyDefinition, EditorWorkspaceHostDefinition,
    EditorWorkspaceLayoutDefinition, EditorWorkspacePanelTabDefinition,
    EditorWorkspaceProfileDefinition, EditorWorkspaceSplitAxisDefinition,
    apply_editor_lab_operation, editor_definition_has_blocking_diagnostics,
    validate_editor_definition_document,
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};
use ui_definition::{
    AuthoredId, AuthoredUiNodePath, AuthoredUiTemplate, FormedRetainedUiProduct,
    UiDefinitionContext, UiDefinitionDiagnostic, UiDefinitionDiagnosticSeverity, UiNodeDefinition,
    UiRecipeExpansionRequest, UiRecipeId, UiRecipeLibrary, UiRecipeTargetProfileId, UiValueBinding,
    UiVisualLayoutEditKind, UiVisualLayoutOperation, VersionedAuthoredUiTemplate, expand_ui_recipe,
    migrate_authored_ui_template, normalize_authored_template,
};
use ui_theme::ThemeTokens;

use crate::shell::editor_lab_evidence::{
    EditorLabDescriptorCompatibility, EditorLabDescriptorCompatibilityEvidencePacket,
    EditorLabEvidenceArtifact, EditorLabEvidenceArtifactKind, EditorLabEvidenceArtifactProvenance,
    EditorLabPerformanceBaseline, EditorLabPerformanceBaselineKind,
    EditorLabReadOnlyFixtureBindingDescriptor, EditorLabRuntimeProductEvidencePacket,
    EditorLabScenarioEvidencePacket, EditorLabSourceRevision, EditorLabUnsupportedCheckDiagnostic,
    EditorLabValidatedIntentDescriptor, game_runtime,
};
use crate::shell::editor_lab_project::{
    DefinitionApplyDiffFamily, DefinitionApplyDiffRow, DefinitionApplyReview,
    DefinitionApplyReviewStatus, EditorDefinitionActivationPayload, EditorLabDocumentStore,
    EditorLabProjectImportReport, EditorLabProjectLoadReport, EditorLabProjectPackage,
    EditorLabProjectStoreReport, EditorLabRollbackRecord, EditorLabRollbackStatus,
};
use crate::shell::ui_definition_assets::{EDITOR_BINDINGS_SOURCE, EDITOR_UI_ASSET_SOURCES};

pub const EDITOR_DEFINITION_EXPORT_PACKAGE_VERSION: u32 = 1;
pub const EDITOR_DEFINITION_EXPORT_PACKAGE_KIND: &str = "runenwerk.editor.definition.export";
const UI_DESIGNER_SCENARIO_EVIDENCE_TARGETS: [&str; 2] = [
    UI_DESIGNER_WORKBENCH_TARGET_PROFILE,
    game_runtime::GAME_RUNTIME_TARGET_PROFILE,
];
const UI_DESIGNER_WORKBENCH_TARGET_PROFILE: &str = "editor.workbench";

#[derive(Debug, Clone, Default)]
pub struct SelfAuthoringWorkspaceState {
    drafts: BTreeMap<EditorDefinitionId, EditorDefinitionDocument>,
    applied: BTreeMap<EditorDefinitionId, EditorDefinitionDocument>,
    selected_document_id: Option<EditorDefinitionId>,
    selected_ui_node_id: Option<String>,
    last_apply_preview: Option<DefinitionApplyPreview>,
    last_apply_review: Option<DefinitionApplyReview>,
    last_operation_report: Option<EditorLabOperationReport>,
    operation_history: EditorLabOperationHistory,
    document_store: EditorLabDocumentStore,
    rollback_snapshots: BTreeMap<EditorDefinitionId, Option<EditorDefinitionDocument>>,
    rollback_records: Vec<EditorLabRollbackRecord>,
    last_applied_snapshots: BTreeMap<EditorDefinitionId, EditorDefinitionDocument>,
    last_scenario_evidence_packets: Vec<EditorLabScenarioEvidencePacket>,
    source_revision_epoch: u64,
    recipe_catalog_filter: String,
}

impl SelfAuthoringWorkspaceState {
    pub fn from_checked_in_fixtures() -> Result<Self> {
        let mut drafts = BTreeMap::new();
        for (path, source) in EDITOR_UI_ASSET_SOURCES {
            let template: AuthoredUiTemplate =
                ron::from_str(source).with_context(|| format!("failed to parse {path}"))?;
            let id = EditorDefinitionId::new(template.id.as_str().to_string());
            let document = EditorDefinitionDocument::current(
                id.clone(),
                path.strip_prefix("assets/editor/ui/")
                    .unwrap_or(path)
                    .to_string(),
                EditorDefinitionDocumentKind::UiLayout,
                EditorDefinitionDocumentContent::UiTemplate(template),
            );
            drafts.insert(id, document);
        }

        let bindings = ron::from_str(EDITOR_BINDINGS_SOURCE)
            .context("failed to parse assets/editor/ui/editor_bindings.ron")?;
        let bindings_id = EditorDefinitionId::from("runenwerk.editor.bindings");
        drafts.insert(
            bindings_id.clone(),
            EditorDefinitionDocument::current(
                bindings_id,
                "editor_bindings.ron",
                EditorDefinitionDocumentKind::EditorBindings,
                EditorDefinitionDocumentContent::EditorBindings(bindings),
            ),
        );
        for document in default_editor_definition_documents() {
            drafts.insert(document.id.clone(), document);
        }

        let selected_document_id = Some(EditorDefinitionId::from("runenwerk.editor.toolbar"))
            .filter(|id| drafts.contains_key(id))
            .or_else(|| drafts.keys().next().cloned());
        let selected_ui_node_id = selected_document_id
            .as_ref()
            .and_then(|id| selected_ui_default_node_id(&drafts, id));
        Ok(Self {
            drafts,
            applied: BTreeMap::new(),
            selected_document_id,
            selected_ui_node_id,
            last_apply_preview: None,
            last_apply_review: None,
            last_operation_report: None,
            operation_history: EditorLabOperationHistory::default(),
            document_store: EditorLabDocumentStore::default(),
            rollback_snapshots: BTreeMap::new(),
            rollback_records: Vec::new(),
            last_applied_snapshots: BTreeMap::new(),
            last_scenario_evidence_packets: Vec::new(),
            source_revision_epoch: 1,
            recipe_catalog_filter: String::new(),
        })
    }

    pub fn draft_documents(&self) -> impl Iterator<Item = &EditorDefinitionDocument> {
        self.drafts.values()
    }

    pub fn selected_document_id(&self) -> Option<&EditorDefinitionId> {
        self.selected_document_id.as_ref()
    }

    pub fn selected_source_version_label(&self) -> Option<String> {
        self.selected_source_revision()
            .map(|revision| revision.display_label())
    }

    pub fn selected_source_revision(&self) -> Option<EditorLabSourceRevision> {
        self.selected_document()
            .map(|document| self.source_revision_for_document(document))
    }

    pub fn recipe_catalog_filter(&self) -> &str {
        &self.recipe_catalog_filter
    }

    pub fn set_recipe_catalog_filter(&mut self, query: impl Into<String>) {
        self.recipe_catalog_filter = query.into();
    }

    pub fn selected_document(&self) -> Option<&EditorDefinitionDocument> {
        self.selected_document_id
            .as_ref()
            .and_then(|id| self.drafts.get(id))
    }

    pub fn selected_ui_node_id(&self) -> Option<&str> {
        self.selected_ui_node_id.as_deref()
    }

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

    pub fn select_document(&mut self, document_id: EditorDefinitionId) -> bool {
        if !self.drafts.contains_key(&document_id) {
            return false;
        }
        self.selected_ui_node_id = selected_ui_default_node_id(&self.drafts, &document_id);
        self.selected_document_id = Some(document_id);
        self.record_source_change();
        true
    }

    pub fn select_document_by_str(&mut self, document_id: &str) -> bool {
        self.select_document(EditorDefinitionId::new(document_id.to_string()))
    }

    pub fn generated_duplicate_id(&self) -> Option<EditorDefinitionId> {
        let base = self.selected_document_id.as_ref()?.as_str();
        for index in 1..=999 {
            let candidate = EditorDefinitionId::new(format!("{base}.copy{index}"));
            if !self.drafts.contains_key(&candidate) {
                return Some(candidate);
            }
        }
        None
    }

    pub fn create_document(
        &mut self,
        document: EditorDefinitionDocument,
    ) -> Result<(), UiDefinitionDiagnostic> {
        if self.drafts.contains_key(&document.id) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.create.duplicate",
                format!(
                    "definition document '{}' already exists",
                    document.id.as_str()
                ),
            ));
        }
        let diagnostics = validate_editor_definition_document(&document);
        if editor_definition_has_blocking_diagnostics(&diagnostics) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.create.blocked",
                "definition document has blocking validation diagnostics",
            ));
        }
        self.selected_ui_node_id = selected_ui_default_node_for_document(&document);
        self.selected_document_id = Some(document.id.clone());
        self.drafts.insert(document.id.clone(), document);
        self.record_source_change();
        Ok(())
    }

    pub fn create_custom_workbench_package(
        &mut self,
    ) -> Result<EditorDefinitionId, UiDefinitionDiagnostic> {
        let package_index = (1..=999)
            .find(|index| {
                let composition_id = EditorDefinitionId::from(
                    format!("runenwerk.editor.workbench.custom{index}").as_str(),
                );
                !self.drafts.contains_key(&composition_id)
            })
            .ok_or_else(|| {
                UiDefinitionDiagnostic::error(
                    "editor.self_authoring.workbench.create.exhausted",
                    "no custom workbench id is available",
                )
            })?;
        let composition_id = format!("runenwerk.editor.workbench.custom{package_index}");
        let profile_id = format!("runenwerk.editor.workspace.custom{package_index}");
        let layout_id = format!("runenwerk.editor.layout.custom{package_index}");
        let layout = self
            .selected_workspace_layout()
            .cloned()
            .map(|mut layout| {
                layout.id = layout_id.clone();
                layout.label = format!("Custom {package_index} Layout");
                layout
            })
            .unwrap_or_else(|| default_custom_workbench_layout(&layout_id, package_index));

        let documents = vec![
            EditorDefinitionDocument::current(
                EditorDefinitionId::from(composition_id.as_str()),
                format!("Custom Workbench {package_index}"),
                EditorDefinitionDocumentKind::WorkbenchComposition,
                EditorDefinitionDocumentContent::WorkbenchComposition(
                    EditorWorkbenchCompositionDefinition {
                        id: composition_id.clone(),
                        label: format!("Custom Workbench {package_index}"),
                        installed_suites: vec![
                            "runenwerk.editor".to_string(),
                            "runenwerk.editor_design".to_string(),
                        ],
                        profile_refs: vec![profile_id.clone()],
                        default_profile_ref: profile_id.clone(),
                        host_policy: EditorWorkbenchHostPolicyDefinition::AllowAll,
                    },
                ),
            ),
            EditorDefinitionDocument::current(
                EditorDefinitionId::from(profile_id.as_str()),
                format!("Custom Workspace {package_index}"),
                EditorDefinitionDocumentKind::WorkspaceDefinition,
                EditorDefinitionDocumentContent::WorkspaceProfile(
                    EditorWorkspaceProfileDefinition {
                        id: profile_id,
                        label: format!("Custom {package_index}"),
                        default_modes: vec!["editor-design".to_string()],
                        document_kind_filters: vec![
                            "UiLayout".to_string(),
                            "WorkspaceDefinition".to_string(),
                            "Theme".to_string(),
                            "Shortcut".to_string(),
                            "Menu".to_string(),
                            "CommandBinding".to_string(),
                        ],
                        default_layout: layout_id,
                    },
                ),
            ),
            EditorDefinitionDocument::current(
                EditorDefinitionId::from(layout.id.as_str()),
                layout.label.clone(),
                EditorDefinitionDocumentKind::WorkspaceDefinition,
                EditorDefinitionDocumentContent::WorkspaceLayout(layout),
            ),
        ];

        for document in &documents {
            if self.drafts.contains_key(&document.id) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.self_authoring.workbench.create.duplicate",
                    format!(
                        "definition document '{}' already exists",
                        document.id.as_str()
                    ),
                ));
            }
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                return Err(UiDefinitionDiagnostic::error(
                    "editor.self_authoring.workbench.create.blocked",
                    format!(
                        "custom workbench document '{}' has blocking validation diagnostics",
                        document.id.as_str()
                    ),
                ));
            }
        }

        let selected_id = EditorDefinitionId::from(composition_id.as_str());
        for document in documents {
            self.drafts.insert(document.id.clone(), document);
        }
        self.selected_ui_node_id = None;
        self.selected_document_id = Some(selected_id.clone());
        self.record_source_change();
        Ok(selected_id)
    }

    pub fn selected_workbench_composition_payload(
        &self,
    ) -> Result<EditorDefinitionActivationPayload, UiDefinitionDiagnostic> {
        let (composition, profiles, layouts) = self.selected_workbench_composition_package()?;
        Ok(
            EditorDefinitionActivationPayload::WorkbenchCompositionPackage {
                composition,
                profiles,
                layouts,
            },
        )
    }

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

    pub fn duplicate_selected(
        &mut self,
        new_id: EditorDefinitionId,
        display_name: impl Into<String>,
    ) -> Result<EditorDefinitionId, UiDefinitionDiagnostic> {
        if self.drafts.contains_key(&new_id) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.duplicate.duplicate",
                format!("definition document '{}' already exists", new_id.as_str()),
            ));
        }
        let selected = self.selected_document().cloned().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.duplicate.no_selection",
                "no definition document is selected",
            )
        })?;
        let mut duplicate = selected;
        duplicate.id = new_id.clone();
        duplicate.display_name = display_name.into();
        duplicate.lifecycle_state = EditorDefinitionLifecycleState::Draft;
        if let EditorDefinitionDocumentContent::UiTemplate(template) = &mut duplicate.content {
            template.id = ui_definition::UiTemplateId::new(new_id.as_str().to_string());
        }
        self.drafts.insert(new_id.clone(), duplicate);
        self.selected_document_id = Some(new_id.clone());
        self.selected_ui_node_id = selected_ui_default_node_id(&self.drafts, &new_id);
        self.record_source_change();
        Ok(new_id)
    }

    pub fn rename_selected(
        &mut self,
        display_name: impl Into<String>,
    ) -> Result<(), UiDefinitionDiagnostic> {
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.rename.no_selection",
                "no definition document is selected",
            )
        })?;
        let document = self.drafts.get_mut(&document_id).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.rename.unresolved",
                "selected definition document is not loaded",
            )
        })?;
        document.display_name = display_name.into();
        self.record_source_change();
        Ok(())
    }

    pub fn delete_selected(&mut self) -> Result<EditorDefinitionDocument, UiDefinitionDiagnostic> {
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.delete.no_selection",
                "no definition document is selected",
            )
        })?;
        if self.applied.contains_key(&document_id) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.delete.active",
                "applied definitions must be rolled back before deletion",
            ));
        }
        let removed = self.drafts.remove(&document_id).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.delete.unresolved",
                "selected definition document is not loaded",
            )
        })?;
        self.rollback_snapshots.remove(&document_id);
        self.last_applied_snapshots.remove(&document_id);
        self.selected_document_id = self.drafts.keys().next().cloned();
        self.selected_ui_node_id = self
            .selected_document_id
            .as_ref()
            .and_then(|id| selected_ui_default_node_id(&self.drafts, id));
        self.record_source_change();
        Ok(removed)
    }

    pub fn import_versioned_ui_template_document(
        &mut self,
        source: &str,
        display_name: impl Into<String>,
    ) -> Result<EditorDefinitionId, UiDefinitionDiagnostic> {
        let versioned: VersionedAuthoredUiTemplate = ron::from_str(source).map_err(|error| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.import.parse_failed",
                format!("failed to parse imported UI definition: {error}"),
            )
        })?;
        let migration = migrate_authored_ui_template(versioned);
        if migration.has_errors() {
            return Err(migration.diagnostics.into_iter().next().unwrap_or_else(|| {
                UiDefinitionDiagnostic::error(
                    "editor.self_authoring.import.migration_failed",
                    "imported UI definition migration failed",
                )
            }));
        }
        let template = migration.migrated.template;
        let id = EditorDefinitionId::new(template.id.as_str().to_string());
        self.create_document(EditorDefinitionDocument::current(
            id.clone(),
            display_name,
            EditorDefinitionDocumentKind::UiLayout,
            EditorDefinitionDocumentContent::UiTemplate(template),
        ))?;
        Ok(id)
    }

    pub fn select_ui_node(
        &mut self,
        node_id: impl Into<String>,
    ) -> Result<(), UiDefinitionDiagnostic> {
        let node_id = node_id.into();
        let document = self.selected_document().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.node.no_selection",
                "no definition document is selected",
            )
        })?;
        let EditorDefinitionDocumentContent::UiTemplate(template) = &document.content else {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.node.not_ui_template",
                "selected definition is not a UI template",
            ));
        };
        if !ui_node_exists(&template.root, &node_id) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.node.unresolved",
                format!("UI node '{node_id}' is not present in the selected definition"),
            ));
        }
        self.selected_ui_node_id = Some(node_id);
        Ok(())
    }

    pub fn set_selected_ui_node_text(
        &mut self,
        node_id: &str,
        text: impl Into<String>,
    ) -> Result<(), UiDefinitionDiagnostic> {
        self.select_ui_node(node_id.to_string())?;
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.node.no_selection",
                "no definition document is selected",
            )
        })?;
        let document = self.drafts.get_mut(&document_id).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.node.unresolved_document",
                "selected definition document is not loaded",
            )
        })?;
        let EditorDefinitionDocumentContent::UiTemplate(template) = &mut document.content else {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.node.not_ui_template",
                "selected definition is not a UI template",
            ));
        };
        set_ui_node_text(&mut template.root, node_id, text.into()).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.node.unsupported_text_edit",
                format!("UI node '{node_id}' does not expose an authored text value"),
            )
        })?;
        self.record_source_change();
        Ok(())
    }

    pub fn set_selected_theme_color(
        &mut self,
        token: &str,
        value: impl Into<String>,
    ) -> Result<(), UiDefinitionDiagnostic> {
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.theme.no_selection",
                "no definition document is selected",
            )
        })?;
        let document = self.drafts.get_mut(&document_id).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.theme.unresolved_document",
                "selected definition document is not loaded",
            )
        })?;
        let EditorDefinitionDocumentContent::Theme(theme) = &mut document.content else {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.theme.not_theme",
                "selected definition is not a theme document",
            ));
        };
        theme.colors.insert(token.to_string(), value.into());
        self.record_source_change();
        Ok(())
    }

    pub fn add_selected_workspace_layout_tab(
        &mut self,
        label: impl Into<String>,
        tool_surface: impl Into<String>,
    ) -> Result<String, UiDefinitionDiagnostic> {
        let layout = self.selected_workspace_layout_mut("add_tab")?;
        let host = first_tab_stack_mut(&mut layout.root).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.workspace.no_tab_stack",
                "selected workspace layout has no authored tab stack",
            )
        })?;
        let next_index = host.tabs.len() + 1;
        let tab_id = format!("authored-tab-{next_index}");
        host.tabs.push(EditorWorkspacePanelTabDefinition {
            id: tab_id.clone(),
            label: label.into(),
            tool_surface: tool_surface.into(),
        });
        *host.active_tab = Some(tab_id.clone());
        self.record_source_change();
        Ok(tab_id)
    }

    pub fn split_selected_workspace_layout_root(
        &mut self,
        axis: EditorWorkspaceSplitAxisDefinition,
    ) -> Result<(), UiDefinitionDiagnostic> {
        let layout = self.selected_workspace_layout_mut("split_root")?;
        if matches!(layout.root, EditorWorkspaceHostDefinition::Split { .. }) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.workspace.already_split",
                "selected workspace layout root is already split",
            ));
        }
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
        self.record_source_change();
        Ok(())
    }

    pub fn close_selected_workspace_layout_last_tab(
        &mut self,
    ) -> Result<EditorWorkspacePanelTabDefinition, UiDefinitionDiagnostic> {
        let layout = self.selected_workspace_layout_mut("close_last_tab")?;
        let host = first_tab_stack_mut(&mut layout.root).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.workspace.no_tab_stack",
                "selected workspace layout has no authored tab stack",
            )
        })?;
        if host.tabs.len() <= 1 {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.workspace.last_tab_guard",
                "authored tab stack must keep at least one tab",
            ));
        }
        let removed = host.tabs.pop().expect("tab length was checked");
        *host.active_tab = host.tabs.last().map(|tab| tab.id.clone());
        self.record_source_change();
        Ok(removed)
    }

    fn selected_workspace_layout_mut(
        &mut self,
        operation: &'static str,
    ) -> Result<&mut EditorWorkspaceLayoutDefinition, UiDefinitionDiagnostic> {
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                format!("editor.self_authoring.workspace.{operation}.no_selection"),
                "no definition document is selected",
            )
        })?;
        let document = self.drafts.get_mut(&document_id).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                format!("editor.self_authoring.workspace.{operation}.unresolved_document"),
                "selected definition document is not loaded",
            )
        })?;
        let EditorDefinitionDocumentContent::WorkspaceLayout(layout) = &mut document.content else {
            return Err(UiDefinitionDiagnostic::error(
                format!("editor.self_authoring.workspace.{operation}.not_layout"),
                "selected definition is not a workspace layout document",
            ));
        };
        Ok(layout)
    }

    fn selected_workspace_layout(&self) -> Option<&EditorWorkspaceLayoutDefinition> {
        let document = self.selected_document()?;
        let EditorDefinitionDocumentContent::WorkspaceLayout(layout) = &document.content else {
            return None;
        };
        Some(layout)
    }

    fn selected_workbench_composition_package(
        &self,
    ) -> Result<
        (
            EditorWorkbenchCompositionDefinition,
            Vec<EditorWorkspaceProfileDefinition>,
            Vec<EditorWorkspaceLayoutDefinition>,
        ),
        UiDefinitionDiagnostic,
    > {
        let document = self.selected_document().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.workbench.activate.no_selection",
                "no workbench composition document is selected",
            )
        })?;
        let EditorDefinitionDocumentContent::WorkbenchComposition(composition) = &document.content
        else {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.workbench.activate.not_composition",
                "selected definition is not a workbench composition document",
            ));
        };

        let mut profiles = Vec::new();
        let mut layouts = Vec::new();
        let mut layout_ids = BTreeSet::<String>::new();
        for profile_ref in &composition.profile_refs {
            let profile_document = self
                .drafts
                .values()
                .find(|candidate| {
                    matches!(
                        &candidate.content,
                        EditorDefinitionDocumentContent::WorkspaceProfile(profile)
                            if profile.id == *profile_ref
                    )
                })
                .ok_or_else(|| {
                    UiDefinitionDiagnostic::error(
                        "editor.self_authoring.workbench.activate.profile_missing",
                        format!("workbench composition references missing profile `{profile_ref}`"),
                    )
                })?;
            let EditorDefinitionDocumentContent::WorkspaceProfile(profile) =
                &profile_document.content
            else {
                unreachable!("profile document content was matched above");
            };
            profiles.push(profile.clone());

            if layout_ids.insert(profile.default_layout.clone()) {
                let layout_document = self
                    .drafts
                    .values()
                    .find(|candidate| {
                        matches!(
                            &candidate.content,
                            EditorDefinitionDocumentContent::WorkspaceLayout(layout)
                                if layout.id == profile.default_layout
                        )
                    })
                    .ok_or_else(|| {
                        UiDefinitionDiagnostic::error(
                            "editor.self_authoring.workbench.activate.layout_missing",
                            format!(
                                "workspace profile `{}` references missing layout `{}`",
                                profile.id, profile.default_layout
                            ),
                        )
                    })?;
                let EditorDefinitionDocumentContent::WorkspaceLayout(layout) =
                    &layout_document.content
                else {
                    unreachable!("layout document content was matched above");
                };
                layouts.push(layout.clone());
            }
        }

        Ok((composition.clone(), profiles, layouts))
    }

    fn selected_workbench_composition_package_documents(
        &self,
    ) -> Result<Vec<EditorDefinitionDocument>, UiDefinitionDiagnostic> {
        let (composition, profiles, layouts) = self.selected_workbench_composition_package()?;
        Ok(workbench_composition_package_documents(
            &composition,
            &profiles,
            &layouts,
        ))
    }

    pub fn export_selected_to_ron(&self) -> Result<String, UiDefinitionDiagnostic> {
        let package = self.export_selected_package()?;
        ron::ser::to_string_pretty(&package, PrettyConfig::new()).map_err(|error| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.export.serialize_failed",
                format!("failed to export definition package: {error}"),
            )
        })
    }

    pub fn export_project_package(&self) -> EditorLabProjectPackage {
        EditorLabProjectPackage::current(
            self.drafts.values().cloned(),
            self.applied.values().cloned(),
            self.last_applied_snapshots.values().cloned(),
        )
    }

    pub fn save_project_package_to_ron(&mut self) -> Result<String, UiDefinitionDiagnostic> {
        let package = self.export_project_package();
        self.document_store.save_package_source(&package)
    }

    pub fn save_project_package_to_path(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<EditorLabProjectStoreReport, UiDefinitionDiagnostic> {
        let package = self.export_project_package();
        self.document_store.save_package_to_path(&package, path)
    }

    pub fn load_project_package_from_ron(
        &mut self,
        source: &str,
    ) -> Result<EditorLabProjectLoadReport, UiDefinitionDiagnostic> {
        let package = self.document_store.load_package_source(source)?;
        self.load_project_package(package)
    }

    pub fn reload_last_saved_project_package(
        &mut self,
    ) -> Result<EditorLabProjectLoadReport, UiDefinitionDiagnostic> {
        let source = self
            .document_store
            .last_saved_package_source()
            .ok_or_else(|| {
                UiDefinitionDiagnostic::error(
                    "editor.lab.project.reload.no_saved_package",
                    "no Editor Lab project package has been saved in this session",
                )
            })?
            .to_string();
        self.load_project_package_from_ron(&source)
    }

    pub fn last_saved_project_package_source(&self) -> Option<&str> {
        self.document_store.last_saved_package_source()
    }

    pub fn last_loaded_project_package_source(&self) -> Option<&str> {
        self.document_store.last_loaded_package_source()
    }

    pub fn last_invalid_project_package_source(&self) -> Option<&str> {
        self.document_store.last_invalid_package_source()
    }

    pub fn last_invalid_project_package_diagnostics(&self) -> &[UiDefinitionDiagnostic] {
        self.document_store.last_invalid_package_diagnostics()
    }

    pub fn import_selected_package_from_ron(
        &mut self,
        source: &str,
    ) -> Result<EditorLabProjectImportReport, UiDefinitionDiagnostic> {
        let package: EditorDefinitionExportPackage = ron::from_str(source).map_err(|error| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.import.parse_failed",
                format!("failed to parse selected definition package: {error}"),
            )
        })?;
        if package.package_version != EDITOR_DEFINITION_EXPORT_PACKAGE_VERSION {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.import.unsupported_version",
                format!(
                    "unsupported selected definition package version {}",
                    package.package_version
                ),
            ));
        }
        if package.package_kind != EDITOR_DEFINITION_EXPORT_PACKAGE_KIND {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.import.unsupported_kind",
                format!(
                    "unsupported selected definition package kind '{}'",
                    package.package_kind
                ),
            ));
        }
        let document = package.document;
        let diagnostics = validate_editor_definition_document(&document);
        if editor_definition_has_blocking_diagnostics(&diagnostics) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.import.blocked",
                "imported definition has blocking validation diagnostics",
            ));
        }
        let replaced_existing = self
            .drafts
            .insert(document.id.clone(), document.clone())
            .is_some();
        self.selected_document_id = Some(document.id.clone());
        self.selected_ui_node_id = selected_ui_default_node_id(&self.drafts, &document.id);
        self.record_source_change();
        Ok(EditorLabProjectImportReport {
            document_id: document.id,
            display_name: document.display_name,
            replaced_existing,
        })
    }

    pub fn export_selected_package(
        &self,
    ) -> Result<EditorDefinitionExportPackage, UiDefinitionDiagnostic> {
        let document = self.selected_document().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.export.no_selection",
                "no definition document is selected",
            )
        })?;
        let diagnostics = validate_editor_definition_document(document);
        if editor_definition_has_blocking_diagnostics(&diagnostics) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.export.blocked",
                "definition has blocking validation diagnostics",
            ));
        }
        Ok(EditorDefinitionExportPackage::current(document.clone()))
    }

    pub fn diagnostics_for_document(
        &self,
        document_id: &EditorDefinitionId,
    ) -> Vec<UiDefinitionDiagnostic> {
        self.drafts
            .get(document_id)
            .map(validate_editor_definition_document)
            .unwrap_or_else(|| {
                vec![UiDefinitionDiagnostic {
                    severity: UiDefinitionDiagnosticSeverity::Error,
                    code: "editor.self_authoring.document.unresolved".to_string(),
                    message: format!(
                        "definition document '{}' is not loaded",
                        document_id.as_str()
                    ),
                    path: None,
                }]
            })
    }

    pub fn selected_diagnostics(&self) -> Vec<UiDefinitionDiagnostic> {
        self.selected_document_id
            .as_ref()
            .map(|id| self.diagnostics_for_document(id))
            .unwrap_or_default()
    }

    pub fn formed_selected_preview(&self, theme: &ThemeTokens) -> Option<FormedRetainedUiProduct> {
        self.formed_selected_preview_with_scope(theme, None)
    }

    pub fn formed_selected_preview_with_scope(
        &self,
        theme: &ThemeTokens,
        widget_id_scope_base: Option<u64>,
    ) -> Option<FormedRetainedUiProduct> {
        let document = self.selected_document()?;
        let EditorDefinitionDocumentContent::UiTemplate(template) = &document.content else {
            return None;
        };
        let normalized = normalize_authored_template(template.clone());
        let mut context = UiDefinitionContext::new(theme.clone());
        if let Some(base) = widget_id_scope_base {
            context = context.with_widget_id_scope(ui_definition::WidgetIdScope::new(base));
        }
        Some(ui_definition::form_retained_ui(&normalized, &mut context))
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

    pub fn last_scenario_evidence_packets(&self) -> &[EditorLabScenarioEvidencePacket] {
        &self.last_scenario_evidence_packets
    }

    pub fn capture_pm005_scenario_evidence_packets(
        &mut self,
        theme: &ThemeTokens,
    ) -> Result<Vec<EditorLabScenarioEvidencePacket>, UiDefinitionDiagnostic> {
        self.capture_pm005_scenario_evidence_packets_with_product_capture(
            theme,
            EditorLabProductPathEvidenceCapture::default(),
        )
    }

    pub fn capture_pm005_scenario_evidence_packets_with_product_capture(
        &mut self,
        theme: &ThemeTokens,
        product_capture: EditorLabProductPathEvidenceCapture,
    ) -> Result<Vec<EditorLabScenarioEvidencePacket>, UiDefinitionDiagnostic> {
        let document = self.selected_document().cloned().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.no_selected_document",
                "no selected UI Designer document is available for scenario evidence capture",
            )
        })?;
        let source_revision = self.source_revision_for_document(&document);
        let diagnostics = self.diagnostics_for_document(&document.id);
        let runtime_product_capture =
            self.runtime_product_evidence_capture_for_selected_document(theme, product_capture)?;
        let mut packets = Vec::new();
        for target_profile in UI_DESIGNER_SCENARIO_EVIDENCE_TARGETS {
            let packet = self.capture_pm005_scenario_evidence_packet_for_target(
                &document,
                source_revision.clone(),
                diagnostics.clone(),
                theme,
                target_profile,
                &runtime_product_capture,
            )?;
            packets.push(packet);
        }
        self.last_scenario_evidence_packets = packets.clone();
        Ok(packets)
    }

    pub fn applied_document(&self, id: &EditorDefinitionId) -> Option<&EditorDefinitionDocument> {
        self.applied.get(id)
    }

    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }

    fn load_project_package(
        &mut self,
        package: EditorLabProjectPackage,
    ) -> Result<EditorLabProjectLoadReport, UiDefinitionDiagnostic> {
        package.validate()?;
        self.drafts = package
            .draft_documents
            .into_iter()
            .map(|document| (document.id.clone(), document))
            .collect();
        self.applied = package
            .applied_documents
            .into_iter()
            .map(|document| (document.id.clone(), document))
            .collect();
        self.last_applied_snapshots = package
            .last_applied_documents
            .into_iter()
            .map(|document| (document.id.clone(), document))
            .collect();
        let selected_missing = match self.selected_document_id.as_ref() {
            Some(id) => !self.drafts.contains_key(id),
            None => true,
        };
        if selected_missing {
            self.selected_document_id = self.drafts.keys().next().cloned();
        }
        self.selected_ui_node_id = self
            .selected_document_id
            .as_ref()
            .and_then(|id| selected_ui_default_node_id(&self.drafts, id));
        self.last_apply_preview = None;
        self.last_apply_review = None;
        self.last_scenario_evidence_packets.clear();
        self.record_source_change();
        Ok(EditorLabProjectLoadReport {
            draft_count: self.drafts.len(),
            applied_count: self.applied.len(),
            last_applied_count: self.last_applied_snapshots.len(),
        })
    }

    fn source_revision_for_document(
        &self,
        document: &EditorDefinitionDocument,
    ) -> EditorLabSourceRevision {
        let serialized = ron::ser::to_string_pretty(document, PrettyConfig::new())
            .unwrap_or_else(|_| format!("{document:?}"));
        EditorLabSourceRevision::new(
            document.id.as_str(),
            document.schema_version,
            format!("blake3:{}", blake3::hash(serialized.as_bytes()).to_hex()),
            self.source_revision_epoch,
        )
    }

    fn capture_pm005_scenario_evidence_packet_for_target(
        &self,
        document: &EditorDefinitionDocument,
        source_revision: EditorLabSourceRevision,
        diagnostics: Vec<UiDefinitionDiagnostic>,
        _theme: &ThemeTokens,
        target_profile: &str,
        runtime_product_capture: &EditorLabProductPathEvidenceCapture,
    ) -> Result<EditorLabScenarioEvidencePacket, UiDefinitionDiagnostic> {
        if target_profile == game_runtime::GAME_RUNTIME_TARGET_PROFILE {
            let unsupported_report = format!(
                "target={target_profile}\ndocument={}\nsource={}\nstatus=descriptor-compatible\nruntime-proof-owner=PT-GAME-RUNTIME-UI\n",
                document.id.as_str(),
                source_revision.display_label()
            );
            let artifact = EditorLabEvidenceArtifact::from_content(
                EditorLabEvidenceArtifactKind::UnsupportedCheckReport,
                format!(
                    "evidence://ui-designer/v1-closure/pm005/{}/{}/descriptor-compatibility",
                    target_profile.replace('.', "-"),
                    document.id.as_str()
                ),
                unsupported_report.as_bytes(),
                EditorLabEvidenceArtifactProvenance::UnsupportedCheck,
                "game.runtime descriptor compatibility report; runtime proof is owned by PT-GAME-RUNTIME-UI",
            );
            let packet = EditorLabDescriptorCompatibilityEvidencePacket::new(
                "runenwerk.editor.ui_designer_workbench.v1",
                document.id.as_str(),
                source_revision,
                target_profile,
                pm005_scenario_id(target_profile),
            )
            .with_diagnostics(diagnostics)
            .with_artifacts(vec![artifact])
            .with_unsupported_checks(vec![EditorLabUnsupportedCheckDiagnostic::new(
                "concrete game HUD runtime",
                "PT-GAME-RUNTIME-UI owns concrete game HUD behavior; UI Designer records descriptor compatibility only",
            )])
            .with_fixture_bindings(pm005_fixture_binding_descriptors(target_profile))
            .with_intent_descriptors(pm005_intent_descriptors(target_profile));

            let packet = EditorLabScenarioEvidencePacket::descriptor(packet);
            packet.validate_scenario_evidence()?;
            return Ok(packet);
        }

        let packet = EditorLabRuntimeProductEvidencePacket::new(
            "runenwerk.editor.ui_designer_workbench.v1",
            document.id.as_str(),
            source_revision,
            target_profile,
            pm005_scenario_id(target_profile),
        )
        .with_diagnostics(diagnostics)
        .with_artifacts(runtime_product_capture.artifacts.clone())
        .with_performance_baselines(runtime_product_capture.performance_baselines.clone());

        let packet = EditorLabScenarioEvidencePacket::runtime(packet);
        packet.validate_scenario_evidence()?;
        Ok(packet)
    }

    fn measured_product_path_baseline(
        &self,
        kind: EditorLabPerformanceBaselineKind,
        description: impl Into<String>,
        sample: impl FnOnce() -> usize,
    ) -> EditorLabPerformanceBaseline {
        let started = Instant::now();
        let sample_count = sample().max(1);
        let elapsed_micros = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        EditorLabPerformanceBaseline::product_path(kind, elapsed_micros, sample_count, description)
    }

    fn runtime_product_evidence_capture_for_selected_document(
        &self,
        theme: &ThemeTokens,
        mut product_capture: EditorLabProductPathEvidenceCapture,
    ) -> Result<EditorLabProductPathEvidenceCapture, UiDefinitionDiagnostic> {
        let document = self.selected_document().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.lab.evidence.runtime.no_selected_document",
                "runtime product evidence capture requires a selected document",
            )
        })?;
        let source_revision = self.source_revision_for_document(document);
        let preview_started = Instant::now();
        let preview = self.formed_selected_preview(theme).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.lab.evidence.runtime.no_retained_preview",
                "runtime product evidence capture requires a retained UI preview product",
            )
        })?;
        let preview_debug = format!("{preview:#?}");
        let preview_elapsed = preview_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        product_capture
            .artifacts
            .push(EditorLabEvidenceArtifact::from_content(
                EditorLabEvidenceArtifactKind::RetainedUiDebug,
                format!(
                    "evidence://ui-designer/runtime/{}/{}/retained-ui-debug",
                    document.id.as_str(),
                    source_revision.content_hash
                ),
                preview_debug.as_bytes(),
                EditorLabEvidenceArtifactProvenance::ProductPath,
                "retained UI Designer workbench product formed through ui_definition",
            ));
        product_capture
            .performance_baselines
            .push(EditorLabPerformanceBaseline::product_path(
                EditorLabPerformanceBaselineKind::CanvasInteraction,
                preview_elapsed,
                1,
                "editor.workbench retained canvas preview formation through ui_definition",
            ));
        product_capture
            .performance_baselines
            .push(self.measured_product_path_baseline(
                EditorLabPerformanceBaselineKind::Resize,
                "editor.workbench retained canvas formation across responsive widget id scopes",
                || {
                    [10_u64, 20_u64, 30_u64]
                        .into_iter()
                        .filter(|scope| {
                            self.formed_selected_preview_with_scope(theme, Some(*scope))
                                .is_some()
                        })
                        .count()
                },
            ));
        product_capture
            .performance_baselines
            .push(self.measured_product_path_baseline(
                EditorLabPerformanceBaselineKind::CatalogProjection,
                "editor.workbench component catalog projection from editor design recipe library",
                || {
                    editor_shell::editor_design_system_recipe_library()
                        .declarations
                        .len()
                },
            ));
        product_capture
            .performance_baselines
            .push(self.measured_product_path_baseline(
            EditorLabPerformanceBaselineKind::DiagnosticsProjection,
            "editor.workbench selected diagnostics projection through editor_definition validation",
            || self.selected_diagnostics().len(),
        ));
        if !product_capture
            .performance_baselines
            .iter()
            .any(|baseline| baseline.kind == EditorLabPerformanceBaselineKind::FrameBuild)
        {
            product_capture
                .performance_baselines
                .push(self.measured_product_path_baseline(
                    EditorLabPerformanceBaselineKind::FrameBuild,
                    "editor.workbench apply preview frame inventory from selected product state",
                    || {
                        usize::from(self.build_apply_preview().is_some())
                            + self.draft_documents().count()
                            + self.applied_count()
                    },
                ));
        }

        dedupe_product_path_baselines(&mut product_capture.performance_baselines);
        Ok(product_capture)
    }

    fn record_source_change(&mut self) {
        self.source_revision_epoch = self.source_revision_epoch.saturating_add(1);
        self.last_scenario_evidence_packets.clear();
    }
}

fn pm005_scenario_id(target_profile: &str) -> String {
    format!(
        "ui-designer.v1-closure.pm005.{}",
        target_profile.replace('.', "-")
    )
}

fn dedupe_product_path_baselines(baselines: &mut Vec<EditorLabPerformanceBaseline>) {
    let mut seen = BTreeSet::new();
    baselines.retain(|baseline| seen.insert(baseline.kind));
}

fn pm005_fixture_binding_descriptors(
    target_profile: &str,
) -> Vec<EditorLabReadOnlyFixtureBindingDescriptor> {
    if target_profile == game_runtime::GAME_RUNTIME_TARGET_PROFILE {
        return game_runtime::descriptor_fixture_bindings();
    }
    let profile_segment = target_profile.replace('.', "-");
    vec![
        EditorLabReadOnlyFixtureBindingDescriptor::new(
            format!("fixture.{profile_segment}.selected-template"),
            format!("binding.{profile_segment}.source-version"),
            target_profile,
            EditorLabDescriptorCompatibility::Compatible,
            "source-versioned selected UI template fixture",
        ),
        EditorLabReadOnlyFixtureBindingDescriptor::new(
            format!("fixture.{profile_segment}.safe-area"),
            format!("binding.{profile_segment}.target-profile"),
            target_profile,
            EditorLabDescriptorCompatibility::Compatible,
            "read-only target-profile compatibility fixture",
        ),
    ]
}

fn pm005_intent_descriptors(target_profile: &str) -> Vec<EditorLabValidatedIntentDescriptor> {
    if target_profile == game_runtime::GAME_RUNTIME_TARGET_PROFILE {
        return game_runtime::validated_intent_descriptors();
    }
    let profile_segment = target_profile.replace('.', "-");
    vec![
        EditorLabValidatedIntentDescriptor::new(
            format!("intent.{profile_segment}.select-node"),
            target_profile,
            "validated EditorDefinition route descriptor",
        ),
        EditorLabValidatedIntentDescriptor::new(
            format!("intent.{profile_segment}.preview-activation"),
            target_profile,
            "validated preview intent descriptor; no game-runtime command execution",
        ),
    ]
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

fn default_editor_definition_documents() -> Vec<EditorDefinitionDocument> {
    vec![
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.workbench.editor_design"),
            "editor_design_workbench.ron",
            EditorDefinitionDocumentKind::WorkbenchComposition,
            EditorDefinitionDocumentContent::WorkbenchComposition(
                EditorWorkbenchCompositionDefinition {
                    id: "runenwerk.editor.workbench.editor_design".to_string(),
                    label: "Editor Design Workbench".to_string(),
                    installed_suites: vec![
                        "runenwerk.editor".to_string(),
                        "runenwerk.editor_design".to_string(),
                    ],
                    profile_refs: vec!["runenwerk.editor.workspace.editor_design".to_string()],
                    default_profile_ref: "runenwerk.editor.workspace.editor_design".to_string(),
                    host_policy: EditorWorkbenchHostPolicyDefinition::AllowAll,
                },
            ),
        ),
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.workspace.editor_design"),
            "editor_design_workspace.ron",
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceProfile(EditorWorkspaceProfileDefinition {
                id: "runenwerk.editor.workspace.editor_design".to_string(),
                label: "Editor Design".to_string(),
                default_modes: vec!["editor-design".to_string()],
                document_kind_filters: vec![
                    "UiLayout".to_string(),
                    "WorkspaceDefinition".to_string(),
                    "Theme".to_string(),
                    "Shortcut".to_string(),
                    "Menu".to_string(),
                    "CommandBinding".to_string(),
                ],
                default_layout: "runenwerk.editor.layout.editor_design".to_string(),
            }),
        ),
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.layout.editor_design"),
            "editor_design_layout.ron",
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceLayout(EditorWorkspaceLayoutDefinition {
                id: "runenwerk.editor.layout.editor_design".to_string(),
                label: "Editor Design Layout".to_string(),
                root: EditorWorkspaceHostDefinition::TabStack {
                    id: "editor-design-main".to_string(),
                    tabs: vec![
                        EditorWorkspacePanelTabDefinition {
                            id: "definition-outliner".to_string(),
                            label: "Definitions".to_string(),
                            tool_surface: "editor_design_outliner".to_string(),
                        },
                        EditorWorkspacePanelTabDefinition {
                            id: "ui-canvas".to_string(),
                            label: "Canvas".to_string(),
                            tool_surface: "ui_canvas".to_string(),
                        },
                    ],
                    active_tab: Some("definition-outliner".to_string()),
                },
                floating_hosts: Vec::new(),
            }),
        ),
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.theme.default"),
            "default_theme.ron",
            EditorDefinitionDocumentKind::Theme,
            EditorDefinitionDocumentContent::Theme(EditorThemeDefinition {
                id: "runenwerk.editor.theme.default".to_string(),
                label: "Runenwerk Default".to_string(),
                colors: BTreeMap::from([
                    ("accent".to_string(), "#5f8cff".to_string()),
                    ("background".to_string(), "#000000".to_string()),
                    ("surface".to_string(), "#050506".to_string()),
                    ("border".to_string(), "#1c1c1e".to_string()),
                ]),
                spacing: BTreeMap::from([("panel_gap".to_string(), 4.0)]),
                typography: BTreeMap::from([(
                    "body".to_string(),
                    EditorTypographyTokenDefinition {
                        font_family: "inter".to_string(),
                        size: 13.0,
                        weight: 400,
                    },
                )]),
                radius: BTreeMap::from([("control".to_string(), 0.0)]),
            }),
        ),
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.shortcuts.default"),
            "default_shortcuts.ron",
            EditorDefinitionDocumentKind::Shortcut,
            EditorDefinitionDocumentContent::Shortcuts(EditorShortcutSetDefinition {
                id: "runenwerk.editor.shortcuts.default".to_string(),
                label: "Default Shortcuts".to_string(),
                shortcuts: vec![
                    EditorShortcutDefinition {
                        id: "save_scene".to_string(),
                        command: "editor.scene.save".to_string(),
                        chord: "Cmd+S".to_string(),
                        context: Some("scene".to_string()),
                    },
                    EditorShortcutDefinition {
                        id: "apply_definition".to_string(),
                        command: "editor.definition.apply_selected".to_string(),
                        chord: "Cmd+Shift+A".to_string(),
                        context: Some("editor-design".to_string()),
                    },
                ],
            }),
        ),
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.menu.default"),
            "default_menu.ron",
            EditorDefinitionDocumentKind::Menu,
            EditorDefinitionDocumentContent::Menu(EditorMenuDefinition {
                id: "runenwerk.editor.menu.default".to_string(),
                label: "Runenwerk".to_string(),
                items: vec![EditorMenuItemDefinition {
                    id: "apply_definition".to_string(),
                    label: "Apply Definition".to_string(),
                    command: Some("editor.definition.apply_selected".to_string()),
                    children: Vec::new(),
                    availability: Some("editor-design.definition-selected".to_string()),
                }],
            }),
        ),
        EditorDefinitionDocument::current(
            EditorDefinitionId::from("runenwerk.editor.commands.default"),
            "default_command_bindings.ron",
            EditorDefinitionDocumentKind::CommandBinding,
            EditorDefinitionDocumentContent::CommandBindings(EditorCommandBindingSetDefinition {
                id: "runenwerk.editor.commands.default".to_string(),
                label: "Default Command Bindings".to_string(),
                bindings: vec![EditorCommandBindingDefinition {
                    id: "apply_selected_definition".to_string(),
                    command: "editor.definition.apply_selected".to_string(),
                    route_target: "self-authoring.apply-selected".to_string(),
                    capability_requirements: vec!["ratify".to_string()],
                    undoable: true,
                }],
            }),
        ),
    ]
}

fn default_custom_workbench_layout(
    layout_id: &str,
    package_index: usize,
) -> EditorWorkspaceLayoutDefinition {
    EditorWorkspaceLayoutDefinition {
        id: layout_id.to_string(),
        label: format!("Custom {package_index} Layout"),
        root: EditorWorkspaceHostDefinition::TabStack {
            id: format!("custom-{package_index}-main"),
            tabs: vec![
                EditorWorkspacePanelTabDefinition {
                    id: "definition-outliner".to_string(),
                    label: "Definitions".to_string(),
                    tool_surface: "editor_design_outliner".to_string(),
                },
                EditorWorkspacePanelTabDefinition {
                    id: "ui-canvas".to_string(),
                    label: "Canvas".to_string(),
                    tool_surface: "ui_canvas".to_string(),
                },
            ],
            active_tab: Some("definition-outliner".to_string()),
        },
        floating_hosts: Vec::new(),
    }
}

fn workbench_composition_package_documents(
    composition: &EditorWorkbenchCompositionDefinition,
    profiles: &[EditorWorkspaceProfileDefinition],
    layouts: &[EditorWorkspaceLayoutDefinition],
) -> Vec<EditorDefinitionDocument> {
    let mut documents = Vec::with_capacity(1 + profiles.len() + layouts.len());
    documents.push(EditorDefinitionDocument::current(
        EditorDefinitionId::from(composition.id.as_str()),
        composition.label.clone(),
        EditorDefinitionDocumentKind::WorkbenchComposition,
        EditorDefinitionDocumentContent::WorkbenchComposition(composition.clone()),
    ));
    documents.extend(profiles.iter().map(|profile| {
        EditorDefinitionDocument::current(
            EditorDefinitionId::from(profile.id.as_str()),
            profile.label.clone(),
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceProfile(profile.clone()),
        )
    }));
    documents.extend(layouts.iter().map(|layout| {
        EditorDefinitionDocument::current(
            EditorDefinitionId::from(layout.id.as_str()),
            layout.label.clone(),
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceLayout(layout.clone()),
        )
    }));
    documents
}

fn selected_ui_default_node_id(
    drafts: &BTreeMap<EditorDefinitionId, EditorDefinitionDocument>,
    document_id: &EditorDefinitionId,
) -> Option<String> {
    drafts
        .get(document_id)
        .and_then(selected_ui_default_node_for_document)
}

fn selected_ui_default_node_for_document(document: &EditorDefinitionDocument) -> Option<String> {
    match &document.content {
        EditorDefinitionDocumentContent::UiTemplate(template) => {
            first_text_editable_ui_node_id(&template.root)
                .or_else(|| Some(template.root.id().as_str().to_string()))
        }
        _ => None,
    }
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

fn ui_node_exists(node: &ui_definition::UiNodeDefinition, node_id: &str) -> bool {
    node.id().as_str() == node_id
        || node
            .children()
            .iter()
            .any(|child| ui_node_exists(child, node_id))
}

fn set_ui_node_text(
    node: &mut ui_definition::UiNodeDefinition,
    node_id: &str,
    text: String,
) -> Option<()> {
    if node.id().as_str() == node_id {
        return match node {
            ui_definition::UiNodeDefinition::Label { label, .. }
            | ui_definition::UiNodeDefinition::Button { label, .. }
            | ui_definition::UiNodeDefinition::Toggle { label, .. } => {
                *label = ui_definition::UiValueBinding::static_text(text);
                Some(())
            }
            ui_definition::UiNodeDefinition::TextInput { value, .. } => {
                *value = ui_definition::UiValueBinding::static_text(text);
                Some(())
            }
            _ => None,
        };
    }

    match node {
        ui_definition::UiNodeDefinition::Panel { children, .. }
        | ui_definition::UiNodeDefinition::Row { children, .. }
        | ui_definition::UiNodeDefinition::Column { children, .. }
        | ui_definition::UiNodeDefinition::Stack { children, .. }
        | ui_definition::UiNodeDefinition::Scroll { children, .. }
        | ui_definition::UiNodeDefinition::Split { children, .. }
        | ui_definition::UiNodeDefinition::Control { children, .. } => {
            for child in children {
                if set_ui_node_text(child, node_id, text.clone()).is_some() {
                    return Some(());
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_ui_fixtures_load_as_editable_definition_documents() {
        let state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");

        assert!(state.draft_documents().count() >= EDITOR_UI_ASSET_SOURCES.len());
        assert!(
            state
                .draft_documents()
                .any(|document| document.display_name == "toolbar.ron")
        );
    }

    #[test]
    fn selected_ui_definition_forms_retained_preview() {
        let state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");

        let preview = state.formed_selected_preview(&ThemeTokens::default());

        assert!(preview.is_some());
        assert!(state.selected_ui_node_id().is_some());
    }

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
    fn editor_lab_project_package_round_trips_and_preserves_invalid_input() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let selected = state
            .selected_document_id()
            .expect("selected document should exist")
            .clone();

        state
            .apply_selected()
            .expect("selected fixture should create an applied snapshot");
        let saved = state
            .save_project_package_to_ron()
            .expect("project package should serialize");
        assert!(saved.contains(crate::shell::EDITOR_LAB_PROJECT_PACKAGE_KIND));
        assert!(state.last_saved_project_package_source().is_some());

        let path = std::env::temp_dir().join("runenwerk-editor-lab-package-round-trip.ron");
        let report = state
            .save_project_package_to_path(&path)
            .expect("project package should write to an app-owned store path");
        assert!(report.source_bytes > 0);

        let mut loaded =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let source = std::fs::read_to_string(path).expect("test package file should be readable");
        let load_report = loaded
            .load_project_package_from_ron(&source)
            .expect("saved package should reload");
        assert!(load_report.draft_count >= EDITOR_UI_ASSET_SOURCES.len());
        assert_eq!(load_report.applied_count, 1);
        assert_eq!(load_report.last_applied_count, 1);
        assert!(loaded.applied_document(&selected).is_some());

        let invalid = "not a valid Editor Lab project package";
        assert!(loaded.load_project_package_from_ron(invalid).is_err());
        assert_eq!(loaded.last_invalid_project_package_source(), Some(invalid));
        assert_eq!(loaded.last_invalid_project_package_diagnostics().len(), 1);
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
    fn create_duplicate_rename_delete_import_and_export_are_explicit_document_flows() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");

        let duplicate_id = state
            .duplicate_selected(
                EditorDefinitionId::from("runenwerk.editor.toolbar.copy"),
                "Copy",
            )
            .expect("selected template should duplicate");
        state
            .rename_selected("Renamed Copy")
            .expect("selected duplicate should rename");
        let exported = state
            .export_selected_to_ron()
            .expect("selected duplicate should export");
        assert!(exported.contains("Renamed Copy"));
        let exported_package: EditorDefinitionExportPackage =
            ron::from_str(&exported).expect("export should be a versioned package");
        assert_eq!(
            exported_package.package_version,
            EDITOR_DEFINITION_EXPORT_PACKAGE_VERSION
        );
        assert_eq!(
            exported_package.package_kind,
            EDITOR_DEFINITION_EXPORT_PACKAGE_KIND
        );
        assert_eq!(exported_package.document.display_name, "Renamed Copy");
        let import_report = state
            .import_selected_package_from_ron(&exported)
            .expect("selected definition package should import explicitly");
        assert!(import_report.replaced_existing);
        assert_eq!(import_report.display_name, "Renamed Copy");
        let removed = state
            .delete_selected()
            .expect("unapplied duplicate should delete");
        assert_eq!(removed.id, duplicate_id);

        let imported = VersionedAuthoredUiTemplate::current(
            ui_definition::AuthoredUiDefinitionCategory::Fixture,
            AuthoredUiTemplate {
                id: "runenwerk.editor.imported".into(),
                root: ui_definition::UiNodeDefinition::Label {
                    id: "root".into(),
                    label: ui_definition::UiValueBinding::static_text("Imported"),
                    availability: None,
                },
                templates: Vec::new(),
                menus: Vec::new(),
            },
        );
        let source = ron::to_string(&imported).expect("import fixture should serialize");
        let imported_id = state
            .import_versioned_ui_template_document(&source, "Imported")
            .expect("current versioned UI definition should import");

        assert!(state.select_document(imported_id));
    }

    #[test]
    fn retained_ui_node_and_theme_edits_stay_in_draft_documents() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let node_id = state
            .selected_ui_node_id()
            .expect("selected UI fixture should expose a root node")
            .to_string();

        state
            .set_selected_ui_node_text(&node_id, "Edited")
            .expect("selected UI text should edit");
        let preview = state
            .formed_selected_preview(&ThemeTokens::default())
            .expect("edited UI definition should still preview");
        assert!(preview.diagnostics.is_empty());

        assert!(state.select_document_by_str("runenwerk.editor.theme.default"));
        state
            .set_selected_theme_color("accent", "#3366ff")
            .expect("theme color token should edit");
        let selected = state
            .selected_document()
            .expect("theme document should be selected");
        let EditorDefinitionDocumentContent::Theme(theme) = &selected.content else {
            panic!("selected document should be a theme definition");
        };
        assert_eq!(
            theme.colors.get("accent").map(String::as_str),
            Some("#3366ff")
        );
    }

    #[test]
    fn recipe_insertion_updates_source_versioned_draft_and_history() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let library = editor_shell::editor_design_system_recipe_library();
        let before_source_version = state
            .selected_source_version_label()
            .expect("default UI definition should expose a source version");

        let report = state
            .insert_selected_ui_recipe(
                &library,
                ui_definition::UiRecipeId::new(editor_shell::EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID),
                ui_definition::UiRecipeTargetProfileId::new(
                    editor_shell::UI_DESIGNER_WORKBENCH_TARGET_PROFILE,
                ),
            )
            .expect("primary button recipe should insert into selected UI template");

        let inserted_node = state
            .selected_ui_node_id()
            .expect("inserted recipe root should become selected")
            .to_string();
        assert!(inserted_node.contains("editor.pattern.primary_button.primary-button"));
        assert_eq!(report.status, EditorLabOperationStatus::Accepted);
        assert_eq!(state.operation_history_snapshot().undo_count, 1);
        assert_eq!(
            state
                .last_operation_report()
                .expect("recipe insertion should record a report")
                .operation_id,
            report.operation_id
        );
        let after_source_version = state
            .selected_source_version_label()
            .expect("inserted UI definition should expose a source version");
        assert_ne!(before_source_version, after_source_version);
        assert!(after_source_version.contains(":epoch"));
        assert!(after_source_version.contains("blake3:"));

        let selected = state
            .selected_document()
            .expect("recipe insertion should keep the document selected");
        let EditorDefinitionDocumentContent::UiTemplate(template) = &selected.content else {
            panic!("selected document should remain a UI template");
        };
        assert!(ui_node_exists(&template.root, &inserted_node));
        let diff = report
            .diff
            .expect("accepted insertion should produce a diff");
        assert_eq!(diff.changes.len(), 1);
        assert_eq!(
            diff.changes[0].family,
            EditorLabOperationDiffFamily::UiVisualLayout
        );
        assert!(
            diff.changes[0]
                .after
                .as_deref()
                .is_some_and(|after| after.contains("Primary action"))
        );

        state
            .undo_editor_lab_operation()
            .expect("recipe insertion should be undoable");
        let undo_document = state
            .selected_document()
            .expect("undo should keep the UI document selected");
        let EditorDefinitionDocumentContent::UiTemplate(undo_template) = &undo_document.content
        else {
            panic!("undo should keep the selected document as a UI template");
        };
        assert!(!ui_node_exists(&undo_template.root, &inserted_node));
        assert_eq!(state.operation_history_snapshot().redo_count, 1);

        state
            .redo_editor_lab_operation()
            .expect("recipe insertion should be redoable");
        let redo_document = state
            .selected_document()
            .expect("redo should keep the UI document selected");
        let EditorDefinitionDocumentContent::UiTemplate(redo_template) = &redo_document.content
        else {
            panic!("redo should keep the selected document as a UI template");
        };
        assert!(ui_node_exists(&redo_template.root, &inserted_node));
        assert_eq!(state.operation_history_snapshot().undo_count, 1);
    }

    #[test]
    fn source_revision_changes_and_evidence_invalidates_on_mutations() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let theme = ThemeTokens::default();
        let library = editor_shell::editor_design_system_recipe_library();

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("initial evidence capture should validate");
        let before_revision = state
            .selected_source_revision()
            .expect("selected document should expose source revision");
        assert_eq!(state.last_scenario_evidence_packets().len(), 2);

        state
            .rename_selected("toolbar source revision test")
            .expect("rename should mutate selected document");
        let after_rename = state
            .selected_source_revision()
            .expect("renamed document should expose source revision");
        assert_ne!(before_revision, after_rename);
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture after rename should validate");
        state
            .insert_selected_ui_recipe(
                &library,
                ui_definition::UiRecipeId::new(editor_shell::EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID),
                ui_definition::UiRecipeTargetProfileId::new(
                    editor_shell::UI_DESIGNER_WORKBENCH_TARGET_PROFILE,
                ),
            )
            .expect("recipe insertion should mutate selected document");
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture after insert should validate");
        state
            .undo_editor_lab_operation()
            .expect("undo should mutate selected document");
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture after undo should validate");
        state
            .redo_editor_lab_operation()
            .expect("redo should mutate selected document");
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture before apply should validate");
        state
            .apply_selected()
            .expect("apply should mutate applied source state");
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture before rollback should validate");
        state
            .rollback_selected()
            .expect("rollback should mutate applied source state");
        assert!(state.last_scenario_evidence_packets().is_empty());

        let saved = state
            .save_project_package_to_ron()
            .expect("project package should save");
        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture before project load should validate");
        state
            .load_project_package_from_ron(&saved)
            .expect("project package load should mutate session source state");
        assert!(state.last_scenario_evidence_packets().is_empty());
    }

    #[test]
    fn recipe_insertion_rejects_incompatible_target_without_mutating_history() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let library = editor_shell::editor_design_system_recipe_library();
        let before_history = state.operation_history_snapshot();
        let before_document = operation_history_document_snapshot(
            state
                .selected_document()
                .expect("default UI definition should be selected"),
        );

        let diagnostic = state
            .insert_selected_ui_recipe(
                &library,
                ui_definition::UiRecipeId::new(editor_shell::EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID),
                ui_definition::UiRecipeTargetProfileId::new(
                    game_runtime::GAME_RUNTIME_TARGET_PROFILE,
                ),
            )
            .expect_err("editor recipe should be rejected in the game runtime target");

        assert_eq!(diagnostic.code, "ui.recipe.target_profile.unsupported");
        assert_eq!(state.operation_history_snapshot(), before_history);
        let after_document = operation_history_document_snapshot(
            state
                .selected_document()
                .expect("rejected recipe insertion should preserve selection"),
        );
        assert_eq!(after_document, before_document);
        let report = state
            .last_operation_report()
            .expect("rejected recipe insertion should record diagnostics");
        assert_eq!(report.status, EditorLabOperationStatus::Rejected);
        assert!(report.diff.is_none());
        assert_eq!(
            report
                .diagnostics
                .first()
                .map(|diagnostic| diagnostic.code.as_str()),
            Some("ui.recipe.target_profile.unsupported")
        );
    }

    #[test]
    fn authored_workspace_layout_tabs_and_splits_are_draft_edits() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        assert!(state.select_document_by_str("runenwerk.editor.layout.editor_design"));

        let tab_id = state
            .add_selected_workspace_layout_tab("Validation", "definition_validation")
            .expect("layout tab should be added");
        assert_eq!(tab_id, "authored-tab-3");

        state
            .split_selected_workspace_layout_root(EditorWorkspaceSplitAxisDefinition::Horizontal)
            .expect("layout root should split");

        let selected = state
            .selected_document()
            .expect("layout document should stay selected");
        let EditorDefinitionDocumentContent::WorkspaceLayout(layout) = &selected.content else {
            panic!("selected document should be a workspace layout");
        };
        assert!(matches!(
            layout.root,
            EditorWorkspaceHostDefinition::Split { .. }
        ));
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
