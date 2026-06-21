use ui_composition::{CompositionSnapshot, CompositionState};

use super::{EditorCompositionExtensionV1, EditorCompositionRejection};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCompositionRuntime {
    pub(super) composition: CompositionState,
    pub(super) extension: EditorCompositionExtensionV1,
    pub(super) extension_history: EditorExtensionHistory,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct EditorExtensionHistory {
    journal: Vec<EditorExtensionHistoryEntry>,
    cursor: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EditorExtensionHistoryEntry {
    before: EditorCompositionExtensionV1,
    after: EditorCompositionExtensionV1,
}

impl EditorExtensionHistory {
    pub(super) fn record(
        &mut self,
        before: EditorCompositionExtensionV1,
        after: EditorCompositionExtensionV1,
    ) {
        self.journal.truncate(self.cursor);
        self.journal
            .push(EditorExtensionHistoryEntry { before, after });
        self.cursor = self.journal.len();
    }

    pub(super) fn undo_extension(&self) -> Option<&EditorCompositionExtensionV1> {
        self.cursor
            .checked_sub(1)
            .and_then(|index| self.journal.get(index))
            .map(|entry| &entry.before)
    }

    pub(super) fn redo_extension(&self) -> Option<&EditorCompositionExtensionV1> {
        self.journal.get(self.cursor).map(|entry| &entry.after)
    }

    pub(super) fn commit_undo(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub(super) fn commit_redo(&mut self) {
        self.cursor = (self.cursor + 1).min(self.journal.len());
    }
}

impl EditorCompositionRuntime {
    pub fn install(
        composition: CompositionState,
        extension: EditorCompositionExtensionV1,
    ) -> Result<Self, EditorCompositionRejection> {
        extension.validate_against(&composition)?;
        Ok(Self {
            composition,
            extension,
            extension_history: EditorExtensionHistory::default(),
        })
    }

    pub fn composition(&self) -> &CompositionState {
        &self.composition
    }

    pub fn snapshot(&self) -> CompositionSnapshot<'_> {
        self.composition.snapshot()
    }

    pub fn extension(&self) -> &EditorCompositionExtensionV1 {
        &self.extension
    }
}
