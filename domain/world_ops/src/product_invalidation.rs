use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use spatial::{ChunkId, GridPartitionConfig};

use crate::{
    BuildGeneration, BuildGraph, BuildGraphNode, BuildGraphPhase, BuildQueue, BuildQueueClass,
    BuildQueueItem, ChunkRevision, OperationId, RegionInvalidationJournal, WorldRevision,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductInvalidationSource {
    AssetSourceChanged,
    FieldProductRejected,
    ArtifactReplaced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductInvalidationPlan {
    pub source: ProductInvalidationSource,
    pub world_revision: WorldRevision,
    pub operation_id: Option<OperationId>,
    pub touched_chunks: BTreeSet<ChunkId>,
    pub queue_class: BuildQueueClass,
    pub priority_score: i64,
}

impl ProductInvalidationPlan {
    pub fn new(
        source: ProductInvalidationSource,
        world_revision: WorldRevision,
        touched_chunks: impl IntoIterator<Item = ChunkId>,
    ) -> Self {
        Self {
            source,
            world_revision,
            operation_id: None,
            touched_chunks: touched_chunks.into_iter().collect(),
            queue_class: BuildQueueClass::Background,
            priority_score: 0,
        }
    }

    pub fn interactive(mut self, priority_score: i64) -> Self {
        self.queue_class = BuildQueueClass::Interactive;
        self.priority_score = priority_score;
        self
    }

    pub fn with_operation_id(mut self, operation_id: OperationId) -> Self {
        self.operation_id = Some(operation_id);
        self
    }

    pub fn record_invalidation(
        &self,
        partition: &GridPartitionConfig,
        journal: &mut RegionInvalidationJournal,
    ) {
        if let Some(operation_id) = self.operation_id {
            journal.append_ingress_record(
                partition,
                self.touched_chunks.clone(),
                self.world_revision,
                operation_id,
            );
        } else {
            for chunk_id in &self.touched_chunks {
                journal.append_integration_record(partition, *chunk_id, self.world_revision);
            }
        }
    }

    pub fn enqueue_builds(&self, queue: &mut BuildQueue) {
        for chunk_id in &self.touched_chunks {
            queue.enqueue(BuildQueueItem {
                chunk_id: *chunk_id,
                queue_class: self.queue_class,
                priority_score: self.priority_score,
                starvation_age: 0,
            });
        }
    }

    pub fn build_graph(
        &self,
        target_chunk_revision: ChunkRevision,
        input_generation_stamp: BuildGeneration,
    ) -> BuildGraph {
        BuildGraph {
            nodes: self
                .touched_chunks
                .iter()
                .flat_map(|chunk_id| {
                    [
                        BuildGraphNode {
                            chunk_id: *chunk_id,
                            phase: BuildGraphPhase::DirtyPlan,
                            target_chunk_revision,
                            input_generation_stamp,
                        },
                        BuildGraphNode {
                            chunk_id: *chunk_id,
                            phase: BuildGraphPhase::SdfFieldBuild,
                            target_chunk_revision,
                            input_generation_stamp,
                        },
                        BuildGraphNode {
                            chunk_id: *chunk_id,
                            phase: BuildGraphPhase::Publish,
                            target_chunk_revision,
                            input_generation_stamp,
                        },
                    ]
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spatial::{ChunkCoord3, WorldId};

    #[test]
    fn product_invalidation_enqueues_touched_chunks_once() {
        let chunk = ChunkId::new(WorldId(1), ChunkCoord3 { x: 1, y: 0, z: 0 });
        let plan = ProductInvalidationPlan::new(
            ProductInvalidationSource::AssetSourceChanged,
            WorldRevision(3),
            [chunk],
        )
        .interactive(10);
        let mut queue = BuildQueue::default();

        plan.enqueue_builds(&mut queue);
        plan.enqueue_builds(&mut queue);

        assert!(queue.contains_chunk(chunk));
        assert_eq!(queue.queue_depths(), (1, 0));
    }

    #[test]
    fn changed_source_invalidation_records_regions_and_build_graph() {
        let chunk = ChunkId::new(WorldId(1), ChunkCoord3 { x: 2, y: 0, z: -1 });
        let partition = GridPartitionConfig::default();
        let mut journal = RegionInvalidationJournal {
            max_records: 8,
            ..Default::default()
        };
        let plan = ProductInvalidationPlan::new(
            ProductInvalidationSource::AssetSourceChanged,
            WorldRevision(9),
            [chunk],
        );

        plan.record_invalidation(&partition, &mut journal);
        let graph = plan.build_graph(ChunkRevision(3), BuildGeneration(4));

        assert_eq!(journal.recent_records.len(), 1);
        assert_eq!(graph.nodes.len(), 3);
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.phase == BuildGraphPhase::SdfFieldBuild)
        );
    }
}
