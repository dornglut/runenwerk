use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::render::{RenderRuntimeSet, SurfaceFrameSubmissionRegistryResource};
use crate::runtime::{RenderPrepare, SystemConfigExt};

use super::{
    UiMountRequestsResource, UiPointerActivationResource, UiRuntimeDiagnosticsResource,
    UiRuntimeEvaluationResource, UiRuntimeFramePublicationResource,
    UiRuntimeFramePublicationTarget, UiRuntimeHitTargetResource, UiRuntimePreparedFrameResource,
    UiRuntimeReportResource, UiRuntimeResource, UiRuntimeSet, UiRuntimeTraceResource,
    publish_ui_runtime_frame_system,
};

/// Installs the engine-owned UI runtime foundation resources.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiRuntimeResource>();
        app.init_resource::<UiRuntimeDiagnosticsResource>();
        app.init_resource::<UiRuntimeReportResource>();
        app.init_resource::<UiMountRequestsResource>();
        app.init_resource::<UiRuntimeEvaluationResource>();
        app.init_resource::<UiRuntimeTraceResource>();
        app.init_resource::<UiRuntimeFramePublicationTarget>();
        app.init_resource::<UiRuntimePreparedFrameResource>();
        app.init_resource::<UiRuntimeHitTargetResource>();
        app.init_resource::<UiPointerActivationResource>();
        app.init_resource::<UiRuntimeFramePublicationResource>();
        app.init_resource::<SurfaceFrameSubmissionRegistryResource>();

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

        app.add_systems(
            RenderPrepare,
            publish_ui_runtime_frame_system
                .in_set(UiRuntimeSet::RenderPublication)
                .before(RenderRuntimeSet::FramePrepare),
        );
    }
}
