use engine::plugins::gpu::{GpuResourceDescriptor, GpuTextureFormat, GpuWorkResourceId};
use engine::plugins::render::api::RenderPassId;
use engine::plugins::render::{
    GpuParams, GpuUniform, RenderFlow, RenderGpuResourceLowering, RenderImportedBufferSemantic,
    RenderImportedTextureSemantic, RenderResourceDeclaration, RenderTargetAliasKind,
    detect_duplicate_resource_ids,
};

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ResourceTestParams {
    value: u32,
}

fn test_resource_ids(count: usize) -> Vec<GpuWorkResourceId> {
    let labels = (0..count)
        .map(|index| format!("test.resource.{index}"))
        .collect::<Vec<_>>();
    let flow = labels
        .iter()
        .fold(RenderFlow::new("test.resource.ids"), |flow, label| {
            flow.with_color_target(label.clone())
                .expect("render flow authoring should succeed")
        });
    labels
        .iter()
        .map(|label| flow.resource_id(label).expect("test resource should exist"))
        .collect()
}

#[test]
fn descriptor_construction_tracks_resource_kind_and_type_metadata() {
    let id = test_resource_ids(1)[0];
    let descriptor =
        RenderResourceDeclaration::declare_uniform::<ResourceTestParams>(id, "test uniform")
            .expect("uniform declaration should be valid");

    match descriptor {
        RenderResourceDeclaration::Uniform(value) => {
            assert_eq!(*value.id(), id);
            assert_eq!(
                value.params_type_id(),
                std::any::TypeId::of::<ResourceTestParams>()
            );
            assert!(value.params_type_name().contains("ResourceTestParams"));
            assert!(value.size_bytes() > 0);
            let raw = ResourceTestParams { value: 9 }.to_gpu();
            assert_eq!(raw.bytes.len() as u64, value.size_bytes());
            assert_eq!(u32::from_le_bytes(raw.bytes[0..4].try_into().unwrap()), 9);
        }
        other => panic!("unexpected descriptor variant: {other:?}"),
    }
}

#[test]
fn typed_ids_preserve_pass_raw_value_and_sort_resources_by_owner_local_identity() {
    let pass = RenderPassId::try_from_raw(7).unwrap();
    let raw: u64 = pass.into();
    assert_eq!(raw, 7);

    let ids = test_resource_ids(2);
    let a = ids[0];
    let b = ids[1];
    assert!(a < b);
    let (owner, local) = a.diagnostic_parts();
    assert_ne!(owner, 0);
    assert_eq!(local, 1);
    assert_eq!(a.to_string(), format!("{owner}:1"));
}

#[test]
fn duplicate_resource_detection_finds_collisions() {
    let ids = test_resource_ids(2);
    let duplicate = ids[0];
    let descriptors = vec![
        RenderResourceDeclaration::declare_sampled_texture(duplicate, "sampled"),
        RenderResourceDeclaration::declare_color_attachment(ids[1], "color"),
        RenderResourceDeclaration::declare_imported_external_texture(duplicate, "external"),
    ];

    let duplicates = detect_duplicate_resource_ids(&descriptors);
    assert_eq!(duplicates, vec![duplicate]);
}

#[test]
fn owned_buffer_lowering_returns_normalized_buffer() {
    let id = test_resource_ids(1)[0];
    let declaration =
        RenderResourceDeclaration::declare_uniform::<ResourceTestParams>(id, "owned buffer")
            .unwrap();

    let lowering = declaration
        .lower_gpu_resource((64, 64), GpuTextureFormat::Rgba8Unorm)
        .unwrap();
    let RenderGpuResourceLowering::Normalized(normalized) = lowering else {
        panic!("owned buffer should have normalized descriptor facts");
    };

    assert!(matches!(
        normalized.as_ref(),
        GpuResourceDescriptor::Buffer(buffer) if buffer.size_bytes() > 0
    ));
}

#[test]
fn owned_texture_lowering_returns_normalized_texture() {
    let id = test_resource_ids(1)[0];
    let declaration = RenderResourceDeclaration::declare_color_attachment(id, "owned texture");

    let lowering = declaration
        .lower_gpu_resource((320, 180), GpuTextureFormat::Bgra8UnormSrgb)
        .unwrap();
    let RenderGpuResourceLowering::Normalized(normalized) = lowering else {
        panic!("owned texture should have normalized descriptor facts");
    };

    assert!(matches!(
        normalized.as_ref(),
        GpuResourceDescriptor::Texture(texture)
            if texture.extent().width() == 320
                && texture.extent().height() == 180
                && texture.format() == GpuTextureFormat::Bgra8UnormSrgb
    ));
}

#[test]
fn imported_texture_lowering_preserves_unresolved_render_intent() {
    let id = test_resource_ids(1)[0];
    let declaration =
        RenderResourceDeclaration::declare_imported_external_texture(id, "external texture");

    let lowering = declaration
        .lower_gpu_resource((320, 180), GpuTextureFormat::Rgba8Unorm)
        .unwrap();

    assert!(matches!(
        lowering,
        RenderGpuResourceLowering::ImportedTexture(intent)
            if intent.id == id
                && intent.label == "external texture"
                && intent.semantic == RenderImportedTextureSemantic::External
    ));
}

#[test]
fn imported_buffer_lowering_preserves_unresolved_render_intent() {
    let id = test_resource_ids(1)[0];
    let declaration =
        RenderResourceDeclaration::declare_imported_history_buffer(id, "history buffer");

    let lowering = declaration
        .lower_gpu_resource((320, 180), GpuTextureFormat::Rgba8Unorm)
        .unwrap();

    assert!(matches!(
        lowering,
        RenderGpuResourceLowering::ImportedBuffer(intent)
            if intent.id == id
                && intent.label == "history buffer"
                && intent.semantic == RenderImportedBufferSemantic::HistoryBuffer
    ));
}

#[test]
fn target_alias_lowering_remains_a_render_relationship() {
    let id = test_resource_ids(1)[0];
    let declaration = RenderResourceDeclaration::declare_target_alias(
        id,
        "surface alias",
        RenderTargetAliasKind::Color,
    );

    let lowering = declaration
        .lower_gpu_resource((320, 180), GpuTextureFormat::Rgba8Unorm)
        .unwrap();

    assert!(matches!(
        lowering,
        RenderGpuResourceLowering::TargetAlias(alias)
            if alias.id == id
                && alias.label == "surface alias"
                && alias.kind == RenderTargetAliasKind::Color
    ));
}

#[test]
fn unresolved_import_lowering_rejects_an_empty_diagnostic_label() {
    let id = test_resource_ids(1)[0];
    let declaration = RenderResourceDeclaration::declare_imported_external_buffer(id, "   ");

    assert!(
        declaration
            .lower_gpu_resource((1, 1), GpuTextureFormat::Rgba8Unorm)
            .is_err()
    );
}

#[test]
fn diagnostic_labels_do_not_change_render_declaration_equality() {
    let ids = test_resource_ids(3);

    assert_eq!(
        RenderResourceDeclaration::declare_color_attachment(ids[0], "first texture label"),
        RenderResourceDeclaration::declare_color_attachment(ids[0], "second texture label"),
    );
    assert_eq!(
        RenderResourceDeclaration::declare_imported_external_texture(ids[1], "first import label"),
        RenderResourceDeclaration::declare_imported_external_texture(ids[1], "second import label",),
    );
    assert_eq!(
        RenderResourceDeclaration::declare_target_alias(
            ids[2],
            "first alias label",
            RenderTargetAliasKind::Color,
        ),
        RenderResourceDeclaration::declare_target_alias(
            ids[2],
            "second alias label",
            RenderTargetAliasKind::Color,
        ),
    );
}
