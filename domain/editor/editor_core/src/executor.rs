//! File: domain/editor/editor_core/src/executor.rs
//! Purpose: Generic command execution primitives.

use crate::{Command, CommandMetadata, CommandOutcome, TransactionMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedCommand {
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedTransaction {
    pub transaction: TransactionMetadata,
    pub commands: Vec<ExecutedCommand>,
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn execute_command<C>(
        ctx: &mut C::Context<'_>,
        command: &mut C,
    ) -> Result<Option<ExecutedCommand>, C::Error>
    where
        C: Command,
    {
        match command.apply(ctx)? {
            CommandOutcome::NoOp => Ok(None),
            CommandOutcome::Applied => Ok(Some(ExecutedCommand {
                metadata: command.metadata().clone(),
            })),
        }
    }

    pub fn execute_transaction<C>(
        ctx: &mut C::Context<'_>,
        transaction: TransactionMetadata,
        commands: &mut [C],
    ) -> Result<ExecutedTransaction, C::Error>
    where
        C: Command,
    {
        let mut executed = Vec::new();

        for command in commands {
            if let Some(entry) = Self::execute_command(ctx, command)? {
                executed.push(entry);
            }
        }

        Ok(ExecutedTransaction {
            transaction,
            commands: executed,
        })
    }
}
