use crate::app::App;
use crate::plugin::Plugin;

use super::{UiRuntimeDiagnosticsResource, UiRuntimeReportResource, UiRuntimeResource};

/// Installs the engine-owned UI runtime foundation resources.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiRuntimeResource>();
        app.init_resource::<UiRuntimeDiagnosticsResource>();
        app.init_resource::<UiRuntimeReportResource>();

        let diagnostic_count = app
            .world()
            .resource::<UiRuntimeDiagnosticsResource>()
            .map(|diagnostics| diagnostics.len())
            .unwrap_or_default();

        if let Ok(runtime) = app.world_mut().resource_mut::<UiRuntimeResource>() {
            runtime.mark_installed();
        }

        if let Ok(report) = app.world_mut().resource_mut::<UiRuntimeReportResource>() {
            report.record_plugin_installed(diagnostic_count);
        }
    }
}
