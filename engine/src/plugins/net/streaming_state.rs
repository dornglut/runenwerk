use super::{owner_for_connection, route_connection_targets};
use crate::plugins::world::adapters::resources::RegionInvalidationJournalResource;
use crate::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use crate::runtime::WorldMut;
use ecs::OwnerRole;
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
pub struct ConnectionStreamingState {
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

impl Default for ConnectionStreamingState {
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
pub struct NetStreamingStateResource {
    pub per_connection: BTreeMap<ConnectionId, ConnectionStreamingState>,
}

impl NetStreamingStateResource {
    pub fn state_for_connection_mut(
        &mut self,
        connection_id: ConnectionId,
    ) -> &mut ConnectionStreamingState {
        self.per_connection.entry(connection_id).or_default()
    }

    pub fn mark_snapshot_sent(
        &mut self,
        connection_id: ConnectionId,
        cursor: SyncCursor,
        sent_full_snapshot: bool,
    ) {
        let state = self.state_for_connection_mut(connection_id);
        if cursor.0 >= state.last_sent_cursor.0 {
            state.last_sent_cursor = cursor;
        }
        state.pending_cursor_markers.insert(
            cursor,
            PendingStreamingSnapshot {
                region_sequence: state.prepared_region_sequence,
                full_resync_payload: state.prepared_full_resync_payload,
            },
        );
        while state.pending_cursor_markers.len() > MAX_PENDING_CURSOR_MARKERS {
            if let Some(oldest_cursor) = state.pending_cursor_markers.keys().next().copied() {
                state.pending_cursor_markers.remove(&oldest_cursor);
            } else {
                break;
            }
        }
        if sent_full_snapshot {
            state.needs_full_resync = false;
        }
    }

    pub fn mark_snapshot_acknowledged(&mut self, connection_id: ConnectionId, cursor: SyncCursor) {
        let state = self.state_for_connection_mut(connection_id);
        if cursor.0 < state.last_ack_cursor.0 {
            return;
        }
        state.last_ack_cursor = cursor;

        let acknowledged_cursors = state
            .pending_cursor_markers
            .keys()
            .copied()
            .take_while(|sent_cursor| sent_cursor.0 <= cursor.0)
            .collect::<Vec<_>>();
        for sent_cursor in acknowledged_cursors {
            if let Some(marker) = state.pending_cursor_markers.remove(&sent_cursor) {
                state.acked_region_sequence = state.acked_region_sequence.max(marker.region_sequence);
                if marker.full_resync_payload {
                    state.needs_full_resync = false;
                }
            }
        }

        if cursor.0 >= state.last_sent_cursor.0 {
            state.acked_region_sequence = state
                .acked_region_sequence
                .max(state.prepared_region_sequence);
            if state.prepared_full_resync_payload {
                state.needs_full_resync = false;
            }
        }
    }

    pub fn mark_needs_full_resync(&mut self, connection_id: ConnectionId) {
        let state = self.state_for_connection_mut(connection_id);
        state.needs_full_resync = true;
    }
}

pub fn sync_connection_streaming_state_system(mut world: WorldMut) {
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

    let connection_roles = active_connections
        .iter()
        .copied()
        .map(|connection_id| {
            let role = owner_for_connection(&world, connection_id)
                .and_then(|owner_id| world.owner_role(owner_id));
            (connection_id, role)
        })
        .collect::<BTreeMap<_, _>>();

    let owned_target_counts = active_connections
        .iter()
        .copied()
        .map(|connection_id| {
            let count = route_connection_targets(&world, connection_id).len();
            (connection_id, count)
        })
        .collect::<BTreeMap<_, _>>();

    let Ok(streaming_state) = world.resource_mut::<NetStreamingStateResource>() else {
        return;
    };
    streaming_state
        .per_connection
        .retain(|connection_id, _| active_connections.contains(connection_id));

    for connection_id in active_connections {
        let state = streaming_state.state_for_connection_mut(connection_id);
        let role = connection_roles.get(&connection_id).copied().flatten();
        let owned_target_count = owned_target_counts
            .get(&connection_id)
            .copied()
            .unwrap_or(0);

        if matches!(role, Some(OwnerRole::Observer))
            || (matches!(role, Some(OwnerRole::Active)) && owned_target_count == 0)
        {
            state.relevant_chunks.clear();
            state.gameplay_locked_chunks.clear();
            state.prepared_region_sequence = journal_max_sequence;
            state.prepared_full_resync_payload = false;
            continue;
        }

        let journal_gap = journal_min_sequence
            .is_some_and(|min_sequence| state.acked_region_sequence.saturating_add(1) < min_sequence);

        let full_resync_payload = state.needs_full_resync || state.last_ack_cursor.0 == 0 || journal_gap;

        if journal_gap {
            state.needs_full_resync = true;
        }

        let mut relevant_chunks = BTreeSet::<ChunkId>::new();
        if full_resync_payload {
            relevant_chunks = runtime_chunks.clone();
        } else {
            for (sequence, chunk_ids) in &journal_records {
                if *sequence <= state.acked_region_sequence {
                    continue;
                }
                for chunk_id in chunk_ids {
                    if runtime_chunks.contains(chunk_id) {
                        relevant_chunks.insert(*chunk_id);
                    }
                }
            }
        }

        state.relevant_chunks = relevant_chunks;
        state.gameplay_locked_chunks = gameplay_locked_chunks.clone();
        state.prepared_region_sequence = journal_max_sequence;
        state.prepared_full_resync_payload = full_resync_payload;
    }
}
