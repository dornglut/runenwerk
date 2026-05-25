//! File: domain/editor/editor_shell/src/story_lab/workbench.rs
//! Purpose: UI Designer workbench Story Lab contracts and fixture evidence.

use crate::{
    EditorDefinitionSurfaceAction, EditorLabActionViewModel, EditorLabDiagnosticViewModel,
    UiDesignerWorkbenchPaneKind, UiDesignerWorkbenchPaneViewModel,
    UiDesignerWorkbenchReadinessStatus, UiDesignerWorkbenchReadinessViewModel,
    UiDesignerWorkbenchViewModel, VisibleWidgetState,
};

pub const UI_DESIGNER_WORKBENCH_STORY_ID: &str = "editor.ui_designer.workbench.standalone";
pub const UI_DESIGNER_WORKBENCH_TARGET_PROFILE: &str = "editor.workbench";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorUxWorkbenchEvidence {
    pub target_profile: &'static str,
    pub pane_kinds: Vec<&'static str>,
    pub route_kinds: Vec<&'static str>,
    pub readiness_checks: Vec<&'static str>,
    pub legacy_self_authoring_bypass: bool,
}

pub fn ui_designer_workbench_evidence() -> EditorUxWorkbenchEvidence {
    EditorUxWorkbenchEvidence {
        target_profile: UI_DESIGNER_WORKBENCH_TARGET_PROFILE,
        pane_kinds: vec![
            UiDesignerWorkbenchPaneKind::Canvas.as_key(),
            UiDesignerWorkbenchPaneKind::Hierarchy.as_key(),
            UiDesignerWorkbenchPaneKind::Inspector.as_key(),
            UiDesignerWorkbenchPaneKind::Properties.as_key(),
            UiDesignerWorkbenchPaneKind::TokenRecipePreview.as_key(),
            UiDesignerWorkbenchPaneKind::BindingPreview.as_key(),
            UiDesignerWorkbenchPaneKind::ScenarioMatrix.as_key(),
            UiDesignerWorkbenchPaneKind::Readiness.as_key(),
            UiDesignerWorkbenchPaneKind::Diagnostics.as_key(),
            UiDesignerWorkbenchPaneKind::NativeEvidencePreview.as_key(),
        ],
        route_kinds: vec![
            "select_ui_node",
            "set_ui_node_text",
            "build_apply_review",
            "apply_selected",
            "undo_operation",
            "redo_operation",
        ],
        readiness_checks: vec![
            "typed_pane_chain",
            "operation_intent_roundtrip",
            "story_lab_native_evidence",
            "legacy_self_authoring_bypass",
        ],
        legacy_self_authoring_bypass: true,
    }
}

pub fn ui_designer_workbench_required_states() -> [VisibleWidgetState; 4] {
    [
        VisibleWidgetState::Default,
        VisibleWidgetState::Focused,
        VisibleWidgetState::Disabled,
        VisibleWidgetState::Overflow,
    ]
}

pub fn ui_designer_workbench_fixture_view_model() -> UiDesignerWorkbenchViewModel {
    let select_action = EditorLabActionViewModel::enabled(
        "Select toolbar.save",
        EditorDefinitionSurfaceAction::SelectUiNode {
            node_id: "toolbar.save".to_string(),
        },
    );
    let edit_action = EditorLabActionViewModel::enabled(
        "Apply text edit to toolbar.save",
        EditorDefinitionSurfaceAction::SetUiNodeText {
            node_id: "toolbar.save".to_string(),
            text: "Save Project".to_string(),
        },
    );
    let apply_review = EditorLabActionViewModel::enabled(
        "Build apply review",
        EditorDefinitionSurfaceAction::BuildApplyReview,
    );
    let apply_selected = EditorLabActionViewModel::enabled(
        "Apply selected definition",
        EditorDefinitionSurfaceAction::ApplySelected,
    );

    UiDesignerWorkbenchViewModel::new(
        "Standalone UI Designer Workbench",
        UI_DESIGNER_WORKBENCH_TARGET_PROFILE,
        [
            UiDesignerWorkbenchPaneViewModel::new(UiDesignerWorkbenchPaneKind::Canvas, "Canvas")
                .with_summary_lines([
                    "retained preview available".to_string(),
                    "selected node: toolbar.save".to_string(),
                    "operation history: undo=1 redo=0".to_string(),
                ])
                .with_actions([select_action.clone(), edit_action.clone()]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::Hierarchy,
                "Hierarchy",
            )
            .with_summary_lines([
                "root: editor.toolbar".to_string(),
                "child: toolbar.save".to_string(),
                "child: toolbar.reload".to_string(),
            ])
            .with_actions([select_action]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::Inspector,
                "Inspector",
            )
            .with_summary_lines([
                "Document: Editor toolbar".to_string(),
                "Selected node: toolbar.save".to_string(),
                "Node text: Save".to_string(),
            ])
            .with_actions([edit_action]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::Properties,
                "Properties",
            )
            .with_summary_lines([
                "Display name: Editor toolbar".to_string(),
                "Lifecycle: Draft".to_string(),
                "Target profile: editor.workbench".to_string(),
            ]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::TokenRecipePreview,
                "Tokens and Recipes",
            )
            .with_summary_lines([
                "recipe: editor.pattern.primary_button".to_string(),
                "tokens: color.accent, color.text.primary".to_string(),
                "states: default, focused, high-contrast".to_string(),
            ]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::BindingPreview,
                "Bindings",
            )
            .with_summary_lines([
                "toolbar template: editor.toolbar".to_string(),
                "command intent: editor.project.save".to_string(),
                "capability: editor.definition.apply".to_string(),
            ]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::ScenarioMatrix,
                "Scenario Matrix",
            )
            .with_summary_lines([
                "density: comfortable, compact".to_string(),
                "viewport: narrow, standard, wide".to_string(),
                "input: pointer, keyboard".to_string(),
            ]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::Readiness,
                "Readiness",
            )
            .with_summary_lines([
                "visible-widget scan: required".to_string(),
                "native capture: screenshot or platform-impossible report".to_string(),
                "accessibility and timing reports: required".to_string(),
            ]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::Diagnostics,
                "Diagnostics",
            )
            .with_summary_lines(["blocking diagnostics: 0".to_string()])
            .with_diagnostics([EditorLabDiagnosticViewModel {
                severity: "Info".to_string(),
                code: "editor.ui_designer.workbench.ready".to_string(),
                message: "standalone workbench fixture has typed route and evidence coverage"
                    .to_string(),
                path: Some("editor.ui_designer.workbench".to_string()),
            }]),
            UiDesignerWorkbenchPaneViewModel::new(
                UiDesignerWorkbenchPaneKind::NativeEvidencePreview,
                "Native Evidence",
            )
            .with_summary_lines([
                "retained UI debug: required".to_string(),
                "focus traversal: required".to_string(),
                "platform-impossible fallback: typed".to_string(),
            ]),
        ],
    )
    .with_selected_document(Some(
        "Editor toolbar (runenwerk.editor.toolbar)".to_string(),
    ))
    .with_readiness([
        UiDesignerWorkbenchReadinessViewModel::new(
            "typed_pane_chain",
            UiDesignerWorkbenchReadinessStatus::Passed,
            "canvas, hierarchy, inspector, properties, previews, matrix, diagnostics",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "operation_intent_roundtrip",
            UiDesignerWorkbenchReadinessStatus::Passed,
            "selection, text edit, apply, undo, and redo use typed EditorDefinition routes",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "story_lab_native_evidence",
            UiDesignerWorkbenchReadinessStatus::Passed,
            "manifest requires retained UI, focus, accessibility, timing, and native proof",
        ),
        UiDesignerWorkbenchReadinessViewModel::new(
            "legacy_self_authoring_bypass",
            UiDesignerWorkbenchReadinessStatus::Passed,
            "workbench evidence rejects generic self-authoring action-list proof",
        ),
    ])
    .with_actions([
        apply_review,
        apply_selected,
        EditorLabActionViewModel::enabled(
            "Undo operation",
            EditorDefinitionSurfaceAction::UndoOperation,
        ),
        EditorLabActionViewModel::enabled(
            "Redo operation",
            EditorDefinitionSurfaceAction::RedoOperation,
        )
        .disabled("no undone workbench operation is available"),
    ])
}
