use crate::{OperationId, WorldRevision};
use serde::{Deserialize, Serialize};
use spatial::{ChunkId, GridPartitionConfig, RegionId};
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionInvalidationSource {
    EditIngress,
    BuildIntegration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionInvalidationRecord {
    pub sequence: u64,
    pub source: RegionInvalidationSource,
    pub world_revision: WorldRevision,
    pub op_id: Option<OperationId>,
    pub chunk_ids: BTreeSet<ChunkId>,
    pub region_ids: BTreeSet<RegionId>,
}

#[derive(Debug, Clone)]
pub struct RegionInvalidationJournal {
    pub next_sequence: u64,
    pub max_records: usize,
    pub recent_records: VecDeque<RegionInvalidationRecord>,
}

impl Default for RegionInvalidationJournal {
    fn default() -> Self {
        Self {
            next_sequence: 1,
            max_records: 128,
            recent_records: VecDeque::new(),
        }
    }
}

impl RegionInvalidationJournal {
    pub fn append_ingress_record(
	    &mut self,
	    partition: &GridPartitionConfig,
	    touched_chunks: BTreeSet<ChunkId>,
	    world_revision: WorldRevision,
	    op_id: OperationId,
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
        self.push_record(RegionInvalidationRecord {
            sequence,
            source: RegionInvalidationSource::EditIngress,
            world_revision,
            op_id: Some(op_id),
            chunk_ids: touched_chunks,
            region_ids,
        });
    }

    pub fn append_integration_record(
	    &mut self,
	    partition: &GridPartitionConfig,
	    chunk_id: ChunkId,
	    world_revision: WorldRevision,
    ) {
        let mut chunk_ids = BTreeSet::new();
        chunk_ids.insert(chunk_id);
        let mut region_ids = BTreeSet::new();
        region_ids.insert(partition.region_id_from_chunk_id(chunk_id));
        let sequence = self.next_sequence();
        self.push_record(RegionInvalidationRecord {
            sequence,
            source: RegionInvalidationSource::BuildIntegration,
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

    fn push_record(&mut self, record: RegionInvalidationRecord) {
        self.recent_records.push_back(record);
        while self.recent_records.len() > self.max_records.max(1) {
            self.recent_records.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RegionInvalidationJournal;
    use crate::{OperationId, WorldRevision};
    use spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
    use std::collections::BTreeSet;

    #[test]
    fn append_ingress_records_dedupe_regions_for_touched_chunks() {
        let mut journal = RegionInvalidationJournal::default();
        let partition = GridPartitionConfig::default();
        let planet = WorldId(0);
        let mut touched_chunks = BTreeSet::new();
        touched_chunks.insert(ChunkId::new(planet, ChunkCoord3 { x: 0, y: 0, z: 0 }));
        touched_chunks.insert(ChunkId::new(planet, ChunkCoord3 { x: 1, y: 0, z: 0 }));

        journal.append_ingress_record(
            &partition,
            touched_chunks.clone(),
            WorldRevision(3),
            OperationId(7),
        );

        let record = journal
            .recent_records
            .front()
            .expect("journal should contain one record");
        assert_eq!(record.chunk_ids, touched_chunks);
        assert_eq!(record.region_ids.len(), 1);
    }

    #[test]
    fn journal_capacity_evicts_oldest_records() {
        let mut journal = RegionInvalidationJournal {
            max_records: 2,
            ..Default::default()
        };
        let partition = GridPartitionConfig::default();
        let chunk = ChunkId::new(WorldId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });

        journal.append_integration_record(&partition, chunk, WorldRevision(1));
        journal.append_integration_record(&partition, chunk, WorldRevision(2));
        journal.append_integration_record(&partition, chunk, WorldRevision(3));

        let sequences = journal
            .recent_records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>();
        assert_eq!(sequences, vec![2, 3]);
    }
}
