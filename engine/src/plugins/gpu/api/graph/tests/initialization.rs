use super::support::*;

#[test]
fn descriptor_and_partial_write_initialization_are_region_aware() {
    let mut allocator = allocator();
    let zeroed = buffer(
        &mut allocator,
        "zeroed",
        GpuBufferInitialization::Zeroed,
        [GpuBufferUsage::Storage],
    );
    let zeroed_range = GpuBufferRange::whole(&zeroed).unwrap();
    let mut readable = builder("descriptor initialized");
    readable
        .declare_resource(GpuResourceRef::Buffer(zeroed.clone()))
        .unwrap();
    add_compute(
        &mut readable,
        "read",
        [buffer_access(
            &zeroed,
            zeroed_range,
            GpuBufferAccessKind::StorageRead,
        )],
    );
    assert!(
        GpuPreparedWorkGraph::prepare(label("descriptor graph"), [readable.finish().unwrap()])
            .is_ok()
    );

    let prepared_data = PreparedGpuData::<TransferData>::from_pod_transfer(
        "prepared buffer bytes",
        &[0_u8; 64],
        provenance("prepared buffer bytes"),
    )
    .unwrap();
    let prepared = buffer(
        &mut allocator,
        "prepared",
        GpuBufferInitialization::Prepared(prepared_data),
        [GpuBufferUsage::Storage, GpuBufferUsage::CopyDestination],
    );
    let mut readable = builder("prepared descriptor initialized");
    readable
        .declare_resource(GpuResourceRef::Buffer(prepared.clone()))
        .unwrap();
    add_compute(
        &mut readable,
        "read prepared",
        [buffer_access(
            &prepared,
            GpuBufferRange::whole(&prepared).unwrap(),
            GpuBufferAccessKind::StorageRead,
        )],
    );
    assert!(
        GpuPreparedWorkGraph::prepare(
            label("prepared descriptor graph"),
            [readable.finish().unwrap()]
        )
        .is_ok()
    );

    let uninitialized = buffer(
        &mut allocator,
        "uninitialized",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::Storage],
    );
    let written = GpuBufferRange::new(&uninitialized, 0, 16).unwrap();
    let disjoint = GpuBufferRange::new(&uninitialized, 32, 16).unwrap();
    let mut partial = builder("partial");
    partial
        .declare_resource(GpuResourceRef::Buffer(uninitialized.clone()))
        .unwrap();
    add_compute(
        &mut partial,
        "write",
        [buffer_access(
            &uninitialized,
            written,
            GpuBufferAccessKind::StorageWrite,
        )],
    );
    add_compute(
        &mut partial,
        "read written",
        [buffer_access(
            &uninitialized,
            written,
            GpuBufferAccessKind::StorageRead,
        )],
    );
    add_compute(
        &mut partial,
        "read disjoint",
        [buffer_access(
            &uninitialized,
            disjoint,
            GpuBufferAccessKind::StorageRead,
        )],
    );
    let error = GpuPreparedWorkGraph::prepare(label("partial graph"), [partial.finish().unwrap()])
        .unwrap_err();
    assert_eq!(error.cause(), GpuWorkGraphCause::ReadBeforeInitialization);
    assert_eq!(error.resource(), Some(uninitialized.diagnostic_identity()));
}

#[test]
fn texture_descriptor_initialization_distinguishes_zeroed_and_prepared_coverage() {
    let mut allocator = allocator();
    let zeroed = texture(
        &mut allocator,
        "zeroed texture",
        GpuTextureInitialization::Zeroed,
        2,
        1,
        [GpuTextureUsage::Sampled, GpuTextureUsage::CopyDestination],
    );
    let prepared = texture(
        &mut allocator,
        "prepared texture",
        prepared_texture_initialization("prepared texture"),
        2,
        1,
        [GpuTextureUsage::Sampled, GpuTextureUsage::CopyDestination],
    );
    let prepared_base = GpuTextureSubresourceRange::new(
        prepared.descriptor().common().label(),
        0,
        1,
        0,
        1,
        GpuTextureAspect::Color,
    )
    .unwrap();
    let mut fragment = builder("initialized textures");
    for resource in [&zeroed, &prepared] {
        fragment
            .declare_resource(GpuResourceRef::Texture(resource.clone()))
            .unwrap();
    }
    add_compute(
        &mut fragment,
        "read zeroed",
        [texture_access(
            &zeroed,
            GpuTextureSubresourceRange::whole(&zeroed).unwrap(),
            GpuTextureAccessKind::SampledRead,
        )],
    );
    add_compute(
        &mut fragment,
        "read prepared base",
        [texture_access(
            &prepared,
            prepared_base,
            GpuTextureAccessKind::SampledRead,
        )],
    );
    let graph = GpuPreparedWorkGraph::prepare(
        label("initialized texture graph"),
        [fragment.finish().unwrap()],
    )
    .unwrap();
    let initial_mip_count = |identity| {
        graph
            .initialization()
            .iter()
            .find(|summary| summary.resource().diagnostic_identity() == identity)
            .unwrap()
            .initial()
            .unwrap()
            .texture_subresource_values()
            .unwrap()
            .len()
    };
    assert_eq!(initial_mip_count(zeroed.diagnostic_identity()), 2);
    assert_eq!(initial_mip_count(prepared.diagnostic_identity()), 1);
}

#[test]
fn texture_reads_reject_uninitialized_or_unprepared_mips() {
    let mut allocator = allocator();
    let prepared = texture(
        &mut allocator,
        "partially prepared texture",
        prepared_texture_initialization("partially prepared texture"),
        2,
        1,
        [GpuTextureUsage::Sampled, GpuTextureUsage::CopyDestination],
    );
    let uninitialized = texture(
        &mut allocator,
        "uninitialized texture",
        GpuTextureInitialization::Uninitialized,
        1,
        1,
        [GpuTextureUsage::Sampled],
    );
    let prepared_mip_one = GpuTextureSubresourceRange::new(
        prepared.descriptor().common().label(),
        1,
        1,
        0,
        1,
        GpuTextureAspect::Color,
    )
    .unwrap();
    for (name, texture, range) in [
        ("unprepared mip", prepared, prepared_mip_one),
        (
            "uninitialized texture",
            uninitialized.clone(),
            GpuTextureSubresourceRange::whole(&uninitialized).unwrap(),
        ),
    ] {
        let mut fragment = builder(name);
        fragment
            .declare_resource(GpuResourceRef::Texture(texture.clone()))
            .unwrap();
        add_compute(
            &mut fragment,
            "invalid read",
            [texture_access(
                &texture,
                range,
                GpuTextureAccessKind::SampledRead,
            )],
        );
        assert_eq!(
            GpuPreparedWorkGraph::prepare(label(name), [fragment.finish().unwrap()])
                .unwrap_err()
                .cause(),
            GpuWorkGraphCause::ReadBeforeInitialization
        );
    }
}

#[test]
fn attachment_store_preserves_and_discard_invalidates_exact_coverage() {
    let mut allocator = allocator();
    let texture = texture(
        &mut allocator,
        "attachment",
        GpuTextureInitialization::Uninitialized,
        1,
        1,
        [GpuTextureUsage::ColorAttachment, GpuTextureUsage::Sampled],
    );
    let range = GpuTextureSubresourceRange::whole(&texture).unwrap();
    let render = |store| {
        GpuWorkOperation::Render(
            GpuRenderOperation::new(
                [GpuRenderColorAttachment::new(
                    GpuTextureAccessResource::Texture(texture.clone()),
                    range,
                    GpuColorAttachmentLoad::Clear(
                        GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
                    ),
                    store,
                    None,
                )
                .unwrap()],
                None,
                [],
                [],
            )
            .unwrap(),
        )
    };
    let sampled = || {
        GpuResourceAccess::Texture(
            GpuTextureAccess::new(
                GpuTextureAccessResource::Texture(texture.clone()),
                range,
                GpuTextureAccessKind::SampledRead,
            )
            .unwrap(),
        )
    };

    let mut stored = builder("stored");
    stored
        .declare_resource(GpuResourceRef::Texture(texture.clone()))
        .unwrap();
    stored
        .add_node(
            label("clear store"),
            render(GpuAttachmentStore::Store),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::GraphicsRequired,
            provenance("clear store"),
        )
        .unwrap();
    add_compute(&mut stored, "sample", [sampled()]);
    assert!(
        GpuPreparedWorkGraph::prepare(label("stored graph"), [stored.finish().unwrap()]).is_ok()
    );

    let mut discarded = builder("discarded");
    discarded
        .declare_resource(GpuResourceRef::Texture(texture.clone()))
        .unwrap();
    discarded
        .add_node(
            label("clear discard"),
            render(GpuAttachmentStore::Discard),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::GraphicsRequired,
            provenance("clear discard"),
        )
        .unwrap();
    add_compute(&mut discarded, "sample", [sampled()]);
    let error =
        GpuPreparedWorkGraph::prepare(label("discard graph"), [discarded.finish().unwrap()])
            .unwrap_err();
    assert_eq!(error.cause(), GpuWorkGraphCause::ReadBeforeInitialization);
}

#[test]
fn depth_attachment_load_clear_store_and_discard_drive_initialization() {
    let mut allocator = allocator();
    let depth = depth_texture(&mut allocator, "depth attachment");
    let range = GpuTextureSubresourceRange::whole(&depth).unwrap();
    let render = |load, store, draws: Vec<GpuDrawIntent>| {
        GpuWorkOperation::Render(
            GpuRenderOperation::new(
                [],
                Some(
                    GpuRenderDepthStencilAttachment::new(
                        GpuTextureAccessResource::Texture(depth.clone()),
                        range,
                        GpuDepthStencilAccess::ReadWrite,
                        load,
                        store,
                    )
                    .unwrap(),
                ),
                draws,
                [],
            )
            .unwrap(),
        )
    };
    let sampled = || texture_access(&depth, range, GpuTextureAccessKind::SampledRead);
    let clear = GpuDepthAttachmentLoad::Clear(GpuDepthClearValue::new(0.5).unwrap());
    for (name, store, succeeds) in [
        ("stored depth", GpuAttachmentStore::Store, true),
        ("discarded depth", GpuAttachmentStore::Discard, false),
    ] {
        let mut fragment = builder(name);
        fragment
            .declare_resource(GpuResourceRef::Texture(depth.clone()))
            .unwrap();
        fragment
            .add_node(
                label("clear depth"),
                render(clear, store, Vec::new()),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::GraphicsRequired,
                provenance("clear depth"),
            )
            .unwrap();
        add_compute(&mut fragment, "sample depth", [sampled()]);
        let result = GpuPreparedWorkGraph::prepare(label(name), [fragment.finish().unwrap()]);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert_eq!(
                result.unwrap_err().cause(),
                GpuWorkGraphCause::ReadBeforeInitialization
            );
        }
    }

    let mut load = builder("load depth");
    load.declare_resource(GpuResourceRef::Texture(depth.clone()))
        .unwrap();
    let draw = GpuDrawIntent::direct(
        GpuDrawRange::new(0, 3).unwrap(),
        GpuDrawRange::new(0, 1).unwrap(),
    );
    load.add_node(
        label("load depth"),
        render(
            GpuDepthAttachmentLoad::Load,
            GpuAttachmentStore::Store,
            vec![draw],
        ),
        [],
        GpuCapabilityRequirements::new(),
        GpuExecutionPreference::GraphicsRequired,
        provenance("load depth"),
    )
    .unwrap();
    assert_eq!(
        GpuPreparedWorkGraph::prepare(label("load depth graph"), [load.finish().unwrap()])
            .unwrap_err()
            .cause(),
        GpuWorkGraphCause::ReadBeforeInitialization
    );
}

#[test]
fn timestamp_resolve_and_copy_form_one_initialized_dependency_chain() {
    let mut allocator = allocator();
    let queries = allocator
        .allocate_query_set_handle(
            GpuQuerySetDescriptor::new(common("timestamps"), GpuQueryKind::Timestamp, 2).unwrap(),
        )
        .unwrap();
    let resolve = buffer(
        &mut allocator,
        "resolve",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
    );
    let readback = buffer(
        &mut allocator,
        "readback",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::CopyDestination],
    );
    let query_range = GpuQueryRange::whole(&queries).unwrap();
    let timestamp_access =
        GpuQueryAccess::new(&queries, query_range, GpuQueryAccessKind::WriteTimestamp).unwrap();
    let render = GpuWorkOperation::Render(
        GpuRenderOperation::new([], None, [], [timestamp_access]).unwrap(),
    );
    let query_resolve = GpuQueryResolveOperation::new(&queries, query_range, &resolve, 0).unwrap();
    let resolve_range = query_resolve.destination_range();
    let mut unresolved = builder("unresolved timing");
    for resource in [
        GpuResourceRef::QuerySet(queries.clone()),
        GpuResourceRef::Buffer(resolve.clone()),
    ] {
        unresolved.declare_resource(resource).unwrap();
    }
    unresolved
        .add_node(
            label("resolve unwritten timestamps"),
            GpuWorkOperation::Resolve(query_resolve.clone()),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance("resolve unwritten timestamps"),
        )
        .unwrap();
    assert_eq!(
        GpuPreparedWorkGraph::prepare(
            label("unresolved timing graph"),
            [unresolved.finish().unwrap()],
        )
        .unwrap_err()
        .cause(),
        GpuWorkGraphCause::ReadBeforeInitialization
    );
    let copy = GpuCopyOperation::buffer_to_buffer(
        GpuBufferRegion::new(&resolve, resolve_range).unwrap(),
        GpuBufferRegion::new(
            &readback,
            GpuBufferRange::new(&readback, 0, resolve_range.size()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let mut fragment = builder("timing");
    for resource in [
        GpuResourceRef::QuerySet(queries),
        GpuResourceRef::Buffer(resolve),
        GpuResourceRef::Buffer(readback),
    ] {
        fragment.declare_resource(resource).unwrap();
    }
    fragment
        .add_node(
            label("write timestamps"),
            render,
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::GraphicsRequired,
            provenance("write timestamps"),
        )
        .unwrap();
    fragment
        .add_node(
            label("resolve timestamps"),
            GpuWorkOperation::Resolve(query_resolve),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance("resolve timestamps"),
        )
        .unwrap();
    fragment
        .add_node(
            label("copy readback"),
            GpuWorkOperation::Copy(copy),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance("copy readback"),
        )
        .unwrap();
    let fragment = fragment.finish().unwrap();
    let graph = GpuPreparedWorkGraph::prepare(label("timing graph"), [fragment.clone()]).unwrap();
    let repeated = GpuPreparedWorkGraph::prepare(label("timing graph"), [fragment]).unwrap();
    assert_eq!(graph.nodes(), repeated.nodes());
    assert_eq!(graph.topological_order(), repeated.topological_order());
    assert_eq!(graph.dependencies(), repeated.dependencies());
    assert_eq!(graph.initialization(), repeated.initialization());
    assert_eq!(graph.requirements(), repeated.requirements());
    assert_eq!(graph.outputs(), repeated.outputs());
    assert_eq!(graph.diagnostics(), repeated.diagnostics());
    assert_eq!(
        graph
            .nodes()
            .iter()
            .map(GpuPreparedWorkNode::id)
            .collect::<Vec<_>>(),
        graph.topological_order()
    );
    assert_eq!(
        graph
            .nodes()
            .iter()
            .map(|node| node.node().label().as_str())
            .collect::<Vec<_>>(),
        vec!["write timestamps", "resolve timestamps", "copy readback"]
    );
    assert!(graph.nodes().iter().all(|node| {
        node.node()
            .accesses()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    }));
    assert!(graph.dependencies().iter().any(|dependency| {
        dependency.before().local_node() == 1 && dependency.after().local_node() == 2
    }));
    assert!(graph.dependencies().iter().any(|dependency| {
        dependency.before().local_node() == 2 && dependency.after().local_node() == 3
    }));
    assert_eq!(
        graph
            .requirements()
            .get(GpuCapabilityFeature::TimestampQuery),
        Some(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::TimestampQuery
        ))
    );
    assert_eq!(
        graph
            .requirements()
            .iter()
            .map(GpuCapabilityRequirement::feature)
            .collect::<Vec<_>>(),
        vec![
            GpuCapabilityFeature::RenderPipeline,
            GpuCapabilityFeature::Copy,
            GpuCapabilityFeature::TimestampQuery,
        ]
    );
}

#[test]
fn buffer_zero_initializes_only_its_checked_range() {
    let mut allocator = allocator();
    let buffer = buffer(
        &mut allocator,
        "zero region",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::CopyDestination, GpuBufferUsage::Storage],
    );
    let zeroed = GpuBufferRange::new(&buffer, 8, 16).unwrap();
    let clear =
        GpuClearOperation::buffer_zero(GpuBufferRegion::new(&buffer, zeroed).unwrap()).unwrap();
    let mut fragment = builder("buffer zero");
    fragment
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    fragment
        .add_node(
            label("zero"),
            GpuWorkOperation::Clear(clear),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance("zero"),
        )
        .unwrap();
    add_compute(
        &mut fragment,
        "read zeroed",
        [buffer_access(
            &buffer,
            zeroed,
            GpuBufferAccessKind::StorageRead,
        )],
    );
    let graph =
        GpuPreparedWorkGraph::prepare(label("buffer zero graph"), [fragment.finish().unwrap()])
            .unwrap();
    let summary = graph
        .initialization()
        .iter()
        .find(|summary| summary.resource().diagnostic_identity() == buffer.diagnostic_identity())
        .unwrap();
    let final_ranges = summary
        .final_coverage()
        .unwrap()
        .buffer_range_values()
        .unwrap();
    assert_eq!(final_ranges, [zeroed]);
}

#[test]
fn explicit_graph_entry_input_initializes_only_declared_coverage() {
    let mut allocator = allocator();
    let imported_label = label("imported retained");
    let retained_label = label("owned retained");
    let commons = [
        GpuResourceCommon::imported(
            imported_label.clone(),
            GpuResourceLifetime::Retained,
            provenance("imported retained"),
        ),
        GpuResourceCommon::owned(
            retained_label.clone(),
            GpuResourceLifetime::Retained,
            GpuMemoryIntent::Device,
            GpuReconstruction::SourceBacked,
            provenance("owned retained"),
        )
        .unwrap(),
    ];
    for common in commons {
        let resource_label = common.label().clone();
        let resource = allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common,
                    64,
                    GpuBufferUsages::new(&resource_label, [GpuBufferUsage::Storage]).unwrap(),
                    GpuBufferInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        let initialized = GpuBufferRange::new(&resource, 16, 16).unwrap();
        let input = GpuWorkResourceInput::new(
            GpuResourceRef::Buffer(resource.clone()),
            GpuInitialCoverage::buffer_ranges(&resource, [initialized]).unwrap(),
            provenance("external input"),
        )
        .unwrap();
        let prepare_read = |name, read_range| {
            let mut fragment = builder(name);
            fragment
                .declare_resource(GpuResourceRef::Buffer(resource.clone()))
                .unwrap();
            fragment.add_input(input.clone()).unwrap();
            add_compute(
                &mut fragment,
                "read input",
                [buffer_access(
                    &resource,
                    read_range,
                    GpuBufferAccessKind::StorageRead,
                )],
            );
            GpuPreparedWorkGraph::prepare(label(name), [fragment.finish().unwrap()])
        };
        assert!(prepare_read("initialized input", initialized).is_ok());
        assert_eq!(
            prepare_read(
                "uninitialized input",
                GpuBufferRange::new(&resource, 0, 16).unwrap(),
            )
            .unwrap_err()
            .cause(),
            GpuWorkGraphCause::ReadBeforeInitialization
        );
    }
}

#[test]
fn output_access_and_coverage_mismatches_are_structured() {
    let mut allocator = allocator();
    let resource = buffer(
        &mut allocator,
        "mismatch",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::Storage],
    );
    let written = GpuBufferRange::new(&resource, 0, 16).unwrap();
    let overstated = GpuBufferRange::new(&resource, 0, 32).unwrap();
    let make_fragment = |intent, coverage_range| {
        let mut fragment = builder("mismatch producer");
        fragment
            .declare_resource(GpuResourceRef::Buffer(resource.clone()))
            .unwrap();
        add_compute(
            &mut fragment,
            "write",
            [buffer_access(
                &resource,
                written,
                GpuBufferAccessKind::StorageWrite,
            )],
        );
        fragment
            .add_output(
                GpuWorkOutput::new(
                    GpuExportRelationship::new(
                        GpuResourceRef::Buffer(resource.clone()),
                        GpuExportKey::new("mismatch.output").unwrap(),
                        intent,
                        provenance("mismatch output"),
                    ),
                    GpuInitialCoverage::buffer_ranges(&resource, [coverage_range]).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        fragment.finish().unwrap()
    };
    assert_eq!(
        GpuPreparedWorkGraph::prepare(
            label("intent mismatch"),
            [make_fragment(GpuResourceAccessIntent::Read, written)]
        )
        .unwrap_err()
        .cause(),
        GpuWorkGraphCause::ImportExportMismatch
    );
    assert_eq!(
        GpuPreparedWorkGraph::prepare(
            label("coverage mismatch"),
            [make_fragment(GpuResourceAccessIntent::Write, overstated)]
        )
        .unwrap_err()
        .cause(),
        GpuWorkGraphCause::ImportExportMismatch
    );
}

#[test]
fn multisample_resolve_initializes_destination_despite_source_discard() {
    let mut allocator = allocator();
    let resource_label = label("msaa");
    let source = allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::new(
                common("msaa"),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 8, 8, 1).unwrap(),
                1,
                4,
                GpuTextureFormat::Rgba8Unorm,
                GpuTextureUsages::new(&resource_label, [GpuTextureUsage::ColorAttachment]).unwrap(),
                GpuTextureInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap();
    let destination = texture(
        &mut allocator,
        "resolved",
        GpuTextureInitialization::Uninitialized,
        1,
        1,
        [GpuTextureUsage::ColorAttachment, GpuTextureUsage::Sampled],
    );
    let source_range = GpuTextureSubresourceRange::whole(&source).unwrap();
    let destination_range = GpuTextureSubresourceRange::whole(&destination).unwrap();
    let resolve_target = GpuMultisampleResolveTarget::new(
        GpuTextureAccessResource::Texture(destination.clone()),
        destination_range,
    )
    .unwrap();
    let attachment = GpuRenderColorAttachment::new(
        GpuTextureAccessResource::Texture(source.clone()),
        source_range,
        GpuColorAttachmentLoad::Clear(GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap()),
        GpuAttachmentStore::Discard,
        Some(resolve_target),
    )
    .unwrap();
    let mut fragment = builder("resolve");
    for resource in [
        GpuResourceRef::Texture(source),
        GpuResourceRef::Texture(destination.clone()),
    ] {
        fragment.declare_resource(resource).unwrap();
    }
    fragment
        .add_node(
            label("render resolve"),
            GpuWorkOperation::Render(GpuRenderOperation::new([attachment], None, [], []).unwrap()),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::GraphicsRequired,
            provenance("render resolve"),
        )
        .unwrap();
    add_compute(
        &mut fragment,
        "sample resolve",
        [GpuResourceAccess::Texture(
            GpuTextureAccess::new(
                GpuTextureAccessResource::Texture(destination),
                destination_range,
                GpuTextureAccessKind::SampledRead,
            )
            .unwrap(),
        )],
    );
    assert!(
        GpuPreparedWorkGraph::prepare(label("resolve graph"), [fragment.finish().unwrap()]).is_ok()
    );
}
