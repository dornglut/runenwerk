//! File: apps/runenwerk_editor/src/shell/self_authoring/mod.rs
//! Purpose: App-owned UI/editor definition authoring state, preview, apply, and rollback.

mod apply;
mod documents;
mod evidence;
mod operations;
mod project_io;
mod recipes;
mod session;
mod workbench;

pub use apply::DefinitionApplyPreview;
pub use documents::{
    EDITOR_DEFINITION_EXPORT_PACKAGE_KIND, EDITOR_DEFINITION_EXPORT_PACKAGE_VERSION,
    EditorDefinitionExportPackage,
};
pub use evidence::EditorLabProductPathEvidenceCapture;
use operations::*;
use session::EditorLabOperationHistory;
#[cfg(test)]
use session::operation_history_document_snapshot;
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
use std::{collections::BTreeMap, time::Instant};
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

    fn record_source_change(&mut self) {
        self.source_revision_epoch = self.source_revision_epoch.saturating_add(1);
        self.last_scenario_evidence_packets.clear();
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
}
