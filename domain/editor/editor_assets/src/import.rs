//! File: domain/editor/editor_assets/src/import.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportStatus {
	Pending,
	Imported,
	Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRecord {
	pub source_path: String,
	pub status: ImportStatus,
	pub message: Option<String>,
}

impl ImportRecord {
	pub fn new(source_path: impl Into<String>, status: ImportStatus) -> Self {
		Self {
			source_path: source_path.into(),
			status,
			message: None,
		}
	}

	pub fn with_message(mut self, message: impl Into<String>) -> Self {
		self.message = Some(message.into());
		self
	}
}