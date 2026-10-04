//! Project package and apply-preview DTOs for UI Designer self-authoring.

use super::*;

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

impl SelfAuthoringWorkspaceState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
