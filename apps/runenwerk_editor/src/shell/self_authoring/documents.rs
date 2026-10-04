//! Single-definition catalog, CRUD, and versioned import/export ownership for Editor Lab.

use super::*;

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

impl SelfAuthoringWorkspaceState {
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

    pub fn export_selected_to_ron(&self) -> Result<String, UiDefinitionDiagnostic> {
        let package = self.export_selected_package()?;
        ron::ser::to_string_pretty(&package, PrettyConfig::new()).map_err(|error| {
            UiDefinitionDiagnostic::error(
                "editor.self_authoring.export.serialize_failed",
                format!("failed to export definition package: {error}"),
            )
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
