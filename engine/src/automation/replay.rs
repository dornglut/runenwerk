//! Normalized in-memory application input-trace replay.
//!
//! This module owns only the first bounded headless replay contract accepted by the application
//! automation semantic model. Scene replay, RunenUI replay, native input injection, persistence,
//! and authored scenarios remain separate owners.

use std::collections::HashSet;
use std::fmt;

use crate::app::App;
use crate::plugins::input::input_integration_is_active;
use crate::plugins::InputState;
use runen_input::{
    ContinuityLoss, InputContext, InputObservation, InputObservationGroup, InputSourceId,
    PointerButton,
};

use super::{AppAutomationInputTraceExt, AutomationInputTrace};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutomationInputReplaySourceMap {
    entries: Vec<(InputSourceId, InputSourceId)>,
}

impl AutomationInputReplaySourceMap {
    pub fn new(entries: impl IntoIterator<Item = (InputSourceId, InputSourceId)>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    fn mapped_source(&self, recorded: InputSourceId) -> Option<InputSourceId> {
        let mut matches = self
            .entries
            .iter()
            .filter_map(|(candidate, replay)| (*candidate == recorded).then_some(*replay));
        let mapped = matches.next()?;
        matches.next().is_none().then_some(mapped)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationInputReplayInitialState {
    RecordedSourcesNeutralAtCaptureStart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationInputReplayOutcome {
    Completed,
    UnsupportedTraceShape,
    InvalidSourceMapping,
    InvalidOrRejectedInput,
    UnframedTrailingGroups,
    TargetStateConflict,
    InfrastructureFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationInputReplayReport {
    outcome: AutomationInputReplayOutcome,
    completed_frames: u64,
    failing_frame_ordinal: Option<u64>,
    failing_group_index: Option<usize>,
    detail: Option<String>,
}

impl AutomationInputReplayReport {
    pub fn outcome(&self) -> AutomationInputReplayOutcome {
        self.outcome
    }

    pub fn completed_frames(&self) -> u64 {
        self.completed_frames
    }

    pub fn failing_frame_ordinal(&self) -> Option<u64> {
        self.failing_frame_ordinal
    }

    pub fn failing_group_index(&self) -> Option<usize> {
        self.failing_group_index
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    fn completed(frame_count: usize) -> Self {
        Self {
            outcome: AutomationInputReplayOutcome::Completed,
            completed_frames: frame_count as u64,
            failing_frame_ordinal: None,
            failing_group_index: None,
            detail: None,
        }
    }

    fn rejected(outcome: AutomationInputReplayOutcome, detail: impl Into<String>) -> Self {
        Self {
            outcome,
            completed_frames: 0,
            failing_frame_ordinal: None,
            failing_group_index: None,
            detail: Some(detail.into()),
        }
    }

    fn failed_after_start(
        outcome: AutomationInputReplayOutcome,
        completed_frames: u64,
        frame_ordinal: u64,
        group_index: Option<usize>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            outcome,
            completed_frames,
            failing_frame_ordinal: Some(frame_ordinal),
            failing_group_index: group_index,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationInputReplayTeardownError {
    InvalidSourceMapping,
    InputIntegrationUnavailable,
}

impl fmt::Display for AutomationInputReplayTeardownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSourceMapping => "automation input replay source mapping is invalid",
            Self::InputIntegrationUnavailable => "Runenwerk input integration is unavailable",
        })
    }
}

impl std::error::Error for AutomationInputReplayTeardownError {}

pub trait AppAutomationInputReplayExt {
    fn replay_automation_input_trace(
        &mut self,
        trace: &AutomationInputTrace,
        source_map: &AutomationInputReplaySourceMap,
        initial_state: AutomationInputReplayInitialState,
    ) -> AutomationInputReplayReport;

    fn teardown_automation_input_replay(
        &mut self,
        trace: &AutomationInputTrace,
        source_map: &AutomationInputReplaySourceMap,
    ) -> Result<&mut Self, AutomationInputReplayTeardownError>;
}

#[derive(Debug)]
struct AutomationInputReplayPlan {
    replay_sources: Vec<InputSourceId>,
    pointer_buttons: Vec<PointerButton>,
}

fn input_replay_preflight(
    trace: &AutomationInputTrace,
    source_map: &AutomationInputReplaySourceMap,
    _initial_state: AutomationInputReplayInitialState,
) -> Result<AutomationInputReplayPlan, AutomationInputReplayReport> {
    if !trace.trailing_groups.is_empty() {
        return Err(AutomationInputReplayReport::rejected(
            AutomationInputReplayOutcome::UnframedTrailingGroups,
            "normalized replay rejects input groups without established App-frame membership",
        ));
    }

    let mut recorded_sources = Vec::new();
    let mut pointer_buttons = Vec::new();
    for (frame_index, frame) in trace.frames.iter().enumerate() {
        if frame.frame_ordinal != frame_index as u64 {
            return Err(AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::UnsupportedTraceShape,
                format!(
                    "trace frame ordinal {} is not the expected contiguous ordinal {}",
                    frame.frame_ordinal, frame_index
                ),
            ));
        }
        for group in &frame.groups {
            if !recorded_sources.contains(&group.context.source) {
                recorded_sources.push(group.context.source);
            }
            if group.observations.is_empty() {
                return Err(AutomationInputReplayReport::rejected(
                    AutomationInputReplayOutcome::UnsupportedTraceShape,
                    "empty input observation groups are not replayable",
                ));
            }

            let all_tablet = group
                .observations
                .iter()
                .all(|observation| matches!(observation, InputObservation::Tablet(_)));
            if all_tablet {
                continue;
            }

            if group.observations.len() != 1 {
                return Err(AutomationInputReplayReport::rejected(
                    AutomationInputReplayOutcome::UnsupportedTraceShape,
                    "multi-observation non-tablet groups are not supported",
                ));
            }

            match &group.observations[0] {
                InputObservation::PointerButton(input) => {
                    if !pointer_buttons.contains(&input.button) {
                        pointer_buttons.push(input.button);
                    }
                }
                InputObservation::RelativeMotion { .. } | InputObservation::Scroll(_) => {}
                InputObservation::Keyboard(_)
                | InputObservation::AbsolutePointerPosition { .. }
                | InputObservation::Contact(_)
                | InputObservation::ContinuityLoss(_)
                | InputObservation::Tablet(_) => {
                    return Err(AutomationInputReplayReport::rejected(
                        AutomationInputReplayOutcome::UnsupportedTraceShape,
                        "trace contains an input family that lacks a self-contained first-slice replay contract",
                    ));
                }
            }
        }
    }

    let mut replay_sources = Vec::with_capacity(recorded_sources.len());
    let mut unique_replay_sources = HashSet::with_capacity(recorded_sources.len());
    for recorded in recorded_sources {
        let Some(replay) = source_map.mapped_source(recorded) else {
            return Err(AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::InvalidSourceMapping,
                format!("recorded input source {} has no unique replay mapping", recorded.raw()),
            ));
        };
        if !unique_replay_sources.insert(replay) {
            return Err(AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::InvalidSourceMapping,
                "distinct recorded input sources map to the same replay source",
            ));
        }
        replay_sources.push(replay);
    }

    Ok(AutomationInputReplayPlan {
        replay_sources,
        pointer_buttons,
    })
}

fn input_replay_target_state_conflict(
    app: &App,
    pointer_buttons: &[PointerButton],
) -> Option<String> {
    if app.automation_input_trace_active() {
        return Some("automation input trace recording is already active".to_owned());
    }

    let input = match app.world().resource::<InputState>() {
        Ok(input) => input,
        Err(_) => return Some("Runenwerk input integration is unavailable".to_owned()),
    };
    if input.admitted_input_capture_active() {
        return Some("admitted-input capture is already active".to_owned());
    }
    if !input.frame_projection_is_quiescent() {
        return Some("target App has non-quiescent frame-local input projection".to_owned());
    }
    if pointer_buttons
        .iter()
        .copied()
        .any(|button| input.pointer_button_down_anywhere(button))
    {
        return Some("a pointer button used by the trace is already held in target input state".to_owned());
    }
    None
}

fn remap_input_group(
    group: &InputObservationGroup,
    replay_source: InputSourceId,
) -> InputObservationGroup {
    let remapped_context = InputContext::new(replay_source, group.context.device);
    let mut observations = group.observations.clone();
    for observation in &mut observations {
        if let InputObservation::Tablet(tablet) = observation
            && let Some(source_time) = &mut tablet.source_time
        {
            source_time.context = remapped_context;
        }
    }
    InputObservationGroup::new(remapped_context, observations)
}

fn admit_replay_group(
    input: &mut InputState,
    group: &InputObservationGroup,
    replay_source: InputSourceId,
) -> Result<(), String> {
    let remapped = remap_input_group(group, replay_source);
    if remapped
        .observations
        .iter()
        .all(|observation| matches!(observation, InputObservation::Tablet(_)))
    {
        return input
            .admit_device_observation_group(remapped)
            .map_err(|error| error.to_string());
    }

    let observation = remapped
        .observations
        .into_iter()
        .next()
        .ok_or_else(|| "replay group unexpectedly contained no observations".to_owned())?;
    match input.admit_automation_observation(remapped.context, observation) {
        Ok(true) => Ok(()),
        Ok(false) => Err("replay group became unsupported after preflight".to_owned()),
        Err(error) => Err(error.to_string()),
    }
}

fn cleanup_replay_sources(input: &mut InputState, replay_sources: &[InputSourceId]) {
    for source in replay_sources {
        input.handle_continuity_loss(
            InputContext::new(*source, None),
            ContinuityLoss::Source,
        );
    }
}

fn cleanup_failed_admission_frame(
    input: &mut InputState,
    replay_sources: &[InputSourceId],
) {
    cleanup_replay_sources(input, replay_sources);
    // Preflight requires a quiescent headless target, and no App frame runs while the groups for
    // one recorded frame are being admitted. Therefore any frame-local projection present on this
    // admission-failure path was created by replay itself and can be discarded without erasing
    // unrelated caller evidence.
    input.clear_frame();
}

impl AppAutomationInputReplayExt for App {
    fn replay_automation_input_trace(
        &mut self,
        trace: &AutomationInputTrace,
        source_map: &AutomationInputReplaySourceMap,
        initial_state: AutomationInputReplayInitialState,
    ) -> AutomationInputReplayReport {
        if let Err(error) = self.require_headless_host("normalized input trace replay") {
            return AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::TargetStateConflict,
                error.to_string(),
            );
        }
        if !input_integration_is_active(self.world()) {
            return AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::TargetStateConflict,
                "Runenwerk input integration is unavailable",
            );
        }

        let plan = match input_replay_preflight(trace, source_map, initial_state) {
            Ok(plan) => plan,
            Err(report) => return report,
        };
        if let Some(detail) = input_replay_target_state_conflict(self, &plan.pointer_buttons) {
            return AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::TargetStateConflict,
                detail,
            );
        }

        if let Err(error) = self.run_headless_for_frames(0) {
            return AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::InfrastructureFailure,
                format!("failed to prepare headless replay App: {error:#}"),
            );
        }
        if let Some(detail) = input_replay_target_state_conflict(self, &plan.pointer_buttons) {
            return AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::TargetStateConflict,
                detail,
            );
        }

        let mut completed_frames = 0u64;
        for frame in &trace.frames {
            for (group_index, group) in frame.groups.iter().enumerate() {
                let Some(replay_source) = source_map.mapped_source(group.context.source) else {
                    let input = self
                        .world_mut()
                        .resource_mut::<InputState>()
                        .expect("input integration was checked before replay");
                    cleanup_replay_sources(input, &plan.replay_sources);
                    return AutomationInputReplayReport::failed_after_start(
                        AutomationInputReplayOutcome::InvalidSourceMapping,
                        completed_frames,
                        frame.frame_ordinal,
                        Some(group_index),
                        "replay source mapping changed after preflight",
                    );
                };

                let admission_result = {
                    let input = self
                        .world_mut()
                        .resource_mut::<InputState>()
                        .expect("input integration was checked before replay");
                    admit_replay_group(input, group, replay_source)
                };
                if let Err(detail) = admission_result {
                    let input = self
                        .world_mut()
                        .resource_mut::<InputState>()
                        .expect("input integration was checked before replay");
                    cleanup_failed_admission_frame(input, &plan.replay_sources);
                    return AutomationInputReplayReport::failed_after_start(
                        AutomationInputReplayOutcome::InvalidOrRejectedInput,
                        completed_frames,
                        frame.frame_ordinal,
                        Some(group_index),
                        format!("normalized replay input was rejected: {detail}"),
                    );
                }
            }

            if let Err(error) = self.run_headless_for_frames(1) {
                let input = self
                    .world_mut()
                    .resource_mut::<InputState>()
                    .expect("input integration was checked before replay");
                cleanup_replay_sources(input, &plan.replay_sources);
                return AutomationInputReplayReport::failed_after_start(
                    AutomationInputReplayOutcome::InfrastructureFailure,
                    completed_frames,
                    frame.frame_ordinal,
                    None,
                    format!("failed to advance replay App frame: {error:#}"),
                );
            }
            completed_frames += 1;
        }

        AutomationInputReplayReport::completed(trace.frames.len())
    }

    fn teardown_automation_input_replay(
        &mut self,
        trace: &AutomationInputTrace,
        source_map: &AutomationInputReplaySourceMap,
    ) -> Result<&mut Self, AutomationInputReplayTeardownError> {
        let plan = input_replay_preflight(
            trace,
            source_map,
            AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
        )
        .map_err(|_| AutomationInputReplayTeardownError::InvalidSourceMapping)?;
        let input = self
            .world_mut()
            .resource_mut::<InputState>()
            .map_err(|_| AutomationInputReplayTeardownError::InputIntegrationUnavailable)?;
        cleanup_replay_sources(input, &plan.replay_sources);
        Ok(self)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::InputFinalizePlugin;
    use crate::runtime::{ResMut, Update};
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

    fn source_map(
        entries: impl IntoIterator<Item = (u64, u64)>,
    ) -> AutomationInputReplaySourceMap {
        AutomationInputReplaySourceMap::new(
            entries
                .into_iter()
                .map(|(recorded, replay)| {
                    (InputSourceId::new(recorded), InputSourceId::new(replay))
                }),
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
            AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
        );
        assert_eq!(
            missing.outcome(),
            AutomationInputReplayOutcome::InvalidSourceMapping
        );
        assert!(!app.world().resource::<InputState>().unwrap().left_mouse_down());

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
            AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
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
            AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
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
            AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
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
                    AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
                )
                .outcome(),
            AutomationInputReplayOutcome::TargetStateConflict
        );
        assert!(capture
            .world()
            .resource::<InputState>()
            .unwrap()
            .admitted_input_capture_active());

        let mut tracing = trace_app();
        tracing.start_automation_input_trace().unwrap();
        assert_eq!(
            tracing
                .replay_automation_input_trace(
                    &trace,
                    &mapping,
                    AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
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
                    AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
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
                AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
            )
            .outcome(),
            AutomationInputReplayOutcome::TargetStateConflict
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
            AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
        );
        assert_eq!(report.outcome(), AutomationInputReplayOutcome::Completed);
        assert_eq!(report.completed_frames(), 3);
        assert_eq!(app.world().resource::<ReplayFrameCounter>().unwrap().0, 3);
        let input = app.world().resource::<InputState>().unwrap();
        assert!(input.left_mouse_down());
        assert!(input.right_mouse_down(), "unrelated held source must survive replay");

        app.teardown_automation_input_replay(&trace, &mapping)
            .expect("replay teardown should succeed");
        let input = app.world().resource::<InputState>().unwrap();
        assert!(!input.left_mouse_down());
        assert!(input.right_mouse_down());
    }

    #[test]
    fn replay_remaps_atomic_tablet_group_and_source_time_context() {
        let recorded_context =
            InputContext::new(InputSourceId::new(2_030), Some(InputDeviceId::new(77)));
        let source_time = SourceTime::new(
            recorded_context,
            123_456,
            SourceTimeUnit::Microseconds,
        );
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
            let remapped_time = tablet.source_time.expect("source time should remain present");
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
            AutomationInputReplayInitialState::RecordedSourcesNeutralAtCaptureStart,
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


}
