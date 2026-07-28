use super::super::{
    GpuResourceAccess, GpuWorkGraphCause, GpuWorkGraphError, GpuWorkGraphErrorContext,
};
use super::{
    authoring::{GpuWorkFragment, GpuWorkNode, accesses_overlap},
    composition::FragmentRelations,
    dependency::GpuDependencyReason,
    diagnostics::{GraphErrorOrigin, graph_error},
    identity::GpuPreparedWorkNodeId,
    initialization::access_region_description,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type DependencyEdges =
    BTreeMap<(GpuPreparedWorkNodeId, GpuPreparedWorkNodeId), BTreeSet<GpuDependencyReason>>;

pub(super) fn infer_fragment_hazards(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    edges: &mut DependencyEdges,
) -> Result<(), GpuWorkGraphError> {
    for (fragment_index, fragment) in fragments.iter().enumerate() {
        for earlier_index in 0..fragment.nodes().len() {
            for later_index in (earlier_index + 1)..fragment.nodes().len() {
                let earlier = &fragment.nodes()[earlier_index];
                let later = &fragment.nodes()[later_index];
                let reasons = hazard_reasons(earlier, later);
                if reasons.is_empty() {
                    continue;
                }
                let before = prepared_node_id(graph_label, fragment_index, fragment, earlier)?;
                let after = prepared_node_id(graph_label, fragment_index, fragment, later)?;
                edges.entry((before, after)).or_default().extend(reasons);
            }
        }
    }
    Ok(())
}

pub(super) fn infer_cross_fragment_hazards(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    relations: &FragmentRelations,
    edges: &mut DependencyEdges,
) -> Result<(), GpuWorkGraphError> {
    for left_fragment_index in 0..fragments.len() {
        for right_fragment_index in (left_fragment_index + 1)..fragments.len() {
            let left_fragment = &fragments[left_fragment_index];
            let right_fragment = &fragments[right_fragment_index];
            for left_node in left_fragment.nodes() {
                for right_node in right_fragment.nodes() {
                    for left_access in left_node.accesses() {
                        for right_access in right_node.accesses() {
                            if left_access.resource_identity() != right_access.resource_identity()
                                || !accesses_overlap(left_access, right_access)
                                || (!left_access.writes() && !right_access.writes())
                            {
                                continue;
                            }
                            let resource = left_access.resource_identity();
                            let left_to_right = relations.contains(&(
                                left_fragment_index,
                                right_fragment_index,
                                resource,
                            ));
                            let right_to_left = relations.contains(&(
                                right_fragment_index,
                                left_fragment_index,
                                resource,
                            ));
                            if left_to_right == right_to_left {
                                let cause = if left_access.writes() && right_access.writes() {
                                    GpuWorkGraphCause::AmbiguousWriter
                                } else {
                                    GpuWorkGraphCause::MissingCrossFragmentCausality
                                };
                                return Err(GpuWorkGraphError::invalid(
                                    "infer cross-fragment GPU hazard",
                                    GpuWorkGraphErrorContext::new(
                                        graph_label,
                                        Some(left_fragment.label().as_str().to_string()),
                                        Some(left_node.label().as_str().to_string()),
                                        None,
                                        Some(resource),
                                        Some(format!(
                                            "{} versus {}",
                                            access_region_description(left_access),
                                            access_region_description(right_access)
                                        )),
                                        Some(left_node.provenance().clone()),
                                    ),
                                    cause,
                                    "bind one unique typed producer output to the consumer import for this storage resource",
                                ));
                            }
                            let (
                                before_fragment,
                                before_node,
                                before_access,
                                after_fragment,
                                after_node,
                                after_access,
                            ) = if left_to_right {
                                (
                                    left_fragment_index,
                                    left_node,
                                    left_access,
                                    right_fragment_index,
                                    right_node,
                                    right_access,
                                )
                            } else {
                                (
                                    right_fragment_index,
                                    right_node,
                                    right_access,
                                    left_fragment_index,
                                    left_node,
                                    left_access,
                                )
                            };
                            let before = prepared_node_id(
                                graph_label,
                                before_fragment,
                                &fragments[before_fragment],
                                before_node,
                            )?;
                            let after = prepared_node_id(
                                graph_label,
                                after_fragment,
                                &fragments[after_fragment],
                                after_node,
                            )?;
                            edges
                                .entry((before, after))
                                .or_default()
                                .extend(access_pair_hazard_reasons(before_access, after_access));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn hazard_reasons(earlier: &GpuWorkNode, later: &GpuWorkNode) -> BTreeSet<GpuDependencyReason> {
    let mut reasons = BTreeSet::new();
    for earlier_access in earlier.accesses() {
        for later_access in later.accesses() {
            if earlier_access.resource_identity() == later_access.resource_identity()
                && accesses_overlap(earlier_access, later_access)
            {
                reasons.extend(access_pair_hazard_reasons(earlier_access, later_access));
            }
        }
    }
    reasons
}

fn access_pair_hazard_reasons(
    earlier: &GpuResourceAccess,
    later: &GpuResourceAccess,
) -> BTreeSet<GpuDependencyReason> {
    let mut reasons = BTreeSet::new();
    let resource = earlier.resource_identity();
    if earlier.writes() && later.reads() {
        reasons.insert(GpuDependencyReason::ReadAfterWrite { resource });
    }
    if earlier.reads() && later.writes() {
        reasons.insert(GpuDependencyReason::WriteAfterRead { resource });
    }
    if earlier.writes() && later.writes() {
        reasons.insert(GpuDependencyReason::WriteAfterWrite { resource });
    }
    reasons
}

fn prepared_node_id(
    graph_label: &str,
    fragment_index: usize,
    fragment: &GpuWorkFragment,
    node: &GpuWorkNode,
) -> Result<GpuPreparedWorkNodeId, GpuWorkGraphError> {
    let ordinal = u32::try_from(fragment_index).map_err(|_| {
        graph_error(
            "assign prepared GPU work-node identity",
            graph_label,
            GraphErrorOrigin::new(Some(fragment), Some(node)),
            None,
            None,
            GpuWorkGraphCause::UnknownIdentity,
            "prepare fewer fragments in one bounded graph",
        )
    })?;
    Ok(GpuPreparedWorkNodeId::new(ordinal, node.id().local))
}

pub(super) fn add_explicit_orders(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    edges: &mut DependencyEdges,
) -> Result<(), GpuWorkGraphError> {
    for (fragment_index, fragment) in fragments.iter().enumerate() {
        for order in fragment.explicit_orders() {
            if !order.before().belongs_to(&fragment.identity)
                || !order.after().belongs_to(&fragment.identity)
            {
                return Err(graph_error(
                    "validate explicit GPU work order",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    None,
                    GpuWorkGraphCause::ForeignIdentity,
                    "use endpoints allocated by the containing fragment",
                ));
            }
            let before_node = fragment
                .nodes()
                .iter()
                .find(|node| node.id() == order.before())
                .ok_or_else(|| {
                    graph_error(
                        "validate explicit GPU work-order endpoint",
                        graph_label,
                        GraphErrorOrigin::new(Some(fragment), None),
                        None,
                        None,
                        GpuWorkGraphCause::UnknownIdentity,
                        "retain both typed endpoints in the immutable fragment",
                    )
                })?;
            let after_node = fragment
                .nodes()
                .iter()
                .find(|node| node.id() == order.after())
                .ok_or_else(|| {
                    graph_error(
                        "validate explicit GPU work-order endpoint",
                        graph_label,
                        GraphErrorOrigin::new(Some(fragment), None),
                        None,
                        None,
                        GpuWorkGraphCause::UnknownIdentity,
                        "retain both typed endpoints in the immutable fragment",
                    )
                })?;
            let before = prepared_node_id(graph_label, fragment_index, fragment, before_node)?;
            let after = prepared_node_id(graph_label, fragment_index, fragment, after_node)?;
            if dependency_data_path_exists(edges, before, after) {
                return Err(graph_error(
                    "add explicit GPU work order",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), Some(after_node)),
                    Some(after),
                    None,
                    GpuWorkGraphCause::RedundantExplicitDataOrder,
                    "remove the explicit edge and rely on typed access-derived dependency",
                ));
            }
            if dependency_data_path_exists(edges, after, before) {
                return Err(graph_error(
                    "add explicit GPU work order",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), Some(after_node)),
                    Some(after),
                    None,
                    GpuWorkGraphCause::ExplicitOrderConflict,
                    "orient the non-data constraint consistently with inferred data order",
                ));
            }
            edges.entry((before, after)).or_default().insert(
                GpuDependencyReason::ExplicitNonData {
                    reason: order.reason().to_string(),
                },
            );
        }
    }
    Ok(())
}

fn dependency_data_path_exists(
    edges: &DependencyEdges,
    start: GpuPreparedWorkNodeId,
    target: GpuPreparedWorkNodeId,
) -> bool {
    let mut ready = vec![(start, false)];
    let mut visited = BTreeSet::new();
    while let Some((node, path_has_data)) = ready.pop() {
        if !visited.insert((node, path_has_data)) {
            continue;
        }
        for (&(before, after), reasons) in edges {
            if before != node {
                continue;
            }
            let edge_has_data = reasons
                .iter()
                .any(|reason| !matches!(reason, GpuDependencyReason::ExplicitNonData { .. }));
            let next_has_data = path_has_data || edge_has_data;
            if after == target && next_has_data {
                return true;
            }
            ready.push((after, next_has_data));
        }
    }
    false
}

pub(super) fn topological_node_order(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    node_locations: &BTreeMap<GpuPreparedWorkNodeId, (usize, usize)>,
    edges: &DependencyEdges,
) -> Result<Vec<GpuPreparedWorkNodeId>, GpuWorkGraphError> {
    let mut outgoing = BTreeMap::<GpuPreparedWorkNodeId, BTreeSet<GpuPreparedWorkNodeId>>::new();
    let mut indegree = node_locations
        .keys()
        .copied()
        .map(|node| (node, 0_usize))
        .collect::<BTreeMap<_, _>>();
    for &(before, after) in edges.keys() {
        if !indegree.contains_key(&before) || !indegree.contains_key(&after) {
            return Err(GpuWorkGraphError::invalid(
                "validate GPU work dependency identity",
                GpuWorkGraphErrorContext::new(
                    graph_label,
                    None,
                    None,
                    Some(after),
                    None,
                    None,
                    None,
                ),
                GpuWorkGraphCause::UnknownIdentity,
                "retain dependency endpoints in the prepared node set",
            ));
        }
        if outgoing.entry(before).or_default().insert(after)
            && let Some(degree) = indegree.get_mut(&after)
        {
            *degree += 1;
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(&node, &degree)| (degree == 0).then_some(node))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(node_locations.len());
    while let Some(node) = ready.pop_first() {
        ordered.push(node);
        if let Some(dependents) = outgoing.get(&node) {
            for &dependent in dependents {
                if let Some(degree) = indegree.get_mut(&dependent) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.insert(dependent);
                    }
                }
            }
        }
    }
    if ordered.len() != node_locations.len() {
        let cycle_node = indegree
            .iter()
            .find_map(|(&node, &degree)| (degree > 0).then_some(node));
        let fragment = cycle_node
            .and_then(|node| node_locations.get(&node))
            .map(|&(fragment, _)| &fragments[fragment]);
        return Err(GpuWorkGraphError::invalid(
            "topologically order GPU work",
            GpuWorkGraphErrorContext::new(
                graph_label,
                fragment.map(|fragment| fragment.label().as_str().to_string()),
                None,
                cycle_node,
                None,
                None,
                fragment.map(|fragment| fragment.provenance().clone()),
            ),
            GpuWorkGraphCause::Cycle,
            "remove cyclic explicit order or cyclic typed resource causality",
        ));
    }
    Ok(ordered)
}
