use engine::plugins::ui::{
    UiMountFailureReason, UiMountRequest, UiMountRequestsResource, UiMountSource, UiPlugin,
    UiRuntimeDiagnosticCode, UiRuntimeDiagnosticsResource,
};
use engine::prelude::{App, AppUiExt};

#[derive(Debug, Copy, Clone)]
struct CounterScreen;

impl From<CounterScreen> for UiMountRequest {
    fn from(_: CounterScreen) -> Self {
        UiMountRequest::new("CounterScreen")
    }
}

#[test]
fn ui_mount_normal_path_records_request_without_surface_factory_setup() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    app.mount_ui(CounterScreen);

    let requests = app.world().resource::<UiMountRequestsResource>().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests.reports().len(), 1);

    let record = &requests.records()[0];
    assert_eq!(record.screen_identity(), "CounterScreen");
    assert_eq!(record.mount_source(), UiMountSource::AppMountUi);
    assert_eq!(record.request(), &UiMountRequest::from(CounterScreen));
}

#[test]
fn ui_mount_advanced_path_records_equivalent_request_with_report_hook() {
    let request = UiMountRequest::from(CounterScreen).with_report_label("counter-root");

    let mut normal_app = App::headless();
    normal_app.add_plugin(UiPlugin).mount_ui(request.clone());
    let normal_record = normal_app
        .world()
        .resource::<UiMountRequestsResource>()
        .unwrap()
        .records()[0]
        .clone();

    let mut advanced_app = App::headless();
    advanced_app.add_plugin(UiPlugin);
    let report = advanced_app.ui().mount(request);
    let advanced_requests = advanced_app
        .world()
        .resource::<UiMountRequestsResource>()
        .unwrap();
    let advanced_record = &advanced_requests.records()[0];

    assert!(report.is_accepted());
    assert_eq!(report.mount_source(), UiMountSource::AppUiMount);
    assert_eq!(report.screen_identity(), "CounterScreen");
    assert_eq!(report.report_label(), Some("counter-root"));
    assert_eq!(normal_record.request(), advanced_record.request());
}

#[test]
fn ui_mount_diagnostics_include_screen_source_and_stable_failure_reason() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let report = app.ui().mount("");

    assert!(!report.is_accepted());
    assert_eq!(report.screen_identity(), "");
    assert_eq!(report.mount_source(), UiMountSource::AppUiMount);
    assert_eq!(
        report.failure_reason(),
        Some(UiMountFailureReason::MissingScreenIdentity)
    );

    let requests = app.world().resource::<UiMountRequestsResource>().unwrap();
    assert!(requests.records().is_empty());
    assert_eq!(requests.latest_report(), Some(&report));

    let diagnostics = app
        .world()
        .resource::<UiRuntimeDiagnosticsResource>()
        .unwrap();
    assert_eq!(diagnostics.len(), 1);

    let diagnostic = &diagnostics.entries()[0];
    assert_eq!(
        diagnostic.code,
        UiRuntimeDiagnosticCode::MountRequestRejected
    );
    let mount = diagnostic.mount.as_ref().unwrap();
    assert_eq!(mount.screen_identity, "");
    assert_eq!(mount.mount_source, UiMountSource::AppUiMount);
    assert_eq!(
        mount.failure_reason,
        UiMountFailureReason::MissingScreenIdentity
    );
}
