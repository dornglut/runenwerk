use engine::plugins::gpu::*;

fn label(value: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(value).unwrap()
}

fn common(value: &str) -> GpuResourceCommon {
    let label = label(value);
    GpuResourceCommon::owned(
        label.clone(),
        GpuResourceLifetime::Transient,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        GpuResourceProvenance::new(label, None, None),
    )
    .unwrap()
}

fn texture(
    allocator: &mut GpuWorkResourceIdAllocator,
    name: &str,
    format: GpuTextureFormat,
) -> GpuTextureHandle {
    let resource_label = label(name);
    allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::new(
                common(name),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 16, 8, 2).unwrap(),
                2,
                1,
                format,
                GpuTextureUsages::new(
                    &resource_label,
                    [
                        GpuTextureUsage::CopySource,
                        GpuTextureUsage::CopyDestination,
                    ],
                )
                .unwrap(),
                GpuTextureInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap()
}

#[test]
fn depth_copy_region_requires_complete_mip_extent_from_zero_origin() {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let depth = texture(&mut allocator, "depth copy", GpuTextureFormat::Depth32Float);

    assert!(
        GpuTextureCopyRegion::new(
            &depth,
            1,
            GpuTextureOrigin::new(0, 0, 0),
            GpuTextureAspect::DepthOnly,
            GpuCopyExtent::new(7, 4, 2).unwrap(),
        )
        .is_err(),
        "partial depth width must not survive into private buffer/texture or texture/texture encoding"
    );
    assert!(
        GpuTextureCopyRegion::new(
            &depth,
            1,
            GpuTextureOrigin::new(0, 0, 1),
            GpuTextureAspect::DepthOnly,
            GpuCopyExtent::new(8, 4, 1).unwrap(),
        )
        .is_err(),
        "partial depth array-layer coverage must not survive into private encoding"
    );

    let full = GpuTextureCopyRegion::new(
        &depth,
        1,
        GpuTextureOrigin::new(0, 0, 0),
        GpuTextureAspect::All,
        GpuCopyExtent::new(8, 4, 2).unwrap(),
    )
    .unwrap();
    assert_eq!(full.origin(), GpuTextureOrigin::new(0, 0, 0));
    assert_eq!(full.extent(), GpuCopyExtent::new(8, 4, 2).unwrap());
    assert_eq!(full.aspect(), GpuTextureAspect::DepthOnly);
}

#[test]
fn color_copy_region_remains_partial_and_layer_scoped() {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let color = texture(&mut allocator, "color copy", GpuTextureFormat::Rgba8Unorm);

    let region = GpuTextureCopyRegion::new(
        &color,
        0,
        GpuTextureOrigin::new(3, 2, 1),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(5, 3, 1).unwrap(),
    )
    .unwrap();

    assert_eq!(region.origin(), GpuTextureOrigin::new(3, 2, 1));
    assert_eq!(region.extent(), GpuCopyExtent::new(5, 3, 1).unwrap());
    assert_eq!(region.subresources().base_array_layer(), 1);
    assert_eq!(region.subresources().array_layer_count(), 1);
}
