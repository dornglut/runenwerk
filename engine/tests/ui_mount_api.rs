use engine::plugins::ui::{
    UiMountConfig, UiMountFailureReason, UiMountRequest, UiMountRequestsResource, UiMountSource,
    UiPlugin, UiRuntimeDiagnosticCode, UiRuntimeDiagnosticsResource,
};
use engine::prelude::{App, AppUiExt};
use ui_surface::{SessionRetentionClass, SurfaceInstanceId};

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
    assert_eq!(requests.mounted_sessions().len(), 1);
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
    assert!(report.mounted_surface().is_some());
    assert!(report.session().is_some());
    assert_eq!(normal_record.request(), advanced_record.request());
}

#[test]
fn ui_mount_creates_surface_session_record_with_generation_retention_and_report() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let report = app.ui().mount(
        UiMountRequest::from(CounterScreen).with_config(
            UiMountConfig::new()
                .with_report_label("counter-root")
                .with_retention_class(SessionRetentionClass::Persistent),
        ),
    );

    assert!(report.is_accepted());
    assert_eq!(report.report_label(), Some("counter-root"));
    assert_eq!(report.retention_class(), SessionRetentionClass::Persistent);
    assert_eq!(
        report.surface_instance_id(),
        Some(SurfaceInstanceId::new(1))
    );
    assert_eq!(report.generation(), Some(1));
    assert_eq!(report.session_scope_id(), Some(1));

    let mounted_surface = report
        .mounted_surface()
        .expect("accepted UI mount should report the mounted surface");
    let session = report
        .session()
        .expect("accepted UI mount should report the session handle");
    assert_eq!(
        mounted_surface.surface_instance_id,
        session.surface_instance_id
    );
    assert_eq!(session.retention_class, SessionRetentionClass::Persistent);

    let requests = app.world().resource::<UiMountRequestsResource>().unwrap();
    assert_eq!(requests.mounted_generation(), 1);
    assert_eq!(requests.mounted_sessions().len(), 1);
    assert_eq!(
        requests.mounted_surface(mounted_surface.surface_instance_id),
        Some(mounted_surface)
    );

    let mounted_session = &requests.mounted_sessions()[0];
    assert_eq!(mounted_session.screen_identity(), "CounterScreen");
    assert_eq!(mounted_session.mount_source(), UiMountSource::AppUiMount);
    assert_eq!(mounted_session.report_label(), Some("counter-root"));
    assert_eq!(mounted_session.mounted_surface(), mounted_surface);
    assert_eq!(mounted_session.session(), session);
}

#[test]
fn ui_mount_unmount_and_remount_are_deterministic() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let first = app.ui().mount(CounterScreen);
    let first_surface_id = first
        .surface_instance_id()
        .expect("accepted mount should have a surface instance id");
    assert_eq!(first_surface_id, SurfaceInstanceId::new(1));
    assert_eq!(first.generation(), Some(1));

    let requests = app
        .world_mut()
        .resource_mut::<UiMountRequestsResource>()
        .unwrap();
    let unmount = requests.unmount_surface(first_surface_id);
    assert!(unmount.is_removed());
    assert_eq!(unmount.surface_instance_id(), first_surface_id);
    assert_eq!(unmount.generation(), 2);
    assert_eq!(unmount.remaining_mounted_surfaces(), 0);
    assert!(requests.mounted_sessions().is_empty());

    let second = app.ui().mount(CounterScreen);
    assert_eq!(
        second.surface_instance_id(),
        Some(SurfaceInstanceId::new(2))
    );
    assert_eq!(second.session_scope_id(), Some(2));
    assert_eq!(second.generation(), Some(3));

    let requests = app.world().resource::<UiMountRequestsResource>().unwrap();
    assert_eq!(requests.mounted_generation(), 3);
    assert_eq!(requests.unmount_reports(), &[unmount]);
}

#[test]
fn ui_mount_multiple_screens_do_not_collide() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let counter = app.ui().mount(CounterScreen);
    let inventory = app.ui().mount("InventoryScreen");

    assert_eq!(
        counter.surface_instance_id(),
        Some(SurfaceInstanceId::new(1))
    );
    assert_eq!(
        inventory.surface_instance_id(),
        Some(SurfaceInstanceId::new(2))
    );
    assert_ne!(counter.host_instance_id(), inventory.host_instance_id());
    assert_ne!(counter.definition_id(), inventory.definition_id());
    assert_ne!(counter.session_scope_id(), inventory.session_scope_id());

    let requests = app.world().resource::<UiMountRequestsResource>().unwrap();
    assert_eq!(requests.mounted_sessions().len(), 2);
    assert_eq!(requests.mounted_generation(), 2);
    assert_eq!(requests.mounted_surfaces().count(), 2);
    assert_eq!(
        requests
            .mounted_surface(SurfaceInstanceId::new(1))
            .expect("first mounted surface should remain active")
            .generation,
        2
    );
    assert_eq!(
        requests
            .mounted_surface(SurfaceInstanceId::new(2))
            .expect("second mounted surface should remain active")
            .generation,
        2
    );
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
