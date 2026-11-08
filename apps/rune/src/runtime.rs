use anyhow::{Context, Result, anyhow, ensure};
use engine::{
    App,
    app::{AppNativeHostExt, NativePrimaryClosePolicy},
    plugins::{
        default_plugins,
        render::host::RenderSurfaceId,
        ui::{AppUiExt, UiRuntimeFontConfiguration, UiScreen, UiTypedScreenId, UiTypedSource},
    },
    prelude::AppRenderExt,
};
use std::{
    fs,
    path::{Component, Path},
};
use ui_definition::UiNodeDefinition;
use ui_program::UiProgramSourceId;

use crate::project;

struct FileScreen {
    root: UiNodeDefinition,
}

impl UiScreen for FileScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new("app.main")
    }

    fn build_source(&self) -> UiTypedSource {
        UiTypedSource::new(
            self.screen_id(),
            UiProgramSourceId::new("app.main.source"),
            self.root.clone(),
        )
    }
}

fn load_ui(project_root: &Path, source: &str) -> Result<UiNodeDefinition> {
    let relative = Path::new(source);
    ensure!(
        !source.trim().is_empty()
            && relative
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
        "UI source must be a nonempty project-relative path without '..'"
    );

    let root = project_root
        .canonicalize()
        .with_context(|| format!("cannot resolve project {}", project_root.display()))?;
    let path = root
        .join(relative)
        .canonicalize()
        .with_context(|| format!("cannot resolve UI source {}", root.join(relative).display()))?;

    ensure!(
        path.starts_with(&root),
        "UI source resolves outside the application project"
    );

    let content = fs::read_to_string(&path)
        .with_context(|| format!("cannot read UI source {}", path.display()))?;
    ron::from_str(&content).with_context(|| format!("invalid UI source {}", path.display()))
}

pub fn run(project_root: &Path) -> Result<()> {
    let manifest = project::load(project_root)?;
    let root = manifest
        .ui
        .as_ref()
        .map(|ui| load_ui(project_root, &ui.source))
        .transpose()?;

    let mut app = App::new();
    app.set_title(manifest.window.title);
    app.add_plugins(default_plugins());
    app.with_primary_close_policy(NativePrimaryClosePolicy::ApproveUnhandled)?;

    if let Some(root) = root {
        app.select_native_ui_only()
            .context("cannot select native UI-only rendering")?;

        const FONT: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/JetBrainsMono-Regular.ttf"
        ));

        let fonts = UiRuntimeFontConfiguration::new(vec![FONT.to_vec()]).with_system_fonts();

        let report = app.ui().mount_with_fonts(FileScreen { root }, &fonts);
        let slot = report
            .slot_id()
            .ok_or_else(|| anyhow!("UI mount rejected: {report:?}"))?;

        app.ui()
            .present(slot, RenderSurfaceId::primary())
            .context("cannot present UI on native primary surface")?;
        app.ui()
            .bind_native_primary_input(slot)
            .context("cannot bind native primary UI input")?;
    }

    app.run().context("Runenwerk runtime failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_generated_ui_from_ron() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let project_root = project::create(temp.path(), "app")?;
        assert!(matches!(
            load_ui(&project_root, "ui/main.ron")?,
            UiNodeDefinition::Label { .. }
        ));
        Ok(())
    }

    #[test]
    fn rejects_parent_and_absolute_paths() -> Result<()> {
        let temp = tempfile::tempdir()?;
        for source in ["../outside.ron", "/outside.ron", "ui/../main.ron", ""] {
            assert!(
                load_ui(temp.path(), source).is_err(),
                "source {source:?} must be rejected"
            );
        }
        Ok(())
    }

    #[test]
    fn rejects_invalid_ui_document() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::create_dir(temp.path().join("ui"))?;
        fs::write(temp.path().join("ui/main.ron"), "not valid RON")?;
        assert!(load_ui(temp.path(), "ui/main.ron").is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() -> Result<()> {
        use std::os::unix::fs::symlink;

        let project_root = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        fs::write(
            outside.path().join("external.ron"),
            "Spacer(id: AuthoredId(\"x\"))",
        )?;
        symlink(
            outside.path().join("external.ron"),
            project_root.path().join("external.ron"),
        )?;

        assert!(load_ui(project_root.path(), "external.ron").is_err());
        Ok(())
    }
}
