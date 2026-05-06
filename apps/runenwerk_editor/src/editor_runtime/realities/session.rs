//! File: apps/runenwerk_editor/src/editor_runtime/realities/session.rs
//! Purpose: Read-only session-reality boundary for editor interaction state.

use editor_core::{EditorSession, HistoryStack, ModeId, SelectionSet, ToolId};

#[derive(Debug, Clone, Copy)]
pub struct SessionReality<'a> {
    session: &'a EditorSession,
}

impl<'a> SessionReality<'a> {
    pub fn new(session: &'a EditorSession) -> Self {
        Self { session }
    }

    pub fn session(&self) -> &'a EditorSession {
        self.session
    }

    pub fn mode(&self) -> ModeId {
        self.session.mode()
    }

    pub fn active_tool(&self) -> Option<ToolId> {
        self.session.active_tool()
    }

    pub fn selection(&self) -> &'a SelectionSet {
        self.session.selection()
    }

    pub fn history(&self) -> &'a HistoryStack {
        self.session.history()
    }
}
