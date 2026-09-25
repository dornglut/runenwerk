//! Bounded in-process application automation orchestration.
//!
//! This module owns session/mode/result mechanics only. Product command and query vocabularies
//! remain in their owning crates.

use std::fmt::Display;
use std::time::Duration;

use crate::plugins::InputState;
pub use runen_input::{
    DigitalState, InputObservation, InputSourceId, PointerButton, PointerButtonInput,
    RelativeMotionUnit, ScrollDelta, ScrollDomain, ScrollInput, Vector2,
};
use runen_input::{ContinuityLoss, InputContext};

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

#[derive(Debug, Clone, PartialEq)]
pub enum AutomationStepResult<T> {
    Dispatched,
    AdmittedOrDelivered,
    EffectConfirmed(T),
    AssertionPassed,
    Unsupported,
    Inconclusive,
    Cancelled,
    InfrastructureFailure(String),
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
}

impl AutomationSession {
    pub const fn new(id: AutomationSessionId, source: InputSourceId) -> Self {
        Self {
            id,
            source,
            active: true,
            cancelled: false,
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

    pub fn dispatch_product<A: AutomationOwnerAdapter>(
        &mut self,
        mode: AutomationExecutionMode,
        adapter: &mut A,
        target: &A::Target,
        command: A::Command,
    ) -> AutomationStepResult<()> {
        if !self.active {
            return self.inactive_result();
        }
        if mode != AutomationExecutionMode::ProductSemantic {
            return AutomationStepResult::Unsupported;
        }
        match adapter.dispatch(target, command) {
            Ok(()) => AutomationStepResult::Dispatched,
            Err(error) => AutomationStepResult::InfrastructureFailure(error.to_string()),
        }
    }

    pub fn query_owner<A: AutomationOwnerAdapter>(
        &mut self,
        adapter: &mut A,
        target: &A::Target,
        query: A::Query,
    ) -> AutomationStepResult<A::Observation> {
        if !self.active {
            return self.inactive_result();
        }
        match adapter.query(target, query) {
            Ok(observation) => AutomationStepResult::EffectConfirmed(observation),
            Err(error) => AutomationStepResult::InfrastructureFailure(error.to_string()),
        }
    }

    pub fn inject_normalized(
        &mut self,
        mode: AutomationExecutionMode,
        input: &mut InputState,
        observation: InputObservation,
    ) -> AutomationStepResult<()> {
        if !self.active {
            return self.inactive_result();
        }
        if mode != AutomationExecutionMode::NormalizedInput {
            return AutomationStepResult::Unsupported;
        }

        let context = InputContext::new(self.source, None);
        match input.admit_automation_observation(context, observation) {
            Ok(true) => AutomationStepResult::AdmittedOrDelivered,
            Ok(false) => AutomationStepResult::Unsupported,
            Err(error) => AutomationStepResult::InfrastructureFailure(format!(
                "normalized automation input rejected: {error:?}"
            )),
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
        if !self.active {
            return self.inactive_result();
        }

        let start = elapsed();
        loop {
            if !self.active {
                return self.inactive_result();
            }
            let observation = match adapter.query(target, query.clone()) {
                Ok(observation) => observation,
                Err(error) => {
                    return AutomationStepResult::InfrastructureFailure(error.to_string());
                }
            };
            if predicate(&observation) {
                return AutomationStepResult::EffectConfirmed(observation);
            }
            if elapsed().saturating_sub(start) >= timeout {
                return AutomationStepResult::Inconclusive;
            }
            std::thread::yield_now();
        }
    }

    pub fn finish(&mut self, input: &mut InputState) -> AutomationStepResult<()> {
        if !self.active {
            return self.inactive_result();
        }
        self.cleanup_input(input);
        self.active = false;
        AutomationStepResult::EffectConfirmed(())
    }

    pub fn cancel(&mut self, input: &mut InputState) -> AutomationStepResult<()> {
        if !self.active {
            return self.inactive_result();
        }
        self.cleanup_input(input);
        self.cancelled = true;
        self.active = false;
        AutomationStepResult::Cancelled
    }

    fn cleanup_input(&self, input: &mut InputState) {
        input.handle_continuity_loss(
            InputContext::new(self.source, None),
            ContinuityLoss::Source,
        );
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let session = AutomationSession::new(
            AutomationSessionId::new(41),
            InputSourceId::new(901),
        );
        assert_eq!(session.id().raw(), 41);
        assert_eq!(session.source(), InputSourceId::new(901));
        assert!(session.is_active());
        assert!(!session.is_cancelled());
    }

    #[test]
    fn native_mode_is_unsupported_without_normalized_fallback() {
        let mut session = AutomationSession::new(
            AutomationSessionId::new(42),
            InputSourceId::new(902),
        );
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
        let mut session = AutomationSession::new(
            AutomationSessionId::new(43),
            InputSourceId::new(903),
        );
        let mut input = InputState::new();

        assert_eq!(
            session.inject_normalized(
                AutomationExecutionMode::NormalizedInput,
                &mut input,
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

        assert_eq!(
            session.finish(&mut input),
            AutomationStepResult::EffectConfirmed(())
        );

        assert!(!input.left_mouse_down());
        assert!(input.right_mouse_down());
        assert!(!input.left_mouse_released());
        assert!(input.mouse_button_transitions().is_empty());
    }

    #[test]
    fn condition_wait_confirms_owner_state_and_times_out_deterministically() {
        let mut session = AutomationSession::new(
            AutomationSessionId::new(44),
            InputSourceId::new(904),
        );
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
    fn cancellation_prevents_later_mutation_and_cleans_owned_input() {
        let mut session = AutomationSession::new(
            AutomationSessionId::new(45),
            InputSourceId::new(905),
        );
        let mut input = InputState::new();
        let mut adapter = DummyAdapter::default();

        assert_eq!(
            session.inject_normalized(
                AutomationExecutionMode::NormalizedInput,
                &mut input,
                InputObservation::PointerButton(PointerButtonInput {
                    button: PointerButton::Left,
                    state: DigitalState::Pressed,
                }),
            ),
            AutomationStepResult::AdmittedOrDelivered
        );
        assert_eq!(session.cancel(&mut input), AutomationStepResult::Cancelled);
        assert!(!input.left_mouse_down());

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
    }
}
