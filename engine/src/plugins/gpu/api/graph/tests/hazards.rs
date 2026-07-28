use super::support::*;

#[test]
fn inferred_hazards_are_typed_and_disjoint_regions_remain_independent() {
    let mut allocator = allocator();
    let buffer = buffer(
        &mut allocator,
        "hazards",
        GpuBufferInitialization::Zeroed,
        [GpuBufferUsage::Storage],
    );
    let first = GpuBufferRange::new(&buffer, 0, 16).unwrap();
    let second = GpuBufferRange::new(&buffer, 32, 16).unwrap();
    let mut fragment = builder("hazards");
    fragment
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    add_compute(
        &mut fragment,
        "read",
        [buffer_access(
            &buffer,
            first,
            GpuBufferAccessKind::StorageRead,
        )],
    );
    add_compute(
        &mut fragment,
        "write",
        [buffer_access(
            &buffer,
            first,
            GpuBufferAccessKind::StorageWrite,
        )],
    );
    add_compute(
        &mut fragment,
        "read again",
        [buffer_access(
            &buffer,
            first,
            GpuBufferAccessKind::StorageRead,
        )],
    );
    add_compute(
        &mut fragment,
        "disjoint write",
        [buffer_access(
            &buffer,
            second,
            GpuBufferAccessKind::StorageWrite,
        )],
    );
    let graph =
        GpuPreparedWorkGraph::prepare(label("hazard graph"), [fragment.finish().unwrap()]).unwrap();
    let reasons = graph
        .dependencies()
        .iter()
        .flat_map(GpuWorkDependency::reasons)
        .collect::<Vec<_>>();
    assert!(
        reasons
            .iter()
            .any(|reason| matches!(reason, GpuDependencyReason::WriteAfterRead { .. }))
    );
    assert!(
        reasons
            .iter()
            .any(|reason| matches!(reason, GpuDependencyReason::ReadAfterWrite { .. }))
    );
    assert!(graph.dependencies().iter().all(|dependency| {
        dependency.before().local_node() != 4 && dependency.after().local_node() != 4
    }));
}

#[test]
fn buffer_hazard_truth_table_is_lexically_oriented() {
    let mut allocator = allocator();
    let buffer = buffer(
        &mut allocator,
        "truth table",
        GpuBufferInitialization::Zeroed,
        [GpuBufferUsage::Storage],
    );
    let range = GpuBufferRange::whole(&buffer).unwrap();
    let mut fragment = builder("truth table");
    fragment
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    for (name, kind) in [
        ("read one", GpuBufferAccessKind::StorageRead),
        ("read two", GpuBufferAccessKind::StorageRead),
        ("write one", GpuBufferAccessKind::StorageWrite),
        ("write two", GpuBufferAccessKind::StorageWrite),
        ("read write", GpuBufferAccessKind::StorageReadWrite),
    ] {
        add_compute(&mut fragment, name, [buffer_access(&buffer, range, kind)]);
    }
    let graph =
        GpuPreparedWorkGraph::prepare(label("truth table graph"), [fragment.finish().unwrap()])
            .unwrap();
    let reasons = |before, after| {
        graph.dependencies().iter().find_map(|dependency| {
            (dependency.before().local_node() == before && dependency.after().local_node() == after)
                .then(|| dependency.reasons())
        })
    };
    assert!(reasons(1, 2).is_none());
    assert!(
        reasons(1, 3)
            .unwrap()
            .iter()
            .any(|reason| { matches!(reason, GpuDependencyReason::WriteAfterRead { .. }) })
    );
    assert!(
        reasons(3, 4)
            .unwrap()
            .iter()
            .any(|reason| { matches!(reason, GpuDependencyReason::WriteAfterWrite { .. }) })
    );
    let read_write = reasons(4, 5).unwrap();
    assert!(
        read_write
            .iter()
            .any(|reason| matches!(reason, GpuDependencyReason::ReadAfterWrite { .. }))
    );
    assert!(
        read_write
            .iter()
            .any(|reason| matches!(reason, GpuDependencyReason::WriteAfterWrite { .. }))
    );
}

#[test]
fn disjoint_query_ranges_remain_independent() {
    let mut allocator = allocator();
    let queries = allocator
        .allocate_query_set_handle(
            GpuQuerySetDescriptor::new(common("disjoint queries"), GpuQueryKind::Timestamp, 4)
                .unwrap(),
        )
        .unwrap();
    let operation = |range| {
        GpuWorkOperation::Render(
            GpuRenderOperation::new(
                [],
                None,
                [],
                [
                    GpuQueryAccess::new(&queries, range, GpuQueryAccessKind::WriteTimestamp)
                        .unwrap(),
                ],
            )
            .unwrap(),
        )
    };
    let mut fragment = builder("disjoint queries");
    fragment
        .declare_resource(GpuResourceRef::QuerySet(queries.clone()))
        .unwrap();
    for (name, range) in [
        ("first queries", GpuQueryRange::new(&queries, 0, 2).unwrap()),
        (
            "second queries",
            GpuQueryRange::new(&queries, 2, 2).unwrap(),
        ),
    ] {
        fragment
            .add_node(
                label(name),
                operation(range),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::GraphicsRequired,
                provenance(name),
            )
            .unwrap();
    }
    let graph =
        GpuPreparedWorkGraph::prepare(label("disjoint query graph"), [fragment.finish().unwrap()])
            .unwrap();
    assert!(graph.dependencies().is_empty());
}

#[test]
fn disjoint_texture_subresources_do_not_create_false_dependencies() {
    let mut allocator = allocator();
    let texture = texture(
        &mut allocator,
        "subresources",
        GpuTextureInitialization::Uninitialized,
        2,
        2,
        [GpuTextureUsage::StorageWrite],
    );
    let first_range = GpuTextureSubresourceRange::new(
        texture.descriptor().common().label(),
        0,
        1,
        0,
        1,
        GpuTextureAspect::Color,
    )
    .unwrap();
    let second_range = GpuTextureSubresourceRange::new(
        texture.descriptor().common().label(),
        1,
        1,
        1,
        1,
        GpuTextureAspect::Color,
    )
    .unwrap();
    let access = |range| {
        GpuResourceAccess::Texture(
            GpuTextureAccess::new(
                GpuTextureAccessResource::Texture(texture.clone()),
                range,
                GpuTextureAccessKind::StorageWrite,
            )
            .unwrap(),
        )
    };
    let mut fragment = builder("texture disjoint");
    fragment
        .declare_resource(GpuResourceRef::Texture(texture.clone()))
        .unwrap();
    add_compute(&mut fragment, "first mip", [access(first_range)]);
    add_compute(&mut fragment, "second mip", [access(second_range)]);
    let graph = GpuPreparedWorkGraph::prepare(label("texture graph"), [fragment.finish().unwrap()])
        .unwrap();
    assert!(graph.dependencies().is_empty());
    assert_eq!(graph.topological_order().len(), 2);
}
