use crate::app::App;

use super::{
    UiMountReport, UiMountRequest, UiMountRequestsResource, UiMountSource, UiRuntimeDiagnostic,
    UiRuntimeDiagnosticsResource,
};

pub trait AppUiExt {
    fn mount_ui<S>(&mut self, screen: S) -> &mut Self
    where
        S: Into<UiMountRequest>;

    fn ui(&mut self) -> UiAppMounting<'_>;
}

impl AppUiExt for App {
    fn mount_ui<S>(&mut self, screen: S) -> &mut Self
    where
        S: Into<UiMountRequest>,
    {
        record_ui_mount(self, screen.into(), UiMountSource::AppMountUi);
        self
    }

    fn ui(&mut self) -> UiAppMounting<'_> {
        UiAppMounting { app: self }
    }
}

pub struct UiAppMounting<'a> {
    app: &'a mut App,
}

impl UiAppMounting<'_> {
    pub fn mount<S>(&mut self, screen: S) -> UiMountReport
    where
        S: Into<UiMountRequest>,
    {
        record_ui_mount(self.app, screen.into(), UiMountSource::AppUiMount)
    }
}

fn record_ui_mount(
    app: &mut App,
    request: UiMountRequest,
    mount_source: UiMountSource,
) -> UiMountReport {
    app.init_resource::<UiMountRequestsResource>();
    app.init_resource::<UiRuntimeDiagnosticsResource>();

    let report = {
        let mount_requests = app
            .world_mut()
            .resource_mut::<UiMountRequestsResource>()
            .expect("UiMountRequestsResource was initialized before recording a UI mount");
        mount_requests.record_mount_request(request, mount_source)
    };

    if let Some(reason) = report.failure_reason()
        && let Ok(diagnostics) = app
            .world_mut()
            .resource_mut::<UiRuntimeDiagnosticsResource>()
    {
        diagnostics.push(UiRuntimeDiagnostic::mount_rejected(
            report.screen_identity().to_string(),
            report.mount_source(),
            reason,
        ));
    }

    report
}
