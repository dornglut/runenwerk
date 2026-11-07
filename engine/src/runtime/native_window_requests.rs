//! Native-owner scoped, bounded command ingress for secondary window creation.
//!
//! Admission is distinct from Winit realization. These handles own no ECS World,
//! OS window, UI runtime or Render surface. The existing Native window registry
//! remains the sole physical window record authority.

use crate::runtime::window::NativeWindowId;
use std::collections::VecDeque;
use std::sync::{
    Arc, Mutex,
    mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError},
};
use std::time::Duration;

// Bounds pending queue admission only. Drained in-flight work and user title
// byte lengths have separate lifetimes and are not covered by this cap.
const MAX_PENDING_REQUESTS: usize = 64;
type NativeWake = Arc<dyn Fn() -> bool + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NativeWindowRequestReadiness {
    #[default]
    NotReady,
    Ready,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeWindowRequest {
    pub title: String,
    pub size_px: (u32, u32),
}

impl NativeWindowRequest {
    pub fn new(title: impl Into<String>, size_px: (u32, u32)) -> Self {
        Self {
            title: title.into(),
            size_px,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NativeWindowRequestId(u64);

impl NativeWindowRequestId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeWindowRequestOutcome {
    Created {
        native_window_id: NativeWindowId,
    },
    CreationFailed {
        reason: String,
    },
    /// Physical realization failed while owner cleanup/rollback could not be
    /// confirmed. The Native window was not published as Created, but this
    /// process-local identity may still have Render-owned recovery obligations.
    /// Caller should not infer resource retirement from the failed request.
    CreationFailedUnsettled {
        native_window_id: NativeWindowId,
        reason: String,
    },
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NativeWindowRequestError {
    #[error("native window requests require a selected native-window Host")]
    UnsupportedHost,
    #[error("native window requests must be acquired while App is configuring")]
    SealedComposition,
    #[error("native window request owner resource is unavailable")]
    MissingOwner,
    #[error("native window request owner state is poisoned")]
    Poisoned,
    #[error("native window request Host is not ready")]
    NotReady,
    #[error("native window request Host is closed")]
    Closed,
    #[error("native window request queue is full")]
    AtCapacity,
    #[error("native window request identity is exhausted")]
    IdentityExhausted,
    #[error("native window width and height must both be nonzero")]
    InvalidExtent,
}

#[derive(Debug)]
pub struct NativeWindowRequestTicket {
    id: NativeWindowRequestId,
    result: Receiver<NativeWindowRequestOutcome>,
}

impl NativeWindowRequestTicket {
    pub fn id(&self) -> NativeWindowRequestId {
        self.id
    }

    pub fn try_recv(&self) -> Result<NativeWindowRequestOutcome, TryRecvError> {
        self.result.try_recv()
    }

    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<NativeWindowRequestOutcome, RecvTimeoutError> {
        self.result.recv_timeout(timeout)
    }
}

pub(crate) struct PendingNativeWindowRequest {
    pub(crate) request: NativeWindowRequest,
    completion: Option<Sender<NativeWindowRequestOutcome>>,
}

impl PendingNativeWindowRequest {
    pub(crate) fn finish(&mut self, outcome: NativeWindowRequestOutcome) {
        if let Some(completion) = self.completion.take() {
            // A dropped client never changes Native ownership of realized work.
            let _ = completion.send(outcome);
        }
    }
}

impl Drop for PendingNativeWindowRequest {
    fn drop(&mut self) {
        // Any accepted request not explicitly completed by Winit is terminally
        // cancelled, including fatal dispatch errors and exiting the event loop.
        self.finish(NativeWindowRequestOutcome::Cancelled);
    }
}

#[derive(Default)]
struct Shared {
    readiness: NativeWindowRequestReadiness,
    wake: Option<NativeWake>,
    pending: VecDeque<PendingNativeWindowRequest>,
    next_id: u64,
}

/// Cheap, thread-safe Native window ingress, acquired before App::run consumes App.
#[derive(Clone)]
pub struct NativeWindowRequests {
    shared: Arc<Mutex<Shared>>,
}

impl NativeWindowRequests {
    /// Poll owner readiness without binding a thread or waiting indefinitely.
    pub fn readiness(&self) -> Result<NativeWindowRequestReadiness, NativeWindowRequestError> {
        let shared = self
            .shared
            .lock()
            .map_err(|_| NativeWindowRequestError::Poisoned)?;
        Ok(shared.readiness)
    }

    /// Success means admitted and woken, NOT physically realized.
    /// A returned ticket receives exactly one terminal result.
    pub fn request(
        &self,
        request: NativeWindowRequest,
    ) -> Result<NativeWindowRequestTicket, NativeWindowRequestError> {
        if request.size_px.0 == 0 || request.size_px.1 == 0 {
            return Err(NativeWindowRequestError::InvalidExtent);
        }
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| NativeWindowRequestError::Poisoned)?;
        match shared.readiness {
            NativeWindowRequestReadiness::NotReady => {
                return Err(NativeWindowRequestError::NotReady);
            }
            NativeWindowRequestReadiness::Closed => return Err(NativeWindowRequestError::Closed),
            NativeWindowRequestReadiness::Ready => {}
        }
        if shared.pending.len() >= MAX_PENDING_REQUESTS {
            return Err(NativeWindowRequestError::AtCapacity);
        }
        let next_id = shared
            .next_id
            .checked_add(1)
            .ok_or(NativeWindowRequestError::IdentityExhausted)?;
        let (completion, result) = mpsc::channel();
        shared.next_id = next_id;
        shared.pending.push_back(PendingNativeWindowRequest {
            request,
            completion: Some(completion),
        });

        // Serializes admission and rejection. A failed Winit wake cannot
        // produce a spurious accepted ticket or strand earlier accepted work.
        if !shared.wake.as_ref().is_some_and(|wake| wake()) {
            shared.readiness = NativeWindowRequestReadiness::Closed;
            shared.wake = None;
            shared.pending.clear();
            return Err(NativeWindowRequestError::Closed);
        }
        Ok(NativeWindowRequestTicket {
            id: NativeWindowRequestId(next_id),
            result,
        })
    }
}

/// The single owning side of request ingress. The selected Native App World
/// owns this resource, which closes tickets when it is retired.
#[derive(Default)]
pub(crate) struct NativeWindowRequestOwner {
    shared: Arc<Mutex<Shared>>,
}

impl runen_ecs::Resource for NativeWindowRequestOwner {}

impl NativeWindowRequestOwner {
    pub(crate) fn handle(&self) -> NativeWindowRequests {
        NativeWindowRequests {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Attach a Winit wake after event-loop construction; readiness stays
    /// NotReady until the runner can actually receive and realize requests.
    pub(crate) fn attach_wake(
        &self,
        wake: impl Fn() -> bool + Send + Sync + 'static,
    ) -> Result<(), NativeWindowRequestError> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| NativeWindowRequestError::Poisoned)?;
        if shared.readiness == NativeWindowRequestReadiness::Closed {
            return Err(NativeWindowRequestError::Closed);
        }
        if shared.readiness == NativeWindowRequestReadiness::Ready || shared.wake.is_some() {
            return Err(NativeWindowRequestError::SealedComposition);
        }
        shared.wake = Some(Arc::new(wake));
        Ok(())
    }

    /// Called when the selected Winit owner can actually receive and drain
    /// requests. The zero-initial-window path admits ingress before Startup so
    /// Startup may author a request; the retained primary-first path becomes
    /// ready after its initial window and one-shot Startup are realized.
    pub(crate) fn mark_ready(&self) -> Result<(), NativeWindowRequestError> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| NativeWindowRequestError::Poisoned)?;
        match shared.readiness {
            NativeWindowRequestReadiness::Closed => Err(NativeWindowRequestError::Closed),
            NativeWindowRequestReadiness::Ready => Ok(()),
            NativeWindowRequestReadiness::NotReady => {
                if shared.wake.is_none() {
                    return Err(NativeWindowRequestError::NotReady);
                }
                shared.readiness = NativeWindowRequestReadiness::Ready;
                Ok(())
            }
        }
    }

    pub(crate) fn take_pending(
        &self,
    ) -> Result<Vec<PendingNativeWindowRequest>, NativeWindowRequestError> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| NativeWindowRequestError::Poisoned)?;
        if shared.readiness != NativeWindowRequestReadiness::Ready {
            return Ok(Vec::new());
        }
        Ok(shared.pending.drain(..).collect())
    }

    pub(crate) fn close(&self) {
        // A poisoned mutex must not strand accepted work on owner shutdown.
        let mut shared = self
            .shared
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        shared.readiness = NativeWindowRequestReadiness::Closed;
        shared.wake = None;
        shared.pending.clear();
    }
}

impl Drop for NativeWindowRequestOwner {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use std::sync::{
        Barrier,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    use std::thread;

    fn sample() -> NativeWindowRequest {
        NativeWindowRequest::new("secondary", (640, 480))
    }

    #[test]
    fn native_handle_rejects_headless_and_is_not_ready_before_run() {
        let mut headless = App::headless();
        assert!(matches!(
            headless.native_window_requests(),
            Err(NativeWindowRequestError::UnsupportedHost)
        ));
        let mut app = App::new();
        let handle = app.native_window_requests().unwrap();
        assert_eq!(
            handle.readiness().unwrap(),
            NativeWindowRequestReadiness::NotReady
        );
        assert!(matches!(
            handle.request(sample()),
            Err(NativeWindowRequestError::NotReady)
        ));
        drop(app);
        assert_eq!(
            handle.readiness().unwrap(),
            NativeWindowRequestReadiness::Closed
        );
        assert!(matches!(
            handle.request(sample()),
            Err(NativeWindowRequestError::Closed)
        ));
    }

    #[test]
    fn accepted_ticket_has_one_terminal_result_and_bounded_capacity() {
        let owner = NativeWindowRequestOwner::default();
        let handle = owner.handle();
        let wakes = Arc::new(AtomicUsize::new(0));
        let wake_count = Arc::clone(&wakes);
        owner
            .attach_wake(move || {
                wake_count.fetch_add(1, Ordering::SeqCst);
                true
            })
            .unwrap();
        owner.mark_ready().unwrap();
        let mut tickets = Vec::new();
        for _ in 0..MAX_PENDING_REQUESTS {
            tickets.push(handle.request(sample()).unwrap());
        }
        assert!(matches!(
            handle.request(sample()),
            Err(NativeWindowRequestError::AtCapacity)
        ));
        assert_eq!(wakes.load(Ordering::SeqCst), MAX_PENDING_REQUESTS);
        let mut pending = owner.take_pending().unwrap();
        assert_eq!(pending.len(), MAX_PENDING_REQUESTS);
        pending[0].finish(NativeWindowRequestOutcome::CreationFailed {
            reason: "physical rejection".into(),
        });
        assert_eq!(
            tickets[0].recv_timeout(Duration::from_secs(1)).unwrap(),
            NativeWindowRequestOutcome::CreationFailed {
                reason: "physical rejection".into()
            }
        );
        assert_eq!(tickets[0].try_recv(), Err(TryRecvError::Disconnected));
        drop(pending);
        for ticket in tickets.iter().skip(1) {
            assert_eq!(
                ticket.recv_timeout(Duration::from_secs(1)).unwrap(),
                NativeWindowRequestOutcome::Cancelled
            );
            assert_eq!(ticket.try_recv(), Err(TryRecvError::Disconnected));
        }
        owner.close();
        assert_eq!(
            handle.readiness().unwrap(),
            NativeWindowRequestReadiness::Closed
        );
    }

    #[test]
    fn delivery_failure_closes_ingress_and_cancels_previously_accepted_work() {
        let owner = NativeWindowRequestOwner::default();
        let handle = owner.handle();
        let allow = Arc::new(AtomicBool::new(true));
        let allow_wake = Arc::clone(&allow);
        owner
            .attach_wake(move || allow_wake.load(Ordering::SeqCst))
            .unwrap();
        owner.mark_ready().unwrap();
        let first = handle.request(sample()).unwrap();
        allow.store(false, Ordering::SeqCst);
        assert!(matches!(
            handle.request(sample()),
            Err(NativeWindowRequestError::Closed)
        ));
        assert_eq!(
            first.recv_timeout(Duration::from_secs(1)).unwrap(),
            NativeWindowRequestOutcome::Cancelled
        );
        assert_eq!(first.try_recv(), Err(TryRecvError::Disconnected));
    }

    #[test]
    fn drained_inflight_ticket_settles_after_owner_close() {
        let owner = NativeWindowRequestOwner::default();
        let handle = owner.handle();
        owner.attach_wake(|| true).unwrap();
        owner.mark_ready().unwrap();
        let ticket = handle.request(sample()).unwrap();
        let pending = owner.take_pending().unwrap();
        assert_eq!(pending.len(), 1);

        // The Native owner may start stopping while its invoker still owns
        // a drained batch. A live ticket is not lost when queue state closes.
        owner.close();
        assert_eq!(
            handle.readiness().unwrap(),
            NativeWindowRequestReadiness::Closed
        );
        assert!(matches!(
            handle.request(sample()),
            Err(NativeWindowRequestError::Closed)
        ));
        assert_eq!(ticket.try_recv(), Err(TryRecvError::Empty));
        drop(pending);
        assert_eq!(
            ticket.recv_timeout(Duration::from_secs(1)).unwrap(),
            NativeWindowRequestOutcome::Cancelled
        );
        assert_eq!(ticket.try_recv(), Err(TryRecvError::Disconnected));
    }

    #[test]
    fn dropped_ticket_does_not_retract_accepted_native_work() {
        let owner = NativeWindowRequestOwner::default();
        let handle = owner.handle();
        owner.attach_wake(|| true).unwrap();
        owner.mark_ready().unwrap();

        let ticket = handle.request(sample()).unwrap();
        drop(ticket);
        let mut pending = owner.take_pending().unwrap();
        assert_eq!(pending.len(), 1);
        pending[0].finish(NativeWindowRequestOutcome::CreationFailed {
            reason: "injected Native owner failure".into(),
        });
        drop(pending);

        // Delivery to an absent receiver is nonfatal; Winit retains the
        // actual admission/realization authority and remains operable.
        assert_eq!(
            handle.readiness().unwrap(),
            NativeWindowRequestReadiness::Ready
        );
        let next = handle.request(sample()).unwrap();
        assert_eq!(next.id().raw(), 2);
        drop(owner);
        assert_eq!(
            next.recv_timeout(Duration::from_secs(1)).unwrap(),
            NativeWindowRequestOutcome::Cancelled
        );
    }

    #[test]
    fn invalid_extent_does_not_consume_request_identity() {
        let owner = NativeWindowRequestOwner::default();
        let handle = owner.handle();
        owner.attach_wake(|| true).unwrap();
        owner.mark_ready().unwrap();
        assert!(matches!(
            handle.request(NativeWindowRequest::new("zero", (0, 200))),
            Err(NativeWindowRequestError::InvalidExtent)
        ));
        assert!(matches!(
            handle.request(NativeWindowRequest::new("zero", (200, 0))),
            Err(NativeWindowRequestError::InvalidExtent)
        ));
        let first = handle.request(sample()).unwrap();
        assert_eq!(first.id().raw(), 1);
        drop(owner);
        assert_eq!(
            first.recv_timeout(Duration::from_secs(1)).unwrap(),
            NativeWindowRequestOutcome::Cancelled
        );
    }

    #[test]
    fn racing_producers_cannot_obtain_a_ticket_after_owner_closure() {
        let owner = NativeWindowRequestOwner::default();
        let handle = owner.handle();
        owner.attach_wake(|| true).unwrap();
        owner.mark_ready().unwrap();
        let barrier = Arc::new(Barrier::new(3));
        let joins: Vec<_> = (0..2)
            .map(|_| {
                let h = handle.clone();
                let b = Arc::clone(&barrier);
                thread::spawn(move || {
                    b.wait();
                    h.request(sample())
                })
            })
            .collect();
        barrier.wait();
        owner.close();
        for join in joins {
            match join.join().unwrap() {
                Ok(ticket) => {
                    assert_eq!(
                        ticket.recv_timeout(Duration::from_secs(1)).unwrap(),
                        NativeWindowRequestOutcome::Cancelled
                    );
                    assert_eq!(ticket.try_recv(), Err(TryRecvError::Disconnected));
                }
                Err(NativeWindowRequestError::Closed) => {}
                other => panic!("unexpected race outcome: {other:?}"),
            }
        }
        assert!(matches!(
            handle.request(sample()),
            Err(NativeWindowRequestError::Closed)
        ));
    }
}
