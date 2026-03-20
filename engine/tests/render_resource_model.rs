use engine::plugins::render::api::{RenderPassId, RenderResourceId, validate_namespaced_id};
use engine::plugins::render::resource::{RenderResourceDescriptor, detect_duplicate_resource_ids};
use engine::plugins::render::{GpuParams, GpuUniform};

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ResourceTestParams {
    value: u32,
}

#[test]
fn descriptor_construction_tracks_resource_kind_and_type_metadata() {
    let descriptor = RenderResourceDescriptor::uniform_buffer::<ResourceTestParams>(
        RenderResourceId::new("test.params"),
    );

    match descriptor {
        RenderResourceDescriptor::UniformBuffer(value) => {
            assert_eq!(value.id.as_str(), "test.params");
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
fn namespaced_id_validation_accepts_dot_separated_identifiers() {
    assert!(validate_namespaced_id("resource", "post.bloom.extract").is_ok());
    assert!(
        RenderPassId::new("ui.composite")
            .validate_namespaced()
            .is_ok()
    );

    let err =
        validate_namespaced_id("resource", "invalid").expect_err("must reject missing namespace");
    assert!(err.to_string().contains("expected dot-separated namespace"));
}

#[test]
fn duplicate_resource_detection_finds_collisions() {
    let descriptors = vec![
        RenderResourceDescriptor::sampled_texture("post.input"),
        RenderResourceDescriptor::color_target("post.output"),
        RenderResourceDescriptor::imported_texture("post.input"),
    ];

    let duplicates = detect_duplicate_resource_ids(&descriptors);
    assert_eq!(duplicates, vec!["post.input".to_string()]);
}
