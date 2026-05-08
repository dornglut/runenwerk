//! File: apps/runenwerk_editor/src/shell/self_authoring.rs
//! Purpose: App-owned UI/editor definition authoring state, preview, apply, and rollback.

use anyhow::{Context, Result};
use editor_definition::{
    EditorCommandBindingDefinition, EditorCommandBindingSetDefinition, EditorDefinitionDocument,
    EditorDefinitionDocumentContent, EditorDefinitionDocumentKind, EditorDefinitionId,
    EditorDefinitionLifecycleState, EditorMenuDefinition, EditorMenuItemDefinition,
    EditorShortcutDefinition, EditorShortcutSetDefinition, EditorThemeDefinition,
    EditorTypographyTokenDefinition, EditorWorkspaceHostDefinition,
    EditorWorkspaceLayoutDefinition, EditorWorkspacePanelTabDefinition,
    EditorWorkspaceProfileDefinition, EditorWorkspaceSplitAxisDefinition,
    editor_definition_has_blocking_diagnostics, validate_editor_definition_document,
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ui_definition::{
    AuthoredUiTemplate, FormedRetainedUiProduct, UiDefinitionContext, UiDefinitionDiagnostic,
    UiDefinitionDiagnosticSeverity, VersionedAuthoredUiTemplate, migrate_authored_ui_template,
    normalize_authored_template,
};
use ui_theme::ThemeTokens;

use crate::shell::ui_definition_assets::{EDITOR_BINDINGS_SOURCE, EDITOR_UI_ASSET_SOURCES};

pub const EDITOR_DEFINITION_EXPORT_PACKAGE_VERSION: u32 = 1;
pub const EDITOR_DEFINITION_EXPORT_PACKAGE_KIND: &str = "runenwerk.editor.definition.export";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorDefinitionExportPackage {
    pub package_version: u32,
    pub package_kind: String,
    pub document: EditorDefinitionDocument,
}

impl EditorDefinitionExportPackage {
    pub fn current(document: EditorDefinitionDocument) -> Self {
        Self {
            package_version: EDITOR_DEFINITION_EXPORT_PACKAGE_VERSION,
            package_kind: EDITOR_DEFINITION_EXPORT_PACKAGE_KIND.to_string(),
            document,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefinitionApplyPreview {
    pub document_id: EditorDefinitionId,
    pub display_name: String,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
    pub summary: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SelfAuthoringWorkspaceState {
    drafts: BTreeMap<EditorDefinitionId, EditorDefinitionDocument>,
    applied: BTreeMap<EditorDefinitionId, EditorDefinitionDocument>,
    selected_document_id: Option<EditorDefinitionId>,
    selected_ui_node_id: Option<String>,
    last_apply_preview: Option<DefinitionApplyPreview>,
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
        })
    }

    pub fn draft_documents(&self) -> impl Iterator<Item = &EditorDefinitionDocument> {
        self.drafts.values()
    }

    pub fn selected_document_id(&self) -> Option<&EditorDefinitionId> {
        self.selected_document_id.as_ref()
    }

    pub fn selected_document(&self) -> Option<&EditorDefinitionDocument> {
        self.selected_document_id
            .as_ref()
            .and_then(|id| self.drafts.get(id))
    }

    pub fn selected_ui_node_id(&self) -> Option<&str> {
        self.selected_ui_node_id.as_deref()
    }

    pub fn select_document(&mut self, document_id: EditorDefinitionId) -> bool {
        if !self.drafts.contains_key(&document_id) {
            return false;
        }
        self.selected_ui_node_id = selected_ui_default_node_id(&self.drafts, &document_id);
        self.selected_document_id = Some(document_id);
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
        Ok(())
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
        self.selected_document_id = self.drafts.keys().next().cloned();
        self.selected_ui_node_id = self
            .selected_document_id
            .as_ref()
            .and_then(|id| selected_ui_default_node_id(&self.drafts, id));
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
        })
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

    pub fn export_selected_to_ron(&self) -> Result<String, UiDefinitionDiagnostic> {
        let package = self.export_selected_package()?;
        ron::ser::to_string_pretty(&package, PrettyConfig::new()).map_err(|error| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.export.serialize_failed",
                format!("failed to export definition package: {error}"),
            )
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

    pub fn last_apply_preview(&self) -> Option<&DefinitionApplyPreview> {
        self.last_apply_preview.as_ref()
    }

    pub fn apply_selected(&mut self) -> Result<DefinitionApplyPreview, UiDefinitionDiagnostic> {
        let preview = self.build_apply_preview().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.apply.no_selection",
                "no definition document is selected",
            )
        })?;
        if editor_definition_has_blocking_diagnostics(&preview.diagnostics) {
            self.last_apply_preview = Some(preview.clone());
            return Err(UiDefinitionDiagnostic::error(
                "editor.self_authoring.apply.blocked",
                "definition has blocking validation diagnostics",
            ));
        }
        let mut applied = self
            .drafts
            .get(&preview.document_id)
            .cloned()
            .ok_or_else(|| {
                UiDefinitionDiagnostic::error(
                    "editor.self_authoring.apply.unresolved",
                    "selected definition document is not loaded",
                )
            })?;
        applied.lifecycle_state = EditorDefinitionLifecycleState::Applied;
        self.applied.insert(preview.document_id.clone(), applied);
        self.last_apply_preview = Some(preview.clone());
        Ok(preview)
    }

    pub fn rollback_selected(
        &mut self,
    ) -> Result<EditorDefinitionDocument, UiDefinitionDiagnostic> {
        let document_id = self.selected_document_id.clone().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.rollback.no_selection",
                "no definition document is selected",
            )
        })?;
        let mut rolled_back = self.applied.remove(&document_id).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.rollback.no_applied_snapshot",
                "selected definition has no applied snapshot",
            )
        })?;
        rolled_back.lifecycle_state = EditorDefinitionLifecycleState::RolledBack;
        Ok(rolled_back)
    }

    pub fn applied_document(&self, id: &EditorDefinitionId) -> Option<&EditorDefinitionDocument> {
        self.applied.get(id)
    }

    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }
}

fn default_editor_definition_documents() -> Vec<EditorDefinitionDocument> {
    vec![
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

fn first_text_editable_ui_node_id(node: &ui_definition::UiNodeDefinition) -> Option<String> {
    match node {
        ui_definition::UiNodeDefinition::Label { id, .. }
        | ui_definition::UiNodeDefinition::Button { id, .. }
        | ui_definition::UiNodeDefinition::Toggle { id, .. }
        | ui_definition::UiNodeDefinition::TextInput { id, .. } => {
            return Some(id.as_str().to_string());
        }
        _ => {}
    }

    node.children()
        .iter()
        .find_map(first_text_editable_ui_node_id)
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
        | ui_definition::UiNodeDefinition::Split { children, .. } => {
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
