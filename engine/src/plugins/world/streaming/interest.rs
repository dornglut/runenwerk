use super::super::adapters::resources::RegionInvalidationJournalResource;
use super::super::chunks::lifecycle::WorldChunkRuntimeMapResource;
use crate::runtime::WorldMut;
use engine_net::{ConnectionId, ServerSessionState};
use spatial::ChunkId;
use std::collections::{BTreeMap, BTreeSet};
use world_ops::SyncCursor;

const MAX_PENDING_CURSOR_MARKERS: usize = 256;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PendingStreamingSnapshot {
    pub region_sequence: u64,
    pub full_resync_payload: bool,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct ConnectionChunkInterest {
    pub relevant_chunks: BTreeSet<ChunkId>,
    pub gameplay_locked_chunks: BTreeSet<ChunkId>,
    pub last_sent_cursor: SyncCursor,
    pub last_ack_cursor: SyncCursor,
    pub needs_full_resync: bool,
    pub acked_region_sequence: u64,
    pub prepared_region_sequence: u64,
    pub prepared_full_resync_payload: bool,
    pub pending_cursor_markers: BTreeMap<SyncCursor, PendingStreamingSnapshot>,
}

impl Default for ConnectionChunkInterest {
    fn default() -> Self {
        Self {
            relevant_chunks: BTreeSet::new(),
            gameplay_locked_chunks: BTreeSet::new(),
            last_sent_cursor: SyncCursor::default(),
            last_ack_cursor: SyncCursor::default(),
            needs_full_resync: true,
            acked_region_sequence: 0,
            prepared_region_sequence: 0,
            prepared_full_resync_payload: true,
            pending_cursor_markers: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldStreamingInterestResource {
    pub per_connection: BTreeMap<ConnectionId, ConnectionChunkInterest>,
}

impl WorldStreamingInterestResource {
    pub fn interest_for_connection_mut(
        &mut self,
        connection_id: ConnectionId,
    ) -> &mut ConnectionChunkInterest {
        self.per_connection.entry(connection_id).or_default()
    }

    pub fn mark_snapshot_sent(
        &mut self,
        connection_id: ConnectionId,
        cursor: SyncCursor,
        sent_full_snapshot: bool,
    ) {
        let interest = self.interest_for_connection_mut(connection_id);
        if cursor.0 >= interest.last_sent_cursor.0 {
            interest.last_sent_cursor = cursor;
        }
        interest.pending_cursor_markers.insert(
            cursor,
            PendingStreamingSnapshot {
                region_sequence: interest.prepared_region_sequence,
                full_resync_payload: interest.prepared_full_resync_payload,
            },
        );
        while interest.pending_cursor_markers.len() > MAX_PENDING_CURSOR_MARKERS {
            if let Some(oldest_cursor) = interest.pending_cursor_markers.keys().next().copied() {
                interest.pending_cursor_markers.remove(&oldest_cursor);
            } else {
                break;
            }
        }
        if sent_full_snapshot {
            interest.needs_full_resync = false;
        }
    }

    pub fn mark_snapshot_acknowledged(
        &mut self,
        connection_id: ConnectionId,
        cursor: SyncCursor,
    ) {
        let interest = self.interest_for_connection_mut(connection_id);
        if cursor.0 < interest.last_ack_cursor.0 {
            return;
        }
        interest.last_ack_cursor = cursor;

        let acknowledged_cursors = interest
            .pending_cursor_markers
            .keys()
            .copied()
            .take_while(|sent_cursor| sent_cursor.0 <= cursor.0)
            .collect::<Vec<_>>();
        for sent_cursor in acknowledged_cursors {
            if let Some(marker) = interest.pending_cursor_markers.remove(&sent_cursor) {
                interest.acked_region_sequence =
                    interest.acked_region_sequence.max(marker.region_sequence);
                if marker.full_resync_payload {
                    interest.needs_full_resync = false;
                }
            }
        }

        if cursor.0 >= interest.last_sent_cursor.0 {
            interest.acked_region_sequence = interest
                .acked_region_sequence
                .max(interest.prepared_region_sequence);
            if interest.prepared_full_resync_payload {
                interest.needs_full_resync = false;
            }
        }
    }

    pub fn mark_needs_full_resync(&mut self, connection_id: ConnectionId) {
        let interest = self.interest_for_connection_mut(connection_id);
        interest.needs_full_resync = true;
    }
}

pub fn sync_world_streaming_interest_system(mut world: WorldMut) {
    let Ok(session) = world.resource::<ServerSessionState>() else {
        return;
    };
    let active_connections = session.active_connections.clone();
    let (runtime_chunks, gameplay_locked_chunks) =
        if let Ok(chunk_runtime) = world.resource::<WorldChunkRuntimeMapResource>() {
            let mut runtime_chunks = BTreeSet::<ChunkId>::new();
            let mut gameplay_locked_chunks = BTreeSet::<ChunkId>::new();
            for record in chunk_runtime.by_chunk_id.values() {
                runtime_chunks.insert(record.chunk_id);
                if record.gameplay_locked {
                    gameplay_locked_chunks.insert(record.chunk_id);
                }
            }
            (runtime_chunks, gameplay_locked_chunks)
        } else {
            (BTreeSet::new(), BTreeSet::new())
        };
    let (journal_min_sequence, journal_max_sequence, journal_records) =
        if let Ok(journal) = world.resource::<RegionInvalidationJournalResource>() {
            let min_sequence = journal.recent_records.front().map(|record| record.sequence);
            let max_sequence = journal.recent_records.back().map(|record| record.sequence);
            let records = journal
                .recent_records
                .iter()
                .map(|record| (record.sequence, record.chunk_ids.clone()))
                .collect::<Vec<_>>();
            (min_sequence, max_sequence.unwrap_or(0), records)
        } else {
            (None, 0, Vec::new())
        };

    let Ok(mut streaming_interest) = world.resource_mut::<WorldStreamingInterestResource>() else {
        return;
    };
    streaming_interest
        .per_connection
        .retain(|connection_id, _| active_connections.contains(connection_id));

    for connection_id in active_connections {
        let interest = streaming_interest.interest_for_connection_mut(connection_id);
        let journal_gap = journal_min_sequence.is_some_and(|min_sequence| {
            interest.acked_region_sequence.saturating_add(1) < min_sequence
        });

        let full_resync_payload =
            interest.needs_full_resync || interest.last_ack_cursor.0 == 0 || journal_gap;

        if journal_gap {
            interest.needs_full_resync = true;
        }

        let mut relevant_chunks = BTreeSet::<ChunkId>::new();
        if full_resync_payload {
            relevant_chunks = runtime_chunks.clone();
        } else {
            for (sequence, chunk_ids) in &journal_records {
                if *sequence <= interest.acked_region_sequence {
                    continue;
                }
                for chunk_id in chunk_ids {
                    if runtime_chunks.contains(chunk_id) {
                        relevant_chunks.insert(*chunk_id);
                    }
                }
            }
        }

        interest.relevant_chunks = relevant_chunks;
        interest.gameplay_locked_chunks = gameplay_locked_chunks.clone();
        interest.prepared_region_sequence = journal_max_sequence;
        interest.prepared_full_resync_payload = full_resync_payload;
    }
}
