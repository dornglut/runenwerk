use crate::domain::loot::EnemyDropTable;
use crate::domain::world::geometry_graph::{GeometryEditEvent, GeometryPrimitiveId};
use crate::domain::world::worldgen::{RoomId, RoomRole};
use engine::prelude::{Entity, SimulationTick};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// src/domain/resources.rs

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ecs::Resource)]
pub struct CavernSeed(pub u64);

impl Default for CavernSeed {
    fn default() -> Self {
        Self(0xCA4E_2026_0000_0001)
    }
}
