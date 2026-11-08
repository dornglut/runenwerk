use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use ui_definition::{AuthoredId, UiNodeDefinition, UiValueBinding};

const SCHEMA_VERSION: u32 = 1;
const DEFAULT_UI_SOURCE: &str = "ui/main.ron";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppManifest {
    pub schema_version: u32,
    pub name: String,
    pub window: WindowConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiConfig {
    pub source: String,
}

pub fn create(parent: &Path, name: &str) -> Result<PathBuf> {
    let mut chars = name.chars();
    let valid = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

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
        ui: Some(UiConfig {
            source: DEFAULT_UI_SOURCE.to_owned(),
        }),
    };

    let label = UiNodeDefinition::Label {
        id: AuthoredId::new("app.main.title"),
        label: UiValueBinding::static_text("Hello from Runenwerk"),
        availability: None,
    };

    let pretty = ron::ser::PrettyConfig::default();
    let app_content = ron::ser::to_string_pretty(&manifest, pretty.clone())?;
    let ui_content = ron::ser::to_string_pretty(&label, pretty)?;

    fs::create_dir(&root)
        .with_context(|| format!("cannot create {}", root.display()))?;

    let write_result = (|| -> Result<()> {
        fs::create_dir(root.join("ui")).context("cannot create ui directory")?;
        fs::write(root.join("app.ron"), format!("{app_content}\n"))
            .context("cannot write app.ron")?;
        fs::write(root.join(DEFAULT_UI_SOURCE), format!("{ui_content}\n"))
            .context("cannot write ui/main.ron")?;
        Ok(())
    })();

    if let Err(error) = write_result {
        if let Err(cleanup) = fs::remove_dir_all(&root) {
            return Err(error.context(format!(
                "project generation failed and cleanup of {} also failed: {cleanup}",
                root.display()
            )));
        }
        return Err(error);
    }

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

    if let Some(ui) = &manifest.ui {
        ensure!(
            !ui.source.trim().is_empty(),
            "UI source cannot be empty"
        );
    }

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
        assert_eq!(
            manifest.ui.as_ref().map(|ui| ui.source.as_str()),
            Some("ui/main.ron")
        );
        assert!(!root.join("Cargo.toml").exists());

        let source = fs::read_to_string(root.join(DEFAULT_UI_SOURCE))?;
        let node: UiNodeDefinition = ron::from_str(&source)?;
        assert!(matches!(
            node,
            UiNodeDefinition::Label {
                label: UiValueBinding::Static(ui_definition::UiValue::Text(ref text)),
                ..
            } if text == "Hello from Runenwerk"
        ));

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
    fn rejects_existing_project_without_modification() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = create(temp.path(), "existing")?;
        let original = fs::read(root.join("app.ron"))?;

        assert!(create(temp.path(), "existing").is_err());
        assert_eq!(fs::read(root.join("app.ron"))?, original);
        Ok(())
    }

    #[test]
    fn accepts_window_only_manifest() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(
            temp.path().join("app.ron"),
            r#"(
                schema_version: 1,
                name: "legacy",
                window: (title: "Legacy"),
            )"#,
        )?;

        let manifest = load(temp.path())?;
        assert!(manifest.ui.is_none());
        Ok(())
    }

    #[test]
    fn rejects_empty_ui_source() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(
            temp.path().join("app.ron"),
            r#"(
                schema_version: 1,
                name: "test",
                window: (title: "Test"),
                ui: Some((source: "  ")),
            )"#,
        )?;

        assert!(load(temp.path()).is_err());
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
