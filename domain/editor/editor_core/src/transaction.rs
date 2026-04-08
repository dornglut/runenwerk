//! File: domain/editor/editor_core/src/transaction.rs
//! Purpose: Grouped editor mutation contracts.

use crate::CommandMetadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionMetadata {
    pub id: TransactionId,
    pub label: String,
}

impl TransactionMetadata {
    pub fn new(id: TransactionId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

pub trait Transaction {
    fn metadata(&self) -> &TransactionMetadata;

    fn command_metadata(&self) -> &[CommandMetadata];
}
