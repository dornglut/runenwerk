use super::tests::runner_with_frame_pacing;
use super::*;
use crate::plugins::render::host::{RenderSurfaceLifecycleState, RenderSurfaceRegistryResource};

struct PendingLifecycleApp;

impl runenui_core::UiApp for PendingLifecycleApp {
    type State = ();
    type Action = ();
    type HostProtocol = runenui_core::NoHostProtocol;

    fn initial_producer_budget(_: &()) -> runenui_core::ProducerBudget {
        runenui_core::ProducerBudget::new(1, 0).with_reconciliation(1, 0, 1)
    }

    fn action_producer_budget(_: &(), _: &()) -> runenui_core::ProducerBudget {
        runenui_core::ProducerBudget::new(1, 0).with_reconciliation(1, 1, 1)
    }

    fn interaction_producer_budget(_: &()) -> runenui_core::ProducerBudget {
        runenui_core::ProducerBudget::new(13, 9)
            .with_reconciliation(1, 1, 1)
            .with_work_starts(runenui_core::WorkStartBudget::new(0, 0, 0, 0, 1))
            .with_work_residency(runenui_core::WorkResidencyBudget::new(0, 0, 0, 0, 0, 2))
    }

    fn root(_: &()) -> impl runenui_core::View<()> {
        runenui_core::button("pending native lifecycle").on_activate(|| ())
    }

    fn update(_: &mut (), _: ()) -> impl runenui_core::IntoUpdateOutput<(), Self::HostProtocol> {}
}

fn pending_lifecycle_runner(backlog: usize) -> (WinitRunner, UiRuntimeSlotId) {
    use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};

    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner.state.world.insert_resource(InputState::new());
    runner.state.world.insert_resource(ActionState::new());
    runner
        .state
        .world
        .insert_resource(UiRuntimePresentationAssociationsResource::default());
    let window = NativeWindowId::primary();
    let context = runner
        .native_input_mut()
        .window_context(window, winit::event::DeviceId::dummy())
        .unwrap();
    let device = WinitRunner::native_ui_device(context).unwrap();
    let mut slots = UiRuntimeSlotsResource::default();
    let slot = slots
        .mount_app::<PendingLifecycleApp>(
            UiTypedScreenId::new("native.pending.lifecycle"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .unwrap();
    slots
        .test_request_first_typed_focus::<PendingLifecycleApp>(slot)
        .unwrap();
    let mut recipients = UiNativeInputRecipientsResource::default();
    recipients
        .bind(
            window,
            NativeInputRecipient::RunenUiSlot(slot),
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .unwrap(),
            &mut slots,
        )
        .unwrap();
    let epoch = recipients.binding_epoch_for_window(window).unwrap();
    runner.state.world.insert_resource(slots);
    runner.state.world.insert_resource(recipients);
    // Even a translator whose earlier key had a definite pre-queue nonclaim
    // must survive ordinary pending work at the native lifecycle boundary.
    runner.native_ui_keyboard_translator(window, slot, context.source, device, epoch);
    let slots = runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    slots
        .test_queue_unpumped_typed_actions::<PendingLifecycleApp>(slot, backlog, || ())
        .unwrap();
    if backlog != 0 {
        slots.test_mark_native_pending(slot).unwrap();
    }
    (runner, slot)
}

#[test]
fn pending_native_work_does_not_make_focus_loss_fatal() {
    let (mut runner, _) = pending_lifecycle_runner(390);
    runner
        .apply_native_focus_change(NativeWindowId::primary(), false)
        .expect("legal pending M13 work is not a native lifecycle failure");
    assert!(
        !runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap()
            .record(NativeWindowId::primary())
            .unwrap()
            .focused
    );
    assert!(
        !runner
            .native_ui_keyboards
            .contains_key(&NativeWindowId::primary())
    );
}

#[test]
fn original_preedit_is_not_replayed_when_earlier_work_retires_its_editable() {
    use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};
    use runenui_core::{
        TextAffinity, TextDocumentId, TextDocumentRevision, TextDocumentSnapshot, TextPosition,
        TextSelection, View,
    };

    struct RetiringIme;
    impl runenui_core::UiApp for RetiringIme {
        type State = bool;
        type Action = bool;
        type HostProtocol = runenui_core::NoHostProtocol;
        fn initial_producer_budget(_: &bool) -> runenui_core::ProducerBudget {
            runenui_core::ProducerBudget::new(1, 0).with_reconciliation(1, 0, 1)
        }
        fn action_producer_budget(_: &bool, _: &bool) -> runenui_core::ProducerBudget {
            runenui_core::ProducerBudget::new(1, 0).with_reconciliation(1, 1, 1)
        }
        fn interaction_producer_budget(_: &bool) -> runenui_core::ProducerBudget {
            runenui_core::ProducerBudget::new(13, 9)
                .with_reconciliation(1, 1, 1)
                .with_work_starts(runenui_core::WorkStartBudget::new(0, 0, 0, 0, 1))
                .with_work_residency(runenui_core::WorkResidencyBudget::new(0, 0, 0, 0, 0, 2))
        }
        fn root(visible: &bool) -> impl View<bool> {
            if *visible {
                let document = TextDocumentSnapshot::new(
                    TextDocumentId::new(1276),
                    TextDocumentRevision::new(0),
                );
                let caret = TextPosition::new(document, "", 0, TextAffinity::Downstream).unwrap();
                runenui_core::text_field(
                    document,
                    String::new(),
                    TextSelection::collapsed(caret),
                    runenui_core::SemanticEditableMode::SingleLine,
                    |_| true,
                )
                .unwrap()
                .into_element()
            } else {
                runenui_core::text("original editable retired").into_element()
            }
        }
        fn update(
            state: &mut bool,
            visible: bool,
        ) -> impl runenui_core::IntoUpdateOutput<bool, Self::HostProtocol> {
            *state = visible;
        }
    }

    for backlog in [1, 391] {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let window = NativeWindowId::primary();
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        runner
            .native_input_mut()
            .window_context(window, winit::event::DeviceId::dummy())
            .unwrap();
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<RetiringIme>(
                UiTypedScreenId::new("native.ime.retirement"),
                true,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        slots
            .test_request_first_typed_focus::<RetiringIme>(slot)
            .unwrap();
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                window,
                NativeInputRecipient::RunenUiSlot(slot),
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .unwrap();
        let scope = slots.native_input_ownership(slot).unwrap().scope().clone();
        slots
            .test_queue_unpumped_typed_actions::<RetiringIme>(slot, backlog, || false)
            .unwrap();
        slots.test_mark_native_pending(slot).unwrap();
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);
        runner
            .route_ui_ime_preedit(slot, "あ".to_owned(), None)
            .expect("exact rejected Start after original target retirement is nonfatal");
        let slots = runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .unwrap();
        let after = slots.native_input_ownership(slot).unwrap();
        assert_eq!(after.scope(), &scope);
        assert!(after.keyboard().composition_generation().is_none());
        assert!(after.keyboard().focused_node().is_none());
        assert_eq!(slots.test_typed_state::<RetiringIme>(slot), Some(&false));
        assert_eq!(slots.native_input_is_pending(slot).unwrap(), backlog > 1);
        let mut settled = false;
        for _ in 0..32 {
            match crate::plugins::ui::settle_native_ui_slot(&mut runner.state.world, slot) {
            Ok(()) => { settled = true; break; }
            Err(error) if error.is::<crate::plugins::ui::native_transaction::NativeUiMutationNeedsSettlement>() => {}
            Err(error) => panic!("retired original Start failed ordinary settlement: {error:?}"),
        }
        }
        assert!(settled);
        let slots = runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .unwrap();
        assert!(
            slots
                .native_input_ownership(slot)
                .unwrap()
                .keyboard()
                .composition_generation()
                .is_none()
        );
        assert_eq!(slots.test_typed_state::<RetiringIme>(slot), Some(&false));
    }
}

fn admit_lifecycle_touch(
    runner: &mut WinitRunner,
    slot: UiRuntimeSlotId,
) -> runenui_core::PointerId {
    let window = NativeWindowId::primary();
    let source = runner
        .native_input_mut()
        .window_context(window, winit::event::DeviceId::dummy())
        .unwrap();
    let device = WinitRunner::native_ui_device(source).unwrap();
    let epoch = runner
        .state
        .world
        .resource::<UiNativeInputRecipientsResource>()
        .unwrap()
        .binding_epoch_for_window(window)
        .unwrap();
    let slots = runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    let style = runenui_core::StyleEnvironment::default();
    let context = runenui_runtime::SurfaceBuildContext::tight(
        &style,
        runenui_runtime::LogicalSize::try_new(640.0, 480.0).unwrap(),
    );
    let publication = crate::plugins::ui::native_transaction::test_selected_slot_operation(
        slots,
        slot,
        |slots| slots.publish_surface_for_slot(slot, &context),
    )
    .unwrap();
    let scope = slots.input_ownership(slot).unwrap().scope().clone();
    let mut translator = NativeUiTouchTranslator {
        recipient: slot,
        source: source.source,
        binding_epoch: epoch,
        scope,
        backend_devices: HashSet::new(),
        touch: TouchInputState::default(),
    };
    let event = translator
        .transition(
            &Touch {
                device_id: winit::event::DeviceId::dummy(),
                phase: winit::event::TouchPhase::Started,
                location: PhysicalPosition::new(20.0, 20.0),
                force: None,
                id: 1,
            },
            device,
            LogicalPoint::new(20.0, 20.0).unwrap(),
            publication.input_context().clone(),
        )
        .unwrap();
    let pointer = event.pointer_id();
    slots.submit_native_pointer(slot, event).unwrap();
    let (_, status) = slots.pump_native_input_records(slot).unwrap();
    assert!(status.is_ok());
    runner.native_ui_touches.insert(window, translator);
    pointer
}

fn queue_lifecycle_work(runner: &mut WinitRunner, slot: UiRuntimeSlotId) {
    let slots = runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    slots
        .test_queue_unpumped_typed_actions::<PendingLifecycleApp>(slot, 1000, || ())
        .unwrap();
    slots.test_mark_native_pending(slot).unwrap();
}

#[test]
fn pending_touch_cancel_is_observed_by_later_common_slot_pumps() {
    let (mut runner, slot) = pending_lifecycle_runner(0);
    let pointer = admit_lifecycle_touch(&mut runner, slot);
    queue_lifecycle_work(&mut runner, slot);
    runner
        .cancel_native_ui_touch(NativeWindowId::primary())
        .expect("Cancel remains in the original M13 FIFO without fatal global-quiescence demand");
    let slots = runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    assert!(
        slots.native_cancellations_pending(slot).unwrap(),
        "admission is not terminal evidence"
    );
    assert!(
        slots
            .native_input_ownership(slot)
            .unwrap()
            .pointers()
            .iter()
            .any(|owner| owner.pointer_id() == pointer)
    );
    for _ in 0..32 {
        let (_, status) = slots.pump_native_input_records(slot).unwrap();
        assert!(matches!(
            status,
            Ok(()) | Err(crate::plugins::ui::UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        ));
        if !slots.native_cancellations_pending(slot).unwrap() {
            break;
        }
    }
    assert!(
        !slots.native_cancellations_pending(slot).unwrap(),
        "the actual later exact receipt discharges the expectation"
    );
    assert!(
        !slots
            .native_input_ownership(slot)
            .unwrap()
            .pointers()
            .iter()
            .any(|owner| owner.pointer_id() == pointer)
    );
}

#[test]
fn pending_native_retirement_quarantines_binding_until_exact_cancels() {
    use crate::plugins::ui::NativeInputRecipientError;
    let (mut runner, slot) = pending_lifecycle_runner(0);
    admit_lifecycle_touch(&mut runner, slot);
    queue_lifecycle_work(&mut runner, slot);
    let window = NativeWindowId::primary();
    assert!(runner.retire_native_input_recipient(window).unwrap());
    assert!(
        runner
            .state
            .world
            .resource::<UiNativeInputRecipientsResource>()
            .unwrap()
            .host_cancellation_pending(window)
    );
    let mut recipients = runner
        .state
        .world
        .remove_resource::<UiNativeInputRecipientsResource>()
        .unwrap();
    let windows = runner
        .state
        .world
        .remove_resource::<WindowStateRegistryResource>()
        .unwrap();
    let slots = runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    assert_eq!(
        recipients.bind(
            window,
            NativeInputRecipient::LegacyNativeConsumer,
            &windows,
            slots
        ),
        Err(NativeInputRecipientError::UnsettledNativeInput)
    );
    // Passive reconciliation cannot turn queue admission into successful cleanup.
    recipients.reconcile_host_cancellations(slots).unwrap();
    assert!(recipients.host_cancellation_pending(window));
    for _ in 0..32 {
        let (_, status) = slots.pump_native_input_records(slot).unwrap();
        assert!(matches!(
            status,
            Ok(()) | Err(crate::plugins::ui::UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        ));
        recipients.reconcile_host_cancellations(slots).unwrap();
        if !recipients.host_cancellation_pending(window) {
            break;
        }
    }
    assert!(!recipients.host_cancellation_pending(window));
    assert_eq!(
        recipients.bind(
            window,
            NativeInputRecipient::LegacyNativeConsumer,
            &windows,
            slots
        ),
        Ok(true)
    );
    runner.state.world.insert_resource(windows);
    runner.state.world.insert_resource(recipients);
}

#[test]
fn queued_native_space_cancel_clears_only_the_original_runtime_owner() {
    use runenui_core::{KeyLocation, KeyboardEvent, KeyboardPhase, LogicalKey, PhysicalKey};
    let (mut runner, slot) = pending_lifecycle_runner(0);
    let window = NativeWindowId::primary();
    let context = runner
        .native_input_mut()
        .window_context(window, winit::event::DeviceId::dummy())
        .unwrap();
    let device = WinitRunner::native_ui_device(context).unwrap();
    let key = |phase| {
        KeyboardEvent::new(
            phase,
            PhysicalKey::Space,
            LogicalKey::Space,
            KeyModifiers::NONE,
            false,
            KeyLocation::Standard,
            KeyboardCompositionState::Inactive,
            Some(device),
        )
    };
    let slots = runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    slots
        .submit_native_keyboard(slot, key(KeyboardPhase::Down))
        .unwrap();
    let (_, status) = slots.pump_native_input_records(slot).unwrap();
    assert!(status.is_ok());
    assert_eq!(
        slots
            .native_input_ownership(slot)
            .unwrap()
            .keyboard()
            .space_activation_device_id(),
        Some(device)
    );
    queue_lifecycle_work(&mut runner, slot);
    let slots = runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    let cancelled = slots
        .submit_native_keyboard(slot, key(KeyboardPhase::Cancel))
        .unwrap();
    let (batch, status) = slots.pump_native_input_records(slot).unwrap();
    assert!(native_m13_original_deferred(
        batch.final_ownership().scope(),
        &batch,
        &status
    ));
    assert!(
        batch
            .processed_through()
            .is_some_and(|end| end < cancelled.sequence())
    );
    assert!(slots.native_cancellations_pending(slot).unwrap());
    for _ in 0..32 {
        let (_, status) = slots.pump_native_input_records(slot).unwrap();
        assert!(matches!(
            status,
            Ok(()) | Err(crate::plugins::ui::UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        ));
        if !slots.native_cancellations_pending(slot).unwrap() {
            break;
        }
    }
    assert!(!slots.native_cancellations_pending(slot).unwrap());
    assert!(
        slots
            .native_input_ownership(slot)
            .unwrap()
            .keyboard()
            .space_activation_owner()
            .is_none()
    );
}

#[test]
fn retired_pending_cancel_cannot_be_certified_after_its_scope_is_lost() {
    use crate::plugins::ui::NativeInputRecipientError;
    let (mut runner, slot) = pending_lifecycle_runner(0);
    admit_lifecycle_touch(&mut runner, slot);
    queue_lifecycle_work(&mut runner, slot);
    let window = NativeWindowId::primary();
    assert!(runner.retire_native_input_recipient(window).unwrap());
    let mut recipients = runner
        .state
        .world
        .remove_resource::<UiNativeInputRecipientsResource>()
        .unwrap();
    let slots = runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    slots.test_inject_external_scope_loss(slot).unwrap();
    assert!(matches!(
        recipients.reconcile_host_cancellations(slots),
        Err(NativeInputRecipientError::OwnershipUnavailable(_))
    ));
    assert!(recipients.host_cancellation_pending(window));
    runner.state.world.insert_resource(recipients);
}

#[test]
fn close_neutral_keyboard_journals_original_unicode_atomically_without_editor_or_game_permission() {
    use runen_input::{DigitalState, KeyLocation, KeyboardInput, LogicalKey, ObservationOrigin};
    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner.state.world.insert_resource(InputState::new());
    runner.state.world.insert_resource(ActionState::new());
    let window = NativeWindowId::primary();
    let context = runner
        .native_input_mut()
        .window_context(window, winit::event::DeviceId::dummy())
        .unwrap();
    let input = KeyboardInput {
        physical_key: PhysicalKeyIdentity::code("KeyW"),
        logical_key: LogicalKey::Character("漢".to_owned()),
        location: KeyLocation::Standard,
        state: DigitalState::Pressed,
        repeat: false,
        origin: ObservationOrigin::SourceReport,
    };
    runner
        .apply_pending_close_keyboard_observation(
            window,
            context,
            input.clone(),
            Some("漢".to_owned()),
        )
        .unwrap();
    let journal = runner
        .state
        .world
        .resource::<PlatformWindowEventQueueResource>()
        .unwrap();
    assert_eq!(journal.events().len(), 2);
    assert!(
        matches!(&journal.events()[1].event, PlatformEvent::TextInput { text } if text == "漢")
    );
    assert!(
        journal
            .events()
            .iter()
            .all(|entry| !entry.authorized_for_legacy_editor())
    );
    assert!(
        !runner
            .state
            .world
            .resource::<ActionState>()
            .unwrap()
            .action_down(crate::plugins::action::WORLD_MOVE_UP)
    );
    runner
        .apply_pending_close_keyboard_observation(
            window,
            context,
            KeyboardInput {
                state: DigitalState::Released,
                ..input.clone()
            },
            None,
        )
        .unwrap();
    assert!(
        runner
            .state
            .world
            .resource::<InputState>()
            .unwrap()
            .keyboard_down_generation_in(context, &input.physical_key)
            .is_none()
    );
    let journal = runner
        .state
        .world
        .resource_mut::<PlatformWindowEventQueueResource>()
        .unwrap();
    journal.retire_frame();
    for _ in 0..4095 {
        journal
            .publish(PlatformWindowEvent::new(
                window,
                PlatformEvent::RedrawRequested,
            ))
            .unwrap();
    }
    assert!(
        runner
            .apply_pending_close_keyboard_observation(
                window,
                context,
                input.clone(),
                Some("漢".to_owned())
            )
            .is_err()
    );
    assert!(
        runner
            .state
            .world
            .resource::<InputState>()
            .unwrap()
            .keyboard_down_generation_in(context, &input.physical_key)
            .is_none()
    );
    assert_eq!(
        runner
            .state
            .world
            .resource::<PlatformWindowEventQueueResource>()
            .unwrap()
            .events()
            .len(),
        4095
    );
}

#[test]
fn refused_secondary_close_preserves_host_identity_and_redraw() {
    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    // This is a real source-retirement fixture: its required neutral reducer
    // must exist so the typed Render refusal, not missing InputState, is tested.
    runner.state.world.insert_resource(InputState::new());
    runner
        .state
        .world
        .insert_resource(RenderSurfaceRegistryResource::default());
    let requested = runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .unwrap()
        .request_window("secondary", (640, 480));
    let secondary = requested.native_window_id;
    let id = runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .reserve_surface_for_native_window(secondary, (640, 480));
    runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .confirm_surface_attachment(id, secondary, (640, 480))
        .unwrap();
    runner.owned_redraws.insert(secondary);
    let input = runner
        .native_input_mut()
        .window_context(secondary, winit::event::DeviceId::dummy())
        .unwrap();
    let source = runner.native_input().window_source_context(secondary);
    assert_eq!(source.map(|context| context.source), Some(input.source));

    let error = runner
        .authorize_secondary_close(secondary)
        .expect_err("an attached surface without GPU must fail closed");
    assert!(format!("{error:#}").contains("no GPU attachment"));
    assert!(runner.owned_redraws.contains(&secondary));
    assert_eq!(
        runner.native_input().window_source_context(secondary),
        source
    );
    assert!(
        runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap()
            .record(secondary)
            .is_some()
    );
    let render = runner
        .state
        .world
        .resource::<RenderSurfaceRegistryResource>()
        .unwrap();
    assert_eq!(render.surface_for_native_window(secondary), Some(id));
    assert_eq!(
        render.record(id).map(|record| record.lifecycle_state),
        Some(RenderSurfaceLifecycleState::Attached)
    );
    assert!(
        !runner
            .window_realizer
            .realized_windows()
            .iter()
            .any(|(_, id, _)| *id == secondary)
    );
}

#[test]
fn creation_failure_records_diagnostic_without_discarding_unresolved_render_attachment() {
    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner
        .state
        .world
        .insert_resource(RenderSurfaceRegistryResource::default());
    let request = runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .unwrap()
        .request_window("secondary", (640, 480));
    let surface = runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .reserve_surface_for_native_window(request.native_window_id, (640, 480));
    runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .confirm_surface_attachment(surface, request.native_window_id, (640, 480))
        .unwrap();

    let error = runner
        .mark_window_creation_failed(request.native_window_id, "injected failure")
        .expect_err("an attached Render surface without a GPU attachment cannot be retired");
    let error_text = format!("{error:#}");
    assert!(error_text.contains("injected failure"));
    assert!(error_text.contains("secondary rollback refused"));

    let record = runner
        .state
        .world
        .resource::<WindowStateRegistryResource>()
        .unwrap()
        .record(request.native_window_id)
        .unwrap();
    // Native creation failed. This does not mean Render cleanup succeeded.
    assert_eq!(
        record.lifecycle_state,
        crate::runtime::NativeWindowLifecycleState::CreationFailed
    );
    let reason = record.failure_reason.as_deref().unwrap();
    assert!(reason.contains("injected failure"));
    assert!(reason.contains("secondary rollback refused"));

    let render = runner
        .state
        .world
        .resource::<RenderSurfaceRegistryResource>()
        .unwrap();
    assert_eq!(
        render.surface_for_native_window(request.native_window_id),
        Some(surface),
        "a refused Render rollback must keep its owner mapping"
    );
    assert_eq!(
        render.record(surface).map(|record| record.lifecycle_state),
        Some(RenderSurfaceLifecycleState::Attached),
        "Native creation failure may not silently retire the Render surface"
    );
    assert!(
        !runner
            .window_realizer
            .realized_windows()
            .iter()
            .any(|(_, id, _)| *id == request.native_window_id),
        "failed creation cannot publish a realized native window"
    );
}

#[test]
fn external_window_receipt_requires_an_authoritative_terminal_native_record() {
    let mut registry = WindowStateRegistryResource::default();
    let request = registry.request_window("external", (640, 480));
    let native_window_id = request.native_window_id;
    let success: Result<()> = Ok(());

    let missing = WinitRunner::external_window_request_outcome(native_window_id, &success, None);
    assert!(matches!(
        missing,
        NativeWindowRequestOutcome::CreationFailedUnsettled {
            native_window_id: observed,
            reason,
        } if observed == native_window_id && reason.contains("terminal record")
    ));

    let requested = WinitRunner::external_window_request_outcome(
        native_window_id,
        &success,
        registry.record(native_window_id),
    );
    assert!(matches!(
        requested,
        NativeWindowRequestOutcome::CreationFailedUnsettled {
            native_window_id: observed,
            ..
        } if observed == native_window_id
    ));

    registry
        .record_mut(native_window_id)
        .unwrap()
        .mark_creation_failed("physical creation rejected and rolled back");
    assert_eq!(
        WinitRunner::external_window_request_outcome(
            native_window_id,
            &success,
            registry.record(native_window_id)
        ),
        NativeWindowRequestOutcome::CreationFailed {
            reason: "physical creation rejected and rolled back".to_string()
        }
    );

    registry.register_created_window(native_window_id, "external", (640, 480), 1.0, false);
    assert_eq!(
        WinitRunner::external_window_request_outcome(
            native_window_id,
            &success,
            registry.record(native_window_id)
        ),
        NativeWindowRequestOutcome::Created { native_window_id }
    );

    let failure: Result<()> = Err(anyhow::anyhow!("Render rollback refused"));
    let unsettled = WinitRunner::external_window_request_outcome(
        native_window_id,
        &failure,
        registry.record(native_window_id),
    );
    assert!(matches!(
        unsettled,
        NativeWindowRequestOutcome::CreationFailedUnsettled {
            native_window_id: observed,
            reason,
        } if observed == native_window_id && reason.contains("rollback refused")
    ));
}

#[test]
fn saturated_source_journal_cannot_retire_render_or_a_selected_ui_recipient() {
    use crate::plugins::ui::{
        NativeInputRecipient, UiRuntimeFontConfiguration, UiRuntimeSlotsResource, UiTypedScreenId,
    };

    struct Probe;
    impl runenui_core::UiApp for Probe {
        // One existing mounted text owner; no initial effects or subscriptions.
        fn initial_producer_budget(_state: &Self::State) -> runenui_core::ProducerBudget {
            runenui_core::ProducerBudget::new(1, 0).with_reconciliation(1, 0, 1)
        }

        fn action_producer_budget(
            _state: &Self::State,
            _action: &Self::Action,
        ) -> runenui_core::ProducerBudget {
            runenui_core::ProducerBudget::new(1, 0).with_reconciliation(1, 1, 1)
        }

        fn interaction_producer_budget(_state: &Self::State) -> runenui_core::ProducerBudget {
            runenui_core::ProducerBudget::new(1, 0)
        }

        type State = ();
        type Action = ();
        type HostProtocol = runenui_core::NoHostProtocol;

        fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
            runenui_core::text("native secondary close admission")
        }

        fn update(
            _: &mut Self::State,
            _: Self::Action,
        ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        }
    }

    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner.state.world.insert_resource(InputState::new());
    runner
        .state
        .world
        .insert_resource(RenderSurfaceRegistryResource::default());
    let secondary = NativeWindowId::try_from_raw(2).expect("secondary window");
    runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .unwrap()
        .register_created_window(secondary, "secondary", (640, 480), 1.0, false);
    let surface = runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .reserve_surface_for_native_window(secondary, (640, 480));
    let source = runner
        .native_input_mut()
        .window_context(secondary, winit::event::DeviceId::dummy())
        .expect("one source-qualified physical input");

    let mut slots = UiRuntimeSlotsResource::default();
    let slot = slots
        .mount_app::<Probe>(
            UiTypedScreenId::new("native.secondary.close.preflight"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .expect("original typed RunenUI runtime");
    let mut recipients = UiNativeInputRecipientsResource::default();
    recipients
        .bind(
            secondary,
            NativeInputRecipient::RunenUiSlot(slot),
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .unwrap(),
            &mut slots,
        )
        .expect("selected native UI recipient");
    runner.state.world.insert_resource(slots);
    runner.state.world.insert_resource(recipients);
    runner
        .state
        .world
        .insert_resource(UiRuntimePresentationAssociationsResource::default());

    let journal = runner
        .state
        .world
        .resource_mut::<PlatformWindowEventQueueResource>()
        .expect("single bounded native input journal");
    for _ in 0..4096 {
        journal
            .publish(PlatformWindowEvent::new(
                secondary,
                PlatformEvent::RedrawRequested,
            ))
            .expect("fill exactly the finite original-event capacity");
    }
    assert!(
        runner.authorize_secondary_close(secondary).is_err(),
        "the original source loss must refuse BEFORE Render is logically retired"
    );
    assert_eq!(
        runner
            .state
            .world
            .resource::<RenderSurfaceRegistryResource>()
            .unwrap()
            .surface_for_native_window(secondary),
        Some(surface),
        "journal pressure must not release an original secondary Render reservation"
    );
    assert_eq!(
        runner
            .native_input()
            .window_source_context(secondary)
            .map(|ctx| ctx.source),
        Some(source.source),
        "no fabricated source retirement on journal refusal"
    );
    assert_eq!(
        runner.native_input_recipient(secondary).unwrap().0,
        NativeInputRecipient::RunenUiSlot(slot),
        "no RunenUI recipient or binding revocation before original admission"
    );
    assert_eq!(
        runner
            .state
            .world
            .resource::<PlatformWindowEventQueueResource>()
            .unwrap()
            .events()
            .len(),
        4096,
        "the journal must retain its exact original events without partial mutation"
    );
}
