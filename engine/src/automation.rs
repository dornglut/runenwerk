//! Bounded in-process application automation orchestration.
//!
//! This module owns session/mode/result mechanics only. Product command and query vocabularies
//! remain in their owning crates.

use std::fmt::{self, Display};
use std::time::Duration;

use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::InputState;
use crate::plugins::input::input_integration_is_active;
use crate::runtime::{CoreSet, FrameEnd, ResMut, SystemConfigExt};
use runen_input::{ContinuityLoss, InputContext};
pub use runen_input::{
    DigitalState, InputObservation, InputObservationGroup, InputSourceId, PointerButton,
    PointerButtonInput, RelativeMotionUnit, ScrollDelta, ScrollDomain, ScrollInput, Vector2,
};

#[derive(Debug, Clone, PartialEq)]
pub struct AutomationInputTraceFrame {
    frame_ordinal: u64,
    groups: Vec<InputObservationGroup>,
}

impl AutomationInputTraceFrame {
    pub fn frame_ordinal(&self) -> u64 {
        self.frame_ordinal
    }

    pub fn groups(&self) -> &[InputObservationGroup] {
        &self.groups
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AutomationInputTrace {
    frames: Vec<AutomationInputTraceFrame>,
    trailing_groups: Vec<InputObservationGroup>,
}

impl AutomationInputTrace {
    pub fn frames(&self) -> &[AutomationInputTraceFrame] {
        &self.frames
    }

    pub fn trailing_groups(&self) -> &[InputObservationGroup] {
        &self.trailing_groups
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationInputTraceControlError {
    TraceIntegrationUnavailable,
    InputIntegrationUnavailable,
    TraceAlreadyActive,
    TraceNotActive,
    AdmittedInputCaptureAlreadyActive,
    AdmittedInputCaptureInactive,
}

impl fmt::Display for AutomationInputTraceControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TraceIntegrationUnavailable => {
                "automation input trace integration is not selected"
            }
            Self::InputIntegrationUnavailable => "Runenwerk input integration is unavailable",
            Self::TraceAlreadyActive => "automation input trace is already active",
            Self::TraceNotActive => "automation input trace is not active",
            Self::AdmittedInputCaptureAlreadyActive => {
                "admitted-input capture is already owned by another caller"
            }
            Self::AdmittedInputCaptureInactive => {
                "automation input trace lost its admitted-input capture"
            }
        })
    }
}

impl std::error::Error for AutomationInputTraceControlError {}

#[derive(Debug, Default, runen_ecs::Resource)]
struct AutomationInputTraceRecorder {
    active: bool,
    next_frame_ordinal: u64,
    frames: Vec<AutomationInputTraceFrame>,
}

impl AutomationInputTraceRecorder {
    fn start(&mut self) {
        self.active = true;
        self.next_frame_ordinal = 0;
        self.frames.clear();
    }

    fn finish(&mut self, trailing_groups: Vec<InputObservationGroup>) -> AutomationInputTrace {
        self.active = false;
        self.next_frame_ordinal = 0;
        AutomationInputTrace {
            frames: std::mem::take(&mut self.frames),
            trailing_groups,
        }
    }

    fn close_frame(&mut self, groups: Vec<InputObservationGroup>) {
        if !self.active {
            return;
        }
        self.frames.push(AutomationInputTraceFrame {
            frame_ordinal: self.next_frame_ordinal,
            groups,
        });
        self.next_frame_ordinal = self.next_frame_ordinal.saturating_add(1);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AutomationInputTracePlugin;

impl Plugin for AutomationInputTracePlugin {
    fn build(&self, app: &mut App) {
        if !input_integration_is_active(app.world()) {
            app.record_missing_capability("AutomationInputTracePlugin", "InputFinalizePlugin");
            return;
        }

        app.init_resource::<AutomationInputTraceRecorder>();
        app.add_systems(
            FrameEnd,
            automation_input_trace_frame_end_system.after(CoreSet::FrameEnd),
        );
    }
}

fn automation_input_trace_frame_end_system(
    mut input: ResMut<InputState>,
    mut recorder: ResMut<AutomationInputTraceRecorder>,
) {
    if !recorder.active {
        return;
    }
    recorder.close_frame(input.drain_admitted_input_capture());
}

pub trait AppAutomationInputTraceExt {
    fn start_automation_input_trace(
        &mut self,
    ) -> Result<&mut Self, AutomationInputTraceControlError>;

    fn stop_automation_input_trace(
        &mut self,
    ) -> Result<AutomationInputTrace, AutomationInputTraceControlError>;

    fn automation_input_trace_active(&self) -> bool;
}

impl AppAutomationInputTraceExt for App {
    fn start_automation_input_trace(
        &mut self,
    ) -> Result<&mut Self, AutomationInputTraceControlError> {
        let recorder = self
            .world()
            .resource::<AutomationInputTraceRecorder>()
            .map_err(|_| AutomationInputTraceControlError::TraceIntegrationUnavailable)?;
        if recorder.active {
            return Err(AutomationInputTraceControlError::TraceAlreadyActive);
        }

        let input = self
            .world()
            .resource::<InputState>()
            .map_err(|_| AutomationInputTraceControlError::InputIntegrationUnavailable)?;
        if input.admitted_input_capture_active() {
            return Err(AutomationInputTraceControlError::AdmittedInputCaptureAlreadyActive);
        }

        self.world_mut()
            .resource_mut::<AutomationInputTraceRecorder>()
            .expect("trace integration presence was checked")
            .start();
        self.world_mut()
            .resource_mut::<InputState>()
            .expect("input integration presence was checked")
            .start_admitted_input_capture();
        Ok(self)
    }

    fn stop_automation_input_trace(
        &mut self,
    ) -> Result<AutomationInputTrace, AutomationInputTraceControlError> {
        let recorder = self
            .world()
            .resource::<AutomationInputTraceRecorder>()
            .map_err(|_| AutomationInputTraceControlError::TraceIntegrationUnavailable)?;
        if !recorder.active {
            return Err(AutomationInputTraceControlError::TraceNotActive);
        }

        let input = self
            .world()
            .resource::<InputState>()
            .map_err(|_| AutomationInputTraceControlError::InputIntegrationUnavailable)?;
        if !input.admitted_input_capture_active() {
            return Err(AutomationInputTraceControlError::AdmittedInputCaptureInactive);
        }

        let trailing_groups = self
            .world_mut()
            .resource_mut::<InputState>()
            .expect("input integration presence was checked")
            .stop_admitted_input_capture();

        Ok(self
            .world_mut()
            .resource_mut::<AutomationInputTraceRecorder>()
            .expect("trace integration presence was checked")
            .finish(trailing_groups))
    }

    fn automation_input_trace_active(&self) -> bool {
        self.world()
            .resource::<AutomationInputTraceRecorder>()
            .is_ok_and(|recorder| recorder.active)
    }
}

mod persistence;
pub use persistence::*;
mod replay;
pub use replay::*;
mod scenario_persistence;
pub use scenario_persistence::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AutomationSessionId(u64);

impl AutomationSessionId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationExecutionMode {
    ProductSemantic,
    NormalizedInput,
    NativeOs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationStepKind {
    ProductDispatch,
    OwnerQuery,
    OwnerAssertion,
    NormalizedInjection,
    PersistedTraceReplay,
    ConditionWait,
    ReplayCleanup,
    Finish,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationStepOutcome {
    Dispatched,
    AdmittedOrDelivered,
    EffectConfirmed,
    AssertionPassed,
    AssertionFailed,
    Unsupported,
    Rejected,
    Inconclusive,
    Cancelled,
    InfrastructureFailure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AutomationStepResult<T> {
    Dispatched,
    AdmittedOrDelivered,
    EffectConfirmed(T),
    AssertionPassed,
    AssertionFailed,
    Unsupported,
    Inconclusive,
    Cancelled,
    InfrastructureFailure(String),
}

impl<T> AutomationStepResult<T> {
    pub const fn outcome(&self) -> AutomationStepOutcome {
        match self {
            Self::Dispatched => AutomationStepOutcome::Dispatched,
            Self::AdmittedOrDelivered => AutomationStepOutcome::AdmittedOrDelivered,
            Self::EffectConfirmed(_) => AutomationStepOutcome::EffectConfirmed,
            Self::AssertionPassed => AutomationStepOutcome::AssertionPassed,
            Self::AssertionFailed => AutomationStepOutcome::AssertionFailed,
            Self::Unsupported => AutomationStepOutcome::Unsupported,
            Self::Inconclusive => AutomationStepOutcome::Inconclusive,
            Self::Cancelled => AutomationStepOutcome::Cancelled,
            Self::InfrastructureFailure(_) => AutomationStepOutcome::InfrastructureFailure,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutomationReplayProgress {
    outcome: AutomationInputReplayOutcome,
    completed_frames: u64,
    failing_frame_ordinal: Option<u64>,
    failing_group_index: Option<usize>,
}

impl AutomationReplayProgress {
    fn from_report(report: &AutomationInputReplayReport) -> Self {
        Self {
            outcome: report.outcome(),
            completed_frames: report.completed_frames(),
            failing_frame_ordinal: report.failing_frame_ordinal(),
            failing_group_index: report.failing_group_index(),
        }
    }

    pub const fn outcome(&self) -> AutomationInputReplayOutcome {
        self.outcome
    }

    pub const fn completed_frames(&self) -> u64 {
        self.completed_frames
    }

    pub const fn failing_frame_ordinal(&self) -> Option<u64> {
        self.failing_frame_ordinal
    }

    pub const fn failing_group_index(&self) -> Option<usize> {
        self.failing_group_index
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationStepRecord {
    sequence: u64,
    kind: AutomationStepKind,
    execution_mode: Option<AutomationExecutionMode>,
    outcome: AutomationStepOutcome,
    replay_progress: Option<AutomationReplayProgress>,
    diagnostic_detail: Option<String>,
}

impl AutomationStepRecord {
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn kind(&self) -> AutomationStepKind {
        self.kind
    }

    pub const fn execution_mode(&self) -> Option<AutomationExecutionMode> {
        self.execution_mode
    }

    pub const fn outcome(&self) -> AutomationStepOutcome {
        self.outcome
    }

    pub fn replay_progress(&self) -> Option<&AutomationReplayProgress> {
        self.replay_progress.as_ref()
    }

    pub fn diagnostic_detail(&self) -> Option<&str> {
        self.diagnostic_detail.as_deref()
    }
}

pub const MAX_AUTOMATION_STEP_DIAGNOSTIC_BYTES: usize = 2_048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationPersistedReplayError {
    SessionClosed { cancelled: bool },
    UnsupportedMode(AutomationExecutionMode),
    ReplayLeasePending,
    Import(AutomationInputTraceImportError),
    SourceIdentityExhausted,
    Replay(AutomationInputReplayReport),
}

impl fmt::Display for AutomationPersistedReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionClosed { cancelled: true } => {
                formatter.write_str("automation session is cancelled")
            }
            Self::SessionClosed { cancelled: false } => {
                formatter.write_str("automation session is already closed")
            }
            Self::UnsupportedMode(mode) => write!(
                formatter,
                "persisted normalized replay is unsupported in execution mode {mode:?}"
            ),
            Self::ReplayLeasePending => formatter
                .write_str("automation session already owns a completed replay awaiting teardown"),
            Self::Import(error) => write!(
                formatter,
                "persisted normalized replay import failed: {error}"
            ),
            Self::SourceIdentityExhausted => {
                formatter.write_str("exhausted replay-owned input source identity")
            }
            Self::Replay(report) => write!(
                formatter,
                "normalized replay failed: outcome={:?}, completed_frames={}, failing_frame={:?}, failing_group={:?}, detail={}",
                report.outcome(),
                report.completed_frames(),
                report.failing_frame_ordinal(),
                report.failing_group_index(),
                report.detail().unwrap_or("none"),
            ),
        }
    }
}

impl std::error::Error for AutomationPersistedReplayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Import(error) => Some(error),
            Self::SessionClosed { .. }
            | Self::UnsupportedMode(_)
            | Self::ReplayLeasePending
            | Self::SourceIdentityExhausted
            | Self::Replay(_) => None,
        }
    }
}

pub trait AutomationOwnerAdapter {
    type Target;
    type Command;
    type Query: Clone;
    type Observation;
    type Error: Display;

    fn dispatch(
        &mut self,
        target: &Self::Target,
        command: Self::Command,
    ) -> Result<(), Self::Error>;

    fn query(
        &mut self,
        target: &Self::Target,
        query: Self::Query,
    ) -> Result<Self::Observation, Self::Error>;
}

#[derive(Debug)]
pub struct AutomationSession {
    id: AutomationSessionId,
    source: InputSourceId,
    active: bool,
    cancelled: bool,
    next_step_sequence: u64,
    history: Vec<AutomationStepRecord>,
    replay_teardown_pending: bool,
}

impl AutomationSession {
    pub const fn new(id: AutomationSessionId, source: InputSourceId) -> Self {
        Self {
            id,
            source,
            active: true,
            cancelled: false,
            next_step_sequence: 0,
            history: Vec::new(),
            replay_teardown_pending: false,
        }
    }

    pub const fn id(&self) -> AutomationSessionId {
        self.id
    }

    pub const fn source(&self) -> InputSourceId {
        self.source
    }

    pub const fn is_active(&self) -> bool {
        self.active
    }

    pub const fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn history(&self) -> &[AutomationStepRecord] {
        &self.history
    }

    pub const fn replay_teardown_pending(&self) -> bool {
        self.replay_teardown_pending
    }

    pub fn dispatch_product<A: AutomationOwnerAdapter>(
        &mut self,
        mode: AutomationExecutionMode,
        adapter: &mut A,
        target: &A::Target,
        command: A::Command,
    ) -> AutomationStepResult<()> {
        let result = if !self.active {
            self.inactive_result()
        } else if mode != AutomationExecutionMode::ProductSemantic {
            AutomationStepResult::Unsupported
        } else {
            match adapter.dispatch(target, command) {
                Ok(()) => AutomationStepResult::Dispatched,
                Err(error) => AutomationStepResult::InfrastructureFailure(error.to_string()),
            }
        };
        self.record_step_result(
            AutomationStepKind::ProductDispatch,
            Some(mode),
            &result,
            None,
        );
        result
    }

    pub fn query_owner<A: AutomationOwnerAdapter>(
        &mut self,
        adapter: &mut A,
        target: &A::Target,
        query: A::Query,
    ) -> AutomationStepResult<A::Observation> {
        let result = if !self.active {
            self.inactive_result()
        } else {
            match adapter.query(target, query) {
                Ok(observation) => AutomationStepResult::EffectConfirmed(observation),
                Err(error) => AutomationStepResult::InfrastructureFailure(error.to_string()),
            }
        };
        self.record_step_result(AutomationStepKind::OwnerQuery, None, &result, None);
        result
    }

    pub fn assert_observation<T, P>(
        &mut self,
        observation: &T,
        predicate: P,
    ) -> AutomationStepResult<()>
    where
        P: FnOnce(&T) -> bool,
    {
        let result = if !self.active {
            self.inactive_result()
        } else if predicate(observation) {
            AutomationStepResult::AssertionPassed
        } else {
            AutomationStepResult::AssertionFailed
        };
        self.record_step_result(AutomationStepKind::OwnerAssertion, None, &result, None);
        result
    }

    pub fn inject_normalized(
        &mut self,
        mode: AutomationExecutionMode,
        input: &mut InputState,
        observation: InputObservation,
    ) -> AutomationStepResult<()> {
        let result = if !self.active {
            self.inactive_result()
        } else if mode != AutomationExecutionMode::NormalizedInput {
            AutomationStepResult::Unsupported
        } else {
            let context = InputContext::new(self.source, None);
            match input.admit_automation_observation(context, observation) {
                Ok(true) => AutomationStepResult::AdmittedOrDelivered,
                Ok(false) => AutomationStepResult::Unsupported,
                Err(error) => AutomationStepResult::InfrastructureFailure(format!(
                    "normalized automation input rejected: {error:?}"
                )),
            }
        };
        self.record_step_result(
            AutomationStepKind::NormalizedInjection,
            Some(mode),
            &result,
            None,
        );
        result
    }

    pub fn replay_persisted_normalized_trace(
        &mut self,
        mode: AutomationExecutionMode,
        app: &mut App,
        bytes: &[u8],
        state_assumption: AutomationInputReplayStateAssumption,
    ) -> Result<AutomationInputReplayReport, AutomationPersistedReplayError> {
        if !self.active {
            let outcome = if self.cancelled {
                AutomationStepOutcome::Cancelled
            } else {
                AutomationStepOutcome::InfrastructureFailure
            };
            self.record_step(
                AutomationStepKind::PersistedTraceReplay,
                Some(mode),
                outcome,
                None,
                Some(if self.cancelled {
                    "automation session is cancelled".to_owned()
                } else {
                    "automation session is already closed".to_owned()
                }),
            );
            return Err(AutomationPersistedReplayError::SessionClosed {
                cancelled: self.cancelled,
            });
        }
        if mode != AutomationExecutionMode::NormalizedInput {
            self.record_step(
                AutomationStepKind::PersistedTraceReplay,
                Some(mode),
                AutomationStepOutcome::Unsupported,
                None,
                Some(format!(
                    "persisted normalized replay is unsupported in execution mode {mode:?}"
                )),
            );
            return Err(AutomationPersistedReplayError::UnsupportedMode(mode));
        }
        if self.replay_teardown_pending {
            self.record_step(
                AutomationStepKind::PersistedTraceReplay,
                Some(mode),
                AutomationStepOutcome::Rejected,
                None,
                Some(
                    "automation session already owns a completed replay awaiting teardown"
                        .to_owned(),
                ),
            );
            return Err(AutomationPersistedReplayError::ReplayLeasePending);
        }

        let imported = match import_automation_input_trace(bytes) {
            Ok(imported) => imported,
            Err(error) => {
                let outcome = import_error_step_outcome(&error);
                let detail = error.to_string();
                self.record_step(
                    AutomationStepKind::PersistedTraceReplay,
                    Some(mode),
                    outcome,
                    None,
                    Some(detail),
                );
                return Err(AutomationPersistedReplayError::Import(error));
            }
        };
        let source_map = match self.fresh_replay_source_map(imported.trace()) {
            Ok(source_map) => source_map,
            Err(error) => {
                self.record_step(
                    AutomationStepKind::PersistedTraceReplay,
                    Some(mode),
                    AutomationStepOutcome::InfrastructureFailure,
                    None,
                    Some(error.to_string()),
                );
                return Err(error);
            }
        };

        let report =
            app.replay_automation_input_trace(imported.trace(), &source_map, state_assumption);
        let progress = Some(AutomationReplayProgress::from_report(&report));
        let outcome = replay_step_outcome(report.outcome());
        let detail = if report.outcome() == AutomationInputReplayOutcome::Completed {
            None
        } else {
            Some(
                report
                    .detail()
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("normalized replay failed: {:?}", report.outcome())),
            )
        };
        self.record_step(
            AutomationStepKind::PersistedTraceReplay,
            Some(mode),
            outcome,
            progress,
            detail,
        );

        if report.outcome() == AutomationInputReplayOutcome::Completed {
            self.replay_teardown_pending = true;
            Ok(report)
        } else {
            Err(AutomationPersistedReplayError::Replay(report))
        }
    }

    pub fn wait_for<A, P, E>(
        &mut self,
        adapter: &mut A,
        target: &A::Target,
        query: A::Query,
        timeout: Duration,
        mut elapsed: E,
        predicate: P,
    ) -> AutomationStepResult<A::Observation>
    where
        A: AutomationOwnerAdapter,
        P: Fn(&A::Observation) -> bool,
        E: FnMut() -> Duration,
    {
        let result = if !self.active {
            self.inactive_result()
        } else {
            let start = elapsed();
            loop {
                if !self.active {
                    break self.inactive_result();
                }
                let observation = match adapter.query(target, query.clone()) {
                    Ok(observation) => observation,
                    Err(error) => {
                        break AutomationStepResult::InfrastructureFailure(error.to_string());
                    }
                };
                if predicate(&observation) {
                    break AutomationStepResult::EffectConfirmed(observation);
                }
                if elapsed().saturating_sub(start) >= timeout {
                    break AutomationStepResult::Inconclusive;
                }
                std::thread::yield_now();
            }
        };
        self.record_step_result(AutomationStepKind::ConditionWait, None, &result, None);
        result
    }

    pub fn finish(&mut self, app: &mut App) -> AutomationStepResult<()> {
        let result = if !self.active {
            self.inactive_result()
        } else if let Err(detail) = self.cleanup_owned_state(app) {
            AutomationStepResult::InfrastructureFailure(detail)
        } else {
            self.active = false;
            AutomationStepResult::EffectConfirmed(())
        };
        self.record_step_result(AutomationStepKind::Finish, None, &result, None);
        result
    }

    pub fn cancel(&mut self, app: &mut App) -> AutomationStepResult<()> {
        let result = if !self.active {
            self.inactive_result()
        } else if let Err(detail) = self.cleanup_owned_state(app) {
            AutomationStepResult::InfrastructureFailure(detail)
        } else {
            self.cancelled = true;
            self.active = false;
            AutomationStepResult::Cancelled
        };
        self.record_step_result(AutomationStepKind::Cancel, None, &result, None);
        result
    }

    fn cleanup_owned_state(&mut self, app: &mut App) -> Result<(), String> {
        self.cleanup_completed_replay(app)?;

        let input = app
            .world_mut()
            .resource_mut::<InputState>()
            .map_err(|_| "Runenwerk input integration is unavailable".to_owned())?;
        input.handle_continuity_loss(InputContext::new(self.source, None), ContinuityLoss::Source);
        Ok(())
    }

    fn cleanup_completed_replay(&mut self, app: &mut App) -> Result<(), String> {
        if !self.replay_teardown_pending {
            return Ok(());
        }

        match app.teardown_automation_input_replay() {
            Ok(_) => {
                self.replay_teardown_pending = false;
                let result = AutomationStepResult::EffectConfirmed(());
                self.record_step_result(
                    AutomationStepKind::ReplayCleanup,
                    Some(AutomationExecutionMode::NormalizedInput),
                    &result,
                    None,
                );
                Ok(())
            }
            Err(error) => {
                let detail = error.to_string();
                let result = AutomationStepResult::<()>::InfrastructureFailure(detail.clone());
                self.record_step_result(
                    AutomationStepKind::ReplayCleanup,
                    Some(AutomationExecutionMode::NormalizedInput),
                    &result,
                    None,
                );
                Err(detail)
            }
        }
    }

    fn fresh_replay_source_map(
        &self,
        trace: &AutomationInputTrace,
    ) -> Result<AutomationInputReplaySourceMap, AutomationPersistedReplayError> {
        let mut recorded_sources = Vec::new();
        for frame in trace.frames() {
            for group in frame.groups() {
                if !recorded_sources.contains(&group.context.source) {
                    recorded_sources.push(group.context.source);
                }
            }
        }

        let mut entries = Vec::with_capacity(recorded_sources.len());
        let mut next_raw = u64::MAX;
        for recorded in &recorded_sources {
            let replay_source = loop {
                let candidate = InputSourceId::new(next_raw);
                next_raw = next_raw
                    .checked_sub(1)
                    .ok_or(AutomationPersistedReplayError::SourceIdentityExhausted)?;
                if candidate != self.source
                    && !recorded_sources.contains(&candidate)
                    && !entries
                        .iter()
                        .any(|(_, replay): &(InputSourceId, InputSourceId)| *replay == candidate)
                {
                    break candidate;
                }
            };
            entries.push((*recorded, replay_source));
        }

        Ok(AutomationInputReplaySourceMap::new(entries))
    }

    fn record_step_result<T>(
        &mut self,
        kind: AutomationStepKind,
        execution_mode: Option<AutomationExecutionMode>,
        result: &AutomationStepResult<T>,
        replay_progress: Option<AutomationReplayProgress>,
    ) {
        let detail = match result {
            AutomationStepResult::InfrastructureFailure(detail) => Some(detail.clone()),
            _ => None,
        };
        self.record_step(
            kind,
            execution_mode,
            result.outcome(),
            replay_progress,
            detail,
        );
    }

    fn record_step(
        &mut self,
        kind: AutomationStepKind,
        execution_mode: Option<AutomationExecutionMode>,
        outcome: AutomationStepOutcome,
        replay_progress: Option<AutomationReplayProgress>,
        diagnostic_detail: Option<String>,
    ) {
        let sequence = self.next_step_sequence;
        self.next_step_sequence = self.next_step_sequence.saturating_add(1);
        self.history.push(AutomationStepRecord {
            sequence,
            kind,
            execution_mode,
            outcome,
            replay_progress,
            diagnostic_detail: diagnostic_detail.map(bounded_diagnostic_detail),
        });
    }

    fn inactive_result<T>(&self) -> AutomationStepResult<T> {
        if self.cancelled {
            AutomationStepResult::Cancelled
        } else {
            AutomationStepResult::InfrastructureFailure(
                "automation session is already closed".to_owned(),
            )
        }
    }
}

fn bounded_diagnostic_detail(detail: String) -> String {
    if detail.len() <= MAX_AUTOMATION_STEP_DIAGNOSTIC_BYTES {
        return detail;
    }

    let mut end = MAX_AUTOMATION_STEP_DIAGNOSTIC_BYTES;
    while !detail.is_char_boundary(end) {
        end -= 1;
    }
    detail[..end].to_owned()
}

fn import_error_step_outcome(error: &AutomationInputTraceImportError) -> AutomationStepOutcome {
    match error {
        AutomationInputTraceImportError::WrongArtifactKind(_)
        | AutomationInputTraceImportError::UnsupportedSchemaVersion(_)
        | AutomationInputTraceImportError::UnsupportedTraceShape(_)
        | AutomationInputTraceImportError::UnsupportedRecordingWitness => {
            AutomationStepOutcome::Unsupported
        }
        AutomationInputTraceImportError::ArtifactTooLarge
        | AutomationInputTraceImportError::ParseFailure(_)
        | AutomationInputTraceImportError::MalformedArtifact(_)
        | AutomationInputTraceImportError::UnknownField(_)
        | AutomationInputTraceImportError::UnknownVariant(_)
        | AutomationInputTraceImportError::ResourceLimitExceeded(_)
        | AutomationInputTraceImportError::InvalidIdentityReference(_)
        | AutomationInputTraceImportError::InvalidNormalizedInput(_) => {
            AutomationStepOutcome::Rejected
        }
    }
}

const fn replay_step_outcome(outcome: AutomationInputReplayOutcome) -> AutomationStepOutcome {
    match outcome {
        AutomationInputReplayOutcome::Completed => AutomationStepOutcome::AdmittedOrDelivered,
        AutomationInputReplayOutcome::UnsupportedTraceShape => AutomationStepOutcome::Unsupported,
        AutomationInputReplayOutcome::InvalidSourceMapping
        | AutomationInputReplayOutcome::InvalidOrRejectedInput
        | AutomationInputReplayOutcome::UnframedTrailingGroups
        | AutomationInputReplayOutcome::TargetStateConflict => AutomationStepOutcome::Rejected,
        AutomationInputReplayOutcome::InfrastructureFailure => {
            AutomationStepOutcome::InfrastructureFailure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::InputFinalizePlugin;
    use runen_input::{
        ContactId, ContactPhase, ContactPresence, CoordinateSpace, DeliveryRole, EvidenceStatus,
        InputDeviceId, InputToolKind, ObservationOrigin, PhysicalTabletControls, Point2,
        TabletCapabilities, TabletObservation,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DummyCommand {
        Increment,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DummyQuery {
        Value,
    }

    #[derive(Debug, Default)]
    struct DummyAdapter {
        value: u32,
    }

    impl AutomationOwnerAdapter for DummyAdapter {
        type Target = ();
        type Command = DummyCommand;
        type Query = DummyQuery;
        type Observation = u32;
        type Error = &'static str;

        fn dispatch(
            &mut self,
            _target: &Self::Target,
            command: Self::Command,
        ) -> Result<(), Self::Error> {
            match command {
                DummyCommand::Increment => self.value += 1,
            }
            Ok(())
        }

        fn query(
            &mut self,
            _target: &Self::Target,
            _query: Self::Query,
        ) -> Result<Self::Observation, Self::Error> {
            Ok(self.value)
        }
    }

    #[test]
    fn session_identity_and_modes_are_explicit() {
        let session = AutomationSession::new(AutomationSessionId::new(41), InputSourceId::new(901));
        assert_eq!(session.id().raw(), 41);
        assert_eq!(session.source(), InputSourceId::new(901));
        assert!(session.is_active());
        assert!(!session.is_cancelled());
    }

    #[test]
    fn native_mode_is_unsupported_without_normalized_fallback() {
        let mut session =
            AutomationSession::new(AutomationSessionId::new(42), InputSourceId::new(902));
        let mut input = InputState::new();

        assert_eq!(
            session.inject_normalized(
                AutomationExecutionMode::NativeOs,
                &mut input,
                InputObservation::PointerButton(PointerButtonInput {
                    button: PointerButton::Left,
                    state: DigitalState::Pressed,
                }),
            ),
            AutomationStepResult::Unsupported
        );
        assert!(!input.left_mouse_down());
    }

    #[test]
    fn cleanup_invalidates_only_automation_owned_state_without_release_edges() {
        let mut session =
            AutomationSession::new(AutomationSessionId::new(43), InputSourceId::new(903));
        let mut app = App::headless();
        app.add_plugin(InputFinalizePlugin);

        {
            let input = app
                .world_mut()
                .resource_mut::<InputState>()
                .expect("input fixture should install InputState");
            assert_eq!(
                session.inject_normalized(
                    AutomationExecutionMode::NormalizedInput,
                    input,
                    InputObservation::PointerButton(PointerButtonInput {
                        button: PointerButton::Left,
                        state: DigitalState::Pressed,
                    }),
                ),
                AutomationStepResult::AdmittedOrDelivered
            );
            input.handle_mouse_input(
                winit::event::ElementState::Pressed,
                winit::event::MouseButton::Right,
            );
            input.clear_frame();

            assert!(input.left_mouse_down());
            assert!(input.right_mouse_down());
        }

        assert_eq!(
            session.finish(&mut app),
            AutomationStepResult::EffectConfirmed(())
        );

        let input = app
            .world()
            .resource::<InputState>()
            .expect("input fixture should retain InputState");
        assert!(!input.left_mouse_down());
        assert!(input.right_mouse_down());
        assert!(!input.left_mouse_released());
        assert!(input.mouse_button_transitions().is_empty());
    }

    #[test]
    fn condition_wait_confirms_owner_state_and_times_out_deterministically() {
        let mut session =
            AutomationSession::new(AutomationSessionId::new(44), InputSourceId::new(904));
        let mut adapter = DummyAdapter::default();
        assert_eq!(
            session.dispatch_product(
                AutomationExecutionMode::ProductSemantic,
                &mut adapter,
                &(),
                DummyCommand::Increment,
            ),
            AutomationStepResult::Dispatched
        );

        let mut ticks = 0u64;
        let confirmed = session.wait_for(
            &mut adapter,
            &(),
            DummyQuery::Value,
            Duration::from_millis(5),
            || {
                ticks += 1;
                Duration::from_millis(ticks)
            },
            |value| *value == 1,
        );
        assert_eq!(confirmed, AutomationStepResult::EffectConfirmed(1));

        let mut ticks = 0u64;
        let timeout = session.wait_for(
            &mut adapter,
            &(),
            DummyQuery::Value,
            Duration::from_millis(3),
            || {
                ticks += 1;
                Duration::from_millis(ticks)
            },
            |value| *value == 99,
        );
        assert_eq!(timeout, AutomationStepResult::Inconclusive);
    }

    #[test]
    fn normalized_automation_uses_the_shared_admitted_input_capture_seam() {
        let mut session =
            AutomationSession::new(AutomationSessionId::new(46), InputSourceId::new(906));
        let mut input = InputState::new();
        input.start_admitted_input_capture();

        assert_eq!(
            session.inject_normalized(
                AutomationExecutionMode::NormalizedInput,
                &mut input,
                InputObservation::RelativeMotion {
                    delta: Vector2::new(4.0, -3.0),
                    unit: RelativeMotionUnit::BackendDeviceUnits,
                },
            ),
            AutomationStepResult::AdmittedOrDelivered
        );

        let captured = input.stop_admitted_input_capture();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].context.source, session.source());
        assert_eq!(captured[0].context.device, None);
        assert_eq!(
            captured[0].observations,
            vec![InputObservation::RelativeMotion {
                delta: Vector2::new(4.0, -3.0),
                unit: RelativeMotionUnit::BackendDeviceUnits,
            }]
        );
    }

    fn trace_app() -> App {
        let mut app = App::headless();
        app.add_plugin(InputFinalizePlugin);
        app.add_plugin(AutomationInputTracePlugin);
        app
    }

    fn admit_left_press(app: &mut App, source: u64) {
        app.world_mut()
            .resource_mut::<InputState>()
            .expect("trace fixture should install InputState")
            .handle_pointer_button(
                InputContext::new(InputSourceId::new(source), None),
                PointerButtonInput {
                    button: PointerButton::Left,
                    state: DigitalState::Pressed,
                },
            );
    }

    #[test]
    fn input_trace_is_explicit_and_requires_input_integration() {
        let mut bare = App::headless();
        assert!(matches!(
            bare.start_automation_input_trace(),
            Err(AutomationInputTraceControlError::TraceIntegrationUnavailable)
        ));

        let mut missing_input = App::headless();
        missing_input.add_plugin(AutomationInputTracePlugin);
        let error = match missing_input.run_for_frames(0) {
            Ok(_) => panic!("trace integration without input must fail composition"),
            Err(error) => error,
        };
        assert!(
            format!("{error:#}").contains("AutomationInputTracePlugin")
                && format!("{error:#}").contains("InputFinalizePlugin")
        );
    }

    #[test]
    fn framed_trace_preserves_group_order_idle_frames_and_capture_local_ordinals() {
        let mut app = trace_app();
        app.start_automation_input_trace()
            .expect("trace should start");

        admit_left_press(&mut app, 1_001);
        app.world_mut()
            .resource_mut::<InputState>()
            .expect("trace fixture should install InputState")
            .handle_relative_motion(
                InputContext::new(InputSourceId::new(1_002), None),
                3.0,
                -2.0,
            );

        app = app
            .run_for_frames(1)
            .expect("first traced frame should run");
        app = app.run_for_frames(1).expect("idle traced frame should run");

        app.world_mut()
            .resource_mut::<InputState>()
            .expect("trace fixture should install InputState")
            .handle_scroll_input(
                InputContext::new(InputSourceId::new(1_003), None),
                ScrollInput {
                    delta: ScrollDelta::vertical_only(1.25),
                    domain: ScrollDomain::Unspecified,
                    phase: None,
                },
            );
        app = app
            .run_for_frames(1)
            .expect("third traced frame should run");

        let trace = app
            .stop_automation_input_trace()
            .expect("trace should stop");
        assert_eq!(trace.frames().len(), 3);
        assert_eq!(trace.frames()[0].frame_ordinal(), 0);
        assert_eq!(trace.frames()[1].frame_ordinal(), 1);
        assert_eq!(trace.frames()[2].frame_ordinal(), 2);
        assert_eq!(trace.frames()[0].groups().len(), 2);
        assert!(trace.frames()[1].groups().is_empty());
        assert_eq!(trace.frames()[2].groups().len(), 1);
        assert_eq!(
            trace.frames()[0].groups()[0].context.source,
            InputSourceId::new(1_001)
        );
        assert_eq!(
            trace.frames()[0].groups()[1].context.source,
            InputSourceId::new(1_002)
        );
        assert!(trace.trailing_groups().is_empty());
        assert!(!app.automation_input_trace_active());
    }

    #[test]
    fn manual_frame_clear_does_not_fabricate_trace_frame_and_stop_keeps_trailing_groups() {
        let mut app = trace_app();
        app.start_automation_input_trace()
            .expect("trace should start");
        app.world_mut()
            .resource_mut::<InputState>()
            .expect("trace fixture should install InputState")
            .handle_relative_motion(InputContext::new(InputSourceId::new(1_010), None), 7.0, 8.0);
        app.world_mut()
            .resource_mut::<InputState>()
            .expect("trace fixture should install InputState")
            .clear_frame();

        let trace = app
            .stop_automation_input_trace()
            .expect("trace should stop");
        assert!(trace.frames().is_empty());
        assert_eq!(trace.trailing_groups().len(), 1);
        assert!(matches!(
            trace.trailing_groups()[0].observations.as_slice(),
            [InputObservation::RelativeMotion { .. }]
        ));
    }

    #[test]
    fn framed_trace_refuses_to_steal_standalone_capture_and_restarts_at_zero() {
        let mut app = trace_app();
        {
            let input = app
                .world_mut()
                .resource_mut::<InputState>()
                .expect("trace fixture should install InputState");
            input.start_admitted_input_capture();
            input.handle_mouse_wheel_delta(0.5);
        }

        assert!(matches!(
            app.start_automation_input_trace(),
            Err(AutomationInputTraceControlError::AdmittedInputCaptureAlreadyActive)
        ));
        assert_eq!(
            app.world_mut()
                .resource_mut::<InputState>()
                .expect("trace fixture should install InputState")
                .stop_admitted_input_capture()
                .len(),
            1,
            "failed framed-trace start must not clear another caller's capture"
        );

        app.start_automation_input_trace()
            .expect("first framed trace should start");
        app = app
            .run_for_frames(1)
            .expect("first framed trace should advance");
        let first = app
            .stop_automation_input_trace()
            .expect("first framed trace should stop");
        assert_eq!(first.frames()[0].frame_ordinal(), 0);

        app.start_automation_input_trace()
            .expect("second framed trace should start");
        app = app
            .run_for_frames(1)
            .expect("second framed trace should advance");
        let second = app
            .stop_automation_input_trace()
            .expect("second framed trace should stop");
        assert_eq!(second.frames()[0].frame_ordinal(), 0);
    }

    #[test]
    fn framed_trace_preserves_automation_source_and_atomic_tablet_group() {
        let mut app = trace_app();
        app.start_automation_input_trace()
            .expect("trace should start");

        let mut session =
            AutomationSession::new(AutomationSessionId::new(47), InputSourceId::new(1_020));
        {
            let input = app
                .world_mut()
                .resource_mut::<InputState>()
                .expect("trace fixture should install InputState");
            assert_eq!(
                session.inject_normalized(
                    AutomationExecutionMode::NormalizedInput,
                    input,
                    InputObservation::RelativeMotion {
                        delta: Vector2::new(1.0, 2.0),
                        unit: RelativeMotionUnit::BackendDeviceUnits,
                    },
                ),
                AutomationStepResult::AdmittedOrDelivered
            );
        }

        let tablet_context =
            InputContext::new(InputSourceId::new(1_021), Some(InputDeviceId::new(77)));
        let tablet = |contact, delivery| {
            InputObservation::Tablet(TabletObservation {
                contact: ContactId::new(contact),
                tool: None,
                tool_kind: InputToolKind::Pen,
                phase: ContactPhase::Update,
                presence: ContactPresence::Contact,
                position: Point2::new(contact as f32, 10.0, CoordinateSpace::WindowPhysicalPixels),
                delta: Vector2::new(1.0, 0.0),
                pressure: None,
                tangential_pressure: None,
                tilt: None,
                twist: None,
                controls: PhysicalTabletControls::default(),
                capabilities: TabletCapabilities::default(),
                source_time: None,
                evidence: EvidenceStatus::ObservedConfirmed,
                delivery,
                origin: ObservationOrigin::SourceReport,
            })
        };
        let tablet_group = InputObservationGroup::new(
            tablet_context,
            vec![
                tablet(1, DeliveryRole::HistoricalCoalesced),
                tablet(2, DeliveryRole::OrdinaryCurrent),
            ],
        );
        app.world_mut()
            .resource_mut::<InputState>()
            .expect("trace fixture should install InputState")
            .admit_device_observation_group(tablet_group.clone())
            .expect("tablet group should admit");

        app = app.run_for_frames(1).expect("framed trace should advance");
        let trace = app
            .stop_automation_input_trace()
            .expect("trace should stop");

        assert_eq!(trace.frames().len(), 1);
        assert_eq!(trace.frames()[0].groups().len(), 2);
        assert_eq!(
            trace.frames()[0].groups()[0].context.source,
            session.source()
        );
        assert_eq!(
            trace.frames()[0].groups()[1],
            tablet_group,
            "tablet observations must remain one exact atomic group"
        );
    }

    #[test]
    fn session_history_distinguishes_query_assertion_pass_and_failure() {
        let mut session =
            AutomationSession::new(AutomationSessionId::new(48), InputSourceId::new(907));
        let mut adapter = DummyAdapter::default();

        assert_eq!(
            session.dispatch_product(
                AutomationExecutionMode::ProductSemantic,
                &mut adapter,
                &(),
                DummyCommand::Increment,
            ),
            AutomationStepResult::Dispatched
        );
        let observed = match session.query_owner(&mut adapter, &(), DummyQuery::Value) {
            AutomationStepResult::EffectConfirmed(value) => value,
            other => panic!("owner query should confirm value, got {other:?}"),
        };
        assert_eq!(
            session.assert_observation(&observed, |value| *value == 2),
            AutomationStepResult::AssertionFailed
        );
        assert_eq!(
            session.assert_observation(&observed, |value| *value == 1),
            AutomationStepResult::AssertionPassed
        );

        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::sequence)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::kind)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepKind::ProductDispatch,
                AutomationStepKind::OwnerQuery,
                AutomationStepKind::OwnerAssertion,
                AutomationStepKind::OwnerAssertion,
            ]
        );
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::outcome)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepOutcome::Dispatched,
                AutomationStepOutcome::EffectConfirmed,
                AutomationStepOutcome::AssertionFailed,
                AutomationStepOutcome::AssertionPassed,
            ]
        );
    }

    #[test]
    fn session_persisted_replay_keeps_state_until_finish_then_cleans_it() {
        let mut recording = trace_app();
        recording
            .start_automation_input_trace()
            .expect("trace should start");
        admit_left_press(&mut recording, 2_001);
        recording = recording
            .run_for_frames(1)
            .expect("recorded press frame should run");
        let trace = recording
            .stop_automation_input_trace()
            .expect("trace should stop");
        let encoded = export_automation_input_trace_v1(
            &trace,
            AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart,
            None,
        )
        .expect("press trace should persist");

        let mut replay = trace_app();
        let mut session =
            AutomationSession::new(AutomationSessionId::new(49), InputSourceId::new(2_100));
        let report = session
            .replay_persisted_normalized_trace(
                AutomationExecutionMode::NormalizedInput,
                &mut replay,
                encoded.as_bytes(),
                AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
            )
            .expect("persisted press trace should replay");
        assert_eq!(report.outcome(), AutomationInputReplayOutcome::Completed);
        assert!(session.replay_teardown_pending());

        let held = replay
            .world()
            .resource::<InputState>()
            .expect("replay target should retain InputState")
            .left_mouse_down();
        assert!(
            held,
            "completed replay state must remain observable before finish"
        );
        assert_eq!(
            session.assert_observation(&held, |value| *value),
            AutomationStepResult::AssertionPassed
        );

        assert_eq!(
            session.finish(&mut replay),
            AutomationStepResult::EffectConfirmed(())
        );
        assert!(!session.replay_teardown_pending());
        assert!(
            !replay
                .world()
                .resource::<InputState>()
                .expect("replay target should retain InputState")
                .left_mouse_down()
        );

        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::kind)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepKind::PersistedTraceReplay,
                AutomationStepKind::OwnerAssertion,
                AutomationStepKind::ReplayCleanup,
                AutomationStepKind::Finish,
            ]
        );
        assert_eq!(
            session.history()[0].outcome(),
            AutomationStepOutcome::AdmittedOrDelivered,
            "completed normalized replay confirms delivery, not product semantic effect"
        );
        assert_eq!(
            session.history()[0]
                .replay_progress()
                .expect("replay step should retain progress")
                .completed_frames(),
            1
        );
    }

    #[test]
    fn malformed_persisted_replay_keeps_typed_error_and_schedules_no_teardown() {
        let mut replay = trace_app();
        let mut session =
            AutomationSession::new(AutomationSessionId::new(50), InputSourceId::new(2_101));

        let error = session
            .replay_persisted_normalized_trace(
                AutomationExecutionMode::NormalizedInput,
                &mut replay,
                b"this is not RON",
                AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
            )
            .expect_err("malformed persisted trace should fail");
        let AutomationPersistedReplayError::Import(import_error) = error else {
            panic!("malformed persisted trace should preserve the typed A8 import error");
        };
        assert!(
            !import_error.to_string().is_empty(),
            "typed A8 import error should retain diagnostic detail"
        );
        assert!(!session.replay_teardown_pending());
        assert_eq!(
            session.finish(&mut replay),
            AutomationStepResult::EffectConfirmed(())
        );
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::kind)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepKind::PersistedTraceReplay,
                AutomationStepKind::Finish,
            ],
            "failed import must not fabricate replay cleanup"
        );
        assert_eq!(
            session.history()[0].outcome(),
            AutomationStepOutcome::Rejected
        );
    }

    #[test]
    fn native_os_persisted_replay_is_unsupported_without_fallback() {
        let mut replay = trace_app();
        let mut session =
            AutomationSession::new(AutomationSessionId::new(51), InputSourceId::new(2_102));

        assert_eq!(
            session
                .replay_persisted_normalized_trace(
                    AutomationExecutionMode::NativeOs,
                    &mut replay,
                    b"unused because mode is rejected first",
                    AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
                )
                .expect_err("NativeOs replay must be unsupported"),
            AutomationPersistedReplayError::UnsupportedMode(AutomationExecutionMode::NativeOs)
        );
        assert_eq!(session.history().len(), 1);
        assert_eq!(
            session.history()[0].outcome(),
            AutomationStepOutcome::Unsupported
        );
        assert!(!session.replay_teardown_pending());
    }

    #[test]
    fn history_bounds_infrastructure_diagnostic_detail() {
        struct LongErrorAdapter;

        impl AutomationOwnerAdapter for LongErrorAdapter {
            type Target = ();
            type Command = ();
            type Query = ();
            type Observation = ();
            type Error = String;

            fn dispatch(
                &mut self,
                _target: &Self::Target,
                _command: Self::Command,
            ) -> Result<(), Self::Error> {
                Err("x".repeat(MAX_AUTOMATION_STEP_DIAGNOSTIC_BYTES + 128))
            }

            fn query(
                &mut self,
                _target: &Self::Target,
                _query: Self::Query,
            ) -> Result<Self::Observation, Self::Error> {
                Ok(())
            }
        }

        let mut session =
            AutomationSession::new(AutomationSessionId::new(52), InputSourceId::new(2_103));
        let mut adapter = LongErrorAdapter;
        assert!(matches!(
            session.dispatch_product(
                AutomationExecutionMode::ProductSemantic,
                &mut adapter,
                &(),
                (),
            ),
            AutomationStepResult::InfrastructureFailure(_)
        ));
        assert!(
            session.history()[0]
                .diagnostic_detail()
                .expect("failure history should retain bounded detail")
                .len()
                <= MAX_AUTOMATION_STEP_DIAGNOSTIC_BYTES
        );
    }

    #[test]
    fn replay_target_rejection_schedules_no_teardown() {
        let mut recording = trace_app();
        recording
            .start_automation_input_trace()
            .expect("recording trace should start");
        admit_left_press(&mut recording, 2_106);
        recording = recording
            .run_for_frames(1)
            .expect("recording frame should run");
        let trace = recording
            .stop_automation_input_trace()
            .expect("recording trace should stop");
        let encoded = export_automation_input_trace_v1(
            &trace,
            AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart,
            None,
        )
        .expect("trace should persist");

        let mut replay = trace_app();
        replay
            .world_mut()
            .resource_mut::<InputState>()
            .expect("replay fixture should install InputState")
            .handle_mouse_motion(3.0, 4.0);

        let mut session =
            AutomationSession::new(AutomationSessionId::new(54), InputSourceId::new(2_107));
        let error = session
            .replay_persisted_normalized_trace(
                AutomationExecutionMode::NormalizedInput,
                &mut replay,
                encoded.as_bytes(),
                AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
            )
            .expect_err("dirty replay target should be rejected");
        let AutomationPersistedReplayError::Replay(report) = error else {
            panic!("target-state rejection should remain an A6 replay report");
        };
        assert_eq!(
            report.outcome(),
            AutomationInputReplayOutcome::TargetStateConflict
        );
        assert_eq!(report.completed_frames(), 0);
        assert!(!session.replay_teardown_pending());
        assert_eq!(session.history().len(), 1);
        assert_eq!(
            session.history()[0].kind(),
            AutomationStepKind::PersistedTraceReplay
        );
        assert_eq!(
            session.history()[0].outcome(),
            AutomationStepOutcome::Rejected
        );
        assert_eq!(
            session.history()[0]
                .replay_progress()
                .expect("A6 rejection should retain replay progress")
                .outcome(),
            AutomationInputReplayOutcome::TargetStateConflict
        );

        assert_eq!(
            session.finish(&mut replay),
            AutomationStepResult::EffectConfirmed(())
        );
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::kind)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepKind::PersistedTraceReplay,
                AutomationStepKind::Finish,
            ],
            "rejected replay must not fabricate replay cleanup"
        );
    }

    #[test]
    fn replay_teardown_failure_remains_visible_and_retryable() {
        let mut recording = trace_app();
        recording
            .start_automation_input_trace()
            .expect("recording trace should start");
        admit_left_press(&mut recording, 2_104);
        recording = recording
            .run_for_frames(1)
            .expect("recording frame should run");
        let trace = recording
            .stop_automation_input_trace()
            .expect("recording trace should stop");
        let encoded = export_automation_input_trace_v1(
            &trace,
            AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart,
            None,
        )
        .expect("trace should persist");

        let mut replay = trace_app();
        let mut session =
            AutomationSession::new(AutomationSessionId::new(53), InputSourceId::new(2_105));
        let report = session
            .replay_persisted_normalized_trace(
                AutomationExecutionMode::NormalizedInput,
                &mut replay,
                encoded.as_bytes(),
                AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
            )
            .expect("persisted replay should complete");
        assert_eq!(report.outcome(), AutomationInputReplayOutcome::Completed);
        assert!(session.replay_teardown_pending());

        replay
            .start_automation_input_trace()
            .expect("active evidence capture should block replay teardown");
        assert!(matches!(
            session.finish(&mut replay),
            AutomationStepResult::InfrastructureFailure(_)
        ));
        assert!(session.is_active());
        assert!(session.replay_teardown_pending());
        assert_eq!(
            session.history()[1].kind(),
            AutomationStepKind::ReplayCleanup
        );
        assert_eq!(
            session.history()[1].outcome(),
            AutomationStepOutcome::InfrastructureFailure
        );
        assert_eq!(session.history()[2].kind(), AutomationStepKind::Finish);
        assert_eq!(
            session.history()[2].outcome(),
            AutomationStepOutcome::InfrastructureFailure
        );

        let _ = replay
            .stop_automation_input_trace()
            .expect("evidence capture should stop");
        assert_eq!(
            session.finish(&mut replay),
            AutomationStepResult::EffectConfirmed(())
        );
        assert!(!session.is_active());
        assert!(!session.replay_teardown_pending());
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::kind)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepKind::PersistedTraceReplay,
                AutomationStepKind::ReplayCleanup,
                AutomationStepKind::Finish,
                AutomationStepKind::ReplayCleanup,
                AutomationStepKind::Finish,
            ]
        );
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::outcome)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepOutcome::AdmittedOrDelivered,
                AutomationStepOutcome::InfrastructureFailure,
                AutomationStepOutcome::InfrastructureFailure,
                AutomationStepOutcome::EffectConfirmed,
                AutomationStepOutcome::EffectConfirmed,
            ]
        );
    }

    #[test]
    fn cancellation_prevents_later_mutation_and_cleans_owned_input() {
        let mut session =
            AutomationSession::new(AutomationSessionId::new(45), InputSourceId::new(905));
        let mut app = App::headless();
        app.add_plugin(InputFinalizePlugin);
        let mut adapter = DummyAdapter::default();

        {
            let input = app
                .world_mut()
                .resource_mut::<InputState>()
                .expect("input fixture should install InputState");
            assert_eq!(
                session.inject_normalized(
                    AutomationExecutionMode::NormalizedInput,
                    input,
                    InputObservation::PointerButton(PointerButtonInput {
                        button: PointerButton::Left,
                        state: DigitalState::Pressed,
                    }),
                ),
                AutomationStepResult::AdmittedOrDelivered
            );
        }
        assert_eq!(session.cancel(&mut app), AutomationStepResult::Cancelled);
        assert!(
            !app.world()
                .resource::<InputState>()
                .expect("input fixture should retain InputState")
                .left_mouse_down()
        );

        assert_eq!(
            session.dispatch_product(
                AutomationExecutionMode::ProductSemantic,
                &mut adapter,
                &(),
                DummyCommand::Increment,
            ),
            AutomationStepResult::Cancelled
        );
        assert_eq!(adapter.value, 0);
        assert_eq!(
            session
                .history()
                .iter()
                .map(AutomationStepRecord::kind)
                .collect::<Vec<_>>(),
            vec![
                AutomationStepKind::NormalizedInjection,
                AutomationStepKind::Cancel,
                AutomationStepKind::ProductDispatch,
            ]
        );
    }

    #[test]
    fn session_version_aware_persisted_replay_accepts_v2_absolute_pointer() {
        let trace = AutomationInputTrace {
            frames: vec![AutomationInputTraceFrame {
                frame_ordinal: 0,
                groups: vec![InputObservationGroup::single(
                    InputContext::new(InputSourceId::new(1), None),
                    InputObservation::AbsolutePointerPosition {
                        position: Point2::new(18.0, 27.0, CoordinateSpace::WindowPhysicalPixels),
                    },
                )],
            }],
            trailing_groups: Vec::new(),
        };
        let encoded = export_automation_input_trace_v2(
            &trace,
            AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart,
            None,
        )
        .expect("V2 absolute pointer trace should persist");

        let mut replay = App::headless();
        replay.add_plugin(InputFinalizePlugin);
        let mut session =
            AutomationSession::new(AutomationSessionId::new(55), InputSourceId::new(2_108));

        let report = session
            .replay_persisted_normalized_trace(
                AutomationExecutionMode::NormalizedInput,
                &mut replay,
                encoded.as_bytes(),
                AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine,
            )
            .expect("A12 version-aware persisted replay should accept V2");
        assert_eq!(report.outcome(), AutomationInputReplayOutcome::Completed);
        assert!(session.replay_teardown_pending());
        assert_eq!(
            session.history()[0].outcome(),
            AutomationStepOutcome::AdmittedOrDelivered
        );

        assert_eq!(
            session.finish(&mut replay),
            AutomationStepResult::EffectConfirmed(())
        );
        assert!(!session.replay_teardown_pending());
        assert!(!session.is_active());
    }
}
