use super::support::*;

fn dependency_reasons(
    graph: &GpuPreparedWorkGraph,
    before: u64,
    after: u64,
) -> &[GpuDependencyReason] {
    graph
        .dependencies()
        .iter()
        .find(|dependency| {
            dependency.before().local_node() == before && dependency.after().local_node() == after
        })
        .map(GpuWorkDependency::reasons)
        .expect("the requested dependency should exist")
}

#[test]
fn texture_view_candidates_use_normalized_parent_texture_identity() {
    let mut allocator = allocator();
    let texture = texture(
        &mut allocator,
        "candidate parent texture",
        GpuTextureInitialization::Zeroed,
        2,
        1,
        [GpuTextureUsage::StorageRead, GpuTextureUsage::StorageWrite],
    );
    let range = GpuTextureSubresourceRange::new(
        texture.descriptor().common().label(),
        0,
        1,
        0,
        1,
        GpuTextureAspect::Color,
    )
    .unwrap();
    let view = texture_view(
        &mut allocator,
        &texture,
        "candidate texture view",
        range,
    );

    let mut fragment = builder("normalized texture candidates");
    fragment
        .declare_resource(GpuResourceRef::Texture(texture.clone()))
        .unwrap();
    fragment
        .declare_resource(GpuResourceRef::TextureView(view.clone()))
        .unwrap();
    add_compute(
        &mut fragment,
        "write parent texture",
        [texture_access(
            &texture,
            range,
            GpuTextureAccessKind::StorageWrite,
        )],
    );
    add_compute(
        &mut fragment,
        "read through view",
        [GpuResourceAccess::Texture(
            GpuTextureAccess::new(
                GpuTextureAccessResource::TextureView(view),
                range,
                GpuTextureAccessKind::StorageRead,
            )
            .unwrap(),
        )],
    );

    let graph = GpuPreparedWorkGraph::prepare(
        label("normalized texture candidate graph"),
        [fragment.finish().unwrap()],
    )
    .unwrap();

    assert_eq!(
        dependency_reasons(&graph, 1, 2),
        [GpuDependencyReason::ReadAfterWrite {
            resource: texture.diagnostic_identity(),
            region: GpuDependencyRegion::Texture(range),
        }]
    );
}

#[test]
fn same_node_access_candidates_never_self_edge_and_duplicate_reasons_collapse() {
    let mut allocator = allocator();
    let buffer = buffer(
        &mut allocator,
        "candidate buffer",
        GpuBufferInitialization::Zeroed,
        [GpuBufferUsage::Storage],
    );
    let first_read = GpuBufferRange::new(&buffer, 0, 32).unwrap();
    let second_read = GpuBufferRange::new(&buffer, 0, 48).unwrap();
    let write = GpuBufferRange::new(&buffer, 8, 8).unwrap();

    let mut fragment = builder("same-node candidates");
    fragment
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    add_compute(
        &mut fragment,
        "overlapping reads",
        [
            buffer_access(&buffer, first_read, GpuBufferAccessKind::StorageRead),
            buffer_access(&buffer, second_read, GpuBufferAccessKind::StorageRead),
        ],
    );
    add_compute(
        &mut fragment,
        "write shared overlap",
        [buffer_access(
            &buffer,
            write,
            GpuBufferAccessKind::StorageWrite,
        )],
    );

    let graph = GpuPreparedWorkGraph::prepare(
        label("same-node candidate graph"),
        [fragment.finish().unwrap()],
    )
    .unwrap();

    assert!(
        graph
            .dependencies()
            .iter()
            .all(|dependency| dependency.before() != dependency.after())
    );
    assert_eq!(graph.dependencies().len(), 1);
    assert_eq!(
        dependency_reasons(&graph, 1, 2),
        [GpuDependencyReason::WriteAfterRead {
            resource: buffer.diagnostic_identity(),
            region: GpuDependencyRegion::Buffer(write),
        }]
    );
}
