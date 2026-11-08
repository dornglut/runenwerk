
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
	fs,
	path::{Path, PathBuf},
};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppManifest {
	pub schema_version: u32,
	pub name: String,
	pub window: WindowConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
	pub title: String,
}

pub fn create(parent: &Path, name: &str) -> Result<PathBuf> {
	let mut chars = name.chars();

	let valid = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
		&& chars.all(|c| {
		c.is_ascii_alphanumeric() || c == '-' || c == '_'
	});

	ensure!(
        valid,
        "project name must start with a letter and contain only letters, numbers, '-' or '_'"
    );

	let root = parent.join(name);

	let manifest = AppManifest {
		schema_version: SCHEMA_VERSION,
		name: name.to_owned(),
		window: WindowConfig {
			title: name.to_owned(),
		},
	};

	let content = ron::ser::to_string_pretty(
		&manifest,
		ron::ser::PrettyConfig::default(),
	)?;

	fs::create_dir(&root)
		.with_context(|| format!("cannot create {}", root.display()))?;

	fs::write(root.join("app.ron"), format!("{content}\n"))
		.context("cannot write app.ron")?;

	Ok(root)
}

pub fn load(root: &Path) -> Result<AppManifest> {
	let path = root.join("app.ron");

	let content = fs::read_to_string(&path)
		.with_context(|| format!("cannot read {}", path.display()))?;

	let manifest: AppManifest = ron::from_str(&content)
		.with_context(|| format!("invalid {}", path.display()))?;

	ensure!(
        manifest.schema_version == SCHEMA_VERSION,
        "unsupported app.ron schema version: {}",
        manifest.schema_version
    );

	ensure!(
        !manifest.name.trim().is_empty(),
        "application name cannot be empty"
    );

	ensure!(
        !manifest.window.title.trim().is_empty(),
        "window title cannot be empty"
    );

	Ok(manifest)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn generated_project_is_loadable() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let root = create(temp.path(), "hello-rune")?;
		let manifest = load(&root)?;

		assert_eq!(manifest.schema_version, 1);
		assert_eq!(manifest.name, "hello-rune");
		assert_eq!(manifest.window.title, "hello-rune");
		assert!(!root.join("Cargo.toml").exists());

		Ok(())
	}

	#[test]
	fn rejects_invalid_project_names() -> Result<()> {
		let temp = tempfile::tempdir()?;

		assert!(create(temp.path(), "../outside").is_err());
		assert!(create(temp.path(), "hello world").is_err());

		Ok(())
	}

	#[test]
	fn rejects_unknown_manifest_fields() -> Result<()> {
		let temp = tempfile::tempdir()?;

		fs::write(
			temp.path().join("app.ron"),
			r#"(
                schema_version: 1,
                name: "test",
                window: (title: "Test"),
                unknown: true,
            )"#,
		)?;

		assert!(load(temp.path()).is_err());
		Ok(())
	}
}
