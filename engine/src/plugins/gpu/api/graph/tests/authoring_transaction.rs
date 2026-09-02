use super::support::*;

fn colliding_buffer(original: &GpuBufferHandle, name: &str) -> GpuBufferHandle {
    let resource_label = label(name);
    GpuBufferHandle::from_descriptor(
        original.diagnostic_identity(),
        GpuBufferDescriptor::new(
            common(name),
            64,
            GpuBufferUsages::new(&resource_label, [GpuBufferUsage::CopyDestination]).unwrap(),
            GpuBufferInitialization::Uninitialized,
        )
        .unwrap(),
    )
}

#[test]
fn failed_lexical_resource_transaction_rolls_back_and_preserves_next_node_identity() {
    let mut allocator = allocator();
    let retained = buffer(
        &mut allocator,
        "retained destination",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::CopyDestination],
    );
    let source = buffer(
        &mut allocator,
        "tentative source",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::CopySource, GpuBufferUsage::CopyDestination],
    );
    let conflicting_destination = colliding_buffer(&retained, "conflicting destination");
    assert_ne!(
        GpuResourceRef::Buffer(retained.clone()),
        GpuResourceRef::Buffer(conflicting_destination.clone())
    );

    let mut fragment = builder("lexical rollback");
    fragment
        .declare_resource(GpuResourceRef::Buffer(retained.clone()))
        .unwrap();

    let conflicting_copy = GpuCopyOperation::buffer_to_buffer(
        GpuBufferRegion::whole(&source).unwrap(),
        GpuBufferRegion::whole(&conflicting_destination).unwrap(),
    )
    .unwrap();
    let error = fragment
        .operation("conflicting copy", conflicting_copy)
        .unwrap_err();
    assert_eq!(error.cause(), GpuWorkAuthoringCause::InvalidResourceKind);

    // This succeeds only when the source inserted before the destination collision was rolled back.
    fragment
        .declare_resource(GpuResourceRef::Buffer(source.clone()))
        .unwrap();
    let continued = fragment
        .operation(
            "continued clear",
            GpuClearOperation::buffer_zero(GpuBufferRegion::whole(&source).unwrap()).unwrap(),
        )
        .unwrap();
    assert_eq!(continued.diagnostic_local(), 1);

    let fragment = fragment.finish().unwrap();
    assert_eq!(fragment.nodes().len(), 1);
    assert_eq!(fragment.resources().len(), 2);
    assert!(
        fragment
            .resources()
            .contains(&GpuResourceRef::Buffer(retained))
    );
    assert!(
        fragment
            .resources()
            .contains(&GpuResourceRef::Buffer(source))
    );
}

#[test]
fn successful_lexical_resource_transaction_reuses_exact_and_declares_missing_once() {
    let mut allocator = allocator();
    let source = buffer(
        &mut allocator,
        "predeclared source",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::CopySource],
    );
    let destination = buffer(
        &mut allocator,
        "derived destination",
        GpuBufferInitialization::Uninitialized,
        [GpuBufferUsage::CopyDestination],
    );

    let mut fragment = builder("lexical success");
    fragment
        .declare_resource(GpuResourceRef::Buffer(source.clone()))
        .unwrap();
    let copy = GpuCopyOperation::buffer_to_buffer(
        GpuBufferRegion::whole(&source).unwrap(),
        GpuBufferRegion::whole(&destination).unwrap(),
    )
    .unwrap();
    let id = fragment.operation("copy", copy).unwrap();
    assert_eq!(id.diagnostic_local(), 1);

    let fragment = fragment.finish().unwrap();
    assert_eq!(fragment.nodes().len(), 1);
    assert_eq!(fragment.resources().len(), 2);
    assert!(
        fragment
            .resources()
            .contains(&GpuResourceRef::Buffer(source))
    );
    assert!(
        fragment
            .resources()
            .contains(&GpuResourceRef::Buffer(destination))
    );
}
