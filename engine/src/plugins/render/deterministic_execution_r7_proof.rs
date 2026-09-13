//! Focused Vulkan-capable proof for the maintained R7 deterministic execution seam.
//!
//! The fixture intentionally owns no method or evaluator authority. It reaches execution only
//! through `admit_deterministic_render`, then proves the maintained ordinary and verified paths use
//! their intended observation policy: ordinary execution authors no readback at all, while verified
//! execution retains exactly the renderer-private canonical-output, definedness, and evaluator-status
//! readbacks from the same exact `GpuSubmission`, can establish the private RR566-EVAL-001 witness,
//! and can form public FORM-001 result evidence through the maintained submitted-render surface.

use super::admission::{
    RenderOutputBinding, RenderOutputDestination, RenderRepresentationAvailabilityFact,
    RenderRepresentationAvailabilityState,
};
use super::deterministic_admission::{AdmittedDeterministicRender, admit_deterministic_render};
use super::deterministic_execution::{
    RenderDeterministicResultFormationError, submit_deterministic_render,
    submit_deterministic_render_for_verified_result,
};
use super::deterministic_verification::{
    submit_deterministic_render_for_verified_formation, verify_completed_deterministic_render,
};
use super::participation::RenderObjectParticipation;
use super::representation::{
    RENDER_SURFACE_QUERY_PROTOCOL_REVISION, RenderRefinementEvidence, RenderRepresentationRecord,
    RenderSurfaceProtocolEvidence,
};
use super::request::{
    RenderDistanceConvention, RenderObservationSpec, RenderOutputSpec, RenderOutputValue,
    RenderPerspectiveObservation, RenderRequest, RenderRequestedOutput, RenderResultTopology,
    RenderSamplingSupport, RenderSemanticTolerance,
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
    GpuCapabilityProfile, GpuContext, GpuContextDescriptor, GpuContextRequestErrorCategory,
    GpuFormatRole, GpuReadbackId, GpuReadbackStatus, GpuReconstruction, GpuResourceLifetime,
    GpuSubmission, GpuSubmissionStatus, GpuTextureDescriptor, GpuTextureFormat,
    GpuTextureInitialization, GpuTextureUsage, GpuWorkResourceIdAllocator,
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
                RenderResultTopology::sample_lattice_2d(2, 2)
                    .expect("R7 maintained 2x2 lattice topology"),
                RenderSemanticTolerance::exact(),
            )
            .expect("R7 maintained object-identity output"),
        )],
    )
    .expect("R7 maintained proof request");
    let semantic_input = RenderSurfaceSemanticInput::sphere(
        [0.0, 0.0, 0.0],
        2.0,
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

fn reordered_8x6_request() -> RenderRequest {
    let shutter = instant();
    let observation = RenderObservationSpec::Perspective(
        RenderPerspectiveObservation::new(
            RenderAffineTransform3::identity(),
            std::f64::consts::FRAC_PI_2 * 1.25,
            8.0 / 6.0,
            shutter,
            RenderSamplingSupport::ideal_ray(),
        )
        .expect("semantically legal wide R7 perspective"),
    );
    let lattice = || {
        RenderResultTopology::sample_lattice_2d(8, 6).expect("R7 maintained structural 8x6 lattice")
    };
    RenderRequest::new(
        shutter,
        vec![observation],
        vec![
            RenderRequestedOutput::new(
                0,
                RenderOutputSpec::new(
                    RenderOutputValue::ObjectIdentity,
                    lattice(),
                    RenderSemanticTolerance::exact(),
                )
                .expect("R7 reordered identity output"),
            ),
            RenderRequestedOutput::new(
                0,
                RenderOutputSpec::new(
                    RenderOutputValue::Distance {
                        convention: RenderDistanceConvention::ObservationForwardDepth,
                    },
                    lattice(),
                    RenderSemanticTolerance::exact(),
                )
                .expect("R7 reordered depth output"),
            ),
        ],
    )
    .expect("R7 reordered 8x6 request")
}

fn request_execution_context() -> Option<GpuContext> {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .require_format_role(GpuTextureFormat::R32Uint, GpuFormatRole::CopyDestination)
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
        .allocate_texture_handle(
            GpuTextureDescriptor::ordinary_owned_2d(
                label,
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                2,
                2,
                GpuTextureFormat::R32Uint,
                [GpuTextureUsage::CopyDestination],
                GpuTextureInitialization::Uninitialized,
            )
            .expect("R7 maintained writable-only lattice descriptor"),
        )
        .expect("R7 maintained writable-only lattice handle");
    let output_bindings = [RenderOutputBinding::new(
        0,
        RenderOutputDestination::SampleLatticeTexture(destination),
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

fn lattice_bindings(width: u32, height: u32, output_count: usize) -> Vec<RenderOutputBinding> {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    (0..output_count)
        .map(|output_index| {
            let destination = allocator
                .allocate_texture_handle(
                    GpuTextureDescriptor::ordinary_owned_2d(
                        format!("R7 structural output {output_index}"),
                        GpuResourceLifetime::Transient,
                        GpuReconstruction::SourceBacked,
                        width,
                        height,
                        GpuTextureFormat::R32Uint,
                        [GpuTextureUsage::CopyDestination],
                        GpuTextureInitialization::Uninitialized,
                    )
                    .expect("R7 structural lattice descriptor"),
                )
                .expect("R7 structural lattice handle");
            RenderOutputBinding::new(
                output_index,
                RenderOutputDestination::SampleLatticeTexture(destination),
            )
        })
        .collect()
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

    let raw_canonical = match submission
        .readback(correlation.canonical_output())
        .expect("canonical correlation")
        .status()
    {
        GpuReadbackStatus::Ready(bytes) => bytes,
        status => panic!("canonical observation must be ready after wait: {status:?}"),
    };
    assert_eq!(
        raw_canonical.layout().byte_len(),
        u64::try_from(raw_canonical.as_bytes().len()).expect("readback byte length must fit u64")
    );
    assert!(
        raw_canonical.layout().byte_len() >= 4 * 4,
        "2x2 canonical observation must contain at least four one-word logical samples"
    );

    let verified = verify_completed_deterministic_render(verified)
        .expect("same-submission object-identity samples must satisfy RR566-EVAL-001");
    assert!(matches!(
        verified.submitted().submission().status(),
        GpuSubmissionStatus::Completed
    ));
}

#[test]
fn public_verified_result_path_forms_once_from_exact_submission() {
    let Some(context) = request_execution_context() else {
        return;
    };
    let fixture = maintained_fixture();
    let admitted =
        admit_with_writable_only_destination(&fixture, &context, "R7 public verified destination");
    let mut submitted = pollster::block_on(submit_deterministic_render_for_verified_result(
        admitted, &context,
    ))
    .expect("public verified-result path must author one maintained submission");

    assert_eq!(
        submitted.submission().readbacks().len(),
        3,
        "one public verified output must retain exactly three renderer-private observations"
    );

    let deadline = Instant::now() + Duration::from_secs(15);
    let result = loop {
        context.progress();
        match submitted.try_form_verified_result() {
            Ok(Some(result)) => break result,
            Ok(None) => {}
            Err(error) => panic!("public verified-result formation failed: {error}"),
        }
        assert!(
            Instant::now() < deadline,
            "public verified-result formation did not complete before timeout"
        );
        std::thread::yield_now();
    };

    assert_eq!(result.request().outputs().len(), 1);
    assert_eq!(
        result.surface_semantic_inputs(),
        fixture.semantic_inputs.as_slice()
    );
    assert_eq!(result.outputs().len(), 1);
    assert_eq!(result.outputs()[0].output_index(), 0);
    assert_eq!(
        submitted.try_form_verified_result(),
        Err(RenderDeterministicResultFormationError::ResultAlreadyFormed),
        "one exact verified submission must not mint semantic result evidence twice"
    );
}

#[test]
fn maintained_ordinary_execution_supports_reordered_8x6_subset_outside_verifier_domain() {
    let Some(context) = request_execution_context() else {
        return;
    };
    let fixture = maintained_fixture();
    let request = reordered_8x6_request();
    let output_bindings = lattice_bindings(8, 6, request.outputs().len());
    let admitted = admit_deterministic_render(
        &fixture.scene,
        &request,
        &fixture.semantic_inputs,
        &fixture.availability,
        &output_bindings,
        &context,
    )
    .expect("wide reordered 8x6 subset must remain legal maintained evaluator work");

    let requested = admitted.admitted().plan().request().outputs();
    assert_eq!(
        requested.len(),
        2,
        "structural proof intentionally uses an output subset"
    );
    assert!(matches!(
        requested[0].spec().value(),
        RenderOutputValue::ObjectIdentity
    ));
    assert!(matches!(
        requested[1].spec().value(),
        RenderOutputValue::Distance {
            convention: RenderDistanceConvention::ObservationForwardDepth
        }
    ));
    for output in requested {
        assert_eq!(
            output.spec().topology().sample_lattice_dimensions(),
            Some((8, 6)),
            "maintained topology must come from the exact request"
        );
    }

    let submitted = pollster::block_on(submit_deterministic_render(admitted, &context))
        .expect("reordered 8x6 subset must submit through ordinary maintained execution");
    assert!(
        submitted.submission().readbacks().is_empty(),
        "structural ordinary execution must not gain verification readbacks"
    );
    wait_for_submission(&context, submitted.submission());
}
