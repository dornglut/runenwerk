use crate::plugins::render::{
    FeatureContributionStatus, FeatureFallbackPolicy, PreparedFeatureContribution,
    PreparedFeaturePayload, PreparedFlowInvocation, PreparedRenderFrame, PreparedTargetBinding,
    PreparedViewFrame, PreparedViewKind, RenderDynamicTextureRetention,
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey,
    RenderDynamicTextureUploadDescriptor, RenderFeatureId, RenderProductSurfaceDiagnostic,
    RenderProductSurfaceDiagnosticKind, RenderProductSurfaceDiagnosticSeverity,
    RenderProductSurfaceManifest, RenderProductSurfaceRequestKind, RenderProductSurfaceStatus,
    RenderProductSurfaceStatusKind, RenderTextureSampleMode, RenderTextureTargetFormat,
    RenderTextureTargetUsage,
};
use product::{RenderProductSelection, RenderResidencyRequest, RenderSelectedProduct};
use ui_render_data::{
    ProductSurfaceTextureBindingSource, ViewportSurfaceBindingSource, ViewportSurfaceEmbedSlotId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRenderFrameInspection {
    pub frame_index: u64,
    pub prepare_epoch: u64,
    pub render_surface_id: u64,
    pub native_window_id: Option<u64>,
    pub surface_size: (u32, u32),
    pub views: Vec<PreparedViewInspectionEntry>,
    pub flow_invocations: Vec<PreparedFlowInvocationInspectionEntry>,
    pub feature_contributions: Vec<PreparedFeatureContributionInspectionEntry>,
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
pub struct PreparedFeatureContributionInspectionEntry {
    pub feature_id: String,
    pub status: String,
    pub fallback_policy: String,
    pub payload_kind: String,
    pub registered_payload_summary: Option<String>,
    pub registered_payload_fields: Vec<(String, String)>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductSurfaceManifestInspection {
    pub producer_id: u64,
    pub product_family: String,
    pub dynamic_texture_targets: Vec<DynamicTextureTargetInspectionEntry>,
    pub dynamic_texture_uploads: Vec<DynamicTextureUploadInspectionEntry>,
    pub prepared_views: Vec<PreparedViewInspectionEntry>,
    pub flow_invocations: Vec<PreparedFlowInvocationRequestInspectionEntry>,
    pub product_bindings: Vec<ProductSurfaceBindingInspectionEntry>,
    pub viewport_bindings: Vec<ViewportSurfaceBindingInspectionEntry>,
    pub statuses: Vec<ProductSurfaceStatusInspectionEntry>,
    pub diagnostics: Vec<ProductSurfaceDiagnosticInspectionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicTextureUploadInspectionEntry {
    pub target_key: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub origin_x: u32,
    pub origin_y: u32,
    pub product_generation: u64,
    pub byte_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFlowInvocationRequestInspectionEntry {
    pub invocation_id: String,
    pub flow_id: String,
    pub view_id: String,
    pub target_alias_bindings: Vec<TargetAliasBindingInspectionEntry>,
    pub uniform_override_count: usize,
    pub history_signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductSurfaceBindingInspectionEntry {
    pub surface_key: String,
    pub source: String,
    pub upload_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportSurfaceBindingInspectionEntry {
    pub viewport_id: u64,
    pub slot: u16,
    pub surface_key: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductSurfaceStatusInspectionEntry {
    pub surface_key: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductSurfaceDiagnosticInspectionEntry {
    pub producer_id: u64,
    pub product_family: String,
    pub surface_key: Option<String>,
    pub dynamic_target_key: Option<String>,
    pub view_id: Option<String>,
    pub invocation_id: Option<String>,
    pub request_kind: String,
    pub severity: String,
    pub diagnostic_kind: String,
    pub status: Option<String>,
    pub message: String,
}

pub fn inspect_prepared_render_frame(frame: &PreparedRenderFrame) -> PreparedRenderFrameInspection {
    PreparedRenderFrameInspection {
        frame_index: frame.context.frame_index,
        prepare_epoch: frame.context.prepare_epoch,
        render_surface_id: frame.surface.render_surface_id.raw(),
        native_window_id: frame.surface.native_window_id.map(|id| id.raw()),
        surface_size: frame.surface.target_size_px,
        views: frame.views.iter().map(inspect_prepared_view).collect(),
        flow_invocations: frame
            .flow_invocations
            .iter()
            .map(inspect_prepared_flow_invocation)
            .collect(),
        feature_contributions: frame
            .contributions
            .by_feature
            .iter()
            .map(|(feature_id, contribution)| {
                inspect_feature_contribution(feature_id, contribution)
            })
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

pub fn inspect_render_product_surface_manifest(
    manifest: &RenderProductSurfaceManifest,
) -> RenderProductSurfaceManifestInspection {
    RenderProductSurfaceManifestInspection {
        producer_id: manifest.producer_id().raw(),
        product_family: manifest.product_family().to_string(),
        dynamic_texture_targets: manifest
            .dynamic_targets()
            .iter()
            .map(inspect_dynamic_texture_target)
            .collect(),
        dynamic_texture_uploads: manifest
            .dynamic_uploads()
            .iter()
            .map(inspect_dynamic_texture_upload)
            .collect(),
        prepared_views: manifest.views().iter().map(inspect_prepared_view).collect(),
        flow_invocations: manifest
            .flow_invocations()
            .iter()
            .map(inspect_prepared_flow_invocation_request)
            .collect(),
        product_bindings: manifest
            .product_bindings()
            .iter()
            .map(|binding| ProductSurfaceBindingInspectionEntry {
                surface_key: binding.surface_key.clone(),
                source: product_surface_binding_source_label(&binding.source),
                upload_required: binding.upload_required,
            })
            .collect(),
        viewport_bindings: manifest
            .viewport_bindings()
            .iter()
            .map(|binding| ViewportSurfaceBindingInspectionEntry {
                viewport_id: binding.viewport_id,
                slot: viewport_surface_slot_raw(binding.slot),
                surface_key: binding.surface_key.clone(),
                source: viewport_surface_binding_source_label(&binding.source),
            })
            .collect(),
        statuses: manifest
            .statuses()
            .iter()
            .map(inspect_product_surface_status)
            .collect(),
        diagnostics: manifest
            .diagnostics()
            .iter()
            .map(inspect_product_surface_diagnostic)
            .collect(),
    }
}

fn inspect_feature_contribution(
    feature_id: &RenderFeatureId,
    contribution: &PreparedFeatureContribution,
) -> PreparedFeatureContributionInspectionEntry {
    let (payload_kind, registered_payload_summary, registered_payload_fields) =
        inspect_feature_payload(&contribution.payload);
    PreparedFeatureContributionInspectionEntry {
        feature_id: feature_id.to_string(),
        status: feature_contribution_status_label(contribution.status).to_string(),
        fallback_policy: feature_fallback_policy_label(contribution.fallback_policy).to_string(),
        payload_kind,
        registered_payload_summary,
        registered_payload_fields,
    }
}

fn inspect_feature_payload(
    payload: &PreparedFeaturePayload,
) -> (String, Option<String>, Vec<(String, String)>) {
    match payload {
        PreparedFeaturePayload::Empty => ("empty".to_string(), None, Vec::new()),
        PreparedFeaturePayload::Ui(_) => ("ui".to_string(), None, Vec::new()),
        PreparedFeaturePayload::SceneRoute(_) => ("scene_route".to_string(), None, Vec::new()),
        PreparedFeaturePayload::Draw(_) => ("draw".to_string(), None, Vec::new()),
        PreparedFeaturePayload::World(_) => ("world".to_string(), None, Vec::new()),
        PreparedFeaturePayload::Caves(_) => ("caves".to_string(), None, Vec::new()),
        PreparedFeaturePayload::Detail(_) => ("detail".to_string(), None, Vec::new()),
        PreparedFeaturePayload::ProceduralWorld(_) => {
            ("procedural_world".to_string(), None, Vec::new())
        }
        PreparedFeaturePayload::WindFields(_) => ("wind_fields".to_string(), None, Vec::new()),
        PreparedFeaturePayload::Material(_) => ("material".to_string(), None, Vec::new()),
        PreparedFeaturePayload::Deformation(_) => ("deformation".to_string(), None, Vec::new()),
        PreparedFeaturePayload::Registered(value) => {
            let inspection = value.inspect();
            (
                inspection.payload_kind,
                Some(inspection.summary),
                inspection.fields,
            )
        }
    }
}

fn feature_contribution_status_label(status: FeatureContributionStatus) -> &'static str {
    match status {
        FeatureContributionStatus::Ready => "ready",
        FeatureContributionStatus::Stale => "stale",
        FeatureContributionStatus::Disabled => "disabled",
        FeatureContributionStatus::Missing => "missing",
    }
}

fn feature_fallback_policy_label(policy: FeatureFallbackPolicy) -> &'static str {
    match policy {
        FeatureFallbackPolicy::ReuseLastGood => "reuse_last_good",
        FeatureFallbackPolicy::EmptyContribution => "empty_contribution",
        FeatureFallbackPolicy::SkipFeaturePasses => "skip_feature_passes",
        FeatureFallbackPolicy::FailFrame => "fail_frame",
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

fn inspect_dynamic_texture_upload(
    upload: &RenderDynamicTextureUploadDescriptor,
) -> DynamicTextureUploadInspectionEntry {
    DynamicTextureUploadInspectionEntry {
        target_key: upload.target_key.to_string(),
        width: upload.width,
        height: upload.height,
        format: dynamic_texture_format_name(upload.format).to_string(),
        origin_x: upload.origin_x,
        origin_y: upload.origin_y,
        product_generation: upload.product_generation,
        byte_len: upload.rgba8.len(),
    }
}

fn inspect_prepared_flow_invocation_request(
    invocation: &crate::plugins::render::PreparedFlowInvocationRequest,
) -> PreparedFlowInvocationRequestInspectionEntry {
    PreparedFlowInvocationRequestInspectionEntry {
        invocation_id: invocation.invocation_id.0.clone(),
        flow_id: invocation.flow_id.to_string(),
        view_id: invocation.view_id.clone(),
        target_alias_bindings: invocation
            .target_alias_bindings
            .iter()
            .map(|(alias, binding)| TargetAliasBindingInspectionEntry {
                alias: alias.clone(),
                binding: prepared_target_binding_label(binding),
            })
            .collect(),
        uniform_override_count: invocation.uniform_overrides.len(),
        history_signature: invocation.history_signature.clone(),
    }
}

fn inspect_product_surface_status(
    status: &RenderProductSurfaceStatus,
) -> ProductSurfaceStatusInspectionEntry {
    ProductSurfaceStatusInspectionEntry {
        surface_key: status.surface_key.clone(),
        status: product_surface_status_label(status.status).to_string(),
        message: status.message.clone(),
    }
}

fn inspect_product_surface_diagnostic(
    diagnostic: &RenderProductSurfaceDiagnostic,
) -> ProductSurfaceDiagnosticInspectionEntry {
    ProductSurfaceDiagnosticInspectionEntry {
        producer_id: diagnostic.producer_id.raw(),
        product_family: diagnostic.product_family.clone(),
        surface_key: diagnostic.surface_key.clone(),
        dynamic_target_key: diagnostic
            .dynamic_target_key
            .as_ref()
            .map(ToString::to_string),
        view_id: diagnostic.view_id.clone(),
        invocation_id: diagnostic.invocation_id.as_ref().map(ToString::to_string),
        request_kind: product_surface_request_kind_label(diagnostic.request_kind).to_string(),
        severity: product_surface_severity_label(diagnostic.severity).to_string(),
        diagnostic_kind: product_surface_diagnostic_kind_label(diagnostic.diagnostic_kind)
            .to_string(),
        status: diagnostic
            .status
            .map(product_surface_status_label)
            .map(str::to_string),
        message: diagnostic.message.clone(),
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

fn product_surface_binding_source_label(source: &ProductSurfaceTextureBindingSource) -> String {
    match source {
        ProductSurfaceTextureBindingSource::DynamicTexture {
            namespace,
            target_id,
        } => format!("dynamic_texture({namespace}:{target_id})"),
    }
}

fn viewport_surface_binding_source_label(source: &ViewportSurfaceBindingSource) -> String {
    match source {
        ViewportSurfaceBindingSource::DynamicTexture {
            namespace,
            target_id,
        } => format!("dynamic_texture({namespace}:{target_id})"),
    }
}

fn viewport_surface_slot_raw(slot: ViewportSurfaceEmbedSlotId) -> u16 {
    slot.raw()
}

fn product_surface_request_kind_label(kind: RenderProductSurfaceRequestKind) -> &'static str {
    match kind {
        RenderProductSurfaceRequestKind::DynamicTarget => "dynamic_target",
        RenderProductSurfaceRequestKind::DynamicUpload => "dynamic_upload",
        RenderProductSurfaceRequestKind::PreparedView => "prepared_view",
        RenderProductSurfaceRequestKind::FlowInvocation => "flow_invocation",
        RenderProductSurfaceRequestKind::ViewportSurfaceBinding => "viewport_surface_binding",
        RenderProductSurfaceRequestKind::ProductSurfaceBinding => "product_surface_binding",
        RenderProductSurfaceRequestKind::HistorySignature => "history_signature",
        RenderProductSurfaceRequestKind::Status => "status",
    }
}

fn product_surface_severity_label(
    severity: RenderProductSurfaceDiagnosticSeverity,
) -> &'static str {
    match severity {
        RenderProductSurfaceDiagnosticSeverity::Info => "info",
        RenderProductSurfaceDiagnosticSeverity::Warning => "warning",
        RenderProductSurfaceDiagnosticSeverity::Error => "error",
    }
}

fn product_surface_diagnostic_kind_label(kind: RenderProductSurfaceDiagnosticKind) -> &'static str {
    match kind {
        RenderProductSurfaceDiagnosticKind::DuplicateSurfaceKey => "duplicate_surface_key",
        RenderProductSurfaceDiagnosticKind::MissingDynamicTarget => "missing_dynamic_target",
        RenderProductSurfaceDiagnosticKind::MissingUpload => "missing_upload",
        RenderProductSurfaceDiagnosticKind::NonSampleableUiBinding => "non_sampleable_ui_binding",
        RenderProductSurfaceDiagnosticKind::ConflictingHistorySignature => {
            "conflicting_history_signature"
        }
        RenderProductSurfaceDiagnosticKind::ProducerStatus => "producer_status",
    }
}

fn product_surface_status_label(status: RenderProductSurfaceStatusKind) -> &'static str {
    match status {
        RenderProductSurfaceStatusKind::Ready => "ready",
        RenderProductSurfaceStatusKind::Stale => "stale",
        RenderProductSurfaceStatusKind::Fallback => "fallback",
        RenderProductSurfaceStatusKind::Rejected => "rejected",
        RenderProductSurfaceStatusKind::Unavailable => "unavailable",
        RenderProductSurfaceStatusKind::FailedPreserved => "failed_preserved",
    }
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
