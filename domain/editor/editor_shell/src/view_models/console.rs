//! File: domain/editor/editor_shell/src/view_models/console.rs
//! Purpose: Console shell view model.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsoleViewModel {
    pub lines: Vec<ConsoleLineViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleLineViewModel {
    pub text: String,
    pub kind: ConsoleLineKind,
}

impl ConsoleLineViewModel {
    pub fn new(kind: ConsoleLineKind, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConsoleLineKind {
    Input,
    Error,
    Warning,
    #[default]
    Info,
    Debug,
}
