use super::*;
use runen_input::{
    AnalogMeasurement, ContactId, ContactInput, CoordinateSpace, InputContext, InputDeviceId,
    InputSourceId, KeyLocation, KeyboardInput, PhysicalKeyIdentity, Point2, PointerButtonInput,
    ScrollDelta, ScrollInput,
};

fn window() -> NativeWindowId {
    NativeWindowId::primary()
}

fn scoped_context(source: u64, device: Option<u64>) -> InputContext {
    InputContext::new(InputSourceId::new(source), device.map(InputDeviceId::new))
}

fn context(device: Option<u64>) -> InputContext {
    scoped_context(7, device)
}

fn keyboard_in(
    context: InputContext,
    physical: &str,
    logical: LogicalKey,
    state: DigitalState,
    repeat: bool,
    origin: ObservationOrigin,
) -> PlatformEvent {
    PlatformEvent::KeyboardInput {
        context,
        input: KeyboardInput {
            physical_key: PhysicalKeyIdentity::code(physical),
            logical_key: logical,
            location: KeyLocation::Standard,
            state,
            repeat,
            origin,
        },
    }
}

fn keyboard(
    physical: &str,
    logical: LogicalKey,
    state: DigitalState,
    repeat: bool,
    origin: ObservationOrigin,
) -> PlatformEvent {
    keyboard_in(context(Some(9)), physical, logical, state, repeat, origin)
}

fn one(runtime: &mut EditorTargetInputRuntimeResource, event: PlatformEvent) -> UiInputEvent {
    let events = translate_platform_event(runtime, window(), event);
    assert_eq!(events.len(), 1);
    events.into_iter().next().unwrap()
}

#[test]
fn logical_keyboard_meaning_not_physical_code_drives_ui_key() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let event = one(
        &mut runtime,
        keyboard(
            "KeyZ",
            LogicalKey::Character("y".to_owned()),
            DigitalState::Pressed,
            false,
            ObservationOrigin::SourceReport,
        ),
    );
    assert!(matches!(
        event,
        UiInputEvent::Keyboard(KeyboardEvent {
            key: Key::Character(ref value),
            state: KeyState::Pressed,
            ..
        }) if value == "y"
    ));
}

#[test]
fn repeat_and_reconciliation_remain_distinct() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let repeated = one(
        &mut runtime,
        keyboard(
            "KeyA",
            LogicalKey::Character("a".to_owned()),
            DigitalState::Pressed,
            true,
            ObservationOrigin::SourceReport,
        ),
    );
    assert!(matches!(
        repeated,
        UiInputEvent::Keyboard(KeyboardEvent {
            state: KeyState::Repeated,
            ..
        })
    ));

    let reconciled = translate_platform_event(
        &mut runtime,
        window(),
        keyboard(
            "ShiftLeft",
            LogicalKey::Named("Shift".to_owned()),
            DigitalState::Pressed,
            false,
            ObservationOrigin::BackendSyntheticReconciliation,
        ),
    );
    assert!(reconciled.is_empty());
    let following = one(
        &mut runtime,
        keyboard(
            "KeyB",
            LogicalKey::Character("b".to_owned()),
            DigitalState::Pressed,
            false,
            ObservationOrigin::SourceReport,
        ),
    );
    assert!(matches!(
        following,
        UiInputEvent::Keyboard(KeyboardEvent { modifiers, .. }) if modifiers.shift
    ));
}

#[test]
fn left_and_right_modifier_lifetimes_do_not_alias() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    for physical in ["ShiftLeft", "ShiftRight"] {
        let _ = translate_platform_event(
            &mut runtime,
            window(),
            keyboard(
                physical,
                LogicalKey::Named("Shift".to_owned()),
                DigitalState::Pressed,
                false,
                ObservationOrigin::SourceReport,
            ),
        );
    }
    let _ = translate_platform_event(
        &mut runtime,
        window(),
        keyboard(
            "ShiftLeft",
            LogicalKey::Named("Shift".to_owned()),
            DigitalState::Released,
            false,
            ObservationOrigin::SourceReport,
        ),
    );
    let event = one(
        &mut runtime,
        keyboard(
            "KeyC",
            LogicalKey::Character("c".to_owned()),
            DigitalState::Pressed,
            false,
            ObservationOrigin::SourceReport,
        ),
    );
    assert!(matches!(
        event,
        UiInputEvent::Keyboard(KeyboardEvent { modifiers, .. }) if modifiers.shift
    ));
}

#[test]
fn modifier_lifetimes_are_aggregated_without_aliasing_devices() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let first = scoped_context(7, Some(9));
    let second = scoped_context(7, Some(10));
    for keyboard_context in [first, second] {
        let _ = translate_platform_event(
            &mut runtime,
            window(),
            keyboard_in(
                keyboard_context,
                "ShiftLeft",
                LogicalKey::Named("Shift".to_owned()),
                DigitalState::Pressed,
                false,
                ObservationOrigin::SourceReport,
            ),
        );
    }
    let _ = translate_platform_event(
        &mut runtime,
        window(),
        keyboard_in(
            first,
            "ShiftLeft",
            LogicalKey::Named("Shift".to_owned()),
            DigitalState::Released,
            false,
            ObservationOrigin::SourceReport,
        ),
    );
    let still_shifted = one(
        &mut runtime,
        keyboard_in(
            first,
            "KeyA",
            LogicalKey::Character("a".to_owned()),
            DigitalState::Pressed,
            false,
            ObservationOrigin::SourceReport,
        ),
    );
    assert!(matches!(
        still_shifted,
        UiInputEvent::Keyboard(KeyboardEvent { modifiers, .. }) if modifiers.shift
    ));

    let _ = translate_platform_event(
        &mut runtime,
        window(),
        keyboard_in(
            second,
            "ShiftLeft",
            LogicalKey::Named("Shift".to_owned()),
            DigitalState::Released,
            false,
            ObservationOrigin::SourceReport,
        ),
    );
    let unshifted = one(
        &mut runtime,
        keyboard_in(
            first,
            "KeyB",
            LogicalKey::Character("b".to_owned()),
            DigitalState::Pressed,
            false,
            ObservationOrigin::SourceReport,
        ),
    );
    assert!(matches!(
        unshifted,
        UiInputEvent::Keyboard(KeyboardEvent { modifiers, .. }) if !modifiers.shift
    ));
}

#[test]
fn text_is_a_sibling_path() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    assert!(matches!(
        one(
            &mut runtime,
            keyboard(
                "KeyA",
                LogicalKey::Character("a".to_owned()),
                DigitalState::Pressed,
                false,
                ObservationOrigin::SourceReport,
            )
        ),
        UiInputEvent::Keyboard(_)
    ));
    assert_eq!(
        one(
            &mut runtime,
            PlatformEvent::TextInput {
                text: "ä".to_owned(),
            }
        ),
        UiInputEvent::Text(TextInputEvent {
            text: "ä".to_owned()
        })
    );
}

#[test]
fn mouse_position_is_scoped_to_the_normalized_source_stream() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let first = scoped_context(7, Some(1));
    let second = scoped_context(8, Some(2));
    let _ = one(
        &mut runtime,
        PlatformEvent::CursorMoved {
            context: first,
            position: Point2::new(10.0, 20.0, CoordinateSpace::WindowPhysicalPixels),
        },
    );
    let _ = one(
        &mut runtime,
        PlatformEvent::CursorMoved {
            context: second,
            position: Point2::new(100.0, 200.0, CoordinateSpace::WindowPhysicalPixels),
        },
    );
    let event = one(
        &mut runtime,
        PlatformEvent::MouseInput {
            context: first,
            input: PointerButtonInput {
                button: EnginePointerButton::Left,
                state: DigitalState::Pressed,
            },
        },
    );
    assert!(matches!(
        event,
        UiInputEvent::Pointer(PointerEvent { position, .. })
            if position == UiPoint::new(10.0, 20.0)
    ));
}

#[test]
fn scroll_preserves_both_axes_and_converts_pixel_domain_explicitly() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let event = one(
        &mut runtime,
        PlatformEvent::MouseWheel {
            context: context(None),
            input: ScrollInput {
                delta: ScrollDelta {
                    horizontal: Some(56.0),
                    vertical: Some(-28.0),
                },
                domain: ScrollDomain::WindowPhysicalPixels,
                phase: None,
            },
        },
    );
    assert!(matches!(
        event,
        UiInputEvent::Pointer(PointerEvent { delta, .. })
            if delta == UiVector::new(2.0, -1.0)
    ));
}

#[test]
fn touch_contacts_keep_independent_lifetimes_and_cancel_is_not_semantic_cancel() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let touch = |id, phase, x, y| PlatformEvent::Touch {
        context: context(Some(3)),
        input: ContactInput {
            contact: ContactId::new(id),
            phase,
            position: Point2::new(x, y, CoordinateSpace::WindowPhysicalPixels),
            pressure: None,
            altitude_angle_radians: None,
        },
    };
    let _ = one(&mut runtime, touch(1, ContactPhase::Begin, 10.0, 10.0));
    let _ = one(&mut runtime, touch(2, ContactPhase::Begin, 100.0, 100.0));
    let first_update = one(&mut runtime, touch(1, ContactPhase::Update, 13.0, 15.0));
    assert!(matches!(
        first_update,
        UiInputEvent::Pointer(PointerEvent { delta, packet, .. })
            if delta == UiVector::new(3.0, 5.0)
                && packet.contact_id == Some(PointerContactId(1))
                && packet.contact_phase == Some(PointerContactPhase::Update)
    ));
    let cancelled = one(&mut runtime, touch(2, ContactPhase::Cancel, 101.0, 102.0));
    assert!(matches!(
        cancelled,
        UiInputEvent::Pointer(PointerEvent {
            kind: PointerEventKind::Leave,
            packet,
            ..
        }) if packet.contact_id == Some(PointerContactId(2))
            && packet.contact_phase == Some(PointerContactPhase::Cancel)
    ));
}

#[test]
fn equal_touch_ids_on_distinct_devices_do_not_alias() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let touch = |touch_context, phase, x, y| PlatformEvent::Touch {
        context: touch_context,
        input: ContactInput {
            contact: ContactId::new(1),
            phase,
            position: Point2::new(x, y, CoordinateSpace::WindowPhysicalPixels),
            pressure: None,
            altitude_angle_radians: None,
        },
    };
    let first = scoped_context(7, Some(3));
    let second = scoped_context(7, Some(4));
    let first_begin = one(&mut runtime, touch(first, ContactPhase::Begin, 10.0, 10.0));
    let second_begin = one(
        &mut runtime,
        touch(second, ContactPhase::Begin, 100.0, 100.0),
    );
    let first_update = one(&mut runtime, touch(first, ContactPhase::Update, 13.0, 15.0));

    let first_device = match first_begin {
        UiInputEvent::Pointer(event) => event.packet.device_id,
        _ => None,
    };
    let second_device = match second_begin {
        UiInputEvent::Pointer(event) => event.packet.device_id,
        _ => None,
    };
    assert_ne!(first_device, second_device);
    assert!(matches!(
        first_update,
        UiInputEvent::Pointer(PointerEvent { delta, packet, .. })
            if delta == UiVector::new(3.0, 5.0)
                && packet.contact_id == Some(PointerContactId(1))
                && packet.device_id == first_device
    ));
}

#[test]
fn non_finite_observations_are_rejected_before_ui_state_changes() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let pointer_context = context(Some(3));
    let _ = one(
        &mut runtime,
        PlatformEvent::CursorMoved {
            context: pointer_context,
            position: Point2::new(10.0, 20.0, CoordinateSpace::WindowPhysicalPixels),
        },
    );
    assert!(
        translate_platform_event(
            &mut runtime,
            window(),
            PlatformEvent::CursorMoved {
                context: pointer_context,
                position: Point2::new(f32::INFINITY, 30.0, CoordinateSpace::WindowPhysicalPixels,),
            },
        )
        .is_empty()
    );
    let button = one(
        &mut runtime,
        PlatformEvent::MouseInput {
            context: pointer_context,
            input: PointerButtonInput {
                button: EnginePointerButton::Left,
                state: DigitalState::Pressed,
            },
        },
    );
    assert!(matches!(
        button,
        UiInputEvent::Pointer(PointerEvent { position, .. })
            if position == UiPoint::new(10.0, 20.0)
    ));

    let touch = |phase, x, pressure| PlatformEvent::Touch {
        context: pointer_context,
        input: ContactInput {
            contact: ContactId::new(9),
            phase,
            position: Point2::new(x, 1.0, CoordinateSpace::WindowPhysicalPixels),
            pressure,
            altitude_angle_radians: None,
        },
    };
    let _ = one(&mut runtime, touch(ContactPhase::Begin, 1.0, None));
    assert!(
        translate_platform_event(
            &mut runtime,
            window(),
            touch(
                ContactPhase::Update,
                2.0,
                Some(AnalogMeasurement {
                    value: f32::NAN,
                    domain: MeasurementDomain::NormalizedUnitInterval,
                }),
            ),
        )
        .is_empty()
    );
    let update = one(&mut runtime, touch(ContactPhase::Update, 3.0, None));
    assert!(matches!(
        update,
        UiInputEvent::Pointer(PointerEvent { delta, .. }) if delta == UiVector::new(2.0, 0.0)
    ));
}

#[test]
fn only_normalized_pressure_projects_into_ui_pressure() {
    let mut runtime = EditorTargetInputRuntimeResource::default();
    let touch = |pressure| PlatformEvent::Touch {
        context: context(None),
        input: ContactInput {
            contact: ContactId::new(1),
            phase: ContactPhase::Begin,
            position: Point2::new(1.0, 2.0, CoordinateSpace::WindowPhysicalPixels),
            pressure,
            altitude_angle_radians: None,
        },
    };
    let normalized = one(
        &mut runtime,
        touch(Some(AnalogMeasurement {
            value: 0.5,
            domain: MeasurementDomain::NormalizedUnitInterval,
        })),
    );
    assert!(matches!(
        normalized,
        UiInputEvent::Pointer(PointerEvent { packet, .. }) if packet.pressure == Some(0.5)
    ));
    let calibrated = one(
        &mut runtime,
        touch(Some(AnalogMeasurement {
            value: 2.0,
            domain: MeasurementDomain::CalibratedForce {
                max_possible_force: 4.0,
            },
        })),
    );
    assert!(matches!(
        calibrated,
        UiInputEvent::Pointer(PointerEvent { packet, .. }) if packet.pressure.is_none()
    ));
    let out_of_range_normalized = one(
        &mut runtime,
        touch(Some(AnalogMeasurement {
            value: 1.25,
            domain: MeasurementDomain::NormalizedUnitInterval,
        })),
    );
    assert!(matches!(
        out_of_range_normalized,
        UiInputEvent::Pointer(PointerEvent { packet, .. }) if packet.pressure.is_none()
    ));
}
