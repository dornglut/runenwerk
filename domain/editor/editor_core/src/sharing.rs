//! File: domain/editor/editor_core/src/sharing.rs
//! Purpose: Shared-reality propagation contracts for ratified changes.

use std::collections::VecDeque;

use crate::RatifiedChange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SharedChangeSequence(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharingPolicy {
    LocalOnly,
    SessionBroadcast,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedChangeEnvelope {
    pub sequence: SharedChangeSequence,
    pub change: RatifiedChange,
}

impl SharedChangeEnvelope {
    pub fn new(sequence: SharedChangeSequence, change: RatifiedChange) -> Self {
        Self { sequence, change }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SharedChangeOutbox {
    queued: VecDeque<SharedChangeEnvelope>,
}

impl SharedChangeOutbox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue(&mut self, envelope: SharedChangeEnvelope) {
        self.queued.push_back(envelope);
    }

    pub fn enqueue_front(&mut self, envelope: SharedChangeEnvelope) {
        self.queued.push_front(envelope);
    }

    pub fn len(&self) -> usize {
        self.queued.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queued.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<SharedChangeEnvelope> {
        self.queued.pop_front()
    }

    pub fn drain(&mut self) -> Vec<SharedChangeEnvelope> {
        self.queued.drain(..).collect()
    }
}

pub trait SharedChangePropagationSink {
    type Error;

    fn push_shared_change(&mut self, envelope: SharedChangeEnvelope) -> Result<(), Self::Error>;
}
