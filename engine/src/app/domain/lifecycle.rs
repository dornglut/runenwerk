use std::error::Error;
use std::fmt;

use anyhow::{Result, anyhow};
use state_machine::{Lifecycle, LifecycleModel, TransitionModel};

/// The five states actually supported by the current Engine runtime.
///
/// `ShuttingDown` and `Terminated` must only be added when owner-correct
/// Host shutdown and settlement are implemented and verified (ADR 0026).
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct InvalidAppTransition {
    from: AppLifecycleState,
    event: AppLifecycleEvent,
}

impl fmt::Display for InvalidAppTransition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid App lifecycle transition from {} using {:?}",
            self.from.name(),
            self.event
        )
    }
}

impl Error for InvalidAppTransition {}

#[derive(Debug)]
struct AppLifecyclePolicy;

impl TransitionModel for AppLifecyclePolicy {
    type State = AppLifecycleState;
    type Event = AppLifecycleEvent;
    type Error = InvalidAppTransition;

    fn initial() -> Self::State {
        AppLifecycleState::Configuring
    }

    fn next(
        from: Self::State,
        event: &Self::Event,
    ) -> std::result::Result<Self::State, Self::Error> {
        use AppLifecycleEvent as E;
        use AppLifecycleState as S;

        match (from, event) {
            (S::Configuring, E::Prepare) => Ok(S::Prepared),
            (S::Prepared, E::BeginStartup) => Ok(S::Starting),
            (S::Starting, E::StartupSucceeded) => Ok(S::Running),
            (S::Starting, E::StartupFailed) => Ok(S::Failed),
            _ => Err(InvalidAppTransition {
                from,
                event: *event,
            }),
        }
    }
}

impl LifecycleModel for AppLifecyclePolicy {
    fn is_terminal(state: Self::State) -> bool {
        state == AppLifecycleState::Failed
    }
}

/// App-owned facade around reusable state validation.
///
/// The Engine owns when to report preparation/startup facts. No generic
/// machine is permitted to execute Startup or to retire App resources.
#[derive(Debug, Default)]
pub(crate) struct AppLifecycle {
    machine: Lifecycle<AppLifecyclePolicy>,
}

impl AppLifecycle {
    pub(crate) fn state(&self) -> AppLifecycleState {
        self.machine.state()
    }

    pub(crate) fn is_configuring(&self) -> bool {
        self.state() == AppLifecycleState::Configuring
    }

    pub(crate) fn name(&self) -> &'static str {
        self.state().name()
    }

    pub(crate) fn prepare_for_execution(&mut self) -> Result<()> {
        match self.state() {
            AppLifecycleState::Configuring => {
                self.machine.transition(AppLifecycleEvent::Prepare)?;
                Ok(())
            }
            AppLifecycleState::Prepared | AppLifecycleState::Running => Ok(()),
            AppLifecycleState::Starting => Err(anyhow!(
                "App lifecycle is Starting; the same Startup attempt cannot be prepared again"
            )),
            AppLifecycleState::Failed => Err(anyhow!(
                "App lifecycle is Failed; the same runtime instance cannot be advanced again"
            )),
        }
    }

    /// True means the Startup schedule may now be executed exactly once.
    /// False means successful Startup was already performed.
    pub(crate) fn begin_startup(&mut self) -> Result<bool> {
        match self.state() {
            AppLifecycleState::Running => Ok(false),
            AppLifecycleState::Prepared => {
                self.machine.transition(AppLifecycleEvent::BeginStartup)?;
                Ok(true)
            }
            AppLifecycleState::Configuring => Err(anyhow!(
                "Startup requires prepared App composition; lifecycle is Configuring"
            )),
            AppLifecycleState::Starting => Err(anyhow!(
                "Startup was already attempted for this App; lifecycle is Starting"
            )),
            AppLifecycleState::Failed => Err(anyhow!(
                "Startup previously failed for this App; the same runtime instance is non-runnable"
            )),
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
    fn preparation_seals_composition_and_is_repeatable() {
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
        assert!(lifecycle.begin_startup().unwrap());
        lifecycle.startup_succeeded().unwrap();
        assert!(!lifecycle.begin_startup().unwrap());
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
    fn failed_startup_is_non_retryable() {
        let mut lifecycle = AppLifecycle::default();
        lifecycle.prepare_for_execution().unwrap();
        lifecycle.begin_startup().unwrap();
        lifecycle.startup_failed().unwrap();
        assert_eq!(lifecycle.state(), AppLifecycleState::Failed);
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
}
