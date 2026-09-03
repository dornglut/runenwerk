use super::*;
use crate::plugins::gpu::*;
use core::num::NonZeroU64;
use std::collections::BTreeSet;

fn label(value: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(value).unwrap()
}

fn provenance(value: &str) -> GpuResourceProvenance {
    GpuResourceProvenance::new(label(value), None, None)
}

fn affinity() -> GpuContextAffinity {
    GpuContextAffinity::test_value(
        GpuContextId::test_value(NonZeroU64::new(101).unwrap()),
        GpuDeviceGeneration::test_value(NonZeroU64::new(7).unwrap()),
    )
}

fn submission(value: u64) -> GpuSubmissionId {
    GpuSubmissionId::from_nonzero(NonZeroU64::new(value).unwrap())
}

fn retained_buffer() -> GpuBufferHandle {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let resource_label = label("terminal retained buffer");
    let common = GpuResourceCommon::owned(
        resource_label.clone(),
        GpuResourceLifetime::Retained,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        provenance("terminal retained buffer"),
    )
    .unwrap();
    allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common,
                64,
                GpuBufferUsages::new(
                    &resource_label,
                    [GpuBufferUsage::Storage, GpuBufferUsage::CopyDestination],
                )
                .unwrap(),
                GpuBufferInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap()
}

fn coverage(buffer: &GpuBufferHandle, offset: u64, size: u64) -> GpuInitialCoverage {
    GpuInitialCoverage::buffer(
        buffer,
        [GpuBufferCoverage::dense(
            GpuBufferRange::new(buffer, offset, size).unwrap(),
        )],
    )
    .unwrap()
}

fn transition(
    buffer: &GpuBufferHandle,
    consumed_seed: Option<GpuInitialCoverage>,
    initial: Option<GpuInitialCoverage>,
    final_coverage: Option<GpuInitialCoverage>,
    failure_preserved_coverage: Option<GpuInitialCoverage>,
) -> PreparedRetainedContinuity {
    PreparedRetainedContinuity {
        resources: [(
            buffer.diagnostic_identity(),
            PreparedRetainedResource {
                resource: GpuResourceRef::Buffer(buffer.clone()),
                consumed_seed,
                initial,
                final_coverage,
                failure_preserved_coverage,
            },
        )]
        .into_iter()
        .collect(),
    }
}

fn establish(
    state: &RetainedContinuityState,
    buffer: &GpuBufferHandle,
    initialized: &GpuInitialCoverage,
    completed_write: GpuSubmissionId,
) {
    let prepared = transition(buffer, None, None, Some(initialized.clone()), None);
    state.validate_and_reserve(&prepared).unwrap();
    state.complete(
        completed_write,
        &prepared,
        &BTreeSet::from([buffer.diagnostic_identity()]),
    );
}

fn execution_source() -> &'static str {
    include_str!("../../execution.rs")
}

#[test]
fn queue_submission_makes_current_opaque_content_unknown_until_completion() {
    let buffer = retained_buffer();
    let full = coverage(&buffer, 0, 32);
    let failure_safe = coverage(&buffer, 0, 16);
    let state = RetainedContinuityState::new(affinity());
    let first_write = submission(1);
    establish(&state, &buffer, &full, first_write);

    let prepared = transition(
        &buffer,
        Some(full.clone()),
        Some(full.clone()),
        Some(full.clone()),
        Some(failure_safe.clone()),
    );
    let writes = BTreeSet::from([buffer.diagnostic_identity()]);
    state.validate_and_reserve(&prepared).unwrap();

    state.mark_may_execute(&prepared, &writes);

    let in_flight = state.snapshot(buffer.diagnostic_identity()).unwrap();
    assert_eq!(
        in_flight.opaque_content(),
        GpuOpaqueContentContinuity::Unknown
    );
    assert_eq!(in_flight.initialized_coverage(), Some(&failure_safe));

    let second_write = submission(2);
    state.complete(second_write, &prepared, &writes);

    let completed = state.snapshot(buffer.diagnostic_identity()).unwrap();
    assert_eq!(completed.initialized_coverage(), Some(&full));
    assert_eq!(
        completed.opaque_content(),
        GpuOpaqueContentContinuity::Established {
            last_completed_write: second_write,
        }
    );
}

#[test]
fn first_queue_submitted_write_is_unknown_without_inventing_initialized_coverage() {
    let buffer = retained_buffer();
    let final_coverage = coverage(&buffer, 0, 16);
    let state = RetainedContinuityState::new(affinity());
    let prepared = transition(&buffer, None, None, Some(final_coverage), None);
    let writes = BTreeSet::from([buffer.diagnostic_identity()]);
    state.validate_and_reserve(&prepared).unwrap();

    state.mark_may_execute(&prepared, &writes);

    let in_flight = state.snapshot(buffer.diagnostic_identity()).unwrap();
    assert_eq!(
        in_flight.opaque_content(),
        GpuOpaqueContentContinuity::Unknown
    );
    assert!(in_flight.initialized_coverage().is_none());

    state.fail_after_acceptance(&prepared, &writes);
    let failed = state.snapshot(buffer.diagnostic_identity()).unwrap();
    assert_eq!(failed.opaque_content(), GpuOpaqueContentContinuity::Unknown);
    assert!(failed.initialized_coverage().is_none());
}

#[test]
fn first_queue_submitted_write_establishes_only_after_successful_completion() {
    let buffer = retained_buffer();
    let initialized = coverage(&buffer, 0, 16);
    let state = RetainedContinuityState::new(affinity());
    let prepared = transition(&buffer, None, None, Some(initialized.clone()), None);
    let writes = BTreeSet::from([buffer.diagnostic_identity()]);
    state.validate_and_reserve(&prepared).unwrap();

    state.mark_may_execute(&prepared, &writes);
    let in_flight = state.snapshot(buffer.diagnostic_identity()).unwrap();
    assert_eq!(
        in_flight.opaque_content(),
        GpuOpaqueContentContinuity::Unknown
    );
    assert!(in_flight.initialized_coverage().is_none());

    let completed_write = submission(3);
    state.complete(completed_write, &prepared, &writes);
    let completed = state.snapshot(buffer.diagnostic_identity()).unwrap();
    assert_eq!(completed.initialized_coverage(), Some(&initialized));
    assert_eq!(
        completed.opaque_content(),
        GpuOpaqueContentContinuity::Established {
            last_completed_write: completed_write,
        }
    );
}

#[test]
fn ordinary_success_after_revocation_does_not_reestablish_opaque_history() {
    let buffer = retained_buffer();
    let initialized = coverage(&buffer, 0, 32);
    let state = RetainedContinuityState::new(affinity());
    establish(&state, &buffer, &initialized, submission(4));
    let writes = BTreeSet::from([buffer.diagnostic_identity()]);

    let failed_write = transition(
        &buffer,
        Some(initialized.clone()),
        Some(initialized.clone()),
        Some(initialized.clone()),
        Some(initialized.clone()),
    );
    state.validate_and_reserve(&failed_write).unwrap();
    state.mark_may_execute(&failed_write, &writes);
    state.fail_after_acceptance(&failed_write, &writes);

    let revoked = state.snapshot(buffer.diagnostic_identity()).unwrap();
    assert_eq!(
        revoked.opaque_content(),
        GpuOpaqueContentContinuity::Unknown
    );
    assert_eq!(revoked.initialized_coverage(), Some(&initialized));

    let later_write = transition(
        &buffer,
        Some(initialized.clone()),
        Some(initialized.clone()),
        Some(initialized.clone()),
        Some(initialized.clone()),
    );
    state.validate_and_reserve(&later_write).unwrap();
    state.mark_may_execute(&later_write, &writes);
    state.complete(submission(5), &later_write, &writes);

    let completed = state.snapshot(buffer.diagnostic_identity()).unwrap();
    assert_eq!(
        completed.opaque_content(),
        GpuOpaqueContentContinuity::Unknown
    );
    assert_eq!(completed.initialized_coverage(), Some(&initialized));
}

#[test]
fn execution_spine_marks_retained_writes_only_after_queue_submission() {
    let source = execution_source();
    let start = source
        .find("fn encode_submit_and_register(")
        .expect("execution submission helper must exist");
    let end = source[start..]
        .find("\nfn texture_copy_info")
        .map(|offset| start + offset)
        .expect("execution submission helper must end before texture_copy_info");
    let body = &source[start..end];
    let queue_submit = body
        .find("backend.queue.submit([segment.command_buffer]);")
        .expect("physical queue submission must remain explicit");
    let mark_may_execute = body
        .find("execution.mark_segment_may_execute(submission, &segment.retained_writes);")
        .expect("retained may-execute transition must remain explicit");

    assert!(queue_submit < mark_may_execute);
}

#[test]
fn progress_serializes_and_consumes_completion_before_terminal_fault_failure() {
    let source = execution_source();
    let start = source
        .find("pub fn progress(&self) -> GpuExecutionStats {")
        .expect("GpuContext::progress must exist");
    let end = source[start..]
        .find("\n}\n\nasync fn prepare_execution_plan")
        .map(|offset| start + offset)
        .expect("GpuContext::progress must end before execution-plan preparation");
    let body = &source[start..end];
    let submission_order = body
        .find(".submission_order")
        .expect("progress must share submission-order serialization");
    let poll = body
        .find(".device.poll(PollType::Poll)")
        .expect("progress must poll the device");
    let drains = body
        .match_indices("self.backend.execution.drain_events();")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let terminal_fault = body
        .find("self.backend.health.terminal_fault()")
        .expect("progress must inspect terminal device fault state");

    assert_eq!(drains.len(), 2);
    assert!(submission_order < drains[0]);
    assert!(drains[0] < poll);
    assert!(poll < drains[1]);
    assert!(drains[1] < terminal_fault);
}
