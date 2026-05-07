use std::collections::BTreeMap;

use engine::plugins::render::{
    PreparedFlowInvocationId, PreparedFlowInvocationRequest, PreparedRenderFrameRequestResource,
    PreparedTargetBinding, PreparedViewFrame, PreparedViewKind, RenderDynamicTextureRetention,
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetDescriptorError,
    RenderDynamicTextureTargetKey, RenderDynamicTextureTargetRequestRegistryResource, RenderFlowId,
    RenderFrameProducerId, RenderTextureSampleMode, RenderTextureTargetFormat,
    RenderTextureTargetUsage,
};

fn dynamic_descriptor(
    target_id: &str,
    width: u32,
    height: u32,
    format: RenderTextureTargetFormat,
    usage: RenderTextureTargetUsage,
    sample_mode: RenderTextureSampleMode,
) -> RenderDynamicTextureTargetDescriptor {
    RenderDynamicTextureTargetDescriptor::new(
        RenderDynamicTextureTargetKey::new("test.dynamic", target_id),
        width,
        height,
        format,
        usage,
        sample_mode,
        RenderDynamicTextureRetention::RetainWhileRequested,
    )
}

fn producer(raw: u64) -> RenderFrameProducerId {
    RenderFrameProducerId::try_from_raw(raw).unwrap()
}

#[test]
fn dynamic_target_descriptor_validation_rejects_invalid_shapes() {
    let zero = dynamic_descriptor(
        "zero",
        0,
        64,
        RenderTextureTargetFormat::Rgba8Unorm,
        RenderTextureTargetUsage::color_sampled(),
        RenderTextureSampleMode::FilterableFloat,
    );
    assert!(matches!(
        zero.validate(),
        Err(RenderDynamicTextureTargetDescriptorError::ZeroDimensions {
            width: 0,
            height: 64
        })
    ));

    let sampled_without_usage = dynamic_descriptor(
        "sampled-without-usage",
        64,
        64,
        RenderTextureTargetFormat::Rgba8Unorm,
        RenderTextureTargetUsage::color_attachment_only(),
        RenderTextureSampleMode::FilterableFloat,
    );
    assert!(matches!(
        sampled_without_usage.validate(),
        Err(RenderDynamicTextureTargetDescriptorError::SampleModeRequiresSampledUsage)
    ));

    let uint_with_float_sampling = dynamic_descriptor(
        "uint-float-sampling",
        64,
        64,
        RenderTextureTargetFormat::R32Uint,
        RenderTextureTargetUsage::color_sampled(),
        RenderTextureSampleMode::FilterableFloat,
    );
    assert!(matches!(
        uint_with_float_sampling.validate(),
        Err(RenderDynamicTextureTargetDescriptorError::InvalidSampleModeForFormat)
    ));

    let depth_with_color_usage = dynamic_descriptor(
        "depth-color",
        64,
        64,
        RenderTextureTargetFormat::Depth32Float,
        RenderTextureTargetUsage::color_sampled(),
        RenderTextureSampleMode::Depth,
    );
    assert!(matches!(
        depth_with_color_usage.validate(),
        Err(RenderDynamicTextureTargetDescriptorError::InvalidDepthUsage)
    ));
}

#[test]
fn dynamic_target_request_registry_snapshots_valid_requests_by_key() {
    let mut registry = RenderDynamicTextureTargetRequestRegistryResource::default();
    registry
        .replace_contribution(
            producer(1),
            [
                dynamic_descriptor(
                    "b",
                    64,
                    64,
                    RenderTextureTargetFormat::Rgba8Unorm,
                    RenderTextureTargetUsage::color_sampled(),
                    RenderTextureSampleMode::FilterableFloat,
                ),
                dynamic_descriptor(
                    "a",
                    128,
                    64,
                    RenderTextureTargetFormat::Rgba8UnormSrgb,
                    RenderTextureTargetUsage::color_sampled(),
                    RenderTextureSampleMode::FilterableFloat,
                ),
            ],
        )
        .unwrap();

    let labels = registry
        .snapshot()
        .iter()
        .map(|descriptor| descriptor.key.label())
        .collect::<Vec<_>>();
    assert_eq!(labels, vec!["test.dynamic:a", "test.dynamic:b"]);

    let invalid = RenderDynamicTextureTargetDescriptor::new(
        RenderDynamicTextureTargetKey::new("", "bad"),
        64,
        64,
        RenderTextureTargetFormat::Rgba8Unorm,
        RenderTextureTargetUsage::color_sampled(),
        RenderTextureSampleMode::FilterableFloat,
        RenderDynamicTextureRetention::RetainWhileRequested,
    );
    assert!(
        registry
            .replace_contribution(producer(2), [invalid])
            .is_err()
    );
    assert_eq!(registry.diagnostics().len(), 1);
    assert_eq!(registry.snapshot().len(), 2);
}

#[test]
fn dynamic_target_request_registry_rejects_cross_producer_key_collisions() {
    let mut registry = RenderDynamicTextureTargetRequestRegistryResource::default();
    registry
        .replace_contribution(
            producer(1),
            [dynamic_descriptor(
                "shared",
                64,
                64,
                RenderTextureTargetFormat::Rgba8Unorm,
                RenderTextureTargetUsage::color_sampled(),
                RenderTextureSampleMode::FilterableFloat,
            )],
        )
        .unwrap();

    let err = registry
        .replace_contribution(
            producer(2),
            [dynamic_descriptor(
                "shared",
                64,
                64,
                RenderTextureTargetFormat::Rgba8Unorm,
                RenderTextureTargetUsage::color_sampled(),
                RenderTextureSampleMode::FilterableFloat,
            )],
        )
        .expect_err("dynamic target keys must have one owning producer");

    assert!(
        err.to_string().contains("already owned"),
        "unexpected error: {err}"
    );
    assert_eq!(registry.snapshot().len(), 1);
}

#[test]
fn prepared_frame_requests_carry_offscreen_view_and_target_alias_data() {
    let flow_id = RenderFlowId::try_from_raw(7).unwrap();
    let dynamic_key = RenderDynamicTextureTargetKey::new("test.dynamic", "viewport.7.scene");
    let mut alias_bindings = BTreeMap::new();
    alias_bindings.insert(
        "scene_color".to_string(),
        PreparedTargetBinding::DynamicTexture(dynamic_key.clone()),
    );

    let mut requests = PreparedRenderFrameRequestResource::default();
    requests
        .replace_contribution(
            producer(1),
            [PreparedViewFrame::offscreen_product(
                "viewport.7",
                (320, 180),
            )],
            [PreparedFlowInvocationRequest {
                invocation_id: PreparedFlowInvocationId::new("viewport.7.editor"),
                flow_id,
                view_id: "viewport.7".to_string(),
                target_alias_bindings: alias_bindings,
                uniform_overrides: BTreeMap::new(),
                history_signature: Some("viewport.7:v1".to_string()),
            }],
        )
        .unwrap();

    let views = requests.requested_views();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].kind, PreparedViewKind::OffscreenProduct);
    assert_eq!(views[0].target_size_px, (320, 180));

    let invocations = requests.requested_flow_invocations();
    assert_eq!(invocations.len(), 1);
    assert_eq!(invocations[0].flow_id, flow_id);
    assert_eq!(invocations[0].view_id, "viewport.7");
    assert_eq!(
        invocations[0].target_alias_bindings.get("scene_color"),
        Some(&PreparedTargetBinding::DynamicTexture(dynamic_key))
    );
    assert_eq!(
        invocations[0].history_signature.as_deref(),
        Some("viewport.7:v1")
    );
    assert!(
        invocations[0].uniform_overrides.is_empty(),
        "prepared invocation requests should expose an explicit override map even when unused"
    );
}

#[test]
fn prepared_frame_requests_reject_duplicate_invocation_ids_across_producers() {
    let flow_id = RenderFlowId::try_from_raw(7).unwrap();
    let request = |view_id: &str| PreparedFlowInvocationRequest {
        invocation_id: PreparedFlowInvocationId::new("shared.invocation"),
        flow_id,
        view_id: view_id.to_string(),
        target_alias_bindings: BTreeMap::new(),
        uniform_overrides: BTreeMap::new(),
        history_signature: None,
    };
    let mut requests = PreparedRenderFrameRequestResource::default();
    requests
        .replace_contribution(
            producer(1),
            [PreparedViewFrame::offscreen_product(
                "viewport.1",
                (320, 180),
            )],
            [request("viewport.1")],
        )
        .unwrap();

    let err = requests
        .replace_contribution(
            producer(2),
            [PreparedViewFrame::offscreen_product(
                "viewport.2",
                (320, 180),
            )],
            [request("viewport.2")],
        )
        .expect_err("invocation ids must be globally unique inside a frame");

    assert!(
        err.to_string().contains("duplicate invocation"),
        "unexpected error: {err}"
    );
}
