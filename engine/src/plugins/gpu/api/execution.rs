use super::{GpuContextAffinity, GpuReadbackBytes, GpuReadbackId};
use core::fmt;
use core::num::{NonZeroU64, NonZeroUsize};
use std::sync::{Arc, Mutex, Weak};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuExecutionPolicy {
    max_prepared_submissions: NonZeroUsize,
    max_in_flight_submissions: NonZeroUsize,
    max_upload_bytes_in_flight: u64,
    max_readback_bytes_in_flight: u64,
    max_pending_readbacks: usize,
}

impl GpuExecutionPolicy {
    pub const fn new(
        max_prepared_submissions: NonZeroUsize,
        max_in_flight_submissions: NonZeroUsize,
        max_upload_bytes_in_flight: u64,
        max_readback_bytes_in_flight: u64,
        max_pending_readbacks: usize,
    ) -> Self {
        Self {
            max_prepared_submissions,
            max_in_flight_submissions,
            max_upload_bytes_in_flight,
            max_readback_bytes_in_flight,
            max_pending_readbacks,
        }
    }

    pub const fn max_prepared_submissions(self) -> NonZeroUsize {
        self.max_prepared_submissions
    }

    pub const fn max_in_flight_submissions(self) -> NonZeroUsize {
        self.max_in_flight_submissions
    }

    pub const fn max_upload_bytes_in_flight(self) -> u64 {
        self.max_upload_bytes_in_flight
    }

    pub const fn max_readback_bytes_in_flight(self) -> u64 {
        self.max_readback_bytes_in_flight
    }

    pub const fn max_pending_readbacks(self) -> usize {
        self.max_pending_readbacks
    }
}

impl Default for GpuExecutionPolicy {
    fn default() -> Self {
        Self::new(
            NonZeroUsize::new(64).unwrap_or(NonZeroUsize::MIN),
            NonZeroUsize::new(32).unwrap_or(NonZeroUsize::MIN),
            64 * 1024 * 1024,
            64 * 1024 * 1024,
            64,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuExecutionStats {
    prepared_submissions: usize,
    in_flight_submissions: usize,
    upload_bytes_in_flight: u64,
    readback_bytes_in_flight: u64,
    pending_readbacks: usize,
}

impl GpuExecutionStats {
    pub(crate) const fn new(
        prepared_submissions: usize,
        in_flight_submissions: usize,
        upload_bytes_in_flight: u64,
        readback_bytes_in_flight: u64,
        pending_readbacks: usize,
    ) -> Self {
        Self {
            prepared_submissions,
            in_flight_submissions,
            upload_bytes_in_flight,
            readback_bytes_in_flight,
            pending_readbacks,
        }
    }

    pub const fn prepared_submissions(self) -> usize {
        self.prepared_submissions
    }

    pub const fn in_flight_submissions(self) -> usize {
        self.in_flight_submissions
    }

    pub const fn upload_bytes_in_flight(self) -> u64 {
        self.upload_bytes_in_flight
    }

    pub const fn readback_bytes_in_flight(self) -> u64 {
        self.readback_bytes_in_flight
    }

    pub const fn pending_readbacks(self) -> usize {
        self.pending_readbacks
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuSubmissionId(NonZeroU64);

impl GpuSubmissionId {
    pub(crate) const fn from_nonzero(value: NonZeroU64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for GpuSubmissionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuSubmissionFailureKind {
    BackendValidation,
    BackendResourceExhaustion,
    ContextOrDeviceUnavailableOrLost,
    ReadbackMapping,
    ContextDropped,
    InternalInvariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuSubmissionFailure {
    kind: GpuSubmissionFailureKind,
    detail: String,
}

impl GpuSubmissionFailure {
    pub(crate) fn new(kind: GpuSubmissionFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub const fn kind(&self) -> GpuSubmissionFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuSubmissionStatus {
    Accepted,
    Completed,
    Failed(GpuSubmissionFailure),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuReadbackStatus {
    Pending,
    Ready(GpuReadbackBytes),
    Failed(GpuSubmissionFailure),
}

#[derive(Clone)]
pub struct GpuReadback {
    id: GpuReadbackId,
    status: Arc<Mutex<GpuReadbackStatus>>,
}

impl GpuReadback {
    pub(crate) fn new(id: GpuReadbackId, status: Arc<Mutex<GpuReadbackStatus>>) -> Self {
        Self { id, status }
    }

    pub const fn id(&self) -> GpuReadbackId {
        self.id
    }

    pub fn status(&self) -> GpuReadbackStatus {
        self.status
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl fmt::Debug for GpuReadback {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GpuReadback")
            .field("id", &self.id)
            .field("status", &self.status())
            .finish()
    }
}

#[derive(Clone)]
pub struct GpuSubmission {
    id: GpuSubmissionId,
    affinity: GpuContextAffinity,
    status: Arc<Mutex<GpuSubmissionStatus>>,
    readbacks: Arc<[GpuReadback]>,
}

impl GpuSubmission {
    pub(crate) fn new(
        id: GpuSubmissionId,
        affinity: GpuContextAffinity,
        status: Arc<Mutex<GpuSubmissionStatus>>,
        readbacks: Vec<GpuReadback>,
    ) -> Self {
        Self {
            id,
            affinity,
            status,
            readbacks: readbacks.into(),
        }
    }

    pub const fn id(&self) -> GpuSubmissionId {
        self.id
    }

    pub const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub fn status(&self) -> GpuSubmissionStatus {
        self.status
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn readbacks(&self) -> &[GpuReadback] {
        &self.readbacks
    }

    pub fn readback(&self, id: GpuReadbackId) -> Option<&GpuReadback> {
        self.readbacks.iter().find(|readback| readback.id() == id)
    }
}

impl fmt::Debug for GpuSubmission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GpuSubmission")
            .field("id", &self.id)
            .field("affinity", &self.affinity)
            .field("status", &self.status())
            .field("readbacks", &self.readbacks)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuSubmissionPreparationErrorKind {
    CapabilityNotAdmitted,
    PreparedCapacityExceeded,
    UploadDemandExceedsPolicy,
    ReadbackDemandExceedsPolicy,
    PendingReadbacksExceedPolicy,
    UnsupportedOperation,
    ResourceRealizationFailed,
    ContextOrDeviceUnavailableOrLost,
    IdentityExhausted,
    InternalInvariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuSubmissionPreparationError {
    kind: GpuSubmissionPreparationErrorKind,
    detail: String,
}

impl GpuSubmissionPreparationError {
    pub(crate) fn new(kind: GpuSubmissionPreparationErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub const fn kind(&self) -> GpuSubmissionPreparationErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for GpuSubmissionPreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "GPU submission preparation rejected ({:?}): {}",
            self.kind, self.detail
        )
    }
}

impl std::error::Error for GpuSubmissionPreparationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuSubmissionRejectionKind {
    ForeignContext,
    StaleDeviceGeneration,
    PreparedRecordUnavailable,
    InFlightCapacityExceeded,
    UploadBytesInFlightExceeded,
    ReadbackBytesInFlightExceeded,
    PendingReadbacksExceeded,
    ContextOrDeviceUnavailableOrLost,
    IdentityExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuSubmissionRejectionReason {
    kind: GpuSubmissionRejectionKind,
    detail: String,
}

impl GpuSubmissionRejectionReason {
    pub(crate) fn new(kind: GpuSubmissionRejectionKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub const fn kind(&self) -> GpuSubmissionRejectionKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub struct GpuPreparedSubmission {
    pub(crate) ticket: NonZeroU64,
    pub(crate) affinity: GpuContextAffinity,
    pub(crate) execution: Weak<crate::plugins::gpu::backend::WgpuExecutionState>,
    pub(crate) armed: bool,
    planned_readbacks: Arc<[GpuReadbackId]>,
}

impl GpuPreparedSubmission {
    pub(crate) fn new(
        ticket: NonZeroU64,
        affinity: GpuContextAffinity,
        execution: Weak<crate::plugins::gpu::backend::WgpuExecutionState>,
        planned_readbacks: Vec<GpuReadbackId>,
    ) -> Self {
        Self {
            ticket,
            affinity,
            execution,
            armed: true,
            planned_readbacks: planned_readbacks.into(),
        }
    }

    pub const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub fn planned_readbacks(&self) -> &[GpuReadbackId] {
        &self.planned_readbacks
    }

    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }
}

impl fmt::Debug for GpuPreparedSubmission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GpuPreparedSubmission")
            .field("affinity", &self.affinity)
            .field("planned_readbacks", &self.planned_readbacks)
            .finish_non_exhaustive()
    }
}

impl Drop for GpuPreparedSubmission {
    fn drop(&mut self) {
        if self.armed {
            if let Some(execution) = self.execution.upgrade() {
                execution.release_prepared(self.ticket);
            }
        }
    }
}

pub struct GpuPreparedSubmissionRejected {
    prepared: GpuPreparedSubmission,
    reason: GpuSubmissionRejectionReason,
}

impl GpuPreparedSubmissionRejected {
    pub(crate) fn new(
        prepared: GpuPreparedSubmission,
        reason: GpuSubmissionRejectionReason,
    ) -> Self {
        Self { prepared, reason }
    }

    pub fn prepared(&self) -> &GpuPreparedSubmission {
        &self.prepared
    }

    pub fn reason(&self) -> &GpuSubmissionRejectionReason {
        &self.reason
    }

    pub fn into_parts(self) -> (GpuPreparedSubmission, GpuSubmissionRejectionReason) {
        (self.prepared, self.reason)
    }
}

impl fmt::Debug for GpuPreparedSubmissionRejected {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GpuPreparedSubmissionRejected")
            .field("prepared", &self.prepared)
            .field("reason", &self.reason)
            .finish()
    }
}
