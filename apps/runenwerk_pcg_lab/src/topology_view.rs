//! Read-only, reproducible display coordinates for one qualified cave topology.
//! These points have no World, wall, metric-distance or collision meaning.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use procgen::{CaveChamberId, CavePassageId, ProcgenCaveTopologyCandidate};

const VIEW_MARGIN: i32 = 96;
const LAYER_STEP: i32 = 176;
const CHAMBER_STEP: i32 = 112;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChamberView {
    pub id: CaveChamberId,
    pub center: ViewPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassageView {
    pub id: CavePassageId,
    pub from: CaveChamberId,
    pub to: CaveChamberId,
    pub start: ViewPoint,
    pub end: ViewPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveSelection {
    Chamber(CaveChamberId),
    Passage(CavePassageId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveViewError {
    DuplicateChamber,
    UnknownEndpoint,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveTopologyView {
    pub chambers: Vec<ChamberView>,
    pub passages: Vec<PassageView>,
}

impl CaveTopologyView {
    #[must_use]
    pub fn contains(&self, selection: CaveSelection) -> bool {
        match selection {
            CaveSelection::Chamber(id) => self.chambers.iter().any(|node| node.id == id),
            CaveSelection::Passage(id) => self.passages.iter().any(|edge| edge.id == id),
        }
    }
}

/// Deterministic BFS-layer diagram, with ties sorted by stable chamber ID.
/// Nothing in this projection determines actual cave-room placement or widths.
pub fn project_cave_topology(
    candidate: &ProcgenCaveTopologyCandidate,
) -> Result<CaveTopologyView, CaveViewError> {
    let mut adjacent = BTreeMap::<CaveChamberId, BTreeSet<CaveChamberId>>::new();
    for chamber in candidate.chambers() {
        if adjacent.insert(chamber.id, BTreeSet::new()).is_some() {
            return Err(CaveViewError::DuplicateChamber);
        }
    }
    for passage in candidate.passages() {
        if !adjacent.contains_key(&passage.from) || !adjacent.contains_key(&passage.to) {
            return Err(CaveViewError::UnknownEndpoint);
        }
        // The C1A domain ratifier already owns connectivity and endpoint
        // validity. Rebuild only the read-only adjacency required for layout.
        adjacent
            .get_mut(&passage.from)
            .ok_or(CaveViewError::UnknownEndpoint)?
            .insert(passage.to);
        adjacent
            .get_mut(&passage.to)
            .ok_or(CaveViewError::UnknownEndpoint)?
            .insert(passage.from);
    }

    let mut depth = BTreeMap::<CaveChamberId, usize>::new();
    let mut queue = VecDeque::new();
    if let Some(&first) = adjacent.keys().next() {
        depth.insert(first, 0);
        queue.push_back(first);
    }
    while let Some(node) = queue.pop_front() {
        let layer = depth[&node];
        for &next in &adjacent[&node] {
            if let std::collections::btree_map::Entry::Vacant(entry) = depth.entry(next) {
                entry.insert(layer + 1);
                queue.push_back(next);
            }
        }
    }
    if depth.len() != adjacent.len() {
        return Err(CaveViewError::Disconnected);
    }

    let mut ranks = BTreeMap::<usize, i32>::new();
    let mut positions = BTreeMap::new();
    for (id, layer) in depth {
        let row = ranks.entry(layer).or_default();
        let position = ViewPoint {
            x: VIEW_MARGIN + i32::try_from(layer).expect("C1A bounds chambers") * LAYER_STEP,
            y: VIEW_MARGIN + *row * CHAMBER_STEP,
        };
        *row += 1;
        positions.insert(id, position);
    }

    let chambers = positions
        .iter()
        .map(|(&id, &center)| ChamberView { id, center })
        .collect();
    let passages = candidate
        .passages()
        .iter()
        .map(|passage| {
            Ok(PassageView {
                id: passage.id,
                from: passage.from,
                to: passage.to,
                start: *positions
                    .get(&passage.from)
                    .ok_or(CaveViewError::UnknownEndpoint)?,
                end: *positions
                    .get(&passage.to)
                    .ok_or(CaveViewError::UnknownEndpoint)?,
            })
        })
        .collect::<Result<Vec<_>, CaveViewError>>()?;
    Ok(CaveTopologyView { chambers, passages })
}
