//! App-owned Editor Lab project IO, apply review, activation, and rollback contracts.

use editor_definition::{EditorDefinitionDocument, EditorDefinitionId};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use ui_definition::UiDefinitionDiagnostic;

pub const EDITOR_LAB_PROJECT_PACKAGE_VERSION: u32 = 1;
pub const EDITOR_LAB_PROJECT_PACKAGE_KIND: &str = "runenwerk.editor.lab.project";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorLabProjectPackage {
    pub package_version: u32,
    pub package_kind: String,
    pub draft_documents: Vec<EditorDefinitionDocument>,
    pub applied_documents: Vec<EditorDefinitionDocument>,
    pub last_applied_documents: Vec<EditorDefinitionDocument>,
}

impl EditorLabProjectPackage {
    pub fn current(
        draft_documents: impl IntoIterator<Item = EditorDefinitionDocument>,
        applied_documents: impl IntoIterator<Item = EditorDefinitionDocument>,
        last_applied_documents: impl IntoIterator<Item = EditorDefinitionDocument>,
    ) -> Self {
        Self {
            package_version: EDITOR_LAB_PROJECT_PACKAGE_VERSION,
            package_kind: EDITOR_LAB_PROJECT_PACKAGE_KIND.to_string(),
            draft_documents: sorted_documents(draft_documents),
            applied_documents: sorted_documents(applied_documents),
            last_applied_documents: sorted_documents(last_applied_documents),
        }
    }

    pub fn validate(&self) -> Result<(), UiDefinitionDiagnostic> {
        if self.package_version != EDITOR_LAB_PROJECT_PACKAGE_VERSION {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.project.package.unsupported_version",
                format!(
                    "unsupported Editor Lab project package version {}",
                    self.package_version
                ),
            ));
        }
        if self.package_kind != EDITOR_LAB_PROJECT_PACKAGE_KIND {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.project.package.unsupported_kind",
                format!(
                    "unsupported Editor Lab project package kind '{}'",
                    self.package_kind
                ),
            ));
        }
        reject_duplicate_ids("draft_documents", &self.draft_documents)?;
        reject_duplicate_ids("applied_documents", &self.applied_documents)?;
        reject_duplicate_ids("last_applied_documents", &self.last_applied_documents)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditorLabDocumentStore {
    last_saved_package_source: Option<String>,
    last_loaded_package_source: Option<String>,
    last_invalid_package_source: Option<String>,
    last_invalid_package_diagnostics: Vec<UiDefinitionDiagnostic>,
}

impl EditorLabDocumentStore {
    pub fn last_saved_package_source(&self) -> Option<&str> {
        self.last_saved_package_source.as_deref()
    }

    pub fn last_loaded_package_source(&self) -> Option<&str> {
        self.last_loaded_package_source.as_deref()
    }

    pub fn last_invalid_package_source(&self) -> Option<&str> {
        self.last_invalid_package_source.as_deref()
    }

    pub fn last_invalid_package_diagnostics(&self) -> &[UiDefinitionDiagnostic] {
        &self.last_invalid_package_diagnostics
    }

    pub fn save_package_source(
        &mut self,
        package: &EditorLabProjectPackage,
    ) -> Result<String, UiDefinitionDiagnostic> {
        let source = serialize_editor_lab_project_package(package)?;
        self.last_saved_package_source = Some(source.clone());
        Ok(source)
    }

    pub fn save_package_to_path(
        &mut self,
        package: &EditorLabProjectPackage,
        path: impl AsRef<Path>,
    ) -> Result<EditorLabProjectStoreReport, UiDefinitionDiagnostic> {
        let source = self.save_package_source(package)?;
        std::fs::write(path.as_ref(), &source).map_err(|error| {
            UiDefinitionDiagnostic::error(
                "editor.lab.project.store.write_failed",
                format!(
                    "failed to write Editor Lab project package '{}': {error}",
                    path.as_ref().display()
                ),
            )
        })?;
        Ok(EditorLabProjectStoreReport {
            source_bytes: source.len(),
            draft_count: package.draft_documents.len(),
            applied_count: package.applied_documents.len(),
            last_applied_count: package.last_applied_documents.len(),
        })
    }

    pub fn load_package_source(
        &mut self,
        source: &str,
    ) -> Result<EditorLabProjectPackage, UiDefinitionDiagnostic> {
        match deserialize_editor_lab_project_package(source) {
            Ok(package) => {
                self.last_loaded_package_source = Some(source.to_string());
                self.last_invalid_package_source = None;
                self.last_invalid_package_diagnostics.clear();
                Ok(package)
            }
            Err(diagnostic) => {
                self.last_invalid_package_source = Some(source.to_string());
                self.last_invalid_package_diagnostics = vec![diagnostic.clone()];
                Err(diagnostic)
            }
        }
    }

    pub fn load_package_from_path(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<EditorLabProjectPackage, UiDefinitionDiagnostic> {
        let source = std::fs::read_to_string(path.as_ref()).map_err(|error| {
            UiDefinitionDiagnostic::error(
                "editor.lab.project.store.read_failed",
                format!(
                    "failed to read Editor Lab project package '{}': {error}",
                    path.as_ref().display()
                ),
            )
        })?;
        self.load_package_source(&source)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorLabProjectStoreReport {
    pub source_bytes: usize,
    pub draft_count: usize,
    pub applied_count: usize,
    pub last_applied_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorLabProjectLoadReport {
    pub draft_count: usize,
    pub applied_count: usize,
    pub last_applied_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabProjectImportReport {
    pub document_id: EditorDefinitionId,
    pub display_name: String,
    pub replaced_existing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionApplyReview {
    pub id: String,
    pub document_id: EditorDefinitionId,
    pub display_name: String,
    pub status: DefinitionApplyReviewStatus,
    pub draft_snapshot: EditorDefinitionDocument,
    pub applied_before: Option<EditorDefinitionDocument>,
    pub proposed_applied_snapshot: EditorDefinitionDocument,
    pub diff_rows: Vec<DefinitionApplyDiffRow>,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
    pub rollback_target_available: bool,
}

impl DefinitionApplyReview {
    pub fn with_status(mut self, status: DefinitionApplyReviewStatus) -> Self {
        self.status = status;
        self
    }

    pub fn has_blocking_diagnostics(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == ui_definition::UiDefinitionDiagnosticSeverity::Error
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefinitionApplyReviewStatus {
    Pending,
    Rejected,
    Accepted,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionApplyDiffRow {
    pub path: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingEditorDefinitionActivation {
    pub review_id: Option<String>,
    pub document: EditorDefinitionDocument,
}

impl PendingEditorDefinitionActivation {
    pub fn new(review_id: Option<String>, document: EditorDefinitionDocument) -> Self {
        Self {
            review_id,
            document,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorDefinitionActivationReport {
    pub id: String,
    pub review_id: Option<String>,
    pub document_id: EditorDefinitionId,
    pub display_name: String,
    pub status: EditorDefinitionActivationStatus,
    pub summary: Vec<String>,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
    pub previous_state_preserved: bool,
}

impl EditorDefinitionActivationReport {
    pub fn from_request(
        request: &PendingEditorDefinitionActivation,
        status: EditorDefinitionActivationStatus,
        summary: Vec<String>,
        diagnostics: Vec<UiDefinitionDiagnostic>,
        previous_state_preserved: bool,
    ) -> Self {
        let status_key = match status {
            EditorDefinitionActivationStatus::Queued => "queued",
            EditorDefinitionActivationStatus::Applied => "applied",
            EditorDefinitionActivationStatus::NoLiveActivation => "no-live-activation",
            EditorDefinitionActivationStatus::Failed => "failed",
            EditorDefinitionActivationStatus::Rejected => "rejected",
        };
        Self {
            id: format!(
                "editor-lab.activation.{status_key}.{}",
                request.document.id.as_str()
            ),
            review_id: request.review_id.clone(),
            document_id: request.document.id.clone(),
            display_name: request.document.display_name.clone(),
            status,
            summary,
            diagnostics,
            previous_state_preserved,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorDefinitionActivationStatus {
    Queued,
    Applied,
    NoLiveActivation,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorLabRollbackRecord {
    pub id: String,
    pub document_id: EditorDefinitionId,
    pub display_name: String,
    pub status: EditorLabRollbackStatus,
    pub removed_document: Option<EditorDefinitionDocument>,
    pub restored_document: Option<EditorDefinitionDocument>,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLabRollbackStatus {
    RolledBack,
    Unavailable,
}

pub fn serialize_editor_lab_project_package(
    package: &EditorLabProjectPackage,
) -> Result<String, UiDefinitionDiagnostic> {
    package.validate()?;
    ron::ser::to_string_pretty(package, PrettyConfig::new()).map_err(|error| {
        UiDefinitionDiagnostic::error(
            "editor.lab.project.package.serialize_failed",
            format!("failed to serialize Editor Lab project package: {error}"),
        )
    })
}

pub fn deserialize_editor_lab_project_package(
    source: &str,
) -> Result<EditorLabProjectPackage, UiDefinitionDiagnostic> {
    let package: EditorLabProjectPackage = ron::from_str(source).map_err(|error| {
        UiDefinitionDiagnostic::error(
            "editor.lab.project.package.parse_failed",
            format!("failed to parse Editor Lab project package: {error}"),
        )
    })?;
    package.validate()?;
    Ok(package)
}

pub fn editor_lab_document_source(
    document: &EditorDefinitionDocument,
) -> Result<String, UiDefinitionDiagnostic> {
    ron::ser::to_string_pretty(document, PrettyConfig::new()).map_err(|error| {
        UiDefinitionDiagnostic::error(
            "editor.lab.project.document.serialize_failed",
            format!("failed to serialize Editor Lab definition document: {error}"),
        )
    })
}

fn sorted_documents(
    documents: impl IntoIterator<Item = EditorDefinitionDocument>,
) -> Vec<EditorDefinitionDocument> {
    let mut documents = documents.into_iter().collect::<Vec<_>>();
    documents.sort_by(|left, right| left.id.cmp(&right.id));
    documents
}

fn reject_duplicate_ids(
    field: &str,
    documents: &[EditorDefinitionDocument],
) -> Result<(), UiDefinitionDiagnostic> {
    let mut ids = BTreeSet::new();
    for document in documents {
        if !ids.insert(document.id.clone()) {
            return Err(UiDefinitionDiagnostic::error(
                "editor.lab.project.package.duplicate_document",
                format!(
                    "Editor Lab project package field '{field}' contains duplicate document id '{}'",
                    document.id.as_str()
                ),
            ));
        }
    }
    Ok(())
}
