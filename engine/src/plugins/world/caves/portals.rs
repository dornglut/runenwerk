use super::sectors::CaveSectorId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct CavePortalEdge {
    pub a: CaveSectorId,
    pub b: CaveSectorId,
    pub bidirectional: bool,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldCavePortalGraphResource {
    pub portals: Vec<CavePortalEdge>,
}

impl WorldCavePortalGraphResource {
    pub fn neighbors(&self, sector_id: CaveSectorId) -> Vec<CaveSectorId> {
        self.portals
            .iter()
            .filter_map(|edge| {
                if edge.a == sector_id {
                    Some(edge.b)
                } else if edge.bidirectional && edge.b == sector_id {
                    Some(edge.a)
                } else {
                    None
                }
            })
            .collect()
    }
}
