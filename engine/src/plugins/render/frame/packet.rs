use super::{
    PreparedFrameContext, PreparedFrameContributions, PreparedUiFrameContribution,
    PreparedViewFrame,
};
use crate::plugins::render::admission::RenderRepresentationAvailabilityFact;
use crate::plugins::render::request::RenderRequest;
use crate::plugins::render::scene::RenderSceneSnapshot;
use crate::plugins::render::surface_input::RenderSurfaceSemanticInputBinding;
use crate::plugins::render::{
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey,
    RenderDynamicTextureUploadDescriptor, RenderFlowId, RenderFrameProducerId,
    RenderGpuResourceAdapterError, RenderPassId, RenderTargetAliasKey, backend::RenderSurfaceId,
};
use crate::runtime::NativeWindowId;
use product::RenderProductSelection;
use runen_gpu::GpuWorkResourceId;
use std::collections::{BTreeMap, BTreeSet};
use ui_render_data::ViewportSurfaceBindingRegistry;

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct PreparedRenderFrameResource {
    frames: BTreeMap<RenderSurfaceId, PreparedRenderFrame>,
    next_frame_index: u64,
    next_prepare_epoch: u64,
}

/// Product-owned semantic work published for exactly one native render frame.
///
/// This is intentionally free of GPU objects and maintained-carrier details. The renderer admits
/// and lowers it only after the frame's dynamic targets have been realized, then composes its
/// typed work into the canonical frame graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderDeterministicFrameContribution {
    pub producer_id: RenderFrameProducerId,
    pub render_surface_id: RenderSurfaceId,
    pub scene: RenderSceneSnapshot,
    pub request: RenderRequest,
    pub semantic_inputs: Vec<RenderSurfaceSemanticInputBinding>,
    pub availability: Vec<RenderRepresentationAvailabilityFact>,
    pub output_index: usize,
    pub target_key: RenderDynamicTextureTargetKey,
}

/// Frame-scoped semantic contributions keyed by their owning producer.
///
/// Multiple surfaces/producers may be present in one frame. Each producer may publish one surface
/// per frame because the deterministic lowerer gives each producer one reusable physical-resource
/// namespace; repeated publication by the same producer replaces only that producer's contribution.
#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct RenderDeterministicFrameContributionResource {
    contributions: BTreeMap<RenderFrameProducerId, RenderDeterministicFrameContribution>,
}

impl RenderDeterministicFrameContributionResource {
    pub fn replace(&mut self, contribution: RenderDeterministicFrameContribution) {
        self.contributions
            .insert(contribution.producer_id, contribution);
    }

    pub fn remove(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
    ) -> Option<RenderDeterministicFrameContribution> {
        self.contributions.remove(&producer_id.into())
    }

    pub fn take_all(&mut self) -> Vec<RenderDeterministicFrameContribution> {
        std::mem::take(&mut self.contributions)
            .into_values()
            .collect()
    }
}

#[cfg(test)]
mod deterministic_contribution_tests {
    use super::*;
    use crate::plugins::render::request::{
        RenderObservationSpec, RenderOutputSpec, RenderOutputValue, RenderProbeObservation,
        RenderRadiometricRepresentation, RenderRequestedOutput, RenderResultTopology,
        RenderSamplingSupport, RenderSemanticTolerance,
    };
    use crate::plugins::render::scene::RenderSceneStore;
    use crate::plugins::render::space_time::{
        RenderAffineTransform3, RenderTimeInterval, RenderTimePoint,
    };

    fn producer(raw: u64) -> RenderFrameProducerId {
        RenderFrameProducerId::try_from_raw(raw).expect("test producer id should be nonzero")
    }

    fn contribution(producer_id: RenderFrameProducerId) -> RenderDeterministicFrameContribution {
        let shutter = RenderTimeInterval::instant(
            RenderTimePoint::from_seconds(0.0).expect("test time should be finite"),
        );
        let observation = RenderObservationSpec::Probe(
            RenderProbeObservation::new(
                RenderAffineTransform3::identity(),
                shutter,
                RenderSamplingSupport::ideal_ray(),
            )
            .expect("test observation should be valid"),
        );
        let output = RenderOutputSpec::new(
            RenderOutputValue::Radiance {
                representation: RenderRadiometricRepresentation::spectral_at_wavelength_meters(
                    550.0e-9,
                )
                .expect("test radiance representation should be valid"),
            },
            RenderResultTopology::scalar(),
            RenderSemanticTolerance::absolute(1.0e-4).expect("test tolerance should be valid"),
        )
        .expect("test output should be valid");
        RenderDeterministicFrameContribution {
            producer_id,
            render_surface_id: RenderSurfaceId::primary(),
            scene: RenderSceneStore::new().snapshot(),
            request: RenderRequest::new(
                shutter,
                vec![observation],
                vec![RenderRequestedOutput::new(0, output)],
            )
            .expect("test request should be valid"),
            semantic_inputs: Vec::new(),
            availability: Vec::new(),
            output_index: 0,
            target_key: RenderDynamicTextureTargetKey::new("test", "radiance"),
        }
    }

    #[test]
    fn deterministic_contributions_replace_by_producer_without_cross_surface_aliasing() {
        let mut resource = RenderDeterministicFrameContributionResource::default();
        resource.replace(contribution(producer(1)));
        resource.replace(contribution(producer(2)));
        resource.replace(contribution(producer(1)));
        assert_eq!(resource.take_all().len(), 2);
    }
}

impl PreparedRenderFrameResource {
    pub fn publish(&mut self, frame: PreparedRenderFrame) {
        self.next_frame_index = frame.context.frame_index.saturating_add(1);
        self.next_prepare_epoch = frame.context.prepare_epoch.saturating_add(1);
        self.frames.clear();
        self.frames.insert(frame.surface.render_surface_id, frame);
    }

    pub fn publish_set(&mut self, frames: impl IntoIterator<Item = PreparedRenderFrame>) {
        self.frames.clear();
        for frame in frames {
            self.next_frame_index = self
                .next_frame_index
                .max(frame.context.frame_index.saturating_add(1));
            self.next_prepare_epoch = self
                .next_prepare_epoch
                .max(frame.context.prepare_epoch.saturating_add(1));
            self.frames.insert(frame.surface.render_surface_id, frame);
        }
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn frame(&self) -> Option<&PreparedRenderFrame> {
        self.frames
            .get(&RenderSurfaceId::primary())
            .or_else(|| self.frames.values().next())
    }

    pub fn frames(&self) -> impl Iterator<Item = &PreparedRenderFrame> {
        self.frames.values()
    }

    pub fn take(&mut self) -> Option<PreparedRenderFrame> {
        let render_surface_id = if self.frames.contains_key(&RenderSurfaceId::primary()) {
            RenderSurfaceId::primary()
        } else {
            self.frames.keys().next().copied()?
        };
        self.frames.remove(&render_surface_id)
    }

    pub fn take_all(&mut self) -> Vec<PreparedRenderFrame> {
        std::mem::take(&mut self.frames).into_values().collect()
    }

    pub fn allocate_frame_index(&mut self) -> u64 {
        let frame_index = self.next_frame_index;
        self.next_frame_index = self.next_frame_index.saturating_add(1);
        frame_index
    }

    pub fn allocate_prepare_epoch(&mut self) -> u64 {
        let prepare_epoch = self.next_prepare_epoch;
        self.next_prepare_epoch = self.next_prepare_epoch.saturating_add(1);
        prepare_epoch
    }
}

#[derive(Debug, Clone)]
pub struct PreparedRenderFrame {
    pub context: PreparedFrameContext,
    pub surface: PreparedSurfaceInfo,
    pub views: Vec<PreparedViewFrame>,
    pub flows: BTreeMap<RenderFlowId, PreparedFlowInputs>,
    pub flow_invocations: Vec<PreparedFlowInvocation>,
    pub dynamic_texture_targets: Vec<RenderDynamicTextureTargetDescriptor>,
    pub dynamic_texture_uploads: Vec<RenderDynamicTextureUploadDescriptor>,
    pub product_selections: Vec<RenderProductSelection>,
    pub viewport_surface_bindings: ViewportSurfaceBindingRegistry,
    pub contributions: PreparedFrameContributions,
    pub shader: PreparedShaderSnapshot,
}

impl PreparedRenderFrame {
    pub fn flow_inputs(&self, flow_id: RenderFlowId) -> Option<&PreparedFlowInputs> {
        self.flows.get(&flow_id)
    }

    pub fn main_view(&self) -> Option<&PreparedViewFrame> {
        self.views
            .iter()
            .find(|view| matches!(view.kind, super::PreparedViewKind::MainSurface))
            .or_else(|| self.views.first())
    }

    pub fn view(&self, view_id: &str) -> Option<&PreparedViewFrame> {
        self.views.iter().find(|view| view.view_id == view_id)
    }

    pub fn flow_invocations_for_flow(
        &self,
        flow_id: RenderFlowId,
    ) -> impl Iterator<Item = &PreparedFlowInvocation> {
        self.flow_invocations
            .iter()
            .filter(move |invocation| invocation.flow_id == flow_id)
    }

    pub fn ui(&self) -> Option<&PreparedUiFrameContribution> {
        self.contributions.ui()
    }

    pub fn scene_route_labels(&self) -> Option<(&str, &str)> {
        self.contributions.scene_route_labels()
    }

    pub fn dynamic_target_history_signatures(
        &self,
    ) -> anyhow::Result<BTreeMap<RenderDynamicTextureTargetKey, String>> {
        let mut signatures = BTreeMap::<RenderDynamicTextureTargetKey, String>::new();
        for invocation in &self.flow_invocations {
            let view_signature = self
                .view(invocation.view_id.as_str())
                .and_then(|view| view.history_signature.as_ref());
            let Some(signature) = invocation
                .history_signature
                .as_ref()
                .or(view_signature)
                .cloned()
            else {
                continue;
            };
            for binding in invocation.target_alias_bindings.values() {
                let PreparedTargetBinding::DynamicTexture(key) = binding else {
                    continue;
                };
                if let Some(existing) = signatures.get(key)
                    && existing != &signature
                {
                    anyhow::bail!(
                        "dynamic target '{}' has incompatible history signatures '{}' and '{}'",
                        key,
                        existing,
                        signature
                    );
                }
                signatures.insert(key.clone(), signature.clone());
            }
        }
        Ok(signatures)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreparedSurfaceInfo {
    pub render_surface_id: RenderSurfaceId,
    pub native_window_id: Option<NativeWindowId>,
    pub target_size_px: (u32, u32),
}

impl PreparedSurfaceInfo {
    pub fn unbound_primary(target_size_px: (u32, u32)) -> Self {
        Self {
            render_surface_id: RenderSurfaceId::primary(),
            native_window_id: None,
            target_size_px,
        }
    }

    pub fn for_surface(
        render_surface_id: RenderSurfaceId,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
    ) -> Self {
        Self {
            render_surface_id,
            native_window_id: Some(native_window_id),
            target_size_px,
        }
    }

    pub fn target_size_px(&self) -> (u32, u32) {
        self.target_size_px
    }
}

#[derive(Debug, Clone, Default)]
pub struct PreparedFlowInputs {
    pub projected_uniform_bytes: BTreeMap<GpuWorkResourceId, Vec<u8>>,
    pub projected_dispatch_workgroups: BTreeMap<RenderPassId, [u32; 3]>,
    pub required_state_types: Vec<PreparedStateTypeInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreparedFlowInvocationId(pub String);

impl PreparedFlowInvocationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<String> for PreparedFlowInvocationId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for PreparedFlowInvocationId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for PreparedFlowInvocationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct PreparedFlowInvocation {
    pub invocation_id: PreparedFlowInvocationId,
    pub flow_id: RenderFlowId,
    pub view_id: String,
    pub inputs: PreparedFlowInputs,
    pub target_alias_bindings: BTreeMap<RenderTargetAliasKey, PreparedTargetBinding>,
    pub history_signature: Option<String>,
}

impl PreparedFlowInvocation {
    pub fn main(flow_id: RenderFlowId, inputs: PreparedFlowInputs) -> Self {
        Self {
            invocation_id: PreparedFlowInvocationId::new(format!("{flow_id}.main")),
            flow_id,
            view_id: "main".to_string(),
            inputs,
            target_alias_bindings: BTreeMap::new(),
            history_signature: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedTargetBinding {
    DynamicTexture(RenderDynamicTextureTargetKey),
    SurfaceColor,
    SurfaceDepth,
    FlowOwned(GpuWorkResourceId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFlowInvocationRequest {
    pub invocation_id: PreparedFlowInvocationId,
    pub flow_id: RenderFlowId,
    pub view_id: String,
    pub target_alias_bindings: BTreeMap<RenderTargetAliasKey, PreparedTargetBinding>,
    pub uniform_overrides: BTreeMap<GpuWorkResourceId, Vec<u8>>,
    pub history_signature: Option<String>,
}

impl PreparedFlowInvocationRequest {
    pub fn new(
        invocation_id: impl Into<PreparedFlowInvocationId>,
        flow_id: RenderFlowId,
        view_id: impl Into<String>,
    ) -> Self {
        Self {
            invocation_id: invocation_id.into(),
            flow_id,
            view_id: view_id.into(),
            target_alias_bindings: BTreeMap::new(),
            uniform_overrides: BTreeMap::new(),
            history_signature: None,
        }
    }

    pub fn bind_target_alias(
        mut self,
        alias: impl Into<String>,
        binding: PreparedTargetBinding,
    ) -> Result<Self, RenderGpuResourceAdapterError> {
        self.target_alias_bindings
            .insert(RenderTargetAliasKey::new(alias)?, binding);
        Ok(self)
    }

    pub fn bind_dynamic_texture_alias(
        self,
        alias: impl Into<String>,
        key: RenderDynamicTextureTargetKey,
    ) -> Result<Self, RenderGpuResourceAdapterError> {
        self.bind_target_alias(alias, PreparedTargetBinding::DynamicTexture(key))
    }

    pub fn bind_surface_color_alias(
        self,
        alias: impl Into<String>,
    ) -> Result<Self, RenderGpuResourceAdapterError> {
        self.bind_target_alias(alias, PreparedTargetBinding::SurfaceColor)
    }

    pub fn bind_surface_depth_alias(
        self,
        alias: impl Into<String>,
    ) -> Result<Self, RenderGpuResourceAdapterError> {
        self.bind_target_alias(alias, PreparedTargetBinding::SurfaceDepth)
    }

    pub fn bind_flow_owned_alias(
        self,
        alias: impl Into<String>,
        resource_id: GpuWorkResourceId,
    ) -> Result<Self, RenderGpuResourceAdapterError> {
        self.bind_target_alias(alias, PreparedTargetBinding::FlowOwned(resource_id))
    }

    pub fn with_history_signature(mut self, signature: impl Into<String>) -> Self {
        self.history_signature = Some(signature.into());
        self
    }

    pub fn with_uniform_override(mut self, uniform_id: GpuWorkResourceId, bytes: Vec<u8>) -> Self {
        self.uniform_overrides.insert(uniform_id, bytes);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedRenderFrameRequestKind {
    View,
    Invocation,
    AutomaticMainReplacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRenderFrameRequestDiagnostic {
    pub producer_id: RenderFrameProducerId,
    pub existing_producer_id: Option<RenderFrameProducerId>,
    pub view_id: Option<String>,
    pub invocation_id: Option<PreparedFlowInvocationId>,
    pub flow_id: Option<RenderFlowId>,
    pub request_kind: PreparedRenderFrameRequestKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PreparedRenderFrameRequestError {
    #[error(
        "prepared render frame producer {producer_id:?} publishes duplicate view '{view_id}' within one contribution"
    )]
    DuplicateViewWithinProducer {
        producer_id: RenderFrameProducerId,
        view_id: String,
    },
    #[error(
        "prepared render frame producer {producer_id:?} publishes view '{view_id}' already owned by producer {existing_producer_id:?}"
    )]
    DuplicateViewAcrossProducers {
        producer_id: RenderFrameProducerId,
        existing_producer_id: RenderFrameProducerId,
        view_id: String,
    },
    #[error(
        "prepared render frame producer {producer_id:?} publishes duplicate invocation '{invocation_id}' within one contribution"
    )]
    DuplicateInvocationWithinProducer {
        producer_id: RenderFrameProducerId,
        invocation_id: PreparedFlowInvocationId,
    },
    #[error(
        "prepared render frame producer {producer_id:?} publishes invocation '{invocation_id}' already owned by producer {existing_producer_id:?}"
    )]
    DuplicateInvocationAcrossProducers {
        producer_id: RenderFrameProducerId,
        existing_producer_id: RenderFrameProducerId,
        invocation_id: PreparedFlowInvocationId,
    },
    #[error(
        "prepared render frame producer {producer_id:?} claims automatic-main replacement for flow {flow_id:?} more than once"
    )]
    DuplicateAutomaticMainReplacementWithinProducer {
        producer_id: RenderFrameProducerId,
        flow_id: RenderFlowId,
    },
    #[error(
        "prepared render frame producer {producer_id:?} claims automatic-main replacement for flow {flow_id:?} already owned by producer {existing_producer_id:?}"
    )]
    DuplicateAutomaticMainReplacementAcrossProducers {
        producer_id: RenderFrameProducerId,
        existing_producer_id: RenderFrameProducerId,
        flow_id: RenderFlowId,
    },
    #[error(
        "prepared render frame producer {producer_id:?} claims automatic-main replacement for flow {flow_id:?} without an explicit invocation for that flow"
    )]
    MissingAutomaticMainReplacementInvocation {
        producer_id: RenderFrameProducerId,
        flow_id: RenderFlowId,
    },
}

impl PreparedRenderFrameRequestError {
    pub fn diagnostic(&self) -> PreparedRenderFrameRequestDiagnostic {
        match self {
            Self::DuplicateViewWithinProducer {
                producer_id,
                view_id,
            } => PreparedRenderFrameRequestDiagnostic {
                producer_id: *producer_id,
                existing_producer_id: None,
                view_id: Some(view_id.clone()),
                invocation_id: None,
                flow_id: None,
                request_kind: PreparedRenderFrameRequestKind::View,
                message: self.to_string(),
            },
            Self::DuplicateViewAcrossProducers {
                producer_id,
                existing_producer_id,
                view_id,
            } => PreparedRenderFrameRequestDiagnostic {
                producer_id: *producer_id,
                existing_producer_id: Some(*existing_producer_id),
                view_id: Some(view_id.clone()),
                invocation_id: None,
                flow_id: None,
                request_kind: PreparedRenderFrameRequestKind::View,
                message: self.to_string(),
            },
            Self::DuplicateInvocationWithinProducer {
                producer_id,
                invocation_id,
            } => PreparedRenderFrameRequestDiagnostic {
                producer_id: *producer_id,
                existing_producer_id: None,
                view_id: None,
                invocation_id: Some(invocation_id.clone()),
                flow_id: None,
                request_kind: PreparedRenderFrameRequestKind::Invocation,
                message: self.to_string(),
            },
            Self::DuplicateInvocationAcrossProducers {
                producer_id,
                existing_producer_id,
                invocation_id,
            } => PreparedRenderFrameRequestDiagnostic {
                producer_id: *producer_id,
                existing_producer_id: Some(*existing_producer_id),
                view_id: None,
                invocation_id: Some(invocation_id.clone()),
                flow_id: None,
                request_kind: PreparedRenderFrameRequestKind::Invocation,
                message: self.to_string(),
            },
            Self::DuplicateAutomaticMainReplacementWithinProducer {
                producer_id,
                flow_id,
            }
            | Self::MissingAutomaticMainReplacementInvocation {
                producer_id,
                flow_id,
            } => PreparedRenderFrameRequestDiagnostic {
                producer_id: *producer_id,
                existing_producer_id: None,
                view_id: None,
                invocation_id: None,
                flow_id: Some(*flow_id),
                request_kind: PreparedRenderFrameRequestKind::AutomaticMainReplacement,
                message: self.to_string(),
            },
            Self::DuplicateAutomaticMainReplacementAcrossProducers {
                producer_id,
                existing_producer_id,
                flow_id,
            } => PreparedRenderFrameRequestDiagnostic {
                producer_id: *producer_id,
                existing_producer_id: Some(*existing_producer_id),
                view_id: None,
                invocation_id: None,
                flow_id: Some(*flow_id),
                request_kind: PreparedRenderFrameRequestKind::AutomaticMainReplacement,
                message: self.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct PreparedRenderFrameRequestResource {
    contributions: BTreeMap<RenderFrameProducerId, PreparedRenderFrameRequestContribution>,
    diagnostics: Vec<PreparedRenderFrameRequestDiagnostic>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RenderFrameSurfaceScope {
    #[default]
    AllSurfaces,
    Surface(RenderSurfaceId),
}

impl RenderFrameSurfaceScope {
    pub fn applies_to(self, render_surface_id: RenderSurfaceId) -> bool {
        matches!(self, Self::AllSurfaces)
            || matches!(self, Self::Surface(surface_id) if surface_id == render_surface_id)
    }

    pub fn overlaps(self, other: Self) -> bool {
        match (self, other) {
            (Self::AllSurfaces, _) | (_, Self::AllSurfaces) => true,
            (Self::Surface(left), Self::Surface(right)) => left == right,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PreparedRenderFrameRequestContribution {
    scope: RenderFrameSurfaceScope,
    views: BTreeMap<String, PreparedViewFrame>,
    flow_invocations: Vec<PreparedFlowInvocationRequest>,
    automatic_main_replacements: BTreeSet<RenderFlowId>,
}

impl PreparedRenderFrameRequestResource {
    pub fn clear(&mut self) {
        self.contributions.clear();
        self.diagnostics.clear();
    }

    pub fn remove_contribution(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
    ) -> Option<PreparedRenderFrameRequestContribution> {
        let producer_id = producer_id.into();
        let removed = self.contributions.remove(&producer_id);
        self.clear_diagnostics_for_producer(&producer_id);
        removed
    }

    pub fn replace_contribution(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
        views: impl IntoIterator<Item = PreparedViewFrame>,
        flow_invocations: impl IntoIterator<Item = PreparedFlowInvocationRequest>,
    ) -> Result<Option<PreparedRenderFrameRequestContribution>, PreparedRenderFrameRequestError>
    {
        self.replace_contribution_with_automatic_main_replacements(
            producer_id,
            views,
            flow_invocations,
            std::iter::empty::<RenderFlowId>(),
        )
    }

    pub fn replace_contribution_with_automatic_main_replacements(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
        views: impl IntoIterator<Item = PreparedViewFrame>,
        flow_invocations: impl IntoIterator<Item = PreparedFlowInvocationRequest>,
        automatic_main_replacements: impl IntoIterator<Item = RenderFlowId>,
    ) -> Result<Option<PreparedRenderFrameRequestContribution>, PreparedRenderFrameRequestError>
    {
        self.replace_scoped_contribution(
            producer_id.into(),
            RenderFrameSurfaceScope::AllSurfaces,
            views,
            flow_invocations,
            automatic_main_replacements,
        )
    }

    pub fn replace_surface_contribution_with_automatic_main_replacements(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
        render_surface_id: RenderSurfaceId,
        views: impl IntoIterator<Item = PreparedViewFrame>,
        flow_invocations: impl IntoIterator<Item = PreparedFlowInvocationRequest>,
        automatic_main_replacements: impl IntoIterator<Item = RenderFlowId>,
    ) -> Result<Option<PreparedRenderFrameRequestContribution>, PreparedRenderFrameRequestError>
    {
        self.replace_scoped_contribution(
            producer_id.into(),
            RenderFrameSurfaceScope::Surface(render_surface_id),
            views,
            flow_invocations,
            automatic_main_replacements,
        )
    }

    fn replace_scoped_contribution(
        &mut self,
        producer_id: RenderFrameProducerId,
        scope: RenderFrameSurfaceScope,
        views: impl IntoIterator<Item = PreparedViewFrame>,
        flow_invocations: impl IntoIterator<Item = PreparedFlowInvocationRequest>,
        automatic_main_replacements: impl IntoIterator<Item = RenderFlowId>,
    ) -> Result<Option<PreparedRenderFrameRequestContribution>, PreparedRenderFrameRequestError>
    {
        self.clear_diagnostics_for_producer(&producer_id);
        let contribution = match PreparedRenderFrameRequestContribution::from_requests(
            &producer_id,
            scope,
            views,
            flow_invocations,
            automatic_main_replacements,
        ) {
            Ok(contribution) => contribution,
            Err(error) => {
                self.record_error(&error);
                return Err(error);
            }
        };
        if let Err(error) = self.validate_replacement(&producer_id, &contribution) {
            self.record_error(&error);
            return Err(error);
        }
        Ok(self.contributions.insert(producer_id, contribution))
    }

    pub fn diagnostics(&self) -> &[PreparedRenderFrameRequestDiagnostic] {
        &self.diagnostics
    }

    pub fn requested_views(&self) -> Vec<&PreparedViewFrame> {
        self.contributions
            .values()
            .flat_map(|contribution| contribution.views.values())
            .collect()
    }

    pub fn requested_views_for_surface(
        &self,
        render_surface_id: RenderSurfaceId,
    ) -> Vec<&PreparedViewFrame> {
        self.contributions
            .values()
            .filter(|contribution| contribution.scope.applies_to(render_surface_id))
            .flat_map(|contribution| contribution.views.values())
            .collect()
    }

    pub fn requested_flow_invocations(&self) -> Vec<&PreparedFlowInvocationRequest> {
        self.contributions
            .values()
            .flat_map(|contribution| contribution.flow_invocations.iter())
            .collect()
    }

    pub fn requested_flow_invocations_for_surface(
        &self,
        render_surface_id: RenderSurfaceId,
    ) -> Vec<&PreparedFlowInvocationRequest> {
        self.contributions
            .values()
            .filter(|contribution| contribution.scope.applies_to(render_surface_id))
            .flat_map(|contribution| contribution.flow_invocations.iter())
            .collect()
    }

    pub fn replaces_automatic_main_flow(
        &self,
        render_surface_id: RenderSurfaceId,
        flow_id: RenderFlowId,
    ) -> bool {
        self.contributions.values().any(|contribution| {
            contribution.scope.applies_to(render_surface_id)
                && contribution.automatic_main_replacements.contains(&flow_id)
        })
    }

    pub fn is_empty(&self) -> bool {
        self.contributions.is_empty()
    }

    fn validate_replacement(
        &self,
        producer_id: &RenderFrameProducerId,
        replacement: &PreparedRenderFrameRequestContribution,
    ) -> Result<(), PreparedRenderFrameRequestError> {
        let mut view_ids = BTreeMap::<&str, &RenderFrameProducerId>::new();
        let mut invocation_ids =
            BTreeMap::<&PreparedFlowInvocationId, &RenderFrameProducerId>::new();
        let mut automatic_main_replacements =
            BTreeMap::<RenderFlowId, &RenderFrameProducerId>::new();

        for (existing_producer_id, contribution) in &self.contributions {
            if existing_producer_id == producer_id
                || !replacement.scope.overlaps(contribution.scope)
            {
                continue;
            }
            for view_id in contribution.views.keys() {
                view_ids.insert(view_id.as_str(), existing_producer_id);
            }
            for request in &contribution.flow_invocations {
                invocation_ids.insert(&request.invocation_id, existing_producer_id);
            }
            for flow_id in &contribution.automatic_main_replacements {
                automatic_main_replacements.insert(*flow_id, existing_producer_id);
            }
        }

        for view_id in replacement.views.keys() {
            if let Some(existing_producer_id) = view_ids.get(view_id.as_str()) {
                return Err(
                    PreparedRenderFrameRequestError::DuplicateViewAcrossProducers {
                        producer_id: *producer_id,
                        existing_producer_id: **existing_producer_id,
                        view_id: view_id.clone(),
                    },
                );
            }
        }
        for request in &replacement.flow_invocations {
            if let Some(existing_producer_id) = invocation_ids.get(&request.invocation_id) {
                return Err(
                    PreparedRenderFrameRequestError::DuplicateInvocationAcrossProducers {
                        producer_id: *producer_id,
                        existing_producer_id: **existing_producer_id,
                        invocation_id: request.invocation_id.clone(),
                    },
                );
            }
        }
        for flow_id in &replacement.automatic_main_replacements {
            if let Some(existing_producer_id) = automatic_main_replacements.get(flow_id) {
                return Err(
                    PreparedRenderFrameRequestError::DuplicateAutomaticMainReplacementAcrossProducers {
                        producer_id: *producer_id,
                        existing_producer_id: **existing_producer_id,
                        flow_id: *flow_id,
                    },
                );
            }
        }

        Ok(())
    }

    fn record_error(&mut self, error: &PreparedRenderFrameRequestError) {
        self.diagnostics.push(error.diagnostic());
    }

    fn clear_diagnostics_for_producer(&mut self, producer_id: &RenderFrameProducerId) {
        self.diagnostics.retain(|diagnostic| {
            diagnostic.producer_id != *producer_id
                && diagnostic.existing_producer_id != Some(*producer_id)
        });
    }
}

impl PreparedRenderFrameRequestContribution {
    fn from_requests(
        producer_id: &RenderFrameProducerId,
        scope: RenderFrameSurfaceScope,
        views: impl IntoIterator<Item = PreparedViewFrame>,
        flow_invocations: impl IntoIterator<Item = PreparedFlowInvocationRequest>,
        automatic_main_replacements: impl IntoIterator<Item = RenderFlowId>,
    ) -> Result<Self, PreparedRenderFrameRequestError> {
        let mut view_map = BTreeMap::<String, PreparedViewFrame>::new();
        for view in views {
            let view_id = view.view_id.clone();
            if view_map.insert(view_id.clone(), view).is_some() {
                return Err(
                    PreparedRenderFrameRequestError::DuplicateViewWithinProducer {
                        producer_id: *producer_id,
                        view_id,
                    },
                );
            }
        }

        let mut invocation_ids = BTreeSet::<PreparedFlowInvocationId>::new();
        let flow_invocations = flow_invocations.into_iter().collect::<Vec<_>>();
        for request in &flow_invocations {
            if !invocation_ids.insert(request.invocation_id.clone()) {
                return Err(
                    PreparedRenderFrameRequestError::DuplicateInvocationWithinProducer {
                        producer_id: *producer_id,
                        invocation_id: request.invocation_id.clone(),
                    },
                );
            }
        }

        let mut replacement_flows = BTreeSet::<RenderFlowId>::new();
        for flow_id in automatic_main_replacements {
            if !replacement_flows.insert(flow_id) {
                return Err(
                    PreparedRenderFrameRequestError::DuplicateAutomaticMainReplacementWithinProducer {
                        producer_id: *producer_id,
                        flow_id,
                    },
                );
            }
        }
        for flow_id in &replacement_flows {
            if !flow_invocations
                .iter()
                .any(|request| request.flow_id == *flow_id)
            {
                return Err(
                    PreparedRenderFrameRequestError::MissingAutomaticMainReplacementInvocation {
                        producer_id: *producer_id,
                        flow_id: *flow_id,
                    },
                );
            }
        }

        Ok(Self {
            scope,
            views: view_map,
            flow_invocations,
            automatic_main_replacements: replacement_flows,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedStateTypeInfo {
    pub type_name: &'static str,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PreparedShaderSnapshot {
    pub registry_revision: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_frame(index: u64) -> PreparedRenderFrame {
        PreparedRenderFrame {
            context: PreparedFrameContext {
                frame_index: index,
                flow_registry_revision: 7,
                shader_registry_revision: 11,
                prepare_epoch: 3,
            },
            surface: PreparedSurfaceInfo::unbound_primary((1280, 720)),
            views: vec![PreparedViewFrame::main((1280, 720))],
            flows: BTreeMap::new(),
            flow_invocations: Vec::new(),
            dynamic_texture_targets: Vec::new(),
            dynamic_texture_uploads: Vec::new(),
            product_selections: Vec::new(),
            viewport_surface_bindings: ViewportSurfaceBindingRegistry::default(),
            contributions: PreparedFrameContributions::default(),
            shader: PreparedShaderSnapshot {
                registry_revision: 11,
            },
        }
    }

    fn producer(raw: u64) -> RenderFrameProducerId {
        RenderFrameProducerId::try_from_raw(raw).expect("test producer id should be nonzero")
    }

    fn flow(raw: u64) -> RenderFlowId {
        RenderFlowId::try_from_raw(raw).expect("test flow id should be nonzero")
    }

    #[test]
    fn automatic_main_replacement_requires_explicit_invocation_for_same_flow() {
        let mut requests = PreparedRenderFrameRequestResource::default();
        let flow_id = flow(7);

        let error = requests
            .replace_contribution_with_automatic_main_replacements(producer(1), [], [], [flow_id])
            .expect_err("replacement without invocation must fail closed");

        assert!(matches!(
            error,
            PreparedRenderFrameRequestError::MissingAutomaticMainReplacementInvocation {
                flow_id: rejected,
                ..
            } if rejected == flow_id
        ));
        assert_eq!(
            requests.diagnostics()[0].request_kind,
            PreparedRenderFrameRequestKind::AutomaticMainReplacement
        );
        assert_eq!(requests.diagnostics()[0].flow_id, Some(flow_id));
    }

    #[test]
    fn automatic_main_replacement_is_single_owner_per_flow() {
        let mut requests = PreparedRenderFrameRequestResource::default();
        let flow_id = flow(7);
        let first = PreparedFlowInvocationRequest::new("fixed.first", flow_id, "fixed.first.view");
        requests
            .replace_contribution_with_automatic_main_replacements(
                producer(1),
                [PreparedViewFrame::offscreen_product(
                    "fixed.first.view",
                    (1280, 720),
                )],
                [first],
                [flow_id],
            )
            .expect("first replacement owner should be admitted");

        let second =
            PreparedFlowInvocationRequest::new("fixed.second", flow_id, "fixed.second.view");
        let error = requests
            .replace_contribution_with_automatic_main_replacements(
                producer(2),
                [PreparedViewFrame::offscreen_product(
                    "fixed.second.view",
                    (1280, 720),
                )],
                [second],
                [flow_id],
            )
            .expect_err("second replacement owner must fail closed");

        assert!(matches!(
            error,
            PreparedRenderFrameRequestError::DuplicateAutomaticMainReplacementAcrossProducers {
                flow_id: rejected,
                ..
            } if rejected == flow_id
        ));
        assert!(requests.replaces_automatic_main_flow(RenderSurfaceId::primary(), flow_id));
        assert!(!requests.replaces_automatic_main_flow(RenderSurfaceId::primary(), flow(8)));
    }

    #[test]
    fn surface_scoped_requests_allow_independent_replacement_on_distinct_surfaces() {
        let mut requests = PreparedRenderFrameRequestResource::default();
        let flow_id = flow(7);
        let primary = RenderSurfaceId::primary();
        let secondary = RenderSurfaceId::try_from_raw(2).expect("test surface should be nonzero");

        for (producer_id, surface_id) in [(producer(1), primary), (producer(2), secondary)] {
            requests
                .replace_surface_contribution_with_automatic_main_replacements(
                    producer_id,
                    surface_id,
                    [PreparedViewFrame::offscreen_product(
                        "fixed.shared.view",
                        (1280, 720),
                    )],
                    [PreparedFlowInvocationRequest::new(
                        "fixed.shared",
                        flow_id,
                        "fixed.shared.view",
                    )],
                    [flow_id],
                )
                .expect("disjoint surface scopes should admit identical local identities");
        }

        assert_eq!(requests.requested_views_for_surface(primary).len(), 1);
        assert_eq!(requests.requested_views_for_surface(secondary).len(), 1);
        assert_eq!(
            requests
                .requested_flow_invocations_for_surface(primary)
                .len(),
            1
        );
        assert_eq!(
            requests
                .requested_flow_invocations_for_surface(secondary)
                .len(),
            1
        );
        assert!(requests.replaces_automatic_main_flow(primary, flow_id));
        assert!(requests.replaces_automatic_main_flow(secondary, flow_id));
    }

    #[test]
    fn prepared_frame_resource_allocates_monotonic_indices() {
        let mut resource = PreparedRenderFrameResource::default();
        assert_eq!(resource.allocate_frame_index(), 0);
        assert_eq!(resource.allocate_frame_index(), 1);
    }

    #[test]
    fn prepared_frame_resource_publish_and_take_roundtrip() {
        let mut resource = PreparedRenderFrameResource::default();
        resource.publish(dummy_frame(4));
        assert_eq!(
            resource
                .frame()
                .expect("frame should be present after publish")
                .context
                .frame_index,
            4
        );
        assert_eq!(resource.allocate_frame_index(), 5);

        let taken = resource.take().expect("take should return a frame");
        assert_eq!(taken.context.frame_index, 4);
        assert!(resource.frame().is_none());
    }
}
