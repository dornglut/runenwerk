use crate::plugins::render::{
    PreparedFlowInvocation, PreparedRenderFrame, PreparedTargetBinding, PreparedViewFrame,
    PreparedViewKind, RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
    RenderDynamicTextureTargetKey, RenderTextureSampleMode, RenderTextureTargetFormat,
    RenderTextureTargetUsage,
};
use product::{RenderProductSelection, RenderResidencyRequest, RenderSelectedProduct};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRenderFrameInspection {
    pub frame_index: u64,
    pub prepare_epoch: u64,
    pub surface_size: (u32, u32),
    pub views: Vec<PreparedViewInspectionEntry>,
    pub flow_invocations: Vec<PreparedFlowInvocationInspectionEntry>,
    pub dynamic_texture_targets: Vec<DynamicTextureTargetInspectionEntry>,
    pub product_selections: Vec<RenderProductSelectionInspectionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedViewInspectionEntry {
    pub view_id: String,
    pub kind: String,
    pub target_size_px: (u32, u32),
    pub history_signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFlowInvocationInspectionEntry {
    pub invocation_id: String,
    pub flow_id: String,
    pub view_id: String,
    pub projected_uniform_count: usize,
    pub projected_dispatch_count: usize,
    pub required_state_type_count: usize,
    pub target_alias_bindings: Vec<TargetAliasBindingInspectionEntry>,
    pub history_signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetAliasBindingInspectionEntry {
    pub alias: String,
    pub binding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicTextureTargetInspectionEntry {
    pub key: String,
    pub namespace: String,
    pub target_id: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub usage: String,
    pub sample_mode: String,
    pub retention: String,
    pub displayable: bool,
    pub sampleable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductSelectionInspectionEntry {
    pub view_id: String,
    pub selected_products: Vec<RenderSelectedProductInspectionEntry>,
    pub required_targets: Vec<RenderSelectionTargetInspectionEntry>,
    pub residency_requests: Vec<RenderResidencyRequestInspectionEntry>,
    pub diagnostic_count: usize,
    pub overlays_enabled: bool,
    pub selected_overlay_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSelectedProductInspectionEntry {
    pub product_id: u64,
    pub scale_band: String,
    pub generation: u64,
    pub freshness: String,
    pub residency: String,
    pub authority_class: String,
    pub query_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSelectionTargetInspectionEntry {
    pub target_id: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderResidencyRequestInspectionEntry {
    pub product_id: u64,
    pub residency: String,
    pub priority: i32,
    pub hard_pin: bool,
}

pub fn inspect_prepared_render_frame(frame: &PreparedRenderFrame) -> PreparedRenderFrameInspection {
    PreparedRenderFrameInspection {
        frame_index: frame.context.frame_index,
        prepare_epoch: frame.context.prepare_epoch,
        surface_size: frame.surface.target_size_px,
        views: frame.views.iter().map(inspect_prepared_view).collect(),
        flow_invocations: frame
            .flow_invocations
            .iter()
            .map(inspect_prepared_flow_invocation)
            .collect(),
        dynamic_texture_targets: frame
            .dynamic_texture_targets
            .iter()
            .map(inspect_dynamic_texture_target)
            .collect(),
        product_selections: frame
            .product_selections
            .iter()
            .map(inspect_render_product_selection)
            .collect(),
    }
}

fn inspect_prepared_view(view: &PreparedViewFrame) -> PreparedViewInspectionEntry {
    PreparedViewInspectionEntry {
        view_id: view.view_id.clone(),
        kind: prepared_view_kind_name(view.kind).to_string(),
        target_size_px: view.target_size_px,
        history_signature: view.history_signature.clone(),
    }
}

fn inspect_prepared_flow_invocation(
    invocation: &PreparedFlowInvocation,
) -> PreparedFlowInvocationInspectionEntry {
    PreparedFlowInvocationInspectionEntry {
        invocation_id: invocation.invocation_id.0.clone(),
        flow_id: invocation.flow_id.to_string(),
        view_id: invocation.view_id.clone(),
        projected_uniform_count: invocation.inputs.projected_uniform_bytes.len(),
        projected_dispatch_count: invocation.inputs.projected_dispatch_workgroups.len(),
        required_state_type_count: invocation.inputs.required_state_types.len(),
        target_alias_bindings: invocation
            .target_alias_bindings
            .iter()
            .map(|(alias, binding)| TargetAliasBindingInspectionEntry {
                alias: alias.clone(),
                binding: prepared_target_binding_label(binding),
            })
            .collect(),
        history_signature: invocation.history_signature.clone(),
    }
}

fn inspect_dynamic_texture_target(
    target: &RenderDynamicTextureTargetDescriptor,
) -> DynamicTextureTargetInspectionEntry {
    DynamicTextureTargetInspectionEntry {
        key: target.key.to_string(),
        namespace: target.key.namespace.clone(),
        target_id: target.key.target_id.clone(),
        width: target.width,
        height: target.height,
        format: dynamic_texture_format_name(target.format).to_string(),
        usage: dynamic_texture_usage_label(target.usage),
        sample_mode: dynamic_texture_sample_mode_name(target.sample_mode).to_string(),
        retention: dynamic_texture_retention_label(target.retention),
        displayable: target.format.is_displayable(),
        sampleable: target.sample_mode.is_sampled(),
    }
}

fn inspect_render_product_selection(
    selection: &RenderProductSelection,
) -> RenderProductSelectionInspectionEntry {
    RenderProductSelectionInspectionEntry {
        view_id: selection.view_id.clone(),
        selected_products: selection
            .selected_products
            .iter()
            .map(inspect_render_selected_product)
            .collect(),
        required_targets: selection
            .required_targets
            .iter()
            .map(|target| RenderSelectionTargetInspectionEntry {
                target_id: target.target_id.clone(),
                width: target.width,
                height: target.height,
                format: target.format.clone(),
            })
            .collect(),
        residency_requests: selection
            .residency_requests
            .iter()
            .map(inspect_render_residency_request)
            .collect(),
        diagnostic_count: selection.diagnostics.len(),
        overlays_enabled: selection.diagnostics_selection.overlays_enabled,
        selected_overlay_ids: selection.diagnostics_selection.selected_overlay_ids.clone(),
    }
}

fn inspect_render_selected_product(
    selected: &RenderSelectedProduct,
) -> RenderSelectedProductInspectionEntry {
    RenderSelectedProductInspectionEntry {
        product_id: selected.product_id.raw(),
        scale_band: format!("{:?}", selected.scale_band),
        generation: selected.generation,
        freshness: format!("{:?}", selected.freshness),
        residency: format!("{:?}", selected.residency),
        authority_class: format!("{:?}", selected.authority_class),
        query_policy: format!("{:?}", selected.query_policy),
    }
}

fn inspect_render_residency_request(
    request: &RenderResidencyRequest,
) -> RenderResidencyRequestInspectionEntry {
    RenderResidencyRequestInspectionEntry {
        product_id: request.product_id.raw(),
        residency: format!("{:?}", request.residency),
        priority: request.priority,
        hard_pin: request.hard_pin,
    }
}

fn prepared_target_binding_label(binding: &PreparedTargetBinding) -> String {
    match binding {
        PreparedTargetBinding::DynamicTexture(key) => {
            format!("dynamic_texture({})", dynamic_texture_key_label(key))
        }
        PreparedTargetBinding::SurfaceColor => "surface_color".to_string(),
        PreparedTargetBinding::SurfaceDepth => "surface_depth".to_string(),
        PreparedTargetBinding::FlowOwned(resource_id) => {
            format!("flow_owned({resource_id})")
        }
    }
}

fn dynamic_texture_key_label(key: &RenderDynamicTextureTargetKey) -> String {
    format!("{}:{}", key.namespace, key.target_id)
}

fn prepared_view_kind_name(kind: PreparedViewKind) -> &'static str {
    match kind {
        PreparedViewKind::MainSurface => "main_surface",
        PreparedViewKind::OffscreenProduct => "offscreen_product",
    }
}

fn dynamic_texture_format_name(format: RenderTextureTargetFormat) -> &'static str {
    match format {
        RenderTextureTargetFormat::Rgba8Unorm => "rgba8_unorm",
        RenderTextureTargetFormat::Rgba8UnormSrgb => "rgba8_unorm_srgb",
        RenderTextureTargetFormat::R32Uint => "r32_uint",
        RenderTextureTargetFormat::Depth32Float => "depth32_float",
    }
}

fn dynamic_texture_sample_mode_name(sample_mode: RenderTextureSampleMode) -> &'static str {
    match sample_mode {
        RenderTextureSampleMode::FilterableFloat => "filterable_float",
        RenderTextureSampleMode::NonFilterableFloat => "non_filterable_float",
        RenderTextureSampleMode::Uint => "uint",
        RenderTextureSampleMode::Depth => "depth",
        RenderTextureSampleMode::NotSampled => "not_sampled",
    }
}

fn dynamic_texture_retention_label(retention: RenderDynamicTextureRetention) -> String {
    match retention {
        RenderDynamicTextureRetention::RetainWhileRequested => "retain_while_requested".to_string(),
        RenderDynamicTextureRetention::RetainUntilViewportClose => {
            "retain_until_viewport_close".to_string()
        }
        RenderDynamicTextureRetention::RetainForFrames(frames) => {
            format!("retain_for_frames({frames})")
        }
    }
}

fn dynamic_texture_usage_label(usage: RenderTextureTargetUsage) -> String {
    let mut parts = Vec::<&'static str>::new();
    if usage.color_attachment {
        parts.push("color_attachment");
    }
    if usage.depth_attachment {
        parts.push("depth_attachment");
    }
    if usage.sampled {
        parts.push("sampled");
    }
    if usage.storage {
        parts.push("storage");
    }
    if usage.copy_src {
        parts.push("copy_src");
    }
    if usage.copy_dst {
        parts.push("copy_dst");
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join("|")
    }
}
