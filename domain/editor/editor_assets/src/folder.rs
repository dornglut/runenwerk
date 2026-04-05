//! File: domain/editor/editor_assets/src/folder.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetFolder {
	pub path: String,
	pub display_name: String,
	pub children: Vec<String>,
}

impl AssetFolder {
	pub fn new(path: impl Into<String>, display_name: impl Into<String>) -> Self {
		Self {
			path: path.into(),
			display_name: display_name.into(),
			children: Vec::new(),
		}
	}

	pub fn with_children(mut self, children: Vec<String>) -> Self {
		self.children = children;
		self
	}
}