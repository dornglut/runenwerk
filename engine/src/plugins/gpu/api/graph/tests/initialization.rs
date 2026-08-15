use super::support::*;

#[test]
fn descriptor_initialization_is_region_aware_and_generic_writes_do_not_initialize() {
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
        [
            GpuTextureUsage::ColorAttachment,
            GpuTextureUsage::CopyDestination,
            GpuTextureUsage::Sampled,
        ],
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
fn generic_timestamp_access_does_not_initialize_query_state() {
    let mut allocator = allocator();
    let queries = allocator
        .allocate_query_set_handle(
            GpuQuerySetDescriptor::new(common("generic timestamps"), GpuQueryKind::Timestamp, 1)
                .unwrap(),
        )
        .unwrap();
    let destination = buffer(
        &mut allocator,
        "generic timestamp resolve",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::QueryResolve],
    );
    let range = GpuQueryRange::whole(&queries).unwrap();
    let resolve = GpuQueryResolveOperation::new(&queries, range, &destination, 0).unwrap();
    let mut fragment = builder("generic timestamp");
    for resource in [
        GpuResourceRef::QuerySet(queries.clone()),
        GpuResourceRef::Buffer(destination),
    ] {
        fragment.declare_resource(resource).unwrap();
    }
    add_compute(
        &mut fragment,
        "generic timestamp write",
        [GpuResourceAccess::Query(
            GpuQueryAccess::new(&queries, range, GpuQueryAccessKind::WriteTimestamp).unwrap(),
        )],
    );
    fragment
        .add_node(
            label("resolve generic timestamp"),
            GpuWorkOperation::Resolve(resolve),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance("resolve generic timestamp"),
        )
        .unwrap();
    assert_eq!(
        GpuPreparedWorkGraph::prepare(
            label("generic timestamp graph"),
            [fragment.finish().unwrap()]
        )
        .unwrap_err()
        .cause(),
        GpuWorkGraphCause::ReadBeforeInitialization
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
    let final_coverage = summary.final_coverage().unwrap().buffer_values().unwrap();
    assert_eq!(final_coverage, [GpuBufferCoverage::dense(zeroed)]);
}

#[test]
fn operation_effects_require_their_checked_write_role() {
    let mut allocator = allocator();
    let buffer = buffer(
        &mut allocator,
        "effect role",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::CopyDestination, GpuBufferUsage::Storage],
    );
    let range = GpuBufferRange::new(&buffer, 0, 16).unwrap();
    let clear =
        GpuClearOperation::buffer_zero(GpuBufferRegion::new(&buffer, range).unwrap()).unwrap();
    let mut fragment = builder("effect role");
    fragment
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    fragment
        .add_node(
            label("clear"),
            GpuWorkOperation::Clear(clear),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance("clear"),
        )
        .unwrap();
    let mut fragment = fragment.finish().unwrap();
    fragment.nodes[0].accesses = vec![buffer_access(
        &buffer,
        range,
        GpuBufferAccessKind::StorageWrite,
    )];
    assert_eq!(
        GpuPreparedWorkGraph::prepare(label("effect role graph"), [fragment])
            .unwrap_err()
            .cause(),
        GpuWorkGraphCause::OperationAccessContradiction
    );
}

#[test]
fn generic_attachment_shaped_access_cannot_discard_initialization() {
    let mut allocator = allocator();
    let texture = texture(
        &mut allocator,
        "generic discard",
        GpuTextureInitialization::Zeroed,
        1,
        1,
        [
            GpuTextureUsage::ColorAttachment,
            GpuTextureUsage::CopyDestination,
            GpuTextureUsage::Sampled,
        ],
    );
    let range = GpuTextureSubresourceRange::whole(&texture).unwrap();
    let mut fragment = builder("generic discard");
    fragment
        .declare_resource(GpuResourceRef::Texture(texture.clone()))
        .unwrap();
    add_compute(
        &mut fragment,
        "generic discard shaped access",
        [texture_access(
            &texture,
            range,
            GpuTextureAccessKind::ColorAttachment {
                load_kind: GpuAttachmentLoadKind::Clear,
                store: GpuAttachmentStore::Discard,
            },
        )],
    );
    add_compute(
        &mut fragment,
        "sample retained texture",
        [texture_access(
            &texture,
            range,
            GpuTextureAccessKind::SampledRead,
        )],
    );
    assert!(
        GpuPreparedWorkGraph::prepare(label("generic discard graph"), [fragment.finish().unwrap()])
            .is_ok()
    );
}

#[test]
fn complete_d2_array_copy_initializes_only_the_selected_nonzero_layers() {
    let mut allocator = allocator();
    let source = texture(
        &mut allocator,
        "array copy source",
        GpuTextureInitialization::Zeroed,
        1,
        4,
        [
            GpuTextureUsage::CopySource,
            GpuTextureUsage::CopyDestination,
        ],
    );
    let destination = texture(
        &mut allocator,
        "array copy destination",
        GpuTextureInitialization::Uninitialized,
        1,
        4,
        [GpuTextureUsage::CopyDestination, GpuTextureUsage::Sampled],
    );
    let source_region = GpuTextureCopyRegion::new(
        &source,
        0,
        GpuTextureOrigin::new(0, 0, 1),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(8, 8, 2).unwrap(),
    )
    .unwrap();
    let destination_region = GpuTextureCopyRegion::new(
        &destination,
        0,
        GpuTextureOrigin::new(0, 0, 1),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(8, 8, 2).unwrap(),
    )
    .unwrap();
    let copy = GpuCopyOperation::texture_to_texture(source_region, destination_region).unwrap();
    let selected = GpuTextureSubresourceRange::new(
        destination.descriptor().common().label(),
        0,
        1,
        1,
        2,
        GpuTextureAspect::Color,
    )
    .unwrap();
    let make_fragment = |read| {
        let mut fragment = builder("array copy");
        for resource in [
            GpuResourceRef::Texture(source.clone()),
            GpuResourceRef::Texture(destination.clone()),
        ] {
            fragment.declare_resource(resource).unwrap();
        }
        fragment
            .add_node(
                label("copy selected layers"),
                GpuWorkOperation::Copy(copy.clone()),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::TransferPreferred,
                provenance("copy selected layers"),
            )
            .unwrap();
        add_compute(
            &mut fragment,
            "sample destination",
            [texture_access(
                &destination,
                read,
                GpuTextureAccessKind::SampledRead,
            )],
        );
        fragment.finish().unwrap()
    };
    assert!(
        GpuPreparedWorkGraph::prepare(
            label("selected array copy graph"),
            [make_fragment(selected)]
        )
        .is_ok()
    );
    assert_eq!(
        GpuPreparedWorkGraph::prepare(
            label("whole array copy graph"),
            [make_fragment(
                GpuTextureSubresourceRange::whole(&destination).unwrap()
            )]
        )
        .unwrap_err()
        .cause(),
        GpuWorkGraphCause::ReadBeforeInitialization
    );
}

#[test]
fn buffer_effect_union_and_prepared_publication_are_canonical_and_deterministic() {
    let mut allocator = allocator();
    let buffer = buffer(
        &mut allocator,
        "canonical effects",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::CopyDestination],
    );
    let make_fragment = || {
        let mut fragment = builder("canonical effects");
        fragment
            .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
            .unwrap();
        for (name, offset) in [("clear second", 16), ("clear first", 0)] {
            let range = GpuBufferRange::new(&buffer, offset, 16).unwrap();
            fragment
                .add_node(
                    label(name),
                    GpuWorkOperation::Clear(
                        GpuClearOperation::buffer_zero(
                            GpuBufferRegion::new(&buffer, range).unwrap(),
                        )
                        .unwrap(),
                    ),
                    [],
                    GpuCapabilityRequirements::new(),
                    GpuExecutionPreference::TransferPreferred,
                    provenance(name),
                )
                .unwrap();
        }
        fragment.finish().unwrap()
    };
    let first =
        GpuPreparedWorkGraph::prepare(label("canonical effects graph"), [make_fragment()]).unwrap();
    let second =
        GpuPreparedWorkGraph::prepare(label("canonical effects graph"), [make_fragment()]).unwrap();
    assert_eq!(first.initialization(), second.initialization());
    let coverage = first
        .initialization()
        .iter()
        .find(|summary| summary.resource().diagnostic_identity() == buffer.diagnostic_identity())
        .unwrap()
        .final_coverage()
        .unwrap()
        .buffer_values()
        .unwrap();
    assert_eq!(
        coverage,
        [GpuBufferCoverage::dense(
            GpuBufferRange::new(&buffer, 0, 32).unwrap()
        )]
    );
}

#[test]
fn strided_effect_union_and_prepared_publication_retain_the_canonical_superset() {
    let mut allocator = allocator();
    let destination_label = label("strided union destination");
    let destination = allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common("strided union destination"),
                512,
                GpuBufferUsages::new(&destination_label, [GpuBufferUsage::CopyDestination])
                    .unwrap(),
                GpuBufferInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap();
    let source = texture(
        &mut allocator,
        "strided union source",
        GpuTextureInitialization::Zeroed,
        1,
        1,
        [
            GpuTextureUsage::CopySource,
            GpuTextureUsage::CopyDestination,
        ],
    );
    let copy = |height| {
        GpuCopyOperation::texture_to_buffer(
            GpuTextureCopyRegion::new(
                &source,
                0,
                GpuTextureOrigin::new(0, 0, 0),
                GpuTextureAspect::Color,
                GpuCopyExtent::new(8, height, 1).unwrap(),
            )
            .unwrap(),
            GpuBufferTextureLayout::new(&destination, 0, 64, 0).unwrap(),
        )
        .unwrap()
    };
    let mut fragment = builder("strided union");
    for resource in [
        GpuResourceRef::Texture(source.clone()),
        GpuResourceRef::Buffer(destination.clone()),
    ] {
        fragment.declare_resource(resource).unwrap();
    }
    for (name, height) in [("copy complete rows", 8), ("copy contained rows", 4)] {
        fragment
            .add_node(
                label(name),
                GpuWorkOperation::Copy(copy(height)),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::TransferPreferred,
                provenance(name),
            )
            .unwrap();
    }
    let fragment = fragment.finish().unwrap();
    let first =
        GpuPreparedWorkGraph::prepare(label("strided union graph"), [fragment.clone()]).unwrap();
    let second = GpuPreparedWorkGraph::prepare(label("strided union graph"), [fragment]).unwrap();
    assert_eq!(first.initialization(), second.initialization());
    let coverage = first
        .initialization()
        .iter()
        .find(|summary| {
            summary.resource().diagnostic_identity() == destination.diagnostic_identity()
        })
        .unwrap()
        .final_coverage()
        .unwrap()
        .buffer_values()
        .unwrap();
    assert_eq!(
        coverage,
        [GpuBufferCoverage::strided(
            GpuBufferStridedCoverage::new(&destination, 0, 32, 64, 8, 0, 1).unwrap()
        )]
    );
}

#[test]
fn padded_buffer_texture_copy_requires_and_initializes_only_logical_bytes() {
    let mut allocator = allocator();
    let source_label = label("padded source");
    let source = allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common("padded source"),
                512,
                GpuBufferUsages::new(&source_label, [GpuBufferUsage::CopySource]).unwrap(),
                GpuBufferInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap();
    let destination = texture(
        &mut allocator,
        "padded destination",
        GpuTextureInitialization::Uninitialized,
        1,
        1,
        [GpuTextureUsage::CopyDestination, GpuTextureUsage::Sampled],
    );
    let logical_rows = GpuBufferStridedCoverage::new(&source, 0, 32, 64, 8, 0, 1).unwrap();
    let input = GpuWorkResourceInput::new(
        GpuResourceRef::Buffer(source.clone()),
        GpuInitialCoverage::buffer(&source, [GpuBufferCoverage::strided(logical_rows.clone())])
            .unwrap(),
        provenance("padded source input"),
    )
    .unwrap();
    let destination_region = GpuTextureCopyRegion::new(
        &destination,
        0,
        GpuTextureOrigin::new(0, 0, 0),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(8, 8, 1).unwrap(),
    )
    .unwrap();
    let copy = GpuCopyOperation::buffer_to_texture(
        GpuBufferTextureLayout::new(&source, 0, 64, 0).unwrap(),
        destination_region,
    )
    .unwrap();
    let mut fragment = builder("padded copy");
    for resource in [
        GpuResourceRef::Buffer(source.clone()),
        GpuResourceRef::Texture(destination.clone()),
    ] {
        fragment.declare_resource(resource).unwrap();
    }
    fragment.add_input(input).unwrap();
    fragment
        .add_node(
            label("copy"),
            GpuWorkOperation::Copy(copy),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance("copy"),
        )
        .unwrap();
    add_compute(
        &mut fragment,
        "sample copied texture",
        [texture_access(
            &destination,
            GpuTextureSubresourceRange::whole(&destination).unwrap(),
            GpuTextureAccessKind::SampledRead,
        )],
    );
    assert!(
        GpuPreparedWorkGraph::prepare(label("padded copy graph"), [fragment.finish().unwrap()])
            .is_ok()
    );

    let partial_destination = texture(
        &mut allocator,
        "partial padded destination",
        GpuTextureInitialization::Uninitialized,
        1,
        1,
        [GpuTextureUsage::CopyDestination, GpuTextureUsage::Sampled],
    );
    let partial_copy = GpuCopyOperation::buffer_to_texture(
        GpuBufferTextureLayout::new(&source, 0, 64, 0).unwrap(),
        GpuTextureCopyRegion::new(
            &partial_destination,
            0,
            GpuTextureOrigin::new(0, 0, 0),
            GpuTextureAspect::Color,
            GpuCopyExtent::new(4, 8, 1).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut partial = builder("partial padded copy");
    for resource in [
        GpuResourceRef::Buffer(source.clone()),
        GpuResourceRef::Texture(partial_destination.clone()),
    ] {
        partial.declare_resource(resource).unwrap();
    }
    partial
        .add_input(
            GpuWorkResourceInput::new(
                GpuResourceRef::Buffer(source.clone()),
                GpuInitialCoverage::buffer(&source, [GpuBufferCoverage::strided(logical_rows)])
                    .unwrap(),
                provenance("partial padded source input"),
            )
            .unwrap(),
        )
        .unwrap();
    partial
        .add_node(
            label("partial copy"),
            GpuWorkOperation::Copy(partial_copy),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance("partial copy"),
        )
        .unwrap();
    add_compute(
        &mut partial,
        "sample partial texture",
        [texture_access(
            &partial_destination,
            GpuTextureSubresourceRange::whole(&partial_destination).unwrap(),
            GpuTextureAccessKind::SampledRead,
        )],
    );
    assert_eq!(
        GpuPreparedWorkGraph::prepare(
            label("partial padded copy graph"),
            [partial.finish().unwrap()]
        )
        .unwrap_err()
        .cause(),
        GpuWorkGraphCause::ReadBeforeInitialization
    );
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
            GpuInitialCoverage::buffer(&resource, [GpuBufferCoverage::dense(initialized)]).unwrap(),
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
        [GpuBufferUsage::Storage, GpuBufferUsage::CopyDestination],
    );
    let written = GpuBufferRange::new(&resource, 0, 16).unwrap();
    let overstated = GpuBufferRange::new(&resource, 0, 32).unwrap();
    let make_fragment = |intent, coverage_range| {
        let mut fragment = builder("mismatch producer");
        fragment
            .declare_resource(GpuResourceRef::Buffer(resource.clone()))
            .unwrap();
        fragment
            .add_node(
                label("write"),
                GpuWorkOperation::Clear(
                    GpuClearOperation::buffer_zero(
                        GpuBufferRegion::new(&resource, written).unwrap(),
                    )
                    .unwrap(),
                ),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::TransferPreferred,
                provenance("write"),
            )
            .unwrap();
        fragment
            .add_output(
                GpuWorkOutput::new(
                    GpuExportRelationship::new(
                        GpuResourceRef::Buffer(resource.clone()),
                        GpuExportKey::new("mismatch.output").unwrap(),
                        intent,
                        provenance("mismatch output"),
                    ),
                    GpuInitialCoverage::buffer(
                        &resource,
                        [GpuBufferCoverage::dense(coverage_range)],
                    )
                    .unwrap(),
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
