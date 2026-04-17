use std::collections::VecDeque;

use editor_core::RatifiedChange;

const DEFAULT_RATIFIED_CHANGE_LOG_CAPACITY: usize = 1024;

#[derive(Debug, Clone)]
pub struct RatifiedChangeLog {
    entries: VecDeque<RatifiedChange>,
    max_entries: usize,
}

impl Default for RatifiedChangeLog {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_RATIFIED_CHANGE_LOG_CAPACITY)
    }
}

impl RatifiedChangeLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: max_entries.max(1),
        }
    }

    pub fn push(&mut self, change: RatifiedChange) {
        if self.entries.len() == self.max_entries {
            self.entries.pop_front();
        }

        self.entries.push_back(change);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &RatifiedChange> {
        self.entries.iter()
    }

    pub fn last(&self) -> Option<&RatifiedChange> {
        self.entries.back()
    }
}
