//! File: domain/editor/editor_assets/src/asset.rs

use editor_core::AssetId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetKind {
	Scene,
	Material,
	Shader,
	Texture,
	Mesh,
	Audio,
	Animation,
	Rig,
	SdfModel,
	Graph,
	Custom(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDescriptor {
	pub id: AssetId,
	pub kind: AssetKind,
	pub display_name: String,
	pub path: String,
	pub is_dirty: bool,
}

impl AssetDescriptor {
	pub fn new(
		id: AssetId,
		kind: AssetKind,
		display_name: impl Into<String>,
		path: impl Into<String>,
	) -> Self {
		Self {
			id,
			kind,
			display_name: display_name.into(),
			path: path.into(),
			is_dirty: false,
		}
	}

	pub fn with_dirty(mut self, is_dirty: bool) -> Self {
		self.is_dirty = is_dirty;
		self
	}
}