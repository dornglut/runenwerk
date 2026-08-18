use super::{
    GpuRuntimeBindingDeviceFacts, GpuRuntimeBindingResource, GpuRuntimeBindingSet,
    GpuRuntimeBindingValue, GpuRuntimeBufferBinding, GpuValidatedBindGroupBindings,
};
use crate::plugins::gpu::{
    GpuBindGroupLayoutDescriptor, GpuBindingDeclaration, GpuBindingKey, GpuBindingKind,
    GpuBindingProvenance, GpuBufferAccessKind, GpuBufferDescriptor, GpuBufferHandle,
    GpuBufferInitialization, GpuBufferUsage, GpuBufferUsages, GpuMemoryIntent,
    GpuPipelineLayoutDescriptor, GpuProgramContractCause, GpuReconstruction, GpuResourceAccess,
    GpuResourceCommon, GpuResourceLabel, GpuResourceLifetime, GpuResourceProvenance,
    GpuShaderStage, GpuShaderStages, GpuStorageBufferAccess, GpuWorkResourceIdAllocator,
};
use core::num::{NonZeroU32, NonZeroU64};

fn storage_buffer(size: u64) -> GpuBufferHandle {
    let label = GpuResourceLabel::new("runtime-binding-storage").unwrap();
    let common = GpuResourceCommon::owned(
        label.clone(),
        GpuResourceLifetime::Transient,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        GpuResourceProvenance::new(label.clone(), None, None),
    )
    .unwrap();
    let usages = GpuBufferUsages::new(&label, [GpuBufferUsage::Storage]).unwrap();
    let descriptor =
        GpuBufferDescriptor::new(common, size, usages, GpuBufferInitialization::Uninitialized)
            .unwrap();
    let mut allocator = GpuWorkResourceIdAllocator::for_owner_scope(NonZeroU64::new(17).unwrap());
    allocator.allocate_buffer_handle(descriptor).unwrap()
}

fn declaration(array_count: Option<NonZeroU32>) -> GpuBindingDeclaration {
    GpuBindingDeclaration::new(
        GpuBindingKey::try_new(0, 0).unwrap(),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::storage_buffer(
            GpuStorageBufferAccess::ReadWrite,
            true,
            NonZeroU64::new(16),
        ),
        array_count,
        "storage",
        GpuBindingProvenance::new("runtime-binding-test", None).unwrap(),
    )
    .unwrap()
}

fn device_facts() -> GpuRuntimeBindingDeviceFacts {
    device_facts_with_dynamic_limits(8, 4)
}

fn device_facts_with_dynamic_limits(
    max_dynamic_uniform_buffers: u32,
    max_dynamic_storage_buffers: u32,
) -> GpuRuntimeBindingDeviceFacts {
    GpuRuntimeBindingDeviceFacts::new(
        NonZeroU64::new(16).unwrap(),
        NonZeroU64::new(16).unwrap(),
        max_dynamic_uniform_buffers,
        max_dynamic_storage_buffers,
        [],
    )
}

fn runtime_value(dynamic_offset: u64) -> GpuRuntimeBindingValue {
    GpuRuntimeBindingValue::new(
        GpuBindingKey::try_new(0, 0).unwrap(),
        [GpuRuntimeBindingResource::Buffer(
            GpuRuntimeBufferBinding::new(
                storage_buffer(64),
                0,
                NonZeroU64::new(32).unwrap(),
                Some(dynamic_offset),
            ),
        )],
    )
    .unwrap()
}

#[test]
fn runtime_bindings_validate_usage_range_alignment_and_layout() {
    let layout = GpuBindGroupLayoutDescriptor::new(0, [declaration(None)]).unwrap();
    let validated =
        GpuValidatedBindGroupBindings::new(layout, [runtime_value(16)], &device_facts())
            .expect("compatible runtime binding should validate");

    assert_eq!(validated.layout().group(), 0);
    assert!(validated.value(0).is_some());
    assert_eq!(validated.values().len(), 1);
}

#[test]
fn runtime_bindings_reject_unaligned_dynamic_offsets() {
    let layout = GpuBindGroupLayoutDescriptor::new(0, [declaration(None)]).unwrap();
    let error = GpuValidatedBindGroupBindings::new(layout, [runtime_value(4)], &device_facts())
        .expect_err("unaligned dynamic offsets must be rejected");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::RuntimeBindingIncompatible
    );
}

#[test]
fn runtime_bindings_reject_missing_values() {
    let layout = GpuBindGroupLayoutDescriptor::new(0, [declaration(None)]).unwrap();
    let error = GpuValidatedBindGroupBindings::new(layout, [], &device_facts())
        .expect_err("every declaration needs one runtime value");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::RuntimeBindingIncompatible
    );
}

#[test]
fn runtime_bindings_reject_wrong_fixed_array_cardinality_before_backend_realization() {
    // This proves only G4B's logical resource-count compatibility. Native array
    // feature admission and backend group restrictions belong to G4C2.
    let layout =
        GpuBindGroupLayoutDescriptor::new(0, [declaration(Some(NonZeroU32::new(2).unwrap()))])
            .unwrap();
    let error = GpuValidatedBindGroupBindings::new(layout, [runtime_value(16)], &device_facts())
        .expect_err("fixed binding arrays require exact cardinality");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::RuntimeBindingIncompatible
    );
}

#[test]
fn runtime_binding_set_is_complete_pipeline_shaped_logical_use() {
    let group = GpuBindGroupLayoutDescriptor::new(0, [declaration(None)]).unwrap();
    let layout = GpuPipelineLayoutDescriptor::new([group]).unwrap();
    let bindings = GpuRuntimeBindingSet::new(layout.clone(), [runtime_value(16)], &device_facts())
        .expect("one complete dynamic storage binding should validate");

    assert_eq!(bindings.layout(), &layout);
    assert_eq!(bindings.groups().len(), 1);
    assert!(bindings.group(0).is_some());
    assert_eq!(bindings.values().count(), 1);
    assert_eq!(bindings.accesses().len(), 1);
    let GpuResourceAccess::Buffer(access) = &bindings.accesses()[0] else {
        panic!("storage runtime binding must derive one buffer access");
    };
    assert_eq!(access.kind(), GpuBufferAccessKind::StorageReadWrite);
    assert_eq!(access.range().offset(), 16);
    assert_eq!(access.range().size(), 32);
}

#[test]
fn runtime_binding_set_rejects_dynamic_storage_when_admitted_limit_is_zero() {
    let group = GpuBindGroupLayoutDescriptor::new(0, [declaration(None)]).unwrap();
    let layout = GpuPipelineLayoutDescriptor::new([group]).unwrap();
    let error = GpuRuntimeBindingSet::new(
        layout,
        [runtime_value(16)],
        &device_facts_with_dynamic_limits(8, 0),
    )
    .expect_err("zero admitted dynamic-storage capacity must reject the declaration");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::RuntimeBindingIncompatible
    );
    assert_eq!(error.label(), "dynamic storage buffers");
}
