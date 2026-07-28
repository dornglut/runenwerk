use super::support::*;

#[test]
fn initial_coverage_is_checked_normalized_and_kind_preserving() {
    let mut allocator = allocator();
    let buffer = buffer(
        &mut allocator,
        "coverage",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::Storage],
    );
    let coverage = GpuInitialCoverage::buffer_ranges(
        &buffer,
        [
            GpuBufferRange::new(&buffer, 16, 16).unwrap(),
            GpuBufferRange::new(&buffer, 0, 16).unwrap(),
            GpuBufferRange::new(&buffer, 8, 8).unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(coverage.kind(), GpuInitialCoverageKind::BufferRanges);
    assert_eq!(coverage.buffer_range_values().unwrap().len(), 1);
    assert_eq!(coverage.buffer_range_values().unwrap()[0].size(), 32);

    let queries = allocator
        .allocate_query_set_handle(
            GpuQuerySetDescriptor::new(common("queries"), GpuQueryKind::Timestamp, 4).unwrap(),
        )
        .unwrap();
    assert!(
        GpuInitialCoverage::descriptor_initialization(GpuResourceRef::QuerySet(queries)).is_err()
    );
}

#[test]
fn same_node_access_deduplicates_merges_and_rejects_incompatible_roles() {
    let mut allocator = allocator();
    let buffer = buffer(
        &mut allocator,
        "storage",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::Storage, GpuBufferUsage::Uniform],
    );
    let range = GpuBufferRange::whole(&buffer).unwrap();
    let read = buffer_access(&buffer, range, GpuBufferAccessKind::StorageRead);
    let write = buffer_access(&buffer, range, GpuBufferAccessKind::StorageWrite);
    let mut fragment = builder("normalize");
    fragment
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    add_compute(&mut fragment, "read write", [read.clone(), read, write]);
    let fragment = fragment.finish().unwrap();
    assert_eq!(fragment.nodes()[0].accesses().len(), 1);
    assert!(matches!(
        fragment.nodes()[0].accesses()[0],
        GpuResourceAccess::Buffer(ref access)
            if access.kind() == GpuBufferAccessKind::StorageReadWrite
    ));

    let mut invalid = builder("invalid normalize");
    invalid
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    let error = invalid
        .add_node(
            label("contradiction"),
            compute_operation(),
            [
                buffer_access(&buffer, range, GpuBufferAccessKind::UniformRead),
                buffer_access(&buffer, range, GpuBufferAccessKind::StorageWrite),
            ],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::Automatic,
            provenance("contradiction"),
        )
        .unwrap_err();
    assert_eq!(
        error.cause(),
        GpuWorkAuthoringCause::IncompatibleSameNodeAccess
    );
}
