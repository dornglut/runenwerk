//! File: domain/editor/editor_assets/src/browser.rs

use editor_core::AssetId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBrowserQuery {
	pub text: String,
	pub kind_filter: Option<String>,
	pub folder_path: Option<String>,
}

impl AssetBrowserQuery {
	pub fn new() -> Self {
		Self {
			text: String::new(),
			kind_filter: None,
			folder_path: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBrowserSelection {
	pub primary: Option<AssetId>,
	pub selected: Vec<AssetId>,
}

impl AssetBrowserSelection {
	pub fn new() -> Self {
		Self {
			primary: None,
			selected: Vec::new(),
		}
	}
}