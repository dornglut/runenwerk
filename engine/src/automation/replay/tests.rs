use super::*;
use crate::plugins::InputFinalizePlugin;
use crate::runtime::ResMut;
use runen_input::{
    ContactId, ContactPhase, ContactPresence, CoordinateSpace, DeliveryRole, DigitalState,
    EvidenceStatus, InputDeviceId, InputToolKind, ObservationOrigin, PhysicalTabletControls,
    Point2, PointerButtonInput, RelativeMotionUnit, ScrollDelta, ScrollDomain, ScrollInput,
    SourceTime, SourceTimeUnit, TabletCapabilities, TabletObservation, Vector2,
};

use crate::automation::{AutomationInputTraceFrame, AutomationInputTracePlugin};

fn one_frame_trace(groups: Vec<InputObservationGroup>) -> AutomationInputTrace {
    AutomationInputTrace {
        frames: vec![AutomationInputTraceFrame {
            frame_ordinal: 0,
            groups,
        }],
        trailing_groups: Vec::new(),
    }
}

fn replay_app() -> App {
    let mut app = App::headless();
    app.add_plugin(InputFinalizePlugin);
    app
}

fn trace_app() -> App {
    let mut app = replay_app();
    app.add_plugin(AutomationInputTracePlugin);
    app
}

fn source_map(entries: impl IntoIterator<Item = (u64, u64)>) -> AutomationInputReplaySourceMap {
    AutomationInputReplaySourceMap::new(
        entries
            .into_iter()
            .map(|(recorded, replay)| (InputSourceId::new(recorded), InputSourceId::new(replay))),
    )
}

#[test]
fn replay_preflight_rejects_invalid_mapping_trailing_and_unsupported_shapes_without_mutation() {
    let pointer = InputObservationGroup::single(
        InputContext::new(InputSourceId::new(2_001), None),
        InputObservation::PointerButton(PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Pressed,
        }),
    );
    let mut app = replay_app();

    let missing = app.replay_automation_input_trace(
        &one_frame_trace(vec![pointer.clone()]),
        &source_map([]),
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        missing.outcome(),
        AutomationInputReplayOutcome::InvalidSourceMapping
    );
    assert!(
        !app.world()
            .resource::<InputState>()
            .unwrap()
            .left_mouse_down()
    );

    let second = InputObservationGroup::single(
        InputContext::new(InputSourceId::new(2_002), None),
        InputObservation::Scroll(ScrollInput {
            delta: ScrollDelta::vertical_only(1.0),
            domain: ScrollDomain::Unspecified,
            phase: None,
        }),
    );
    let non_injective = app.replay_automation_input_trace(
        &one_frame_trace(vec![pointer.clone(), second]),
        &source_map([(2_001, 9_001), (2_002, 9_001)]),
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        non_injective.outcome(),
        AutomationInputReplayOutcome::InvalidSourceMapping
    );

    let trailing = AutomationInputTrace {
        frames: Vec::new(),
        trailing_groups: vec![pointer.clone()],
    };
    let trailing_report = app.replay_automation_input_trace(
        &trailing,
        &source_map([(2_001, 9_001)]),
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        trailing_report.outcome(),
        AutomationInputReplayOutcome::UnframedTrailingGroups
    );

    let unsupported = one_frame_trace(vec![InputObservationGroup::single(
        InputContext::new(InputSourceId::new(2_001), None),
        InputObservation::AbsolutePointerPosition {
            position: Point2::new(1.0, 2.0, CoordinateSpace::WindowPhysicalPixels),
        },
    )]);
    let unsupported_report = app.replay_automation_input_trace(
        &unsupported,
        &source_map([(2_001, 9_001)]),
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        unsupported_report.outcome(),
        AutomationInputReplayOutcome::UnsupportedTraceShape
    );
    assert_eq!(unsupported_report.completed_frames(), 0);
}

#[test]
fn replay_preflight_rejects_capture_trace_dirty_projection_and_used_pointer_conflicts() {
    let trace = one_frame_trace(vec![InputObservationGroup::single(
        InputContext::new(InputSourceId::new(2_010), None),
        InputObservation::PointerButton(PointerButtonInput {
            button: PointerButton::Back,
            state: DigitalState::Pressed,
        }),
    )]);
    let mapping = source_map([(2_010, 9_010)]);

    let mut capture = replay_app();
    capture
        .world_mut()
        .resource_mut::<InputState>()
        .unwrap()
        .start_admitted_input_capture();
    assert_eq!(
        capture
            .replay_automation_input_trace(
                &trace,
                &mapping,
                AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
            )
            .outcome(),
        AutomationInputReplayOutcome::TargetStateConflict
    );
    assert!(
        capture
            .world()
            .resource::<InputState>()
            .unwrap()
            .admitted_input_capture_active()
    );

    let mut tracing = trace_app();
    tracing.start_automation_input_trace().unwrap();
    assert_eq!(
        tracing
            .replay_automation_input_trace(
                &trace,
                &mapping,
                AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
            )
            .outcome(),
        AutomationInputReplayOutcome::TargetStateConflict
    );
    assert!(tracing.automation_input_trace_active());

    let mut dirty = replay_app();
    dirty
        .world_mut()
        .resource_mut::<InputState>()
        .unwrap()
        .handle_mouse_motion(3.0, 4.0);
    assert_eq!(
        dirty
            .replay_automation_input_trace(
                &trace,
                &mapping,
                AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
            )
            .outcome(),
        AutomationInputReplayOutcome::TargetStateConflict
    );
    assert_eq!(
        dirty.world().resource::<InputState>().unwrap().mouse_delta,
        (3.0, 4.0),
        "preflight must not clear unrelated frame-local evidence"
    );

    let mut held = replay_app();
    held.world_mut()
        .resource_mut::<InputState>()
        .unwrap()
        .handle_pointer_button(
            InputContext::new(InputSourceId::new(88), None),
            PointerButtonInput {
                button: PointerButton::Back,
                state: DigitalState::Pressed,
            },
        );
    held.world_mut()
        .resource_mut::<InputState>()
        .unwrap()
        .clear_frame();
    assert_eq!(
        held.replay_automation_input_trace(
            &trace,
            &mapping,
            AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
        )
        .outcome(),
        AutomationInputReplayOutcome::TargetStateConflict
    );
}

#[test]
fn replay_rejects_mapped_source_with_retained_absolute_pointer_state() {
    let recorded_source = InputSourceId::new(2_014);
    let replay_source = InputSourceId::new(9_014);
    let trace = one_frame_trace(vec![InputObservationGroup::single(
        InputContext::new(recorded_source, None),
        InputObservation::PointerButton(PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Pressed,
        }),
    )]);
    let mapping = AutomationInputReplaySourceMap::new([(recorded_source, replay_source)]);
    let mut app = replay_app();

    {
        let input = app.world_mut().resource_mut::<InputState>().unwrap();
        input.handle_cursor_position(
            InputContext::new(replay_source, None),
            Point2::new(42.0, 24.0, CoordinateSpace::WindowPhysicalPixels),
        );
        input.clear_frame();
        assert_eq!(
            input.absolute_pointer_position_for_source(replay_source),
            Some(Point2::new(
                42.0,
                24.0,
                CoordinateSpace::WindowPhysicalPixels,
            ))
        );
    }

    let report = app.replay_automation_input_trace(
        &trace,
        &mapping,
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        report.outcome(),
        AutomationInputReplayOutcome::TargetStateConflict
    );
    assert_eq!(report.completed_frames(), 0);
    assert_eq!(
        app.world()
            .resource::<InputState>()
            .unwrap()
            .absolute_pointer_position_for_source(replay_source),
        Some(Point2::new(
            42.0,
            24.0,
            CoordinateSpace::WindowPhysicalPixels,
        )),
        "preflight must not clear state proving that the mapped source is not pristine"
    );
}

#[test]
fn replay_requires_headless_host_and_teardown_cleans_extended_pointer_buttons() {
    let trace = one_frame_trace(vec![InputObservationGroup::single(
        InputContext::new(InputSourceId::new(2_015), None),
        InputObservation::PointerButton(PointerButtonInput {
            button: PointerButton::Back,
            state: DigitalState::Pressed,
        }),
    )]);
    let mapping = source_map([(2_015, 9_015)]);

    let mut native = App::new();
    native.add_plugin(InputFinalizePlugin);
    let native_report = native.replay_automation_input_trace(
        &trace,
        &mapping,
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        native_report.outcome(),
        AutomationInputReplayOutcome::TargetStateConflict
    );
    assert_eq!(native_report.completed_frames(), 0);

    let mut headless = replay_app();
    let headless_report = headless.replay_automation_input_trace(
        &trace,
        &mapping,
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        headless_report.outcome(),
        AutomationInputReplayOutcome::Completed
    );
    assert!(
        headless
            .world()
            .resource::<InputState>()
            .unwrap()
            .pointer_button_down_anywhere(PointerButton::Back)
    );

    let overlapping = headless.replay_automation_input_trace(
        &trace,
        &mapping,
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        overlapping.outcome(),
        AutomationInputReplayOutcome::TargetStateConflict,
        "one successful replay owns its mapped sources until explicit teardown"
    );

    headless
        .world_mut()
        .resource_mut::<InputState>()
        .unwrap()
        .start_admitted_input_capture();
    assert!(
        matches!(
            headless.teardown_automation_input_replay(),
            Err(AutomationInputReplayTeardownError::EvidenceCaptureActive)
        ),
        "teardown must not inject cleanup continuity into another capture owner"
    );
    assert!(
        headless
            .world()
            .resource::<InputState>()
            .unwrap()
            .pointer_button_down_anywhere(PointerButton::Back),
        "blocked teardown must retain replay ownership for a later retry"
    );
    let captured = headless
        .world_mut()
        .resource_mut::<InputState>()
        .unwrap()
        .stop_admitted_input_capture();
    assert!(captured.is_empty());

    headless
        .teardown_automation_input_replay()
        .expect("teardown should clean the replay-owned Back button source");
    assert!(
        !headless
            .world()
            .resource::<InputState>()
            .unwrap()
            .pointer_button_down_anywhere(PointerButton::Back)
    );
    assert!(
        matches!(
            headless.teardown_automation_input_replay(),
            Err(AutomationInputReplayTeardownError::ReplayNotActive)
        ),
        "teardown cannot be redirected after replay ownership is released"
    );
}

#[derive(Debug, Default, runen_ecs::Resource)]
struct ReplayFrameCounter(u64);

fn count_replay_frame(mut counter: ResMut<ReplayFrameCounter>) {
    counter.0 += 1;
}

#[test]
fn replay_preserves_group_order_idle_frames_and_unrelated_held_state() {
    let recorded_source = InputSourceId::new(2_020);
    let context = InputContext::new(recorded_source, None);
    let trace = AutomationInputTrace {
        frames: vec![
            AutomationInputTraceFrame {
                frame_ordinal: 0,
                groups: vec![
                    InputObservationGroup::single(
                        context,
                        InputObservation::PointerButton(PointerButtonInput {
                            button: PointerButton::Left,
                            state: DigitalState::Pressed,
                        }),
                    ),
                    InputObservationGroup::single(
                        context,
                        InputObservation::RelativeMotion {
                            delta: Vector2::new(5.0, -2.0),
                            unit: RelativeMotionUnit::BackendDeviceUnits,
                        },
                    ),
                ],
            },
            AutomationInputTraceFrame {
                frame_ordinal: 1,
                groups: Vec::new(),
            },
            AutomationInputTraceFrame {
                frame_ordinal: 2,
                groups: vec![InputObservationGroup::single(
                    context,
                    InputObservation::Scroll(ScrollInput {
                        delta: ScrollDelta::vertical_only(1.0),
                        domain: ScrollDomain::Unspecified,
                        phase: None,
                    }),
                )],
            },
        ],
        trailing_groups: Vec::new(),
    };
    let mapping = source_map([(2_020, 9_020)]);
    let mut app = replay_app();
    app.init_resource::<ReplayFrameCounter>();
    app.add_systems(crate::runtime::Update, count_replay_frame);
    {
        let input = app.world_mut().resource_mut::<InputState>().unwrap();
        input.handle_pointer_button(
            InputContext::new(InputSourceId::new(99), None),
            PointerButtonInput {
                button: PointerButton::Right,
                state: DigitalState::Pressed,
            },
        );
        input.clear_frame();
    }

    let report = app.replay_automation_input_trace(
        &trace,
        &mapping,
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(report.outcome(), AutomationInputReplayOutcome::Completed);
    assert_eq!(report.completed_frames(), 3);
    assert_eq!(app.world().resource::<ReplayFrameCounter>().unwrap().0, 3);
    let input = app.world().resource::<InputState>().unwrap();
    assert!(input.left_mouse_down());
    assert!(
        input.right_mouse_down(),
        "unrelated held source must survive replay"
    );

    app.teardown_automation_input_replay()
        .expect("replay teardown should succeed");
    let input = app.world().resource::<InputState>().unwrap();
    assert!(!input.left_mouse_down());
    assert!(input.right_mouse_down());
}

#[test]
fn replay_remaps_atomic_tablet_group_and_source_time_context() {
    let recorded_context =
        InputContext::new(InputSourceId::new(2_030), Some(InputDeviceId::new(77)));
    let source_time = SourceTime::new(recorded_context, 123_456, SourceTimeUnit::Microseconds);
    let tablet = |contact, delivery| {
        InputObservation::Tablet(TabletObservation {
            contact: ContactId::new(contact),
            tool: None,
            tool_kind: InputToolKind::Pen,
            phase: ContactPhase::Update,
            presence: ContactPresence::Contact,
            position: Point2::new(contact as f32, 4.0, CoordinateSpace::WindowPhysicalPixels),
            delta: Vector2::new(1.0, 0.0),
            pressure: None,
            tangential_pressure: None,
            tilt: None,
            twist: None,
            controls: PhysicalTabletControls::default(),
            capabilities: TabletCapabilities::default(),
            source_time: Some(source_time),
            evidence: EvidenceStatus::ObservedConfirmed,
            delivery,
            origin: ObservationOrigin::SourceReport,
        })
    };
    let group = InputObservationGroup::new(
        recorded_context,
        vec![
            tablet(1, DeliveryRole::HistoricalCoalesced),
            tablet(2, DeliveryRole::OrdinaryCurrent),
        ],
    );
    let mut input = InputState::new();

    admit_replay_group(&mut input, &group, InputSourceId::new(9_030))
        .expect("tablet replay ingress should admit");
    let staged = input.drain_device_observation_groups();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].observations.len(), 2);
    assert_eq!(staged[0].context.source, InputSourceId::new(9_030));
    assert_eq!(staged[0].context.device, recorded_context.device);
    for observation in &staged[0].observations {
        let InputObservation::Tablet(tablet) = observation else {
            panic!("staged group should remain all-tablet");
        };
        let remapped_time = tablet
            .source_time
            .expect("source time should remain present");
        assert_eq!(remapped_time.context, staged[0].context);
        assert_eq!(remapped_time.value, source_time.value);
        assert_eq!(remapped_time.unit, source_time.unit);
    }
}

#[test]
fn replay_reports_rejected_input_with_partial_progress_and_cleans_replay_sources() {
    let source = InputSourceId::new(2_040);
    let context = InputContext::new(source, None);
    let trace = AutomationInputTrace {
        frames: vec![
            AutomationInputTraceFrame {
                frame_ordinal: 0,
                groups: vec![InputObservationGroup::single(
                    context,
                    InputObservation::PointerButton(PointerButtonInput {
                        button: PointerButton::Left,
                        state: DigitalState::Pressed,
                    }),
                )],
            },
            AutomationInputTraceFrame {
                frame_ordinal: 1,
                groups: vec![
                    InputObservationGroup::single(
                        context,
                        InputObservation::RelativeMotion {
                            delta: Vector2::new(2.0, 1.0),
                            unit: RelativeMotionUnit::BackendDeviceUnits,
                        },
                    ),
                    InputObservationGroup::single(
                        context,
                        InputObservation::RelativeMotion {
                            delta: Vector2::new(f32::NAN, 0.0),
                            unit: RelativeMotionUnit::BackendDeviceUnits,
                        },
                    ),
                ],
            },
        ],
        trailing_groups: Vec::new(),
    };
    let mapping = source_map([(2_040, 9_040)]);
    let mut app = replay_app();

    let report = app.replay_automation_input_trace(
        &trace,
        &mapping,
        AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
    );
    assert_eq!(
        report.outcome(),
        AutomationInputReplayOutcome::InvalidOrRejectedInput
    );
    assert_eq!(report.completed_frames(), 1);
    assert_eq!(report.failing_frame_ordinal(), Some(1));
    assert_eq!(report.failing_group_index(), Some(1));
    let input = app.world().resource::<InputState>().unwrap();
    assert!(!input.left_mouse_down());
    assert!(
        input.frame_projection_is_quiescent(),
        "failed admission must not leave replay-owned partial-frame projection behind"
    );
}
