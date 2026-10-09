use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{RenderPrepare, RenderSubmit, SystemConfigExt, SystemMobilityExt};

use super::{
    UiPluginStateResource, UiRuntimeDiagnosticsResource, UiRuntimePresentationAssociationsResource,
    UiRuntimeReportResource, UiRuntimeSet, UiRuntimeSlotsResource,
    apply_runenui_terminal_presentations_system, publish_runenui_bound_surfaces_system,
};

/// Installs the Engine-owned RunenUI integration resources.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiPluginStateResource>();
        app.init_resource::<UiRuntimeDiagnosticsResource>();
        app.init_resource::<UiRuntimeReportResource>();
        app.init_resource::<UiRuntimeSlotsResource>();
        app.init_resource::<UiRuntimePresentationAssociationsResource>();
        app.add_systems(
            RenderPrepare,
            publish_runenui_bound_surfaces_system.in_set(UiRuntimeSet::RenderPublication),
        );
        app.add_systems(
            RenderSubmit,
            apply_runenui_terminal_presentations_system
                .after_if_present(crate::plugins::render::runtime::frame_render_submit_system)
                .on_invoker_thread(),
        );

        let diagnostic_count = app
            .world()
            .resource::<UiRuntimeDiagnosticsResource>()
            .map(|diagnostics| diagnostics.len())
            .unwrap_or_default();

        if let Ok(plugin_state) = app.world_mut().resource_mut::<UiPluginStateResource>() {
            plugin_state.mark_installed();
        }

        if let Ok(report) = app.world_mut().resource_mut::<UiRuntimeReportResource>() {
            report.record_plugin_installed(diagnostic_count);
        }
    }
}
