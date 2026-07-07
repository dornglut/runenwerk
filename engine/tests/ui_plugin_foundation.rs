use engine::plugins::ui::{
    UiPlugin, UiRuntimeDiagnosticsResource, UiRuntimeInstallState, UiRuntimeReport,
    UiRuntimeReportResource, UiRuntimeResource, UiRuntimeSet,
};
use engine::prelude::App;
use engine::runtime::IntoSystemSetKey;

#[test]
fn ui_plugin_installs_foundation_resources_without_panicking() {
    let mut app = App::headless();

    app.add_plugin(UiPlugin);

    let runtime = app.world().resource::<UiRuntimeResource>().unwrap();
    assert!(runtime.is_installed());
    assert_eq!(runtime.install_state(), UiRuntimeInstallState::Installed);

    let diagnostics = app
        .world()
        .resource::<UiRuntimeDiagnosticsResource>()
        .unwrap();
    assert!(diagnostics.is_empty());

    let report = app.world().resource::<UiRuntimeReportResource>().unwrap();
    assert_eq!(
        report.latest(),
        UiRuntimeReport {
            install_state: UiRuntimeInstallState::Installed,
            diagnostic_count: 0,
        }
    );
}

#[test]
fn ui_plugin_install_is_idempotent_for_foundation_resources() {
    let mut app = App::headless();

    app.add_plugin(UiPlugin);
    let runtime_after_first = app.world().resource::<UiRuntimeResource>().unwrap().clone();
    let diagnostics_after_first = app
        .world()
        .resource::<UiRuntimeDiagnosticsResource>()
        .unwrap()
        .clone();
    let report_after_first = app
        .world()
        .resource::<UiRuntimeReportResource>()
        .unwrap()
        .clone();

    app.add_plugin(UiPlugin);

    assert_eq!(
        app.world().resource::<UiRuntimeResource>().unwrap(),
        &runtime_after_first
    );
    assert_eq!(
        app.world()
            .resource::<UiRuntimeDiagnosticsResource>()
            .unwrap(),
        &diagnostics_after_first
    );
    assert_eq!(
        app.world().resource::<UiRuntimeReportResource>().unwrap(),
        &report_after_first
    );
}

#[test]
fn ui_plugin_default_resources_are_stable() {
    assert_eq!(
        UiRuntimeResource::default().install_state(),
        UiRuntimeInstallState::Uninstalled
    );
    assert_eq!(UiRuntimeDiagnosticsResource::default().entries(), &[]);
    assert_eq!(
        UiRuntimeReportResource::default().latest(),
        UiRuntimeReport::default()
    );
}

#[test]
fn ui_plugin_schedule_labels_are_stable() {
    let foundation = UiRuntimeSet::Foundation.system_set_key();
    let report = UiRuntimeSet::Report.system_set_key();

    assert_eq!(foundation.name(), "UiRuntimeSet::Foundation");
    assert_eq!(report.name(), "UiRuntimeSet::Report");
    assert_ne!(foundation, report);
}
