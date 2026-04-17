pub(crate) mod adapters;
pub(crate) mod core;

pub use adapters::{
    DiagnosticsConsoleAdapterStateResource, DiagnosticsConsoleFeedResource,
    DiagnosticsFileAdapterStateResource, DiagnosticsStdoutAdapterStateResource,
};
pub use core::{
    DiagnosticsAttachment, DiagnosticsConfigResource, DiagnosticsEntry, DiagnosticsFrameReport,
    DiagnosticsPendingReportsResource, DiagnosticsReportStoreResource, DiagnosticsSeverity,
    DiagnosticsStatus, DiagnosticsSummary, ResolvedDiagnosticsPlan, submit_diagnostics_entry,
};

use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CoreSet, FrameEnd, PreUpdate, SystemConfigExt};

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiagnosticsConfigResource>();
        app.init_resource::<ResolvedDiagnosticsPlan>();
        app.init_resource::<DiagnosticsPendingReportsResource>();
        app.init_resource::<DiagnosticsReportStoreResource>();
        app.init_resource::<DiagnosticsStdoutAdapterStateResource>();
        app.init_resource::<DiagnosticsFileAdapterStateResource>();
        app.init_resource::<DiagnosticsConsoleAdapterStateResource>();
        app.init_resource::<DiagnosticsConsoleFeedResource>();

        app.add_systems(PreUpdate, core::resolve_diagnostics_plan_system);

        app.add_systems(
            FrameEnd,
            core::finalize_diagnostics_reports_system.before(CoreSet::FrameEnd),
        );
        app.add_systems(
            FrameEnd,
            adapters::emit_stdout_adapter_system.before(CoreSet::FrameEnd),
        );
        app.add_systems(
            FrameEnd,
            adapters::persist_reports_and_manifest_system.before(CoreSet::FrameEnd),
        );
        app.add_systems(
            FrameEnd,
            adapters::emit_console_feed_system.before(CoreSet::FrameEnd),
        );
    }
}
