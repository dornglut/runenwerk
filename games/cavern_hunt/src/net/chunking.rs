use anyhow::{Result, ensure};
use engine::prelude::SimulationTick;
use engine_net::RunEvent;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::protocol::CavernRunEventCodeV2;

pub(super) const RUN_EVENT_CHUNK_V2: &str = CavernRunEventCodeV2::Chunk.as_str();
const MAX_RUN_EVENT_PAYLOAD_BYTES: usize = 1_000;
const RUN_EVENT_CHUNK_PAYLOAD_BYTES: usize = 700;
const MAX_IN_FLIGHT_CHUNK_GROUPS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct CavernRunEventChunkV2 {
    pub(super) group_id: u64,
    pub(super) event_code: String,
    pub(super) chunk_index: u16,
    pub(super) chunk_count: u16,
    pub(super) payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ChunkKey {
    group_id: u64,
    event_code: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ChunkAssembly {
    chunk_count: u16,
    received: u16,
    chunks: Vec<Option<Vec<u8>>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ClientRunEventChunkStateV2 {
    assemblies: BTreeMap<ChunkKey, ChunkAssembly>,
}

pub(super) fn chunk_group_id_for_event(
    tick: SimulationTick,
    stream_cursor: u64,
    event_code: &str,
) -> u64 {
    let base = if stream_cursor > 0 { stream_cursor } else { tick.0 };
    (base << 4)
        | CavernRunEventCodeV2::parse(event_code)
            .map(CavernRunEventCodeV2::event_type_tag)
            .unwrap_or(15)
}

pub(super) fn split_run_event_for_mtu(
    event_code: &str,
    payload: Vec<u8>,
    chunk_group_id: u64,
) -> Result<Vec<RunEvent>> {
    if payload.len() <= MAX_RUN_EVENT_PAYLOAD_BYTES {
        return Ok(vec![RunEvent {
            code: event_code.to_string(),
            payload,
        }]);
    }

    let chunk_count = payload.len().div_ceil(RUN_EVENT_CHUNK_PAYLOAD_BYTES);
    ensure!(
        chunk_count <= u16::MAX as usize,
        "run event payload too large to chunk safely: code={} size={} chunks={}",
        event_code,
        payload.len(),
        chunk_count
    );

    let mut events = Vec::with_capacity(chunk_count);
    for (chunk_index, chunk_payload_bytes) in payload.chunks(RUN_EVENT_CHUNK_PAYLOAD_BYTES).enumerate()
    {
        let chunk = CavernRunEventChunkV2 {
            group_id: chunk_group_id,
            event_code: event_code.to_string(),
            chunk_index: chunk_index as u16,
            chunk_count: chunk_count as u16,
            payload: chunk_payload_bytes.to_vec(),
        };
        events.push(RunEvent {
            code: RUN_EVENT_CHUNK_V2.to_string(),
            payload: postcard::to_allocvec(&chunk)?,
        });
    }
    Ok(events)
}

impl ClientRunEventChunkStateV2 {
    pub(super) fn consume_chunk(&mut self, chunk: CavernRunEventChunkV2) -> Option<RunEvent> {
        if chunk.chunk_count == 0 || chunk.chunk_index >= chunk.chunk_count {
            return None;
        }

        let key = ChunkKey {
            group_id: chunk.group_id,
            event_code: chunk.event_code.clone(),
        };
        let chunk_count = chunk.chunk_count;
        let chunk_index = chunk.chunk_index as usize;
        let payload = chunk.payload;

        let assembly = self
            .assemblies
            .entry(key.clone())
            .or_insert_with(|| ChunkAssembly {
                chunk_count,
                received: 0,
                chunks: vec![None; chunk_count as usize],
            });
        if assembly.chunk_count != chunk_count {
            *assembly = ChunkAssembly {
                chunk_count,
                received: 0,
                chunks: vec![None; chunk_count as usize],
            };
        }
        if assembly.chunks[chunk_index].is_none() {
            assembly.chunks[chunk_index] = Some(payload);
            assembly.received = assembly.received.saturating_add(1);
        }

        if assembly.received != assembly.chunk_count {
            trim_assemblies(&mut self.assemblies);
            return None;
        }

        let mut reconstructed = Vec::new();
        for chunk_payload in &assembly.chunks {
            let bytes = chunk_payload.as_ref()?;
            reconstructed.extend_from_slice(bytes);
        }
        self.assemblies.remove(&key);
        Some(RunEvent {
            code: key.event_code,
            payload: reconstructed,
        })
    }
}

fn trim_assemblies(assemblies: &mut BTreeMap<ChunkKey, ChunkAssembly>) {
    while assemblies.len() > MAX_IN_FLIGHT_CHUNK_GROUPS {
        let Some(first_key) = assemblies.keys().next().cloned() else {
            break;
        };
        assemblies.remove(&first_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_large_payload_into_chunk_events() {
        let payload = vec![42_u8; 3_000];
        let events =
            split_run_event_for_mtu("cavern_hunt.keyframe.v2", payload.clone(), 11).unwrap();
        assert!(events.len() > 1);
        assert!(events.iter().all(|event| event.code == RUN_EVENT_CHUNK_V2));
    }

    #[test]
    fn chunk_state_reassembles_payload_once_complete() {
        let payload = vec![7_u8; 2_900];
        let chunk_events = split_run_event_for_mtu("cavern_hunt.patch.v2", payload.clone(), 3).unwrap();
        let mut state = ClientRunEventChunkStateV2::default();
        let mut rebuilt = None;
        for event in chunk_events {
            let chunk: CavernRunEventChunkV2 = postcard::from_bytes(&event.payload).unwrap();
            if let Some(run_event) = state.consume_chunk(chunk) {
                rebuilt = Some(run_event);
            }
        }
        let rebuilt = rebuilt.expect("all chunks should reconstruct a run event");
        assert_eq!(rebuilt.code, "cavern_hunt.patch.v2");
        assert_eq!(rebuilt.payload, payload);
    }
}
