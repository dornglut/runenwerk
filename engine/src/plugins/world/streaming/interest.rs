use super::super::ids::{ChunkId, ChunkSyncCursor};
use engine_net::ConnectionId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct ConnectionChunkInterest {
    pub relevant_chunks: BTreeSet<ChunkId>,
    pub gameplay_locked_chunks: BTreeSet<ChunkId>,
    pub last_sent_cursor: ChunkSyncCursor,
    pub needs_full_resync: bool,
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
}
