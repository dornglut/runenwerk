use super::super::chunks::partition::WorldPartitionConfig;
use super::super::ids::{ChunkId, RegionId, WorldOpId, WorldRevision};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldRegionInvalidationSource {
    EditIngress,
    BuildIntegration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldRegionInvalidationRecord {
    pub sequence: u64,
    pub source: WorldRegionInvalidationSource,
    pub world_revision: WorldRevision,
    pub op_id: Option<WorldOpId>,
    pub chunk_ids: BTreeSet<ChunkId>,
    pub region_ids: BTreeSet<RegionId>,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct WorldRegionInvalidationJournalResource {
    pub next_sequence: u64,
    pub max_records: usize,
    pub recent_records: VecDeque<WorldRegionInvalidationRecord>,
}

impl Default for WorldRegionInvalidationJournalResource {
    fn default() -> Self {
        Self {
            next_sequence: 1,
            max_records: 128,
            recent_records: VecDeque::new(),
        }
    }
}

impl WorldRegionInvalidationJournalResource {
    pub fn append_ingress_record(
        &mut self,
        partition: &WorldPartitionConfig,
        touched_chunks: BTreeSet<ChunkId>,
        world_revision: WorldRevision,
        op_id: WorldOpId,
    ) {
        if touched_chunks.is_empty() {
            return;
        }
        let region_ids = touched_chunks
            .iter()
            .copied()
            .map(|chunk_id| partition.region_id_from_chunk_id(chunk_id))
            .collect::<BTreeSet<_>>();
        let sequence = self.next_sequence();
        self.push_record(WorldRegionInvalidationRecord {
            sequence,
            source: WorldRegionInvalidationSource::EditIngress,
            world_revision,
            op_id: Some(op_id),
            chunk_ids: touched_chunks,
            region_ids,
        });
    }

    pub fn append_integration_record(
        &mut self,
        partition: &WorldPartitionConfig,
        chunk_id: ChunkId,
        world_revision: WorldRevision,
    ) {
        let mut chunk_ids = BTreeSet::new();
        chunk_ids.insert(chunk_id);
        let mut region_ids = BTreeSet::new();
        region_ids.insert(partition.region_id_from_chunk_id(chunk_id));
        let sequence = self.next_sequence();
        self.push_record(WorldRegionInvalidationRecord {
            sequence,
            source: WorldRegionInvalidationSource::BuildIntegration,
            world_revision,
            op_id: None,
            chunk_ids,
            region_ids,
        });
    }

    fn next_sequence(&mut self) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        sequence
    }

    fn push_record(&mut self, record: WorldRegionInvalidationRecord) {
        self.recent_records.push_back(record);
        while self.recent_records.len() > self.max_records.max(1) {
            self.recent_records.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::world::chunks::partition::WorldPartitionConfig;
    use crate::plugins::world::ids::{ChunkCoord3, ChunkId, PlanetId};

    #[test]
    fn append_ingress_records_dedupe_regions_for_touched_chunks() {
        let mut journal = WorldRegionInvalidationJournalResource::default();
        let partition = WorldPartitionConfig::default();
        let planet = PlanetId(0);
        let mut touched_chunks = BTreeSet::new();
        touched_chunks.insert(ChunkId::new(planet, ChunkCoord3 { x: 0, y: 0, z: 0 }));
        touched_chunks.insert(ChunkId::new(planet, ChunkCoord3 { x: 1, y: 0, z: 0 }));

        journal.append_ingress_record(
            &partition,
            touched_chunks.clone(),
            WorldRevision(3),
            WorldOpId(7),
        );

        let record = journal
            .recent_records
            .front()
            .expect("journal should contain one record");
        assert_eq!(record.chunk_ids, touched_chunks);
        assert_eq!(
            record.region_ids.len(),
            1,
            "neighboring chunks should collapse into one region id for default partition dims"
        );
    }

    #[test]
    fn journal_capacity_evicts_oldest_records() {
        let mut journal = WorldRegionInvalidationJournalResource {
            max_records: 2,
            ..Default::default()
        };
        let partition = WorldPartitionConfig::default();
        let chunk = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });

        journal.append_integration_record(&partition, chunk, WorldRevision(1));
        journal.append_integration_record(&partition, chunk, WorldRevision(2));
        journal.append_integration_record(&partition, chunk, WorldRevision(3));

        let sequences = journal
            .recent_records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>();
        assert_eq!(
            sequences,
            vec![2, 3],
            "journal should retain only newest records within configured capacity"
        );
    }
}
