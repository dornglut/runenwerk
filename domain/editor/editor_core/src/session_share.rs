//! File: domain/editor/editor_core/src/session_share.rs
//! Purpose: Session-reality sharing contracts (non-ratifying by default).

use std::collections::VecDeque;
use std::time::SystemTime;

use crate::{ChangeOrigin, EditorMode, SelectionTarget, ToolId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionShareSequence(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionSharePolicy {
    Disabled,
    ObservationSafe,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SessionShareKind {
    ActiveToolSet { tool_id: Option<ToolId> },
    ModeSet { mode: EditorMode },
    SelectionSetSingle { target: SelectionTarget },
    SelectionCleared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionShareEntry {
    pub origin: ChangeOrigin,
    pub kind: SessionShareKind,
    pub timestamp: SystemTime,
}

impl SessionShareEntry {
    pub fn new(origin: ChangeOrigin, kind: SessionShareKind) -> Self {
        Self {
            origin,
            kind,
            timestamp: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionShareEnvelope {
    pub sequence: SessionShareSequence,
    pub entry: SessionShareEntry,
}

impl SessionShareEnvelope {
    pub fn new(sequence: SessionShareSequence, entry: SessionShareEntry) -> Self {
        Self { sequence, entry }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SessionShareOutbox {
    queued: VecDeque<SessionShareEnvelope>,
}

impl SessionShareOutbox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue(&mut self, envelope: SessionShareEnvelope) {
        self.queued.push_back(envelope);
    }

    pub fn len(&self) -> usize {
        self.queued.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queued.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<SessionShareEnvelope> {
        self.queued.pop_front()
    }

    pub fn drain(&mut self) -> Vec<SessionShareEnvelope> {
        self.queued.drain(..).collect()
    }
}
