//! Normalized in-memory application input-trace replay.
//!
//! This module owns only the first bounded headless replay contract accepted by the application
//! automation semantic model. Scene replay, RunenUI replay, native input injection, persistence,
//! and authored scenarios remain separate owners.

use std::collections::HashSet;
use std::fmt;

use crate::app::App;
use crate::plugins::InputState;
use crate::plugins::input::input_integration_is_active;
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

/// Caller-supplied state assumption required by the first bounded replay contract.
///
/// A4 traces do not contain a complete snapshot of state that predates capture. This witness
/// therefore asserts both that each recorded source was pristine when capture began and that each
/// mapped replay source is fresh/exclusively replay-owned before replay starts. "Pristine" includes
/// no pre-existing held controls, active contacts, or retained absolute-pointer position.
///
/// Runenwerk preflight verifies the observable conflicts available to this integration layer, but
/// the caller remains responsible for source-local state that RunenInput does not expose for
/// enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationInputReplayStateAssumption {
    RecordedAndReplaySourcesPristine,
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
    ReplayNotActive,
    InputIntegrationUnavailable,
    EvidenceCaptureActive,
}

impl fmt::Display for AutomationInputReplayTeardownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ReplayNotActive => "no completed automation input replay is awaiting teardown",
            Self::InputIntegrationUnavailable => "Runenwerk input integration is unavailable",
            Self::EvidenceCaptureActive => {
                "automation input replay teardown would contaminate active input capture"
            }
        })
    }
}

impl std::error::Error for AutomationInputReplayTeardownError {}

pub trait AppAutomationInputReplayExt {
    fn replay_automation_input_trace(
        &mut self,
        trace: &AutomationInputTrace,
        source_map: &AutomationInputReplaySourceMap,
        state_assumption: AutomationInputReplayStateAssumption,
    ) -> AutomationInputReplayReport;

    fn teardown_automation_input_replay(
        &mut self,
    ) -> Result<&mut Self, AutomationInputReplayTeardownError>;
}

#[derive(Debug)]
struct AutomationInputReplayPlan {
    replay_sources: Vec<InputSourceId>,
    pointer_buttons: Vec<PointerButton>,
}

#[derive(Debug, Default, runen_ecs::Resource)]
struct AutomationInputReplayLeaseState {
    replay_sources: Option<Vec<InputSourceId>>,
}

fn input_replay_preflight(
    trace: &AutomationInputTrace,
    source_map: &AutomationInputReplaySourceMap,
    _state_assumption: AutomationInputReplayStateAssumption,
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

    let replay_sources =
        resolve_replay_sources(&recorded_sources, source_map).map_err(|detail| {
            AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::InvalidSourceMapping,
                detail,
            )
        })?;

    Ok(AutomationInputReplayPlan {
        replay_sources,
        pointer_buttons,
    })
}

fn recorded_sources(trace: &AutomationInputTrace) -> Vec<InputSourceId> {
    let mut sources = Vec::new();
    for frame in &trace.frames {
        for group in &frame.groups {
            if !sources.contains(&group.context.source) {
                sources.push(group.context.source);
            }
        }
    }
    for group in &trace.trailing_groups {
        if !sources.contains(&group.context.source) {
            sources.push(group.context.source);
        }
    }
    sources
}

fn resolve_replay_sources(
    recorded_sources: &[InputSourceId],
    source_map: &AutomationInputReplaySourceMap,
) -> Result<Vec<InputSourceId>, String> {
    let mut replay_sources = Vec::with_capacity(recorded_sources.len());
    let mut unique_replay_sources = HashSet::with_capacity(recorded_sources.len());
    for recorded in recorded_sources {
        let Some(replay) = source_map.mapped_source(*recorded) else {
            return Err(format!(
                "recorded input source {} has no unique replay mapping",
                recorded.raw()
            ));
        };
        if !unique_replay_sources.insert(replay) {
            return Err("distinct recorded input sources map to the same replay source".to_owned());
        }
        replay_sources.push(replay);
    }
    Ok(replay_sources)
}

fn input_replay_target_state_conflict(
    app: &App,
    replay_sources: &[InputSourceId],
    pointer_buttons: &[PointerButton],
) -> Option<String> {
    if app
        .world()
        .resource::<AutomationInputReplayLeaseState>()
        .is_ok_and(|state| state.replay_sources.is_some())
    {
        return Some("an automation input replay is already awaiting teardown".to_owned());
    }
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
    if replay_sources
        .iter()
        .copied()
        .any(|source| input.absolute_pointer_position_for_source(source).is_some())
    {
        return Some(
            "a mapped replay source already has retained absolute-pointer state".to_owned(),
        );
    }
    if pointer_buttons
        .iter()
        .copied()
        .any(|button| input.pointer_button_down_anywhere(button))
    {
        return Some(
            "a pointer button used by the trace is already held in target input state".to_owned(),
        );
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
        input.handle_continuity_loss(InputContext::new(*source, None), ContinuityLoss::Source);
    }
}

fn cleanup_failed_admission_frame(input: &mut InputState, replay_sources: &[InputSourceId]) {
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
        state_assumption: AutomationInputReplayStateAssumption,
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

        let plan = match input_replay_preflight(trace, source_map, state_assumption) {
            Ok(plan) => plan,
            Err(report) => return report,
        };
        if let Some(detail) =
            input_replay_target_state_conflict(self, &plan.replay_sources, &plan.pointer_buttons)
        {
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
        if let Some(detail) =
            input_replay_target_state_conflict(self, &plan.replay_sources, &plan.pointer_buttons)
        {
            return AutomationInputReplayReport::rejected(
                AutomationInputReplayOutcome::TargetStateConflict,
                detail,
            );
        }

        for (completed_frames, frame) in trace.frames.iter().enumerate() {
            let completed_frames = completed_frames as u64;
            for (group_index, group) in frame.groups.iter().enumerate() {
                let replay_source = source_map.mapped_source(group.context.source).expect(
                    "replay preflight established a unique mapping for every recorded source",
                );

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
        }

        self.init_resource::<AutomationInputReplayLeaseState>();
        let lease = self
            .world_mut()
            .resource_mut::<AutomationInputReplayLeaseState>()
            .expect("replay lease resource was initialized after successful replay");
        debug_assert!(
            lease.replay_sources.is_none(),
            "target-state preflight rejects a second replay before teardown"
        );
        lease.replay_sources = Some(plan.replay_sources);

        AutomationInputReplayReport::completed(trace.frames.len())
    }

    fn teardown_automation_input_replay(
        &mut self,
    ) -> Result<&mut Self, AutomationInputReplayTeardownError> {
        if !input_integration_is_active(self.world()) {
            return Err(AutomationInputReplayTeardownError::InputIntegrationUnavailable);
        }
        if self.automation_input_trace_active()
            || self
                .world()
                .resource::<InputState>()
                .is_ok_and(|input| input.admitted_input_capture_active())
        {
            return Err(AutomationInputReplayTeardownError::EvidenceCaptureActive);
        }

        let replay_sources = {
            let lease = self
                .world_mut()
                .resource_mut::<AutomationInputReplayLeaseState>()
                .map_err(|_| AutomationInputReplayTeardownError::ReplayNotActive)?;
            lease
                .replay_sources
                .take()
                .ok_or(AutomationInputReplayTeardownError::ReplayNotActive)?
        };

        let input = self
            .world_mut()
            .resource_mut::<InputState>()
            .map_err(|_| AutomationInputReplayTeardownError::InputIntegrationUnavailable)?;
        cleanup_replay_sources(input, &replay_sources);
        Ok(self)
    }
}

#[cfg(test)]
mod tests;
