//! File: domain/editor/editor_core/src/workflow.rs
//! Purpose: Workflow-reality log contracts.

use std::collections::VecDeque;
use std::time::SystemTime;

use crate::{MigrationPathId, RatificationId, ReconciliationResult, SharedChangeSequence};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkflowEventId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowEventKind {
    ShellCommandDispatched {
        command: &'static str,
    },
    SharedChangeQueued {
        sequence: SharedChangeSequence,
    },
    SharedChangeReconciled {
        sequence: SharedChangeSequence,
        result: ReconciliationResult,
    },
    RatifiedChangeRecorded {
        ratification_id: RatificationId,
    },
    SceneSaved {
        path: String,
    },
    RetainedChangesSaved {
        path: String,
        entry_count: usize,
    },
    RetainedChangesLoaded {
        path: String,
        entry_count: usize,
    },
    SceneLoaded {
        path: String,
        migration_path: Option<MigrationPathId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowEvent {
    pub id: WorkflowEventId,
    pub kind: WorkflowEventKind,
    pub timestamp: SystemTime,
}

impl WorkflowEvent {
    pub fn new(id: WorkflowEventId, kind: WorkflowEventKind) -> Self {
        Self {
            id,
            kind,
            timestamp: SystemTime::now(),
        }
    }
}

const DEFAULT_WORKFLOW_LOG_CAPACITY: usize = 1024;

#[derive(Debug, Clone)]
pub struct WorkflowLog {
    entries: VecDeque<WorkflowEvent>,
    max_entries: usize,
}

impl Default for WorkflowLog {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_WORKFLOW_LOG_CAPACITY)
    }
}

impl WorkflowLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: max_entries.max(1),
        }
    }

    pub fn push(&mut self, event: WorkflowEvent) {
        if self.entries.len() == self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(event);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn last(&self) -> Option<&WorkflowEvent> {
        self.entries.back()
    }

    pub fn iter(&self) -> impl Iterator<Item = &WorkflowEvent> {
        self.entries.iter()
    }
}
