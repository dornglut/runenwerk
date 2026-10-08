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
    RenderObservationSpec, RenderOutputHandle, RenderOutputSpec, RenderOutputValue,
    RenderPerspectiveObservation, RenderRequestBuilder, RenderResultTopology,
    RenderSamplingSupport, RenderSemanticTolerance,
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
    RenderExecutionErrorKind, RenderExecutionSession, RenderInvocation, RenderInvocationError,
    RenderRadianceCaptureError, RenderRadianceCaptureErrorKind, RenderRadianceCaptureRequest,
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
    let _ = RenderResultSubmissionError::observation;
    let _ = RenderResultSubmissionError::object_id;
    let _ = RenderResultSubmissionError::readback_cardinality;
    let _ = RenderResultSubmissionError::output_correlation;
    let _ = RenderResultSubmissionError::correlation_output;
    let _ = RenderResultSubmissionError::correlation_channel;
    let _ = RenderResultFormationError::kind;
    let _ = RenderResultFormationError::verification_eligibility_kind;
    let _ = RenderResultFormationError::output;
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
    fn assert_evaluation_handle(output: RenderOutputHandle) {
        let selection =
            RenderEvaluationSelection::new(output.clone(), 64, 32).expect("non-zero extent");
        assert_eq!(selection.output(), &output);
        assert_eq!(selection.extent(), (64, 32));
    }
    let _ = assert_evaluation_handle;

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
        let _ = output.output();
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
    let representation = RenderRepresentationRecord::builder(
        representation_id,
        RenderSpatialCoverage::unbounded(),
        RenderTemporalSupport::unbounded(),
    )
    .surface_query(Some(surface))
    .refinement(RenderRefinementEvidence::none())
    .build()
    .expect("public surface representation");
    let participation = RenderObjectParticipation::from_representations(vec![representation])
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
    let mut builder = RenderRequestBuilder::new(shutter);
    let observation = builder.add_observation(observation);
    let output = builder
        .add_output(
            &observation,
            RenderOutputSpec::new(
                RenderOutputValue::ObjectIdentity,
                RenderResultTopology::sample_lattice_2d(1, 1).expect("1x1 public lattice"),
                RenderSemanticTolerance::exact(),
            )
            .expect("public object-identity output"),
        )
        .expect("output belongs to this request");
    let request = builder.finish().expect("public render request");
    // Semantic equality is deliberately not correlation equality between independent requests.
    let mut foreign_builder = RenderRequestBuilder::new(shutter);
    let foreign_observation = foreign_builder.add_observation(request.observations()[0]);
    let foreign_output = foreign_builder
        .add_output(&foreign_observation, request.outputs()[0].spec())
        .expect("foreign request output");
    let foreign_request = foreign_builder.finish().expect("foreign request");
    assert_eq!(request, foreign_request);
    assert_eq!(output.position(), foreign_output.position());
    assert_ne!(output, foreign_output);
    assert!(!request.contains_output(&foreign_output));
    assert_eq!(request.clone().output_handle(0), Some(output.clone()));

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
        let destination = RenderOutputDestination::SampleLatticeTexture(destination);
        let rejected = RenderInvocation::new(
            scene.snapshot(),
            request.clone(),
            semantic_inputs.to_vec(),
            Vec::new(),
            availability.to_vec(),
            vec![RenderOutputBinding::new(
                foreign_output.clone(),
                destination.clone(),
            )],
        )
        .expect_err("equal-position foreign output must not bind to this request");
        assert!(matches!(
            rejected,
            RenderInvocationError::ForeignOutput { output: rejected_output }
                if rejected_output == foreign_output
        ));
        let invocation = RenderInvocation::new(
            scene.snapshot(),
            request.clone(),
            semantic_inputs.to_vec(),
            Vec::new(),
            availability.to_vec(),
            vec![RenderOutputBinding::new(output.clone(), destination)],
        )
        .expect("valid request-owned public invocation");
        admit_render(&invocation, &context).expect("public ordinary admission")
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
    assert!(submitted.object_identity_decoder(&output).is_ok());
    let foreign_decoder = submitted
        .object_identity_decoder(&foreign_output)
        .expect_err("foreign output must not select an execution-local decoder");
    assert_eq!(foreign_decoder.output(), &foreign_output);
    assert_eq!(
        foreign_decoder.kind(),
        runen_render::RenderObjectIdentityDecoderErrorKind::OutputNotAdmitted
    );

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

#[test]
fn request_owned_radiance_survives_retention_and_rejects_foreign_evaluation_lookup_and_capture() {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .require_format_role(GpuTextureFormat::R32Float, GpuFormatRole::CopyDestination)
            .require_format_role(GpuTextureFormat::R32Float, GpuFormatRole::CopySource);
    let context = match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => context,
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENRENDER_R8_REQUIRE_GPU").ok().as_deref(),
                Some("1")
            );
            return;
        }
        Err(error) => panic!("public radiance context failed: {error}"),
    };
    let build_request = || {
        let shutter = RenderTimeInterval::instant(RenderTimePoint::from_seconds(0.0).unwrap());
        let mut builder = RenderRequestBuilder::new(shutter);
        let observation = builder.add_observation(RenderObservationSpec::Perspective(
            RenderPerspectiveObservation::new(
                RenderAffineTransform3::identity(),
                std::f64::consts::FRAC_PI_3,
                1.0,
                shutter,
                RenderSamplingSupport::perspective_lattice_cell(),
            )
            .unwrap(),
        ));
        let output = builder.add_output(&observation, RenderOutputSpec::new(
            RenderOutputValue::Radiance {
                representation: runen_render::request::RenderRadiometricRepresentation::spectral_at_wavelength_meters(550e-9).unwrap(),
            },
            RenderResultTopology::sample_lattice_2d(2, 2).unwrap(),
            RenderSemanticTolerance::exact(),
        ).unwrap()).unwrap();
        (builder.finish().unwrap(), output)
    };
    let (request, output) = build_request();
    let (foreign_request, foreign) = build_request();
    assert_eq!(request, foreign_request);
    assert_eq!(output.position(), foreign.position());
    assert_ne!(output, foreign);
    let destination = GpuWorkResourceIdAllocator::new()
        .allocate_texture_handle(
            GpuTextureDescriptor::ordinary_owned_2d(
                "public retained radiance",
                GpuResourceLifetime::Retained,
                GpuReconstruction::SourceBacked,
                2,
                2,
                GpuTextureFormat::R32Float,
                [
                    GpuTextureUsage::CopyDestination,
                    GpuTextureUsage::CopySource,
                ],
                GpuTextureInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap();
    let invocation = RenderInvocation::new(
        RenderSceneStore::new().snapshot(),
        request.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![RenderOutputBinding::new(
            output.clone(),
            RenderOutputDestination::SampleLatticeTexture(destination),
        )],
    )
    .unwrap();
    let admitted = admit_render(&invocation, &context).unwrap();
    let mut session = RenderExecutionSession::new();
    let error = session
        .prepare(
            admitted.clone(),
            &context,
            RenderEvaluationSelection::new(foreign.clone(), 2, 2),
        )
        .expect_err("foreign finite evaluation must reject before creating an occurrence");
    assert!(
        matches!(error, runen_render::RenderExecutionSessionError::ForeignEvaluationOutput { output } if output == foreign)
    );
    assert!(!session.is_in_flight());
    let one_shot = prepare_render(admitted.clone(), &context).unwrap();
    assert_eq!(one_shot.radiance_output(&output).unwrap().output(), &output);
    assert!(one_shot.radiance_output(&foreign).is_none());
    let occurrence = session
        .prepare(
            admitted,
            &context,
            RenderEvaluationSelection::new(output.clone(), 2, 2),
        )
        .unwrap();
    assert_eq!(
        occurrence.radiance_output(&output).unwrap().output(),
        &output
    );
    assert!(occurrence.radiance_output(&foreign).is_none());
    let submission = pollster::block_on(context.submit_work(
        "public request-owned retained radiance",
        occurrence.work_set().fragments().iter().cloned(),
    ))
    .unwrap();
    let associated = session
        .associate_submission(occurrence, &submission)
        .unwrap();
    wait_for_submission(&context, &submission);
    session.reconcile();
    drop(session);
    let rejected_capture = match associated.request_radiance_capture(&foreign) {
        Err(error) => error,
        Ok(_) => panic!("foreign output cannot capture this completed associated occurrence"),
    };
    assert_eq!(rejected_capture.output(), &foreign);
    assert_eq!(
        rejected_capture.kind(),
        RenderRadianceCaptureRequestErrorKind::OutputNotAdmitted
    );
    let capture = associated.request_radiance_capture(&output).unwrap();
    assert_eq!(capture.output(), &output);
    let readback =
        GpuReadbackOperation::new(capture.source().clone(), capture.readback_id()).unwrap();
    let fragment = runen_gpu::GpuWorkFragment::build("public radiance readback", |work| {
        work.operation("readback", readback)?;
        Ok(())
    })
    .unwrap();
    let readback_submission =
        pollster::block_on(context.submit_work("public radiance readback", [fragment])).unwrap();
    wait_for_submission(&context, &readback_submission);
    let captured = associated
        .capture_radiance(capture, &context, &readback_submission)
        .unwrap();
    assert_eq!(captured.samples(), &[0.0; 4]);
}
