//! Temporary G3 bridge from render-owned authoring and execution payloads to
//! backend-neutral RunenGPU work.
//!
//! The prepared graph is the only access, initialization, hazard, capability,
//! dependency, and topological-order authority. The private sidecar contains
//! only later-phase render execution payload and is deleted as G4/G5 admit
//! programs and own ordinary submission.

use crate::plugins::gpu::*;
use crate::plugins::render::graph::{
    CompiledPassExecutionPlan, CompiledRenderFlowPlan, RenderDrawSource, RenderPassKind,
    RenderPassNode,
};
use crate::plugins::render::{
    RenderDepthPolicy, RenderImportedTextureSemantic, RenderPassId, RenderResourceDeclaration,
    RenderTargetAliasKind,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, thiserror::Error)]
pub enum RenderGpuWorkAdapterError {
    #[error("render GPU-work lowering is missing projected dispatch for pass '{pass_id}'")]
    MissingProjectedDispatch { pass_id: RenderPassId },
    #[error("render GPU-work lowering cannot resolve resource '{resource_id}'")]
    MissingResource { resource_id: GpuWorkResourceId },
    #[error("render GPU-work lowering expected buffer resource '{resource_id}'")]
    ExpectedBuffer { resource_id: GpuWorkResourceId },
    #[error("render GPU-work lowering expected texture resource '{resource_id}'")]
    ExpectedTexture { resource_id: GpuWorkResourceId },
    #[error("render GPU-work lowering cannot allocate timing identity: {0}")]
    Identity(#[from] GpuWorkResourceIdAllocationError),
    #[error(transparent)]
    ResourceAdapter(#[from] super::RenderGpuResourceAdapterError),
    #[error(transparent)]
    Descriptor(#[from] GpuResourceDescriptorError),
    #[error(transparent)]
    Access(#[from] GpuAccessError),
    #[error(transparent)]
    Operation(#[from] GpuWorkOperationError),
    #[error(transparent)]
    Authoring(#[from] GpuWorkAuthoringError),
    #[error(transparent)]
    Graph(#[from] GpuWorkGraphError),
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
    #[error("render GPU-work lowering has no passes to instrument")]
    EmptyTimingWork,
    #[error("render GPU-work lowering is missing required field '{field}' for pass '{pass_id}'")]
    MissingPassField {
        pass_id: RenderPassId,
        field: &'static str,
    },
}

/// Whether the temporary adapter should include the current timestamp-query
/// instrumentation chain. Capability discovery remains render/backend owned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderGpuWorkInstrumentation {
    #[default]
    Disabled,
    TimestampQueries,
}

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
            Self::TimingReadbackCopy => GpuWorkNodeKind::Copy,
            Self::TimingResolve => GpuWorkNodeKind::Resolve,
        }
    }
}

/// Private execution-only payload keyed solely by prepared G3 node identity.
/// It deliberately stores no access, hazard, initialization, capability,
/// dependency, export, diagnostic, or ordering data.
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

/// Per-invocation G3 work plus its temporary render execution bridge.
///
/// Prepared node IDs are process-local references only. This value must never
/// be persisted or used as a stable cache, replay, wire, or cross-process key.
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

    /// Returns render pass identities in the authoritative prepared G3 order.
    /// Timing resolve/copy nodes intentionally have no render-pass identity.
    pub fn ordered_render_pass_ids(&self) -> Result<Vec<RenderPassId>, RenderGpuWorkAdapterError> {
        self.ordered_payloads().map(|entries| {
            entries
                .into_iter()
                .filter_map(|(_, payload)| match payload {
                    RenderGpuWorkPayload::Pass(pass) => Some(execution_pass_id(pass.as_ref())),
                    RenderGpuWorkPayload::TimingResolve
                    | RenderGpuWorkPayload::TimingReadbackCopy => None,
                })
                .collect()
        })
    }
}

#[derive(Debug, Clone)]
struct PendingPayload {
    node_id: GpuWorkNodeId,
    payload: RenderGpuWorkPayload,
}

struct LoweredPendingPass {
    pass_id: RenderPassId,
    label: GpuResourceLabel,
    operation: GpuWorkOperation,
    accesses: Vec<GpuResourceAccess>,
    preference: GpuExecutionPreference,
    provenance: GpuResourceProvenance,
    payload: RenderGpuWorkPayload,
}

struct TimingResources {
    query_set: GpuQuerySetHandle,
    resolve_buffer: GpuBufferHandle,
    readback_buffer: GpuBufferHandle,
    query_count: u32,
}

struct LoweredTimingWork {
    resolve_label: GpuResourceLabel,
    resolve_operation: GpuQueryResolveOperation,
    readback_label: GpuResourceLabel,
    readback_operation: GpuCopyOperation,
}

struct LoweredNonDataOrder {
    before: RenderPassId,
    after: RenderPassId,
    after_label: String,
}

struct LoweredRenderWork {
    graph_label: GpuResourceLabel,
    provenance: GpuResourceProvenance,
    resources: BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    graph_inputs: Vec<GpuWorkResourceInput>,
    timing_resources: Option<TimingResources>,
    passes: Vec<LoweredPendingPass>,
    non_data_orders: Vec<LoweredNonDataOrder>,
    timing_work: Option<LoweredTimingWork>,
}

/// Transactionally lowers one compiled render plan and externally projected
/// dispatch values into immutable G3 work plus its execution-only sidecar.
pub fn prepare_render_gpu_work(
    plan: &CompiledRenderFlowPlan,
    projected_dispatches: &BTreeMap<RenderPassId, [u32; 3]>,
    surface_size: (u32, u32),
    instrumentation: RenderGpuWorkInstrumentation,
) -> Result<PreparedRenderWorkPlan, RenderGpuWorkAdapterError> {
    let lowered = lower_render_work(plan, projected_dispatches, surface_size, instrumentation)?;
    let (fragment, pending) = author_render_work_fragment(&lowered)?;
    let graph = GpuPreparedWorkGraph::prepare(lowered.graph_label.clone(), [fragment])?;
    let sidecar = prepare_sidecar(&graph, pending)?;
    Ok(PreparedRenderWorkPlan { graph, sidecar })
}

fn lower_render_work(
    plan: &CompiledRenderFlowPlan,
    projected_dispatches: &BTreeMap<RenderPassId, [u32; 3]>,
    surface_size: (u32, u32),
    instrumentation: RenderGpuWorkInstrumentation,
) -> Result<LoweredRenderWork, RenderGpuWorkAdapterError> {
    let graph_label = gpu_label(format!("render.flow.{}.work", plan.flow_id))?;
    let provenance = gpu_provenance(graph_label.clone());
    let resources = lower_resources(plan, surface_size)?;
    let timed_pass_ids = if instrumentation == RenderGpuWorkInstrumentation::TimestampQueries {
        plan.execution
            .passes
            .iter()
            .filter(|pass| pass_supports_timestamp_write(pass))
            .map(execution_pass_id)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let timing_resources = if timed_pass_ids.is_empty() {
        None
    } else {
        Some(lower_timing_resources(plan, timed_pass_ids.len())?)
    };

    let mut timestamp_index = BTreeMap::<RenderPassId, u32>::new();
    for (index, pass_id) in timed_pass_ids.iter().copied().enumerate() {
        let first = u32::try_from(index)
            .ok()
            .and_then(|value| value.checked_mul(2))
            .ok_or(RenderGpuWorkAdapterError::EmptyTimingWork)?;
        timestamp_index.insert(pass_id, first);
    }

    let graph_inputs = resources
        .iter()
        .filter(|(semantic_id, _)| resource_is_graph_entry(**semantic_id, plan))
        .map(|(_, resource)| {
            Ok(GpuWorkResourceInput::new(
                resource.clone(),
                graph_entry_coverage(resource)?,
                resource.common().provenance().clone(),
            )?)
        })
        .collect::<Result<Vec<_>, RenderGpuWorkAdapterError>>()?;
    let lowered_passes = plan
        .execution
        .passes
        .iter()
        .map(|payload| {
            let pass_id = execution_pass_id(payload);
            let node = plan
                .render_passes
                .iter()
                .find(|pass| pass.pass_id() == pass_id)
                .map(|pass| pass.node())
                .ok_or_else(|| authoring_missing_resource(plan, pass_id))?;
            let lowered = lower_pass_node(
                node,
                projected_dispatches,
                &resources,
                timing_resources.as_ref(),
                timestamp_index.get(&pass_id).copied(),
            )?;
            let label = gpu_label(node.label.clone())?;
            let provenance = gpu_provenance(label.clone());
            Ok(LoweredPendingPass {
                pass_id,
                label,
                operation: lowered.operation,
                accesses: lowered.accesses,
                preference: lowered.preference,
                provenance,
                payload: RenderGpuWorkPayload::Pass(Box::new(payload.clone())),
            })
        })
        .collect::<Result<Vec<_>, RenderGpuWorkAdapterError>>()?;
    let non_data_orders = plan
        .render_passes
        .iter()
        .flat_map(|pass| {
            pass.node()
                .non_data_order_after
                .iter()
                .copied()
                .map(|before| LoweredNonDataOrder {
                    before,
                    after: pass.pass_id(),
                    after_label: pass.pass_label().to_string(),
                })
        })
        .collect();
    let timing_work = timing_resources
        .as_ref()
        .map(|timing| lower_timing_work(plan, timing))
        .transpose()?;

    Ok(LoweredRenderWork {
        graph_label,
        provenance,
        resources,
        graph_inputs,
        timing_resources,
        passes: lowered_passes,
        non_data_orders,
        timing_work,
    })
}

fn author_render_work_fragment(
    lowered: &LoweredRenderWork,
) -> Result<(GpuWorkFragment, Vec<PendingPayload>), RenderGpuWorkAdapterError> {
    let mut pending = Vec::<PendingPayload>::new();
    let fragment = GpuWorkFragment::build_with_provenance(
        lowered.graph_label.clone(),
        lowered.provenance.clone(),
        |builder| {
            declare_fragment_resources(builder, lowered)?;
            let node_ids = author_pass_nodes(builder, &lowered.passes, &mut pending)?;
            author_non_data_orders(builder, lowered, &node_ids)?;
            author_timing_nodes(builder, lowered.timing_work.as_ref(), &mut pending)?;
            Ok(())
        },
    )?;
    Ok((fragment, pending))
}

fn declare_fragment_resources(
    builder: &mut GpuWorkFragmentBuilder,
    lowered: &LoweredRenderWork,
) -> Result<(), GpuWorkAuthoringError> {
    for resource in lowered.resources.values() {
        builder.declare_resource(resource.clone())?;
    }
    for input in &lowered.graph_inputs {
        builder.add_input(input.clone())?;
    }
    if let Some(timing) = &lowered.timing_resources {
        for resource in [
            GpuResourceRef::QuerySet(timing.query_set.clone()),
            GpuResourceRef::Buffer(timing.resolve_buffer.clone()),
            GpuResourceRef::Buffer(timing.readback_buffer.clone()),
        ] {
            builder.declare_resource(resource)?;
        }
    }
    Ok(())
}

fn author_pass_nodes(
    builder: &mut GpuWorkFragmentBuilder,
    passes: &[LoweredPendingPass],
    pending: &mut Vec<PendingPayload>,
) -> Result<BTreeMap<RenderPassId, GpuWorkNodeId>, GpuWorkAuthoringError> {
    let mut node_ids = BTreeMap::new();
    for pass in passes {
        let id = builder.add_node(
            pass.label.clone(),
            pass.operation.clone(),
            pass.accesses.clone(),
            GpuCapabilityRequirements::new(),
            pass.preference,
            pass.provenance.clone(),
        )?;
        node_ids.insert(pass.pass_id, id.clone());
        pending.push(PendingPayload {
            node_id: id,
            payload: pass.payload.clone(),
        });
    }
    Ok(node_ids)
}

fn author_non_data_orders(
    builder: &mut GpuWorkFragmentBuilder,
    lowered: &LoweredRenderWork,
    node_ids: &BTreeMap<RenderPassId, GpuWorkNodeId>,
) -> Result<(), GpuWorkAuthoringError> {
    for order in &lowered.non_data_orders {
        let missing = || {
            GpuWorkAuthoringError::invalid(
                "lower render non-data order",
                GpuWorkAuthoringErrorContext::new(
                    Some(lowered.graph_label.as_str().to_string()),
                    Some(order.after_label.clone()),
                    None,
                    None,
                    Some(lowered.provenance.clone()),
                ),
                GpuWorkAuthoringCause::UnknownIdentity,
                "reference a pass in the same render work fragment",
            )
        };
        let before = node_ids.get(&order.before).ok_or_else(&missing)?;
        let after = node_ids.get(&order.after).ok_or_else(missing)?;
        builder.add_explicit_order(GpuExplicitOrder::new(
            before,
            after,
            "render-owned non-data order",
        )?)?;
    }
    Ok(())
}

fn author_timing_nodes(
    builder: &mut GpuWorkFragmentBuilder,
    timing: Option<&LoweredTimingWork>,
    pending: &mut Vec<PendingPayload>,
) -> Result<(), GpuWorkAuthoringError> {
    let Some(timing) = timing else {
        return Ok(());
    };
    let resolve_id = builder.add_node(
        timing.resolve_label.clone(),
        GpuWorkOperation::Resolve(timing.resolve_operation.clone()),
        [],
        GpuCapabilityRequirements::new(),
        GpuExecutionPreference::TransferPreferred,
        gpu_provenance(timing.resolve_label.clone()),
    )?;
    pending.push(PendingPayload {
        node_id: resolve_id,
        payload: RenderGpuWorkPayload::TimingResolve,
    });

    let copy_id = builder.add_node(
        timing.readback_label.clone(),
        GpuWorkOperation::Copy(timing.readback_operation.clone()),
        [],
        GpuCapabilityRequirements::new(),
        GpuExecutionPreference::TransferPreferred,
        gpu_provenance(timing.readback_label.clone()),
    )?;
    pending.push(PendingPayload {
        node_id: copy_id,
        payload: RenderGpuWorkPayload::TimingReadbackCopy,
    });
    Ok(())
}

fn prepare_sidecar(
    graph: &GpuPreparedWorkGraph,
    pending: Vec<PendingPayload>,
) -> Result<RenderGpuWorkSidecar, RenderGpuWorkAdapterError> {
    let prepared_by_local = graph
        .nodes()
        .iter()
        .map(|node| (node.id().local_node(), node.id()))
        .collect::<BTreeMap<_, _>>();
    let mut sidecar = RenderGpuWorkSidecar::default();
    for pending_payload in pending {
        let local = pending_payload.node_id.diagnostic_local();
        let prepared_id = prepared_by_local
            .get(&local)
            .copied()
            .ok_or(RenderGpuWorkAdapterError::MissingPreparedNodeMapping { local_node: local })?;
        sidecar.insert(graph, prepared_id, pending_payload.payload)?;
    }
    sidecar.finish(graph)
}

struct LoweredPass {
    operation: GpuWorkOperation,
    accesses: Vec<GpuResourceAccess>,
    preference: GpuExecutionPreference,
}

fn lower_pass_node(
    node: &RenderPassNode,
    projected_dispatches: &BTreeMap<RenderPassId, [u32; 3]>,
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    timing: Option<&TimingResources>,
    first_query: Option<u32>,
) -> Result<LoweredPass, RenderGpuWorkAdapterError> {
    let accesses = lower_caller_accesses(node, resources)?;
    let timestamp_access = match (timing, first_query) {
        (Some(timing), Some(first)) => {
            let range = GpuQueryRange::new(&timing.query_set, first, 2)?;
            Some(GpuQueryAccess::new(
                &timing.query_set,
                range,
                GpuQueryAccessKind::WriteTimestamp,
            )?)
        }
        _ => None,
    };
    let operation = match node.kind {
        RenderPassKind::Compute => {
            let dispatch = projected_dispatches
                .get(&node.id)
                .copied()
                .ok_or(RenderGpuWorkAdapterError::MissingProjectedDispatch { pass_id: node.id })?;
            let operation = GpuComputeOperation::new(GpuDispatchSize::new(
                dispatch[0],
                dispatch[1],
                dispatch[2],
            )?);
            GpuWorkOperation::Compute(match timestamp_access {
                Some(timestamp) => operation.with_timestamp_writes([timestamp])?,
                None => operation,
            })
        }
        RenderPassKind::Fullscreen
        | RenderPassKind::Graphics
        | RenderPassKind::BuiltinUiComposite => {
            let color_attachments = node
                .color_outputs
                .iter()
                .map(|resource_id| lower_color_attachment(node, *resource_id, resources))
                .collect::<Result<Vec<_>, _>>()?;
            let depth = node
                .depth_target
                .map(|resource_id| lower_depth_attachment(node, resource_id, resources))
                .transpose()?;
            let draws = lower_draws(node, resources)?;
            GpuWorkOperation::Render(GpuRenderOperation::new(
                color_attachments,
                depth,
                draws,
                timestamp_access,
            )?)
        }
        RenderPassKind::Copy => GpuWorkOperation::Copy(lower_copy(node, resources)?),
        RenderPassKind::Present => {
            let source_id =
                node.present_source
                    .ok_or(RenderGpuWorkAdapterError::MissingPassField {
                        pass_id: node.id,
                        field: "present_source",
                    })?;
            let source = texture_resource(resources, source_id)?;
            let range = GpuTextureSubresourceRange::whole(source.parent_texture())?;
            GpuWorkOperation::Present(GpuPresentOperation::new(source, range)?)
        }
    };
    let preference = match node.kind {
        RenderPassKind::Compute => GpuExecutionPreference::ComputePreferred,
        RenderPassKind::Fullscreen
        | RenderPassKind::Graphics
        | RenderPassKind::BuiltinUiComposite => GpuExecutionPreference::GraphicsRequired,
        RenderPassKind::Copy => GpuExecutionPreference::TransferPreferred,
        RenderPassKind::Present => GpuExecutionPreference::GraphicsRequired,
    };
    Ok(LoweredPass {
        operation,
        accesses,
        preference,
    })
}

fn lower_caller_accesses(
    node: &RenderPassNode,
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
) -> Result<Vec<GpuResourceAccess>, RenderGpuWorkAdapterError> {
    let mut accesses = Vec::new();
    for binding in &node.uniform_bindings {
        accesses.push(buffer_access(
            resources,
            *binding.uniform_id(),
            GpuBufferAccessKind::UniformRead,
        )?);
    }
    for resource in &node.fixed_step_iteration_uniforms {
        accesses.push(buffer_access(
            resources,
            *resource,
            GpuBufferAccessKind::UniformRead,
        )?);
    }
    let writable = node.storage_writes.iter().copied().collect::<BTreeSet<_>>();
    for resource in node
        .storage_reads
        .iter()
        .chain(&node.storage_writes)
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let reads = node.storage_reads.contains(&resource);
        let writes = writable.contains(&resource);
        let kind = match (reads, writes) {
            (true, true) => GpuBufferAccessKind::StorageReadWrite,
            (true, false) => GpuBufferAccessKind::StorageRead,
            (false, true) => GpuBufferAccessKind::StorageWrite,
            (false, false) => continue,
        };
        accesses.push(buffer_access(resources, resource, kind)?);
    }
    for resource in &node.sampled_textures {
        accesses.push(texture_access(
            resources,
            *resource,
            GpuTextureAccessKind::SampledRead,
        )?);
    }
    for resource in &node.write_textures {
        accesses.push(texture_access(
            resources,
            *resource,
            GpuTextureAccessKind::StorageWrite,
        )?);
    }
    for resource in &node.vertex_buffers {
        accesses.push(buffer_access(
            resources,
            *resource,
            GpuBufferAccessKind::VertexRead,
        )?);
    }
    for resource in &node.instance_buffers {
        accesses.push(buffer_access(
            resources,
            *resource,
            GpuBufferAccessKind::VertexRead,
        )?);
    }
    for resource in &node.index_buffers {
        accesses.push(buffer_access(
            resources,
            *resource,
            GpuBufferAccessKind::IndexRead,
        )?);
    }
    let exact_indirect = match node.draw.map(|draw| draw.source) {
        Some(RenderDrawSource::Indirect { args_buffer, .. }) => Some(args_buffer),
        _ => None,
    };
    for resource in &node.indirect_buffers {
        if Some(*resource) != exact_indirect {
            accesses.push(buffer_access(
                resources,
                *resource,
                GpuBufferAccessKind::IndirectRead,
            )?);
        }
    }
    Ok(accesses)
}

fn lower_color_attachment(
    node: &RenderPassNode,
    resource_id: GpuWorkResourceId,
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
) -> Result<GpuRenderColorAttachment, RenderGpuWorkAdapterError> {
    let resource = texture_resource(resources, resource_id)?;
    let range = GpuTextureSubresourceRange::whole(resource.parent_texture())?;
    let load = match node.clear_color {
        Some(color) => {
            GpuColorAttachmentLoad::Clear(GpuColorClearValue::from_array(color.map(f64::from))?)
        }
        None => GpuColorAttachmentLoad::Load,
    };
    Ok(GpuRenderColorAttachment::new(
        resource,
        range,
        load,
        GpuAttachmentStore::Store,
        None,
    )?)
}

fn lower_depth_attachment(
    node: &RenderPassNode,
    resource_id: GpuWorkResourceId,
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
) -> Result<GpuRenderDepthStencilAttachment, RenderGpuWorkAdapterError> {
    let resource = texture_resource(resources, resource_id)?;
    let range = GpuTextureSubresourceRange::whole(resource.parent_texture())?;
    let access = if node.raster_state.depth_policy == RenderDepthPolicy::ReadOnly {
        GpuDepthStencilAccess::ReadOnly
    } else {
        GpuDepthStencilAccess::ReadWrite
    };
    let load = if access == GpuDepthStencilAccess::ReadOnly {
        GpuDepthAttachmentLoad::Load
    } else {
        GpuDepthAttachmentLoad::Clear(GpuDepthClearValue::new(1.0)?)
    };
    Ok(GpuRenderDepthStencilAttachment::new(
        resource,
        range,
        access,
        load,
        GpuAttachmentStore::Store,
    )?)
}

fn lower_draws(
    node: &RenderPassNode,
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
) -> Result<Vec<GpuDrawIntent>, RenderGpuWorkAdapterError> {
    match node.kind {
        RenderPassKind::Fullscreen => Ok(vec![GpuDrawIntent::direct(
            GpuDrawRange::new(0, 3)?,
            GpuDrawRange::new(0, 1)?,
        )]),
        RenderPassKind::BuiltinUiComposite => Ok(vec![GpuDrawIntent::direct(
            GpuDrawRange::new(0, 6)?,
            GpuDrawRange::new(0, 1)?,
        )]),
        RenderPassKind::Graphics => {
            let draw = node
                .draw
                .ok_or(RenderGpuWorkAdapterError::MissingPassField {
                    pass_id: node.id,
                    field: "draw",
                })?;
            let instances = GpuDrawRange::new(draw.first_instance, draw.instance_count)?;
            let intent = match draw.source {
                RenderDrawSource::Direct if node.index_buffers.is_empty() => GpuDrawIntent::direct(
                    GpuDrawRange::new(draw.first_vertex, draw.vertex_count)?,
                    instances,
                ),
                RenderDrawSource::Direct => GpuDrawIntent::indexed(
                    GpuDrawRange::new(draw.first_vertex, draw.vertex_count)?,
                    0,
                    instances,
                ),
                RenderDrawSource::Indirect {
                    args_buffer,
                    args_kind,
                    byte_offset,
                    ..
                } => {
                    let buffer = buffer_resource(resources, args_buffer)?;
                    let size = match args_kind {
                        crate::plugins::render::RenderIndirectDrawArgsKind::Draw => 16,
                        crate::plugins::render::RenderIndirectDrawArgsKind::DrawIndexed => 20,
                    };
                    let range = GpuBufferRange::new(&buffer, byte_offset, size)?;
                    GpuDrawIntent::indirect(
                        &buffer,
                        range,
                        matches!(
                            args_kind,
                            crate::plugins::render::RenderIndirectDrawArgsKind::DrawIndexed
                        ),
                    )?
                }
            };
            Ok(vec![intent])
        }
        _ => Ok(Vec::new()),
    }
}

fn lower_copy(
    node: &RenderPassNode,
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
) -> Result<GpuCopyOperation, RenderGpuWorkAdapterError> {
    let source_id = node
        .copy_source
        .ok_or(RenderGpuWorkAdapterError::MissingPassField {
            pass_id: node.id,
            field: "copy_source",
        })?;
    let destination_id =
        node.copy_destination
            .ok_or(RenderGpuWorkAdapterError::MissingPassField {
                pass_id: node.id,
                field: "copy_destination",
            })?;
    match (resources.get(&source_id), resources.get(&destination_id)) {
        (Some(GpuResourceRef::Buffer(source)), Some(GpuResourceRef::Buffer(destination))) => {
            let size = source
                .descriptor()
                .size_bytes()
                .min(destination.descriptor().size_bytes());
            let source_range = GpuBufferRange::new(source, 0, size)?;
            let destination_range = GpuBufferRange::new(destination, 0, size)?;
            Ok(GpuCopyOperation::buffer_to_buffer(
                GpuBufferRegion::new(source, source_range)?,
                GpuBufferRegion::new(destination, destination_range)?,
            )?)
        }
        (Some(GpuResourceRef::Texture(source)), Some(GpuResourceRef::Texture(destination))) => {
            let source_extent = source.descriptor().extent();
            let destination_extent = destination.descriptor().extent();
            let extent = GpuCopyExtent::new(
                source_extent.width().min(destination_extent.width()),
                source_extent.height().min(destination_extent.height()),
                source_extent
                    .depth_or_layers()
                    .min(destination_extent.depth_or_layers()),
            )?;
            let origin = GpuTextureOrigin::new(0, 0, 0);
            let aspect = if source.descriptor().format().is_depth() {
                GpuTextureAspect::DepthOnly
            } else {
                GpuTextureAspect::Color
            };
            Ok(GpuCopyOperation::texture_to_texture(
                GpuTextureCopyRegion::new(source, 0, origin, aspect, extent)?,
                GpuTextureCopyRegion::new(destination, 0, origin, aspect, extent)?,
            )?)
        }
        (
            Some(GpuResourceRef::TextureView(source)),
            Some(GpuResourceRef::TextureView(destination)),
        ) => {
            let mut cloned = resources.clone();
            cloned.insert(
                source_id,
                GpuResourceRef::Texture(source.descriptor().texture().clone()),
            );
            cloned.insert(
                destination_id,
                GpuResourceRef::Texture(destination.descriptor().texture().clone()),
            );
            lower_copy(node, &cloned)
        }
        _ => Err(RenderGpuWorkAdapterError::MissingResource {
            resource_id: source_id,
        }),
    }
}

fn lower_resources(
    plan: &CompiledRenderFlowPlan,
    surface_size: (u32, u32),
) -> Result<BTreeMap<GpuWorkResourceId, GpuResourceRef>, RenderGpuWorkAdapterError> {
    let mut resources = BTreeMap::new();
    let mut allocator = GpuWorkResourceIdAllocator::new();
    for declaration in &plan.resources.resources {
        let id = *declaration.id();
        let resource = match declaration {
            RenderResourceDeclaration::Uniform(value) => {
                GpuResourceRef::Buffer(value.handle().clone())
            }
            RenderResourceDeclaration::Storage(value) => {
                GpuResourceRef::Buffer(value.handle().clone())
            }
            RenderResourceDeclaration::Sampled(_)
            | RenderResourceDeclaration::StorageImage(_)
            | RenderResourceDeclaration::ColorAttachment(_)
            | RenderResourceDeclaration::DepthAttachment(_)
            | RenderResourceDeclaration::History(_) => {
                match declaration
                    .lower_gpu_resource(surface_size, super::legacy_surface_validation_format())?
                {
                    super::RenderGpuResourceLowering::Normalized(descriptor) => match *descriptor {
                        GpuResourceDescriptor::Texture(descriptor) => {
                            GpuResourceRef::Texture(allocator.allocate_texture_handle(descriptor)?)
                        }
                        GpuResourceDescriptor::Buffer(descriptor) => {
                            GpuResourceRef::Buffer(allocator.allocate_buffer_handle(descriptor)?)
                        }
                        _ => {
                            return Err(RenderGpuWorkAdapterError::MissingResource {
                                resource_id: id,
                            });
                        }
                    },
                    _ => {
                        return Err(RenderGpuWorkAdapterError::MissingResource { resource_id: id });
                    }
                }
            }
            RenderResourceDeclaration::ImportedTexture(value) => {
                let depth = value.semantic == RenderImportedTextureSemantic::SurfaceDepth;
                GpuResourceRef::Texture(prepared_texture_handle(
                    &mut allocator,
                    value.label.as_str(),
                    surface_size,
                    depth,
                    true,
                )?)
            }
            RenderResourceDeclaration::TargetAlias(value) => {
                let depth = value.kind() == RenderTargetAliasKind::Depth;
                GpuResourceRef::Texture(prepared_texture_handle(
                    &mut allocator,
                    value.binding_key().as_str(),
                    surface_size,
                    depth,
                    true,
                )?)
            }
            RenderResourceDeclaration::ImportedBuffer(value) => GpuResourceRef::Buffer(
                prepared_buffer_handle(&mut allocator, value.label.as_str(), 4, true)?,
            ),
        };
        resources.insert(id, resource);
    }
    Ok(resources)
}

fn prepared_texture_handle(
    allocator: &mut GpuWorkResourceIdAllocator,
    label: &str,
    surface_size: (u32, u32),
    depth: bool,
    imported: bool,
) -> Result<GpuTextureHandle, RenderGpuWorkAdapterError> {
    let label = gpu_label(label.to_string())?;
    let provenance = gpu_provenance(label.clone());
    let common = if imported {
        GpuResourceCommon::imported(label.clone(), GpuResourceLifetime::Retained, provenance)
    } else {
        GpuResourceCommon::owned(
            label.clone(),
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
            GpuReconstruction::SourceBacked,
            provenance,
        )?
    };
    let format = if depth {
        GpuTextureFormat::Depth32Float
    } else {
        GpuTextureFormat::Rgba8Unorm
    };
    let usages = if depth {
        vec![
            GpuTextureUsage::Sampled,
            GpuTextureUsage::DepthStencilAttachment,
            GpuTextureUsage::CopySource,
            GpuTextureUsage::CopyDestination,
        ]
    } else {
        vec![
            GpuTextureUsage::Sampled,
            GpuTextureUsage::StorageRead,
            GpuTextureUsage::StorageWrite,
            GpuTextureUsage::ColorAttachment,
            GpuTextureUsage::CopySource,
            GpuTextureUsage::CopyDestination,
        ]
    };
    let extent = GpuTextureExtent::new(
        common.label(),
        GpuTextureDimension::D2,
        surface_size.0.max(1),
        surface_size.1.max(1),
        1,
    )?;
    let descriptor = GpuTextureDescriptor::new(
        common,
        GpuTextureDimension::D2,
        extent,
        1,
        1,
        format,
        GpuTextureUsages::new(&label, usages)?,
        GpuTextureInitialization::Uninitialized,
    )?;
    Ok(allocator.allocate_texture_handle(descriptor)?)
}

fn prepared_buffer_handle(
    allocator: &mut GpuWorkResourceIdAllocator,
    label: &str,
    size: u64,
    imported: bool,
) -> Result<GpuBufferHandle, RenderGpuWorkAdapterError> {
    let label = gpu_label(label.to_string())?;
    let provenance = gpu_provenance(label.clone());
    let common = if imported {
        GpuResourceCommon::imported(label.clone(), GpuResourceLifetime::Retained, provenance)
    } else {
        GpuResourceCommon::owned(
            label.clone(),
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
            GpuReconstruction::SourceBacked,
            provenance,
        )?
    };
    let usages = GpuBufferUsages::new(
        &label,
        [
            GpuBufferUsage::Uniform,
            GpuBufferUsage::Storage,
            GpuBufferUsage::Vertex,
            GpuBufferUsage::Index,
            GpuBufferUsage::Indirect,
            GpuBufferUsage::CopySource,
            GpuBufferUsage::CopyDestination,
        ],
    )?;
    let descriptor = GpuBufferDescriptor::new(
        common,
        size.max(1),
        usages,
        GpuBufferInitialization::Uninitialized,
    )?;
    Ok(allocator.allocate_buffer_handle(descriptor)?)
}

fn lower_timing_resources(
    plan: &CompiledRenderFlowPlan,
    timed_pass_count: usize,
) -> Result<TimingResources, RenderGpuWorkAdapterError> {
    let query_count = u32::try_from(timed_pass_count)
        .ok()
        .and_then(|count| count.checked_mul(2))
        .ok_or(RenderGpuWorkAdapterError::EmptyTimingWork)?;
    if query_count == 0 {
        return Err(RenderGpuWorkAdapterError::EmptyTimingWork);
    }
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let query_label = gpu_label(format!("{}.timestamp_queries", plan.flow_label))?;
    let query_common = GpuResourceCommon::owned(
        query_label.clone(),
        GpuResourceLifetime::Transient,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        gpu_provenance(query_label.clone()),
    )?;
    let query_set = allocator.allocate_query_set_handle(GpuQuerySetDescriptor::new(
        query_common,
        GpuQueryKind::Timestamp,
        query_count,
    )?)?;
    let byte_len = u64::from(query_count) * 8;
    let resolve_label = gpu_label(format!("{}.timestamp_resolve", plan.flow_label))?;
    let resolve_common = GpuResourceCommon::owned(
        resolve_label.clone(),
        GpuResourceLifetime::Transient,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        gpu_provenance(resolve_label.clone()),
    )?;
    let resolve_buffer = allocator.allocate_buffer_handle(GpuBufferDescriptor::new(
        resolve_common,
        byte_len,
        GpuBufferUsages::new(
            &resolve_label,
            [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
        )?,
        GpuBufferInitialization::Uninitialized,
    )?)?;
    let readback_label = gpu_label(format!("{}.timestamp_readback", plan.flow_label))?;
    let readback_common = GpuResourceCommon::owned(
        readback_label.clone(),
        GpuResourceLifetime::Transient,
        GpuMemoryIntent::Readback,
        GpuReconstruction::SourceBacked,
        gpu_provenance(readback_label.clone()),
    )?;
    let readback_buffer = allocator.allocate_buffer_handle(GpuBufferDescriptor::new(
        readback_common,
        byte_len,
        GpuBufferUsages::new(&readback_label, [GpuBufferUsage::CopyDestination])?,
        GpuBufferInitialization::Uninitialized,
    )?)?;
    Ok(TimingResources {
        query_set,
        resolve_buffer,
        readback_buffer,
        query_count,
    })
}

fn lower_timing_work(
    plan: &CompiledRenderFlowPlan,
    timing: &TimingResources,
) -> Result<LoweredTimingWork, RenderGpuWorkAdapterError> {
    let query_range = GpuQueryRange::new(&timing.query_set, 0, timing.query_count)?;
    let resolve_operation =
        GpuQueryResolveOperation::new(&timing.query_set, query_range, &timing.resolve_buffer, 0)?;
    let resolve_label = gpu_label(format!("{}.timing.resolve", plan.flow_label))?;

    let byte_len = u64::from(timing.query_count) * 8;
    let source_range = GpuBufferRange::new(&timing.resolve_buffer, 0, byte_len)?;
    let destination_range = GpuBufferRange::new(&timing.readback_buffer, 0, byte_len)?;
    let readback_operation = GpuCopyOperation::buffer_to_buffer(
        GpuBufferRegion::new(&timing.resolve_buffer, source_range)?,
        GpuBufferRegion::new(&timing.readback_buffer, destination_range)?,
    )?;
    let readback_label = gpu_label(format!("{}.timing.readback", plan.flow_label))?;

    Ok(LoweredTimingWork {
        resolve_label,
        resolve_operation,
        readback_label,
        readback_operation,
    })
}

fn resource_is_graph_entry(semantic_id: GpuWorkResourceId, plan: &CompiledRenderFlowPlan) -> bool {
    plan.resource_descriptor(semantic_id)
        .is_some_and(|declaration| {
            declaration.is_imported()
                || matches!(declaration, RenderResourceDeclaration::TargetAlias(_))
                || declaration.lifetime().is_retained()
        })
}

fn graph_entry_coverage(
    resource: &GpuResourceRef,
) -> Result<GpuInitialCoverage, RenderGpuWorkAdapterError> {
    match resource {
        GpuResourceRef::Buffer(buffer) => Ok(GpuInitialCoverage::buffer(
            buffer,
            [GpuBufferCoverage::dense(GpuBufferRange::whole(buffer)?)],
        )?),
        GpuResourceRef::Texture(texture) => {
            let access = GpuTextureAccessResource::Texture(texture.clone());
            Ok(GpuInitialCoverage::texture_subresources(
                &access,
                [GpuTextureSubresourceRange::whole(texture)?],
            )?)
        }
        GpuResourceRef::TextureView(view) => {
            let access = GpuTextureAccessResource::TextureView(view.clone());
            Ok(GpuInitialCoverage::texture_subresources(
                &access,
                [view.descriptor().subresources()],
            )?)
        }
        GpuResourceRef::Sampler(_) => Ok(GpuInitialCoverage::descriptor_initialization(
            resource.clone(),
        )?),
        GpuResourceRef::QuerySet(query_set) => Ok(GpuInitialCoverage::query_ranges(
            query_set,
            [GpuQueryRange::new(
                query_set,
                0,
                query_set.descriptor().count(),
            )?],
        )?),
    }
}

fn buffer_access(
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    resource_id: GpuWorkResourceId,
    kind: GpuBufferAccessKind,
) -> Result<GpuResourceAccess, RenderGpuWorkAdapterError> {
    let buffer = buffer_resource(resources, resource_id)?;
    let range = GpuBufferRange::whole(&buffer)?;
    Ok(GpuResourceAccess::Buffer(GpuBufferAccess::new(
        &buffer, range, kind,
    )?))
}

fn texture_access(
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    resource_id: GpuWorkResourceId,
    kind: GpuTextureAccessKind,
) -> Result<GpuResourceAccess, RenderGpuWorkAdapterError> {
    let resource = texture_resource(resources, resource_id)?;
    let range = GpuTextureSubresourceRange::whole(resource.parent_texture())?;
    Ok(GpuResourceAccess::Texture(GpuTextureAccess::new(
        resource, range, kind,
    )?))
}

fn buffer_resource(
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    resource_id: GpuWorkResourceId,
) -> Result<GpuBufferHandle, RenderGpuWorkAdapterError> {
    match resources.get(&resource_id) {
        Some(GpuResourceRef::Buffer(buffer)) => Ok(buffer.clone()),
        Some(_) => Err(RenderGpuWorkAdapterError::ExpectedBuffer { resource_id }),
        None => Err(RenderGpuWorkAdapterError::MissingResource { resource_id }),
    }
}

fn texture_resource(
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    resource_id: GpuWorkResourceId,
) -> Result<GpuTextureAccessResource, RenderGpuWorkAdapterError> {
    match resources.get(&resource_id) {
        Some(GpuResourceRef::Texture(texture)) => {
            Ok(GpuTextureAccessResource::Texture(texture.clone()))
        }
        Some(GpuResourceRef::TextureView(view)) => {
            Ok(GpuTextureAccessResource::TextureView(view.clone()))
        }
        Some(_) => Err(RenderGpuWorkAdapterError::ExpectedTexture { resource_id }),
        None => Err(RenderGpuWorkAdapterError::MissingResource { resource_id }),
    }
}

fn gpu_label(value: String) -> Result<GpuResourceLabel, GpuResourceDescriptorError> {
    GpuResourceLabel::new(value)
}

fn gpu_provenance(label: GpuResourceLabel) -> GpuResourceProvenance {
    GpuResourceProvenance::new(label, None, None)
}

fn execution_pass_id(pass: &CompiledPassExecutionPlan) -> RenderPassId {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.pass_id,
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => value.pass_id,
        CompiledPassExecutionPlan::Copy(value) => value.pass_id,
        CompiledPassExecutionPlan::Present(value) => value.pass_id,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => value.pass_id,
    }
}

fn pass_supports_timestamp_write(pass: &CompiledPassExecutionPlan) -> bool {
    matches!(
        pass,
        CompiledPassExecutionPlan::Compute(_)
            | CompiledPassExecutionPlan::Fullscreen(_)
            | CompiledPassExecutionPlan::Graphics(_)
            | CompiledPassExecutionPlan::BuiltinUiComposite(_)
    )
}

fn authoring_missing_resource(
    plan: &CompiledRenderFlowPlan,
    pass_id: RenderPassId,
) -> GpuWorkAuthoringError {
    GpuWorkAuthoringError::invalid(
        "map render execution payload",
        GpuWorkAuthoringErrorContext::new(
            Some(plan.flow_label.clone()),
            Some(pass_id.to_string()),
            None,
            None,
            None,
        ),
        GpuWorkAuthoringCause::UnknownIdentity,
        "compile one execution payload for every render pass",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{GpuStorage, RenderFlow, compile_flow_plan};

    #[derive(Debug, Clone, Copy, GpuStorage)]
    struct TestElement {
        value: u32,
    }

    #[derive(Debug, Clone, ecs::Resource)]
    struct TestState;

    impl TestState {
        fn dispatch(&self) -> [u32; 3] {
            [1, 1, 1]
        }
    }

    fn adapter_test_flow() -> RenderFlow {
        let (flow, storage) = RenderFlow::new("adapter.g3")
            .with_state::<TestState>()
            .with_color_target("adapter.color")
            .expect("color target should be valid")
            .with_depth_target("adapter.depth")
            .expect("depth target should be valid")
            .storage_array::<TestElement>("adapter.storage", 64)
            .expect("storage should be valid");
        let storage_binding =
            GpuBindingKey::try_new(0, 0).expect("adapter test storage binding should be valid");
        flow.compute_pass("adapter.compute")
            .bind_storage(storage_binding, storage.clone())
            .dispatch_from_state(TestState::dispatch)
            .finish()
            .graphics_pass("adapter.render")
            .bind_storage(storage_binding, storage)
            .write_color_target("adapter.color")
            .depth_target("adapter.depth")
            .clear_color([0.125, 0.25, 0.5, 1.0])
            .draw(3, 1)
            .finish()
            .validate()
            .expect("adapter test flow should validate")
    }

    fn adapter_test_plan() -> CompiledRenderFlowPlan {
        compile_flow_plan(&adapter_test_flow()).expect("adapter test flow should compile")
    }

    fn pass_id(plan: &CompiledRenderFlowPlan, label: &str) -> RenderPassId {
        plan.render_passes
            .iter()
            .find(|pass| pass.pass_label() == label)
            .map(|pass| pass.pass_id())
            .expect("test pass should exist")
    }

    fn prepared_node<'a>(work: &'a PreparedRenderWorkPlan, label: &str) -> &'a GpuPreparedWorkNode {
        work.graph()
            .nodes()
            .iter()
            .find(|prepared| prepared.node().label().as_str() == label)
            .expect("prepared test node should exist")
    }

    fn timing_resource_identities(work: &PreparedRenderWorkPlan) -> [GpuWorkResourceId; 3] {
        let resolve = work
            .graph()
            .nodes()
            .iter()
            .find_map(|prepared| match prepared.node().operation() {
                GpuWorkOperation::Resolve(operation) => Some(operation),
                _ => None,
            })
            .expect("timestamped work should contain a resolve operation");
        let query_set = resolve.source().diagnostic_identity();
        let resolve_buffer = resolve.destination().diagnostic_identity();
        let readback_buffer = work
            .graph()
            .nodes()
            .iter()
            .find_map(|prepared| match prepared.node().operation() {
                GpuWorkOperation::Copy(GpuCopyOperation::BufferToBuffer {
                    source,
                    destination,
                }) if source.buffer().diagnostic_identity() == resolve_buffer => {
                    Some(destination.buffer().diagnostic_identity())
                }
                _ => None,
            })
            .expect("timestamped work should contain a resolve-buffer readback copy");
        [query_set, resolve_buffer, readback_buffer]
    }

    fn semantic_topological_order(work: &PreparedRenderWorkPlan) -> Vec<(GpuWorkNodeKind, String)> {
        work.graph()
            .topological_order()
            .iter()
            .map(|id| {
                let node = work
                    .graph()
                    .nodes()
                    .iter()
                    .find(|node| node.id() == *id)
                    .expect("topological node should be prepared");
                (node.node().kind(), node.node().label().as_str().to_string())
            })
            .collect()
    }

    fn semantic_dependencies(
        work: &PreparedRenderWorkPlan,
    ) -> Vec<(String, String, Vec<&'static str>)> {
        let label_for = |id| {
            work.graph()
                .nodes()
                .iter()
                .find(|node| node.id() == id)
                .expect("dependency node should be prepared")
                .node()
                .label()
                .as_str()
                .to_string()
        };
        work.graph()
            .dependencies()
            .iter()
            .map(|dependency| {
                let reasons = dependency
                    .reasons()
                    .iter()
                    .map(|reason| match reason {
                        GpuDependencyReason::ReadAfterWrite { .. } => "read-after-write",
                        GpuDependencyReason::WriteAfterRead { .. } => "write-after-read",
                        GpuDependencyReason::WriteAfterWrite { .. } => "write-after-write",
                        GpuDependencyReason::ExplicitNonData { .. } => "explicit-non-data",
                    })
                    .collect();
                (
                    label_for(dependency.before()),
                    label_for(dependency.after()),
                    reasons,
                )
            })
            .collect()
    }

    #[test]
    fn adapter_lowers_render_roles_whole_resources_attachments_and_projected_dispatch() {
        let plan = adapter_test_plan();
        let compute_id = pass_id(&plan, "adapter.compute");
        let work = prepare_render_gpu_work(
            &plan,
            &BTreeMap::from([(compute_id, [7, 3, 2])]),
            (640, 360),
            RenderGpuWorkInstrumentation::Disabled,
        )
        .expect("render work should lower through G3");

        let compute = prepared_node(&work, "adapter.compute").node();
        let GpuWorkOperation::Compute(operation) = compute.operation() else {
            panic!("compute pass should lower to a compute operation");
        };
        assert_eq!(operation.dispatch().as_array(), [7, 3, 2]);
        let storage_access = compute
            .accesses()
            .iter()
            .find_map(|access| match access {
                GpuResourceAccess::Buffer(access)
                    if access.kind() == GpuBufferAccessKind::StorageReadWrite =>
                {
                    Some(access)
                }
                _ => None,
            })
            .expect("compute storage access should lower");
        assert_eq!(storage_access.range().offset(), 0);
        assert_eq!(
            storage_access.range().size(),
            storage_access.buffer().descriptor().size_bytes()
        );

        let render = prepared_node(&work, "adapter.render").node();
        let GpuWorkOperation::Render(operation) = render.operation() else {
            panic!("graphics pass should lower to a render operation");
        };
        assert_eq!(operation.draws().len(), 1);
        let [color] = operation.color_attachments() else {
            panic!("render operation should retain one color attachment");
        };
        let GpuColorAttachmentLoad::Clear(clear) = color.load() else {
            panic!("render color attachment should retain its clear");
        };
        assert_eq!(clear.components(), [0.125, 0.25, 0.5, 1.0]);
        assert_eq!(color.store(), GpuAttachmentStore::Store);
        assert_eq!(
            color.subresources(),
            GpuTextureSubresourceRange::whole(color.source().parent_texture())
                .expect("attachment whole range should be valid")
        );
        let depth = operation
            .depth_stencil_attachment()
            .expect("render operation should retain its depth attachment");
        let GpuDepthAttachmentLoad::Clear(clear) = depth.load() else {
            panic!("writable depth should lower to a clear");
        };
        assert_eq!(clear.value(), 1.0);
        assert_eq!(depth.store(), GpuAttachmentStore::Store);
    }

    #[test]
    fn adapter_lowers_timestamp_writes_query_resolve_and_readback_copy() {
        let plan = adapter_test_plan();
        let compute_id = pass_id(&plan, "adapter.compute");
        let work = prepare_render_gpu_work(
            &plan,
            &BTreeMap::from([(compute_id, [2, 1, 1])]),
            (320, 180),
            RenderGpuWorkInstrumentation::TimestampQueries,
        )
        .expect("timestamped render work should lower through G3");

        assert_eq!(work.graph().nodes().len(), 4);
        let compute = prepared_node(&work, "adapter.compute").node();
        let GpuWorkOperation::Compute(compute_operation) = compute.operation() else {
            panic!("compute pass should lower to compute work");
        };
        let [compute_timestamp] = compute_operation.timestamp_writes() else {
            panic!("compute operation should retain one timestamp range");
        };
        assert_eq!(compute_timestamp.kind(), GpuQueryAccessKind::WriteTimestamp);
        assert_eq!(compute_timestamp.range().first(), 0);
        assert_eq!(compute_timestamp.range().count(), 2);
        let derived_compute_timestamp = compute
            .accesses()
            .iter()
            .find_map(|access| match access {
                GpuResourceAccess::Query(access)
                    if access.kind() == GpuQueryAccessKind::WriteTimestamp =>
                {
                    Some(access)
                }
                _ => None,
            })
            .expect("compute timestamp access should lower");
        assert_eq!(derived_compute_timestamp, compute_timestamp);

        let render = prepared_node(&work, "adapter.render").node();
        let GpuWorkOperation::Render(render_operation) = render.operation() else {
            panic!("render pass should lower to render work");
        };
        let [render_timestamp] = render_operation.timestamp_writes() else {
            panic!("render operation should retain one timestamp range");
        };
        assert_eq!(render_timestamp.kind(), GpuQueryAccessKind::WriteTimestamp);
        assert_eq!(render_timestamp.range().first(), 2);
        assert_eq!(render_timestamp.range().count(), 2);

        let resolve = work
            .graph()
            .nodes()
            .iter()
            .find_map(|prepared| match prepared.node().operation() {
                GpuWorkOperation::Resolve(operation) => Some((prepared.id(), operation)),
                _ => None,
            })
            .expect("timestamp query resolve should lower");
        assert_eq!(resolve.1.source_range().first(), 0);
        assert_eq!(resolve.1.source_range().count(), 4);
        assert_eq!(resolve.1.destination_range().size(), 32);

        let readback = work
            .graph()
            .nodes()
            .iter()
            .find_map(|prepared| match prepared.node().operation() {
                GpuWorkOperation::Copy(GpuCopyOperation::BufferToBuffer {
                    source,
                    destination,
                }) if source.buffer() == resolve.1.destination() => {
                    Some((prepared.id(), source, destination))
                }
                _ => None,
            })
            .expect("resolve-buffer readback copy should lower");
        assert_eq!(readback.1.range(), resolve.1.destination_range());
        assert_ne!(readback.1.buffer(), readback.2.buffer());
        for identity in [
            resolve.1.source().diagnostic_identity(),
            resolve.1.destination().diagnostic_identity(),
            readback.2.buffer().diagnostic_identity(),
        ] {
            assert!(
                work.graph()
                    .initialization()
                    .iter()
                    .find(|summary| summary.resource().diagnostic_identity() == identity)
                    .expect("timing resource should have a prepared initialization summary")
                    .initial()
                    .is_none(),
                "timing resources must begin without implicit initialization",
            );
        }
        let query_coverage = work
            .graph()
            .initialization()
            .iter()
            .find(|summary| {
                summary.resource().diagnostic_identity() == resolve.1.source().diagnostic_identity()
            })
            .unwrap()
            .final_coverage()
            .unwrap()
            .query_range_values()
            .unwrap();
        assert_eq!(query_coverage, [resolve.1.source_range()]);
        let resolve_coverage = work
            .graph()
            .initialization()
            .iter()
            .find(|summary| {
                summary.resource().diagnostic_identity()
                    == resolve.1.destination().diagnostic_identity()
            })
            .unwrap()
            .final_coverage()
            .unwrap()
            .buffer_values()
            .unwrap();
        assert_eq!(
            resolve_coverage,
            [GpuBufferCoverage::dense(resolve.1.destination_range())]
        );
        let readback_coverage = work
            .graph()
            .initialization()
            .iter()
            .find(|summary| {
                summary.resource().diagnostic_identity()
                    == readback.2.buffer().diagnostic_identity()
            })
            .unwrap()
            .final_coverage()
            .unwrap()
            .buffer_values()
            .unwrap();
        assert_eq!(
            readback_coverage,
            [GpuBufferCoverage::dense(readback.2.range())]
        );
        assert!(work.graph().dependencies().iter().any(|dependency| {
            dependency.after() == resolve.0
                && dependency.reasons().iter().any(|reason| {
                    matches!(reason, GpuDependencyReason::ReadAfterWrite { resource, .. }
                        if *resource == resolve.1.source().diagnostic_identity())
                })
        }));
        assert!(work.graph().dependencies().iter().any(|dependency| {
            dependency.before() == resolve.0
                && dependency.after() == readback.0
                && dependency.reasons().iter().any(|reason| {
                    matches!(reason, GpuDependencyReason::ReadAfterWrite { resource, .. }
                        if *resource == resolve.1.destination().diagnostic_identity())
                })
        }));
        assert_eq!(
            work.graph()
                .requirements()
                .get(GpuCapabilityFeature::TimestampQuery),
            Some(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::TimestampQuery
            ))
        );
    }

    #[test]
    fn timing_resources_use_a_fresh_scope_without_changing_prepared_semantics() {
        let plan = adapter_test_plan();
        let compute_id = pass_id(&plan, "adapter.compute");
        let dispatches = BTreeMap::from([(compute_id, [2, 1, 1])]);
        let first = prepare_render_gpu_work(
            &plan,
            &dispatches,
            (320, 180),
            RenderGpuWorkInstrumentation::TimestampQueries,
        )
        .expect("first timestamped work should prepare");
        let second = prepare_render_gpu_work(
            &plan,
            &dispatches,
            (320, 180),
            RenderGpuWorkInstrumentation::TimestampQueries,
        )
        .expect("second timestamped work should prepare");

        let flow_owner_scopes = plan
            .resources
            .resources
            .iter()
            .map(|resource| resource.id().diagnostic_parts().0)
            .collect::<BTreeSet<_>>();
        let first_timing = timing_resource_identities(&first);
        let second_timing = timing_resource_identities(&second);
        let first_timing_scopes = first_timing
            .iter()
            .map(|identity| identity.diagnostic_parts().0)
            .collect::<BTreeSet<_>>();
        let second_timing_scopes = second_timing
            .iter()
            .map(|identity| identity.diagnostic_parts().0)
            .collect::<BTreeSet<_>>();

        assert_eq!(first_timing_scopes.len(), 1);
        assert_eq!(second_timing_scopes.len(), 1);
        assert!(first_timing_scopes.is_disjoint(&flow_owner_scopes));
        assert!(second_timing_scopes.is_disjoint(&flow_owner_scopes));
        assert_ne!(first_timing_scopes, second_timing_scopes);
        assert_eq!(
            first_timing
                .iter()
                .map(|identity| identity.diagnostic_parts().1)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            second_timing
                .iter()
                .map(|identity| identity.diagnostic_parts().1)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            semantic_topological_order(&first),
            semantic_topological_order(&second)
        );
        assert_eq!(
            semantic_dependencies(&first),
            semantic_dependencies(&second)
        );
        assert_eq!(
            first
                .ordered_render_pass_ids()
                .expect("first prepared graph should order render passes"),
            second
                .ordered_render_pass_ids()
                .expect("second prepared graph should order render passes")
        );
    }

    #[test]
    fn sidecar_enforces_bijection_and_structured_identity_errors() {
        let plan = adapter_test_plan();
        let compute_id = pass_id(&plan, "adapter.compute");
        let work = prepare_render_gpu_work(
            &plan,
            &BTreeMap::from([(compute_id, [1, 1, 1])]),
            (64, 64),
            RenderGpuWorkInstrumentation::TimestampQueries,
        )
        .expect("timestamped render work should lower");
        assert_eq!(work.sidecar.entries.len(), work.graph.nodes().len());
        assert!(
            work.graph
                .nodes()
                .iter()
                .all(|node| work.sidecar.get(&work.graph, node.id()).is_ok())
        );

        let missing_id = work.graph.nodes()[0].id();
        let mut missing = work.sidecar.clone();
        missing.entries.remove(&missing_id);
        assert!(matches!(
            missing.finish(&work.graph),
            Err(RenderGpuWorkAdapterError::MissingSidecarPayload { node_id })
                if node_id == missing_id
        ));

        let payload = work
            .sidecar
            .get(&work.graph, missing_id)
            .expect("test payload should exist")
            .clone();
        let mut duplicate = RenderGpuWorkSidecar::default();
        duplicate
            .insert(&work.graph, missing_id, payload.clone())
            .expect("first insertion should succeed");
        assert!(matches!(
            duplicate.insert(&work.graph, missing_id, payload),
            Err(RenderGpuWorkAdapterError::DuplicateSidecarPayload { node_id })
                if node_id == missing_id
        ));

        let one_pass = RenderFlow::new("adapter.sidecar.one")
            .compute_pass("only")
            .dispatch([1, 1, 1])
            .finish()
            .validate()
            .expect("one-pass flow should validate");
        let one_plan = compile_flow_plan(&one_pass).expect("one-pass flow should compile");
        let one_work = one_plan
            .structural_work()
            .expect("compiled flow should retain structural G3 work");
        let one_id = one_work.graph.nodes()[0].id();
        let mut mismatch = RenderGpuWorkSidecar::default();
        assert!(matches!(
            mismatch.insert(
                &one_work.graph,
                one_id,
                RenderGpuWorkPayload::TimingReadbackCopy,
            ),
            Err(RenderGpuWorkAdapterError::SidecarOperationKindMismatch {
                node_id,
                expected: GpuWorkNodeKind::Copy,
                actual: GpuWorkNodeKind::Compute,
            }) if node_id == one_id
        ));

        let foreign_id = *work
            .graph
            .topological_order()
            .last()
            .expect("timestamp work should contain a tail node");
        assert!(foreign_id.local_node() > one_id.local_node());
        let foreign_payload = work
            .sidecar
            .get(&work.graph, foreign_id)
            .expect("foreign test payload should exist")
            .clone();
        let mut foreign = RenderGpuWorkSidecar::default();
        assert!(matches!(
            foreign.insert(&one_work.graph, foreign_id, foreign_payload),
            Err(RenderGpuWorkAdapterError::ForeignPreparedNode { node_id })
                if node_id == foreign_id
        ));
    }

    #[test]
    fn prepared_graph_order_is_authoritative_and_sidecar_insertion_is_non_semantic() {
        let plan = adapter_test_plan();
        let compute_id = pass_id(&plan, "adapter.compute");
        let work = prepare_render_gpu_work(
            &plan,
            &BTreeMap::from([(compute_id, [1, 1, 1])]),
            (64, 64),
            RenderGpuWorkInstrumentation::TimestampQueries,
        )
        .expect("timestamped render work should lower");
        let expected = work
            .graph
            .topological_order()
            .iter()
            .filter_map(
                |node_id| match work.sidecar.get(&work.graph, *node_id).ok()? {
                    RenderGpuWorkPayload::Pass(pass) => Some(execution_pass_id(pass)),
                    RenderGpuWorkPayload::TimingResolve
                    | RenderGpuWorkPayload::TimingReadbackCopy => None,
                },
            )
            .collect::<Vec<_>>();
        assert_eq!(
            work.ordered_render_pass_ids()
                .expect("prepared order should map through sidecar"),
            expected
        );

        let mut reversed_entries = work.sidecar.entries.iter().collect::<Vec<_>>();
        reversed_entries.reverse();
        let mut reversed = RenderGpuWorkSidecar::default();
        for (node_id, payload) in reversed_entries {
            reversed
                .insert(&work.graph, *node_id, payload.clone())
                .expect("reordered sidecar insertion should remain valid");
        }
        let reversed = reversed
            .finish(&work.graph)
            .expect("reordered sidecar should preserve the bijection");
        let reordered_work = PreparedRenderWorkPlan {
            graph: work.graph.clone(),
            sidecar: reversed,
        };
        assert_eq!(
            reordered_work
                .ordered_render_pass_ids()
                .expect("prepared graph order should remain available"),
            expected
        );
    }
}
