use engine::plugins::render::{
    RenderDynamicTextureTargetKey, RenderDynamicTextureUploadDescriptor,
    RenderDynamicTextureUploadRegistryError, RenderDynamicTextureUploadRegistryResource,
    RenderFrameProducerId, RenderTextureTargetFormat, RenderTextureUploadAlphaMode,
};

fn producer(raw: u64) -> RenderFrameProducerId {
    RenderFrameProducerId::try_from_raw(raw).unwrap()
}

fn upload(target_id: &str, bytes: Vec<u8>) -> RenderDynamicTextureUploadDescriptor {
    RenderDynamicTextureUploadDescriptor::rgba8(
        RenderDynamicTextureTargetKey::new("test.upload", target_id),
        0,
        0,
        1,
        1,
        RenderTextureUploadAlphaMode::Straight,
        7,
        bytes,
    )
}

#[test]
fn dynamic_texture_upload_registry_rejects_bad_byte_lengths() {
    let mut registry = RenderDynamicTextureUploadRegistryResource::default();
    let err = registry
        .replace_contribution(producer(1), [upload("bad-length", vec![1, 2, 3])])
        .expect_err("upload payloads must match rgba8 dimensions");

    assert!(matches!(
        err,
        RenderDynamicTextureUploadRegistryError::InvalidByteLength {
            expected: 4,
            actual: 3
        }
    ));
    assert_eq!(registry.diagnostics().len(), 1);
    assert!(registry.snapshot().is_empty());
}

#[test]
fn rejected_upload_contribution_preserves_previous_valid_snapshot() {
    let mut registry = RenderDynamicTextureUploadRegistryResource::default();
    registry
        .replace_contribution(producer(1), [upload("tile", vec![10, 20, 30, 255])])
        .expect("valid upload should be accepted");

    let err = registry
        .replace_contribution(producer(1), [upload("tile", vec![10, 20, 30])])
        .expect_err("invalid replacement should be rejected");

    assert!(matches!(
        err,
        RenderDynamicTextureUploadRegistryError::InvalidByteLength { .. }
    ));
    let snapshot = registry.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].rgba8, vec![10, 20, 30, 255]);
}

#[test]
fn premultiplied_alpha_uploads_are_valid_rgba8_uploads() {
    let descriptor = RenderDynamicTextureUploadDescriptor {
        target_key: RenderDynamicTextureTargetKey::new("test.upload", "premultiplied"),
        origin_x: 0,
        origin_y: 0,
        width: 1,
        height: 1,
        format: RenderTextureTargetFormat::Rgba8Unorm,
        alpha_mode: RenderTextureUploadAlphaMode::Premultiplied,
        product_generation: 9,
        rgba8: vec![32, 16, 8, 128],
    };

    let mut registry = RenderDynamicTextureUploadRegistryResource::default();
    registry
        .replace_contribution(producer(1), [descriptor])
        .expect("premultiplied rgba8 uploads should be accepted");
    assert_eq!(registry.snapshot().len(), 1);
}
