use anyhow::{Context, Result, anyhow, ensure};
use engine::{
    App,
    app::{AppNativeHostExt, NativePrimaryClosePolicy},
    plugins::{
        default_plugins,
        render::{
            ManagedRenderProducerIds, RenderFrameProducerId, RunenUiPaintSubmissionRegistryResource,
            SurfaceFrameSubmissionRegistryResource,
            host::RenderSurfaceId,
        },
        ui::{
            AppUiExt, UiRuntimeFontConfiguration, UiRuntimePresentationAssociationsResource,
            UiRuntimePresentationBinding, UiRuntimeSlotId, UiScreen, UiTypedScreenId,
            UiTypedSource,
        },
    },
    prelude::AppRenderExt,
};
use runenui_core::{Color, StyleEnvironment, StyleProperties};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path},
};
use ui_definition::{AuthoredUiTemplate, UiNodeDefinition};
use ui_program::UiProgramSourceId;

use crate::project;

const DEFAULT_FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/fonts/JetBrainsMono-Regular.ttf"
));

fn initial_ui_fonts() -> UiRuntimeFontConfiguration {
    UiRuntimeFontConfiguration::new(vec![DEFAULT_FONT.to_vec()]).with_system_fonts()
}

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
    let template: AuthoredUiTemplate =
        ron::from_str(&content).with_context(|| format!("invalid UI source {}", path.display()))?;
    let diagnostics = ui_definition::workflow::validate_ui_template(&template);
    ensure!(
        diagnostics.is_empty(),
        "invalid UI template {}: {diagnostics:?}",
        path.display()
    );
    ensure!(
        template.templates.is_empty() && template.menus.is_empty(),
        "nested UI templates and menus are not supported by this native UI adapter"
    );

    Ok(template.root)
}

fn initial_ui_style() -> StyleEnvironment {
    // The selected native UI-only recipe clears to black. RunenUI's
    // unstyled text defaults to black, so supply visible host style
    // without changing the meaning of authored UiNodeDefinition labels.
    StyleEnvironment::default()
        .with_framework_defaults(StyleProperties::EMPTY.with_foreground(Color::WHITE))
}

fn present_initial_ui(app: &mut App, slot: UiRuntimeSlotId) -> Result<()> {
    // The accepted expert API permits initial styled binding. Until #1405
    // owns a managed styled-placement convenience, preserve exact managed
    // producer allocation/retirement rather than rebinding after present():
    // replacing a binding would queue withdrawal of the same producer lease.
    let surface = RenderSurfaceId::primary();
    let mut occupied = BTreeSet::<RenderFrameProducerId>::new();
    occupied.extend(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()?
            .bindings()
            .map(|binding| binding.producer_id),
    );
    occupied.extend(
        app.world()
            .resource::<SurfaceFrameSubmissionRegistryResource>()?
            .ordered_submissions()
            .into_iter()
            .map(|submission| submission.producer_id),
    );
    occupied.extend(
        app.world()
            .resource::<RunenUiPaintSubmissionRegistryResource>()?
            .ordered_submissions()
            .into_iter()
            .map(|submission| submission.producer_id),
    );

    let producer = app
        .world_mut()
        .resource_mut::<ManagedRenderProducerIds>()?
        .allocate(slot.value(), &occupied)?;
    let result = app
        .world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()?
        .bind(UiRuntimePresentationBinding::new(
            slot,
            producer,
            surface,
            initial_ui_style(),
        ));
    if let Err(error) = result {
        app.world_mut()
            .resource_mut::<ManagedRenderProducerIds>()?
            .release_owned(producer, slot.value());
        return Err(anyhow!("cannot present styled UI: {error}"));
    }
    Ok(())
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

        let report = app
            .ui()
            .mount_with_fonts(FileScreen { root }, &initial_ui_fonts());
        let slot = report
            .slot_id()
            .ok_or_else(|| anyhow!("UI mount rejected: {report:?}"))?;

        present_initial_ui(&mut app, slot)?;
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
    fn initial_ui_style_is_contrasting_with_native_black_clear() {
        assert_eq!(
            initial_ui_style()
                .framework_defaults()
                .foreground()
                .and_then(runenui_core::ColorValue::as_literal),
            Some(&Color::WHITE)
        );
    }

    #[test]
    fn generated_label_publishes_white_shaped_text_without_native_gpu() -> Result<()> {
        use engine::{
            plugins::{
                RenderPlugin, TimePlugin, UiPlugin, render::host::RenderSurfaceRegistryResource,
            },
            runtime::WindowStateRegistryResource,
        };

        let temp = tempfile::tempdir()?;
        let project_root = project::create(temp.path(), "app")?;
        let root = load_ui(&project_root, "ui/main.ron")?;

        let mut app = App::headless();
        app.add_plugin(TimePlugin);
        app.add_plugin(UiPlugin);
        app.add_plugin(RenderPlugin);

        let slot = app
            .ui()
            .mount_with_fonts(FileScreen { root }, &initial_ui_fonts())
            .slot_id()
            .ok_or_else(|| anyhow!("generated UI was not mounted"))?;

        let surface = RenderSurfaceId::primary();
        let mut windows = WindowStateRegistryResource::default();
        let native = windows.register_primary_window("Hello", (320, 160), 1.0, true);
        app.insert_resource(windows);
        app.world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()?
            .confirm_surface_attachment(surface, native, (320, 160))?;
        app.world_mut()
            .resource_mut::<UiRuntimePresentationAssociationsResource>()?
            .bind(UiRuntimePresentationBinding::new(
                slot,
                RenderFrameProducerId::try_from_raw(71).expect("valid test producer identity"),
                surface,
                initial_ui_style(),
            ))?;

        let app = app.run_for_frames(1)?;
        let paints = app
            .world()
            .resource::<RunenUiPaintSubmissionRegistryResource>()?;
        let submitted = paints.ordered_submissions_for_surface(surface);
        ensure!(
            submitted.len() == 1,
            "the generated UI must publish exactly one paint submission"
        );
        ensure!(
            submitted[0].publication.scene().items().iter().any(|item| {
                item.primitive()
                    .as_shaped_text_run()
                    .is_some_and(|run| run.foreground() == Color::WHITE)
            }),
            "the generated UI must publish visible white shaped text"
        );
        Ok(())
    }

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
