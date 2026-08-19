use super::WgpuContextState;
use super::health::{WgpuDeviceFaultClass, WgpuDeviceFaultEvidence};
use crate::plugins::gpu::{
    GpuCapabilityAdmission, GpuContext, GpuContextAffinity, GpuExecutionPolicy, GpuExecutionStats,
    GpuPreparedSubmission, GpuPreparedSubmissionRejected, GpuPreparedWorkGraph, GpuReadback,
    GpuReadbackBytes, GpuReadbackId, GpuReadbackStatus, GpuRealizedBuffer, GpuSubmission,
    GpuSubmissionFailure, GpuSubmissionFailureKind, GpuSubmissionId, GpuSubmissionPreparationError,
    GpuSubmissionPreparationErrorKind, GpuSubmissionRejectionKind, GpuSubmissionRejectionReason,
    GpuSubmissionStatus, GpuTransferRegion, GpuWorkOperation, GpuWorkResourceId, PreparedGpuData,
    TransferData,
};
use core::num::NonZeroU64;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, MapMode, PollType};

#[derive(Debug)]
pub(crate) struct WgpuExecutionState {
    affinity: GpuContextAffinity,
    policy: GpuExecutionPolicy,
    next_prepared: AtomicU64,
    next_submission: AtomicU64,
    inner: Mutex<ExecutionInner>,
    events: Mutex<VecDeque<ExecutionEvent>>,
}

#[derive(Debug, Default)]
struct ExecutionInner {
    prepared: BTreeMap<NonZeroU64, Option<PreparedBufferPlan>>,
    in_flight: BTreeMap<GpuSubmissionId, InFlightSubmission>,
    upload_bytes_in_flight: u64,
    readback_bytes_in_flight: u64,
    pending_readbacks: usize,
}

#[derive(Debug, Clone)]
struct PreparedBufferPlan {
    operations: Vec<PreparedBufferOperation>,
    upload_bytes: u64,
    readback_bytes: u64,
    readback_ids: Vec<GpuReadbackId>,
}

#[derive(Debug, Clone)]
enum PreparedBufferOperation {
    Upload {
        destination: GpuRealizedBuffer,
        offset: u64,
        payload: PreparedGpuData<TransferData>,
    },
    Copy {
        source: GpuRealizedBuffer,
        source_offset: u64,
        destination: GpuRealizedBuffer,
        destination_offset: u64,
        size: u64,
    },
    Readback {
        id: GpuReadbackId,
        source: GpuRealizedBuffer,
        source_offset: u64,
        size: u64,
    },
}

#[derive(Debug)]
struct InFlightSubmission {
    status: Arc<Mutex<GpuSubmissionStatus>>,
    readbacks: BTreeMap<GpuReadbackId, InFlightReadback>,
    plan: Option<PreparedBufferPlan>,
    upload_staging: Vec<Arc<Buffer>>,
    upload_bytes: u64,
    readback_bytes: u64,
    submission_terminal: bool,
}

#[derive(Debug)]
struct InFlightReadback {
    status: Arc<Mutex<GpuReadbackStatus>>,
    staging: Option<Arc<Buffer>>,
    size: u64,
    terminal: bool,
}

#[derive(Debug)]
enum ExecutionEvent {
    SubmissionCompleted(GpuSubmissionId),
    ReadbackMapped {
        submission: GpuSubmissionId,
        readback: GpuReadbackId,
        result: Result<(), String>,
    },
}

struct PreparationReservation {
    execution: Arc<WgpuExecutionState>,
    ticket: NonZeroU64,
    committed: bool,
}

impl PreparationReservation {
    fn commit(
        mut self,
        plan: PreparedBufferPlan,
    ) -> Result<NonZeroU64, GpuSubmissionPreparationError> {
        self.execution.commit_prepared(self.ticket, plan)?;
        self.committed = true;
        Ok(self.ticket)
    }
}

impl Drop for PreparationReservation {
    fn drop(&mut self) {
        if !self.committed {
            self.execution.release_prepared(self.ticket);
        }
    }
}

struct AcceptedPlan {
    id: GpuSubmissionId,
    plan: PreparedBufferPlan,
    status: Arc<Mutex<GpuSubmissionStatus>>,
    readbacks: Vec<GpuReadback>,
}

struct EncodedSubmission {
    upload_staging: Vec<Arc<Buffer>>,
    readback_staging: Vec<(GpuReadbackId, Arc<Buffer>)>,
}

impl WgpuExecutionState {
    pub(crate) fn new(affinity: GpuContextAffinity, policy: GpuExecutionPolicy) -> Self {
        Self {
            affinity,
            policy,
            next_prepared: AtomicU64::new(1),
            next_submission: AtomicU64::new(1),
            inner: Mutex::new(ExecutionInner::default()),
            events: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) const fn policy(&self) -> GpuExecutionPolicy {
        self.policy
    }

    pub(crate) fn stats(&self) -> GpuExecutionStats {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        GpuExecutionStats::new(
            inner.prepared.len(),
            inner
                .in_flight
                .values()
                .filter(|record| !record.submission_terminal)
                .count(),
            inner.upload_bytes_in_flight,
            inner.readback_bytes_in_flight,
            inner.pending_readbacks,
        )
    }

    fn reserve_prepared(
        self: &Arc<Self>,
    ) -> Result<PreparationReservation, GpuSubmissionPreparationError> {
        let ticket = allocate_nonzero(&self.next_prepared).ok_or_else(|| {
            GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::IdentityExhausted,
                "prepared-submission identity space is exhausted",
            )
        })?;
        let mut inner = self.inner.lock().map_err(|_| {
            GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::ContextOrDeviceUnavailableOrLost,
                "execution preparation authority is unavailable",
            )
        })?;
        if inner.prepared.len() >= self.policy.max_prepared_submissions().get() {
            return Err(GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::PreparedCapacityExceeded,
                format!(
                    "prepared submissions: {}/{}",
                    inner.prepared.len(),
                    self.policy.max_prepared_submissions().get()
                ),
            ));
        }
        inner.prepared.insert(ticket, None);
        drop(inner);
        Ok(PreparationReservation {
            execution: Arc::clone(self),
            ticket,
            committed: false,
        })
    }

    fn commit_prepared(
        &self,
        ticket: NonZeroU64,
        plan: PreparedBufferPlan,
    ) -> Result<(), GpuSubmissionPreparationError> {
        let mut inner = self.inner.lock().map_err(|_| {
            GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::ContextOrDeviceUnavailableOrLost,
                "execution preparation authority is unavailable",
            )
        })?;
        let Some(slot) = inner.prepared.get_mut(&ticket) else {
            return Err(GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::InternalInvariant,
                "prepared reservation disappeared before publication",
            ));
        };
        if slot.is_some() {
            return Err(GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::InternalInvariant,
                "prepared reservation was published more than once",
            ));
        }
        *slot = Some(plan);
        Ok(())
    }

    pub(crate) fn release_prepared(&self, ticket: NonZeroU64) {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .prepared
            .remove(&ticket);
    }

    fn accept_prepared(
        &self,
        prepared: &GpuPreparedSubmission,
    ) -> Result<AcceptedPlan, GpuSubmissionRejectionReason> {
        if prepared.affinity.context() != self.affinity.context() {
            return Err(GpuSubmissionRejectionReason::new(
                GpuSubmissionRejectionKind::ForeignContext,
                "prepared submission belongs to another GPU context",
            ));
        }
        if prepared.affinity.generation() != self.affinity.generation() {
            return Err(GpuSubmissionRejectionReason::new(
                GpuSubmissionRejectionKind::StaleDeviceGeneration,
                "prepared submission belongs to a stale device generation",
            ));
        }

        let mut inner = self.inner.lock().map_err(|_| {
            GpuSubmissionRejectionReason::new(
                GpuSubmissionRejectionKind::ContextOrDeviceUnavailableOrLost,
                "execution acceptance authority is unavailable",
            )
        })?;
        let Some(Some(plan)) = inner.prepared.get(&prepared.ticket) else {
            return Err(GpuSubmissionRejectionReason::new(
                GpuSubmissionRejectionKind::PreparedRecordUnavailable,
                "prepared submission is absent or was already consumed",
            ));
        };
        if inner
            .in_flight
            .values()
            .filter(|record| !record.submission_terminal)
            .count()
            >= self.policy.max_in_flight_submissions().get()
        {
            return Err(GpuSubmissionRejectionReason::new(
                GpuSubmissionRejectionKind::InFlightCapacityExceeded,
                "in-flight submission capacity is occupied",
            ));
        }
        let next_upload = inner
            .upload_bytes_in_flight
            .checked_add(plan.upload_bytes)
            .filter(|value| *value <= self.policy.max_upload_bytes_in_flight())
            .ok_or_else(|| {
                GpuSubmissionRejectionReason::new(
                    GpuSubmissionRejectionKind::UploadBytesInFlightExceeded,
                    "upload staging demand exceeds remaining in-flight capacity",
                )
            })?;
        let next_readback = inner
            .readback_bytes_in_flight
            .checked_add(plan.readback_bytes)
            .filter(|value| *value <= self.policy.max_readback_bytes_in_flight())
            .ok_or_else(|| {
                GpuSubmissionRejectionReason::new(
                    GpuSubmissionRejectionKind::ReadbackBytesInFlightExceeded,
                    "readback staging demand exceeds remaining in-flight capacity",
                )
            })?;
        let next_pending = inner
            .pending_readbacks
            .checked_add(plan.readback_ids.len())
            .filter(|value| *value <= self.policy.max_pending_readbacks())
            .ok_or_else(|| {
                GpuSubmissionRejectionReason::new(
                    GpuSubmissionRejectionKind::PendingReadbacksExceeded,
                    "pending readback count exceeds remaining capacity",
                )
            })?;

        let Some(plan) = inner.prepared.remove(&prepared.ticket).flatten() else {
            return Err(GpuSubmissionRejectionReason::new(
                GpuSubmissionRejectionKind::PreparedRecordUnavailable,
                "prepared submission disappeared before acceptance",
            ));
        };
        let Some(raw_id) = allocate_nonzero(&self.next_submission) else {
            inner.prepared.insert(prepared.ticket, Some(plan));
            return Err(GpuSubmissionRejectionReason::new(
                GpuSubmissionRejectionKind::IdentityExhausted,
                "submission identity space is exhausted",
            ));
        };
        let id = GpuSubmissionId::from_nonzero(raw_id);
        let status = Arc::new(Mutex::new(GpuSubmissionStatus::Accepted));
        let mut readbacks = BTreeMap::new();
        let public_readbacks = plan
            .readback_ids
            .iter()
            .copied()
            .map(|readback_id| {
                let readback_status = Arc::new(Mutex::new(GpuReadbackStatus::Pending));
                readbacks.insert(
                    readback_id,
                    InFlightReadback {
                        status: Arc::clone(&readback_status),
                        staging: None,
                        size: readback_size(&plan, readback_id),
                        terminal: false,
                    },
                );
                GpuReadback::new(readback_id, readback_status)
            })
            .collect::<Vec<_>>();

        inner.upload_bytes_in_flight = next_upload;
        inner.readback_bytes_in_flight = next_readback;
        inner.pending_readbacks = next_pending;
        inner.in_flight.insert(
            id,
            InFlightSubmission {
                status: Arc::clone(&status),
                readbacks,
                plan: Some(plan.clone()),
                upload_staging: Vec::new(),
                upload_bytes: plan.upload_bytes,
                readback_bytes: plan.readback_bytes,
                submission_terminal: false,
            },
        );

        Ok(AcceptedPlan {
            id,
            plan,
            status,
            readbacks: public_readbacks,
        })
    }

    fn attach_staging(&self, id: GpuSubmissionId, encoded: &EncodedSubmission) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = inner.in_flight.get_mut(&id) else {
            return;
        };
        record.upload_staging = encoded.upload_staging.clone();
        for (readback_id, staging) in &encoded.readback_staging {
            if let Some(readback) = record.readbacks.get_mut(readback_id) {
                readback.staging = Some(Arc::clone(staging));
            }
        }
    }

    fn push_event(&self, event: ExecutionEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push_back(event);
    }

    fn drain_events(&self) {
        loop {
            let event = self
                .events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pop_front();
            let Some(event) = event else {
                break;
            };
            match event {
                ExecutionEvent::SubmissionCompleted(id) => self.complete_submission(id),
                ExecutionEvent::ReadbackMapped {
                    submission,
                    readback,
                    result,
                } => self.complete_readback_mapping(submission, readback, result),
            }
        }
    }

    fn complete_submission(&self, id: GpuSubmissionId) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = inner.in_flight.get_mut(&id) else {
            return;
        };
        if record.submission_terminal {
            return;
        }
        let mut status = record
            .status
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(*status, GpuSubmissionStatus::Accepted) {
            *status = GpuSubmissionStatus::Completed;
        }
        drop(status);
        record.submission_terminal = true;
        record.plan = None;
        record.upload_staging.clear();
        inner.upload_bytes_in_flight = inner
            .upload_bytes_in_flight
            .saturating_sub(record.upload_bytes);
        cleanup_submission_if_terminal(&mut inner, id);
    }

    fn complete_readback_mapping(
        &self,
        submission: GpuSubmissionId,
        readback_id: GpuReadbackId,
        result: Result<(), String>,
    ) {
        let staging = {
            let inner = self
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inner
                .in_flight
                .get(&submission)
                .and_then(|record| record.readbacks.get(&readback_id))
                .and_then(|readback| readback.staging.as_ref())
                .cloned()
        };

        let materialized = match (result, staging) {
            (Ok(()), Some(staging)) => {
                let view = staging.slice(..).get_mapped_range();
                let bytes = view.to_vec();
                drop(view);
                staging.unmap();
                Ok(GpuReadbackBytes::from_vec(bytes))
            }
            (Err(detail), _) => Err(GpuSubmissionFailure::new(
                GpuSubmissionFailureKind::ReadbackMapping,
                detail,
            )),
            (Ok(()), None) => Err(GpuSubmissionFailure::new(
                GpuSubmissionFailureKind::InternalInvariant,
                "mapped readback staging record is absent",
            )),
        };

        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = inner.in_flight.get_mut(&submission) else {
            return;
        };
        let Some(readback) = record.readbacks.get_mut(&readback_id) else {
            return;
        };
        if readback.terminal {
            return;
        }
        let mut status = readback
            .status
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *status = match materialized {
            Ok(bytes) => GpuReadbackStatus::Ready(bytes),
            Err(failure) => GpuReadbackStatus::Failed(failure),
        };
        drop(status);
        readback.terminal = true;
        readback.staging = None;
        inner.readback_bytes_in_flight =
            inner.readback_bytes_in_flight.saturating_sub(readback.size);
        inner.pending_readbacks = inner.pending_readbacks.saturating_sub(1);
        cleanup_submission_if_terminal(&mut inner, submission);
    }

    fn fail_submission(&self, id: GpuSubmissionId, failure: GpuSubmissionFailure) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = inner.in_flight.get_mut(&id) else {
            return;
        };
        if !record.submission_terminal {
            let mut status = record
                .status
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if matches!(*status, GpuSubmissionStatus::Accepted) {
                *status = GpuSubmissionStatus::Failed(failure.clone());
            }
            drop(status);
            record.submission_terminal = true;
            record.plan = None;
            record.upload_staging.clear();
            inner.upload_bytes_in_flight = inner
                .upload_bytes_in_flight
                .saturating_sub(record.upload_bytes);
        }
        for readback in record.readbacks.values_mut() {
            if readback.terminal {
                continue;
            }
            *readback
                .status
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                GpuReadbackStatus::Failed(failure.clone());
            readback.terminal = true;
            readback.staging = None;
            inner.readback_bytes_in_flight =
                inner.readback_bytes_in_flight.saturating_sub(readback.size);
            inner.pending_readbacks = inner.pending_readbacks.saturating_sub(1);
        }
        cleanup_submission_if_terminal(&mut inner, id);
    }

    fn fail_active_for_fault(&self, fault: WgpuDeviceFaultEvidence) {
        let failure = failure_from_device_fault(&fault);
        let ids = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .in_flight
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for id in ids {
            self.fail_submission(id, failure.clone());
        }
    }
}

impl Drop for WgpuExecutionState {
    fn drop(&mut self) {
        let failure = GpuSubmissionFailure::new(
            GpuSubmissionFailureKind::ContextDropped,
            "GPU context owner was dropped before execution reached a terminal observation",
        );
        let inner = self
            .inner
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for record in inner.in_flight.values_mut() {
            let mut submission = record
                .status
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if matches!(*submission, GpuSubmissionStatus::Accepted) {
                *submission = GpuSubmissionStatus::Failed(failure.clone());
            }
            drop(submission);
            for readback in record.readbacks.values_mut() {
                let mut status = readback
                    .status
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if matches!(*status, GpuReadbackStatus::Pending) {
                    *status = GpuReadbackStatus::Failed(failure.clone());
                }
            }
        }
    }
}

impl GpuContext {
    pub fn execution_policy(&self) -> GpuExecutionPolicy {
        self.backend.execution.policy()
    }

    pub fn execution_stats(&self) -> GpuExecutionStats {
        self.backend.execution.stats()
    }

    pub async fn prepare_submission(
        &self,
        graph: GpuPreparedWorkGraph,
    ) -> Result<GpuPreparedSubmission, GpuSubmissionPreparationError> {
        if let Some(fault) = self.backend.health.terminal_fault() {
            return Err(GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::ContextOrDeviceUnavailableOrLost,
                fault.detail,
            ));
        }
        GpuCapabilityAdmission::evaluate(
            graph.label().as_str(),
            graph.requirements(),
            self.adapter_facts().supported(),
            self.device_facts().enabled_features(),
        )
        .map_err(|error| {
            GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::CapabilityNotAdmitted,
                error.to_string(),
            )
        })?;

        let reservation = self.backend.execution.reserve_prepared()?;
        let plan = prepare_buffer_plan(self, &graph)?;
        validate_plan_policy(
            plan.upload_bytes,
            plan.readback_bytes,
            plan.readback_ids.len(),
            self.execution_policy(),
        )?;
        let planned_readbacks = plan.readback_ids.clone();
        let ticket = reservation.commit(plan)?;
        Ok(GpuPreparedSubmission::new(
            ticket,
            self.affinity(),
            Arc::downgrade(&self.backend.execution),
            planned_readbacks,
        ))
    }

    pub fn submit_prepared(
        &self,
        mut prepared: GpuPreparedSubmission,
    ) -> Result<GpuSubmission, GpuPreparedSubmissionRejected> {
        if !prepared
            .execution
            .ptr_eq(&Arc::downgrade(&self.backend.execution))
        {
            return Err(GpuPreparedSubmissionRejected::new(
                prepared,
                GpuSubmissionRejectionReason::new(
                    GpuSubmissionRejectionKind::ForeignContext,
                    "prepared submission belongs to another execution owner",
                ),
            ));
        }
        if let Some(fault) = self.backend.health.terminal_fault() {
            return Err(GpuPreparedSubmissionRejected::new(
                prepared,
                GpuSubmissionRejectionReason::new(
                    GpuSubmissionRejectionKind::ContextOrDeviceUnavailableOrLost,
                    fault.detail,
                ),
            ));
        }

        let accepted = match self.backend.execution.accept_prepared(&prepared) {
            Ok(accepted) => accepted,
            Err(reason) => return Err(GpuPreparedSubmissionRejected::new(prepared, reason)),
        };
        prepared.disarm();
        let submission = GpuSubmission::new(
            accepted.id,
            self.affinity(),
            Arc::clone(&accepted.status),
            accepted.readbacks,
        );

        match encode_and_submit_buffers(&self.backend, accepted.id, &accepted.plan) {
            Ok(encoded) => {
                self.backend.execution.attach_staging(accepted.id, &encoded);
                register_callbacks(
                    &self.backend.execution,
                    &self.backend,
                    accepted.id,
                    &encoded,
                );
            }
            Err(failure) => self.backend.execution.fail_submission(accepted.id, failure),
        }
        Ok(submission)
    }

    pub fn progress(&self) -> GpuExecutionStats {
        if let Err(error) = self.backend.device.poll(PollType::Poll) {
            self.backend
                .health
                .mark_scoped_internal(format!("nonblocking WGPU progress poll failed: {error}"));
        }
        self.backend.execution.drain_events();
        if let Some(fault) = self.backend.health.terminal_fault() {
            self.backend.execution.fail_active_for_fault(fault);
        }
        self.backend.execution.stats()
    }
}

fn prepare_buffer_plan(
    context: &GpuContext,
    graph: &GpuPreparedWorkGraph,
) -> Result<PreparedBufferPlan, GpuSubmissionPreparationError> {
    let alignment = context
        .device_facts()
        .device_limits()
        .alignments()
        .copy_buffer_offset
        .ok_or_else(|| {
            GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::UnsupportedOperation,
                "created device did not publish a buffer-copy alignment fact",
            )
        })?;
    let mut cache = BTreeMap::<GpuWorkResourceId, GpuRealizedBuffer>::new();
    let mut operations = Vec::with_capacity(graph.topological_order().len());
    let mut upload_bytes = 0_u64;
    let mut readback_bytes = 0_u64;
    let mut readback_ids = Vec::new();
    let mut seen_readbacks = BTreeSet::new();

    for id in graph.topological_order() {
        let prepared = graph
            .nodes()
            .iter()
            .find(|prepared| prepared.id() == *id)
            .ok_or_else(|| {
                GpuSubmissionPreparationError::new(
                    GpuSubmissionPreparationErrorKind::InternalInvariant,
                    "prepared topological order references an absent work node",
                )
            })?;
        match prepared.node().operation() {
            GpuWorkOperation::Upload(upload) => {
                let GpuTransferRegion::Buffer(destination) = upload.destination() else {
                    return unsupported(
                        "texture Upload remains outside the first G5B buffer lifecycle slice",
                    );
                };
                validate_copy_range(
                    destination.range().offset(),
                    destination.range().size(),
                    alignment,
                )?;
                let realized = realized_buffer(context, &mut cache, destination.buffer())?;
                upload_bytes = upload_bytes
                    .checked_add(destination.range().size())
                    .ok_or_else(|| {
                        GpuSubmissionPreparationError::new(
                            GpuSubmissionPreparationErrorKind::UploadDemandExceedsPolicy,
                            "upload byte demand overflowed the normalized u64 domain",
                        )
                    })?;
                operations.push(PreparedBufferOperation::Upload {
                    destination: realized,
                    offset: destination.range().offset(),
                    payload: upload.payload().clone(),
                });
            }
            GpuWorkOperation::Copy(crate::plugins::gpu::GpuCopyOperation::BufferToBuffer {
                source,
                destination,
            }) => {
                validate_copy_range(source.range().offset(), source.range().size(), alignment)?;
                validate_copy_range(
                    destination.range().offset(),
                    destination.range().size(),
                    alignment,
                )?;
                operations.push(PreparedBufferOperation::Copy {
                    source: realized_buffer(context, &mut cache, source.buffer())?,
                    source_offset: source.range().offset(),
                    destination: realized_buffer(context, &mut cache, destination.buffer())?,
                    destination_offset: destination.range().offset(),
                    size: source.range().size(),
                });
            }
            GpuWorkOperation::Readback(readback) => {
                let GpuTransferRegion::Buffer(source) = readback.source() else {
                    return unsupported(
                        "texture Readback remains outside the first G5B buffer lifecycle slice",
                    );
                };
                validate_copy_range(source.range().offset(), source.range().size(), alignment)?;
                if !seen_readbacks.insert(readback.id()) {
                    return Err(GpuSubmissionPreparationError::new(
                        GpuSubmissionPreparationErrorKind::InternalInvariant,
                        format!(
                            "duplicate readback identity in one prepared graph: {:?}",
                            readback.id()
                        ),
                    ));
                }
                let size = source.range().size();
                readback_bytes = readback_bytes.checked_add(size).ok_or_else(|| {
                    GpuSubmissionPreparationError::new(
                        GpuSubmissionPreparationErrorKind::ReadbackDemandExceedsPolicy,
                        "readback byte demand overflowed the normalized u64 domain",
                    )
                })?;
                readback_ids.push(readback.id());
                operations.push(PreparedBufferOperation::Readback {
                    id: readback.id(),
                    source: realized_buffer(context, &mut cache, source.buffer())?,
                    source_offset: source.range().offset(),
                    size,
                });
            }
            GpuWorkOperation::Copy(_) => {
                return unsupported(
                    "texture-involving Copy remains outside the first G5B buffer lifecycle slice",
                );
            }
            _ => {
                return unsupported(
                    "this first G5B lifecycle slice executes only buffer Upload, Copy, and Readback work",
                );
            }
        }
    }

    Ok(PreparedBufferPlan {
        operations,
        upload_bytes,
        readback_bytes,
        readback_ids,
    })
}

fn realized_buffer(
    context: &GpuContext,
    cache: &mut BTreeMap<GpuWorkResourceId, GpuRealizedBuffer>,
    handle: &crate::plugins::gpu::GpuBufferHandle,
) -> Result<GpuRealizedBuffer, GpuSubmissionPreparationError> {
    let identity = handle.diagnostic_identity();
    if let Some(realized) = cache.get(&identity) {
        return Ok(realized.clone());
    }
    let realized = context.realize_buffer(handle).map_err(|error| {
        GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::ResourceRealizationFailed,
            error.to_string(),
        )
    })?;
    cache.insert(identity, realized.clone());
    Ok(realized)
}

fn validate_copy_range(
    offset: u64,
    size: u64,
    alignment: u64,
) -> Result<(), GpuSubmissionPreparationError> {
    if alignment == 0 || !offset.is_multiple_of(alignment) || !size.is_multiple_of(alignment) {
        return Err(GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::UnsupportedOperation,
            format!(
                "buffer transfer range offset={offset} size={size} is not encodable at admitted copy alignment {alignment}"
            ),
        ));
    }
    Ok(())
}

fn validate_plan_policy(
    upload_bytes: u64,
    readback_bytes: u64,
    pending_readbacks: usize,
    policy: GpuExecutionPolicy,
) -> Result<(), GpuSubmissionPreparationError> {
    if upload_bytes > policy.max_upload_bytes_in_flight() {
        return Err(GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::UploadDemandExceedsPolicy,
            format!(
                "submission upload demand {upload_bytes} exceeds policy {}",
                policy.max_upload_bytes_in_flight()
            ),
        ));
    }
    if readback_bytes > policy.max_readback_bytes_in_flight() {
        return Err(GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::ReadbackDemandExceedsPolicy,
            format!(
                "submission readback demand {readback_bytes} exceeds policy {}",
                policy.max_readback_bytes_in_flight()
            ),
        ));
    }
    if pending_readbacks > policy.max_pending_readbacks() {
        return Err(GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::PendingReadbacksExceedPolicy,
            format!(
                "submission readback count {pending_readbacks} exceeds policy {}",
                policy.max_pending_readbacks()
            ),
        ));
    }
    Ok(())
}

fn unsupported<T>(detail: &'static str) -> Result<T, GpuSubmissionPreparationError> {
    Err(GpuSubmissionPreparationError::new(
        GpuSubmissionPreparationErrorKind::UnsupportedOperation,
        detail,
    ))
}

fn encode_and_submit_buffers(
    backend: &WgpuContextState,
    id: GpuSubmissionId,
    plan: &PreparedBufferPlan,
) -> Result<EncodedSubmission, GpuSubmissionFailure> {
    if let Some(fault) = backend.health.terminal_fault() {
        return Err(failure_from_device_fault(&fault));
    }
    let _attribution_gate = backend.error_attribution_gate.acquire();
    let mut encoder = backend
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("RunenGPU G5B buffer submission"),
        });
    let mut upload_staging = Vec::new();
    let mut readback_staging = Vec::new();

    for operation in &plan.operations {
        match operation {
            PreparedBufferOperation::Upload {
                destination,
                offset,
                payload,
            } => {
                let staging = Arc::new(backend.device.create_buffer(&BufferDescriptor {
                    label: Some("RunenGPU upload staging"),
                    size: payload.layout().byte_len(),
                    usage: BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                }));
                {
                    let mut mapped = staging.slice(..).get_mapped_range_mut();
                    mapped.copy_from_slice(payload.as_bytes());
                }
                staging.unmap();
                encoder.copy_buffer_to_buffer(
                    &staging,
                    0,
                    &destination.record.object,
                    *offset,
                    payload.layout().byte_len(),
                );
                upload_staging.push(staging);
            }
            PreparedBufferOperation::Copy {
                source,
                source_offset,
                destination,
                destination_offset,
                size,
            } => encoder.copy_buffer_to_buffer(
                &source.record.object,
                *source_offset,
                &destination.record.object,
                *destination_offset,
                *size,
            ),
            PreparedBufferOperation::Readback {
                id: readback_id,
                source,
                source_offset,
                size,
            } => {
                let staging = Arc::new(backend.device.create_buffer(&BufferDescriptor {
                    label: Some("RunenGPU readback staging"),
                    size: *size,
                    usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }));
                encoder.copy_buffer_to_buffer(
                    &source.record.object,
                    *source_offset,
                    &staging,
                    0,
                    *size,
                );
                readback_staging.push((*readback_id, staging));
            }
        }
    }

    backend.queue.submit([encoder.finish()]);
    if let Some(fault) = backend.health.terminal_fault() {
        return Err(failure_from_device_fault(&fault));
    }
    let _ = id;
    Ok(EncodedSubmission {
        upload_staging,
        readback_staging,
    })
}

fn register_callbacks(
    execution: &Arc<WgpuExecutionState>,
    backend: &WgpuContextState,
    submission: GpuSubmissionId,
    encoded: &EncodedSubmission,
) {
    for (readback, staging) in &encoded.readback_staging {
        let weak = Arc::downgrade(execution);
        let readback = *readback;
        staging.slice(..).map_async(MapMode::Read, move |result| {
            if let Some(execution) = weak.upgrade() {
                execution.push_event(ExecutionEvent::ReadbackMapped {
                    submission,
                    readback,
                    result: result.map_err(|error| error.to_string()),
                });
            }
        });
    }
    let weak = Arc::downgrade(execution);
    backend.queue.on_submitted_work_done(move || {
        if let Some(execution) = weak.upgrade() {
            execution.push_event(ExecutionEvent::SubmissionCompleted(submission));
        }
    });
}

fn cleanup_submission_if_terminal(inner: &mut ExecutionInner, id: GpuSubmissionId) {
    let should_remove = inner.in_flight.get(&id).is_some_and(|record| {
        record.submission_terminal && record.readbacks.values().all(|readback| readback.terminal)
    });
    if should_remove {
        inner.in_flight.remove(&id);
    }
}

fn readback_size(plan: &PreparedBufferPlan, id: GpuReadbackId) -> u64 {
    plan.operations
        .iter()
        .find_map(|operation| match operation {
            PreparedBufferOperation::Readback {
                id: candidate,
                size,
                ..
            } if *candidate == id => Some(*size),
            _ => None,
        })
        .unwrap_or(0)
}

fn allocate_nonzero(counter: &AtomicU64) -> Option<NonZeroU64> {
    let value = counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            (current != 0).then_some(if current == u64::MAX { 0 } else { current + 1 })
        })
        .ok()?;
    NonZeroU64::new(value)
}

fn failure_from_device_fault(fault: &WgpuDeviceFaultEvidence) -> GpuSubmissionFailure {
    let kind = match fault.class {
        WgpuDeviceFaultClass::UnexpectedValidation => GpuSubmissionFailureKind::BackendValidation,
        WgpuDeviceFaultClass::OutOfMemory => GpuSubmissionFailureKind::BackendResourceExhaustion,
        WgpuDeviceFaultClass::InternalOrDeviceLost => {
            GpuSubmissionFailureKind::ContextOrDeviceUnavailableOrLost
        }
    };
    GpuSubmissionFailure::new(kind, fault.detail.clone())
}
