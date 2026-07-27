use engine::plugins::gpu::GpuWorkResourceId;
use engine::plugins::render::api::RenderPassId;
use engine::plugins::render::{
    GpuParams, GpuUniform, RenderFlow, RenderResourceDeclaration, detect_duplicate_resource_ids,
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
