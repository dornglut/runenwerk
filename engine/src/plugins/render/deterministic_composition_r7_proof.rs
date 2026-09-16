//! Focused R7 proof for composing maintained radiance with a consumer-authored graph.
//!
//! This stays at RunenGPU graph preparation: the renderer owns the prepared producer fragment and
//! typed output correlation, while the consumer owns its import and operation.

use super::adapters::{
    RenderGpuWorkOccurrenceId, ResolvedRenderGpuWorkNode,
    prepare_render_gpu_frame_work_with_composition_for_test,
};
use super::admission::{RenderOutputBinding, RenderOutputDestination};
use super::deterministic_admission::admit_deterministic_render;
use super::deterministic_execution::prepare_deterministic_render;
use super::deterministic_execution_r7_proof::{
    MaintainedExecutionFixture, admit_with_retained_radiance_destination, maintained_fixture,
};
use runen_gpu::{
    GpuAttachmentStore, GpuBindingKey, GpuBindingLayoutRefinement, GpuBufferDescriptor,
    GpuBufferInitialization, GpuBufferTextureLayout, GpuBufferUsage, GpuCapabilityProfile,
    GpuColorAttachmentLoad, GpuColorClearValue, GpuColorTargetStateDescriptor, GpuContext,
    GpuContextDescriptor, GpuContextRequestErrorCategory, GpuCopyOperation, GpuDependencyReason,
    GpuDependencyRegion, GpuDrawRange, GpuExecutionPreference, GpuFormatRole,
    GpuFragmentOutputStateDescriptor, GpuMultisampleStateDescriptor, GpuPipelineConfiguration,
    GpuPreparedWorkGraph, GpuPresentOperation, GpuPrimitiveStateDescriptor, GpuProgramDescriptor,
    GpuReconstruction, GpuRenderEntryPoints, GpuRenderOperation, GpuRenderPipelineDescriptor,
    GpuRenderPipelineStateDescriptor, GpuResourceLabel, GpuResourceLifetime, GpuResourceProvenance,
    GpuRuntimeBindingResource, GpuRuntimeBindingValue, GpuRuntimeTextureViewBinding,
    GpuTextureDescriptor, GpuTextureFormat, GpuTextureInitialization, GpuTextureSampleClass,
    GpuTextureUsage, GpuTextureViewDescriptor, GpuTextureViewDimension,
    GpuVertexInputStateDescriptor, GpuWorkFragment, GpuWorkNodeKind, GpuWorkOperation,
    GpuWorkResourceIdAllocator,
};

fn request_composition_context() -> Option<GpuContext> {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .require_format_role(GpuTextureFormat::R32Float, GpuFormatRole::CopyDestination)
            .require_format_role(GpuTextureFormat::R32Float, GpuFormatRole::CopySource)
            .require_format_role(GpuTextureFormat::R32Float, GpuFormatRole::Sampled)
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
                    GpuTextureUsage::Sampled,
                ],
                GpuTextureInitialization::Zeroed,
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

fn surface_color_view(
    allocator: &mut GpuWorkResourceIdAllocator,
) -> runen_gpu::GpuTextureViewHandle {
    let texture = allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::ordinary_owned_2d(
                "RL2 proof surface color",
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                2,
                2,
                GpuTextureFormat::Rgba8Unorm,
                [GpuTextureUsage::ColorAttachment],
                GpuTextureInitialization::Zeroed,
            )
            .expect("RL2 proof surface descriptor"),
        )
        .expect("RL2 proof surface texture");
    allocator
        .allocate_texture_view_handle(
            GpuTextureViewDescriptor::ordinary_full_owned("RL2 proof surface view", &texture)
                .expect("RL2 proof surface view descriptor"),
        )
        .expect("RL2 proof surface view")
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

#[test]
fn rl2_canonical_composition_uses_real_radiance_import_and_visualizer() {
    let Some(context) = request_composition_context() else {
        return;
    };
    let fixture = radiance_fixture();
    let prepared =
        prepare_deterministic_render(admit_r32float_radiance(&fixture, &context), &context)
            .expect("RL2 maintained preparation must succeed");
    let output = prepared
        .radiance_output(0)
        .expect("RL2 maintained radiance output");
    let texture = output
        .texture()
        .expect("RL2 output must be a texture")
        .clone();
    let producer = prepared.work_set().fragments()[0].clone();

    let mut allocator = GpuWorkResourceIdAllocator::new();
    let surface = surface_color_view(&mut allocator);
    let [source] = runen_gpu::admit_static_wgsl_sources([(
        "runenwerk.render_lab.rl2.visualizer",
        1,
        include_str!("../../../../assets/shaders/runenwerk_render_lab_radiance.wgsl"),
    )])
    .expect("actual Render Lab WGSL should be admitted");
    let binding_key = GpuBindingKey::try_new(0, 0).expect("RL2 radiance binding key");
    let program = GpuProgramDescriptor::new(
        source,
        [
            runen_gpu::GpuEntryPointName::new("vs_main").expect("RL2 vertex entry point"),
            runen_gpu::GpuEntryPointName::new("fs_main").expect("RL2 fragment entry point"),
        ],
        [GpuBindingLayoutRefinement::new(binding_key)
            .with_texture_sample_class(GpuTextureSampleClass::FloatUnfilterable)],
    )
    .expect("actual Render Lab WGSL program");
    let pipeline_state = GpuRenderPipelineStateDescriptor::new(
        GpuVertexInputStateDescriptor::new([]).expect("RL2 vertex input state"),
        Some(GpuFragmentOutputStateDescriptor::new([
            GpuColorTargetStateDescriptor::new(
                GpuTextureFormat::Rgba8Unorm,
                runen_gpu::GpuBlendMode::Replace,
                runen_gpu::GpuColorWriteMask::ALL,
            )
            .expect("RL2 color target state"),
        ])),
        GpuPrimitiveStateDescriptor::default(),
        None,
        GpuMultisampleStateDescriptor::default(),
    )
    .expect("actual Render Lab WGSL render state");
    let pipeline = GpuRenderPipelineDescriptor::new(
        program,
        GpuRenderEntryPoints::new(
            runen_gpu::GpuEntryPointName::new("vs_main").expect("RL2 vertex entry point"),
            Some(runen_gpu::GpuEntryPointName::new("fs_main").expect("RL2 fragment entry point")),
        ),
        pipeline_state,
        GpuPipelineConfiguration::default(),
    )
    .expect("actual Render Lab WGSL render pipeline");
    let radiance_view = allocator
        .allocate_texture_view_handle(
            GpuTextureViewDescriptor::ordinary_full_owned("RL2 proof radiance view", &texture)
                .expect("RL2 radiance view descriptor"),
        )
        .expect("RL2 radiance view");
    let bindings = pipeline
        .runtime_bindings([GpuRuntimeBindingValue::new(
            binding_key,
            [GpuRuntimeBindingResource::TextureView(
                GpuRuntimeTextureViewBinding::new(radiance_view, GpuTextureViewDimension::D2),
            )],
        )
        .expect("RL2 visualizer binding value")])
        .expect("RL2 visualizer runtime bindings");
    let visualizer = GpuRenderOperation::ordinary_color_full_target_direct(
        &pipeline,
        bindings,
        &surface,
        GpuColorAttachmentLoad::Clear(
            GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).expect("RL2 clear color"),
        ),
        GpuAttachmentStore::Store,
        GpuDrawRange::new(0, 3).expect("RL2 fullscreen vertices"),
        GpuDrawRange::new(0, 1).expect("RL2 fullscreen instance"),
    )
    .expect("actual Render Lab visualizer operation");
    let visualizer_occurrence = RenderGpuWorkOccurrenceId::new(20);
    let present_occurrence = RenderGpuWorkOccurrenceId::new(21);
    let present = GpuPresentOperation::whole_view(&surface).expect("RL2 terminal Present");
    let visualizer = ResolvedRenderGpuWorkNode::pass(
        visualizer_occurrence,
        GpuResourceLabel::new("RL2 actual Render Lab visualizer").expect("RL2 visualizer label"),
        GpuWorkOperation::Render(visualizer),
        GpuExecutionPreference::GraphicsRequired,
        [],
    );
    let present = ResolvedRenderGpuWorkNode::present(
        present_occurrence,
        GpuResourceLabel::new("RL2 actual terminal Present").expect("RL2 Present label"),
        present,
        [],
    );
    let consumer_provenance = GpuResourceProvenance::new(
        GpuResourceLabel::new("RL2 actual Render Lab visualizer import").expect("RL2 import label"),
        None,
        None,
    );
    let graph = prepare_render_gpu_frame_work_with_composition_for_test(
        GpuResourceLabel::new("RL2 actual canonical composition").expect("RL2 graph label"),
        [visualizer, present],
        &[producer],
        &[output.import(consumer_provenance)],
    )
    .expect("RL2 actual producer/visualizer composition should prepare");
    let node_by_label = |label: &str| {
        graph
            .nodes()
            .iter()
            .find(|node| node.node().label().as_str() == label)
            .expect("RL2 composed node")
    };
    let visualizer_node = node_by_label("RL2 actual Render Lab visualizer").id();
    let present_node = node_by_label("RL2 actual terminal Present").id();
    let radiance_dependency = graph
        .dependencies()
        .iter()
        .find(|dependency| {
            dependency.after() == visualizer_node
                && dependency.reasons().iter().any(|reason| {
                    matches!(
                        reason,
                        GpuDependencyReason::ReadAfterWrite { resource, .. }
                            if *resource == output.resource().diagnostic_identity()
                    )
                })
        })
        .expect("typed maintained radiance must order the producer before the visualizer");
    assert_eq!(radiance_dependency.before().fragment_ordinal(), 0);
    assert_eq!(graph.topological_order().last(), Some(&present_node));
    assert_eq!(
        graph
            .nodes()
            .iter()
            .filter(|node| node.node().kind() == GpuWorkNodeKind::Present)
            .count(),
        1
    );
    assert!(
        graph
            .nodes()
            .iter()
            .all(|node| node.node().kind() != GpuWorkNodeKind::Readback)
    );
}
