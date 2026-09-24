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
