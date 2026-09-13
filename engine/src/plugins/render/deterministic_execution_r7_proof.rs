//! Focused Vulkan-capable proof for the maintained R7 deterministic execution seam.
//!
//! The fixture intentionally owns no method or evaluator authority. It reaches execution only
//! through `admit_deterministic_render`, then proves the maintained ordinary and verified paths use
//! their intended observation policy: ordinary execution authors no readback at all, while verified
//! execution retains exactly the renderer-private canonical-output, definedness, and evaluator-status
//! readbacks from the same exact `GpuSubmission`.

use super::admission::{
    RenderOutputBinding, RenderOutputDestination, RenderRepresentationAvailabilityFact,
    RenderRepresentationAvailabilityState,
};
use super::deterministic_admission::{AdmittedDeterministicRender, admit_deterministic_render};
use super::deterministic_execution::submit_deterministic_render;
use super::deterministic_verification::submit_deterministic_render_for_verified_formation;
use super::participation::RenderObjectParticipation;
use super::representation::{
    RENDER_SURFACE_QUERY_PROTOCOL_REVISION, RenderRefinementEvidence, RenderRepresentationRecord,
    RenderSurfaceProtocolEvidence,
};
use super::request::{
    RenderObservationSpec, RenderOutputSpec, RenderOutputValue, RenderPerspectiveObservation,
    RenderRequest, RenderRequestedOutput, RenderResultTopology, RenderSamplingSupport,
    RenderSemanticTolerance,
};
use super::scene::{RenderObjectState, RenderSceneSnapshot, RenderSceneStore, RenderSceneUpdate};
use super::space_time::{
    RenderAffineTransform3, RenderHandedness, RenderObjectSpatialState, RenderObjectTemporalState,
    RenderSpaceSpec, RenderSpatialCoverage, RenderTemporalSupport, RenderTimeInterval,
    RenderTimePoint,
};
use super::surface_input::{
    RenderSurfaceSemanticInput, RenderSurfaceSemanticInputBinding,
    RenderSurfaceSemanticInputRequirement,
};
use runen_gpu::{
    GpuBufferDescriptor, GpuBufferInitialization, GpuBufferUsage, GpuCapabilityProfile, GpuContext,
    GpuContextDescriptor, GpuContextRequestErrorCategory, GpuReadbackId, GpuReadbackStatus,
    GpuReconstruction, GpuResourceLifetime, GpuSubmission, GpuSubmissionStatus,
    GpuWorkResourceIdAllocator,
};
use std::time::{Duration, Instant};

struct MaintainedExecutionFixture {
    scene: RenderSceneSnapshot,
    request: RenderRequest,
    semantic_inputs: Vec<RenderSurfaceSemanticInputBinding>,
    availability: Vec<RenderRepresentationAvailabilityFact>,
}

fn instant() -> RenderTimeInterval {
    RenderTimeInterval::instant(
        RenderTimePoint::from_seconds(0.0).expect("finite R7 maintained proof time"),
    )
}

fn translated_identity_state() -> RenderObjectState {
    RenderObjectState::new(
        RenderObjectSpatialState::new(
            RenderSpaceSpec::new(1.0, RenderHandedness::Right)
                .expect("R7 maintained proof local space"),
            RenderAffineTransform3::from_row_major_3x4([
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, -3.0,
            ])
            .expect("finite translation-only R7 maintained proof transform"),
            RenderSpatialCoverage::unbounded(),
        ),
        RenderObjectTemporalState::new(RenderTemporalSupport::unbounded()),
    )
}

fn maintained_surface_evidence() -> RenderSurfaceProtocolEvidence {
    RenderSurfaceProtocolEvidence::exact(RENDER_SURFACE_QUERY_PROTOCOL_REVISION)
        .expect("R7 maintained surface protocol")
        .with_semantic_input_requirement(RenderSurfaceSemanticInputRequirement::current())
}

fn maintained_fixture() -> MaintainedExecutionFixture {
    let mut store = RenderSceneStore::new();
    let object_id = store
        .allocate_object_id()
        .expect("R7 maintained proof object id");
    let mut insert = RenderSceneUpdate::new();
    insert.insert_with_state(object_id, translated_identity_state());
    store
        .commit(insert)
        .expect("insert R7 maintained proof object");

    let representation_id = store
        .allocate_representation_id(object_id)
        .expect("R7 maintained proof representation id");
    let representation = RenderRepresentationRecord::new(
        representation_id,
        RenderSpatialCoverage::unbounded(),
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        Some(maintained_surface_evidence()),
        None,
    )
    .expect("R7 maintained proof representation");
    let participation = RenderObjectParticipation::new(vec![representation], None, None)
        .expect("R7 maintained proof participation");
    let mut attach = RenderSceneUpdate::new();
    attach.replace_participation(object_id, participation);
    store
        .commit(attach)
        .expect("attach R7 maintained proof participation");

    let shutter = instant();
    let observation = RenderObservationSpec::Perspective(
        RenderPerspectiveObservation::new(
            RenderAffineTransform3::identity(),
            std::f64::consts::FRAC_PI_3,
            1.0,
            shutter,
            RenderSamplingSupport::ideal_ray(),
        )
        .expect("R7 maintained perspective observation"),
    );
    let request = RenderRequest::new(
        shutter,
        vec![observation],
        vec![RenderRequestedOutput::new(
            0,
            RenderOutputSpec::new(
                RenderOutputValue::ObjectIdentity,
                RenderResultTopology::scalar(),
                RenderSemanticTolerance::exact(),
            )
            .expect("R7 maintained object-identity output"),
        )],
    )
    .expect("R7 maintained proof request");
    let semantic_input = RenderSurfaceSemanticInput::sphere(
        [0.0, 0.0, 0.0],
        1.0,
        RenderTemporalSupport::unbounded(),
    )
    .expect("R7 maintained sphere semantic input");

    MaintainedExecutionFixture {
        scene: store.snapshot(),
        request,
        semantic_inputs: vec![RenderSurfaceSemanticInputBinding::new(
            representation_id,
            semantic_input,
        )],
        availability: vec![RenderRepresentationAvailabilityFact::new(
            representation_id,
            RenderRepresentationAvailabilityState::Available,
        )],
    }
}

fn request_execution_context() -> Option<GpuContext> {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .with_label("RunenRender R7 maintained execution proof");
    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => Some(context),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENRENDER_R7_REQUIRE_GPU").ok().as_deref(),
                Some("1"),
                "permanent R7 maintained execution CI requires a public RunenGPU adapter"
            );
            None
        }
        Err(error) => panic!("unexpected R7 maintained RunenGPU context failure: {error}"),
    }
}

fn admit_with_writable_only_destination(
    fixture: &MaintainedExecutionFixture,
    context: &GpuContext,
    label: &str,
) -> AdmittedDeterministicRender {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let destination = allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::ordinary_owned(
                label,
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                4,
                [GpuBufferUsage::CopyDestination],
                GpuBufferInitialization::Uninitialized,
            )
            .expect("R7 maintained writable-only scalar descriptor"),
        )
        .expect("R7 maintained writable-only scalar handle");
    let output_bindings = [RenderOutputBinding::new(
        0,
        RenderOutputDestination::ScalarBuffer(destination),
    )];
    admit_deterministic_render(
        &fixture.scene,
        &fixture.request,
        &fixture.semantic_inputs,
        &fixture.availability,
        &output_bindings,
        context,
    )
    .expect("R7 maintained fixture must reach maintained deterministic admission")
}

fn wait_for_submission(context: &GpuContext, submission: &GpuSubmission) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => return,
            GpuSubmissionStatus::Failed(failure) => {
                panic!("R7 maintained submission failed: {failure:?}")
            }
            GpuSubmissionStatus::Accepted => {}
        }
        assert!(
            Instant::now() < deadline,
            "R7 maintained submission did not complete before timeout"
        );
        std::thread::yield_now();
    }
}

fn wait_for_readbacks(
    context: &GpuContext,
    submission: &GpuSubmission,
    readback_ids: &[GpuReadbackId],
) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        context.progress();
        if let GpuSubmissionStatus::Failed(failure) = submission.status() {
            panic!("R7 maintained verified submission failed: {failure:?}");
        }

        let mut all_ready = true;
        for readback_id in readback_ids {
            let readback = submission
                .readback(*readback_id)
                .expect("R7 maintained correlation must stay on the exact submission");
            match readback.status() {
                GpuReadbackStatus::Ready(_) => {}
                GpuReadbackStatus::Failed(failure) => {
                    panic!("R7 maintained private readback failed: {failure:?}")
                }
                GpuReadbackStatus::Pending => all_ready = false,
            }
        }

        if all_ready && matches!(submission.status(), GpuSubmissionStatus::Completed) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "R7 maintained private readbacks did not become ready before timeout"
        );
        std::thread::yield_now();
    }
}

#[test]
fn maintained_execution_keeps_ordinary_unobserved_and_verified_same_submission_observed() {
    let Some(context) = request_execution_context() else {
        return;
    };
    let fixture = maintained_fixture();

    let ordinary_admitted =
        admit_with_writable_only_destination(&fixture, &context, "R7 ordinary destination");
    let ordinary = pollster::block_on(submit_deterministic_render(ordinary_admitted, &context))
        .expect("ordinary maintained execution must submit");
    assert!(
        ordinary.submission().readbacks().is_empty(),
        "ordinary maintained execution must author no CPU readbacks"
    );
    wait_for_submission(&context, ordinary.submission());

    let verified_admitted =
        admit_with_writable_only_destination(&fixture, &context, "R7 verified destination");
    let verified = pollster::block_on(submit_deterministic_render_for_verified_formation(
        verified_admitted,
        &context,
    ))
    .expect("verified maintained execution must submit with same-submission correlation");
    assert_eq!(
        verified.readbacks().len(),
        1,
        "one admitted output must retain one private verification correlation"
    );
    let correlation = verified.readbacks()[0];
    assert_eq!(correlation.output_index(), 0);
    let readback_ids = [
        correlation.canonical_output(),
        correlation.definedness(),
        correlation.status(),
    ];
    let submission = verified.submitted().submission();
    assert_eq!(
        submission.readbacks().len(),
        readback_ids.len(),
        "verified execution must add exactly three renderer-private readbacks per output"
    );
    for readback_id in readback_ids {
        assert!(
            submission.readback(readback_id).is_some(),
            "every retained correlation must resolve through the exact returned submission"
        );
    }
    wait_for_readbacks(&context, submission, &readback_ids);
}
