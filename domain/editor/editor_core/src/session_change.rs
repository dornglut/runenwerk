//! File: domain/editor/editor_core/src/session_change.rs
//! Purpose: Session-reality change contracts (non-ratifying by default).

use std::collections::VecDeque;
use std::time::SystemTime;

use crate::{ChangeOrigin, ModeId, SelectionTarget, ToolId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionChangeId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SessionChangeKind {
    ActiveToolSet { tool_id: Option<ToolId> },
    ModeSet { mode: ModeId },
    SelectionSetSingle { target: SelectionTarget },
    SelectionCleared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionChange {
    pub id: SessionChangeId,
    pub origin: ChangeOrigin,
    pub kind: SessionChangeKind,
    pub timestamp: SystemTime,
}

impl SessionChange {
    pub fn new(id: SessionChangeId, origin: ChangeOrigin, kind: SessionChangeKind) -> Self {
        Self {
            id,
            origin,
            kind,
            timestamp: SystemTime::now(),
        }
    }
}

const DEFAULT_SESSION_CHANGE_LOG_CAPACITY: usize = 512;

#[derive(Debug, Clone)]
pub struct SessionChangeLog {
    entries: VecDeque<SessionChange>,
    max_entries: usize,
}

impl Default for SessionChangeLog {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_SESSION_CHANGE_LOG_CAPACITY)
    }
}

impl SessionChangeLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: max_entries.max(1),
        }
    }

    pub fn push(&mut self, change: SessionChange) {
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

    pub fn last(&self) -> Option<&SessionChange> {
        self.entries.back()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SessionChange> {
        self.entries.iter()
    }
}
