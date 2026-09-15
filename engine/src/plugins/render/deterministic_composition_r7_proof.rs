//! Focused R7 proof for composing maintained radiance with a consumer-authored graph.
//!
//! This stays at RunenGPU graph preparation: the renderer owns the prepared producer fragment and
//! typed output correlation, while the consumer owns its import and operation.

use super::admission::{RenderOutputBinding, RenderOutputDestination};
use super::deterministic_admission::admit_deterministic_render;
use super::deterministic_execution::prepare_deterministic_render;
use super::deterministic_execution_r7_proof::{
    MaintainedExecutionFixture, admit_with_retained_radiance_destination, maintained_fixture,
};
use runen_gpu::{
    GpuBufferDescriptor, GpuBufferInitialization, GpuBufferTextureLayout, GpuBufferUsage,
    GpuCapabilityProfile, GpuContext, GpuContextDescriptor, GpuContextRequestErrorCategory,
    GpuCopyOperation, GpuDependencyReason, GpuDependencyRegion, GpuFormatRole,
    GpuPreparedWorkGraph, GpuReconstruction, GpuResourceLabel, GpuResourceLifetime,
    GpuResourceProvenance, GpuTextureDescriptor, GpuTextureFormat, GpuTextureInitialization,
    GpuTextureUsage, GpuWorkFragment, GpuWorkNodeKind, GpuWorkResourceIdAllocator,
};

fn request_composition_context() -> Option<GpuContext> {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .require_format_role(GpuTextureFormat::R32Float, GpuFormatRole::CopyDestination)
            .require_format_role(GpuTextureFormat::R32Float, GpuFormatRole::CopySource)
            .with_label("RunenRender R7 composition proof");
    match pollster::block_on(GpuContext::request(descriptor)) {
        Ok(context) => Some(context),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENRENDER_R7_REQUIRE_GPU").ok().as_deref(),
                Some("1"),
                "permanent R7 composition CI requires a public RunenGPU adapter"
            );
            None
        }
        Err(error) => panic!("unexpected R7 composition RunenGPU context failure: {error}"),
    }
}

fn radiance_fixture() -> MaintainedExecutionFixture {
    let mut fixture = maintained_fixture();
    fixture.request = super::request::RenderRequest::new(
        fixture.request.render_interval(),
        fixture.request.observations().to_vec(),
        vec![super::request::RenderRequestedOutput::new(
            0,
            super::request::RenderOutputSpec::new(
                super::request::RenderOutputValue::Radiance {
                    representation: super::request::RenderRadiometricRepresentation::spectral_at_wavelength_meters(
                        550.0e-9,
                    )
                    .expect("R7 composition radiance representation"),
                },
                super::request::RenderResultTopology::sample_lattice_2d(2, 2)
                    .expect("R7 composition radiance topology"),
                super::request::RenderSemanticTolerance::exact(),
            )
            .expect("R7 composition radiance output"),
        )],
    )
    .expect("R7 composition radiance request");
    fixture
}

fn admit_r32float_radiance(
    fixture: &MaintainedExecutionFixture,
    context: &GpuContext,
) -> super::deterministic_admission::AdmittedDeterministicRender {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let destination = allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::ordinary_owned_2d(
                "R7 composition radiance destination",
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
            .expect("R7 composition R32Float destination descriptor"),
        )
        .expect("R7 composition R32Float destination handle");
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
    .expect("R7 composition R32Float radiance must be admitted")
}

#[test]
fn maintained_r32float_radiance_is_a_typed_producer_for_consumer_first_graphs() {
    let Some(context) = request_composition_context() else {
        return;
    };
    let fixture = radiance_fixture();
    let legacy = prepare_deterministic_render(
        admit_with_retained_radiance_destination(&fixture, &context),
        &context,
    )
    .expect("legacy R32Uint radiance preparation must succeed");
    assert!(
        legacy.radiance_outputs().is_empty(),
        "legacy R32Uint radiance remains capture-compatible but is not composable"
    );
    let prepared =
        prepare_deterministic_render(admit_r32float_radiance(&fixture, &context), &context)
            .expect("R7 composition preparation must succeed");

    assert_eq!(prepared.radiance_outputs().len(), 1);
    assert!(
        prepared
            .work_set()
            .fragments()
            .iter()
            .flat_map(|fragment| fragment.nodes())
            .all(|node| node.kind() != GpuWorkNodeKind::Readback)
    );
    let output = prepared
        .radiance_output(0)
        .expect("radiance output correlation");
    let texture = output
        .texture()
        .expect("radiance output must be a texture")
        .clone();
    assert_eq!(texture.descriptor().format(), GpuTextureFormat::R32Float);
    assert_eq!(
        output.export_relationship().required_final_access(),
        runen_gpu::GpuResourceAccessIntent::Write
    );

    let coverage = prepared.work_set().fragments()[0].outputs()[0].final_initialized_coverage();
    let ranges = coverage
        .texture_subresource_values()
        .expect("radiance output must carry texture coverage");
    assert_eq!(ranges.len(), 1);
    assert_eq!(
        ranges[0],
        runen_gpu::GpuTextureCopyRegion::whole_base_mip(&texture)
            .expect("R7 composition coverage region")
            .subresources()
    );

    let mut allocator = GpuWorkResourceIdAllocator::new();
    let consumer_buffer = allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::ordinary_owned(
                "R7 composition consumer buffer",
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                16,
                [GpuBufferUsage::CopyDestination],
                GpuBufferInitialization::Uninitialized,
            )
            .expect("R7 composition consumer buffer descriptor"),
        )
        .expect("R7 composition consumer buffer handle");
    let consumer_operation = GpuCopyOperation::texture_to_buffer(
        runen_gpu::GpuTextureCopyRegion::whole_base_mip(&texture)
            .expect("R7 composition source region"),
        GpuBufferTextureLayout::new(&consumer_buffer, 0, 8, 2)
            .expect("R7 composition consumer layout"),
    )
    .expect("R7 composition consumer copy");
    let consumer_provenance = GpuResourceProvenance::new(
        runen_gpu::GpuResourceLabel::new("R7 composition consumer").expect("consumer label"),
        None,
        None,
    );
    let consumer = GpuWorkFragment::build_with_provenance(
        runen_gpu::GpuResourceLabel::new("R7 composition consumer")
            .expect("consumer fragment label"),
        consumer_provenance.clone(),
        |work| {
            work.operation("consume maintained radiance", consumer_operation)?;
            work.add_import(output.import(consumer_provenance))?;
            Ok(())
        },
    )
    .expect("R7 composition consumer fragment");

    let producer = prepared.work_set().fragments()[0].clone();
    let graph = GpuPreparedWorkGraph::prepare(
        GpuResourceLabel::new("R7 consumer-first composition").expect("graph label"),
        [consumer, producer],
    )
    .expect("R7 consumer-first composition graph");
    assert_eq!(graph.outputs().len(), 1);
    assert!(
        graph
            .dependencies()
            .iter()
            .flat_map(|dependency| dependency.reasons())
            .any(|reason| {
                matches!(
                    reason,
                    GpuDependencyReason::ReadAfterWrite { resource, region }
                        if *resource == output.resource().diagnostic_identity()
                            && *region == GpuDependencyRegion::Texture(
                                runen_gpu::GpuTextureCopyRegion::whole_base_mip(&texture)
                                    .expect("R7 composition dependency region")
                                    .subresources()
                            )
                )
            })
    );
    assert_eq!(
        graph
            .topological_order()
            .first()
            .expect("producer node")
            .fragment_ordinal(),
        1,
        "consumer-first authoring must still prepare the producer before the consumer"
    );
}
