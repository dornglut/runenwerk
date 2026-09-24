use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::AssetId;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AssetDependencyGraph {
    dependencies_by_asset: BTreeMap<AssetId, BTreeSet<AssetId>>,
    dependents_by_asset: BTreeMap<AssetId, BTreeSet<AssetId>>,
}

impl AssetDependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dependency(&mut self, asset_id: AssetId, depends_on: AssetId) {
        self.dependencies_by_asset
            .entry(asset_id)
            .or_default()
            .insert(depends_on);
        self.dependents_by_asset
            .entry(depends_on)
            .or_default()
            .insert(asset_id);
    }

    pub fn dependencies(&self, asset_id: AssetId) -> impl Iterator<Item = AssetId> + '_ {
        self.dependencies_by_asset
            .get(&asset_id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    pub fn dependents(&self, asset_id: AssetId) -> impl Iterator<Item = AssetId> + '_ {
        self.dependents_by_asset
            .get(&asset_id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    pub fn dependency_edges(&self) -> impl Iterator<Item = (AssetId, AssetId)> + '_ {
        self.dependencies_by_asset
            .iter()
            .flat_map(|(asset_id, dependencies)| {
                dependencies
                    .iter()
                    .copied()
                    .map(|depends_on| (*asset_id, depends_on))
            })
    }

    pub fn invalidation_order_from(&self, changed_asset_id: AssetId) -> Vec<AssetId> {
        let mut visited = BTreeSet::new();
        let mut stack = vec![changed_asset_id];
        let mut ordered = Vec::new();
        while let Some(asset_id) = stack.pop() {
            if !visited.insert(asset_id) {
                continue;
            }
            ordered.push(asset_id);
            for dependent in self.dependents(asset_id) {
                stack.push(dependent);
            }
        }
        ordered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_id;

    #[test]
    fn dependency_graph_reports_reverse_invalidation_order() {
        let mut graph = AssetDependencyGraph::new();
        graph.add_dependency(asset_id(2), asset_id(1));
        graph.add_dependency(asset_id(3), asset_id(2));

        assert_eq!(
            graph.invalidation_order_from(asset_id(1)),
            vec![asset_id(1), asset_id(2), asset_id(3)]
        );
    }
}
