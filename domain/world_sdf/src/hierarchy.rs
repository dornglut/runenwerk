use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChunkHierarchyNode {
    pub level: u8,
    pub child_mask: u8,
    pub min_distance: i16,
    pub max_distance: i16,
    pub occupied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChunkHierarchySummary {
    pub nodes: Vec<ChunkHierarchyNode>,
}

impl ChunkHierarchySummary {
    pub fn can_skip_space(&self, level: u8) -> bool {
        self.nodes
            .iter()
            .find(|node| node.level == level)
            .map(|node| !node.occupied && node.max_distance <= 0)
            .unwrap_or(false)
    }
}
