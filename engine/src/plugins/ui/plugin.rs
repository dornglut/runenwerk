use crate::app::App;
use crate::plugin::Plugin;

use super::{
    UiPluginStateResource, UiRuntimeDiagnosticsResource, UiRuntimeReportResource,
    UiRuntimeSlotsResource,
};

/// Installs the Engine-owned RunenUI integration resources.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiPluginStateResource>();
        app.init_resource::<UiRuntimeDiagnosticsResource>();
        app.init_resource::<UiRuntimeReportResource>();
        app.init_resource::<UiRuntimeSlotsResource>();

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
