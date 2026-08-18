//! Temporary G3/G5 bridge from fully resolved render execution operations to backend-neutral
//! RunenGPU work.
//!
//! G5A deliberately requires this adapter to receive execution-complete `GpuWorkOperation`
//! values. It does not allocate logical GPU resources, reconstruct operation accesses, invent
//! pipeline/binding state, or project renderer declarations into a second GPU identity space.
//! The prepared graph remains the only access, initialization, hazard, capability, dependency,
//! and topological-order authority. The private sidecar contains only later-phase render
//! execution payload.
//!
//! A compiled render pass is not an execution identity: fixed-step regions may execute one pass
//! repeatedly and feature gates may omit an occurrence. RunenRender therefore supplies distinct
//! occurrence identities plus only render-owned control/non-data requirements. RunenGPU continues
//! to derive every resource dependency and hazard from the canonical operations.

use crate::plugins::gpu::*;
use crate::plugins::render::RenderPassId;
use crate::plugins::render::graph::{CompiledPassExecutionPlan, CompiledRenderFlowPlan};
use std::collections::{BTreeMap, BTreeSet};

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
    #[error("resolved render GPU work contains duplicate execution occurrence '{occurrence}'")]
    DuplicateOccurrence {
        occurrence: RenderGpuWorkOccurrenceId,
    },
    #[error(
        "resolved render control order references occurrence '{occurrence}' that is absent from this invocation's GPU work"
    )]
    MissingOrderedOccurrence {
        occurrence: RenderGpuWorkOccurrenceId,
    },
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

/// Process-local identity for one actual renderer GPU execution occurrence.
///
/// This is deliberately not `RenderPassId`: one compiled pass can occur more than once in an
/// invocation. It is sidecar/control-flow identity only and must not be persisted, serialized, or
/// treated as a resource/dependency identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RenderGpuWorkOccurrenceId(u64);

impl RenderGpuWorkOccurrenceId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for RenderGpuWorkOccurrenceId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(formatter)
    }
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

/// One execution-complete renderer occurrence ready for G3 graph preparation.
///
/// The operation is already the semantic authority for accesses and mechanical capability
/// requirements. `preference` is scheduling preference only; it cannot weaken operation facts.
/// `control_order_after` carries only render-owned non-data/control semantics. It must never be
/// populated from reconstructed resource dependencies.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedRenderGpuWorkNode {
    occurrence: RenderGpuWorkOccurrenceId,
    label: GpuResourceLabel,
    operation: GpuWorkOperation,
    preference: GpuExecutionPreference,
    provenance: GpuResourceProvenance,
    payload: RenderGpuWorkPayload,
    control_order_after: Vec<RenderGpuWorkOccurrenceId>,
}

impl ResolvedRenderGpuWorkNode {
    pub(crate) fn pass(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        pass: CompiledPassExecutionPlan,
        operation: GpuWorkOperation,
        preference: GpuExecutionPreference,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation,
            preference,
            provenance,
            payload: RenderGpuWorkPayload::Pass(Box::new(pass)),
            control_order_after: control_order_after.into_iter().collect(),
        }
    }

    pub(crate) fn timing_resolve(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        operation: GpuQueryResolveOperation,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation: GpuWorkOperation::Resolve(operation),
            preference: GpuExecutionPreference::TransferPreferred,
            provenance,
            payload: RenderGpuWorkPayload::TimingResolve,
            control_order_after: control_order_after.into_iter().collect(),
        }
    }

    pub(crate) fn timing_readback_copy(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        operation: GpuCopyOperation,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation: GpuWorkOperation::Copy(operation),
            preference: GpuExecutionPreference::TransferPreferred,
            provenance,
            payload: RenderGpuWorkPayload::TimingReadbackCopy,
            control_order_after: control_order_after.into_iter().collect(),
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

struct AuthoredRenderFragment {
    fragment: GpuWorkFragment,
    occurrence_nodes: BTreeMap<RenderGpuWorkOccurrenceId, GpuWorkNodeId>,
    pending: Vec<(GpuWorkNodeId, RenderGpuWorkPayload)>,
}

/// Prepares one renderer invocation from execution-complete logical GPU occurrences.
///
/// All kind-preserving resources are discovered from operation-derived accesses. Every non-query
/// resource is supplied to G3R with descriptor initialization coverage, so an uninitialized
/// descriptor remains uninitialized and a Prepared/Zeroed descriptor contributes only the
/// coverage already owned by RunenGPU. Caller-declared duplicate access truth is intentionally
/// absent.
///
/// G3 intentionally rejects explicit order already guaranteed by typed data dependencies. The
/// adapter therefore performs one provisional preparation without control edges, consumes G3's
/// own dependency result, and retains only unsatisfied render-control requirements for the final
/// preparation. No access intersection or hazard rule is duplicated in RunenRender.
pub(crate) fn prepare_render_gpu_work(
    plan: &CompiledRenderFlowPlan,
    nodes: impl IntoIterator<Item = ResolvedRenderGpuWorkNode>,
) -> Result<PreparedRenderWorkPlan, RenderGpuWorkAdapterError> {
    let nodes = nodes.into_iter().collect::<Vec<_>>();
    validate_occurrences(&nodes)?;

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
    let desired_control_orders = collect_desired_control_orders(&nodes);

    let provisional = author_render_fragment(
        &nodes,
        &resources,
        &inputs,
        &graph_label,
        &graph_provenance,
        &BTreeSet::new(),
    )?;
    let provisional_graph =
        GpuPreparedWorkGraph::prepare(graph_label.clone(), [provisional.fragment])?;
    let provisional_occurrences =
        map_prepared_occurrences(&provisional_graph, &provisional.occurrence_nodes)?;
    let required_explicit_orders = normalize_control_orders(
        &provisional_graph,
        &provisional_occurrences,
        &desired_control_orders,
    );

    let (graph, pending) = if required_explicit_orders.is_empty() {
        (provisional_graph, provisional.pending)
    } else {
        let final_fragment = author_render_fragment(
            &nodes,
            &resources,
            &inputs,
            &graph_label,
            &graph_provenance,
            &required_explicit_orders,
        )?;
        let graph = GpuPreparedWorkGraph::prepare(graph_label, [final_fragment.fragment])?;
        (graph, final_fragment.pending)
    };

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

fn validate_occurrences(nodes: &[ResolvedRenderGpuWorkNode]) -> Result<(), RenderGpuWorkAdapterError> {
    let occurrences = nodes
        .iter()
        .map(|node| node.occurrence)
        .collect::<BTreeSet<_>>();
    if occurrences.len() != nodes.len() {
        let mut seen = BTreeSet::new();
        let occurrence = nodes
            .iter()
            .map(|node| node.occurrence)
            .find(|occurrence| !seen.insert(*occurrence))
            .expect("duplicate occurrence count guarantees one repeated identity");
        return Err(RenderGpuWorkAdapterError::DuplicateOccurrence { occurrence });
    }
    for node in nodes {
        for occurrence in &node.control_order_after {
            if !occurrences.contains(occurrence) {
                return Err(RenderGpuWorkAdapterError::MissingOrderedOccurrence {
                    occurrence: *occurrence,
                });
            }
        }
    }
    Ok(())
}

fn collect_desired_control_orders(
    nodes: &[ResolvedRenderGpuWorkNode],
) -> Vec<(RenderGpuWorkOccurrenceId, RenderGpuWorkOccurrenceId)> {
    nodes
        .iter()
        .flat_map(|node| {
            node.control_order_after
                .iter()
                .copied()
                .map(move |before| (before, node.occurrence))
        })
        .collect()
}

fn author_render_fragment(
    nodes: &[ResolvedRenderGpuWorkNode],
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    inputs: &[GpuWorkResourceInput],
    graph_label: &GpuResourceLabel,
    graph_provenance: &GpuResourceProvenance,
    explicit_orders: &BTreeSet<(RenderGpuWorkOccurrenceId, RenderGpuWorkOccurrenceId)>,
) -> Result<AuthoredRenderFragment, RenderGpuWorkAdapterError> {
    let mut occurrence_nodes = BTreeMap::<RenderGpuWorkOccurrenceId, GpuWorkNodeId>::new();
    let mut pending = Vec::<(GpuWorkNodeId, RenderGpuWorkPayload)>::new();
    let fragment = GpuWorkFragment::build_with_provenance(
        graph_label.clone(),
        graph_provenance.clone(),
        |builder| {
            for resource in resources.values() {
                builder.declare_resource(resource.clone())?;
            }
            for input in inputs {
                builder.add_input(input.clone())?;
            }

            for node in nodes {
                let node_id = builder.add_node(
                    node.label.clone(),
                    node.operation.clone(),
                    [],
                    GpuCapabilityRequirements::new(),
                    node.preference,
                    node.provenance.clone(),
                )?;
                occurrence_nodes.insert(node.occurrence, node_id.clone());
                pending.push((node_id, node.payload.clone()));
            }

            for (before_occurrence, after_occurrence) in explicit_orders {
                let before = occurrence_nodes.get(before_occurrence).ok_or_else(|| {
                    GpuWorkAuthoringError::invalid(
                        "author resolved render occurrence order",
                        GpuWorkAuthoringErrorContext::new(
                            Some(graph_label.as_str().to_string()),
                            None,
                            None,
                            None,
                            Some(graph_provenance.clone()),
                        ),
                        GpuWorkAuthoringCause::UnknownIdentity,
                        "include every render control predecessor as an execution occurrence in this invocation",
                    )
                })?;
                let after = occurrence_nodes.get(after_occurrence).ok_or_else(|| {
                    GpuWorkAuthoringError::invalid(
                        "author resolved render occurrence order",
                        GpuWorkAuthoringErrorContext::new(
                            Some(graph_label.as_str().to_string()),
                            None,
                            None,
                            None,
                            Some(graph_provenance.clone()),
                        ),
                        GpuWorkAuthoringCause::UnknownIdentity,
                        "include every render control successor as an execution occurrence in this invocation",
                    )
                })?;
                builder.add_explicit_order(GpuExplicitOrder::new(
                    before,
                    after,
                    "render-owned occurrence control order",
                )?)?;
            }
            Ok(())
        },
    )?;

    Ok(AuthoredRenderFragment {
        fragment,
        occurrence_nodes,
        pending,
    })
}

fn map_prepared_occurrences(
    graph: &GpuPreparedWorkGraph,
    occurrence_nodes: &BTreeMap<RenderGpuWorkOccurrenceId, GpuWorkNodeId>,
) -> Result<BTreeMap<RenderGpuWorkOccurrenceId, GpuPreparedWorkNodeId>, RenderGpuWorkAdapterError> {
    let prepared_by_local = graph
        .nodes()
        .iter()
        .map(|node| (node.id().local_node(), node.id()))
        .collect::<BTreeMap<_, _>>();
    occurrence_nodes
        .iter()
        .map(|(occurrence, node_id)| {
            let local = node_id.diagnostic_local();
            let prepared = prepared_by_local
                .get(&local)
                .copied()
                .ok_or(RenderGpuWorkAdapterError::MissingPreparedNodeMapping {
                    local_node: local,
                })?;
            Ok((*occurrence, prepared))
        })
        .collect()
}

fn normalize_control_orders(
    provisional_graph: &GpuPreparedWorkGraph,
    occurrence_nodes: &BTreeMap<RenderGpuWorkOccurrenceId, GpuPreparedWorkNodeId>,
    desired: &[(RenderGpuWorkOccurrenceId, RenderGpuWorkOccurrenceId)],
) -> BTreeSet<(RenderGpuWorkOccurrenceId, RenderGpuWorkOccurrenceId)> {
    // This graph comes only from a G3 preparation with zero explicit orders. These edges are
    // therefore G3-derived data dependencies, not renderer-reconstructed access semantics.
    let mut satisfied_edges = provisional_graph
        .dependencies()
        .iter()
        .map(|dependency| (dependency.before(), dependency.after()))
        .collect::<BTreeSet<_>>();
    let mut retained = BTreeSet::new();

    for &(before_occurrence, after_occurrence) in desired {
        let before = occurrence_nodes[&before_occurrence];
        let after = occurrence_nodes[&after_occurrence];
        if dependency_path_exists(&satisfied_edges, before, after) {
            continue;
        }
        retained.insert((before_occurrence, after_occurrence));
        satisfied_edges.insert((before, after));
    }

    retained
}

fn dependency_path_exists(
    edges: &BTreeSet<(GpuPreparedWorkNodeId, GpuPreparedWorkNodeId)>,
    start: GpuPreparedWorkNodeId,
    target: GpuPreparedWorkNodeId,
) -> bool {
    let mut ready = vec![start];
    let mut visited = BTreeSet::new();
    while let Some(node) = ready.pop() {
        if !visited.insert(node) {
            continue;
        }
        for &(before, after) in edges {
            if before != node {
                continue;
            }
            if after == target {
                return true;
            }
            ready.push(after);
        }
    }
    false
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
