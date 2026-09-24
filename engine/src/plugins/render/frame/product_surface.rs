use super::{PreparedFlowInvocationId, PreparedFlowInvocationRequest, PreparedViewFrame};
use crate::plugins::render::{
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey,
    RenderDynamicTextureUploadDescriptor, RenderFrameProducerId,
};
use std::collections::{BTreeMap, BTreeSet};
use ui_render_data::{
    ProductSurfaceTextureBindingSource, ViewportSurfaceBindingSource, ViewportSurfaceEmbedSlotId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductSurfaceRequest {
    dynamic_targets: Vec<RenderDynamicTextureTargetDescriptor>,
    view: PreparedViewFrame,
    flow_invocation: PreparedFlowInvocationRequest,
}

impl RenderProductSurfaceRequest {
    pub fn new(view: PreparedViewFrame, flow_invocation: PreparedFlowInvocationRequest) -> Self {
        Self {
            dynamic_targets: Vec::new(),
            view,
            flow_invocation,
        }
    }

    pub fn from_parts(
        dynamic_targets: impl IntoIterator<Item = RenderDynamicTextureTargetDescriptor>,
        view: PreparedViewFrame,
        flow_invocation: PreparedFlowInvocationRequest,
    ) -> Self {
        Self {
            dynamic_targets: dynamic_targets.into_iter().collect(),
            view,
            flow_invocation,
        }
    }

    pub fn with_dynamic_target(mut self, descriptor: RenderDynamicTextureTargetDescriptor) -> Self {
        self.dynamic_targets.push(descriptor);
        self
    }

    pub fn dynamic_targets(&self) -> &[RenderDynamicTextureTargetDescriptor] {
        &self.dynamic_targets
    }

    pub fn view(&self) -> &PreparedViewFrame {
        &self.view
    }

    pub fn flow_invocation(&self) -> &PreparedFlowInvocationRequest {
        &self.flow_invocation
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<RenderDynamicTextureTargetDescriptor>,
        PreparedViewFrame,
        PreparedFlowInvocationRequest,
    ) {
        (self.dynamic_targets, self.view, self.flow_invocation)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderProductSurfaceRequestBatch {
    dynamic_targets: Vec<RenderDynamicTextureTargetDescriptor>,
    views: Vec<PreparedViewFrame>,
    flow_invocations: Vec<PreparedFlowInvocationRequest>,
}

impl RenderProductSurfaceRequestBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_request(request: RenderProductSurfaceRequest) -> Self {
        Self::from_requests([request])
    }

    pub fn from_requests(requests: impl IntoIterator<Item = RenderProductSurfaceRequest>) -> Self {
        let mut batch = Self::new();
        for request in requests {
            batch.push_request(request);
        }
        batch
    }

    pub fn with_dynamic_target(mut self, descriptor: RenderDynamicTextureTargetDescriptor) -> Self {
        self.dynamic_targets.push(descriptor);
        self
    }

    pub fn with_view(mut self, view: PreparedViewFrame) -> Self {
        self.views.push(view);
        self
    }

    pub fn with_flow_invocation(mut self, request: PreparedFlowInvocationRequest) -> Self {
        self.flow_invocations.push(request);
        self
    }

    pub fn push_request(&mut self, request: RenderProductSurfaceRequest) {
        let (dynamic_targets, view, flow_invocation) = request.into_parts();
        self.dynamic_targets.extend(dynamic_targets);
        self.views.push(view);
        self.flow_invocations.push(flow_invocation);
    }

    pub fn extend(&mut self, other: Self) {
        self.dynamic_targets.extend(other.dynamic_targets);
        self.views.extend(other.views);
        self.flow_invocations.extend(other.flow_invocations);
    }

    pub fn dynamic_targets(&self) -> &[RenderDynamicTextureTargetDescriptor] {
        &self.dynamic_targets
    }

    pub fn views(&self) -> &[PreparedViewFrame] {
        &self.views
    }

    pub fn flow_invocations(&self) -> &[PreparedFlowInvocationRequest] {
        &self.flow_invocations
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<RenderDynamicTextureTargetDescriptor>,
        Vec<PreparedViewFrame>,
        Vec<PreparedFlowInvocationRequest>,
    ) {
        (self.dynamic_targets, self.views, self.flow_invocations)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderProductSurfaceRequestKind {
    DynamicTarget,
    DynamicUpload,
    PreparedView,
    FlowInvocation,
    ViewportSurfaceBinding,
    ProductSurfaceBinding,
    HistorySignature,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderProductSurfaceDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderProductSurfaceDiagnosticKind {
    DuplicateSurfaceKey,
    MissingDynamicTarget,
    MissingUpload,
    NonSampleableUiBinding,
    ConflictingHistorySignature,
    ProducerStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderProductSurfaceStatusKind {
    Ready,
    Stale,
    Fallback,
    Rejected,
    Unavailable,
    FailedPreserved,
}

impl RenderProductSurfaceStatusKind {
    pub const fn severity(self) -> RenderProductSurfaceDiagnosticSeverity {
        match self {
            Self::Ready => RenderProductSurfaceDiagnosticSeverity::Info,
            Self::Stale | Self::Fallback => RenderProductSurfaceDiagnosticSeverity::Warning,
            Self::Rejected | Self::Unavailable | Self::FailedPreserved => {
                RenderProductSurfaceDiagnosticSeverity::Error
            }
        }
    }

    pub const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductSurfaceStatus {
    pub surface_key: String,
    pub status: RenderProductSurfaceStatusKind,
    pub message: String,
}

impl RenderProductSurfaceStatus {
    pub fn new(
        surface_key: impl Into<String>,
        status: RenderProductSurfaceStatusKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            surface_key: surface_key.into(),
            status,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductSurfaceDiagnostic {
    pub producer_id: RenderFrameProducerId,
    pub product_family: String,
    pub surface_key: Option<String>,
    pub dynamic_target_key: Option<RenderDynamicTextureTargetKey>,
    pub view_id: Option<String>,
    pub invocation_id: Option<PreparedFlowInvocationId>,
    pub request_kind: RenderProductSurfaceRequestKind,
    pub severity: RenderProductSurfaceDiagnosticSeverity,
    pub diagnostic_kind: RenderProductSurfaceDiagnosticKind,
    pub status: Option<RenderProductSurfaceStatusKind>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductSurfaceProductBinding {
    pub surface_key: String,
    pub source: ProductSurfaceTextureBindingSource,
    pub upload_required: bool,
}

impl RenderProductSurfaceProductBinding {
    pub fn new(surface_key: impl Into<String>, source: ProductSurfaceTextureBindingSource) -> Self {
        Self {
            surface_key: surface_key.into(),
            source,
            upload_required: false,
        }
    }

    pub fn upload_backed(
        surface_key: impl Into<String>,
        source: ProductSurfaceTextureBindingSource,
    ) -> Self {
        Self {
            surface_key: surface_key.into(),
            source,
            upload_required: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductSurfaceViewportBinding {
    pub viewport_id: u64,
    pub slot: ViewportSurfaceEmbedSlotId,
    pub surface_key: String,
    pub source: ViewportSurfaceBindingSource,
}

impl RenderProductSurfaceViewportBinding {
    pub fn new(
        viewport_id: u64,
        slot: ViewportSurfaceEmbedSlotId,
        surface_key: impl Into<String>,
        source: ViewportSurfaceBindingSource,
    ) -> Self {
        Self {
            viewport_id,
            slot,
            surface_key: surface_key.into(),
            source,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductSurfaceManifest {
    producer_id: RenderFrameProducerId,
    product_family: String,
    dynamic_targets: Vec<RenderDynamicTextureTargetDescriptor>,
    dynamic_uploads: Vec<RenderDynamicTextureUploadDescriptor>,
    views: Vec<PreparedViewFrame>,
    flow_invocations: Vec<PreparedFlowInvocationRequest>,
    product_bindings: Vec<RenderProductSurfaceProductBinding>,
    viewport_bindings: Vec<RenderProductSurfaceViewportBinding>,
    statuses: Vec<RenderProductSurfaceStatus>,
    diagnostics: Vec<RenderProductSurfaceDiagnostic>,
}

impl RenderProductSurfaceManifest {
    pub fn new(producer_id: RenderFrameProducerId, product_family: impl Into<String>) -> Self {
        Self {
            producer_id,
            product_family: product_family.into(),
            dynamic_targets: Vec::new(),
            dynamic_uploads: Vec::new(),
            views: Vec::new(),
            flow_invocations: Vec::new(),
            product_bindings: Vec::new(),
            viewport_bindings: Vec::new(),
            statuses: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn from_request_batch(
        producer_id: RenderFrameProducerId,
        product_family: impl Into<String>,
        batch: RenderProductSurfaceRequestBatch,
    ) -> Self {
        let (dynamic_targets, views, flow_invocations) = batch.into_parts();
        Self::new(producer_id, product_family)
            .with_dynamic_targets(dynamic_targets)
            .with_views(views)
            .with_flow_invocations(flow_invocations)
    }

    pub fn producer_id(&self) -> RenderFrameProducerId {
        self.producer_id
    }

    pub fn product_family(&self) -> &str {
        &self.product_family
    }

    pub fn with_request(mut self, request: RenderProductSurfaceRequest) -> Self {
        let (dynamic_targets, view, flow_invocation) = request.into_parts();
        self.dynamic_targets.extend(dynamic_targets);
        self.views.push(view);
        self.flow_invocations.push(flow_invocation);
        self
    }

    pub fn with_dynamic_target(mut self, descriptor: RenderDynamicTextureTargetDescriptor) -> Self {
        self.dynamic_targets.push(descriptor);
        self
    }

    pub fn with_dynamic_targets(
        mut self,
        descriptors: impl IntoIterator<Item = RenderDynamicTextureTargetDescriptor>,
    ) -> Self {
        self.dynamic_targets.extend(descriptors);
        self
    }

    pub fn with_dynamic_upload(mut self, upload: RenderDynamicTextureUploadDescriptor) -> Self {
        self.dynamic_uploads.push(upload);
        self
    }

    pub fn with_dynamic_uploads(
        mut self,
        uploads: impl IntoIterator<Item = RenderDynamicTextureUploadDescriptor>,
    ) -> Self {
        self.dynamic_uploads.extend(uploads);
        self
    }

    pub fn with_view(mut self, view: PreparedViewFrame) -> Self {
        self.views.push(view);
        self
    }

    pub fn with_views(mut self, views: impl IntoIterator<Item = PreparedViewFrame>) -> Self {
        self.views.extend(views);
        self
    }

    pub fn with_flow_invocation(mut self, invocation: PreparedFlowInvocationRequest) -> Self {
        self.flow_invocations.push(invocation);
        self
    }

    pub fn with_flow_invocations(
        mut self,
        invocations: impl IntoIterator<Item = PreparedFlowInvocationRequest>,
    ) -> Self {
        self.flow_invocations.extend(invocations);
        self
    }

    pub fn with_product_surface_binding(
        mut self,
        surface_key: impl Into<String>,
        source: ProductSurfaceTextureBindingSource,
    ) -> Self {
        self.product_bindings
            .push(RenderProductSurfaceProductBinding::new(surface_key, source));
        self
    }

    pub fn with_upload_backed_product_surface_binding(
        mut self,
        surface_key: impl Into<String>,
        source: ProductSurfaceTextureBindingSource,
    ) -> Self {
        self.product_bindings
            .push(RenderProductSurfaceProductBinding::upload_backed(
                surface_key,
                source,
            ));
        self
    }

    pub fn with_viewport_surface_binding(
        mut self,
        viewport_id: u64,
        slot: ViewportSurfaceEmbedSlotId,
        surface_key: impl Into<String>,
        source: ViewportSurfaceBindingSource,
    ) -> Self {
        self.viewport_bindings
            .push(RenderProductSurfaceViewportBinding::new(
                viewport_id,
                slot,
                surface_key,
                source,
            ));
        self
    }

    pub fn with_status(
        mut self,
        surface_key: impl Into<String>,
        status: RenderProductSurfaceStatusKind,
        message: impl Into<String>,
    ) -> Self {
        self.statuses.push(RenderProductSurfaceStatus::new(
            surface_key,
            status,
            message,
        ));
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: RenderProductSurfaceDiagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    pub fn dynamic_targets(&self) -> &[RenderDynamicTextureTargetDescriptor] {
        &self.dynamic_targets
    }

    pub fn dynamic_uploads(&self) -> &[RenderDynamicTextureUploadDescriptor] {
        &self.dynamic_uploads
    }

    pub fn views(&self) -> &[PreparedViewFrame] {
        &self.views
    }

    pub fn flow_invocations(&self) -> &[PreparedFlowInvocationRequest] {
        &self.flow_invocations
    }

    pub fn product_bindings(&self) -> &[RenderProductSurfaceProductBinding] {
        &self.product_bindings
    }

    pub fn viewport_bindings(&self) -> &[RenderProductSurfaceViewportBinding] {
        &self.viewport_bindings
    }

    pub fn statuses(&self) -> &[RenderProductSurfaceStatus] {
        &self.statuses
    }

    pub fn diagnostics(&self) -> Vec<RenderProductSurfaceDiagnostic> {
        let mut diagnostics = self.diagnostics.clone();
        diagnostics.extend(self.structural_diagnostics());
        diagnostics
    }

    pub fn has_error_diagnostics(&self) -> bool {
        self.diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.severity == RenderProductSurfaceDiagnosticSeverity::Error)
    }

    pub fn into_render_parts(
        self,
    ) -> (
        Vec<RenderDynamicTextureTargetDescriptor>,
        Vec<RenderDynamicTextureUploadDescriptor>,
        Vec<PreparedViewFrame>,
        Vec<PreparedFlowInvocationRequest>,
    ) {
        (
            self.dynamic_targets,
            self.dynamic_uploads,
            self.views,
            self.flow_invocations,
        )
    }

    fn structural_diagnostics(&self) -> Vec<RenderProductSurfaceDiagnostic> {
        let mut diagnostics = Vec::new();
        let mut target_keys = BTreeSet::<RenderDynamicTextureTargetKey>::new();
        let mut target_by_key =
            BTreeMap::<RenderDynamicTextureTargetKey, &RenderDynamicTextureTargetDescriptor>::new();
        for target in &self.dynamic_targets {
            if !target_keys.insert(target.key.clone()) {
                diagnostics.push(self.diagnostic(
                    Some(target.key.to_string()),
                    Some(target.key.clone()),
                    None,
                    None,
                    RenderProductSurfaceRequestKind::DynamicTarget,
                    RenderProductSurfaceDiagnosticSeverity::Error,
                    RenderProductSurfaceDiagnosticKind::DuplicateSurfaceKey,
                    None,
                    format!("duplicate dynamic target '{}'", target.key),
                ));
            }
            target_by_key.insert(target.key.clone(), target);
        }

        let mut upload_keys = BTreeSet::<RenderDynamicTextureTargetKey>::new();
        let mut upload_key_set = BTreeSet::<RenderDynamicTextureTargetKey>::new();
        for upload in &self.dynamic_uploads {
            if !upload_keys.insert(upload.target_key.clone()) {
                diagnostics.push(self.diagnostic(
                    Some(upload.target_key.to_string()),
                    Some(upload.target_key.clone()),
                    None,
                    None,
                    RenderProductSurfaceRequestKind::DynamicUpload,
                    RenderProductSurfaceDiagnosticSeverity::Error,
                    RenderProductSurfaceDiagnosticKind::DuplicateSurfaceKey,
                    None,
                    format!("duplicate dynamic upload for '{}'", upload.target_key),
                ));
            }
            if !target_by_key.contains_key(&upload.target_key) {
                diagnostics.push(self.diagnostic(
                    Some(upload.target_key.to_string()),
                    Some(upload.target_key.clone()),
                    None,
                    None,
                    RenderProductSurfaceRequestKind::DynamicUpload,
                    RenderProductSurfaceDiagnosticSeverity::Error,
                    RenderProductSurfaceDiagnosticKind::MissingDynamicTarget,
                    None,
                    format!(
                        "dynamic upload references '{}' but the manifest declares no matching dynamic target",
                        upload.target_key
                    ),
                ));
            }
            upload_key_set.insert(upload.target_key.clone());
        }

        for binding in &self.product_bindings {
            let key = product_binding_key(&binding.source);
            self.validate_ui_binding(
                &mut diagnostics,
                binding.surface_key.as_str(),
                key.clone(),
                RenderProductSurfaceRequestKind::ProductSurfaceBinding,
                &target_by_key,
            );
            if binding.upload_required && !upload_key_set.contains(&key) {
                diagnostics.push(self.diagnostic(
                    Some(binding.surface_key.clone()),
                    Some(key.clone()),
                    None,
                    None,
                    RenderProductSurfaceRequestKind::DynamicUpload,
                    RenderProductSurfaceDiagnosticSeverity::Error,
                    RenderProductSurfaceDiagnosticKind::MissingUpload,
                    None,
                    format!(
                        "upload-backed surface '{}' references '{}' but the manifest declares no matching dynamic upload",
                        binding.surface_key, key
                    ),
                ));
            }
        }

        for binding in &self.viewport_bindings {
            let key = viewport_binding_key(&binding.source);
            self.validate_ui_binding(
                &mut diagnostics,
                binding.surface_key.as_str(),
                key,
                RenderProductSurfaceRequestKind::ViewportSurfaceBinding,
                &target_by_key,
            );
        }

        diagnostics.extend(self.history_signature_diagnostics());
        diagnostics.extend(self.status_diagnostics());
        diagnostics
    }

    fn validate_ui_binding(
        &self,
        diagnostics: &mut Vec<RenderProductSurfaceDiagnostic>,
        surface_key: &str,
        key: RenderDynamicTextureTargetKey,
        request_kind: RenderProductSurfaceRequestKind,
        target_by_key: &BTreeMap<
            RenderDynamicTextureTargetKey,
            &RenderDynamicTextureTargetDescriptor,
        >,
    ) {
        let Some(target) = target_by_key.get(&key) else {
            diagnostics.push(self.diagnostic(
                Some(surface_key.to_string()),
                Some(key.clone()),
                None,
                None,
                request_kind,
                RenderProductSurfaceDiagnosticSeverity::Error,
                RenderProductSurfaceDiagnosticKind::MissingDynamicTarget,
                None,
                format!(
                    "UI binding for surface '{}' references '{}' but the manifest declares no matching dynamic target",
                    surface_key, key
                ),
            ));
            return;
        };
        if !target.sample_mode.is_sampled() {
            diagnostics.push(self.diagnostic(
                Some(surface_key.to_string()),
                Some(key),
                None,
                None,
                request_kind,
                RenderProductSurfaceDiagnosticSeverity::Error,
                RenderProductSurfaceDiagnosticKind::NonSampleableUiBinding,
                None,
                format!(
                    "UI binding for surface '{}' references a non-sampleable dynamic target",
                    surface_key
                ),
            ));
        }
    }

    fn history_signature_diagnostics(&self) -> Vec<RenderProductSurfaceDiagnostic> {
        let view_signatures = self
            .views
            .iter()
            .filter_map(|view| {
                view.history_signature
                    .as_ref()
                    .map(|signature| (view.view_id.as_str(), signature.as_str()))
            })
            .collect::<BTreeMap<_, _>>();
        let mut signatures = BTreeMap::<RenderDynamicTextureTargetKey, String>::new();
        let mut diagnostics = Vec::new();
        for invocation in &self.flow_invocations {
            let signature = invocation
                .history_signature
                .as_deref()
                .or_else(|| view_signatures.get(invocation.view_id.as_str()).copied());
            let Some(signature) = signature else {
                continue;
            };
            for binding in invocation.target_alias_bindings.values() {
                let super::PreparedTargetBinding::DynamicTexture(key) = binding else {
                    continue;
                };
                if let Some(existing) = signatures.get(key) {
                    if existing != signature {
                        diagnostics.push(self.diagnostic(
                            Some(key.to_string()),
                            Some(key.clone()),
                            Some(invocation.view_id.clone()),
                            Some(invocation.invocation_id.clone()),
                            RenderProductSurfaceRequestKind::HistorySignature,
                            RenderProductSurfaceDiagnosticSeverity::Error,
                            RenderProductSurfaceDiagnosticKind::ConflictingHistorySignature,
                            None,
                            format!(
                                "dynamic target '{}' has incompatible history signatures '{}' and '{}'",
                                key, existing, signature
                            ),
                        ));
                    }
                } else {
                    signatures.insert(key.clone(), signature.to_string());
                }
            }
        }
        diagnostics
    }

    fn status_diagnostics(&self) -> Vec<RenderProductSurfaceDiagnostic> {
        self.statuses
            .iter()
            .filter(|status| !status.status.is_ready())
            .map(|status| {
                self.diagnostic(
                    Some(status.surface_key.clone()),
                    None,
                    None,
                    None,
                    RenderProductSurfaceRequestKind::Status,
                    status.status.severity(),
                    RenderProductSurfaceDiagnosticKind::ProducerStatus,
                    Some(status.status),
                    status.message.clone(),
                )
            })
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn diagnostic(
        &self,
        surface_key: Option<String>,
        dynamic_target_key: Option<RenderDynamicTextureTargetKey>,
        view_id: Option<String>,
        invocation_id: Option<PreparedFlowInvocationId>,
        request_kind: RenderProductSurfaceRequestKind,
        severity: RenderProductSurfaceDiagnosticSeverity,
        diagnostic_kind: RenderProductSurfaceDiagnosticKind,
        status: Option<RenderProductSurfaceStatusKind>,
        message: String,
    ) -> RenderProductSurfaceDiagnostic {
        RenderProductSurfaceDiagnostic {
            producer_id: self.producer_id,
            product_family: self.product_family.clone(),
            surface_key,
            dynamic_target_key,
            view_id,
            invocation_id,
            request_kind,
            severity,
            diagnostic_kind,
            status,
            message,
        }
    }
}

fn product_binding_key(
    source: &ProductSurfaceTextureBindingSource,
) -> RenderDynamicTextureTargetKey {
    match source {
        ProductSurfaceTextureBindingSource::DynamicTexture {
            namespace,
            target_id,
        } => RenderDynamicTextureTargetKey::new(namespace.clone(), target_id.clone()),
    }
}

fn viewport_binding_key(source: &ViewportSurfaceBindingSource) -> RenderDynamicTextureTargetKey {
    match source {
        ViewportSurfaceBindingSource::DynamicTexture {
            namespace,
            target_id,
        } => RenderDynamicTextureTargetKey::new(namespace.clone(), target_id.clone()),
    }
}
