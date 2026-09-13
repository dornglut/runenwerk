from pathlib import Path
import subprocess
import textwrap

MAIN = "69f2e09ce64eb44bf450a38120350a2dda4be48d"
OLD_GPU = "git+https://github.com/dornglut/runen-gpu?rev=77c7c8d5ad6922b6f46c6b25e31b1a224c1314a4#77c7c8d5ad6922b6f46c6b25e31b1a224c1314a4"
NEW_GPU = "git+https://github.com/dornglut/runen-gpu?rev=31649491e9e7746da8e128ad984d2be315961640#31649491e9e7746da8e128ad984d2be315961640"

# Restore the accepted-main lockfile and change exactly the RunenGPU source revision.
main_lock = subprocess.check_output(["git", "show", f"{MAIN}:Cargo.lock"], text=True)
if main_lock.count(OLD_GPU) != 1 or NEW_GPU in main_lock:
    raise SystemExit("accepted-main Cargo.lock RunenGPU source did not match exact expectation")
Path("Cargo.lock").write_text(main_lock.replace(OLD_GPU, NEW_GPU, 1))

TEST = r'''
//! Downstream/public proof for product-safe maintained deterministic radiance capture.
//!
//! This integration test intentionally sits outside the renderer module so it cannot reach
//! renderer-private submission or verification state. It proves FORM-001 first, then performs a
//! separate public RunenGPU readback of retained product-owned outputs and asks RunenRender to
//! interpret only the correlated radiance lattice.

use engine::plugins::render::admission::{
    RenderOutputBinding, RenderOutputDestination, RenderRepresentationAvailabilityFact,
    RenderRepresentationAvailabilityState,
};
use engine::plugins::render::appearance::{RenderDiffuseMaterial, RenderDirectionalEmitter};
use engine::plugins::render::deterministic_admission::admit_deterministic_render;
use engine::plugins::render::deterministic_execution::{
    RenderDeterministicRadianceCaptureError, SubmittedDeterministicRender,
    submit_deterministic_render_for_verified_result,
};
use engine::plugins::render::participation::{RenderMaterialAssignment, RenderObjectParticipation};
use engine::plugins::render::representation::{
    RENDER_ORIENTED_SURFACE_QUERY_PROTOCOL_REVISION, RENDER_SURFACE_QUERY_PROTOCOL_REVISION,
    RenderOrientedSurfaceProtocolEvidence, RenderRefinementEvidence, RenderRepresentationRecord,
    RenderSurfaceProtocolEvidence,
};
use engine::plugins::render::request::{
    RenderObservationSpec, RenderOutputSpec, RenderOutputValue, RenderPerspectiveObservation,
    RenderRadiometricRepresentation, RenderRequest, RenderRequestedOutput, RenderResultTopology,
    RenderSamplingSupport, RenderSemanticTolerance,
};
use engine::plugins::render::scene::{
    RenderObjectState, RenderSceneStore, RenderSceneUpdate,
};
use engine::plugins::render::space_time::{
    RenderAffineTransform3, RenderHandedness, RenderObjectSpatialState, RenderObjectTemporalState,
    RenderSpaceSpec, RenderSpatialCoverage, RenderTemporalSupport, RenderTimeInterval,
    RenderTimePoint,
};
use engine::plugins::render::surface_input::{
    RenderSurfaceSemanticInput, RenderSurfaceSemanticInputBinding,
    RenderSurfaceSemanticInputRequirement,
};
use runen_gpu::{
    GpuCapabilityProfile, GpuContext, GpuContextDescriptor, GpuContextRequestErrorCategory,
    GpuFormatRole, GpuReadbackId, GpuReadbackOperation, GpuReadbackStatus, GpuReconstruction,
    GpuResourceLifetime, GpuSubmission, GpuSubmissionStatus, GpuTextureCopyRegion,
    GpuTextureDescriptor, GpuTextureFormat, GpuTextureHandle, GpuTextureInitialization,
    GpuTextureUsage, GpuWorkFragment, GpuWorkResourceIdAllocator,
};
use std::time::{Duration, Instant};

const WAVELENGTH_METERS: f64 = 550.0e-9;
const WIDTH: u32 = 2;
const HEIGHT: u32 = 2;
const EXPECTED_RADIANCE: f32 = 1.0;
const NUMERIC_TOLERANCE: f32 = 1.0e-5;

fn instant() -> RenderTimeInterval {
    RenderTimeInterval::instant(
        RenderTimePoint::from_seconds(0.0).expect("finite public capture proof time"),
    )
}

fn identity_object_state() -> RenderObjectState {
    RenderObjectState::new(
        RenderObjectSpatialState::new(
            RenderSpaceSpec::new(1.0, RenderHandedness::Right)
                .expect("public capture local space"),
            RenderAffineTransform3::identity(),
            RenderSpatialCoverage::unbounded(),
        ),
        RenderObjectTemporalState::new(RenderTemporalSupport::unbounded()),
    )
}

fn request_context() -> Option<GpuContext> {
    let descriptor = GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
        .require_format_role(GpuTextureFormat::R32Uint, GpuFormatRole::CopyDestination)
        .require_format_role(GpuTextureFormat::R32Uint, GpuFormatRole::CopySource)
        .with_label("RunenRender public radiance capture proof");
    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => Some(context),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENRENDER_CAPTURE_REQUIRE_GPU")
                    .ok()
                    .as_deref(),
                Some("1"),
                "public radiance capture CI requires a public RunenGPU adapter"
            );
            None
        }
        Err(error) => panic!("unexpected public capture RunenGPU context failure: {error}"),
    }
}

fn build_scene_and_request() -> (
    engine::plugins::render::scene::RenderSceneSnapshot,
    RenderRequest,
    Vec<RenderSurfaceSemanticInputBinding>,
    Vec<RenderRepresentationAvailabilityFact>,
) {
    let mut store = RenderSceneStore::new();

    let plane_object = store.allocate_object_id().expect("plane object id");
    let mut insert_plane = RenderSceneUpdate::new();
    insert_plane.insert_with_state(plane_object, identity_object_state());
    store.commit(insert_plane).expect("insert plane object");

    let representation_id = store
        .allocate_representation_id(plane_object)
        .expect("plane representation id");
    let surface_evidence = RenderSurfaceProtocolEvidence::exact(
        RENDER_SURFACE_QUERY_PROTOCOL_REVISION,
    )
    .expect("surface protocol")
    .with_oriented_surface(
        RenderOrientedSurfaceProtocolEvidence::exact(
            RENDER_ORIENTED_SURFACE_QUERY_PROTOCOL_REVISION,
        )
        .expect("oriented surface protocol"),
    )
    .with_semantic_input_requirement(RenderSurfaceSemanticInputRequirement::current());
    let representation = RenderRepresentationRecord::new(
        representation_id,
        RenderSpatialCoverage::unbounded(),
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        Some(surface_evidence),
        None,
    )
    .expect("plane representation");
    let material = RenderMaterialAssignment::new(
        RenderDiffuseMaterial::new(0.5).expect("diffuse material"),
    );
    let plane_participation = RenderObjectParticipation::new(
        vec![representation],
        Some(material),
        None,
    )
    .expect("plane participation");
    let mut attach_plane = RenderSceneUpdate::new();
    attach_plane.replace_participation(plane_object, plane_participation);
    store.commit(attach_plane).expect("attach plane participation");

    let emitter_object = store.allocate_object_id().expect("emitter object id");
    let mut insert_emitter = RenderSceneUpdate::new();
    insert_emitter.insert(emitter_object);
    store.commit(insert_emitter).expect("insert emitter object");
    let emitter = RenderDirectionalEmitter::new(
        [0.0, 0.0, 1.0],
        WAVELENGTH_METERS,
        2.0 * std::f64::consts::PI,
    )
    .expect("directional emitter");
    let emitter_participation = RenderObjectParticipation::new(vec![], None, Some(emitter))
        .expect("emitter participation");
    let mut attach_emitter = RenderSceneUpdate::new();
    attach_emitter.replace_participation(emitter_object, emitter_participation);
    store
        .commit(attach_emitter)
        .expect("attach emitter participation");

    let shutter = instant();
    let observation = RenderObservationSpec::Perspective(
        RenderPerspectiveObservation::new(
            RenderAffineTransform3::identity(),
            std::f64::consts::FRAC_PI_3,
            1.0,
            shutter,
            RenderSamplingSupport::ideal_ray(),
        )
        .expect("perspective observation"),
    );
    let lattice = || {
        RenderResultTopology::sample_lattice_2d(WIDTH, HEIGHT).expect("2x2 sample lattice")
    };
    let radiometric = RenderRadiometricRepresentation::spectral_at_wavelength_meters(
        WAVELENGTH_METERS,
    )
    .expect("spectral radiance representation");
    let request = RenderRequest::new(
        shutter,
        vec![observation],
        vec![
            RenderRequestedOutput::new(
                0,
                RenderOutputSpec::new(
                    RenderOutputValue::Radiance {
                        representation: radiometric,
                    },
                    lattice(),
                    RenderSemanticTolerance::absolute(f64::from(NUMERIC_TOLERANCE))
                        .expect("positive radiance tolerance"),
                )
                .expect("radiance output"),
            ),
            RenderRequestedOutput::new(
                0,
                RenderOutputSpec::new(
                    RenderOutputValue::ObjectIdentity,
                    lattice(),
                    RenderSemanticTolerance::exact(),
                )
                .expect("identity output"),
            ),
        ],
    )
    .expect("public capture request");

    let semantic_inputs = vec![RenderSurfaceSemanticInputBinding::new(
        representation_id,
        RenderSurfaceSemanticInput::plane(
            [0.0, 0.0, -3.0],
            [0.0, 0.0, 1.0],
            RenderTemporalSupport::unbounded(),
        )
        .expect("plane semantic input"),
    )];
    let availability = vec![RenderRepresentationAvailabilityFact::new(
        representation_id,
        RenderRepresentationAvailabilityState::Available,
    )];

    (store.snapshot(), request, semantic_inputs, availability)
}

fn allocate_output_texture(
    allocator: &mut GpuWorkResourceIdAllocator,
    label: &str,
) -> GpuTextureHandle {
    allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::ordinary_owned_2d(
                label,
                GpuResourceLifetime::Retained,
                GpuReconstruction::SourceBacked,
                WIDTH,
                HEIGHT,
                GpuTextureFormat::R32Uint,
                [GpuTextureUsage::CopyDestination, GpuTextureUsage::CopySource],
                GpuTextureInitialization::Uninitialized,
            )
            .expect("retained public capture texture descriptor"),
        )
        .expect("retained public capture texture handle")
}

fn wait_for_verified_result(
    context: &GpuContext,
    submitted: &mut SubmittedDeterministicRender,
) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        context.progress();
        match submitted.try_form_verified_result() {
            Ok(Some(_result)) => return,
            Ok(None) => {}
            Err(error) => panic!("verified result formation failed: {error}"),
        }
        if let GpuSubmissionStatus::Failed(failure) = submitted.submission_status() {
            panic!("verified renderer submission failed: {failure:?}");
        }
        assert!(
            Instant::now() < deadline,
            "verified result did not form before timeout"
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
            panic!("public capture readback submission failed: {failure:?}");
        }
        let mut all_ready = true;
        for id in readback_ids {
            match submission.readback(*id).expect("accepted readback identity").status() {
                GpuReadbackStatus::Ready(_) => {}
                GpuReadbackStatus::Pending => all_ready = false,
                GpuReadbackStatus::Failed(failure) => {
                    panic!("public capture readback failed: {failure:?}")
                }
            }
        }
        if all_ready && matches!(submission.status(), GpuSubmissionStatus::Completed) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "public capture readbacks did not complete before timeout"
        );
        std::thread::yield_now();
    }
}

#[test]
fn downstream_public_consumer_correlates_and_interprets_maintained_radiance() {
    let Some(context) = request_context() else {
        return;
    };
    let (scene, request, semantic_inputs, availability) = build_scene_and_request();

    let mut allocator = GpuWorkResourceIdAllocator::new();
    let radiance_texture = allocate_output_texture(&mut allocator, "public retained radiance");
    let identity_texture = allocate_output_texture(&mut allocator, "public retained identity");
    let bindings = [
        RenderOutputBinding::new(
            0,
            RenderOutputDestination::SampleLatticeTexture(radiance_texture.clone()),
        ),
        RenderOutputBinding::new(
            1,
            RenderOutputDestination::SampleLatticeTexture(identity_texture.clone()),
        ),
    ];

    let admitted = admit_deterministic_render(
        &scene,
        &request,
        &semantic_inputs,
        &availability,
        &bindings,
        &context,
    )
    .expect("public founding-direct radiance must admit");
    let mut submitted = pollster::block_on(submit_deterministic_render_for_verified_result(
        admitted,
        &context,
    ))
    .expect("public verified deterministic radiance must submit");
    wait_for_verified_result(&context, &mut submitted);

    let radiance_continuity = context
        .retained_resource_continuity(radiance_texture.diagnostic_identity())
        .expect("retained radiance continuity after completed renderer write");
    let identity_continuity = context
        .retained_resource_continuity(identity_texture.diagnostic_identity())
        .expect("retained identity continuity after completed renderer write");

    let radiance_operation = GpuReadbackOperation::ordinary(
        GpuTextureCopyRegion::whole_base_mip(&radiance_texture)
            .expect("whole radiance texture")
            .into(),
    )
    .expect("public radiance readback operation");
    let radiance_readback_id = radiance_operation.id();
    let identity_operation = GpuReadbackOperation::ordinary(
        GpuTextureCopyRegion::whole_base_mip(&identity_texture)
            .expect("whole identity texture")
            .into(),
    )
    .expect("public identity readback operation");
    let identity_readback_id = identity_operation.id();
    let fragment = GpuWorkFragment::build("RunenRender public capture readbacks", |work| {
        work.operation("read radiance", radiance_operation)?;
        work.operation("read identity", identity_operation)?;
        Ok(())
    })
    .expect("public capture readback fragment");
    let readback_submission = pollster::block_on(context.submit_work(
        "RunenRender public capture readback submission",
        [fragment],
    ))
    .expect("public capture readback submission");
    wait_for_readbacks(
        &context,
        &readback_submission,
        &[radiance_readback_id, identity_readback_id],
    );
    let radiance_readback = readback_submission
        .readback(radiance_readback_id)
        .expect("radiance readback handle");
    let identity_readback = readback_submission
        .readback(identity_readback_id)
        .expect("identity readback handle");

    let captured = submitted
        .interpret_captured_radiance(0, &radiance_continuity, radiance_readback)
        .expect("RunenRender must own maintained radiance carrier interpretation");
    assert_eq!(captured.output_index(), 0);
    assert_eq!(captured.topology(), request.outputs()[0].spec().topology());
    assert_eq!(
        captured.representation(),
        RenderRadiometricRepresentation::spectral_at_wavelength_meters(WAVELENGTH_METERS)
            .expect("same spectral representation")
    );
    assert_eq!(captured.samples().len(), (WIDTH * HEIGHT) as usize);
    for sample in captured.samples() {
        assert!(sample.is_finite());
        assert!(
            (*sample - EXPECTED_RADIANCE).abs() <= NUMERIC_TOLERANCE,
            "unexpected maintained radiance sample: {sample}"
        );
    }

    assert_eq!(
        submitted.interpret_captured_radiance(
            1,
            &identity_continuity,
            identity_readback,
        ),
        Err(RenderDeterministicRadianceCaptureError::OutputNotRadiance { output_index: 1 })
    );
    assert_eq!(
        submitted.interpret_captured_radiance(
            0,
            &radiance_continuity,
            identity_readback,
        ),
        Err(RenderDeterministicRadianceCaptureError::ReadbackSourceMismatch { output_index: 0 })
    );
    assert_eq!(
        submitted.interpret_captured_radiance(
            0,
            &identity_continuity,
            radiance_readback,
        ),
        Err(RenderDeterministicRadianceCaptureError::ContinuityResourceMismatch { output_index: 0 })
    );
}
'''
Path("engine/tests/runenrender_radiance_capture_public.rs").write_text(textwrap.dedent(TEST).lstrip())

ci = Path(".github/workflows/ci.yml")
ci_text = ci.read_text()
marker = "Run public deterministic radiance capture proof"
if marker in ci_text:
    raise SystemExit("capture proof CI step already exists unexpectedly")
ci_text = ci_text.rstrip() + r'''

      - name: Run public deterministic radiance capture proof
        env:
          RUST_BACKTRACE: '1'
          RUNENRENDER_CAPTURE_REQUIRE_GPU: '1'
        run: |
          set -euo pipefail
          cargo +stable test -p engine \
            --test runenrender_radiance_capture_public \
            --locked -- --nocapture --test-threads=1
''' + "\n"
ci.write_text(ci_text)
