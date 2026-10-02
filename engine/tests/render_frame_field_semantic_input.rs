use engine::plugins::render::backend::RenderSurfaceId;
use engine::plugins::render::{
    RenderDeterministicFrameContribution, RenderDeterministicFrameContributionResource,
    RenderDynamicTextureTargetKey, RenderFrameProducerId,
};
use runen_gpu::{
    GpuCapabilityProfile, GpuContext, GpuContextDescriptor, GpuContextRequestErrorCategory,
    GpuFormatRole, GpuReconstruction, GpuResourceLifetime, GpuTextureDescriptor, GpuTextureFormat,
    GpuTextureInitialization, GpuTextureUsage, GpuWorkResourceIdAllocator,
};
use runen_render::admission::{
    RenderOutputBinding, RenderOutputDestination, RenderRepresentationAvailabilityFact,
    RenderRepresentationAvailabilityState,
};
use runen_render::admit_render;
use runen_render::field_input::{
    RenderFieldSemanticInput, RenderFieldSemanticInputBinding, RenderFieldSemanticInputRequirement,
};
use runen_render::participation::RenderObjectParticipation;
use runen_render::representation::{
    RENDER_FIELD_DISTANCE_PROTOCOL_REVISION, RENDER_SURFACE_QUERY_PROTOCOL_REVISION,
    RenderFieldDistanceGuarantee, RenderFieldDistanceProtocolEvidence, RenderRefinementEvidence,
    RenderRepresentationRecord, RenderSurfaceProtocolEvidence,
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

fn object_state(z: f64) -> RenderObjectState {
    RenderObjectState::new(
        RenderObjectSpatialState::new(
            RenderSpaceSpec::new(1.0, RenderHandedness::Right).expect("metric test object space"),
            RenderAffineTransform3::from_row_major_3x4([
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, z,
            ])
            .expect("finite test transform"),
            RenderSpatialCoverage::unbounded(),
        ),
        RenderObjectTemporalState::new(RenderTemporalSupport::unbounded()),
    )
}

fn test_context() -> Option<GpuContext> {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .require_format_role(GpuTextureFormat::R32Uint, GpuFormatRole::CopyDestination)
            .with_label("GP1B4 native-frame field-input carrier proof");
    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => Some(context),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENRENDER_R7_REQUIRE_GPU").ok().as_deref(),
                Some("1"),
                "GP1B4 Vulkan proof requires a public RunenGPU adapter"
            );
            None
        }
        Err(error) => panic!("unexpected GP1B4 RunenGPU context failure: {error}"),
    }
}

#[test]
fn deterministic_frame_contribution_carries_surface_and_field_inputs_into_ordinary_admission() {
    let Some(context) = test_context() else {
        return;
    };

    let mut scene = RenderSceneStore::new();

    let field_object = scene.allocate_object_id().expect("field object id");
    let mut insert_field = RenderSceneUpdate::new();
    insert_field.insert_with_state(field_object, object_state(-3.0));
    scene.commit(insert_field).expect("insert field object");
    let field_representation = scene
        .allocate_representation_id(field_object)
        .expect("field representation id");
    let field_protocol = RenderFieldDistanceProtocolEvidence::new(
        RENDER_FIELD_DISTANCE_PROTOCOL_REVISION,
        RenderFieldDistanceGuarantee::conservative(0.5).expect("field guarantee"),
    )
    .expect("field protocol")
    .with_semantic_input_requirement(RenderFieldSemanticInputRequirement::current());
    let field_record = RenderRepresentationRecord::new(
        field_representation,
        RenderSpatialCoverage::axis_aligned_bounds([-1.0; 3], [1.0; 3]).expect("field coverage"),
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        None,
        Some(field_protocol),
    )
    .expect("field representation");
    let mut attach_field = RenderSceneUpdate::new();
    attach_field.replace_participation(
        field_object,
        RenderObjectParticipation::new(vec![field_record], None, None)
            .expect("field participation"),
    );
    scene
        .commit(attach_field)
        .expect("attach field participation");

    let surface_object = scene.allocate_object_id().expect("surface object id");
    let mut insert_surface = RenderSceneUpdate::new();
    insert_surface.insert_with_state(surface_object, object_state(-6.0));
    scene.commit(insert_surface).expect("insert surface object");
    let surface_representation = scene
        .allocate_representation_id(surface_object)
        .expect("surface representation id");
    let surface_protocol =
        RenderSurfaceProtocolEvidence::exact(RENDER_SURFACE_QUERY_PROTOCOL_REVISION)
            .expect("surface protocol")
            .with_semantic_input_requirement(RenderSurfaceSemanticInputRequirement::current());
    let surface_record = RenderRepresentationRecord::new(
        surface_representation,
        RenderSpatialCoverage::unbounded(),
        RenderTemporalSupport::unbounded(),
        RenderRefinementEvidence::none(),
        Some(surface_protocol),
        None,
    )
    .expect("surface representation");
    let mut attach_surface = RenderSceneUpdate::new();
    attach_surface.replace_participation(
        surface_object,
        RenderObjectParticipation::new(vec![surface_record], None, None)
            .expect("surface participation"),
    );
    scene
        .commit(attach_surface)
        .expect("attach surface participation");

    let shutter =
        RenderTimeInterval::instant(RenderTimePoint::from_seconds(0.0).expect("finite test time"));
    let observation = RenderObservationSpec::Perspective(
        RenderPerspectiveObservation::new(
            RenderAffineTransform3::identity(),
            std::f64::consts::FRAC_PI_3,
            1.0,
            shutter,
            RenderSamplingSupport::ideal_ray(),
        )
        .expect("test perspective"),
    );
    let request = RenderRequest::new(
        shutter,
        vec![observation],
        vec![RenderRequestedOutput::new(
            0,
            RenderOutputSpec::new(
                RenderOutputValue::ObjectIdentity,
                RenderResultTopology::sample_lattice_2d(1, 1).expect("1x1 lattice"),
                RenderSemanticTolerance::exact(),
            )
            .expect("object identity output"),
        )],
    )
    .expect("test request");

    let surface_binding = RenderSurfaceSemanticInputBinding::new(
        surface_representation,
        RenderSurfaceSemanticInput::sphere([0.0; 3], 0.5, RenderTemporalSupport::unbounded())
            .expect("surface sphere input"),
    );
    let field_input = RenderFieldSemanticInput::dense(
        [-1.0; 3],
        [2.0; 3],
        [2, 2, 2],
        vec![0.0; 8],
        0.25,
        RenderTemporalSupport::unbounded(),
    )
    .expect("field semantic input");
    let field_binding = RenderFieldSemanticInputBinding::new(field_representation, field_input);
    let availability = vec![
        RenderRepresentationAvailabilityFact::new(
            surface_representation,
            RenderRepresentationAvailabilityState::Available,
        ),
        RenderRepresentationAvailabilityFact::new(
            field_representation,
            RenderRepresentationAvailabilityState::Available,
        ),
    ];

    let contribution = RenderDeterministicFrameContribution {
        producer_id: RenderFrameProducerId::try_from_raw(1126).expect("non-zero producer"),
        render_surface_id: RenderSurfaceId::primary(),
        scene: scene.snapshot(),
        request,
        semantic_inputs: vec![surface_binding],
        field_semantic_inputs: vec![field_binding],
        availability,
        output_index: 0,
        target_key: RenderDynamicTextureTargetKey::new("gp1b4", "radiance"),
        finite_evaluation_extent: None,
    };

    let mut carrier = RenderDeterministicFrameContributionResource::default();
    carrier.replace(contribution);
    let retained = carrier.take_all();
    assert_eq!(retained.len(), 1);
    let retained = &retained[0];
    assert_eq!(retained.semantic_inputs.len(), 1);
    assert_eq!(retained.field_semantic_inputs.len(), 1);

    let mut allocator = GpuWorkResourceIdAllocator::new();
    let destination = allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::ordinary_owned_2d(
                "GP1B4 admission output",
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                1,
                1,
                GpuTextureFormat::R32Uint,
                [GpuTextureUsage::CopyDestination],
                GpuTextureInitialization::Uninitialized,
            )
            .expect("output descriptor"),
        )
        .expect("output handle");
    let output_binding = RenderOutputBinding::new(
        0,
        RenderOutputDestination::SampleLatticeTexture(destination),
    );

    assert!(
        admit_render(
            &retained.scene,
            &retained.request,
            &retained.semantic_inputs,
            &[],
            &retained.availability,
            std::slice::from_ref(&output_binding),
            &context,
        )
        .is_err(),
        "field-backed representation must fail closed when its binding is absent"
    );

    admit_render(
        &retained.scene,
        &retained.request,
        &retained.semantic_inputs,
        &retained.field_semantic_inputs,
        &retained.availability,
        std::slice::from_ref(&output_binding),
        &context,
    )
    .expect("one contribution must carry surface and field inputs into ordinary admission");
}
