//! File: domain/editor/editor_shell/src/surfaces/inspector.rs
//! Purpose: Inspector surface workflow contracts.

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorFieldControlKind {
    BoolToggle {
        checked: bool,
    },
    IntegerInput {
        value: i64,
    },
    FloatInput {
        value: f64,
    },
    TextInput,
    EnumSelect {
        current: String,
        options: Vec<String>,
        selected_index: Option<usize>,
    },
    ReadOnly,
    Group,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorFieldEditIntent {
    Bool { value: bool },
    Number { value: f64 },
    Text { text: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorSurfaceAction {
    ActivateField { index: usize },
    FocusField { index: usize },
    EditFieldText { index: usize, text: String },
    BackspaceFieldText { index: usize },
    CommitFieldText { index: usize },
    CancelFieldText { index: usize },
    SetFieldBool { index: usize, value: bool },
    SetFieldNumber { index: usize, value: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorSessionMutation {
    ActivateField { index: usize },
    FocusField { index: usize },
    AppendFieldText { index: usize, text: String },
    BackspaceFieldText { index: usize },
    CommitFieldText { index: usize },
    CancelFieldText { index: usize },
    SetFieldBool { index: usize, value: bool },
    SetFieldNumber { index: usize, value: f64 },
}
