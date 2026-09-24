//! Bounded ink lifecycle journal.

const DRAWING_INK_JOURNAL_LIMIT: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingInkJournalStage {
    Formation,
    ProductPublication,
    QuerySnapshotPublication,
    GpuValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingInkJournalEntry {
    pub stage: DrawingInkJournalStage,
    pub accepted: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DrawingInkJournalState {
    journal: Vec<DrawingInkJournalEntry>,
}

impl DrawingInkJournalState {
    pub(crate) fn journal(&self) -> &[DrawingInkJournalEntry] {
        &self.journal
    }

    pub(crate) fn record_journal(
        &mut self,
        stage: DrawingInkJournalStage,
        accepted: bool,
        summary: impl Into<String>,
    ) {
        self.journal.push(DrawingInkJournalEntry {
            stage,
            accepted,
            summary: summary.into(),
        });
        if self.journal.len() > DRAWING_INK_JOURNAL_LIMIT {
            let drain = self.journal.len() - DRAWING_INK_JOURNAL_LIMIT;
            self.journal.drain(0..drain);
        }
    }
}
