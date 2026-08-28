pub(super) use super::super::*;
pub(super) use crate::plugins::gpu::*;
pub(super) use core::num::NonZeroU64;

pub(super) fn label(value: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(value).unwrap()
}

pub(super) fn provenance(value: &str) -> GpuResourceProvenance {
    let label = label(value);
    GpuResourceProvenance::new(label, None, None)
}

pub(super) fn common(value: &str) -> GpuResourceCommon {
    GpuResourceCommon::owned(
        label(value),
        GpuResourceLifetime::Transient,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        provenance(value),
    )
    .unwrap()
}

pub(super) fn allocator() -> GpuWorkResourceIdAllocator {
    GpuWorkResourceIdAllocator::for_owner_scope(NonZeroU64::new(97).unwrap())
}

pub(super) fn buffer(
    allocator: &mut GpuWorkResourceIdAllocator,
    name: &str,
    initialization: GpuBufferInitialization,
    usages: impl IntoIterator<Item = GpuBufferUsage>,
) -> GpuBufferHandle {
    let resource_label = label(name);
    allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common(name),
                64,
                GpuBufferUsages::new(&resource_label, usages).unwrap(),
                initialization,
            )
            .unwrap(),
        )
        .unwrap()
}

pub(super) fn texture(
    allocator: &mut GpuWorkResourceIdAllocator,
    name: &str,
    initialization: GpuTextureInitialization,
    mip_levels: u32,
    layers: u32,
    usages: impl IntoIterator<Item = GpuTextureUsage>,
) -> GpuTextureHandle {
    let resource_label = label(name);
    allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::new(
                common(name),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 8, 8, layers)
                    .unwrap(),
                mip_levels,
                1,
                GpuTextureFormat::Rgba8Unorm,
                GpuTextureUsages::new(&resource_label, usages).unwrap(),
                initialization,
            )
            .unwrap(),
        )
        .unwrap()
}

pub(super) fn texture_view(
    allocator: &mut GpuWorkResourceIdAllocator,
    texture: &GpuTextureHandle,
    name: &str,
    subresources: GpuTextureSubresourceRange,
) -> GpuTextureViewHandle {
    allocator
        .allocate_texture_view_handle(
            GpuTextureViewDescriptor::new(
                common(name),
                texture,
                None,
                GpuTextureDimension::D2,
                subresources,
            )
            .unwrap(),
        )
        .unwrap()
}

pub(super) fn prepared_texture_initialization(name: &str) -> GpuTextureInitialization {
    let resource_label = label(name);
    let extent = GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 8, 8, 1).unwrap();
    let data =
        PreparedGpuData::<TransferData>::from_pod_transfer(name, &[0_u8; 256], provenance(name))
            .unwrap();
    GpuTextureInitialization::Prepared(
        GpuPreparedTextureData::new(
            &resource_label,
            data,
            GpuTextureFormat::Rgba8Unorm,
            extent,
            32,
            0,
        )
        .unwrap(),
    )
}

pub(super) fn depth_texture(
    allocator: &mut GpuWorkResourceIdAllocator,
    name: &str,
) -> GpuTextureHandle {
    let resource_label = label(name);
    allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::new(
                common(name),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 8, 8, 1).unwrap(),
                1,
                1,
                GpuTextureFormat::Depth32Float,
                GpuTextureUsages::new(
                    &resource_label,
                    [
                        GpuTextureUsage::DepthStencilAttachment,
                        GpuTextureUsage::Sampled,
                    ],
                )
                .unwrap(),
                GpuTextureInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap()
}

pub(super) fn compute_operation() -> GpuWorkOperation {
    compute_operation_with_dispatch(GpuDispatchSize::new(1, 1, 1).unwrap())
}

pub(super) fn compute_operation_with_dispatch(dispatch: GpuDispatchSize) -> GpuWorkOperation {
    let mut source_registry = GpuProgramSourceRegistry::new(1, 1024).unwrap();
    let owner = GpuProgramSourceOwnerId::allocate().unwrap();
    let source = source_registry
        .admit_wgsl(
            GpuProgramSourceIdentity::new(
                owner,
                GpuProgramSourceKey::new("graph.test.compute").unwrap(),
                GpuProgramSourceRevision::try_from_raw(1).unwrap(),
            ),
            "@compute @workgroup_size(1) fn main() {}",
            GpuProgramSourceProvenance::new("graph-test", None).unwrap(),
        )
        .unwrap();
    let entry_point = GpuEntryPointName::new("main").unwrap();
    let program = GpuProgramDescriptor::new(
        source,
        [entry_point.clone()],
        std::iter::empty::<GpuBindingLayoutRefinement>(),
    )
    .unwrap();
    let pipeline = GpuComputePipelineDescriptor::new(
        program,
        entry_point,
        GpuPipelineConfiguration::default(),
    )
    .unwrap();
    let bindings = GpuRuntimeBindingSet::new(pipeline.layout().clone(), []).unwrap();
    let dispatch = GpuDispatchIntent::direct(dispatch);
    GpuWorkOperation::Compute(GpuComputeOperation::new(pipeline, bindings, dispatch).unwrap())
}

pub(super) fn builder(name: &str) -> GpuWorkFragmentBuilder {
    GpuWorkFragmentBuilder::new(label(name), provenance(name))
}

pub(super) fn add_compute(
    builder: &mut GpuWorkFragmentBuilder,
    name: &str,
    accesses: impl IntoIterator<Item = GpuResourceAccess>,
) -> GpuWorkNodeId {
    builder
        .add_node(
            label(name),
            compute_operation(),
            accesses,
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::Automatic,
            provenance(name),
        )
        .unwrap()
}

pub(super) fn buffer_access(
    buffer: &GpuBufferHandle,
    range: GpuBufferRange,
    kind: GpuBufferAccessKind,
) -> GpuResourceAccess {
    GpuResourceAccess::Buffer(GpuBufferAccess::new(buffer, range, kind).unwrap())
}

pub(super) fn texture_access(
    texture: &GpuTextureHandle,
    range: GpuTextureSubresourceRange,
    kind: GpuTextureAccessKind,
) -> GpuResourceAccess {
    GpuResourceAccess::Texture(
        GpuTextureAccess::new(
            GpuTextureAccessResource::Texture(texture.clone()),
            range,
            kind,
        )
        .unwrap(),
    )
}
