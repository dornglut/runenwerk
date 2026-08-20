use super::WgpuContextState;
use super::health::{WgpuDeviceFaultClass, WgpuDeviceFaultEvidence};
use crate::plugins::gpu::{
    GpuCapabilityAdmission, GpuContext, GpuContextAffinity, GpuDataLayout, GpuDispatchSize,
    GpuExecutionPolicy, GpuExecutionStats, GpuPipelineRealizationError,
    GpuPipelineRealizationErrorCategory, GpuPreparedSubmission, GpuPreparedSubmissionRejected,
    GpuPreparedWorkGraph, GpuProgramBindingRealizationError,
    GpuProgramBindingRealizationErrorCategory, GpuReadback, GpuReadbackBytes, GpuReadbackId,
    GpuReadbackStatus, GpuRealizedBindGroup, GpuRealizedBuffer, GpuRealizedComputePipeline,
    GpuResourceProvenance, GpuRuntimeBindingResource, GpuSubmission, GpuSubmissionFailure,
    GpuSubmissionFailureKind, GpuSubmissionId, GpuSubmissionPreparationError,
    GpuSubmissionPreparationErrorKind, GpuSubmissionRejectionKind, GpuSubmissionRejectionReason,
    GpuSubmissionStatus, GpuTransferRegion, GpuValidatedBindGroupBindings, GpuWorkOperation,
    GpuWorkResourceId, PreparedGpuData, TransferData,
};
use core::num::NonZeroU64;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, MapMode,
    PollType,
};

#[derive(Debug)]
pub(crate) struct WgpuExecutionState {
    affinity: GpuContextAffinity,
    policy: GpuExecutionPolicy,
    next_prepared: AtomicU64,
    next_submission: AtomicU64,
    submission_order: Mutex<()>,
    inner: Mutex<ExecutionInner>,
    events: Mutex<VecDeque<ExecutionEvent>>,
}

#[derive(Debug, Default)]
struct ExecutionInner {
    prepared: BTreeMap<NonZeroU64, Option<PreparedExecutionPlan>>,
    in_flight: BTreeMap<GpuSubmissionId, InFlightSubmission>,
    upload_bytes_in_flight: u64,
    readback_bytes_in_flight: u64,
    pending_readbacks: usize,
}

#[derive(Debug, Clone)]
struct PreparedExecutionPlan {
    operations: Vec<PreparedExecutionOperation>,
    upload_bytes: u64,
    readback_bytes: u64,
    readback_ids: Vec<GpuReadbackId>,
}

#[derive(Debug, Clone)]
struct PreparedComputeBindGroup {
    index: u32,
    realization: GpuRealizedBindGroup,
    dynamic_offsets: Vec<u32>,
}

#[derive(Debug, Clone)]
struct BufferReadbackMetadata {
    label: String,
    layout: GpuDataLayout,
    provenance: GpuResourceProvenance,
}

#[derive(Debug, Clone)]
enum PreparedExecutionOperation {
    Upload {
        destination: GpuRealizedBuffer,
        offset: u64,
        payload: PreparedGpuData<TransferData>,
    },
    Compute {
        pipeline: GpuRealizedComputePipeline,
        bind_groups: Vec<PreparedComputeBindGroup>,
        dispatch: GpuDispatchSize,
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
        metadata: BufferReadbackMetadata,
    },
}

#[derive(Debug)]
struct InFlightSubmission {
    status: Arc<Mutex<GpuSubmissionStatus>>,
    readbacks: BTreeMap<GpuReadbackId, InFlightReadback>,
    plan: Option<PreparedExecutionPlan>,
    upload_staging: Vec<Arc<Buffer>>,
    upload_bytes: u64,
    submission_terminal: bool,
}

#[derive(Debug)]
struct InFlightReadback {
    status: Arc<Mutex<GpuReadbackStatus>>,
    staging: Option<Arc<Buffer>>,
    size: u64,
    metadata: BufferReadbackMetadata,
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
        plan: PreparedExecutionPlan,
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
    plan: PreparedExecutionPlan,
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
            submission_order: Mutex::new(()),
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
        plan: PreparedExecutionPlan,
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

        let mut readbacks = BTreeMap::new();
        let mut public_readbacks = Vec::with_capacity(plan.readback_ids.len());
        for operation in &plan.operations {
            let PreparedExecutionOperation::Readback {
                id: readback_id,
                size,
                metadata,
                ..
            } = operation
            else {
                continue;
            };
            let readback_status = Arc::new(Mutex::new(GpuReadbackStatus::Pending));
            readbacks.insert(
                *readback_id,
                InFlightReadback {
                    status: Arc::clone(&readback_status),
                    staging: None,
                    size: *size,
                    metadata: metadata.clone(),
                    terminal: false,
                },
            );
            public_readbacks.push(GpuReadback::new(*readback_id, readback_status));
        }
        if readbacks.len() != plan.readback_ids.len() {
            return Err(GpuSubmissionRejectionReason::new(
                GpuSubmissionRejectionKind::PreparedRecordUnavailable,
                "prepared readback metadata is incomplete",
            ));
        }

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

    fn attach_staging(
        &self,
        id: GpuSubmissionId,
        encoded: &EncodedSubmission,
    ) -> Result<(), GpuSubmissionFailure> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = inner.in_flight.get_mut(&id) else {
            return Err(GpuSubmissionFailure::new(
                GpuSubmissionFailureKind::InternalInvariant,
                "accepted submission disappeared before staging attachment",
            ));
        };
        record.upload_staging = encoded.upload_staging.clone();
        for (readback_id, staging) in &encoded.readback_staging {
            let Some(readback) = record.readbacks.get_mut(readback_id) else {
                return Err(GpuSubmissionFailure::new(
                    GpuSubmissionFailureKind::InternalInvariant,
                    "accepted readback disappeared before staging attachment",
                ));
            };
            readback.staging = Some(Arc::clone(staging));
        }
        Ok(())
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
        let upload_release = {
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
            let release = record.upload_bytes;
            record.upload_bytes = 0;
            release
        };
        inner.upload_bytes_in_flight = inner.upload_bytes_in_flight.saturating_sub(upload_release);
        cleanup_submission_if_terminal(&mut inner, id);
    }

    fn complete_readback_mapping(
        &self,
        submission: GpuSubmissionId,
        readback_id: GpuReadbackId,
        result: Result<(), String>,
    ) {
        let (staging, metadata) = {
            let inner = self
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(readback) = inner
                .in_flight
                .get(&submission)
                .and_then(|record| record.readbacks.get(&readback_id))
            else {
                return;
            };
            (
                readback.staging.as_ref().cloned(),
                readback.metadata.clone(),
            )
        };

        let materialized = match (result, staging) {
            (Ok(()), Some(staging)) => {
                let bytes = match staging.slice(..).get_mapped_range() {
                    Ok(view) => {
                        let bytes = view.to_vec();
                        drop(view);
                        Ok(bytes)
                    }
                    Err(error) => Err(GpuSubmissionFailure::new(
                        GpuSubmissionFailureKind::ReadbackMapping,
                        format!("obtain mapped readback range: {error}"),
                    )),
                };
                staging.unmap();
                bytes.and_then(|bytes| {
                    GpuReadbackBytes::from_normalized_bytes(
                        &metadata.label,
                        bytes,
                        metadata.layout,
                        None,
                        metadata.provenance,
                    )
                    .map_err(|error| {
                        GpuSubmissionFailure::new(
                            GpuSubmissionFailureKind::InternalInvariant,
                            error.to_string(),
                        )
                    })
                })
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
        let release = {
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
            let release = readback.size;
            readback.size = 0;
            release
        };
        inner.readback_bytes_in_flight = inner.readback_bytes_in_flight.saturating_sub(release);
        inner.pending_readbacks = inner.pending_readbacks.saturating_sub(1);
        cleanup_submission_if_terminal(&mut inner, submission);
    }

    fn fail_submission(&self, id: GpuSubmissionId, failure: GpuSubmissionFailure) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (upload_release, readback_release, pending_release) = {
            let Some(record) = inner.in_flight.get_mut(&id) else {
                return;
            };
            let upload_release = if record.submission_terminal {
                0
            } else {
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
                let release = record.upload_bytes;
                record.upload_bytes = 0;
                release
            };

            let mut readback_release = 0_u64;
            let mut pending_release = 0_usize;
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
                readback_release = readback_release.saturating_add(readback.size);
                pending_release = pending_release.saturating_add(1);
                readback.size = 0;
            }
            (upload_release, readback_release, pending_release)
        };

        inner.upload_bytes_in_flight = inner.upload_bytes_in_flight.saturating_sub(upload_release);
        inner.readback_bytes_in_flight = inner
            .readback_bytes_in_flight
            .saturating_sub(readback_release);
        inner.pending_readbacks = inner.pending_readbacks.saturating_sub(pending_release);
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
        let plan = prepare_execution_plan(self, &graph).await?;
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
        let expected_affinity = self.affinity();
        if prepared.affinity.context() != expected_affinity.context() {
            return Err(GpuPreparedSubmissionRejected::new(
                prepared,
                GpuSubmissionRejectionReason::new(
                    GpuSubmissionRejectionKind::ForeignContext,
                    "prepared submission belongs to another GPU context",
                ),
            ));
        }
        if prepared.affinity.generation() != expected_affinity.generation() {
            return Err(GpuPreparedSubmissionRejected::new(
                prepared,
                GpuSubmissionRejectionReason::new(
                    GpuSubmissionRejectionKind::StaleDeviceGeneration,
                    "prepared submission belongs to a stale device generation",
                ),
            ));
        }
        if !prepared
            .execution
            .ptr_eq(&Arc::downgrade(&self.backend.execution))
        {
            return Err(GpuPreparedSubmissionRejected::new(
                prepared,
                GpuSubmissionRejectionReason::new(
                    GpuSubmissionRejectionKind::PreparedRecordUnavailable,
                    "prepared submission belongs to a different execution owner for this context generation",
                ),
            ));
        }

        // Submission IDs define this context owner's execution order. Keep irreversible acceptance
        // and the corresponding physical encode/submit in one owner-local interval so concurrent
        // callers cannot publish IDs in one order and queue the work in another.
        let _submission_order = self
            .backend
            .execution
            .submission_order
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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

        if let Err(failure) = encode_submit_and_register(
            &self.backend,
            &self.backend.execution,
            accepted.id,
            &accepted.plan,
        ) {
            self.backend.execution.fail_submission(accepted.id, failure);
        }
        Ok(submission)
    }

    pub fn progress(&self) -> GpuExecutionStats {
        if let Err(error) = self.backend.device.poll(PollType::Poll) {
            self.backend
                .health
                .mark_scoped_internal(format!("nonblocking WGPU progress poll failed: {error}"));
        }
        if let Some(fault) = self.backend.health.terminal_fault() {
            self.backend.execution.fail_active_for_fault(fault);
        }
        self.backend.execution.drain_events();
        self.backend.execution.stats()
    }
}

async fn prepare_execution_plan(
    context: &GpuContext,
    graph: &GpuPreparedWorkGraph,
) -> Result<PreparedExecutionPlan, GpuSubmissionPreparationError> {
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
                        "texture Upload remains outside the current G5B execution checkpoint",
                    );
                };
                let alignment = copy_alignment(context)?;
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
                operations.push(PreparedExecutionOperation::Upload {
                    destination: realized,
                    offset: destination.range().offset(),
                    payload: upload.payload().clone(),
                });
            }
            GpuWorkOperation::Compute(compute) => {
                operations.push(prepare_compute_operation(context, compute).await?);
            }
            GpuWorkOperation::Copy(crate::plugins::gpu::GpuCopyOperation::BufferToBuffer {
                source,
                destination,
            }) => {
                let alignment = copy_alignment(context)?;
                validate_copy_range(source.range().offset(), source.range().size(), alignment)?;
                validate_copy_range(
                    destination.range().offset(),
                    destination.range().size(),
                    alignment,
                )?;
                operations.push(PreparedExecutionOperation::Copy {
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
                        "texture Readback remains outside the current G5B execution checkpoint",
                    );
                };
                let alignment = copy_alignment(context)?;
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
                let common = source.buffer().descriptor().common();
                let metadata = BufferReadbackMetadata {
                    label: common.label().as_str().to_string(),
                    layout: GpuDataLayout::new(common.label().as_str(), size, 1, size, 1).map_err(
                        |error| {
                            GpuSubmissionPreparationError::new(
                                GpuSubmissionPreparationErrorKind::InternalInvariant,
                                error.to_string(),
                            )
                        },
                    )?,
                    provenance: common.provenance().clone(),
                };
                readback_bytes = readback_bytes.checked_add(size).ok_or_else(|| {
                    GpuSubmissionPreparationError::new(
                        GpuSubmissionPreparationErrorKind::ReadbackDemandExceedsPolicy,
                        "readback byte demand overflowed the normalized u64 domain",
                    )
                })?;
                readback_ids.push(readback.id());
                operations.push(PreparedExecutionOperation::Readback {
                    id: readback.id(),
                    source: realized_buffer(context, &mut cache, source.buffer())?,
                    source_offset: source.range().offset(),
                    size,
                    metadata,
                });
            }
            GpuWorkOperation::Copy(_) => {
                return unsupported(
                    "texture-involving Copy remains outside the current G5B execution checkpoint",
                );
            }
            _ => {
                return unsupported(
                    "the current G5B checkpoint executes buffer Upload/Copy/Readback and direct compute only",
                );
            }
        }
    }

    Ok(PreparedExecutionPlan {
        operations,
        upload_bytes,
        readback_bytes,
        readback_ids,
    })
}

async fn prepare_compute_operation(
    context: &GpuContext,
    compute: &crate::plugins::gpu::GpuComputeOperation,
) -> Result<PreparedExecutionOperation, GpuSubmissionPreparationError> {
    if compute.timestamp_writes().is_some() {
        return unsupported("compute timestamp writes remain outside this G5B checkpoint");
    }
    let Some(dispatch) = compute.dispatch().direct_size() else {
        return unsupported("indirect compute dispatch remains outside this G5B checkpoint");
    };

    let descriptor = compute.pipeline();
    let program = context
        .realize_program(descriptor.program())
        .await
        .map_err(preparation_program_binding_failure)?;
    let pipeline_layout = context
        .realize_pipeline_layout(descriptor.layout())
        .await
        .map_err(preparation_program_binding_failure)?;
    let pipeline = context
        .realize_compute_pipeline(descriptor, &program, &pipeline_layout)
        .await
        .map_err(preparation_pipeline_failure)?;

    let mut bind_groups = Vec::with_capacity(compute.bindings().groups().len());
    for group in compute.bindings().groups() {
        let layout = context
            .realize_bind_group_layout(group.layout())
            .await
            .map_err(preparation_program_binding_failure)?;
        let realization = context
            .realize_bind_group(&layout, group.values().cloned())
            .await
            .map_err(preparation_program_binding_failure)?;
        bind_groups.push(PreparedComputeBindGroup {
            index: group.layout().group(),
            realization,
            dynamic_offsets: checked_dynamic_offsets(group)?,
        });
    }

    Ok(PreparedExecutionOperation::Compute {
        pipeline,
        bind_groups,
        dispatch,
    })
}

fn checked_dynamic_offsets(
    group: &GpuValidatedBindGroupBindings,
) -> Result<Vec<u32>, GpuSubmissionPreparationError> {
    let mut offsets = Vec::new();
    for declaration in group.layout().bindings() {
        if !declaration.kind().uses_dynamic_offset() {
            continue;
        }
        let value = group.value(declaration.key().binding()).ok_or_else(|| {
            GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::InternalInvariant,
                format!(
                    "validated dynamic binding {} disappeared before execution preparation",
                    declaration.key()
                ),
            )
        })?;
        for resource in value.resources() {
            let GpuRuntimeBindingResource::Buffer(binding) = resource else {
                return Err(GpuSubmissionPreparationError::new(
                    GpuSubmissionPreparationErrorKind::InternalInvariant,
                    format!(
                        "validated dynamic binding {} no longer contains a buffer",
                        declaration.key()
                    ),
                ));
            };
            let offset = binding.dynamic_offset().ok_or_else(|| {
                GpuSubmissionPreparationError::new(
                    GpuSubmissionPreparationErrorKind::InternalInvariant,
                    format!(
                        "validated dynamic binding {} lost its per-use offset",
                        declaration.key()
                    ),
                )
            })?;
            offsets.push(u32::try_from(offset).map_err(|_| {
                GpuSubmissionPreparationError::new(
                    GpuSubmissionPreparationErrorKind::DynamicOffsetNotEncodable,
                    format!(
                        "logical dynamic offset {offset} for {} exceeds the private WGPU u32 domain",
                        declaration.key()
                    ),
                )
            })?);
        }
    }
    Ok(offsets)
}

fn copy_alignment(context: &GpuContext) -> Result<u64, GpuSubmissionPreparationError> {
    context
        .device_facts()
        .device_limits()
        .alignments()
        .copy_buffer_offset
        .ok_or_else(|| {
            GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::InternalInvariant,
                "created device did not publish its required buffer-copy alignment fact",
            )
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

fn preparation_program_binding_failure(
    error: GpuProgramBindingRealizationError,
) -> GpuSubmissionPreparationError {
    let kind = if error.category()
        == GpuProgramBindingRealizationErrorCategory::ContextOrDeviceUnavailableOrLost
    {
        GpuSubmissionPreparationErrorKind::ContextOrDeviceUnavailableOrLost
    } else {
        GpuSubmissionPreparationErrorKind::ProgramBindingRealizationFailed
    };
    GpuSubmissionPreparationError::new(kind, error.to_string())
}

fn preparation_pipeline_failure(
    error: GpuPipelineRealizationError,
) -> GpuSubmissionPreparationError {
    let kind = if error.category()
        == GpuPipelineRealizationErrorCategory::ContextOrDeviceUnavailableOrLost
    {
        GpuSubmissionPreparationErrorKind::ContextOrDeviceUnavailableOrLost
    } else {
        GpuSubmissionPreparationErrorKind::PipelineRealizationFailed
    };
    GpuSubmissionPreparationError::new(kind, error.to_string())
}

fn validate_copy_range(
    offset: u64,
    size: u64,
    alignment: u64,
) -> Result<(), GpuSubmissionPreparationError> {
    if alignment == 0 || !offset.is_multiple_of(alignment) || !size.is_multiple_of(alignment) {
        return Err(GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::TransferAlignmentNotAdmitted,
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

fn encode_submit_and_register(
    backend: &WgpuContextState,
    execution: &Arc<WgpuExecutionState>,
    submission: GpuSubmissionId,
    plan: &PreparedExecutionPlan,
) -> Result<(), GpuSubmissionFailure> {
    if let Some(fault) = backend.health.terminal_fault() {
        return Err(failure_from_device_fault(&fault));
    }
    // Keep this short synchronous backend interval serialized with the residual renderer path for
    // accepted shared error attribution. Completion/mapping ownership is command-buffer-local and
    // therefore does not depend on queue-relative callback registration.
    let _attribution_gate = backend.error_attribution_gate.acquire();
    let mut encoder = backend
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("RunenGPU G5B submission"),
        });
    let mut upload_staging = Vec::new();
    let mut readback_staging = Vec::new();

    for operation in &plan.operations {
        match operation {
            PreparedExecutionOperation::Upload {
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
                    let mut mapped = staging.slice(..).get_mapped_range_mut().map_err(|error| {
                        GpuSubmissionFailure::new(
                            GpuSubmissionFailureKind::BackendValidation,
                            format!("obtain mapped upload staging range: {error}"),
                        )
                    })?;
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
            PreparedExecutionOperation::Compute {
                pipeline,
                bind_groups,
                dispatch,
            } => {
                if dispatch.as_array().contains(&0) {
                    continue;
                }
                let realized_groups = bind_groups
                    .iter()
                    .map(|group| &group.realization)
                    .collect::<Vec<_>>();
                backend
                    .pipeline_realization
                    .with_execution_compute_pipeline(
                        pipeline,
                        &backend.program_binding_realization,
                        |pipeline_object| {
                            backend.program_binding_realization.with_execution_bind_groups(
                                &realized_groups,
                                |group_objects| {
                                    let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                                        label: Some("RunenGPU G5B compute"),
                                        timestamp_writes: None,
                                    });
                                    pass.set_pipeline(pipeline_object);
                                    for (prepared, object) in bind_groups.iter().zip(group_objects) {
                                        pass.set_bind_group(
                                            prepared.index,
                                            *object,
                                            &prepared.dynamic_offsets,
                                        );
                                    }
                                    let [x, y, z] = dispatch.as_array();
                                    pass.dispatch_workgroups(x, y, z);
                                },
                            )
                        },
                    )
                    .map_err(submission_pipeline_failure)?
                    .map_err(submission_program_binding_failure)?;
            }
            PreparedExecutionOperation::Copy {
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
            PreparedExecutionOperation::Readback {
                id: readback_id,
                source,
                source_offset,
                size,
                ..
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

    let command_buffer = encoder.finish();
    if let Some(fault) = backend.health.terminal_fault() {
        return Err(failure_from_device_fault(&fault));
    }
    let encoded = EncodedSubmission {
        upload_staging,
        readback_staging,
    };
    // Accepted staging is published before the command buffer can be submitted or any deferred
    // mapping/completion callback can become runnable.
    execution.attach_staging(submission, &encoded)?;
    register_callbacks(execution, submission, &encoded, &command_buffer);
    backend.queue.submit([command_buffer]);
    if let Some(fault) = backend.health.terminal_fault() {
        return Err(failure_from_device_fault(&fault));
    }
    Ok(())
}

fn submission_program_binding_failure(
    error: GpuProgramBindingRealizationError,
) -> GpuSubmissionFailure {
    let kind = match error.category() {
        GpuProgramBindingRealizationErrorCategory::BackendResourceExhaustion => {
            GpuSubmissionFailureKind::BackendResourceExhaustion
        }
        GpuProgramBindingRealizationErrorCategory::ContextOrDeviceUnavailableOrLost => {
            GpuSubmissionFailureKind::ContextOrDeviceUnavailableOrLost
        }
        GpuProgramBindingRealizationErrorCategory::ForeignContext
        | GpuProgramBindingRealizationErrorCategory::StaleDeviceGeneration
        | GpuProgramBindingRealizationErrorCategory::CurrentRenderExecutionBridgeViolation => {
            GpuSubmissionFailureKind::InternalInvariant
        }
        _ => GpuSubmissionFailureKind::BackendValidation,
    };
    GpuSubmissionFailure::new(kind, error.to_string())
}

fn submission_pipeline_failure(error: GpuPipelineRealizationError) -> GpuSubmissionFailure {
    let kind = match error.category() {
        GpuPipelineRealizationErrorCategory::BackendResourceExhaustion => {
            GpuSubmissionFailureKind::BackendResourceExhaustion
        }
        GpuPipelineRealizationErrorCategory::ContextOrDeviceUnavailableOrLost => {
            GpuSubmissionFailureKind::ContextOrDeviceUnavailableOrLost
        }
        GpuPipelineRealizationErrorCategory::ForeignContext
        | GpuPipelineRealizationErrorCategory::StaleDeviceGeneration
        | GpuPipelineRealizationErrorCategory::CurrentRenderExecutionBridgeViolation => {
            GpuSubmissionFailureKind::InternalInvariant
        }
        _ => GpuSubmissionFailureKind::BackendValidation,
    };
    GpuSubmissionFailure::new(kind, error.to_string())
}

fn register_callbacks(
    execution: &Arc<WgpuExecutionState>,
    submission: GpuSubmissionId,
    encoded: &EncodedSubmission,
    command_buffer: &wgpu::CommandBuffer,
) {
    for (readback, staging) in &encoded.readback_staging {
        let weak = Arc::downgrade(execution);
        let readback = *readback;
        command_buffer.map_buffer_on_submit(staging, MapMode::Read, .., move |result| {
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
    command_buffer.on_submitted_work_done(move || {
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
