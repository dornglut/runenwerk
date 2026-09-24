//! File: domain/editor/editor_core/src/transaction_builder.rs
//! Purpose: Lightweight transaction assembly before execution/history storage.

use crate::{CommandMetadata, TransactionId, TransactionMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionBuilder {
    metadata: TransactionMetadata,
    commands: Vec<CommandMetadata>,
}

impl TransactionBuilder {
    pub fn new(id: TransactionId, label: impl Into<String>) -> Self {
        Self {
            metadata: TransactionMetadata::new(id, label),
            commands: Vec::new(),
        }
    }

    pub fn metadata(&self) -> &TransactionMetadata {
        &self.metadata
    }

    pub fn command_metadata(&self) -> &[CommandMetadata] {
        &self.commands
    }

    pub fn push_command(&mut self, metadata: CommandMetadata) {
        self.commands.push(metadata);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn into_parts(self) -> (TransactionMetadata, Vec<CommandMetadata>) {
        (self.metadata, self.commands)
    }
}
