use runen_gpu::{
    GpuCapabilityProfile, GpuContext, GpuContextDescriptor, GpuContextRequestErrorCategory,
    GpuFormatRole, GpuReadbackOperation, GpuReconstruction, GpuResourceLifetime, GpuSubmission,
    GpuSubmissionStatus, GpuTextureDescriptor, GpuTextureFormat, GpuTextureInitialization,
    GpuTextureUsage, GpuWorkResourceIdAllocator,
};
use runen_render::admission::{
    RenderOutputBinding, RenderOutputDestination, RenderRepresentationAvailabilityFact,
    RenderRepresentationAvailabilityState,
};
use runen_render::participation::RenderObjectParticipation;
use runen_render::representation::{
    RENDER_SURFACE_QUERY_PROTOCOL_REVISION, RenderRefinementEvidence, RenderRepresentationRecord,
    RenderSurfaceProtocolEvidence,
};
use runen_render::request::{
    RenderObservationSpec, RenderOutputSpec, RenderOutputValue, RenderPerspectiveObservation,
    RenderRequest, RenderRequestedOutput, RenderResultTopology, RenderSamplingSupport,
    RenderSemanticTolerance,
};
use runen_render::scene::{RenderObjectState, RenderSceneStore, RenderSceneUpdate};
use runen_render::space_time::{
    RenderAffineTransform3, RenderHandedness, RenderObjectSpatialState, RenderObjectTemporalState,
    RenderSpaceSpec, RenderSpatialCoverage, RenderTemporalSupport, RenderTimeInterval,
    RenderTimePoint,
};
use runen_render::surface_input::{
    RenderSurfaceSemanticInput, RenderSurfaceSemanticInputBinding,
    RenderSurfaceSemanticInputRequirement,
};
use runen_render::{
    AdmittedRender, PreparedRadianceOutput, PreparedRender, PreparedRenderOccurrence,
    RenderAdmissionError, RenderCapturedRadiance, RenderEvaluationSelection, RenderExecutionError,
    RenderExecutionErrorKind, RenderExecutionSession, RenderRadianceCaptureError,
    RenderRadianceCaptureErrorKind, RenderRadianceCaptureRequest,
    RenderRadianceCaptureRequestError, RenderRadianceCaptureRequestErrorKind,
    RenderResultFormationError, RenderResultFormationErrorKind, RenderResultSubmissionError,
    RenderResultSubmissionErrorKind, RenderTemporalExecutionEvidence,
    RenderVerificationEligibilityErrorKind, SubmittedRender, SubmittedRenderForResult,
    admit_render, prepare_render, submit_render, submit_render_for_result,
};
use std::time::{Duration, Instant};

#[test]
fn ordinary_semantic_renderer_surface_is_public_to_downstream_consumers() {
    let _ = admit_render;
    let _ = prepare_render;
    let _ = submit_render;
    let _ = submit_render_for_result;
    let _ = AdmittedRender::admitted_plan;
    let _ = PreparedRender::admitted_plan;
    let _ = PreparedRender::work_set;
    let _ = PreparedRender::radiance_outputs;
    let _ = PreparedRender::radiance_output;
    let _ = PreparedRadianceOutput::temporal_execution_evidence;
    let _ = PreparedRadianceOutput::import;
    let _ = SubmittedRender::admitted_plan;
    let _ = SubmittedRender::submission_status;
    let _ = SubmittedRender::object_identity_decoder;
    let _ = SubmittedRenderForResult::admitted_plan;
    let _ = SubmittedRenderForResult::submission_status;
    let _ = SubmittedRenderForResult::object_identity_decoder;
    let _ = SubmittedRenderForResult::try_form_result;
    let _ = SubmittedRenderForResult::request_radiance_capture;
    let _ = SubmittedRenderForResult::capture_radiance;
    let _ = RenderRadianceCaptureRequest::source;
    let _ = RenderRadianceCaptureRequest::readback_id;
    let _ = GpuReadbackOperation::new;
    let _ = RenderAdmissionError::kind;
    let _ = RenderExecutionError::kind;
    let _ = RenderExecutionError::runen_gpu_preparation_source;
    let _ = RenderExecutionError::submission_error;
    let _ = RenderResultSubmissionError::kind;
    let _ = RenderResultSubmissionError::verification_eligibility_kind;
    let _ = RenderResultSubmissionError::observation_index;
    let _ = RenderResultSubmissionError::object_id;
    let _ = RenderResultSubmissionError::readback_cardinality;
    let _ = RenderResultSubmissionError::output_correlation;
    let _ = RenderResultSubmissionError::correlation_output_index;
    let _ = RenderResultSubmissionError::correlation_channel;
    let _ = RenderResultFormationError::kind;
    let _ = RenderResultFormationError::verification_eligibility_kind;
    let _ = RenderResultFormationError::output_index;
    let _ = RenderResultFormationError::sample_index;
    let _ = RenderResultFormationError::channel;
    let _ = RenderResultFormationError::gpu_failure_kind;
    let _ = RenderRadianceCaptureRequestError::kind;
    let _ = RenderRadianceCaptureError::kind;
    let _ = RenderRadianceCaptureError::gpu_failure_kind;
    let _ = RenderExecutionSession::new;
    let _ = RenderExecutionSession::reconcile;
    let _ = RenderExecutionSession::associate_submission;
    let _ = RenderExecutionSession::prepare;
    let _ = PreparedRenderOccurrence::work_set;
    let _ = PreparedRenderOccurrence::radiance_output;
    assert!(!RenderExecutionSession::new().is_in_flight());
    let selection = RenderEvaluationSelection::new(0, 64, 32).expect("non-zero extent");
    assert_eq!(selection.output_index(), 0);
    assert_eq!(selection.extent(), (64, 32));

    fn assert_temporal_evidence(evidence: &RenderTemporalExecutionEvidence) {
        let _ = (
            evidence.requested_extent,
            evidence.evaluation_extent,
            evidence.sequence_revision,
            evidence.reconstruction_revision,
            evidence.phase,
            evidence.history_generation,
            evidence.history_age,
            evidence.history_reset,
            evidence.camera_reprojection_eligible,
        );
    }
    let _ = assert_temporal_evidence;

    fn assert_error_kinds(
        execution: RenderExecutionErrorKind,
        submission: RenderResultSubmissionErrorKind,
        formation: RenderResultFormationErrorKind,
        capture_request: RenderRadianceCaptureRequestErrorKind,
        capture: RenderRadianceCaptureErrorKind,
    ) {
        let _ = (execution, submission, formation, capture_request, capture);
    }
    let _ = assert_error_kinds;
    let _ = RenderVerificationEligibilityErrorKind::SamplingSupportUnsupported;

    fn assert_prepared_output_surface(output: &PreparedRadianceOutput<'_>) {
        let _ = output.output_index();
        let _ = output.resource();
        let _ = output.texture();
        let _ = output.export_relationship();
    }

    let _ = assert_prepared_output_surface;

    fn assert_capture_surface(
        submitted: &SubmittedRenderForResult,
        request: RenderRadianceCaptureRequest,
        context: &GpuContext,
        product_submission: &GpuSubmission,
    ) -> Result<RenderCapturedRadiance, RenderRadianceCaptureError> {
        submitted.capture_radiance(request, context, product_submission)
    }

    let _ = assert_capture_surface;

    fn assert_public_error<E: std::error::Error + 'static>() {}
    assert_public_error::<RenderAdmissionError>();
    assert_public_error::<RenderExecutionError>();
    assert_public_error::<RenderResultSubmissionError>();
    assert_public_error::<RenderResultFormationError>();
    assert_public_error::<RenderRadianceCaptureRequestError>();
    assert_public_error::<RenderRadianceCaptureError>();
}

#[test]
fn ordinary_surface_executes_headless_through_public_runengpu_only() {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .require_format_role(GpuTextureFormat::R32Uint, GpuFormatRole::CopyDestination)
            .with_label("RunenRender R8 ordinary public consumer");
    let context = match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => context,
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENRENDER_R8_REQUIRE_GPU").ok().as_deref(),
                Some("1"),
                "R8 ordinary public consumer CI requires a public RunenGPU adapter"
            );
            return;
        }
        Err(error) => panic!("unexpected R8 ordinary public RunenGPU context failure: {error}"),
    };

    let mut scene = RenderSceneStore::new();
    let object_id = scene.allocate_object_id().expect("R8 public object id");
    let object_state = RenderObjectState::new(
        RenderObjectSpatialState::new(
            RenderSpaceSpec::new(1.0, RenderHandedness::Right).expect("metric object space"),
            RenderAffineTransform3::from_row_major_3x4([
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, -3.0,
            ])
            .expect("finite object transform"),
            RenderSpatialCoverage::unbounded(),
        ),
        RenderObjectTemporalState::new(RenderTemporalSupport::unbounded()),
    );
    let mut insert = RenderSceneUpdate::new();
    insert.insert_with_state(object_id, object_state);
    scene.commit(insert).expect("insert public consumer object");

    let representation_id = scene
        .allocate_representation_id(object_id)
        .expect("R8 public representation id");
    let surface = RenderSurfaceProtocolEvidence::exact(RENDER_SURFACE_QUERY_PROTOCOL_REVISION)
        .expect("surface protocol")
        .with_semantic_input_requirement(RenderSurfaceSemanticInputRequirement::current());
    let representation = RenderRepresentationRecord::new(
        representation_id,
        RenderSpatialCoverage::unbounded(),
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        Some(surface),
        None,
    )
    .expect("public surface representation");
    let participation = RenderObjectParticipation::new(vec![representation], None, None)
        .expect("public object participation");
    let mut attach = RenderSceneUpdate::new();
    attach.replace_participation(object_id, participation);
    scene
        .commit(attach)
        .expect("attach public object participation");

    let shutter = RenderTimeInterval::instant(
        RenderTimePoint::from_seconds(0.0).expect("finite public consumer time"),
    );
    let observation = RenderObservationSpec::Perspective(
        RenderPerspectiveObservation::new(
            RenderAffineTransform3::identity(),
            std::f64::consts::FRAC_PI_3,
            1.0,
            shutter,
            RenderSamplingSupport::ideal_ray(),
        )
        .expect("public perspective observation"),
    );
    let request = RenderRequest::new(
        shutter,
        vec![observation],
        vec![RenderRequestedOutput::new(
            0,
            RenderOutputSpec::new(
                RenderOutputValue::ObjectIdentity,
                RenderResultTopology::sample_lattice_2d(1, 1).expect("1x1 public lattice"),
                RenderSemanticTolerance::exact(),
            )
            .expect("public object-identity output"),
        )],
    )
    .expect("public render request");

    let semantic_inputs = [RenderSurfaceSemanticInputBinding::new(
        representation_id,
        RenderSurfaceSemanticInput::sphere(
            [0.0, 0.0, 0.0],
            1.0,
            RenderTemporalSupport::unbounded(),
        )
        .expect("public sphere semantic input"),
    )];
    let availability = [RenderRepresentationAvailabilityFact::new(
        representation_id,
        RenderRepresentationAvailabilityState::Available,
    )];

    let admit_output = |label: &str| {
        let destination = GpuWorkResourceIdAllocator::new()
            .allocate_texture_handle(
                GpuTextureDescriptor::ordinary_owned_2d(
                    label,
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    1,
                    1,
                    GpuTextureFormat::R32Uint,
                    [GpuTextureUsage::CopyDestination],
                    GpuTextureInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        admit_render(
            &scene.snapshot(),
            &request,
            &semantic_inputs,
            &[],
            &availability,
            &[RenderOutputBinding::new(
                0,
                RenderOutputDestination::SampleLatticeTexture(destination),
            )],
            &context,
        )
        .expect("public ordinary admission")
    };
    let admitted = admit_output("R8 ordinary public consumer output");
    let submitted = pollster::block_on(submit_render(admitted.clone(), &context))
        .expect("public ordinary submission");

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        context.progress();
        match submitted.submission_status() {
            GpuSubmissionStatus::Completed => break,
            GpuSubmissionStatus::Failed(failure) => {
                panic!("public ordinary RunenGPU submission failed: {failure:?}")
            }
            GpuSubmissionStatus::Accepted if Instant::now() < deadline => {
                std::thread::yield_now();
            }
            GpuSubmissionStatus::Accepted => {
                panic!("public ordinary RunenGPU submission did not complete before timeout")
            }
        }
    }

    assert_eq!(submitted.admitted_plan().scene_revision(), scene.revision());
    assert_eq!(submitted.admitted_plan().outputs().len(), 1);

    // The downstream Vulkan lane exercises the retained contract alongside the unchanged one-shot
    // path. Independent sessions prepare concurrently and carry exact non-cloneable occurrences
    // across caller-owned composition with unrelated public RunenGPU work.
    let mut first_session = RenderExecutionSession::new();
    let mut second_session = RenderExecutionSession::new();
    let first = first_session
        .prepare(admit_output("first retained output"), &context, None)
        .unwrap();
    let second = second_session
        .prepare(admit_output("second retained output"), &context, None)
        .unwrap();
    let peer = prepare_render(admit_output("one-shot peer output"), &context).unwrap();
    let composed = pollster::block_on(
        context.submit_work(
            "downstream composed retained occurrences and one-shot peer",
            first
                .work_set()
                .fragments()
                .iter()
                .cloned()
                .chain(second.work_set().fragments().iter().cloned())
                .chain(peer.work_set().fragments().iter().cloned()),
        ),
    )
    .unwrap();
    for occurrence in [&first, &second] {
        assert!(
            occurrence
                .work_set()
                .fragments()
                .iter()
                .flat_map(|fragment| fragment.nodes())
                .all(|node| composed.contains_work_node(node.id()))
        );
    }
    first_session
        .associate_submission(first, &composed)
        .unwrap();
    second_session
        .associate_submission(second, &composed)
        .unwrap();
    wait_for_submission(&context, &composed);
    first_session.reconcile();
    second_session.reconcile();
    assert!(!first_session.is_in_flight());
    assert!(!second_session.is_in_flight());

    let missing = first_session
        .prepare(admit_output("first retained output"), &context, None)
        .unwrap();
    let included = second_session
        .prepare(admit_output("second retained output"), &context, None)
        .unwrap();
    let only_second = pollster::block_on(context.submit_work(
        "downstream submission missing the first occurrence",
        included.work_set().fragments().iter().cloned(),
    ))
    .unwrap();
    assert!(matches!(
        first_session.associate_submission(missing, &only_second),
        Err(runen_render::RenderExecutionSessionError::SubmissionMissingRendererWork)
    ));
    second_session
        .associate_submission(included, &only_second)
        .unwrap();
    wait_for_submission(&context, &only_second);
    first_session.reconcile();
    second_session.reconcile();

    let foreign = first_session
        .prepare(admit_output("first retained output"), &context, None)
        .unwrap();
    let own = second_session
        .prepare(admit_output("second ownership output"), &context, None)
        .unwrap();
    let both = pollster::block_on(
        context.submit_work(
            "downstream exact session ownership",
            foreign
                .work_set()
                .fragments()
                .iter()
                .cloned()
                .chain(own.work_set().fragments().iter().cloned()),
        ),
    )
    .unwrap();
    assert!(matches!(
        second_session.associate_submission(foreign, &both),
        Err(runen_render::RenderExecutionSessionError::OccurrenceNotCurrent)
    ));
    second_session.associate_submission(own, &both).unwrap();
    wait_for_submission(&context, &both);
    first_session.reconcile();
    second_session.reconcile();
}

fn wait_for_submission(context: &GpuContext, submission: &GpuSubmission) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => return,
            GpuSubmissionStatus::Accepted if Instant::now() < deadline => std::thread::yield_now(),
            status => panic!("public composed submission did not complete: {status:?}"),
        }
    }
}
