use crate::plugins::render::RenderResourceId;
use crate::plugins::render::graph::RenderFlowGraph;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransientResourceWindow {
    pub resource_id: RenderResourceId,
    pub first_pass_index: usize,
    pub last_pass_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransientAliasCandidate {
    pub left: RenderResourceId,
    pub right: RenderResourceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransientAliasAssignment {
    pub resource_id: RenderResourceId,
    pub slot_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransientAliasSlot {
    pub slot_index: usize,
    pub resources: Vec<RenderResourceId>,
}

pub fn build_transient_windows(graph: &RenderFlowGraph) -> Vec<TransientResourceWindow> {
    let mut pass_usages = BTreeMap::<RenderResourceId, (usize, usize)>::new();
    let transient_ids = graph
        .resources
        .resources
        .iter()
        .filter(|resource| resource.lifetime().is_transient())
        .map(|resource| *resource.id())
        .collect::<BTreeSet<_>>();

    for (pass_index, pass) in graph.passes.passes.iter().enumerate() {
        for id in pass_resource_ids(pass) {
            if !transient_ids.contains(id) {
                continue;
            }
            let entry = pass_usages.entry(*id).or_insert((pass_index, pass_index));
            entry.0 = entry.0.min(pass_index);
            entry.1 = entry.1.max(pass_index);
        }
    }

    transient_ids
        .into_iter()
        .filter_map(|id| {
            pass_usages
                .get(&id)
                .map(|(first, last)| TransientResourceWindow {
                    resource_id: id,
                    first_pass_index: *first,
                    last_pass_index: *last,
                })
        })
        .collect()
}

pub fn find_aliasable_transients(
    windows: &[TransientResourceWindow],
) -> Vec<TransientAliasCandidate> {
    let mut candidates = Vec::<TransientAliasCandidate>::new();
    for i in 0..windows.len() {
        for j in (i + 1)..windows.len() {
            let left = &windows[i];
            let right = &windows[j];
            let disjoint = left.last_pass_index < right.first_pass_index
                || right.last_pass_index < left.first_pass_index;
            if disjoint {
                candidates.push(TransientAliasCandidate {
                    left: left.resource_id.clone(),
                    right: right.resource_id.clone(),
                });
            }
        }
    }
    candidates
}

pub fn build_transient_alias_assignments(
    windows: &[TransientResourceWindow],
) -> Vec<TransientAliasAssignment> {
    let mut ordered_windows = windows.to_vec();
    ordered_windows.sort_by_key(|window| (window.first_pass_index, window.last_pass_index));

    let mut slot_last_use = Vec::<usize>::new();
    let mut assignments = Vec::<TransientAliasAssignment>::new();

    for window in &ordered_windows {
        let mut selected_slot = None::<usize>;
        for (slot_index, last_use) in slot_last_use.iter().enumerate() {
            if *last_use < window.first_pass_index {
                selected_slot = Some(slot_index);
                break;
            }
        }

        let slot_index = match selected_slot {
            Some(index) => {
                slot_last_use[index] = window.last_pass_index;
                index
            }
            None => {
                slot_last_use.push(window.last_pass_index);
                slot_last_use.len() - 1
            }
        };

        assignments.push(TransientAliasAssignment {
            resource_id: window.resource_id.clone(),
            slot_index,
        });
    }

    assignments
}

pub fn build_transient_alias_slots(
    assignments: &[TransientAliasAssignment],
) -> Vec<TransientAliasSlot> {
    let mut slots = BTreeMap::<usize, Vec<RenderResourceId>>::new();
    for assignment in assignments {
        slots
            .entry(assignment.slot_index)
            .or_default()
            .push(assignment.resource_id.clone());
    }

    slots
        .into_iter()
        .map(|(slot_index, resources)| TransientAliasSlot {
            slot_index,
            resources,
        })
        .collect()
}

fn pass_resource_ids(
    pass: &crate::plugins::render::RenderPassNode,
) -> impl Iterator<Item = &crate::plugins::render::RenderResourceId> {
    pass.reads
        .iter()
        .chain(pass.writes.iter())
        .chain(pass.sampled_textures.iter())
        .chain(pass.write_textures.iter())
        .chain(pass.vertex_buffers.iter())
        .chain(pass.index_buffers.iter())
        .chain(pass.instance_buffers.iter())
        .chain(pass.indirect_buffers.iter())
        .chain(pass.depth_target.iter())
}
