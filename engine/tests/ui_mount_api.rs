use engine::plugins::ui::{
    NativeInputRecipient, NativeInputRecipientError, UiNativeInputRecipientsResource, UiPlugin,
    UiRuntimeDiagnosticCode, UiRuntimeDiagnosticsResource, UiRuntimeFontConfiguration,
    UiRuntimeSlotId, UiRuntimeSlotMountFailure, UiRuntimeSlotsResource, UiScreen, UiTypedScreenId,
    UiTypedSource,
};
use engine::prelude::{App, AppUiExt};
use engine::runtime::WindowStateRegistryResource;
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
fn native_primary_input_binding_is_explicit_before_window_creation() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let first = app.ui().mount(CounterScreen);
    let second = app.ui().mount(CounterScreen);
    let first_slot = first.slot_id().expect("first mounted UI slot");
    let second_slot = second.slot_id().expect("second mounted UI slot");
    let mut windows = WindowStateRegistryResource::default();
    windows.register_primary_window("primary", (640, 480), 1.0, true);
    app.world_mut().insert_resource(windows);

    assert!(
        !engine::plugins::ui::realize_requested_native_primary(app.world_mut()).unwrap(),
        "UI mounting alone must not choose a semantic native input target"
    );
    assert_eq!(app.ui().bind_native_primary_input(first_slot), Ok(()));
    assert_eq!(
        app.ui().bind_native_primary_input(first_slot),
        Ok(()),
        "same explicit application request is idempotent"
    );
    assert_eq!(
        app.ui().bind_native_primary_input(second_slot),
        Err(NativeInputRecipientError::ConflictingRecipient),
        "a displayed second UI slot must not steal the input target"
    );

    assert!(
        engine::plugins::ui::realize_requested_native_primary(app.world_mut()).unwrap(),
        "explicit application selection is bound only against the created window"
    );
    let windows = app
        .world_mut()
        .remove_resource::<WindowStateRegistryResource>()
        .expect("created native window record");
    let recipients = app
        .world_mut()
        .remove_resource::<UiNativeInputRecipientsResource>()
        .expect("registry remains the single native input owner");
    let mut slots = app
        .world_mut()
        .remove_resource::<UiRuntimeSlotsResource>()
        .expect("slot identity remains mounted");
    assert_eq!(
        recipients.resolve(
            engine::runtime::NativeWindowId::primary(),
            &windows,
            &mut slots
        ),
        Ok(NativeInputRecipient::RunenUiSlot(first_slot))
    );
    app.world_mut().insert_resource(slots);
    app.world_mut().insert_resource(recipients);
    app.world_mut().insert_resource(windows);
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
    assert_eq!(slots.unmount(slot_id), Ok(true));
    assert!(slots.is_empty());
    assert!(!slots.contains(slot_id));
    assert_eq!(slots.unmount(slot_id), Ok(false));
}

struct DiagnosticTypedApp;

impl runenui_core::UiApp for DiagnosticTypedApp {
    type State = ();
    type Action = ();
    type HostProtocol = runenui_core::NoHostProtocol;

    fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
        runenui_core::text("typed")
    }

    fn update(
        _: &mut Self::State,
        (): Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        runenui_core::Effects::none()
    }
}

#[test]
fn typed_app_mount_rejection_records_the_same_engine_diagnostic_as_counter() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let fonts = UiRuntimeFontConfiguration::new(vec![vec![0, 1, 2, 3]]);
    let report = app.ui().mount_app::<DiagnosticTypedApp>(
        UiTypedScreenId::new("typed.invalid-font"),
        (),
        &fonts,
    );
    assert_eq!(
        report.failure(),
        Some(UiRuntimeSlotMountFailure::FontProvisioning)
    );
    assert!(
        app.world()
            .resource::<UiRuntimeSlotsResource>()
            .unwrap()
            .is_empty()
    );

    let diagnostics = app
        .world()
        .resource::<UiRuntimeDiagnosticsResource>()
        .unwrap();
    let last = diagnostics
        .entries()
        .last()
        .expect("typed mount rejection diagnostic");
    assert_eq!(last.code, UiRuntimeDiagnosticCode::RuntimeSlotMountRejected);
    let facts = last
        .runtime_slot_mount
        .as_ref()
        .expect("typed mount rejection preserves exact screen and reason");
    assert_eq!(facts.screen_id, "typed.invalid-font");
    assert_eq!(
        facts.failure_reason,
        UiRuntimeSlotMountFailure::FontProvisioning
    );
}
