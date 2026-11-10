use anyhow::{Result, anyhow};
use state_machine::{Lifecycle, LifecycleState, Next, Transition, TransitionState};

/// States currently supported by the Engine's App execution owner.
///
/// Failure is non-runnable, *not* completed termination: actual shutdown and
/// settlement are still owned by Host and resource owners (ADR 0026).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub(crate) enum AppLifecycleState {
    #[default]
    Configuring,
    Prepared,
    Starting,
    Running,
    Failed,
}

impl AppLifecycleState {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Configuring => "Configuring",
            Self::Prepared => "Prepared",
            Self::Starting => "Starting",
            Self::Running => "Running",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum AppLifecycleEvent {
    Prepare,
    BeginStartup,
    StartupSucceeded,
    StartupFailed,
}

/// Transition-specific diagnostics; no second admission decision table.
#[derive(Debug, thiserror::Error)]
enum AppTransitionError {
    #[error("App lifecycle is Starting; the same Startup attempt cannot be prepared again")]
    PreparationDuringStartup,
    #[error("App lifecycle is Failed; the same runtime instance cannot be advanced again")]
    PreparationAfterFailure,
    #[error("Startup requires prepared App composition; lifecycle is Configuring")]
    StartupWithoutPreparation,
    #[error("Startup was already attempted for this App; lifecycle is Starting")]
    DuplicateStartupAttempt,
    #[error("Startup previously failed for this App; the same runtime instance is non-runnable")]
    StartupAfterFailure,
    #[error("invalid App lifecycle transition from {from:?} using {event:?}")]
    Invalid {
        from: AppLifecycleState,
        event: AppLifecycleEvent,
    },
}

impl TransitionState for AppLifecycleState {
    type Event = AppLifecycleEvent;
    type Error = AppTransitionError;

    /// Single authority for both state changes and accepted idempotent events.
    fn next(&self, event: &Self::Event) -> std::result::Result<Next<Self>, Self::Error> {
        use AppLifecycleEvent as E;
        use AppLifecycleState as S;

        match (*self, event) {
            (S::Configuring, E::Prepare) => Ok(Next::Enter(S::Prepared)),
            (S::Prepared | S::Running, E::Prepare) => Ok(Next::Stay),
            (S::Prepared, E::BeginStartup) => Ok(Next::Enter(S::Starting)),
            (S::Running, E::BeginStartup) => Ok(Next::Stay),
            (S::Starting, E::StartupSucceeded) => Ok(Next::Enter(S::Running)),
            (S::Starting, E::StartupFailed) => Ok(Next::Enter(S::Failed)),
            (S::Starting, E::Prepare) => Err(AppTransitionError::PreparationDuringStartup),
            (S::Failed, E::Prepare) => Err(AppTransitionError::PreparationAfterFailure),
            (S::Configuring, E::BeginStartup) => Err(AppTransitionError::StartupWithoutPreparation),
            (S::Starting, E::BeginStartup) => Err(AppTransitionError::DuplicateStartupAttempt),
            (S::Failed, E::BeginStartup) => Err(AppTransitionError::StartupAfterFailure),
            _ => Err(AppTransitionError::Invalid {
                from: *self,
                event: *event,
            }),
        }
    }
}

impl LifecycleState for AppLifecycleState {
    fn is_terminal(&self) -> bool {
        // Failed prevents further Startup but does not prove Host cleanup
        // and outstanding owner obligations have been settled.
        false
    }
}

/// Explicit disposition: a running instance does not execute Startup again.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum StartupAdmission {
    Execute,
    AlreadyRunning,
}

/// The domain facade exposes App verbs, never raw generic transition access.
#[derive(Debug)]
pub(crate) struct AppLifecycle {
    machine: Lifecycle<AppLifecycleState>,
}

impl Default for AppLifecycle {
    fn default() -> Self {
        Self {
            machine: Lifecycle::new(AppLifecycleState::Configuring),
        }
    }
}

impl AppLifecycle {
    pub(crate) fn state(&self) -> AppLifecycleState {
        *self.machine.state()
    }

    pub(crate) fn is_configuring(&self) -> bool {
        self.state() == AppLifecycleState::Configuring
    }

    pub(crate) fn name(&self) -> &'static str {
        self.state().name()
    }

    pub(crate) fn prepare_for_execution(&mut self) -> Result<()> {
        self.machine.transition(AppLifecycleEvent::Prepare)?;
        Ok(())
    }

    pub(crate) fn begin_startup(&mut self) -> Result<StartupAdmission> {
        match self.machine.transition(AppLifecycleEvent::BeginStartup)? {
            Transition::Changed => Ok(StartupAdmission::Execute),
            Transition::Unchanged => Ok(StartupAdmission::AlreadyRunning),
        }
    }

    pub(crate) fn startup_succeeded(&mut self) -> Result<()> {
        self.machine
            .transition(AppLifecycleEvent::StartupSucceeded)?;
        Ok(())
    }

    pub(crate) fn startup_failed(&mut self) -> Result<()> {
        self.machine.transition(AppLifecycleEvent::StartupFailed)?;
        Ok(())
    }

    pub(crate) fn require_running(&self) -> Result<()> {
        if self.state() == AppLifecycleState::Running {
            return Ok(());
        }
        Err(anyhow!(
            "App frame advancement requires Running lifecycle state; current lifecycle is {}",
            self.name()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preparation_seals_composition_and_is_idempotent() {
        let mut lifecycle = AppLifecycle::default();
        assert!(lifecycle.is_configuring());
        lifecycle.prepare_for_execution().unwrap();
        lifecycle.prepare_for_execution().unwrap();
        assert_eq!(lifecycle.state(), AppLifecycleState::Prepared);
        assert!(!lifecycle.is_configuring());
    }

    #[test]
    fn successful_startup_is_a_single_attempt() {
        let mut lifecycle = AppLifecycle::default();
        lifecycle.prepare_for_execution().unwrap();
        assert_eq!(
            lifecycle.begin_startup().unwrap(),
            StartupAdmission::Execute
        );
        lifecycle.startup_succeeded().unwrap();
        assert_eq!(
            lifecycle.begin_startup().unwrap(),
            StartupAdmission::AlreadyRunning
        );
        lifecycle.prepare_for_execution().unwrap();
        lifecycle.require_running().unwrap();
    }

    #[test]
    fn startup_cannot_skip_preparation() {
        let mut lifecycle = AppLifecycle::default();
        assert!(lifecycle.begin_startup().is_err());
        assert!(lifecycle.startup_succeeded().is_err());
        assert_eq!(lifecycle.state(), AppLifecycleState::Configuring);
    }

    #[test]
    fn failed_startup_is_non_retryable_but_not_settled() {
        let mut lifecycle = AppLifecycle::default();
        lifecycle.prepare_for_execution().unwrap();
        lifecycle.begin_startup().unwrap();
        lifecycle.startup_failed().unwrap();
        assert_eq!(lifecycle.state(), AppLifecycleState::Failed);
        assert!(!lifecycle.machine.is_terminal());
        assert!(lifecycle.begin_startup().is_err());
        assert!(lifecycle.prepare_for_execution().is_err());
        assert!(lifecycle.startup_succeeded().is_err());
    }

    #[test]
    fn interrupted_startup_is_non_retryable() {
        let mut lifecycle = AppLifecycle::default();
        lifecycle.prepare_for_execution().unwrap();
        lifecycle.begin_startup().unwrap();
        assert_eq!(lifecycle.state(), AppLifecycleState::Starting);
        assert!(lifecycle.begin_startup().is_err());
        assert!(lifecycle.prepare_for_execution().is_err());
        assert!(lifecycle.require_running().is_err());
    }

    #[test]
    fn errors_do_not_change_state() {
        let mut lifecycle = AppLifecycle::default();
        let error = lifecycle.begin_startup().unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires prepared App composition")
        );
        assert_eq!(lifecycle.state(), AppLifecycleState::Configuring);
    }
}
