use engine::plugins::ui::{
    UiPlugin, UiRuntimeDiagnosticCode, UiRuntimeDiagnosticsResource, UiRuntimeSlotId,
    UiRuntimeSlotMountFailure, UiRuntimeSlotsResource, UiScreen, UiTypedScreenId, UiTypedSource,
};
use engine::prelude::{App, AppUiExt};
use ui_definition::{AuthoredId, UiNodeDefinition, UiValueBinding};
use ui_program::UiProgramSourceId;

#[derive(Debug, Copy, Clone)]
struct CounterScreen;

impl UiScreen for CounterScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new("counter.screen")
    }

    fn build_source(&self) -> UiTypedSource {
        UiTypedSource::new(
            self.screen_id(),
            UiProgramSourceId::new("counter.screen.source"),
            UiNodeDefinition::Column {
                id: AuthoredId::new("counter.root"),
                children: vec![UiNodeDefinition::Label {
                    id: AuthoredId::new("counter.title"),
                    label: UiValueBinding::static_text("Counter"),
                    availability: None,
                }],
            },
        )
    }
}

#[test]
fn ui_mount_normal_path_records_typed_slot_rejection_diagnostic() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    app.mount_ui(UnsupportedScreen);

    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert!(slots.is_empty());

    let diagnostics = app
        .world()
        .resource::<UiRuntimeDiagnosticsResource>()
        .unwrap();
    let diagnostic = diagnostics
        .entries()
        .last()
        .expect("rejected ordinary mount should remain observable");
    assert_eq!(
        diagnostic.code,
        UiRuntimeDiagnosticCode::RuntimeSlotMountRejected
    );
    let mount = diagnostic
        .runtime_slot_mount
        .as_ref()
        .expect("slot rejection should include owner-accurate mount facts");
    assert_eq!(mount.screen_id, "unsupported.screen");
    assert_eq!(
        mount.failure_reason,
        UiRuntimeSlotMountFailure::UnsupportedProjection
    );
}

#[derive(Debug, Copy, Clone)]
struct UnsupportedScreen;

impl UiScreen for UnsupportedScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new("unsupported.screen")
    }

    fn build_source(&self) -> UiTypedSource {
        UiTypedSource::new(
            self.screen_id(),
            UiProgramSourceId::new("unsupported.screen.source"),
            UiNodeDefinition::Spacer {
                id: AuthoredId::new("unsupported.spacer"),
            },
        )
    }
}

#[test]
fn ui_mount_normal_path_creates_runenui_runtime_slot() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    app.mount_ui(CounterScreen);

    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert_eq!(slots.len(), 1);
    let slot_id = slots.slot_ids().next().expect("one slot should be mounted");
    assert_eq!(slot_id.value(), 1);
    assert!(slots.contains(slot_id));
    assert_eq!(
        slots.screen_id(slot_id).map(UiTypedScreenId::as_str),
        Some("counter.screen")
    );
}

#[test]
fn ui_mount_advanced_path_returns_engine_slot_report() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let report = app.ui().mount(CounterScreen);

    assert!(report.accepted_mount(), "{report:?}");
    assert_eq!(report.screen_id().as_str(), "counter.screen");
    assert_eq!(report.slot_id().map(UiRuntimeSlotId::value), Some(1));
    assert!(report.failure().is_none());
}

#[test]
fn ui_mount_repeated_screen_instances_have_distinct_engine_slots() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let first = app.ui().mount(CounterScreen);
    let second = app.ui().mount(CounterScreen);

    assert_eq!(first.slot_id().map(UiRuntimeSlotId::value), Some(1));
    assert_eq!(second.slot_id().map(UiRuntimeSlotId::value), Some(2));
    assert_ne!(first.slot_id(), second.slot_id());

    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert_eq!(slots.len(), 2);
    assert_eq!(
        slots
            .screen_id(first.slot_id().unwrap())
            .map(UiTypedScreenId::as_str),
        Some("counter.screen")
    );
    assert_eq!(
        slots
            .screen_id(second.slot_id().unwrap())
            .map(UiTypedScreenId::as_str),
        Some("counter.screen")
    );
}

#[test]
fn ui_mount_unmount_shuts_down_and_removes_engine_slot() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let report = app.ui().mount(CounterScreen);
    let slot_id = report.slot_id().unwrap();

    let slots = app
        .world_mut()
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    assert!(slots.unmount(slot_id));
    assert!(slots.is_empty());
    assert!(!slots.contains(slot_id));
    assert!(!slots.unmount(slot_id));
}
