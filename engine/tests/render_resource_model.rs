use engine::plugins::render::api::{RenderPassId, RenderResourceId};
use engine::plugins::render::resource::{RenderResourceDescriptor, detect_duplicate_resource_ids};
use engine::plugins::render::{GpuParams, GpuUniform};

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ResourceTestParams {
    value: u32,
}

#[test]
fn descriptor_construction_tracks_resource_kind_and_type_metadata() {
    let id = RenderResourceId::try_from_raw(42).unwrap();
    let descriptor = RenderResourceDescriptor::uniform_buffer::<ResourceTestParams>(id);

    match descriptor {
        RenderResourceDescriptor::UniformBuffer(value) => {
            assert_eq!(value.id, id);
            assert_eq!(
                value.params_type_id,
                std::any::TypeId::of::<ResourceTestParams>()
            );
            assert!(value.params_type_name.contains("ResourceTestParams"));
            assert!(value.size_bytes > 0);
            let raw = ResourceTestParams { value: 9 }.to_gpu();
            assert_eq!(raw.bytes.len() as u64, value.size_bytes);
            assert_eq!(u32::from_le_bytes(raw.bytes[0..4].try_into().unwrap()), 9);
        }
        other => panic!("unexpected descriptor variant: {other:?}"),
    }
}

#[test]
fn typed_ids_roundtrip_and_sort_by_raw_value() {
    let pass = RenderPassId::try_from_raw(7).unwrap();
    let raw: u64 = pass.into();
    assert_eq!(raw, 7);

    let a = RenderResourceId::try_from_raw(1).unwrap();
    let b = RenderResourceId::try_from_raw(2).unwrap();
    assert!(a < b);
    assert_eq!(a.to_string(), "1");
}

#[test]
fn duplicate_resource_detection_finds_collisions() {
    let duplicate = RenderResourceId::try_from_raw(9).unwrap();
    let descriptors = vec![
        RenderResourceDescriptor::sampled_texture(duplicate),
        RenderResourceDescriptor::color_target(RenderResourceId::try_from_raw(10).unwrap()),
        RenderResourceDescriptor::imported_texture(duplicate),
    ];

    let duplicates = detect_duplicate_resource_ids(&descriptors);
    assert_eq!(duplicates, vec![duplicate]);
}
