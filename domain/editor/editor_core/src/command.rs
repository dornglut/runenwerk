//! File: domain/editor/editor_core/src/command.rs
//! Purpose: Core editor command contracts for deterministic mutation pipelines.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandMetadata {
	pub id: CommandId,
	pub label: String,
	pub merge_key: Option<String>,
}

impl CommandMetadata {
	pub fn new(id: CommandId, label: impl Into<String>) -> Self {
		Self {
			id,
			label: label.into(),
			merge_key: None,
		}
	}

	pub fn with_merge_key(mut self, merge_key: impl Into<String>) -> Self {
		self.merge_key = Some(merge_key.into());
		self
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
	NoOp,
	Applied,
}

pub trait CommandContext {
	type Error;

	fn mark_document_dirty(
		&mut self,
		_document_id: crate::DocumentId,
		_is_dirty: bool,
	) -> Result<(), Self::Error> {
		Ok(())
	}
}

pub trait Command: Send + Sync {
	type Error;

	type Context<'a>: CommandContext<Error = Self::Error>
	where
		Self: 'a;

	fn metadata(&self) -> &CommandMetadata;

	fn apply<'a>(
		&mut self,
		ctx: &mut Self::Context<'a>,
	) -> Result<CommandOutcome, Self::Error>;

	fn undo<'a>(
		&mut self,
		ctx: &mut Self::Context<'a>,
	) -> Result<CommandOutcome, Self::Error>;
}