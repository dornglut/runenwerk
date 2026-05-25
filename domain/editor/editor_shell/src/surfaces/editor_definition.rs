//! File: domain/editor/editor_shell/src/surfaces/editor_definition.rs
//! Purpose: Editor-definition self-authoring surface workflow contracts.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorDefinitionSurfaceAction {
    SelectDocument { document_id: String },
    DuplicateSelected,
    RenameSelected { display_name: String },
    DeleteSelected,
    SaveProjectPackage,
    ReloadProjectPackage,
    ExportSelected,
    BuildApplyReview,
    RejectApplyReview,
    ApplySelected,
    RollbackSelected,
    ReloadLastApplied,
    UndoOperation,
    RedoOperation,
    SelectUiNode { node_id: String },
    SetUiNodeText { node_id: String, text: String },
    SetThemeColor { token: String, value: String },
    AddWorkspaceLayoutTab { label: String, tool_surface: String },
    SplitWorkspaceLayoutRoot { axis: String },
    CloseWorkspaceLayoutLastTab,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorLabSurfaceViewModel {
    UiDesignerWorkbench(UiDesignerWorkbenchViewModel),
    DefinitionHierarchy(EditorLabDefinitionHierarchyViewModel),
    Palette(EditorLabPaletteViewModel),
    CanvasPreview(EditorLabCanvasPreviewViewModel),
    Inspector(EditorLabInspectorViewModel),
    Review(EditorLabReviewViewModel),
    Diagnostics(EditorLabDiagnosticsViewModel),
    Console(EditorLabConsoleViewModel),
    Degraded(EditorLabDegradedViewModel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiDesignerWorkbenchPaneKind {
    Canvas,
    Hierarchy,
    Inspector,
    Properties,
    TokenRecipePreview,
    BindingPreview,
    ScenarioMatrix,
    Readiness,
    Diagnostics,
    NativeEvidencePreview,
}

impl UiDesignerWorkbenchPaneKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Canvas => "Canvas",
            Self::Hierarchy => "Hierarchy",
            Self::Inspector => "Inspector",
            Self::Properties => "Properties",
            Self::TokenRecipePreview => "Token / Recipe Preview",
            Self::BindingPreview => "Binding Preview",
            Self::ScenarioMatrix => "Scenario Matrix",
            Self::Readiness => "Readiness",
            Self::Diagnostics => "Diagnostics",
            Self::NativeEvidencePreview => "Native Evidence",
        }
    }

    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Canvas => "canvas",
            Self::Hierarchy => "hierarchy",
            Self::Inspector => "inspector",
            Self::Properties => "properties",
            Self::TokenRecipePreview => "token_recipe_preview",
            Self::BindingPreview => "binding_preview",
            Self::ScenarioMatrix => "scenario_matrix",
            Self::Readiness => "readiness",
            Self::Diagnostics => "diagnostics",
            Self::NativeEvidencePreview => "native_evidence_preview",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiDesignerWorkbenchReadinessStatus {
    Passed,
    Warning,
    Blocked,
}

impl UiDesignerWorkbenchReadinessStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDesignerWorkbenchReadinessViewModel {
    pub check: String,
    pub status: UiDesignerWorkbenchReadinessStatus,
    pub evidence: String,
}

impl UiDesignerWorkbenchReadinessViewModel {
    pub fn new(
        check: impl Into<String>,
        status: UiDesignerWorkbenchReadinessStatus,
        evidence: impl Into<String>,
    ) -> Self {
        Self {
            check: check.into(),
            status,
            evidence: evidence.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDesignerWorkbenchPaneViewModel {
    pub kind: UiDesignerWorkbenchPaneKind,
    pub title: String,
    pub summary_lines: Vec<String>,
    pub actions: Vec<EditorLabActionViewModel>,
    pub diagnostics: Vec<EditorLabDiagnosticViewModel>,
}

impl UiDesignerWorkbenchPaneViewModel {
    pub fn new(kind: UiDesignerWorkbenchPaneKind, title: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            summary_lines: Vec::new(),
            actions: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_summary_lines(mut self, lines: impl IntoIterator<Item = String>) -> Self {
        self.summary_lines = lines.into_iter().collect();
        self
    }

    pub fn with_actions(
        mut self,
        actions: impl IntoIterator<Item = EditorLabActionViewModel>,
    ) -> Self {
        self.actions = actions.into_iter().collect();
        self
    }

    pub fn with_diagnostics(
        mut self,
        diagnostics: impl IntoIterator<Item = EditorLabDiagnosticViewModel>,
    ) -> Self {
        self.diagnostics = diagnostics.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDesignerWorkbenchViewModel {
    pub title: String,
    pub selected_document: Option<String>,
    pub target_profile: String,
    pub panes: Vec<UiDesignerWorkbenchPaneViewModel>,
    pub readiness: Vec<UiDesignerWorkbenchReadinessViewModel>,
    pub actions: Vec<EditorLabActionViewModel>,
}

impl UiDesignerWorkbenchViewModel {
    pub fn new(
        title: impl Into<String>,
        target_profile: impl Into<String>,
        panes: impl IntoIterator<Item = UiDesignerWorkbenchPaneViewModel>,
    ) -> Self {
        Self {
            title: title.into(),
            selected_document: None,
            target_profile: target_profile.into(),
            panes: panes.into_iter().collect(),
            readiness: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn with_selected_document(mut self, selected_document: Option<String>) -> Self {
        self.selected_document = selected_document;
        self
    }

    pub fn with_readiness(
        mut self,
        readiness: impl IntoIterator<Item = UiDesignerWorkbenchReadinessViewModel>,
    ) -> Self {
        self.readiness = readiness.into_iter().collect();
        self
    }

    pub fn with_actions(
        mut self,
        actions: impl IntoIterator<Item = EditorLabActionViewModel>,
    ) -> Self {
        self.actions = actions.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabActionViewModel {
    pub label: String,
    pub action: EditorDefinitionSurfaceAction,
    pub selected: bool,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

impl EditorLabActionViewModel {
    pub fn enabled(label: impl Into<String>, action: EditorDefinitionSurfaceAction) -> Self {
        Self {
            label: label.into(),
            action,
            selected: false,
            enabled: true,
            disabled_reason: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.enabled = false;
        self.disabled_reason = Some(reason.into());
        self
    }

    pub fn disabled_if(self, condition: bool, reason: impl Into<String>) -> Self {
        if condition {
            self.disabled(reason)
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabTextFieldViewModel {
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub action: EditorDefinitionSurfaceAction,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

impl EditorLabTextFieldViewModel {
    pub fn new(
        label: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
        action: EditorDefinitionSurfaceAction,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            placeholder: placeholder.into(),
            action,
            enabled: true,
            disabled_reason: None,
        }
    }

    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.enabled = false;
        self.disabled_reason = Some(reason.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabDiagnosticViewModel {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabDefinitionRowViewModel {
    pub document_id: String,
    pub label: String,
    pub kind: String,
    pub lifecycle: String,
    pub selected: bool,
    pub source_path: Option<String>,
    pub diagnostic_count: usize,
    pub select_action: EditorDefinitionSurfaceAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabDefinitionHierarchyViewModel {
    pub title: String,
    pub selected_document: Option<String>,
    pub rows: Vec<EditorLabDefinitionRowViewModel>,
    pub actions: Vec<EditorLabActionViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabPaletteCategoryViewModel {
    pub label: String,
    pub items: Vec<EditorLabActionViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabPaletteViewModel {
    pub title: String,
    pub categories: Vec<EditorLabPaletteCategoryViewModel>,
    pub diagnostics: Vec<EditorLabDiagnosticViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabCanvasPreviewViewModel {
    pub title: String,
    pub selected_document: Option<String>,
    pub retained_preview_available: bool,
    pub status_lines: Vec<String>,
    pub actions: Vec<EditorLabActionViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabInspectorFieldViewModel {
    pub label: String,
    pub value: String,
    pub text_field: Option<EditorLabTextFieldViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabInspectorViewModel {
    pub title: String,
    pub selected_document: Option<String>,
    pub fields: Vec<EditorLabInspectorFieldViewModel>,
    pub actions: Vec<EditorLabActionViewModel>,
    pub diagnostics: Vec<EditorLabDiagnosticViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabReviewViewModel {
    pub title: String,
    pub selected_document: Option<String>,
    pub summary_lines: Vec<String>,
    pub actions: Vec<EditorLabActionViewModel>,
    pub diagnostics: Vec<EditorLabDiagnosticViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabDiagnosticsViewModel {
    pub title: String,
    pub selected_document: Option<String>,
    pub diagnostics: Vec<EditorLabDiagnosticViewModel>,
    pub actions: Vec<EditorLabActionViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabConsoleLineViewModel {
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabConsoleViewModel {
    pub title: String,
    pub lines: Vec<EditorLabConsoleLineViewModel>,
    pub actions: Vec<EditorLabActionViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLabDegradedViewModel {
    pub title: String,
    pub reason: String,
    pub details: Vec<String>,
    pub diagnostics: Vec<EditorLabDiagnosticViewModel>,
    pub recovery_actions: Vec<EditorLabActionViewModel>,
}
