use super::support::*;

fn retained_buffer(allocator: &mut GpuWorkResourceIdAllocator, name: &str) -> GpuBufferHandle {
    let resource_label = label(name);
    let common = GpuResourceCommon::owned(
        resource_label.clone(),
        GpuResourceLifetime::Retained,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        provenance(name),
    )
    .unwrap();
    allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common,
                64,
                GpuBufferUsages::new(&resource_label, [GpuBufferUsage::Storage]).unwrap(),
                GpuBufferInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap()
}

#[test]
fn retained_coverage_seeds_canonical_read_validation_and_initial_summary() {
    let mut allocator = allocator();
    let buffer = retained_buffer(&mut allocator, "retained history");
    let initialized = GpuBufferRange::new(&buffer, 0, 16).unwrap();
    let retained =
        GpuInitialCoverage::buffer(&buffer, [GpuBufferCoverage::dense(initialized)]).unwrap();

    let mut fragment = builder("retained reader");
    fragment
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    add_compute(
        &mut fragment,
        "read retained history",
        [buffer_access(
            &buffer,
            initialized,
            GpuBufferAccessKind::StorageRead,
        )],
    );
    let fragment = fragment.finish().unwrap();

    let unseeded =
        GpuPreparedWorkGraph::prepare(label("unseeded retained read"), [fragment.clone()])
            .unwrap_err();
    assert_eq!(
        unseeded.cause(),
        GpuWorkGraphCause::ReadBeforeInitialization
    );

    let prepared = GpuPreparedWorkGraph::prepare_with_retained_coverage(
        label("seeded retained read"),
        [fragment],
        std::slice::from_ref(&retained),
    )
    .unwrap();
    let summary = prepared
        .initialization()
        .iter()
        .find(|summary| summary.resource().diagnostic_identity() == buffer.diagnostic_identity())
        .unwrap();
    assert_eq!(summary.initial(), Some(&retained));
    assert_eq!(summary.final_coverage(), Some(&retained));
}

#[test]
fn retained_seed_cannot_initialize_current_transient_storage_with_equal_identity() {
    let mut transient_allocator = allocator();
    let transient = buffer(
        &mut transient_allocator,
        "current transient storage",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::Storage],
    );
    let mut retained_allocator = allocator();
    let retained_source = retained_buffer(&mut retained_allocator, "foreign retained descriptor");
    assert_eq!(
        transient.diagnostic_identity(),
        retained_source.diagnostic_identity(),
        "the proof requires equal process-local diagnostic identity with different descriptors"
    );

    let current_range = GpuBufferRange::new(&transient, 0, 16).unwrap();
    let retained_range = GpuBufferRange::new(&retained_source, 0, 16).unwrap();
    let invalid_seed =
        GpuInitialCoverage::buffer(&retained_source, [GpuBufferCoverage::dense(retained_range)])
            .unwrap();

    let mut fragment = builder("transient reader");
    fragment
        .declare_resource(GpuResourceRef::Buffer(transient.clone()))
        .unwrap();
    add_compute(
        &mut fragment,
        "read transient storage",
        [buffer_access(
            &transient,
            current_range,
            GpuBufferAccessKind::StorageRead,
        )],
    );

    let error = GpuPreparedWorkGraph::prepare_with_retained_coverage(
        label("equal identity retained seed versus transient storage"),
        [fragment.finish().unwrap()],
        &[invalid_seed],
    )
    .unwrap_err();
    assert_eq!(error.cause(), GpuWorkGraphCause::ReadBeforeInitialization);
}

#[test]
fn generic_shader_write_does_not_expand_retained_initialized_coverage() {
    let mut allocator = allocator();
    let buffer = retained_buffer(&mut allocator, "partially initialized history");
    let retained_range = GpuBufferRange::new(&buffer, 0, 16).unwrap();
    let shader_written = GpuBufferRange::new(&buffer, 16, 16).unwrap();
    let combined_read = GpuBufferRange::new(&buffer, 0, 32).unwrap();
    let retained =
        GpuInitialCoverage::buffer(&buffer, [GpuBufferCoverage::dense(retained_range)]).unwrap();

    let mut fragment = builder("generic retained write");
    fragment
        .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
        .unwrap();
    add_compute(
        &mut fragment,
        "generic write outside retained coverage",
        [buffer_access(
            &buffer,
            shader_written,
            GpuBufferAccessKind::StorageWrite,
        )],
    );
    add_compute(
        &mut fragment,
        "read retained plus generic write",
        [buffer_access(
            &buffer,
            combined_read,
            GpuBufferAccessKind::StorageRead,
        )],
    );

    let error = GpuPreparedWorkGraph::prepare_with_retained_coverage(
        label("generic write retained coverage"),
        [fragment.finish().unwrap()],
        &[retained],
    )
    .unwrap_err();
    assert_eq!(error.cause(), GpuWorkGraphCause::ReadBeforeInitialization);
    assert_eq!(error.resource(), Some(buffer.diagnostic_identity()));
}
