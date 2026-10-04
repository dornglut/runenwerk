//! Multi-document workbench package creation, resolution, and formation ownership.

use super::*;
use std::collections::BTreeSet;

impl SelfAuthoringWorkspaceState {
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

    pub(super) fn selected_workbench_composition_package_documents(
        &self,
    ) -> Result<Vec<EditorDefinitionDocument>, UiDefinitionDiagnostic> {
        let (composition, profiles, layouts) = self.selected_workbench_composition_package()?;
        Ok(workbench_composition_package_documents(
            &composition,
            &profiles,
            &layouts,
        ))
    }
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

pub(super) fn workbench_composition_package_documents(
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
