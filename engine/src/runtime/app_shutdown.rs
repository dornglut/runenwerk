//! One App-owned, Native-only graceful shutdown ingress. No new event loop,
//! command registry, ECS schedule or UI/Render execution authority is created.
//! The accepted long-running Headless driver is a separate implementation.

use crate::runtime::native_window_requests::NativeWindowRequests;
use std::sync::{Arc, Mutex};

type ShutdownWake = Arc<dyn Fn() -> bool + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppShutdownReadiness {
    #[default]
    NotReady,
    Ready,
    Stopping,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppShutdownRequestOutcome {
    Accepted,
    AlreadyStopping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AppShutdownError {
    #[error("an open-ended App shutdown handle currently requires a Native-window Host")]
    UnsupportedHost,
    #[error("an App shutdown handle must be acquired before App admission")]
    SealedComposition,
    #[error("Native App shutdown owner is missing")]
    MissingOwner,
    #[error("Native App shutdown has not yet reached event-loop readiness")]
    NotReady,
    #[error("Native App shutdown ingress is closed")]
    Closed,
    #[error("Native App shutdown gate is poisoned")]
    Poisoned,
}

struct Shared {
    readiness: AppShutdownReadiness,
    wake: Option<ShutdownWake>,
    window_requests: NativeWindowRequests,
}

/// A cheap, thread-safe request capability obtained before `App::run(self)`.
/// Acceptance requests owner-controlled termination, not completed cleanup.
/// The caller can inspect `readiness` after the event loop has returned.
#[derive(Clone)]
pub struct AppShutdownHandle {
    shared: Arc<Mutex<Shared>>,
}

impl AppShutdownHandle {
    pub fn readiness(&self) -> Result<AppShutdownReadiness, AppShutdownError> {
        self.shared
            .lock()
            .map(|shared| shared.readiness)
            .map_err(|_| AppShutdownError::Poisoned)
    }

    pub fn request_shutdown(&self) -> Result<AppShutdownRequestOutcome, AppShutdownError> {
        let mut shared = self.shared.lock().map_err(|_| AppShutdownError::Poisoned)?;
        match shared.readiness {
            AppShutdownReadiness::NotReady => Err(AppShutdownError::NotReady),
            AppShutdownReadiness::Closed => Err(AppShutdownError::Closed),
            AppShutdownReadiness::Stopping => Ok(AppShutdownRequestOutcome::AlreadyStopping),
            AppShutdownReadiness::Ready => {
                // Linearize the stopping gate against the sole existing Native
                // window request owner. Earlier tickets settle as Cancelled;
                // later requests fail Closed. Neither can be silently lost.
                shared.window_requests.close();
                if !shared.wake.as_ref().is_some_and(|wake| wake()) {
                    shared.readiness = AppShutdownReadiness::Closed;
                    shared.wake = None;
                    return Err(AppShutdownError::Closed);
                }
                shared.readiness = AppShutdownReadiness::Stopping;
                Ok(AppShutdownRequestOutcome::Accepted)
            }
        }
    }
}

/// World-owned authority. Only Winit turns an accepted shutdown wake into
/// an exit and performs the existing owner-specific terminal cleanup.
pub(crate) struct AppShutdownOwner {
    shared: Arc<Mutex<Shared>>,
}

impl runen_ecs::Resource for AppShutdownOwner {}

impl AppShutdownOwner {
    pub(crate) fn new(window_requests: NativeWindowRequests) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared {
                readiness: AppShutdownReadiness::NotReady,
                wake: None,
                window_requests,
            })),
        }
    }

    pub(crate) fn handle(&self) -> AppShutdownHandle {
        AppShutdownHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    pub(crate) fn attach_wake(
        &self,
        wake: impl Fn() -> bool + Send + Sync + 'static,
    ) -> Result<(), AppShutdownError> {
        let mut shared = self.shared.lock().map_err(|_| AppShutdownError::Poisoned)?;
        match shared.readiness {
            AppShutdownReadiness::Closed => Err(AppShutdownError::Closed),
            AppShutdownReadiness::NotReady if shared.wake.is_none() => {
                shared.wake = Some(Arc::new(wake));
                Ok(())
            }
            _ => Err(AppShutdownError::SealedComposition),
        }
    }

    pub(crate) fn mark_ready(&self) -> Result<(), AppShutdownError> {
        let mut shared = self.shared.lock().map_err(|_| AppShutdownError::Poisoned)?;
        match shared.readiness {
            AppShutdownReadiness::NotReady if shared.wake.is_some() => {
                shared.readiness = AppShutdownReadiness::Ready;
                Ok(())
            }
            AppShutdownReadiness::NotReady => Err(AppShutdownError::NotReady),
            AppShutdownReadiness::Ready | AppShutdownReadiness::Stopping => Ok(()),
            AppShutdownReadiness::Closed => Err(AppShutdownError::Closed),
        }
    }

    pub(crate) fn is_stopping(&self) -> Result<bool, AppShutdownError> {
        let shared = self.shared.lock().map_err(|_| AppShutdownError::Poisoned)?;
        Ok(shared.readiness == AppShutdownReadiness::Stopping)
    }

    /// Close ingress before Native cleanup begins. Stopping is observable
    /// until the owner has actually attempted its terminal obligations.
    pub(crate) fn begin_close(&self) {
        let mut shared = self
            .shared
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if shared.readiness != AppShutdownReadiness::Closed {
            shared.readiness = AppShutdownReadiness::Stopping;
        }
        shared.wake = None;
        shared.window_requests.close();
    }

    pub(crate) fn close(&self) {
        // Even poisoned ownership must close Native admission and return an
        // exact terminal result for every pending accepted window ticket.
        // A pre-run dropped App has no admitted runtime cleanup to observe.
        let mut shared = self
            .shared
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        shared.readiness = AppShutdownReadiness::Closed;
        shared.wake = None;
        shared.window_requests.close();
    }
}

impl Drop for AppShutdownOwner {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::native_window_requests::{
        NativeWindowRequest, NativeWindowRequestError, NativeWindowRequestOutcome,
        NativeWindowRequestOwner, NativeWindowRequestReadiness,
    };
    use std::sync::Barrier;
    use std::time::Duration;

    #[test]
    fn shutdown_gate_is_not_ready_until_native_owner_is_live() {
        let windows = NativeWindowRequestOwner::default();
        let owner = AppShutdownOwner::new(windows.handle());
        let handle = owner.handle();
        assert_eq!(handle.readiness().unwrap(), AppShutdownReadiness::NotReady);
        assert_eq!(handle.request_shutdown(), Err(AppShutdownError::NotReady));
        assert_eq!(owner.mark_ready(), Err(AppShutdownError::NotReady));
        owner.attach_wake(|| true).unwrap();
        owner.mark_ready().unwrap();
        assert_eq!(handle.readiness().unwrap(), AppShutdownReadiness::Ready);
        owner.close();
        assert_eq!(handle.readiness().unwrap(), AppShutdownReadiness::Closed);
        assert_eq!(handle.request_shutdown(), Err(AppShutdownError::Closed));
    }

    #[test]
    fn shutdown_acceptance_closes_window_admission_and_cancels_tickets() {
        let windows = NativeWindowRequestOwner::default();
        windows.attach_wake(|| true).unwrap();
        windows.mark_ready().unwrap();
        let window_handle = windows.handle();
        let ticket = window_handle
            .request(NativeWindowRequest::new("accepted before stop", (64, 64)))
            .unwrap();
        let owner = AppShutdownOwner::new(window_handle.clone());
        owner.attach_wake(|| true).unwrap();
        owner.mark_ready().unwrap();
        let shutdown = owner.handle();
        assert_eq!(
            shutdown.request_shutdown(),
            Ok(AppShutdownRequestOutcome::Accepted)
        );
        assert_eq!(shutdown.readiness(), Ok(AppShutdownReadiness::Stopping));
        assert_eq!(
            window_handle.readiness(),
            Ok(NativeWindowRequestReadiness::Closed)
        );
        assert_eq!(
            ticket.recv_timeout(Duration::from_secs(1)).unwrap(),
            NativeWindowRequestOutcome::Cancelled
        );
        assert_eq!(
            window_handle
                .request(NativeWindowRequest::new("late", (64, 64)))
                .err(),
            Some(NativeWindowRequestError::Closed)
        );
        assert_eq!(
            shutdown.request_shutdown(),
            Ok(AppShutdownRequestOutcome::AlreadyStopping)
        );
        owner.close();
        assert_eq!(shutdown.readiness(), Ok(AppShutdownReadiness::Closed));
    }

    #[test]
    fn stopping_is_visible_until_native_cleanup_finishes() {
        let windows = NativeWindowRequestOwner::default();
        windows.attach_wake(|| true).unwrap();
        windows.mark_ready().unwrap();
        let window_handle = windows.handle();
        let owner = AppShutdownOwner::new(window_handle.clone());
        owner.attach_wake(|| true).unwrap();
        owner.mark_ready().unwrap();
        let handle = owner.handle();

        owner.begin_close();
        assert_eq!(handle.readiness(), Ok(AppShutdownReadiness::Stopping));
        assert_eq!(
            handle.request_shutdown(),
            Ok(AppShutdownRequestOutcome::AlreadyStopping)
        );
        assert_eq!(
            window_handle.readiness(),
            Ok(NativeWindowRequestReadiness::Closed)
        );
        owner.close();
        assert_eq!(handle.readiness(), Ok(AppShutdownReadiness::Closed));
    }

    #[test]
    fn failed_native_wake_never_returns_accepted() {
        let windows = NativeWindowRequestOwner::default();
        let window_handle = windows.handle();
        let owner = AppShutdownOwner::new(window_handle.clone());
        owner.attach_wake(|| false).unwrap();
        owner.mark_ready().unwrap();
        let handle = owner.handle();
        assert_eq!(handle.request_shutdown(), Err(AppShutdownError::Closed));
        assert_eq!(handle.readiness(), Ok(AppShutdownReadiness::Closed));
        assert_eq!(
            window_handle.readiness(),
            Ok(NativeWindowRequestReadiness::Closed)
        );
    }

    #[test]
    fn racing_window_request_and_shutdown_have_terminal_outcomes() {
        for _ in 0..32 {
            let windows = NativeWindowRequestOwner::default();
            windows.attach_wake(|| true).unwrap();
            windows.mark_ready().unwrap();
            let window_handle = windows.handle();
            let owner = AppShutdownOwner::new(window_handle.clone());
            owner.attach_wake(|| true).unwrap();
            owner.mark_ready().unwrap();
            let barrier = Arc::new(Barrier::new(3));
            let b1 = Arc::clone(&barrier);
            let b2 = Arc::clone(&barrier);
            let request = std::thread::spawn(move || {
                b1.wait();
                window_handle.request(NativeWindowRequest::new("racing", (64, 64)))
            });
            let shutdown = owner.handle();
            let stop = std::thread::spawn(move || {
                b2.wait();
                shutdown.request_shutdown()
            });
            barrier.wait();
            let result = request.join().unwrap();
            assert_eq!(
                stop.join().unwrap(),
                Ok(AppShutdownRequestOutcome::Accepted)
            );
            match result {
                Ok(ticket) => assert_eq!(
                    ticket.recv_timeout(Duration::from_secs(1)).unwrap(),
                    NativeWindowRequestOutcome::Cancelled
                ),
                Err(error) => assert_eq!(error, NativeWindowRequestError::Closed),
            }
            owner.close();
        }
    }
}
