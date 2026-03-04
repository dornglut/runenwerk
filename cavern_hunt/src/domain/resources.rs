use crate::domain::loot::EnemyDropTable;
use crate::domain::worldgen::{CavernLayout, CavernRoom, CavernTunnel};
use engine::prelude::{Entity, SimulationTick};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CavernSeed(pub u64);

impl Default for CavernSeed {
    fn default() -> Self {
        Self(0xCA4E_2026_0000_0001)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunConfig {
    pub max_players: u8,
    pub seed: CavernSeed,
    pub room_count_min: u8,
    pub room_count_max: u8,
    pub enemy_density: f32,
    pub elite_count: u8,
    pub extract_countdown_seconds: f32,
    pub base_scrap_reward: u32,
}

impl Default for CavernRunConfig {
    fn default() -> Self {
        Self {
            max_players: 4,
            seed: CavernSeed::default(),
            room_count_min: 7,
            room_count_max: 10,
            enemy_density: 1.0,
            elite_count: 1,
            extract_countdown_seconds: 10.0,
            base_scrap_reward: 10,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CavernRunPhase {
    Exploring,
    Extraction,
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunState {
    pub run_id: u64,
    pub seed: CavernSeed,
    pub phase: CavernRunPhase,
    pub elite_defeated: bool,
    pub extraction_active: bool,
    pub extraction_started_at_tick: Option<SimulationTick>,
    pub party_alive_count: u8,
    pub enemy_kills: u32,
}

impl Default for CavernRunState {
    fn default() -> Self {
        Self {
            run_id: 1,
            seed: CavernSeed::default(),
            phase: CavernRunPhase::Exploring,
            elite_defeated: false,
            extraction_active: false,
            extraction_started_at_tick: None,
            party_alive_count: 1,
            enemy_kills: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SpawnDirector {
    pub initialized: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LootTableRegistry {
    pub swarmer: EnemyDropTable,
    pub bruiser: EnemyDropTable,
    pub spitter: EnemyDropTable,
    pub elite: EnemyDropTable,
}

impl Default for LootTableRegistry {
    fn default() -> Self {
        Self {
            swarmer: EnemyDropTable::swarmer(),
            bruiser: EnemyDropTable::bruiser(),
            spitter: EnemyDropTable::spitter(),
            elite: EnemyDropTable::elite(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernMetaProfile {
    pub cavern_marks: u32,
    pub bonus_max_health: u8,
    pub bonus_dash_efficiency: u8,
    pub unlocked_weapon_mod_slot: bool,
    pub revision: u32,
}

impl Default for CavernMetaProfile {
    fn default() -> Self {
        Self {
            cavern_marks: 0,
            bonus_max_health: 0,
            bonus_dash_efficiency: 0,
            unlocked_weapon_mod_slot: false,
            revision: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LocalPlayerRef {
    pub entity: Option<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CavernCameraState {
    pub target: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub distance_min: f32,
    pub distance_max: f32,
    pub fov_y_radians: f32,
}

impl Default for CavernCameraState {
    fn default() -> Self {
        Self {
            target: [0.0, 0.8, 0.0],
            yaw: std::f32::consts::PI,
            pitch: 0.78,
            distance: 24.0,
            pitch_min: 0.45,
            pitch_max: 1.15,
            distance_min: 10.0,
            distance_max: 42.0,
            fov_y_radians: 52.0_f32.to_radians(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct CavernAimState {
    pub world_point: [f32; 2],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernControlState {
    pub movement: [f32; 2],
    pub aim_world: [f32; 2],
    pub fire_pressed: bool,
    pub dash_pressed: bool,
    pub interact_pressed: bool,
    pub source_tick: SimulationTick,
}

impl Default for CavernControlState {
    fn default() -> Self {
        Self {
            movement: [0.0, 0.0],
            aim_world: [0.0, 0.0],
            fire_pressed: false,
            dash_pressed: false,
            interact_pressed: false,
            source_tick: SimulationTick::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfAgent {
    pub pos: [f32; 2],
    pub radius: f32,
    pub health_ratio: f32,
    pub team: u32,
    pub kind: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CavernSdfWorldFrame {
    pub world_bounds: [f32; 4],
    pub floor_height: f32,
    pub rock_height: f32,
    pub camera: CavernCameraState,
    pub rooms: Vec<CavernRoom>,
    pub tunnels: Vec<CavernTunnel>,
    pub agents: Vec<CavernSdfAgent>,
}

impl Default for CavernSdfWorldFrame {
    fn default() -> Self {
        Self {
            world_bounds: [-24.0, -24.0, 24.0, 24.0],
            floor_height: 0.0,
            rock_height: 8.0,
            camera: CavernCameraState::default(),
            rooms: Vec::new(),
            tunnels: Vec::new(),
            agents: Vec::new(),
        }
    }
}

impl CavernSdfWorldFrame {
    pub fn rebuild_from_layout(&mut self, layout: &CavernLayout, camera: &CavernCameraState) {
        self.world_bounds = layout.world_bounds;
        self.camera = camera.clone();
        self.rooms = layout.rooms.clone();
        self.tunnels = layout.connections.clone();
        self.agents.clear();
    }
}
