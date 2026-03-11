// src/domain/gameplay/runtime.rs
use std::collections::{BTreeMap, BTreeSet};

use engine::prelude::SimulationTick;
use serde::{Deserialize, Serialize};

use crate::CavernControlState;
use crate::world::{GeometryEditEvent, GeometryPrimitiveId};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernServerControlMap {
    pub by_player_id: BTreeMap<u32, CavernControlState>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernServerAppliedInputTickMap {
    pub by_player_id: BTreeMap<u32, SimulationTick>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernPlayerOwnershipState {
    pub by_connection_id: BTreeMap<u64, u32>,
}

impl CavernPlayerOwnershipState {
    pub fn retain_active_connections<I>(&mut self, active_connections: I)
    where
        I: IntoIterator<Item = u64>,
    {
        let active_connections = active_connections
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        self.by_connection_id
            .retain(|connection_id, _| active_connections.contains(connection_id));
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CavernGeometryRuntimeState {
    pub extraction_seal_primitive: Option<GeometryPrimitiveId>,
    pub edit_events: Vec<GeometryEditEvent>,
}
