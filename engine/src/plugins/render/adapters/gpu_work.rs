//! Temporary G3/G5 bridge from fully resolved render execution operations to backend-neutral
//! RunenGPU work.
//!
//! G5A deliberately requires this adapter to receive execution-complete `GpuWorkOperation`
//! values. It does not allocate logical GPU resources, reconstruct operation accesses, invent
//! pipeline/binding state, or project renderer declarations into a second GPU identity space.
//! The prepared graph remains the only access, initialization, hazard, capability, dependency,
//! and topological-order authority. The private sidecar contains only later-phase render
//! execution payload.

use crate::plugins::gpu::*;
use crate::plugins::render::RenderPassId;
use crate::plugins::render::graph::{CompiledPassExecutionPlan, CompiledRenderFlowPlan};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum RenderGpuWorkAdapterError {
    #[error(transparent)]
    Descriptor(#[from] GpuResourceDescriptorError),
    #[error(transparent)]
    Operation(#[from] GpuWorkOperationError),
    #[error(transparent)]
    Authoring(#[from] GpuWorkAuthoringError),
    #[error(transparent)]
    Graph(#[from] GpuWorkGraphError),
    #[error(
        "resolved render GPU work reuses logical resource identity '{resource_id}' for incompatible kind-preserving handles"
    )]
    ResourceIdentityConflict { resource_id: GpuWorkResourceId },
    #[error("resolved render GPU work contains duplicate execution payload for pass '{pass_id}'")]
    DuplicatePass { pass_id: RenderPassId },
    #[error(
        "resolved render non-data order references pass '{pass_id}' that is absent from this invocation's GPU work"
    )]
    MissingOrderedPass { pass_id: RenderPassId },
    #[error("prepared render node '{node_id}' has no execution sidecar payload")]
    MissingSidecarPayload { node_id: GpuPreparedWorkNodeId },
    #[error("prepared render node '{node_id}' received duplicate execution sidecar payload")]
    DuplicateSidecarPayload { node_id: GpuPreparedWorkNodeId },
    #[error("prepared node '{node_id}' does not belong to this render work graph")]
    ForeignPreparedNode { node_id: GpuPreparedWorkNodeId },
    #[error(
        "prepared render node '{node_id}' operation kind {actual:?} disagrees with sidecar payload kind {expected:?}"
    )]
    SidecarOperationKindMismatch {
        node_id: GpuPreparedWorkNodeId,
        expected: GpuWorkNodeKind,
        actual: GpuWorkNodeKind,
    },
    #[error("render GPU-work sidecar could not map fragment-local node {local_node}")]
    MissingPreparedNodeMapping { local_node: u64 },
}

/// Execution-only payload associated with one prepared RunenGPU node.
#[derive(Debug, Clone)]
pub(crate) enum RenderGpuWorkPayload {
    Pass(Box<CompiledPassExecutionPlan>),
    TimingResolve,
    TimingReadbackCopy,
}

impl RenderGpuWorkPayload {
    fn operation_kind(&self) -> GpuWorkNodeKind {
        match self {
            Self::Pass(pass) => match pass.as_ref() {
                CompiledPassExecutionPlan::Compute(_) => GpuWorkNodeKind::Compute,
                CompiledPassExecutionPlan::Fullscreen(_)
                | CompiledPassExecutionPlan::Graphics(_)
                | CompiledPassExecutionPlan::BuiltinUiComposite(_) => GpuWorkNodeKind::Render,
                CompiledPassExecutionPlan::Copy(_) => GpuWorkNodeKind::Copy,
                CompiledPassExecutionPlan::Present(_) => GpuWorkNodeKind::Present,
            },
            Self::TimingResolve => GpuWorkNodeKind::Resolve,
            Self::TimingReadbackCopy => GpuWorkNodeKind::Copy,
        }
    }

    fn pass_id(&self) -> Option<RenderPassId> {
        match self {
            Self::Pass(pass) => Some(execution_pass_id(pass.as_ref())),
            Self::TimingResolve | Self::TimingReadbackCopy => None,
        }
    }
}

/// One execution-complete renderer node ready for G3 graph preparation.
///
/// The operation is already the semantic authority for accesses and mechanical capability
/// requirements. `preference` is scheduling preference only; it cannot weaken operation facts.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedRenderGpuWorkNode {
    label: GpuResourceLabel,
    operation: GpuWorkOperation,
    preference: GpuExecutionPreference,
    provenance: GpuResourceProvenance,
    payload: RenderGpuWorkPayload,
}

impl ResolvedRenderGpuWorkNode {
    pub(crate) fn pass(
        label: GpuResourceLabel,
        pass: CompiledPassExecutionPlan,
        operation: GpuWorkOperation,
        preference: GpuExecutionPreference,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            label,
            operation,
            preference,
            provenance,
            payload: RenderGpuWorkPayload::Pass(Box::new(pass)),
        }
    }

    pub(crate) fn timing_resolve(
        label: GpuResourceLabel,
        operation: GpuQueryResolveOperation,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            label,
            operation: GpuWorkOperation::Resolve(operation),
            preference: GpuExecutionPreference::TransferPreferred,
            provenance,
            payload: RenderGpuWorkPayload::TimingResolve,
        }
    }

    pub(crate) fn timing_readback_copy(
        label: GpuResourceLabel,
        operation: GpuCopyOperation,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            label,
            operation: GpuWorkOperation::Copy(operation),
            preference: GpuExecutionPreference::TransferPreferred,
            provenance,
            payload: RenderGpuWorkPayload::TimingReadbackCopy,
        }
    }
}

/// Private execution-only payload keyed solely by prepared G3 node identity.
/// It deliberately stores no access, hazard, initialization, capability, dependency, export,
/// diagnostic, or ordering data.
#[derive(Debug, Clone, Default)]
struct RenderGpuWorkSidecar {
    entries: BTreeMap<GpuPreparedWorkNodeId, RenderGpuWorkPayload>,
}

impl RenderGpuWorkSidecar {
    fn insert(
        &mut self,
        graph: &GpuPreparedWorkGraph,
        node_id: GpuPreparedWorkNodeId,
        payload: RenderGpuWorkPayload,
    ) -> Result<(), RenderGpuWorkAdapterError> {
        let Some(node) = graph.nodes().iter().find(|node| node.id() == node_id) else {
            return Err(RenderGpuWorkAdapterError::ForeignPreparedNode { node_id });
        };
        let expected = payload.operation_kind();
        let actual = node.node().kind();
        if expected != actual {
            return Err(RenderGpuWorkAdapterError::SidecarOperationKindMismatch {
                node_id,
                expected,
                actual,
            });
        }
        if self.entries.contains_key(&node_id) {
            return Err(RenderGpuWorkAdapterError::DuplicateSidecarPayload { node_id });
        }
        self.entries.insert(node_id, payload);
        Ok(())
    }

    fn finish(self, graph: &GpuPreparedWorkGraph) -> Result<Self, RenderGpuWorkAdapterError> {
        for node in graph.nodes() {
            if !self.entries.contains_key(&node.id()) {
                return Err(RenderGpuWorkAdapterError::MissingSidecarPayload {
                    node_id: node.id(),
                });
            }
        }
        if let Some(node_id) = self
            .entries
            .keys()
            .find(|id| graph.nodes().iter().all(|node| node.id() != **id))
            .copied()
        {
            return Err(RenderGpuWorkAdapterError::ForeignPreparedNode { node_id });
        }
        Ok(self)
    }

    fn get(
        &self,
        graph: &GpuPreparedWorkGraph,
        node_id: GpuPreparedWorkNodeId,
    ) -> Result<&RenderGpuWorkPayload, RenderGpuWorkAdapterError> {
        if graph.nodes().iter().all(|node| node.id() != node_id) {
            return Err(RenderGpuWorkAdapterError::ForeignPreparedNode { node_id });
        }
        self.entries
            .get(&node_id)
            .ok_or(RenderGpuWorkAdapterError::MissingSidecarPayload { node_id })
    }
}

/// Per-invocation G3 work plus its temporary render execution sidecar.
///
/// Prepared node IDs are process-local references only. This value must never be persisted or
/// used as a stable cache, replay, wire, or cross-process key.
#[derive(Debug, Clone)]
pub struct PreparedRenderWorkPlan {
    graph: GpuPreparedWorkGraph,
    sidecar: RenderGpuWorkSidecar,
}

impl PreparedRenderWorkPlan {
    pub fn graph(&self) -> &GpuPreparedWorkGraph {
        &self.graph
    }

    pub(crate) fn payload(
        &self,
        node_id: GpuPreparedWorkNodeId,
    ) -> Result<&RenderGpuWorkPayload, RenderGpuWorkAdapterError> {
        self.sidecar.get(&self.graph, node_id)
    }

    pub(crate) fn ordered_payloads(
        &self,
    ) -> Result<Vec<(GpuPreparedWorkNodeId, &RenderGpuWorkPayload)>, RenderGpuWorkAdapterError>
    {
        self.graph
            .topological_order()
            .iter()
            .copied()
            .map(|node_id| self.payload(node_id).map(|payload| (node_id, payload)))
            .collect()
    }

    pub fn ordered_render_pass_ids(&self) -> Result<Vec<RenderPassId>, RenderGpuWorkAdapterError> {
        self.ordered_payloads().map(|entries| {
            entries
                .into_iter()
                .filter_map(|(_, payload)| payload.pass_id())
                .collect()
        })
    }
}

/// Prepares one renderer invocation from execution-complete logical GPU operations.
///
/// All kind-preserving resources are discovered from operation-derived accesses. Every non-query
/// resource is supplied to G3R with descriptor initialization coverage, so an uninitialized
/// descriptor remains uninitialized and a Prepared/Zeroed descriptor contributes only the
/// coverage already owned by RunenGPU. Caller-declared duplicate access truth is intentionally
/// absent.
pub(crate) fn prepare_render_gpu_work(
    plan: &CompiledRenderFlowPlan,
    nodes: impl IntoIterator<Item = ResolvedRenderGpuWorkNode>,
) -> Result<PreparedRenderWorkPlan, RenderGpuWorkAdapterError> {
    let nodes = nodes.into_iter().collect::<Vec<_>>();
    let graph_label = GpuResourceLabel::new(format!("render.flow.{}.work", plan.flow_id))?;
    let graph_provenance = GpuResourceProvenance::new(graph_label.clone(), None, None);
    let resources = collect_operation_resources(&nodes)?;
    let inputs = resources
        .values()
        .filter(|resource| !matches!(resource, GpuResourceRef::QuerySet(_)))
        .map(|resource| {
            Ok(GpuWorkResourceInput::new(
                resource.clone(),
                GpuInitialCoverage::descriptor_initialization(resource.clone())?,
                resource.common().provenance().clone(),
            )?)
        })
        .collect::<Result<Vec<_>, RenderGpuWorkAdapterError>>()?;

    let mut pending = Vec::<(GpuWorkNodeId, RenderGpuWorkPayload)>::new();
    let fragment = GpuWorkFragment::build_with_provenance(
        graph_label.clone(),
        graph_provenance.clone(),
        |builder| {
            for resource in resources.values() {
                builder.declare_resource(resource.clone())?;
            }
            for input in &inputs {
                builder.add_input(input.clone())?;
            }

            let mut pass_nodes = BTreeMap::<RenderPassId, GpuWorkNodeId>::new();
            for node in &nodes {
                let node_id = builder.add_node(
                    node.label.clone(),
                    node.operation.clone(),
                    [],
                    GpuCapabilityRequirements::new(),
                    node.preference,
                    node.provenance.clone(),
                )?;
                if let Some(pass_id) = node.payload.pass_id()
                    && pass_nodes.insert(pass_id, node_id.clone()).is_some()
                {
                    return Err(GpuWorkAuthoringError::invalid(
                        "author resolved render GPU work",
                        GpuWorkAuthoringErrorContext::new(
                            Some(graph_label.as_str().to_string()),
                            Some(node.label.as_str().to_string()),
                            Some(node_id),
                            None,
                            Some(node.provenance.clone()),
                        ),
                        GpuWorkAuthoringCause::DuplicateNodeIdentity,
                        "provide exactly one execution-complete GPU operation per render pass",
                    ));
                }
                pending.push((node_id, node.payload.clone()));
            }

            for pass in &plan.render_passes {
                let after_id = pass.pass_id();
                let Some(after) = pass_nodes.get(&after_id) else {
                    continue;
                };
                for before_id in &pass.node().non_data_order_after {
                    let before = pass_nodes.get(before_id).ok_or_else(|| {
                        GpuWorkAuthoringError::invalid(
                            "author resolved render non-data order",
                            GpuWorkAuthoringErrorContext::new(
                                Some(graph_label.as_str().to_string()),
                                Some(pass.pass_label().to_string()),
                                Some(after.clone()),
                                None,
                                Some(graph_provenance.clone()),
                            ),
                            GpuWorkAuthoringCause::UnknownIdentity,
                            "include every pass referenced by a non-data order in this invocation's resolved GPU work",
                        )
                    })?;
                    builder.add_explicit_order(GpuExplicitOrder::new(
                        before,
                        after,
                        "render-owned non-data order",
                    )?)?;
                }
            }
            Ok(())
        },
    )?;

    let graph = GpuPreparedWorkGraph::prepare(graph_label, [fragment])?;
    let prepared_by_local = graph
        .nodes()
        .iter()
        .map(|node| (node.id().local_node(), node.id()))
        .collect::<BTreeMap<_, _>>();
    let mut sidecar = RenderGpuWorkSidecar::default();
    for (node_id, payload) in pending {
        let local = node_id.diagnostic_local();
        let prepared_id = prepared_by_local
            .get(&local)
            .copied()
            .ok_or(RenderGpuWorkAdapterError::MissingPreparedNodeMapping { local_node: local })?;
        sidecar.insert(&graph, prepared_id, payload)?;
    }

    Ok(PreparedRenderWorkPlan {
        sidecar: sidecar.finish(&graph)?,
        graph,
    })
}

fn collect_operation_resources(
    nodes: &[ResolvedRenderGpuWorkNode],
) -> Result<BTreeMap<GpuWorkResourceId, GpuResourceRef>, RenderGpuWorkAdapterError> {
    let mut resources = BTreeMap::new();
    for node in nodes {
        for access in node.operation.derived_accesses()? {
            let resource = declared_resource_for_access(&access);
            let identity = resource.diagnostic_identity();
            match resources.get(&identity) {
                Some(existing) if existing != &resource => {
                    return Err(RenderGpuWorkAdapterError::ResourceIdentityConflict {
                        resource_id: identity,
                    });
                }
                Some(_) => {}
                None => {
                    resources.insert(identity, resource);
                }
            }
        }
    }
    Ok(resources)
}

fn declared_resource_for_access(access: &GpuResourceAccess) -> GpuResourceRef {
    match access {
        GpuResourceAccess::Buffer(access) => GpuResourceRef::Buffer(access.buffer().clone()),
        GpuResourceAccess::Texture(access) => match access.resource() {
            GpuTextureAccessResource::Texture(texture) => GpuResourceRef::Texture(texture.clone()),
            GpuTextureAccessResource::TextureView(view) => {
                GpuResourceRef::TextureView(view.clone())
            }
        },
        GpuResourceAccess::Query(access) => GpuResourceRef::QuerySet(access.query_set().clone()),
        GpuResourceAccess::Sampler(access) => GpuResourceRef::Sampler(access.sampler().clone()),
    }
}

fn execution_pass_id(pass: &CompiledPassExecutionPlan) -> RenderPassId {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.pass_id,
        CompiledPassExecutionPlan::Fullscreen(value) => value.pass_id,
        CompiledPassExecutionPlan::Graphics(value) => value.pass_id,
        CompiledPassExecutionPlan::Copy(value) => value.pass_id,
        CompiledPassExecutionPlan::Present(value) => value.pass_id,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => value.pass_id,
    }
}
