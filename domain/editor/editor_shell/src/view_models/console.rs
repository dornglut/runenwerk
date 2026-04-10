//! File: domain/editor/editor_shell/src/view_models/console.rs
//! Purpose: Console shell view model.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsoleViewModel {
    pub lines: Vec<String>,
}
