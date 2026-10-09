//! Temporary G3/G5 bridge from fully resolved render execution operations to backend-neutral
//! RunenGPU work.
//!
//! G5A deliberately requires this adapter to receive execution-complete `GpuWorkOperation`
//! values. It does not allocate logical GPU resources, reconstruct operation accesses, invent
//! pipeline/binding state, or project renderer declarations into a second GPU identity space.
//! The prepared graph remains the only access, initialization, hazard, capability, dependency,
//! and topological-order authority.
//!
//! A compiled render pass is not an execution identity: fixed-step regions may execute one pass
//! repeatedly and feature gates may omit an occurrence. RunenRender therefore supplies distinct
//! occurrence identities plus only render-owned control/non-data requirements. RunenGPU continues
//! to derive every resource dependency and hazard from the canonical operations.

use crate::plugins::render::RunenUiPublicationId;
use runen_gpu::*;
use runen_render::execution_2d::{Render2dContributionToken, Render2dPreparedContribution};
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
        "resolved render control order references occurrence '{occurrence}' that is absent from this bounded render work"
    )]
    MissingOrderedOccurrence {
        occurrence: RenderGpuWorkOccurrenceId,
    },
    #[error("render GPU work could not map fragment-local node {local_node}")]
    MissingPreparedNodeMapping { local_node: u64 },
    #[error(
        "renderer composed timing observation work interleaves measured GPU work in the prepared dependency graph"
    )]
    InterleavedTimingObservation,
    #[error("the single-authority mixed UI GPU operation is invalid: {0}")]
    InvalidMixedUi(&'static str),
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
    operation: Option<GpuWorkOperation>,
    preference: GpuExecutionPreference,
    provenance: GpuResourceProvenance,
    control_order_after: Vec<RenderGpuWorkOccurrenceId>,
    ui_composite: bool,
}

impl ResolvedRenderGpuWorkNode {
    pub(crate) fn with_builtin_ui_composite(mut self) -> Self {
        self.ui_composite = true;
        self
    }
    /// A source-only F2 pass still participates in normal render controls even
    /// when no pre-existing generic draws and no timestamp operation exist.
    pub(crate) fn empty_builtin_ui_composite(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation: None,
            preference: GpuExecutionPreference::GraphicsRequired,
            provenance,
            control_order_after: control_order_after.into_iter().collect(),
            ui_composite: true,
        }
    }

    pub(crate) const fn builtin_ui_occurrence(&self) -> Option<RenderGpuWorkOccurrenceId> {
        if self.ui_composite {
            Some(self.occurrence)
        } else {
            None
        }
    }

    pub(crate) fn pass(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        operation: GpuWorkOperation,
        preference: GpuExecutionPreference,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation: Some(operation),
            preference,
            provenance,
            control_order_after: control_order_after.into_iter().collect(),
            ui_composite: false,
        }
    }

    pub(crate) fn upload(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        operation: GpuUploadOperation,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation: Some(GpuWorkOperation::Upload(operation)),
            preference: GpuExecutionPreference::TransferPreferred,
            provenance,
            control_order_after: control_order_after.into_iter().collect(),
            ui_composite: false,
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
            operation: Some(GpuWorkOperation::Resolve(operation)),
            preference: GpuExecutionPreference::TransferPreferred,
            provenance,
            control_order_after: control_order_after.into_iter().collect(),
            ui_composite: false,
        }
    }

    pub(crate) fn timing_readback(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        operation: GpuReadbackOperation,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation: Some(GpuWorkOperation::Readback(operation)),
            preference: GpuExecutionPreference::TransferPreferred,
            provenance,
            control_order_after: control_order_after.into_iter().collect(),
            ui_composite: false,
        }
    }

    pub(crate) fn capture_readback(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        operation: GpuReadbackOperation,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation: Some(GpuWorkOperation::Readback(operation)),
            preference: GpuExecutionPreference::TransferPreferred,
            provenance,
            control_order_after: control_order_after.into_iter().collect(),
            ui_composite: false,
        }
    }

    /// Frame-terminal presentation enters only the canonical GPU work graph. There is
    /// intentionally no renderer executor payload or alternate presentation authority.
    pub(crate) fn present(
        occurrence: RenderGpuWorkOccurrenceId,
        label: GpuResourceLabel,
        operation: GpuPresentOperation,
        control_order_after: impl IntoIterator<Item = RenderGpuWorkOccurrenceId>,
    ) -> Self {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self {
            occurrence,
            label,
            operation: Some(GpuWorkOperation::Present(operation)),
            preference: GpuExecutionPreference::Automatic,
            provenance,
            control_order_after: control_order_after.into_iter().collect(),
            ui_composite: false,
        }
    }
}

struct AuthoredRenderFragment {
    fragment: GpuWorkFragment,
    occurrence_nodes: BTreeMap<RenderGpuWorkOccurrenceId, GpuWorkNodeId>,
    mixed_first_node: Option<GpuWorkNodeId>,
    mixed_tokens: Vec<(RunenUiPublicationId, Option<Render2dContributionToken>)>,
}

#[derive(Debug)]
pub(crate) struct RunenUiMixedWork {
    pub ui_occurrence: RenderGpuWorkOccurrenceId,
    /// Already ordered by the one shared UI compositor.
    pub legacy_draws: Vec<(u32, GpuRenderDraw)>,
    /// A source-neutral F2 contribution with the exact same compositor key.
    pub contributions: Vec<(RunenUiPublicationId, u32, Render2dPreparedContribution)>,
}

pub(crate) struct RunenUiMixedPreparedGraph {
    pub graph: GpuPreparedWorkGraph,
    pub f2_tokens: Vec<(RunenUiPublicationId, Option<Render2dContributionToken>)>,
    pub present_node: GpuWorkNodeId,
}

struct PreparedRenderGpuWorkFrame {
    graph: GpuPreparedWorkGraph,
    mixed_tokens: Vec<(RunenUiPublicationId, Option<Render2dContributionToken>)>,
    present_node: Option<GpuWorkNodeId>,
}

#[derive(Debug, Clone)]
pub(crate) struct RenderGpuFrameTimingBracket {
    fragment: GpuWorkFragment,
    start: GpuWorkNodeId,
    end: GpuWorkNodeId,
    observation_tail: GpuWorkNodeId,
}

impl RenderGpuFrameTimingBracket {
    pub(crate) fn new(
        fragment: GpuWorkFragment,
        start: GpuWorkNodeId,
        end: GpuWorkNodeId,
        observation_tail: GpuWorkNodeId,
    ) -> Self {
        Self {
            fragment,
            start,
            end,
            observation_tail,
        }
    }

    fn fragment(&self) -> &GpuWorkFragment {
        &self.fragment
    }

    fn start(&self) -> &GpuWorkNodeId {
        &self.start
    }

    fn end(&self) -> &GpuWorkNodeId {
        &self.end
    }

    fn observation_tail(&self) -> &GpuWorkNodeId {
        &self.observation_tail
    }
}

/// Composable producer fragments and their imported resources share one
/// external GPU-work boundary when the renderer prepares the canonical graph.
#[derive(Clone, Copy)]
pub(crate) struct RenderGpuExternalWork<'a> {
    pub(crate) producer_fragments: &'a [GpuWorkFragment],
    pub(crate) imports: &'a [GpuWorkImport],
}

/// Owns only the exact mixed UI node authoring seam in the existing canonical
/// graph. The source-neutral F2 contribution is consumed once in the *final*
/// fragment; provisional G3 hazard/control discovery never mints its token.
pub(crate) fn prepare_render_gpu_frame_work_with_mixed_ui(
    context: &GpuContext,
    graph_label: GpuResourceLabel,
    nodes: impl IntoIterator<Item = ResolvedRenderGpuWorkNode>,
    external: RenderGpuExternalWork<'_>,
    timing_bracket: Option<&RenderGpuFrameTimingBracket>,
    mixed: RunenUiMixedWork,
    present_occurrence: RenderGpuWorkOccurrenceId,
) -> Result<RunenUiMixedPreparedGraph, RenderGpuWorkAdapterError> {
    let RenderGpuExternalWork {
        producer_fragments,
        imports,
    } = external;
    let nodes = nodes.into_iter().collect::<Vec<_>>();
    if mixed.legacy_draws.is_empty()
        && mixed
            .contributions
            .iter()
            .all(|(_, _, work)| !work.has_render_work())
    {
        // F2 explicitly declares legitimate nonpainting content (for example
        // an outline-free space glyph). It emits no GPU node. Contract the
        // corresponding no-work UI occurrence through its existing render
        // predecessors, then prove the *same* frame's terminal Present.
        let ui = nodes
            .iter()
            .find(|node| node.occurrence == mixed.ui_occurrence)
            .filter(|node| node.ui_composite)
            .ok_or(RenderGpuWorkAdapterError::MissingOrderedOccurrence {
                occurrence: mixed.ui_occurrence,
            })?;
        let predecessor = ui.control_order_after.clone();
        let mut active = Vec::with_capacity(nodes.len() - 1);
        for mut node in nodes {
            if node.occurrence == mixed.ui_occurrence {
                continue;
            }
            if node.control_order_after.contains(&mixed.ui_occurrence) {
                let mut expanded = Vec::new();
                for before in &node.control_order_after {
                    if *before == mixed.ui_occurrence {
                        for prior in &predecessor {
                            if !expanded.contains(prior) {
                                expanded.push(*prior);
                            }
                        }
                    } else if !expanded.contains(before) {
                        expanded.push(*before);
                    }
                }
                node.control_order_after = expanded;
            }
            active.push(node);
        }
        let ids = mixed
            .contributions
            .into_iter()
            .map(|(id, _, _)| (id, None))
            .collect();
        let prepared = prepare_resolved_render_gpu_work(
            graph_label,
            active,
            RenderGpuExternalWork {
                producer_fragments,
                imports,
            },
            timing_bracket,
            None,
            Some(present_occurrence),
            |label, fragments, graph_orders| {
                context
                    .prepare_work_graph_with_orders(label, fragments, graph_orders)
                    .map_err(RenderGpuWorkAdapterError::from)
            },
        )?;
        let present_node =
            prepared
                .present_node
                .ok_or(RenderGpuWorkAdapterError::MissingOrderedOccurrence {
                    occurrence: present_occurrence,
                })?;
        return Ok(RunenUiMixedPreparedGraph {
            graph: prepared.graph,
            f2_tokens: ids,
            present_node,
        });
    }

    let prepared = prepare_resolved_render_gpu_work(
        graph_label,
        nodes,
        RenderGpuExternalWork {
            producer_fragments,
            imports,
        },
        timing_bracket,
        Some(mixed),
        Some(present_occurrence),
        |label, fragments, graph_orders| {
            context
                .prepare_work_graph_with_orders(label, fragments, graph_orders)
                .map_err(RenderGpuWorkAdapterError::from)
        },
    )?;
    let present_node =
        prepared
            .present_node
            .ok_or(RenderGpuWorkAdapterError::MissingOrderedOccurrence {
                occurrence: present_occurrence,
            })?;
    Ok(RunenUiMixedPreparedGraph {
        graph: prepared.graph,
        f2_tokens: prepared.mixed_tokens,
        present_node,
    })
}

/// Prepares the canonical frame together with renderer-owned composable work. Imports are added
/// only to the canonical consumer fragment; G3 therefore derives producer-to-visualizer ordering
/// from the typed export relationship rather than from fragment order or a product-authored edge.
pub(crate) fn prepare_render_gpu_frame_work(
    context: &GpuContext,
    graph_label: GpuResourceLabel,
    nodes: impl IntoIterator<Item = ResolvedRenderGpuWorkNode>,
    producer_fragments: &[GpuWorkFragment],
    imports: &[GpuWorkImport],
    timing_bracket: Option<&RenderGpuFrameTimingBracket>,
) -> Result<GpuPreparedWorkGraph, RenderGpuWorkAdapterError> {
    prepare_resolved_render_gpu_work(
        graph_label,
        nodes,
        RenderGpuExternalWork {
            producer_fragments,
            imports,
        },
        timing_bracket,
        None,
        None,
        |label, fragments, graph_orders| {
            context
                .prepare_work_graph_with_orders(label, fragments, graph_orders)
                .map_err(RenderGpuWorkAdapterError::from)
        },
    )
    .map(|prepared| prepared.graph)
}

#[cfg(test)]
fn prepare_render_gpu_frame_work_for_test(
    graph_label: GpuResourceLabel,
    nodes: impl IntoIterator<Item = ResolvedRenderGpuWorkNode>,
) -> Result<GpuPreparedWorkGraph, RenderGpuWorkAdapterError> {
    prepare_resolved_render_gpu_work(
        graph_label,
        nodes,
        RenderGpuExternalWork {
            producer_fragments: &[],
            imports: &[],
        },
        None,
        None,
        None,
        |label, fragments, graph_orders| {
            GpuPreparedWorkGraph::prepare_with_orders(label, fragments, graph_orders)
                .map_err(RenderGpuWorkAdapterError::from)
        },
    )
    .map(|prepared| prepared.graph)
}

#[cfg(test)]
pub(crate) fn prepare_render_gpu_frame_work_with_composition_for_test(
    graph_label: GpuResourceLabel,
    nodes: impl IntoIterator<Item = ResolvedRenderGpuWorkNode>,
    producer_fragments: &[GpuWorkFragment],
    imports: &[GpuWorkImport],
) -> Result<GpuPreparedWorkGraph, RenderGpuWorkAdapterError> {
    prepare_resolved_render_gpu_work(
        graph_label,
        nodes,
        RenderGpuExternalWork {
            producer_fragments,
            imports,
        },
        None,
        None,
        None,
        |label, fragments, graph_orders| {
            GpuPreparedWorkGraph::prepare_with_orders(label, fragments, graph_orders)
                .map_err(RenderGpuWorkAdapterError::from)
        },
    )
    .map(|prepared| prepared.graph)
}

#[cfg(test)]
fn prepare_render_gpu_frame_work_with_timing_for_test(
    graph_label: GpuResourceLabel,
    nodes: impl IntoIterator<Item = ResolvedRenderGpuWorkNode>,
    producer_fragments: &[GpuWorkFragment],
    imports: &[GpuWorkImport],
    timing_bracket: &RenderGpuFrameTimingBracket,
) -> Result<GpuPreparedWorkGraph, RenderGpuWorkAdapterError> {
    prepare_resolved_render_gpu_work(
        graph_label,
        nodes,
        RenderGpuExternalWork {
            producer_fragments,
            imports,
        },
        Some(timing_bracket),
        None,
        None,
        |label, fragments, graph_orders| {
            GpuPreparedWorkGraph::prepare_with_orders(label, fragments, graph_orders)
                .map_err(RenderGpuWorkAdapterError::from)
        },
    )
    .map(|prepared| prepared.graph)
}

/// Prepares one bounded render work set from execution-complete logical GPU occurrences.
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
fn prepare_resolved_render_gpu_work(
    graph_label: GpuResourceLabel,
    nodes: impl IntoIterator<Item = ResolvedRenderGpuWorkNode>,
    external: RenderGpuExternalWork<'_>,
    timing_bracket: Option<&RenderGpuFrameTimingBracket>,
    mut mixed: Option<RunenUiMixedWork>,
    present_occurrence: Option<RenderGpuWorkOccurrenceId>,
    mut prepare_graph: impl FnMut(
        GpuResourceLabel,
        Vec<GpuWorkFragment>,
        Vec<GpuGraphExplicitOrder>,
    ) -> Result<GpuPreparedWorkGraph, RenderGpuWorkAdapterError>,
) -> Result<PreparedRenderGpuWorkFrame, RenderGpuWorkAdapterError> {
    let RenderGpuExternalWork {
        producer_fragments,
        imports,
    } = external;
    let nodes = nodes.into_iter().collect::<Vec<_>>();
    validate_occurrences(&nodes)?;

    let graph_provenance = GpuResourceProvenance::new(graph_label.clone(), None, None);
    let mut resources = collect_operation_resources(&nodes)?;
    for import in imports {
        resources
            .entry(import.resource().diagnostic_identity())
            .or_insert_with(|| import.resource().clone());
    }
    let inputs = resources
        .values()
        .filter(|resource| {
            !matches!(resource, GpuResourceRef::QuerySet(_))
                && !imports.iter().any(|import| {
                    import.resource().diagnostic_identity() == resource.diagnostic_identity()
                })
        })
        .map(|resource| {
            Ok(GpuWorkResourceInput::new(
                resource.clone(),
                GpuInitialCoverage::descriptor_initialization(resource.clone())?,
                resource.common().provenance().clone(),
            )?)
        })
        .collect::<Result<Vec<_>, RenderGpuWorkAdapterError>>()?;
    let desired_control_orders = collect_desired_control_orders(&nodes);

    if let Some(mixed_work) = mixed.take() {
        let mixed_ui_occurrence = mixed_work.ui_occurrence;
        // Author RunenRender's single-use F2 contributions exactly once, in the
        // original canonical renderer fragment and shared UI painter order.
        let authored = author_render_fragment_mixed(
            &nodes,
            &resources,
            &inputs,
            &graph_label,
            &graph_provenance,
            imports,
            mixed_work,
        )?;
        let fragments =
            compose_frame_fragments(producer_fragments, &authored.fragment, timing_bracket);
        let provisional_graph = prepare_graph(graph_label.clone(), fragments, Vec::new())?;
        // One logical UiComposite may lower to multiple GPU nodes. Incoming
        // pass/capture controls MUST precede its first work node; outgoing
        // controls MUST follow its last work node. Using one occurrence as both
        // endpoints would allow source paint to race before a preceding pass.
        let satisfied_edges = provisional_graph
            .dependencies()
            .iter()
            .filter(|dependency| {
                dependency
                    .reasons()
                    .iter()
                    .any(|reason| reason.resource().is_some())
            })
            .map(|dependency| (dependency.before(), dependency.after()))
            .collect::<BTreeSet<_>>();
        let prepared_id =
            |node: &GpuWorkNodeId| -> Result<GpuPreparedWorkNodeId, RenderGpuWorkAdapterError> {
                provisional_graph
                    .nodes()
                    .iter()
                    .find(|prepared| prepared.node().id() == node)
                    .map(|prepared| prepared.id())
                    .ok_or(RenderGpuWorkAdapterError::MissingPreparedNodeMapping {
                        local_node: node.diagnostic_local(),
                    })
            };
        let mut graph_orders = Vec::new();
        for &(before_occurrence, after_occurrence) in &desired_control_orders {
            let before = authored.occurrence_nodes.get(&before_occurrence).ok_or(
                RenderGpuWorkAdapterError::MissingOrderedOccurrence {
                    occurrence: before_occurrence,
                },
            )?;
            let after = if after_occurrence == mixed_ui_occurrence {
                authored.mixed_first_node.as_ref()
            } else {
                authored.occurrence_nodes.get(&after_occurrence)
            }
            .ok_or(RenderGpuWorkAdapterError::MissingOrderedOccurrence {
                occurrence: after_occurrence,
            })?;
            let from = prepared_id(before)?;
            let to = prepared_id(after)?;
            if !dependency_path_exists(&satisfied_edges, from, to) {
                // G3 owns control ordering as well as typed hazards. F2 was
                // authored exactly once into the immutable fragment, so add
                // unsatisfied control edges at graph composition time using
                // the accepted same-fragment G3 contract.
                graph_orders.push(GpuGraphExplicitOrder::new(
                    before,
                    after,
                    "render-owned mixed UI occurrence control order",
                )?);
            }
        }
        if let Some(bracket) = timing_bracket {
            graph_orders.extend(compose_timing_graph_orders(
                &provisional_graph,
                producer_fragments,
                &authored.fragment,
                bracket,
            )?);
        }
        let graph = if graph_orders.is_empty() {
            provisional_graph
        } else {
            prepare_graph(
                graph_label,
                compose_frame_fragments(producer_fragments, &authored.fragment, timing_bracket),
                graph_orders,
            )?
        };
        let present_node = present_occurrence
            .and_then(|occurrence| authored.occurrence_nodes.get(&occurrence).cloned());
        return Ok(PreparedRenderGpuWorkFrame {
            graph,
            mixed_tokens: authored.mixed_tokens,
            present_node,
        });
    }

    let provisional = author_render_fragment(
        &nodes,
        &resources,
        &inputs,
        &graph_label,
        &graph_provenance,
        &BTreeSet::new(),
        imports,
    )?;
    let provisional_fragments =
        compose_frame_fragments(producer_fragments, &provisional.fragment, timing_bracket);
    let provisional_graph = prepare_graph(graph_label.clone(), provisional_fragments, Vec::new())?;
    let provisional_occurrences =
        map_prepared_occurrences(&provisional_graph, &provisional.occurrence_nodes)?;
    let required_explicit_orders = normalize_control_orders(
        &provisional_graph,
        &provisional_occurrences,
        &desired_control_orders,
    );

    if required_explicit_orders.is_empty() {
        let Some(bracket) = timing_bracket else {
            return Ok(PreparedRenderGpuWorkFrame {
                graph: provisional_graph,
                mixed_tokens: Vec::new(),
                present_node: present_occurrence
                    .and_then(|id| provisional.occurrence_nodes.get(&id).cloned()),
            });
        };
        let timing_orders = compose_timing_graph_orders(
            &provisional_graph,
            producer_fragments,
            &provisional.fragment,
            bracket,
        )?;
        let graph = prepare_graph(
            graph_label,
            compose_frame_fragments(producer_fragments, &provisional.fragment, Some(bracket)),
            timing_orders,
        )?;
        return Ok(PreparedRenderGpuWorkFrame {
            graph,
            mixed_tokens: Vec::new(),
            present_node: present_occurrence
                .and_then(|id| provisional.occurrence_nodes.get(&id).cloned()),
        });
    }

    let final_fragment = author_render_fragment(
        &nodes,
        &resources,
        &inputs,
        &graph_label,
        &graph_provenance,
        &required_explicit_orders,
        imports,
    )?;
    let present_node =
        present_occurrence.and_then(|id| final_fragment.occurrence_nodes.get(&id).cloned());
    let boundary_graph = prepare_graph(
        graph_label.clone(),
        compose_frame_fragments(producer_fragments, &final_fragment.fragment, timing_bracket),
        Vec::new(),
    )?;
    let Some(bracket) = timing_bracket else {
        return Ok(PreparedRenderGpuWorkFrame {
            graph: boundary_graph,
            mixed_tokens: final_fragment.mixed_tokens,
            present_node,
        });
    };
    let timing_orders = compose_timing_graph_orders(
        &boundary_graph,
        producer_fragments,
        &final_fragment.fragment,
        bracket,
    )?;
    let graph = prepare_graph(
        graph_label,
        compose_frame_fragments(producer_fragments, &final_fragment.fragment, Some(bracket)),
        timing_orders,
    )?;
    Ok(PreparedRenderGpuWorkFrame {
        graph,
        mixed_tokens: final_fragment.mixed_tokens,
        present_node,
    })
}

/// One checked operation inserted at the canonical UI pass position. The
/// renderer owns the surrounding legacy span, not RunenRender's private draw.
enum MixedUiEmission {
    Legacy(Box<GpuRenderOperation>),
    RunenUi(RunenUiPublicationId, Render2dPreparedContribution),
}

fn mixed_ui_emissions(
    render: &GpuRenderOperation,
    mixed: RunenUiMixedWork,
) -> Result<Vec<MixedUiEmission>, RenderGpuWorkAdapterError> {
    if !render
        .draws()
        .iter()
        .eq(mixed.legacy_draws.iter().map(|(_, draw)| draw))
    {
        return Err(RenderGpuWorkAdapterError::InvalidMixedUi(
            "canonical legacy draw list must exactly match the submitted mixed producer order",
        ));
    }
    if !mixed
        .legacy_draws
        .windows(2)
        .all(|pair| pair[0].0 <= pair[1].0)
        || !mixed
            .contributions
            .windows(2)
            .all(|pair| pair[0].1 < pair[1].1)
    {
        return Err(RenderGpuWorkAdapterError::InvalidMixedUi(
            "mixed source compositor positions must be monotonic and unique for F2 producers",
        ));
    }
    let mut result = Vec::new();
    let timestamps = render.timestamp_writes();
    // Timing spans the whole mixed UI pass, not only a legacy sibling.
    if let Some(ts) = timestamps.and_then(|t| t.beginning_of_pass().map(|start| (t, start))) {
        result.push(MixedUiEmission::Legacy(GpuRenderOperation::new(
            render.color_attachments().iter().cloned(),
            render.depth_stencil_attachment().cloned(),
            [],
            Some(GpuTimestampWrites::new(ts.0.query_set(), Some(ts.1), None)?),
        ).map(Box::new)?));
    }
    let mut legacy = mixed.legacy_draws.into_iter().peekable();
    let mut f2 = mixed.contributions.into_iter().peekable();
    while legacy.peek().is_some() || f2.peek().is_some() {
        let next_legacy = legacy.peek().map(|(position, _)| *position);
        let next_f2 = f2.peek().map(|(_, position, _)| *position);
        if next_legacy
            .is_some_and(|position| next_f2.is_none_or(|f2_position| position < f2_position))
        {
            let position = next_legacy.expect("checked some");
            let mut draws = Vec::new();
            while legacy.peek().is_some_and(|(at, _)| *at == position) {
                let (_, draw) = legacy.next().expect("checked legacy ordinal");
                draws.push(draw);
            }
            result.push(MixedUiEmission::Legacy(GpuRenderOperation::new(
                render.color_attachments().iter().cloned(),
                render.depth_stencil_attachment().cloned(),
                draws,
                None,
            ).map(Box::new)?));
        } else {
            let (id, _position, contribution) = f2.next().expect("checked F2 ordinal");
            result.push(MixedUiEmission::RunenUi(id, contribution));
        }
    }
    if let Some(ts) = timestamps.and_then(|t| t.end_of_pass().map(|end| (t, end))) {
        result.push(MixedUiEmission::Legacy(GpuRenderOperation::new(
            render.color_attachments().iter().cloned(),
            render.depth_stencil_attachment().cloned(),
            [],
            Some(GpuTimestampWrites::new(ts.0.query_set(), None, Some(ts.1))?),
        ).map(Box::new)?));
    }
    Ok(result)
}

fn author_render_fragment_mixed(
    nodes: &[ResolvedRenderGpuWorkNode],
    resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    inputs: &[GpuWorkResourceInput],
    graph_label: &GpuResourceLabel,
    graph_provenance: &GpuResourceProvenance,
    imports: &[GpuWorkImport],
    mixed: RunenUiMixedWork,
) -> Result<AuthoredRenderFragment, RenderGpuWorkAdapterError> {
    let ui_occurrence = mixed.ui_occurrence;
    let ui_node = nodes
        .iter()
        .find(|node| node.occurrence == ui_occurrence)
        .ok_or(RenderGpuWorkAdapterError::MissingOrderedOccurrence {
            occurrence: ui_occurrence,
        })?;
    if !ui_node.ui_composite {
        return Err(RenderGpuWorkAdapterError::InvalidMixedUi(
            "mixed compositor occurrence is not the admitted builtin UI pass",
        ));
    }
    let emissions = match &ui_node.operation {
        Some(GpuWorkOperation::Render(base)) => mixed_ui_emissions(base, mixed)?,
        None if mixed.legacy_draws.is_empty() => mixed
            .contributions
            .into_iter()
            .map(|(id, _, contribution)| MixedUiEmission::RunenUi(id, contribution))
            .collect(),
        _ => {
            return Err(RenderGpuWorkAdapterError::InvalidMixedUi(
                "builtin UI work is incompatible with its source producer scope",
            ));
        }
    };
    let mut occurrence_nodes = BTreeMap::new();
    let mut mixed_tokens = Vec::new();
    let mut ui_first_node = None;
    let mut ui_last_node = None;
    let mut invalid_f2_token = false;
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
            for import in imports {
                builder.add_import(import.clone())?;
            }
            let mut emissions = Some(emissions);
            for node in nodes {
                if node.occurrence == ui_occurrence {
                    for emission in emissions.take().expect("exact one UI occurrence") {
                        match emission {
                            MixedUiEmission::Legacy(render) => {
                                let authored = builder.add_node(
                                    node.label.clone(),
                                    GpuWorkOperation::Render(*render),
                                    [],
                                    GpuCapabilityRequirements::new(),
                                    GpuExecutionPreference::GraphicsRequired,
                                    node.provenance.clone(),
                                )?;
                                ui_first_node.get_or_insert_with(|| authored.clone());
                                ui_last_node = Some(authored);
                            }
                            MixedUiEmission::RunenUi(id, contribution) => {
                                let token = contribution.append_to(builder)?;
                                if let Some(token) = token.as_ref() {
                                    // An F2 emission may author several GPU nodes.
                                    // Never use one presumed node index as both
                                    // its execution-order frontiers.
                                    if let Some(first) = token.authored_nodes().first() {
                                        let last =
                                            token.authored_nodes().last().expect("first exists");
                                        ui_first_node.get_or_insert_with(|| first.clone());
                                        ui_last_node = Some(last.clone());
                                    } else {
                                        // The builder closure can only return G3 authoring
                                        // errors; validate the F2-specific invariant after
                                        // immutable fragment authoring instead.
                                        invalid_f2_token = true;
                                    }
                                }
                                mixed_tokens.push((id, token));
                            }
                        }
                    }
                } else {
                    let id = builder.add_node(
                        node.label.clone(),
                        node.operation
                            .clone()
                            .expect("legacy renderer node always has executable work"),
                        [],
                        GpuCapabilityRequirements::new(),
                        node.preference,
                        node.provenance.clone(),
                    )?;
                    occurrence_nodes.insert(node.occurrence, id);
                }
            }
            Ok(())
        },
    )?;
    if invalid_f2_token {
        return Err(RenderGpuWorkAdapterError::InvalidMixedUi(
            "painting F2 token has no authored GPU node",
        ));
    }
    let last_node = ui_last_node.ok_or(RenderGpuWorkAdapterError::InvalidMixedUi(
        "an admitted mixed UI pass produced no executable GPU node",
    ))?;
    occurrence_nodes.insert(ui_occurrence, last_node);
    let first_node = ui_first_node.ok_or(RenderGpuWorkAdapterError::InvalidMixedUi(
        "mixed UI first work node identity was lost",
    ))?;
    Ok(AuthoredRenderFragment {
        fragment,
        occurrence_nodes,
        mixed_first_node: Some(first_node),
        mixed_tokens,
    })
}

fn compose_frame_fragments(
    producer_fragments: &[GpuWorkFragment],
    renderer_fragment: &GpuWorkFragment,
    timing_bracket: Option<&RenderGpuFrameTimingBracket>,
) -> Vec<GpuWorkFragment> {
    let mut fragments = producer_fragments.to_vec();
    fragments.push(renderer_fragment.clone());
    if let Some(bracket) = timing_bracket {
        fragments.push(bracket.fragment().clone());
    }
    fragments
}

fn compose_timing_graph_orders(
    boundary_graph: &GpuPreparedWorkGraph,
    producer_fragments: &[GpuWorkFragment],
    renderer_fragment: &GpuWorkFragment,
    bracket: &RenderGpuFrameTimingBracket,
) -> Result<Vec<GpuGraphExplicitOrder>, RenderGpuWorkAdapterError> {
    let authored_nodes = producer_fragments
        .iter()
        .chain(std::iter::once(renderer_fragment))
        .flat_map(GpuWorkFragment::nodes)
        .collect::<Vec<_>>();
    let prepared_nodes = authored_nodes
        .iter()
        .map(|node| {
            boundary_graph
                .nodes()
                .iter()
                .find(|prepared| prepared.node().id() == node.id())
                .map(|prepared| (prepared.id(), *node))
                .ok_or(RenderGpuWorkAdapterError::MissingPreparedNodeMapping {
                    local_node: node.id().diagnostic_local(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let dependency_edges = boundary_graph
        .dependencies()
        .iter()
        .map(|dependency| (dependency.before(), dependency.after()))
        .collect::<BTreeSet<_>>();

    let measured = prepared_nodes
        .iter()
        .filter(|(_, node)| {
            !matches!(
                node.kind(),
                GpuWorkNodeKind::Present | GpuWorkNodeKind::Resolve | GpuWorkNodeKind::Readback
            )
        })
        .map(|(prepared, _)| *prepared)
        .collect::<BTreeSet<_>>();
    let observation = prepared_nodes
        .iter()
        .filter(|(_, node)| {
            matches!(
                node.kind(),
                GpuWorkNodeKind::Resolve | GpuWorkNodeKind::Readback
            )
        })
        .map(|(prepared, _)| *prepared)
        .collect::<BTreeSet<_>>();

    let is_root = |node: GpuPreparedWorkNodeId, set: &BTreeSet<GpuPreparedWorkNodeId>| {
        !set.iter().copied().any(|candidate| {
            candidate != node && dependency_path_exists(&dependency_edges, candidate, node)
        })
    };
    let is_sink = |node: GpuPreparedWorkNodeId, set: &BTreeSet<GpuPreparedWorkNodeId>| {
        !set.iter().copied().any(|candidate| {
            candidate != node && dependency_path_exists(&dependency_edges, node, candidate)
        })
    };

    if observation.iter().copied().any(|observation_node| {
        measured.iter().copied().any(|measured_before| {
            dependency_path_exists(&dependency_edges, measured_before, observation_node)
                && measured.iter().copied().any(|measured_after| {
                    dependency_path_exists(&dependency_edges, observation_node, measured_after)
                })
        })
    }) {
        return Err(RenderGpuWorkAdapterError::InterleavedTimingObservation);
    }

    let authored_id = |prepared: GpuPreparedWorkNodeId| {
        prepared_nodes
            .iter()
            .find(|(candidate, _)| *candidate == prepared)
            .map(|(_, node)| node.id())
            .expect("timing frontier prepared node must map to authored work")
    };

    let mut graph_orders = Vec::new();
    for &root in measured.iter().filter(|&&node| is_root(node, &measured)) {
        graph_orders.push(GpuGraphExplicitOrder::new(
            bracket.start(),
            authored_id(root),
            "renderer composed timing starts before measured dependency frontier",
        )?);
    }
    for &sink in measured.iter().filter(|&&node| is_sink(node, &measured)) {
        graph_orders.push(GpuGraphExplicitOrder::new(
            authored_id(sink),
            bracket.end(),
            "renderer composed timing ends after measured dependency frontier",
        )?);
    }
    for &root in observation
        .iter()
        .filter(|&&node| is_root(node, &observation))
    {
        graph_orders.push(GpuGraphExplicitOrder::new(
            bracket.end(),
            authored_id(root),
            "renderer composed timing ends before observation tail",
        )?);
    }
    for (_, node) in prepared_nodes
        .iter()
        .filter(|(_, node)| node.kind() == GpuWorkNodeKind::Present)
    {
        graph_orders.push(GpuGraphExplicitOrder::new(
            bracket.observation_tail(),
            node.id(),
            "renderer composed timing observation completes before terminal presentation",
        )?);
    }
    Ok(graph_orders)
}

fn validate_occurrences(
    nodes: &[ResolvedRenderGpuWorkNode],
) -> Result<(), RenderGpuWorkAdapterError> {
    let mut occurrences = BTreeSet::new();
    for node in nodes {
        if !occurrences.insert(node.occurrence) {
            return Err(RenderGpuWorkAdapterError::DuplicateOccurrence {
                occurrence: node.occurrence,
            });
        }
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
    imports: &[GpuWorkImport],
) -> Result<AuthoredRenderFragment, RenderGpuWorkAdapterError> {
    for occurrence in explicit_orders
        .iter()
        .flat_map(|(before, after)| [before, after])
    {
        if !nodes.iter().any(|node| node.occurrence == *occurrence) {
            return Err(RenderGpuWorkAdapterError::MissingOrderedOccurrence {
                occurrence: *occurrence,
            });
        }
    }

    let mut occurrence_nodes = BTreeMap::<RenderGpuWorkOccurrenceId, GpuWorkNodeId>::new();
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
            for import in imports {
                builder.add_import(import.clone())?;
            }

            for node in nodes {
                let node_id = builder.add_node(
                    node.label.clone(),
                    node.operation
                        .clone()
                        .expect("legacy renderer node always has executable work"),
                    [],
                    GpuCapabilityRequirements::new(),
                    node.preference,
                    node.provenance.clone(),
                )?;
                occurrence_nodes.insert(node.occurrence, node_id.clone());
            }

            for (before_occurrence, after_occurrence) in explicit_orders {
                let before = occurrence_nodes
                    .get(before_occurrence)
                    .expect("validated render occurrence predecessor must be authored");
                let after = occurrence_nodes
                    .get(after_occurrence)
                    .expect("validated render occurrence successor must be authored");
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
        mixed_first_node: None,
        mixed_tokens: Vec::new(),
    })
}

fn map_prepared_occurrences(
    graph: &GpuPreparedWorkGraph,
    occurrence_nodes: &BTreeMap<RenderGpuWorkOccurrenceId, GpuWorkNodeId>,
) -> Result<BTreeMap<RenderGpuWorkOccurrenceId, GpuPreparedWorkNodeId>, RenderGpuWorkAdapterError> {
    occurrence_nodes
        .iter()
        .map(|(occurrence, node_id)| {
            let local = node_id.diagnostic_local();
            let prepared = graph
                .nodes()
                .iter()
                .find(|prepared| prepared.node().id() == node_id)
                .map(|prepared| prepared.id())
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
    // Only typed G3 data dependencies may suppress a renderer-owned control order. The
    // provisional graph can also contain composed-timing graph orders; those are instrumentation
    // constraints and must never become authority for renderer control semantics.
    let mut satisfied_edges = provisional_graph
        .dependencies()
        .iter()
        .filter(|dependency| {
            dependency
                .reasons()
                .iter()
                .any(|reason| reason.resource().is_some())
        })
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
        let Some(operation) = node.operation.as_ref() else {
            continue;
        };
        for access in operation.derived_accesses()? {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn label(value: &str) -> GpuResourceLabel {
        GpuResourceLabel::new(value).expect("test label should be valid")
    }

    fn common(value: &str) -> GpuResourceCommon {
        let resource_label = label(value);
        GpuResourceCommon::owned(
            resource_label.clone(),
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
            GpuReconstruction::SourceBacked,
            GpuResourceProvenance::new(resource_label, None, None),
        )
        .expect("test resource common should be valid")
    }

    fn transfer_payload(name: &str, byte_len: usize) -> PreparedGpuData<TransferData> {
        PreparedGpuData::from_pod_transfer(
            name,
            &vec![0_u8; byte_len],
            GpuResourceProvenance::new(label(name), None, None),
        )
        .expect("test transfer payload should be valid")
    }

    fn buffer(
        allocator: &mut GpuWorkResourceIdAllocator,
        name: &str,
        byte_len: u64,
    ) -> GpuBufferHandle {
        let resource_label = label(name);
        allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common(name),
                    byte_len,
                    GpuBufferUsages::new(
                        &resource_label,
                        [GpuBufferUsage::CopySource, GpuBufferUsage::CopyDestination],
                    )
                    .expect("test buffer usage should be valid"),
                    GpuBufferInitialization::Uninitialized,
                )
                .expect("test buffer descriptor should be valid"),
            )
            .expect("test buffer handle should allocate")
    }

    fn zeroed_buffer(
        allocator: &mut GpuWorkResourceIdAllocator,
        name: &str,
        byte_len: u64,
    ) -> GpuBufferHandle {
        let resource_label = label(name);
        allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common(name),
                    byte_len,
                    GpuBufferUsages::new(
                        &resource_label,
                        [GpuBufferUsage::CopySource, GpuBufferUsage::CopyDestination],
                    )
                    .expect("test buffer usage should be valid"),
                    GpuBufferInitialization::Zeroed,
                )
                .expect("test zeroed buffer descriptor should be valid"),
            )
            .expect("test zeroed buffer handle should allocate")
    }

    fn whole_region(buffer: &GpuBufferHandle, byte_len: u64) -> GpuBufferRegion {
        GpuBufferRegion::new(
            buffer,
            GpuBufferRange::new(buffer, 0, byte_len).expect("test range should be valid"),
        )
        .expect("test region should be valid")
    }

    fn color_target_view(allocator: &mut GpuWorkResourceIdAllocator) -> GpuTextureViewHandle {
        let texture_label = label("frame surface color");
        let texture = allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::new(
                    common("frame surface color"),
                    GpuTextureDimension::D2,
                    GpuTextureExtent::new(&texture_label, GpuTextureDimension::D2, 4, 4, 1)
                        .expect("test surface extent should be valid"),
                    1,
                    1,
                    GpuTextureFormat::Rgba8Unorm,
                    GpuTextureUsages::new(&texture_label, [GpuTextureUsage::ColorAttachment])
                        .expect("test surface usage should be valid"),
                    GpuTextureInitialization::Zeroed,
                )
                .expect("test surface texture descriptor should be valid"),
            )
            .expect("test surface texture handle should allocate");
        let view_label = label("frame surface color view");
        allocator
            .allocate_texture_view_handle(
                GpuTextureViewDescriptor::new(
                    common("frame surface color view"),
                    &texture,
                    None,
                    GpuTextureViewDimension::D2,
                    GpuTextureSubresourceRange::new(
                        &view_label,
                        0,
                        1,
                        0,
                        1,
                        GpuTextureAspect::Color,
                    )
                    .expect("test surface subresources should be valid"),
                )
                .expect("test surface view descriptor should be valid"),
            )
            .expect("test surface view handle should allocate")
    }

    #[test]
    fn frame_work_preparation_owns_cross_invocation_raw_and_initialization() {
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let shared = buffer(&mut allocator, "frame shared", 16);
        let copied = buffer(&mut allocator, "frame copied", 16);
        let independent = buffer(&mut allocator, "frame independent", 16);

        let producer = RenderGpuWorkOccurrenceId::new(1);
        let consumer = RenderGpuWorkOccurrenceId::new(2);
        let unrelated = RenderGpuWorkOccurrenceId::new(3);
        let nodes = [
            ResolvedRenderGpuWorkNode::upload(
                producer,
                label("invocation a upload"),
                GpuUploadOperation::new(
                    whole_region(&shared, 16).into(),
                    transfer_payload("invocation a payload", 16),
                )
                .expect("producer upload should be valid"),
                [],
            ),
            ResolvedRenderGpuWorkNode::pass(
                consumer,
                label("invocation b read"),
                GpuWorkOperation::Copy(
                    GpuCopyOperation::buffer_to_buffer(
                        whole_region(&shared, 16),
                        whole_region(&copied, 16),
                    )
                    .expect("consumer copy should be valid"),
                ),
                GpuExecutionPreference::TransferPreferred,
                [],
            ),
            ResolvedRenderGpuWorkNode::upload(
                unrelated,
                label("invocation c independent"),
                GpuUploadOperation::new(
                    whole_region(&independent, 16).into(),
                    transfer_payload("invocation c payload", 16),
                )
                .expect("independent upload should be valid"),
                [],
            ),
        ];

        let prepared =
            prepare_render_gpu_frame_work_for_test(label("render frame test work"), nodes)
                .expect("bounded frame work should prepare");
        let node_id = |label: &str| {
            prepared
                .nodes()
                .iter()
                .find(|node| node.node().label().as_str() == label)
                .expect("prepared node label should exist")
                .id()
        };
        let producer_node = node_id("invocation a upload");
        let consumer_node = node_id("invocation b read");
        let unrelated_node = node_id("invocation c independent");

        assert!(prepared.dependencies().iter().any(|dependency| {
            dependency.before() == producer_node && dependency.after() == consumer_node
        }));
        assert!(prepared.dependencies().iter().all(|dependency| {
            dependency.before() != unrelated_node && dependency.after() != unrelated_node
        }));

        for buffer in [&shared, &copied, &independent] {
            let initialization = prepared
                .initialization()
                .iter()
                .find(|entry| {
                    entry.resource().diagnostic_identity() == buffer.diagnostic_identity()
                })
                .expect("frame graph should retain initialization evidence for every buffer");
            assert!(
                initialization.final_coverage().is_some(),
                "frame graph should own final initialization coverage for '{}'",
                buffer.descriptor().common().label().as_str()
            );
        }
    }

    #[test]
    fn capture_readback_control_order_survives_without_data_hazards() {
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let captured = buffer(&mut allocator, "capture source", 16);
        let pass_source = buffer(&mut allocator, "pass source", 16);
        let pass_destination = buffer(&mut allocator, "pass destination", 16);

        let capture_init = RenderGpuWorkOccurrenceId::new(1);
        let pass_init = RenderGpuWorkOccurrenceId::new(2);
        let before_capture = RenderGpuWorkOccurrenceId::new(3);
        let pass = RenderGpuWorkOccurrenceId::new(4);
        let after_capture = RenderGpuWorkOccurrenceId::new(5);
        let nodes = [
            ResolvedRenderGpuWorkNode::upload(
                capture_init,
                label("capture init"),
                GpuUploadOperation::new(
                    whole_region(&captured, 16).into(),
                    transfer_payload("capture init payload", 16),
                )
                .expect("capture source upload should be valid"),
                [],
            ),
            ResolvedRenderGpuWorkNode::upload(
                pass_init,
                label("pass init"),
                GpuUploadOperation::new(
                    whole_region(&pass_source, 16).into(),
                    transfer_payload("pass init payload", 16),
                )
                .expect("pass source upload should be valid"),
                [],
            ),
            ResolvedRenderGpuWorkNode::capture_readback(
                before_capture,
                label("capture before"),
                GpuReadbackOperation::new(
                    whole_region(&captured, 16).into(),
                    GpuReadbackId::allocate().expect("readback id should allocate"),
                )
                .expect("before capture readback should be valid"),
                [],
            ),
            ResolvedRenderGpuWorkNode::pass(
                pass,
                label("independent pass"),
                GpuWorkOperation::Copy(
                    GpuCopyOperation::buffer_to_buffer(
                        whole_region(&pass_source, 16),
                        whole_region(&pass_destination, 16),
                    )
                    .expect("independent pass copy should be valid"),
                ),
                GpuExecutionPreference::TransferPreferred,
                [before_capture],
            ),
            ResolvedRenderGpuWorkNode::capture_readback(
                after_capture,
                label("capture after"),
                GpuReadbackOperation::new(
                    whole_region(&captured, 16).into(),
                    GpuReadbackId::allocate().expect("readback id should allocate"),
                )
                .expect("after capture readback should be valid"),
                [pass],
            ),
        ];

        let prepared =
            prepare_render_gpu_frame_work_for_test(label("capture stage order test"), nodes)
                .expect("capture stage work should prepare");
        let prepared_node = |label: &str| {
            prepared
                .nodes()
                .iter()
                .find(|node| node.node().label().as_str() == label)
                .expect("prepared node label should exist")
        };
        let before_node = prepared_node("capture before").id();
        let pass_node = prepared_node("independent pass").id();
        let after_node = prepared_node("capture after").id();

        assert!(prepared.dependencies().iter().any(|dependency| {
            dependency.before() == before_node && dependency.after() == pass_node
        }));
        assert!(prepared.dependencies().iter().any(|dependency| {
            dependency.before() == pass_node && dependency.after() == after_node
        }));
        assert_eq!(
            prepared_node("capture before").node().kind(),
            GpuWorkNodeKind::Readback
        );
        assert_eq!(
            prepared_node("capture after").node().kind(),
            GpuWorkNodeKind::Readback
        );
    }

    #[test]
    fn presenting_frame_without_explicit_present_predecessors_ends_at_present() {
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let surface_view = color_target_view(&mut allocator);
        let independent = buffer(&mut allocator, "independent frame upload", 16);
        let render_occurrence = RenderGpuWorkOccurrenceId::new(1);
        let independent_occurrence = RenderGpuWorkOccurrenceId::new(2);
        let present_occurrence = RenderGpuWorkOccurrenceId::new(3);
        let render = GpuRenderOperation::new(
            [GpuRenderColorAttachment::new(
                surface_view.clone(),
                GpuColorAttachmentLoad::Clear(
                    GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0)
                        .expect("test clear value should be valid"),
                ),
                GpuAttachmentStore::Store,
                None,
            )
            .expect("test surface attachment should be valid")],
            None,
            std::iter::empty::<GpuRenderDraw>(),
            None,
        )
        .expect("test surface render should be valid");
        let present = GpuPresentOperation::new(
            surface_view.clone().into(),
            surface_view.descriptor().subresources(),
        )
        .expect("test Present should be valid");
        let nodes = [
            ResolvedRenderGpuWorkNode::pass(
                render_occurrence,
                label("surface render"),
                GpuWorkOperation::Render(render),
                GpuExecutionPreference::GraphicsRequired,
                [],
            ),
            ResolvedRenderGpuWorkNode::upload(
                independent_occurrence,
                label("independent upload"),
                GpuUploadOperation::new(
                    whole_region(&independent, 16).into(),
                    transfer_payload("independent upload payload", 16),
                )
                .expect("independent upload should be valid"),
                [],
            ),
            ResolvedRenderGpuWorkNode::present(
                present_occurrence,
                label("frame terminal Present"),
                present,
                [],
            ),
        ];

        let prepared =
            prepare_render_gpu_frame_work_for_test(label("terminal Present test"), nodes)
                .expect("G3 should order a zero-control Present from resource hazards");
        let prepared_node = |label: &str| {
            prepared
                .nodes()
                .iter()
                .find(|node| node.node().label().as_str() == label)
                .expect("prepared node label should exist")
        };
        let render_node = prepared_node("surface render").id();
        let present_node = prepared_node("frame terminal Present").id();

        assert_eq!(
            prepared
                .nodes()
                .iter()
                .filter(|node| node.node().kind() == GpuWorkNodeKind::Present)
                .count(),
            1
        );
        assert_eq!(prepared.topological_order().last(), Some(&present_node));
        let surface_hazard = prepared
            .dependencies()
            .iter()
            .find(|dependency| {
                dependency.before() == render_node && dependency.after() == present_node
            })
            .expect("G3 must order the surface write before Present");
        assert!(
            surface_hazard
                .reasons()
                .iter()
                .any(|reason| matches!(reason, GpuDependencyReason::ReadAfterWrite { .. }))
        );
        assert!(
            surface_hazard
                .reasons()
                .iter()
                .all(|reason| !matches!(reason, GpuDependencyReason::ExplicitNonData { .. }))
        );
    }

    #[test]
    fn composed_r32float_producer_and_visualizer_are_one_graph_with_one_present() {
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let source = zeroed_buffer(&mut allocator, "typed radiance source", 16);
        let destination = buffer(&mut allocator, "visualizer copy destination", 16);
        let texture_label = label("typed R32Float radiance");
        let radiance = allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::new(
                    common("typed R32Float radiance"),
                    GpuTextureDimension::D2,
                    GpuTextureExtent::new(&texture_label, GpuTextureDimension::D2, 2, 2, 1)
                        .expect("radiance extent should be valid"),
                    1,
                    1,
                    GpuTextureFormat::R32Float,
                    GpuTextureUsages::new(
                        &texture_label,
                        [
                            GpuTextureUsage::CopyDestination,
                            GpuTextureUsage::CopySource,
                        ],
                    )
                    .expect("radiance usage should be valid"),
                    GpuTextureInitialization::Uninitialized,
                )
                .expect("radiance texture descriptor should be valid"),
            )
            .expect("radiance texture handle should allocate");
        let region = GpuTextureCopyRegion::whole_base_mip(&radiance)
            .expect("radiance region should be valid");
        let producer_copy = GpuCopyOperation::buffer_to_texture(
            GpuBufferTextureLayout::new(&source, 0, 8, 2)
                .expect("radiance source layout should be valid"),
            region.clone(),
        )
        .expect("radiance producer copy should be valid");
        let export_key = GpuExportKey::new("runenrender.maintained.radiance.output.0")
            .expect("typed radiance export key should be valid");
        let producer_provenance =
            GpuResourceProvenance::new(label("typed radiance producer"), None, None);
        let mut producer_builder = GpuWorkFragmentBuilder::new(
            label("maintained R32Float producer"),
            producer_provenance.clone(),
        );
        producer_builder
            .declare_resource(GpuResourceRef::Buffer(source.clone()))
            .expect("producer source declaration should succeed");
        producer_builder
            .declare_resource(GpuResourceRef::Texture(radiance.clone()))
            .expect("producer radiance declaration should succeed");
        producer_builder
            .add_input(
                GpuWorkResourceInput::new(
                    GpuResourceRef::Buffer(source.clone()),
                    GpuInitialCoverage::descriptor_initialization(GpuResourceRef::Buffer(
                        source.clone(),
                    ))
                    .expect("producer source initialization should be valid"),
                    source.descriptor().common().provenance().clone(),
                )
                .expect("producer input should be constructible"),
            )
            .expect("producer input should be valid");
        producer_builder
            .operation("write maintained R32Float radiance", producer_copy)
            .expect("producer operation should succeed");
        producer_builder
            .add_output(
                GpuWorkOutput::new(
                    GpuExportRelationship::new(
                        GpuResourceRef::Texture(radiance.clone()),
                        export_key.clone(),
                        GpuResourceAccessIntent::Write,
                        producer_provenance.clone(),
                    ),
                    GpuInitialCoverage::texture_subresources(
                        &GpuTextureAccessResource::Texture(radiance.clone()),
                        [region.subresources()],
                    )
                    .expect("producer radiance coverage should be valid"),
                )
                .expect("producer output should be valid"),
            )
            .expect("producer output should be added");
        let producer = producer_builder
            .finish()
            .expect("producer fragment should finish");

        let consumer_occurrence = RenderGpuWorkOccurrenceId::new(10);
        let consumer_copy = GpuCopyOperation::texture_to_buffer(
            region,
            GpuBufferTextureLayout::new(&destination, 0, 8, 2)
                .expect("visualizer destination layout should be valid"),
        )
        .expect("visualizer copy should be valid");
        let consumer_provenance =
            GpuResourceProvenance::new(label("Render Lab visualizer"), None, None);
        let consumer = ResolvedRenderGpuWorkNode::pass(
            consumer_occurrence,
            label("Render Lab visualizer consumes typed radiance"),
            GpuWorkOperation::Copy(consumer_copy),
            GpuExecutionPreference::TransferPreferred,
            [],
        );
        let surface_view = color_target_view(&mut allocator);
        let present_occurrence = RenderGpuWorkOccurrenceId::new(11);
        let present = ResolvedRenderGpuWorkNode::present(
            present_occurrence,
            label("exactly one terminal Present"),
            GpuPresentOperation::new(
                surface_view.clone().into(),
                surface_view.descriptor().subresources(),
            )
            .expect("Present should be valid"),
            [consumer_occurrence],
        );
        let consumer_import = GpuWorkImport::new(
            GpuResourceRef::Texture(radiance.clone()),
            export_key,
            GpuResourceAccessIntent::Read,
            consumer_provenance,
        );
        let graph = prepare_render_gpu_frame_work_with_composition_for_test(
            label("RL2 composed R32Float frame"),
            [present, consumer],
            &[producer],
            &[consumer_import],
        )
        .expect("composed frame should prepare as one graph");

        let node_by_label = |expected: &str| {
            graph
                .nodes()
                .iter()
                .find(|node| node.node().label().as_str() == expected)
                .expect("composed node should exist")
        };
        let producer_node = node_by_label("write maintained R32Float radiance").id();
        let consumer_node = node_by_label("Render Lab visualizer consumes typed radiance").id();
        let present_node = node_by_label("exactly one terminal Present").id();
        let dependency = graph
            .dependencies()
            .iter()
            .find(|dependency| {
                dependency.before() == producer_node && dependency.after() == consumer_node
            })
            .expect("typed producer export/import must create producer-before-consumer dependency");
        assert!(dependency.reasons().iter().any(|reason| {
            matches!(reason, GpuDependencyReason::ReadAfterWrite { resource, .. }
                if *resource == radiance.diagnostic_identity())
        }));
        assert_eq!(
            graph
                .topological_order()
                .iter()
                .position(|node| *node == producer_node),
            Some(0)
        );
        assert_eq!(
            graph
                .topological_order()
                .iter()
                .position(|node| *node == consumer_node),
            Some(1)
        );
        assert_eq!(graph.topological_order().last(), Some(&present_node));
        assert_eq!(
            graph
                .nodes()
                .iter()
                .filter(|node| node.node().kind() == GpuWorkNodeKind::Present)
                .count(),
            1
        );
        assert!(
            graph
                .nodes()
                .iter()
                .all(|node| node.node().kind() != GpuWorkNodeKind::Readback)
        );
    }
    #[test]
    fn composed_timing_rejects_observation_interleaved_with_measured_work() {
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let shared = buffer(&mut allocator, "interleaved timing shared", 16);

        let first = ResolvedRenderGpuWorkNode::upload(
            RenderGpuWorkOccurrenceId::new(30),
            label("interleaved timing first upload"),
            GpuUploadOperation::new(
                whole_region(&shared, 16).into(),
                transfer_payload("interleaved timing first payload", 16),
            )
            .unwrap(),
            [],
        );
        let observation = ResolvedRenderGpuWorkNode::timing_readback(
            RenderGpuWorkOccurrenceId::new(31),
            label("interleaved timing observation"),
            GpuReadbackOperation::new(
                whole_region(&shared, 16).into(),
                GpuReadbackId::allocate().unwrap(),
            )
            .unwrap(),
            [],
        );
        let second = ResolvedRenderGpuWorkNode::upload(
            RenderGpuWorkOccurrenceId::new(32),
            label("interleaved timing second upload"),
            GpuUploadOperation::new(
                whole_region(&shared, 16).into(),
                transfer_payload("interleaved timing second payload", 16),
            )
            .unwrap(),
            [],
        );

        let query_label = label("interleaved timing markers");
        let query_set = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(
                    GpuResourceCommon::owned(
                        query_label.clone(),
                        GpuResourceLifetime::Transient,
                        GpuMemoryIntent::Device,
                        GpuReconstruction::SourceBacked,
                        GpuResourceProvenance::new(query_label, None, None),
                    )
                    .unwrap(),
                    GpuQueryKind::Timestamp,
                    2,
                )
                .unwrap(),
            )
            .unwrap();
        let resolve_label = label("interleaved timing resolve");
        let resolve_buffer = allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common("interleaved timing resolve"),
                    16,
                    GpuBufferUsages::new(
                        &resolve_label,
                        [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
                    )
                    .unwrap(),
                    GpuBufferInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        let mut start = None;
        let mut end = None;
        let mut readback_node = None;
        let marker_fragment =
            GpuWorkFragment::build("interleaved timing marker fragment", |work| {
                start = Some(work.operation(
                    "interleaved timing start",
                    GpuTimestampMarkerOperation::new(&query_set, 0).unwrap(),
                )?);
                end = Some(work.operation(
                    "interleaved timing end",
                    GpuTimestampMarkerOperation::new(&query_set, 1).unwrap(),
                )?);
                work.operation(
                    "interleaved timing resolve timestamps",
                    GpuQueryResolveOperation::new(
                        &query_set,
                        GpuQueryRange::new(&query_set, 0, 2).unwrap(),
                        &resolve_buffer,
                        0,
                    )
                    .unwrap(),
                )?;
                readback_node = Some(
                    work.operation(
                        "interleaved timing readback timestamps",
                        GpuReadbackOperation::new(
                            whole_region(&resolve_buffer, 16).into(),
                            GpuReadbackId::allocate().unwrap(),
                        )
                        .unwrap(),
                    )?,
                );
                Ok(())
            })
            .unwrap();
        let bracket = RenderGpuFrameTimingBracket::new(
            marker_fragment,
            start.unwrap(),
            end.unwrap(),
            readback_node.unwrap(),
        );

        let error = prepare_render_gpu_frame_work_with_timing_for_test(
            label("interleaved composed timing"),
            [first, observation, second],
            &[],
            &[],
            &bracket,
        )
        .expect_err("observation work between measured nodes must invalidate composed timing");

        assert!(matches!(
            error,
            RenderGpuWorkAdapterError::InterleavedTimingObservation
        ));
    }

    #[test]
    fn composed_timing_frontier_accepts_typed_producer_consumer_causality() {
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let source = zeroed_buffer(&mut allocator, "timed typed source", 16);
        let destination = buffer(&mut allocator, "timed typed destination", 16);
        let texture_label = label("timed typed radiance");
        let radiance = allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::new(
                    common("timed typed radiance"),
                    GpuTextureDimension::D2,
                    GpuTextureExtent::new(&texture_label, GpuTextureDimension::D2, 2, 2, 1)
                        .unwrap(),
                    1,
                    1,
                    GpuTextureFormat::R32Float,
                    GpuTextureUsages::new(
                        &texture_label,
                        [
                            GpuTextureUsage::CopyDestination,
                            GpuTextureUsage::CopySource,
                        ],
                    )
                    .unwrap(),
                    GpuTextureInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        let region = GpuTextureCopyRegion::whole_base_mip(&radiance).unwrap();
        let export_key = GpuExportKey::new("timed.typed.radiance.ready").unwrap();
        let producer_provenance =
            GpuResourceProvenance::new(label("timed typed producer"), None, None);
        let mut producer_builder = GpuWorkFragmentBuilder::new(
            label("timed typed producer fragment"),
            producer_provenance.clone(),
        );
        producer_builder
            .declare_resource(GpuResourceRef::Buffer(source.clone()))
            .unwrap();
        producer_builder
            .declare_resource(GpuResourceRef::Texture(radiance.clone()))
            .unwrap();
        producer_builder
            .add_input(
                GpuWorkResourceInput::new(
                    GpuResourceRef::Buffer(source.clone()),
                    GpuInitialCoverage::descriptor_initialization(GpuResourceRef::Buffer(
                        source.clone(),
                    ))
                    .unwrap(),
                    source.descriptor().common().provenance().clone(),
                )
                .unwrap(),
            )
            .unwrap();
        producer_builder
            .operation(
                "timed typed producer write",
                GpuCopyOperation::buffer_to_texture(
                    GpuBufferTextureLayout::new(&source, 0, 8, 2).unwrap(),
                    region.clone(),
                )
                .unwrap(),
            )
            .unwrap();
        producer_builder
            .add_output(
                GpuWorkOutput::new(
                    GpuExportRelationship::new(
                        GpuResourceRef::Texture(radiance.clone()),
                        export_key.clone(),
                        GpuResourceAccessIntent::Write,
                        producer_provenance,
                    ),
                    GpuInitialCoverage::texture_subresources(
                        &GpuTextureAccessResource::Texture(radiance.clone()),
                        [region.subresources()],
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        let producer = producer_builder.finish().unwrap();

        let consumer_provenance =
            GpuResourceProvenance::new(label("timed typed consumer"), None, None);
        let consumer = ResolvedRenderGpuWorkNode::pass(
            RenderGpuWorkOccurrenceId::new(20),
            label("timed typed consumer read"),
            GpuWorkOperation::Copy(
                GpuCopyOperation::texture_to_buffer(
                    region,
                    GpuBufferTextureLayout::new(&destination, 0, 8, 2).unwrap(),
                )
                .unwrap(),
            ),
            GpuExecutionPreference::TransferPreferred,
            [],
        );
        let surface_view = color_target_view(&mut allocator);
        let present = ResolvedRenderGpuWorkNode::present(
            RenderGpuWorkOccurrenceId::new(21),
            label("timed typed terminal Present"),
            GpuPresentOperation::new(
                surface_view.clone().into(),
                surface_view.descriptor().subresources(),
            )
            .unwrap(),
            [RenderGpuWorkOccurrenceId::new(20)],
        );
        let consumer_import = GpuWorkImport::new(
            GpuResourceRef::Texture(radiance),
            export_key,
            GpuResourceAccessIntent::Read,
            consumer_provenance,
        );

        let query_label = label("timed typed markers");
        let query_set = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(
                    GpuResourceCommon::owned(
                        query_label.clone(),
                        GpuResourceLifetime::Transient,
                        GpuMemoryIntent::Device,
                        GpuReconstruction::SourceBacked,
                        GpuResourceProvenance::new(query_label, None, None),
                    )
                    .unwrap(),
                    GpuQueryKind::Timestamp,
                    2,
                )
                .unwrap(),
            )
            .unwrap();
        let resolve_label = label("timed typed resolve");
        let resolve_buffer = allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common("timed typed resolve"),
                    16,
                    GpuBufferUsages::new(
                        &resolve_label,
                        [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
                    )
                    .unwrap(),
                    GpuBufferInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        let mut start = None;
        let mut end = None;
        let mut readback_node = None;
        let marker_fragment = GpuWorkFragment::build("timed typed marker fragment", |work| {
            start = Some(work.operation(
                "timed typed start",
                GpuTimestampMarkerOperation::new(&query_set, 0).unwrap(),
            )?);
            end = Some(work.operation(
                "timed typed end",
                GpuTimestampMarkerOperation::new(&query_set, 1).unwrap(),
            )?);
            work.operation(
                "timed typed resolve timestamps",
                GpuQueryResolveOperation::new(
                    &query_set,
                    GpuQueryRange::new(&query_set, 0, 2).unwrap(),
                    &resolve_buffer,
                    0,
                )
                .unwrap(),
            )?;
            readback_node = Some(
                work.operation(
                    "timed typed readback timestamps",
                    GpuReadbackOperation::new(
                        whole_region(&resolve_buffer, 16).into(),
                        GpuReadbackId::allocate().unwrap(),
                    )
                    .unwrap(),
                )?,
            );
            Ok(())
        })
        .unwrap();
        let bracket = RenderGpuFrameTimingBracket::new(
            marker_fragment,
            start.unwrap(),
            end.unwrap(),
            readback_node.unwrap(),
        );

        let graph = prepare_render_gpu_frame_work_with_timing_for_test(
            label("timed typed composed frame"),
            [present, consumer],
            &[producer],
            &[consumer_import],
            &bracket,
        )
        .expect("typed producer-consumer composition must prepare under frontier timing");

        let node = |name: &str| {
            graph
                .nodes()
                .iter()
                .find(|node| node.node().label().as_str() == name)
                .unwrap()
                .id()
        };
        let start = node("timed typed start");
        let producer = node("timed typed producer write");
        let consumer = node("timed typed consumer read");
        let end = node("timed typed end");
        let readback = node("timed typed readback timestamps");
        let present = node("timed typed terminal Present");
        let pos = |wanted| {
            graph
                .topological_order()
                .iter()
                .position(|node| *node == wanted)
                .unwrap()
        };

        assert!(pos(start) < pos(producer));
        assert!(pos(producer) < pos(consumer));
        assert!(pos(consumer) < pos(end));
        assert!(pos(end) < pos(readback));
        assert!(pos(readback) < pos(present));
        assert_eq!(graph.topological_order().last(), Some(&present));
        assert!(graph.dependencies().iter().any(|dependency| {
            dependency.before() == producer
                && dependency.after() == consumer
                && dependency
                    .reasons()
                    .iter()
                    .any(|reason| matches!(reason, GpuDependencyReason::ReadAfterWrite { .. }))
        }));
    }

    #[test]
    fn composed_timing_brackets_immutable_producer_and_renderer_work_before_present() {
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let producer_buffer = zeroed_buffer(&mut allocator, "timed immutable producer", 16);
        let producer = GpuWorkFragment::build("timed immutable producer fragment", |work| {
            work.operation(
                "timed producer clear",
                GpuClearOperation::buffer_zero(whole_region(&producer_buffer, 16)).unwrap(),
            )?;
            Ok(())
        })
        .unwrap();

        let surface_view = color_target_view(&mut allocator);
        let render = GpuRenderOperation::new(
            [GpuRenderColorAttachment::new(
                surface_view.clone(),
                GpuColorAttachmentLoad::Clear(GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap()),
                GpuAttachmentStore::Store,
                None,
            )
            .unwrap()],
            None,
            std::iter::empty::<GpuRenderDraw>(),
            None,
        )
        .unwrap();
        let independent_buffer = buffer(&mut allocator, "timed independent control", 16);
        let render_occurrence = RenderGpuWorkOccurrenceId::new(1);
        let independent_occurrence = RenderGpuWorkOccurrenceId::new(2);
        let present_occurrence = RenderGpuWorkOccurrenceId::new(3);
        let nodes = [
            ResolvedRenderGpuWorkNode::pass(
                render_occurrence,
                label("timed renderer work"),
                GpuWorkOperation::Render(render),
                GpuExecutionPreference::GraphicsRequired,
                [],
            ),
            ResolvedRenderGpuWorkNode::upload(
                independent_occurrence,
                label("timed independent control work"),
                GpuUploadOperation::new(
                    whole_region(&independent_buffer, 16).into(),
                    transfer_payload("timed independent control payload", 16),
                )
                .unwrap(),
                [],
            ),
            ResolvedRenderGpuWorkNode::present(
                present_occurrence,
                label("timed terminal Present"),
                GpuPresentOperation::new(
                    surface_view.clone().into(),
                    surface_view.descriptor().subresources(),
                )
                .unwrap(),
                [independent_occurrence],
            ),
        ];

        let query_label = label("timed frame markers");
        let query_common = GpuResourceCommon::owned(
            query_label.clone(),
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
            GpuReconstruction::SourceBacked,
            GpuResourceProvenance::new(query_label, None, None),
        )
        .unwrap();
        let query_set = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(query_common, GpuQueryKind::Timestamp, 2).unwrap(),
            )
            .unwrap();
        let resolve_label = label("timed frame timestamp resolve buffer");
        let resolve_buffer = allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common("timed frame timestamp resolve buffer"),
                    16,
                    GpuBufferUsages::new(
                        &resolve_label,
                        [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
                    )
                    .unwrap(),
                    GpuBufferInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        let resolve = GpuQueryResolveOperation::new(
            &query_set,
            GpuQueryRange::new(&query_set, 0, 2).unwrap(),
            &resolve_buffer,
            0,
        )
        .unwrap();
        let readback = GpuReadbackOperation::new(
            whole_region(&resolve_buffer, 16).into(),
            GpuReadbackId::allocate().unwrap(),
        )
        .unwrap();
        let mut start = None;
        let mut end = None;
        let mut readback_node = None;
        let marker_fragment = GpuWorkFragment::build("timed frame marker fragment", |work| {
            start = Some(
                work.operation(
                    "timed frame start",
                    GpuTimestampMarkerOperation::new(&query_set, 0).unwrap(),
                )
                .unwrap(),
            );
            end = Some(
                work.operation(
                    "timed frame end",
                    GpuTimestampMarkerOperation::new(&query_set, 1).unwrap(),
                )
                .unwrap(),
            );
            work.operation("timed frame timestamp resolve", resolve)?;
            readback_node = Some(work.operation("timed frame timestamp readback", readback)?);
            Ok(())
        })
        .unwrap();
        let bracket = RenderGpuFrameTimingBracket::new(
            marker_fragment,
            start.unwrap(),
            end.unwrap(),
            readback_node.unwrap(),
        );
        let graph = prepare_render_gpu_frame_work_with_timing_for_test(
            label("timed composed frame"),
            nodes,
            &[producer],
            &[],
            &bracket,
        )
        .unwrap();

        let node = |name: &str| {
            graph
                .nodes()
                .iter()
                .find(|node| node.node().label().as_str() == name)
                .unwrap()
                .id()
        };
        let start = node("timed frame start");
        let producer = node("timed producer clear");
        let renderer = node("timed renderer work");
        let independent = node("timed independent control work");
        let end = node("timed frame end");
        let resolve = node("timed frame timestamp resolve");
        let readback = node("timed frame timestamp readback");
        let present = node("timed terminal Present");
        let pos = |wanted| {
            graph
                .topological_order()
                .iter()
                .position(|node| *node == wanted)
                .unwrap()
        };
        assert!(pos(start) < pos(producer));
        assert!(pos(start) < pos(renderer));
        assert!(pos(start) < pos(independent));
        assert!(pos(producer) < pos(end));
        assert!(pos(renderer) < pos(end));
        assert!(pos(independent) < pos(end));
        assert!(pos(end) < pos(resolve));
        assert!(pos(resolve) < pos(readback));
        assert!(pos(readback) < pos(present));
        assert_eq!(graph.topological_order().last(), Some(&present));
        let independent_control = graph
            .dependencies()
            .iter()
            .find(|dependency| dependency.before() == independent && dependency.after() == present)
            .expect("timing must not suppress the renderer-owned independent control into Present");
        assert!(independent_control.reasons().iter().any(|reason| {
            matches!(
                reason,
                GpuDependencyReason::ExplicitNonData { reason }
                    if reason == "render-owned occurrence control order"
            )
        }));
        assert_eq!(
            graph
                .nodes()
                .iter()
                .filter(|node| node.node().kind() == GpuWorkNodeKind::Present)
                .count(),
            1
        );
    }
}

#[cfg(test)]
mod native_mixed_ui_tests {
    use super::*;
    use runen_render::composition_2d::{
        Render2dAffineTransform, Render2dBrush, Render2dColorRgba8, Render2dComposition,
        Render2dEntry, Render2dFontBinding, Render2dGlyph, Render2dItem, Render2dOpacity,
        Render2dPoint, Render2dPrimitive, Render2dRect, Render2dResourceBinding,
        Render2dResourceBindings, Render2dResourceId, Render2dResourceValue, Render2dShape,
        Render2dShapedTextPrimitive, Render2dShapedTextResource,
    };
    use runen_render::execution_2d::{Render2dExecutor, Render2dTarget};
    use std::time::{Duration, Instant};

    /// A genuine multi-node vector/text/vector F2 contribution remains in one
    /// canonical fragment, with independent control-only work before its first
    /// node and after its last. Terminal readback is not a native Present.
    #[test]
    fn source_only_f2_work_is_authored_once_and_proven_on_exact_gpu_submission() {
        prove_f2_with_control_orders(false);
    }

    #[test]
    fn mixed_f2_nodes_obey_independent_control_frontiers_and_exact_completion() {
        prove_f2_with_control_orders(true);
    }

    fn prove_f2_with_control_orders(mixed_vector_text: bool) {
        let descriptor = GpuContextDescriptor::new(
            GpuCapabilityProfile::OffscreenGraphicsBaseline.requirements(),
        )
        .require_format_role(
            GpuTextureFormat::Rgba8UnormSrgb,
            GpuFormatRole::ColorAttachment,
        )
        .require_format_role(GpuTextureFormat::Rgba8UnormSrgb, GpuFormatRole::CopySource)
        .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::ColorAttachment)
        .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::Sampled)
        .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::Filterable)
        .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::CopyDestination)
        .require_format_role(GpuTextureFormat::Rgba8UnormSrgb, GpuFormatRole::Blendable)
        .with_fallback_policy(GpuSoftwareFallbackPolicy::Require)
        .with_allowed_backends([GpuBackendFamily::Vulkan])
        .with_label("Runenwerk U5 source-only mixed F2 proof");
        let context = match pollster::block_on(GpuContext::request(descriptor)) {
            Ok(context) => context,
            Err(error)
                if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable =>
            {
                assert_ne!(
                    std::env::var("RUNEN_RENDER_REQUIRE_GPU").ok().as_deref(),
                    Some("1"),
                    "required Vulkan U5 execution adapter is missing"
                );
                return;
            }
            Err(error) => panic!("unexpected U5 Vulkan adapter failure: {error}"),
        };
        let mut resources = GpuResourceScope::new();
        let texture = resources
            .texture(
                GpuTextureDescriptor::ordinary_owned_2d(
                    "U5 F2 native proof target",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    64,
                    64,
                    GpuTextureFormat::Rgba8UnormSrgb,
                    [
                        GpuTextureUsage::ColorAttachment,
                        GpuTextureUsage::CopySource,
                    ],
                    GpuTextureInitialization::Zeroed,
                )
                .expect("target descriptor"),
            )
            .expect("target identity");
        let view = resources
            .texture_view(
                GpuTextureViewDescriptor::ordinary_full_owned("U5 F2 view", &texture)
                    .expect("target view descriptor"),
            )
            .expect("target view identity");
        let target = Render2dTarget::new(view, 64.0, 64.0, 1.0)
            .expect("exact source target and native raster scale");
        let font = Render2dFontBinding::new(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../assets/fonts/JetBrainsMono-Regular.ttf"
            ))
            .to_vec(),
            0,
            Vec::new(),
            false,
            None,
        )
        .expect("controlled font binding");
        let glyph = Render2dGlyph::new(1, 0.0, 0.0, 24.0).expect("controlled finite glyph");
        let semantic_resource = Render2dShapedTextResource::new(font, 24.0, vec![glyph])
            .expect("immutable shaped text");
        let resource_id = Render2dResourceId::new(1).expect("nonzero semantic id");
        let bindings = Render2dResourceBindings::new(vec![Render2dResourceBinding::new(
            resource_id,
            Render2dResourceValue::ShapedText(semantic_resource),
        )])
        .expect("one source-neutral binding");
        let vector = |x: f64| {
            Render2dEntry::item(Render2dItem::new(
                Render2dPrimitive::Fill {
                    shape: Render2dShape::rect(
                        Render2dRect::new(x, x, 8.0, 8.0).expect("finite rectangle"),
                    ),
                    brush: Render2dBrush::solid(Render2dColorRgba8::WHITE),
                },
                Render2dAffineTransform::IDENTITY,
                Vec::new(),
                Render2dOpacity::OPAQUE,
            ))
        };
        let shaped = Render2dEntry::item(Render2dItem::new(
            Render2dPrimitive::ShapedText(Render2dShapedTextPrimitive::new(
                resource_id,
                Render2dPoint::new(8.0, 32.0).expect("glyph origin"),
                Render2dColorRgba8::WHITE,
            )),
            Render2dAffineTransform::IDENTITY,
            Vec::new(),
            Render2dOpacity::OPAQUE,
        ));
        let entries = if mixed_vector_text {
            vec![vector(0.0), shaped, vector(48.0)]
        } else {
            vec![shaped]
        };
        let composition = Render2dComposition::new(entries).expect("source-neutral F2 composition");
        let contribution = Render2dExecutor::new()
            .prepare(&context, &composition, &bindings, &target)
            .expect("F2 admits the real target and shaped glyph");
        assert!(contribution.has_render_work());

        let control_texture = resources
            .texture(
                GpuTextureDescriptor::ordinary_owned_2d(
                    "U5 independent control surface",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    64,
                    64,
                    GpuTextureFormat::Rgba8UnormSrgb,
                    [GpuTextureUsage::ColorAttachment],
                    GpuTextureInitialization::Zeroed,
                )
                .expect("independent GPU surface descriptor"),
            )
            .expect("independent control texture");
        let control_view = resources
            .texture_view(
                GpuTextureViewDescriptor::ordinary_full_owned(
                    "U5 independent control view",
                    &control_texture,
                )
                .expect("independent GPU surface view descriptor"),
            )
            .expect("independent control view");
        let control_clear = GpuWorkOperation::Render(
            GpuRenderOperation::new(
                [GpuRenderColorAttachment::new(
                    control_view,
                    GpuColorAttachmentLoad::Clear(
                        GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).expect("finite clear"),
                    ),
                    GpuAttachmentStore::Store,
                    None,
                )
                .expect("valid control attachment")],
                None,
                std::iter::empty::<GpuRenderDraw>(),
                None,
            )
            .expect("valid independent control work"),
        );
        let before_occurrence = RenderGpuWorkOccurrenceId::new(1);
        let ui_occurrence = RenderGpuWorkOccurrenceId::new(2);
        let after_occurrence = RenderGpuWorkOccurrenceId::new(3);
        let tail_occurrence = RenderGpuWorkOccurrenceId::new(4);
        let readback = GpuReadbackOperation::ordinary(
            GpuTextureCopyRegion::whole_base_mip(&texture)
                .expect("exact color readback")
                .into(),
        )
        .expect("terminal offscreen observation");
        let nodes = [
            ResolvedRenderGpuWorkNode::pass(
                before_occurrence,
                GpuResourceLabel::new("U5 independent before").unwrap(),
                control_clear.clone(),
                GpuExecutionPreference::GraphicsRequired,
                [],
            ),
            ResolvedRenderGpuWorkNode::empty_builtin_ui_composite(
                ui_occurrence,
                GpuResourceLabel::new("U5 admitted mixed F2 UI").unwrap(),
                [before_occurrence],
            ),
            ResolvedRenderGpuWorkNode::pass(
                after_occurrence,
                GpuResourceLabel::new("U5 independent after").unwrap(),
                control_clear,
                GpuExecutionPreference::GraphicsRequired,
                [ui_occurrence],
            ),
            ResolvedRenderGpuWorkNode::capture_readback(
                tail_occurrence,
                GpuResourceLabel::new("U5 terminal offscreen readback").unwrap(),
                readback,
                [after_occurrence],
            ),
        ];
        let publication_id =
            RunenUiPublicationId::try_from_raw(1).expect("exact producer generation");
        let authored = prepare_render_gpu_frame_work_with_mixed_ui(
            &context,
            GpuResourceLabel::new("one canonical Runenwerk U5 frame").unwrap(),
            nodes,
            RenderGpuExternalWork {
                producer_fragments: &[],
                imports: &[],
            },
            None,
            RunenUiMixedWork {
                ui_occurrence,
                legacy_draws: Vec::new(),
                contributions: vec![(publication_id, 0, contribution)],
            },
            tail_occurrence,
        )
        .expect("RunenGPU admits F2 and readback in one canonical fragment");
        let (observed_id, token) = authored
            .f2_tokens
            .into_iter()
            .next()
            .expect("one exact F2 publication");
        assert_eq!(observed_id, publication_id);
        let token = token.expect("painting F2 work has a single-use node token");
        assert!(
            token.authored_nodes().len() >= if mixed_vector_text { 2 } else { 1 },
            "every executable F2 node must retain its exact authored work identity"
        );
        let graph = &authored.graph;
        let position = |id: &GpuWorkNodeId| -> usize {
            let prepared_id = graph
                .nodes()
                .iter()
                .find(|candidate| candidate.node().id() == id)
                .expect("authored node in final graph")
                .id();
            graph
                .topological_order()
                .iter()
                .position(|candidate| *candidate == prepared_id)
                .expect("node is topologically ordered")
        };
        let before = graph
            .nodes()
            .iter()
            .find(|node| node.node().label().as_str() == "U5 independent before")
            .expect("before work identity")
            .node()
            .id();
        let after = graph
            .nodes()
            .iter()
            .find(|node| node.node().label().as_str() == "U5 independent after")
            .expect("after work identity")
            .node()
            .id();
        for authored_node in token.authored_nodes() {
            assert!(
                position(before) < position(authored_node)
                    && position(authored_node) < position(after),
                "every F2 work node must execute between independent before/after controls"
            );
        }

        let prepared = pollster::block_on(context.prepare_submission(authored.graph))
            .expect("RunenGPU accepts canonical mixed work graph");
        let submission = context
            .submit_prepared(prepared)
            .expect("exact U5 graph submission");
        assert!(submission.contains_work_node(&authored.present_node));
        let deadline = Instant::now() + Duration::from_secs(30);
        while submission.status() == GpuSubmissionStatus::Accepted {
            assert!(
                Instant::now() < deadline,
                "terminal Vulkan work did not complete"
            );
            context.progress();
        }
        assert_eq!(submission.status(), GpuSubmissionStatus::Completed);
        assert_eq!(
            token
                .completed_by(&submission)
                .expect("exact F2 node completed in original submission")
                .submission_id(),
            submission.id(),
        );
    }
}
