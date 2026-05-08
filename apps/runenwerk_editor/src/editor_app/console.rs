//! File: apps/runenwerk_editor/src/editor_app/console.rs
//! Purpose: App-owned retained console message model.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConsoleMessageKind {
    Input,
    Error,
    Warning,
    #[default]
    Info,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleMessage {
    pub text: String,
    pub kind: ConsoleMessageKind,
}

impl ConsoleMessage {
    pub fn new(kind: ConsoleMessageKind, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }
}
