use editor_definition::prelude::*;
use ui_definition::prelude::{AuthoredUiTemplate, UiNodeDefinition, UiValueBinding};

fn main() {
    let document = new_editor_definition_document(
        "editor.lab.template",
        "Editor Lab Template",
        EditorDefinitionDocumentKind::UiLayout,
        EditorDefinitionDocumentContent::UiTemplate(template()),
    );

    let diagnostics = validate_editor_document(&document);
    assert!(
        !editor_document_has_blocking_diagnostics(&diagnostics),
        "{diagnostics:?}"
    );

    let operation = EditorLabOperation {
        id: "title.rename".to_string(),
        document_id: document.id.clone(),
        target_profile: "editor.workbench".to_string(),
        kind: EditorLabOperationKind::SetUiNodeText {
            node_id: "title".to_string(),
            text: "Runtime-Proven Editor Lab".to_string(),
        },
        preview_only: false,
        source: Some("example".to_string()),
    };

    let report = apply_editor_lab_edit(&document, &operation);

    assert!(report.accepted(), "{:?}", report.diagnostics);
    assert!(report.diff.is_some());
}

fn template() -> AuthoredUiTemplate {
    AuthoredUiTemplate {
        id: "editor.lab.template".into(),
        root: UiNodeDefinition::Column {
            id: "root".into(),
            children: vec![UiNodeDefinition::Label {
                id: "title".into(),
                label: UiValueBinding::static_text("Editor Lab"),
                availability: None,
            }],
        },
        templates: Vec::new(),
        menus: Vec::new(),
    }
}
