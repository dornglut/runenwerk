use super::*;
use crate::plugins::render::renderer::Renderer;
use runen_gpu::{
    GpuCapabilityProfile, GpuContextDescriptor, GpuContextRequestErrorCategory, GpuFormatRole,
    GpuReconstruction, GpuResourceLifetime, GpuTextureDescriptor, GpuTextureFormat,
    GpuTextureInitialization, GpuTextureUsage, GpuWorkResourceIdAllocator,
};
use runen_render::admission::{RenderOutputBinding, RenderOutputDestination};
use runen_render::request::{
    RenderObservationSpec, RenderOutputSpec, RenderOutputValue, RenderPerspectiveObservation,
    RenderRadiometricRepresentation, RenderRequestBuilder, RenderResultTopology,
    RenderSamplingSupport, RenderSemanticTolerance,
};
use runen_render::scene::RenderSceneStore;
use runen_render::space_time::{RenderAffineTransform3, RenderTimeInterval, RenderTimePoint};
use std::time::{Duration, Instant};

fn context() -> Option<(GpuContext, GpuContextDescriptor)> {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .require_format_role(GpuTextureFormat::R32Float, GpuFormatRole::CopyDestination)
            .with_label("Runenwerk mapped retained sessions");
    match pollster::block_on(GpuContext::request(descriptor.clone())) {
        Ok(context) => Some((context, descriptor)),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENRENDER_R8_REQUIRE_GPU").ok().as_deref(),
                Some("1")
            );
            None
        }
        Err(error) => panic!("mapped-session context: {error}"),
    }
}

fn contribution(raw: u64) -> RenderDeterministicFrameContribution {
    let key = continuity(raw, RenderSurfaceId::primary());
    let shutter = RenderTimeInterval::instant(RenderTimePoint::from_seconds(0.0).unwrap());
    let observation = RenderPerspectiveObservation::new(
        RenderAffineTransform3::identity(),
        std::f64::consts::FRAC_PI_3,
        1.0,
        shutter,
        RenderSamplingSupport::perspective_lattice_cell(),
    )
    .unwrap();
    let output = RenderOutputSpec::new(
        RenderOutputValue::Radiance {
            representation: RenderRadiometricRepresentation::spectral_at_wavelength_meters(550e-9)
                .unwrap(),
        },
        RenderResultTopology::sample_lattice_2d(2, 2).unwrap(),
        RenderSemanticTolerance::exact(),
    )
    .unwrap();
    let mut builder = RenderRequestBuilder::new(shutter);
    let observation = builder.add_observation(RenderObservationSpec::Perspective(observation));
    let output = builder
        .add_output(&observation, output)
        .expect("own output");
    let request = builder.finish().expect("retained request");
    RenderDeterministicFrameContribution {
        producer_id: key.producer,
        render_surface_id: key.surface,
        target_key: key.target,
        scene: RenderSceneStore::new().snapshot(),
        request,
        semantic_inputs: Vec::new(),
        field_semantic_inputs: Vec::new(),
        availability: Vec::new(),
        output,
        finite_evaluation_extent: None,
    }
}

fn admit(
    contribution: &RenderDeterministicFrameContribution,
    context: &GpuContext,
) -> AdmittedRender {
    let target = GpuWorkResourceIdAllocator::new()
        .allocate_texture_handle(
            GpuTextureDescriptor::ordinary_owned_2d(
                "mapped radiance destination",
                GpuResourceLifetime::Retained,
                GpuReconstruction::SourceBacked,
                2,
                2,
                GpuTextureFormat::R32Float,
                [GpuTextureUsage::CopyDestination],
                GpuTextureInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap();
    let invocation = runen_render::RenderInvocation::new(
        contribution.scene.clone(),
        contribution.request.clone(),
        contribution.semantic_inputs.clone(),
        contribution.field_semantic_inputs.clone(),
        contribution.availability.clone(),
        vec![RenderOutputBinding::new(
            contribution.output.clone(),
            RenderOutputDestination::SampleLatticeTexture(target),
        )],
    )
    .expect("retained request-owned invocation");
    runen_render::admit_render(&invocation, context).unwrap()
}

fn prepare(
    renderer: &mut Renderer,
    targets: &RenderDynamicTextureTargetRequestRegistryResource,
    contribution: &RenderDeterministicFrameContribution,
    context: &GpuContext,
) -> PreparedRetainedOccurrence {
    renderer
        .render_sessions
        .prepare(
            contribution,
            targets,
            admit(contribution, context),
            context,
            RenderEvaluationSelection::new(contribution.output.clone(), 2, 2),
        )
        .unwrap()
}

fn evidence(
    occurrence: &PreparedRetainedOccurrence,
) -> runen_render::RenderTemporalExecutionEvidence {
    occurrence
        .occurrence()
        .radiance_output(
            &occurrence
                .occurrence()
                .admitted_plan()
                .plan()
                .request()
                .output_handle(0)
                .expect("retained requested output"),
        )
        .unwrap()
        .temporal_execution_evidence()
        .unwrap()
}

fn complete(renderer: &mut Renderer, context: &GpuContext, submission: &GpuSubmission) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        // Simulate successive renderer frames through the actual existing progress/reconcile owner.
        renderer.begin_frame_gpu_observation(context).unwrap();
        match submission.status() {
            GpuSubmissionStatus::Completed => return,
            GpuSubmissionStatus::Accepted if Instant::now() < deadline => std::thread::yield_now(),
            status => panic!("mapped-session submission did not complete: {status:?}"),
        }
    }
}

#[test]
fn independent_mapped_occurrences_compose_with_peer_work_and_advance_only_after_completion() {
    let Some((context, _)) = context() else {
        return;
    };
    let a = contribution(21);
    let b = contribution(22);
    let unrelated = contribution(23);
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    for contribution in [&a, &b, &unrelated] {
        register(&mut targets, &Continuity::from(contribution));
    }
    let mut renderer = Renderer::new();
    let first = prepare(&mut renderer, &targets, &a, &context);
    assert!(
        renderer
            .render_sessions
            .prepare(&a, &targets, admit(&a, &context), &context, None)
            .is_err()
    );
    let second = prepare(&mut renderer, &targets, &b, &context);
    assert_eq!(evidence(&first).history_age, 0);
    assert_eq!(evidence(&second).history_age, 0);
    let peer = runen_render::prepare_render(admit(&unrelated, &context), &context).unwrap();
    let submission = pollster::block_on(
        context.submit_work(
            "caller-composed independent occurrences",
            first
                .occurrence()
                .work_set()
                .fragments()
                .iter()
                .cloned()
                .chain(second.occurrence().work_set().fragments().iter().cloned())
                .chain(peer.work_set().fragments().iter().cloned()),
        ),
    )
    .unwrap();
    assert_eq!(submission.status(), GpuSubmissionStatus::Accepted);
    renderer
        .render_sessions
        .associate_composed_submission(vec![first, second], &submission)
        .unwrap();
    assert!(renderer.render_sessions.is_in_flight(&a));
    assert!(renderer.render_sessions.is_in_flight(&b));
    assert!(!renderer.render_sessions.is_in_flight(&unrelated));
    complete(&mut renderer, &context, &submission);
    assert!(!renderer.render_sessions.is_in_flight(&a));
    assert!(!renderer.render_sessions.is_in_flight(&b));
    let next_a = prepare(&mut renderer, &targets, &a, &context);
    let next_b = prepare(&mut renderer, &targets, &b, &context);
    assert_eq!(evidence(&next_a).history_age, 1);
    assert_eq!(evidence(&next_b).history_age, 1);
    assert!(!evidence(&next_a).history_reset);
    assert_eq!(
        evidence(&prepare(&mut renderer, &targets, &unrelated, &context)).history_age,
        0
    );
}

#[test]
fn missing_work_quarantines_only_failed_continuity_and_keeps_exact_peer_association() {
    let Some((context, _)) = context() else {
        return;
    };
    let a = contribution(31);
    let b = contribution(32);
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    for contribution in [&a, &b] {
        register(&mut targets, &Continuity::from(contribution));
    }
    let mut renderer = Renderer::new();
    let missing = prepare(&mut renderer, &targets, &a, &context);
    let included = prepare(&mut renderer, &targets, &b, &context);
    let submission = pollster::block_on(context.submit_work(
        "only the second occurrence",
        included.occurrence().work_set().fragments().iter().cloned(),
    ))
    .unwrap();
    let failure = renderer
        .render_sessions
        .associate_composed_submission(vec![missing, included], &submission)
        .expect_err("missing renderer work must fail exact association");
    assert!(matches!(
        failure.downcast_ref::<runen_render::RenderExecutionSessionError>(),
        Some(runen_render::RenderExecutionSessionError::SubmissionMissingRendererWork)
    ));
    assert!(
        renderer.render_sessions.sessions[&Continuity::from(&a)]
            .unassociated_submission
            .is_some()
    );
    assert!(renderer.render_sessions.is_in_flight(&b));
    complete(&mut renderer, &context, &submission);
    assert!(
        renderer.render_sessions.sessions[&Continuity::from(&a)]
            .unassociated_submission
            .is_none()
    );
    assert!(
        renderer
            .render_sessions
            .prepare(&a, &targets, admit(&a, &context), &context, None)
            .is_err()
    );
    assert_eq!(
        evidence(&prepare(&mut renderer, &targets, &b, &context)).history_age,
        1
    );
    targets.remove_contribution(a.producer_id);
    register(&mut targets, &Continuity::from(&a));
    renderer.render_sessions.synchronize(&targets);
    assert_eq!(
        evidence(&prepare(&mut renderer, &targets, &a, &context)).history_age,
        0
    );
}

#[test]
fn mapped_history_resets_on_device_generation_replacement_and_abandoned_preparation() {
    let Some((mut context, descriptor)) = context() else {
        return;
    };
    let a = contribution(41);
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    register(&mut targets, &Continuity::from(&a));
    let mut renderer = Renderer::new();
    let first = prepare(&mut renderer, &targets, &a, &context);
    let initial = evidence(&first);
    let submission = pollster::block_on(context.submit_work(
        "bootstrap mapped history",
        first.occurrence().work_set().fragments().iter().cloned(),
    ))
    .unwrap();
    renderer
        .render_sessions
        .associate_composed_submission(vec![first], &submission)
        .unwrap();
    complete(&mut renderer, &context, &submission);
    let abandoned = prepare(&mut renderer, &targets, &a, &context);
    assert_eq!(evidence(&abandoned).history_age, 1);
    drop(abandoned);
    let after_abandonment = prepare(&mut renderer, &targets, &a, &context);
    assert_eq!(evidence(&after_abandonment).history_age, 0);
    assert!(evidence(&after_abandonment).history_reset);
    drop(after_abandonment);
    renderer.render_sessions.reconcile();
    pollster::block_on(context.replace_device_generation(descriptor)).unwrap();
    let new_generation = prepare(&mut renderer, &targets, &a, &context);
    let reset = evidence(&new_generation);
    assert!(reset.history_reset);
    assert_eq!(reset.history_age, 0);
    assert!(reset.history_generation > initial.history_generation);
}

#[test]
fn terminal_context_failure_is_reconciled_without_successful_history_or_polling() {
    let Some((context, _)) = context() else {
        return;
    };
    let a = contribution(51);
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    register(&mut targets, &Continuity::from(&a));
    let mut renderer = Renderer::new();
    let prepared = prepare(&mut renderer, &targets, &a, &context);
    let submission = pollster::block_on(context.submit_work(
        "context-owned failure",
        prepared.occurrence().work_set().fragments().iter().cloned(),
    ))
    .unwrap();
    renderer
        .render_sessions
        .associate_composed_submission(vec![prepared], &submission)
        .unwrap();
    drop(context);
    assert!(matches!(
        submission.status(),
        GpuSubmissionStatus::Failed(_)
    ));
    renderer.render_sessions.reconcile();
    assert!(!renderer.render_sessions.is_in_flight(&a));
}
