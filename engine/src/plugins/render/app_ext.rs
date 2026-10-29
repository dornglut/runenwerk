use anyhow::{Result, ensure};

use crate::app::App;
use crate::plugins::ui::{UiPlugin, UiPluginStateResource};

use super::inspect::{RenderDebugConfigResource, RenderDebugControlResource};
use super::{
    RenderFlow, RenderFlowId, RenderFlowRegistryResource, RenderPlugin,
    render_integration_is_active,
};

/// A selected native UI-only composition owns exactly one RenderFlow.
/// Keeping the registry revision also rejects later replacement of the same flow ID.
#[derive(Debug, Clone, Copy, runen_ecs::Resource)]
struct SelectedNativeUiOnlyComposition {
    flow_id: RenderFlowId,
    registry_revision: u64,
}

/// Checked during App composition admission, before native Host execution.
pub(crate) fn validate_selected_native_ui_only(app: &App) -> Result<()> {
    let Ok(selection) = app.world().resource::<SelectedNativeUiOnlyComposition>() else {
        return Ok(());
    };
    ensure!(
        app.is_native_window_host(),
        "native UI-only composition requires App::new() native-window Host"
    );
    ensure!(
        app.world()
            .resource::<UiPluginStateResource>()
            .is_ok_and(UiPluginStateResource::is_installed)
            && render_integration_is_active(app.world()),
        "native UI-only composition requires its selected UiPlugin and RenderPlugin"
    );
    let registry = app.world().resource::<RenderFlowRegistryResource>()?;
    ensure!(
        registry.is_only_flow(selection.flow_id)
            && registry.revision() == selection.registry_revision,
        "native UI-only composition conflicts with a changed or additional RenderFlow; remove the UI-only selection or use the explicit custom RenderFlow path"
    );
    Ok(())
}

pub trait AppRenderExt {
    /// Selects the one native-primary clear, built-in UI composite, Present recipe.
    ///
    /// Installs UiPlugin and RenderPlugin in that order. Call during App::new()
    /// configuration, before installing either plugin or registering custom flows.
    /// A game with its own RenderFlow must use the explicit path instead.
    /// The caller independently mounts and places typed UI via app.ui().
    /// A later RenderFlow mutation rejects App admission before startup.
    fn select_native_ui_only(&mut self) -> Result<&mut Self>;

    fn add_render_flow(&mut self, flow: RenderFlow) -> &mut Self;

    fn update_render_debug_control<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugControlResource);

    fn update_render_debug_config<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugConfigResource);
}

impl AppRenderExt for App {
    fn select_native_ui_only(&mut self) -> Result<&mut Self> {
        ensure!(
            self.allow_topology_mutation("select_native_ui_only", None),
            "native UI-only composition cannot be selected after App configuration"
        );
        ensure!(
            self.is_native_window_host(),
            "native UI-only composition requires App::new() native-window Host"
        );
        ensure!(
            !self
                .world()
                .has_resource::<SelectedNativeUiOnlyComposition>(),
            "native UI-only composition was already selected"
        );
        ensure!(
            !self
                .world()
                .resource::<UiPluginStateResource>()
                .is_ok_and(UiPluginStateResource::is_installed)
                && !render_integration_is_active(self.world()),
            "select native UI-only composition before UiPlugin or RenderPlugin; preselected capabilities cannot be installed again"
        );
        ensure!(
            self.world()
                .resource::<RenderFlowRegistryResource>()
                .map_or(true, |registry| registry.flow_count() == 0),
            "native UI-only composition conflicts with an existing RenderFlow; select the explicit custom RenderFlow path"
        );

        // Construct and validate the complete owned recipe before mutating App.
        let flow = RenderFlow::new("native.ui_only")
            .with_surface_color()?
            .fullscreen_pass("native.ui_only.clear")
            .main_surface_only()
            .clear_color([0.0, 0.0, 0.0, 1.0])
            .write_surface_color()?
            .finish()
            .builtin_ui_composite_pass("native.ui_only.composite")?
            .main_surface_only()
            .finish()
            .present_pass("native.ui_only.present")?
            .main_surface_only()
            .surface_color()?
            .finish()
            .validate()?;
        let flow_id = flow.id();

        self.add_plugin(UiPlugin);
        self.add_plugin(RenderPlugin);
        self.add_render_flow(flow);
        let registry_revision = self
            .world()
            .resource::<RenderFlowRegistryResource>()?
            .revision();
        self.insert_resource(SelectedNativeUiOnlyComposition {
            flow_id,
            registry_revision,
        });
        Ok(self)
    }

    fn add_render_flow(&mut self, flow: RenderFlow) -> &mut Self {
        if self
            .world()
            .resource::<RenderFlowRegistryResource>()
            .is_err()
        {
            self.insert_resource(RenderFlowRegistryResource::default());
        }
        if let Ok(registry) = self
            .world_mut()
            .resource_mut::<RenderFlowRegistryResource>()
        {
            registry.upsert_flow(flow);
        }
        self
    }

    fn update_render_debug_control<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugControlResource),
    {
        self.init_resource::<RenderDebugControlResource>();
        if let Ok(control) = self
            .world_mut()
            .resource_mut::<RenderDebugControlResource>()
        {
            update(control);
        }
        self
    }

    fn update_render_debug_config<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugConfigResource),
    {
        self.init_resource::<RenderDebugConfigResource>();
        if let Ok(config) = self.world_mut().resource_mut::<RenderDebugConfigResource>() {
            update(config);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::inspect::RenderCaptureSelector;
    use crate::plugins::render::{Gfx, RenderPlugin, render_integration_is_active};

    #[test]
    fn selected_native_ui_only_owns_one_render_flow_without_eager_gpu() {
        let mut app = App::new();
        app.select_native_ui_only()
            .expect("native UI-only composition must select");
        assert!(render_integration_is_active(app.world()));
        assert!(
            app.world()
                .resource::<UiPluginStateResource>()
                .is_ok_and(UiPluginStateResource::is_installed)
        );
        let registry = app
            .world()
            .resource::<RenderFlowRegistryResource>()
            .unwrap();
        assert_eq!(registry.flow_count(), 1);
        assert_eq!(registry.revision(), 1);
        assert!(app.world().resource::<Gfx>().is_err());
        app.admit_composition()
            .expect("selected native UI-only flow should admit");
    }

    #[test]
    fn native_ui_only_preexisting_custom_flow_fails_without_plugin_installation() {
        let mut app = App::new();
        app.add_render_flow(RenderFlow::new("game.owns.present"));
        let revision = app
            .world()
            .resource::<RenderFlowRegistryResource>()
            .unwrap()
            .revision();
        let error = app.select_native_ui_only().err().unwrap().to_string();
        assert!(error.contains("existing RenderFlow"), "{error}");
        assert!(!render_integration_is_active(app.world()));
        assert!(
            !app.world()
                .has_resource::<SelectedNativeUiOnlyComposition>()
        );
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .unwrap()
                .revision(),
            revision
        );
        app.admit_composition()
            .expect("rejected opt-in must not invalidate the existing expert graph");
    }

    #[test]
    fn native_ui_only_later_custom_flow_rejects_app_admission() {
        let mut app = App::new();
        app.select_native_ui_only().unwrap();
        app.add_render_flow(RenderFlow::new("game.additional.present"));
        let error = app.admit_composition().err().unwrap().to_string();
        assert!(
            error.contains("changed or additional RenderFlow"),
            "{error}"
        );
    }

    #[test]
    fn native_ui_only_later_retirement_or_same_id_replacement_rejects_admission() {
        let mut app = App::new();
        app.select_native_ui_only().unwrap();
        app.world_mut()
            .resource_mut::<RenderFlowRegistryResource>()
            .unwrap()
            .clear();
        assert!(app.admit_composition().is_err());

        let mut app = App::new();
        app.select_native_ui_only().unwrap();
        let flow_id = app
            .world()
            .resource::<SelectedNativeUiOnlyComposition>()
            .unwrap()
            .flow_id;
        // A direct registry removal followed by an unrelated flow must not be
        // mistaken for the original selected recipe.
        let registry = app
            .world_mut()
            .resource_mut::<RenderFlowRegistryResource>()
            .unwrap();
        assert!(registry.remove_flow(flow_id));
        registry.upsert_flow(RenderFlow::new("game.replacement"));
        assert!(app.admit_composition().is_err());
    }

    #[test]
    fn native_ui_only_headless_preflight_never_installs_gpu_or_capabilities() {
        let mut app = App::headless();
        let error = app.select_native_ui_only().err().unwrap().to_string();
        assert!(error.contains("native-window Host"), "{error}");
        assert!(!render_integration_is_active(app.world()));
        assert!(
            !app.world()
                .has_resource::<SelectedNativeUiOnlyComposition>()
        );
        assert!(!app.world().has_resource::<RenderFlowRegistryResource>());
        assert!(!app.world().has_resource::<Gfx>());
        app.admit_composition().unwrap();
    }

    #[test]
    fn native_ui_only_duplicate_selection_preserves_original_recipe() {
        let mut app = App::new();
        app.select_native_ui_only().unwrap();
        let revision = app
            .world()
            .resource::<RenderFlowRegistryResource>()
            .unwrap()
            .revision();
        let error = app.select_native_ui_only().err().unwrap().to_string();
        assert!(error.contains("already selected"), "{error}");
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .unwrap()
                .revision(),
            revision
        );
        app.admit_composition().unwrap();
    }

    #[test]
    fn native_ui_only_requires_unselected_plugins_in_any_order() {
        for ui_first in [true, false] {
            let mut app = App::new();
            if ui_first {
                app.add_plugin(UiPlugin);
            } else {
                app.add_plugin(RenderPlugin);
            }
            let error = app.select_native_ui_only().err().unwrap().to_string();
            assert!(error.contains("before UiPlugin or RenderPlugin"), "{error}");
            assert!(
            !app.world()
                .has_resource::<SelectedNativeUiOnlyComposition>()
        );
            assert!(
                !app.world().has_resource::<RenderFlowRegistryResource>()
                    || app
                        .world()
                        .resource::<RenderFlowRegistryResource>()
                        .unwrap()
                        .flow_count()
                        == 0
            );
            app.admit_composition().unwrap();
        }
    }

    #[test]
    fn native_ui_only_rejects_selection_after_app_lifecycle_preparation() {
        let mut app = App::new();
        app.prepare_lifecycle_for_execution().unwrap();
        assert!(app.select_native_ui_only().is_err());
        assert!(
            !app.world()
                .has_resource::<SelectedNativeUiOnlyComposition>()
        );
        assert!(app.admit_composition().is_err());
    }

    #[test]
    fn preplugin_flow_registration_is_preserved_without_activating_render() {
        let mut app = App::headless();

        app.add_render_flow(RenderFlow::new("render.owner.preplugin"));

        assert!(!render_integration_is_active(app.world()));
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .expect("owner extension should materialize the flow registry")
                .flow_count(),
            1
        );

        app.add_plugin(RenderPlugin);

        assert!(render_integration_is_active(app.world()));
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .expect("RenderPlugin should preserve the preconfigured flow registry")
                .flow_count(),
            1
        );
    }

    #[test]
    fn preplugin_debug_state_is_preserved_without_activating_render() {
        let mut app = App::headless();

        app.update_render_debug_control(|control| {
            control.provenance_enabled = true;
        });
        app.update_render_debug_config(|config| {
            config
                .capture_selectors
                .push(RenderCaptureSelector::named_pass_surface_color(
                    "render.owner.preplugin",
                    "surface",
                ));
        });

        assert!(!render_integration_is_active(app.world()));
        assert!(
            app.world()
                .resource::<RenderDebugControlResource>()
                .expect("owner extension should materialize debug control")
                .provenance_enabled
        );
        assert_eq!(
            app.world()
                .resource::<RenderDebugConfigResource>()
                .expect("owner extension should materialize debug config")
                .capture_selectors
                .len(),
            1
        );

        app.add_plugin(RenderPlugin);

        assert!(render_integration_is_active(app.world()));
        assert!(
            app.world()
                .resource::<RenderDebugControlResource>()
                .expect("RenderPlugin should preserve preconfigured debug control")
                .provenance_enabled
        );
        assert_eq!(
            app.world()
                .resource::<RenderDebugConfigResource>()
                .expect("RenderPlugin should preserve preconfigured debug config")
                .capture_selectors
                .len(),
            1
        );
    }
}
