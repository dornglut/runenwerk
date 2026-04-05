//! File: domain/editor/editor_assets/src/dependency.rs

use editor_core::AssetId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDependency {
	pub from: AssetId,
	pub to: AssetId,
	pub relation: String,
}

impl AssetDependency {
	pub fn new(from: AssetId, to: AssetId, relation: impl Into<String>) -> Self {
		Self {
			from,
			to,
			relation: relation.into(),
		}
	}
}