use crate::domain::geometry_graph::{GeometryEditEvent, GeometryPrimitiveId};
use crate::domain::loot::EnemyDropTable;
use crate::domain::worldgen::{RoomId, RoomRole};
use engine::prelude::{Entity, SimulationTick};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CavernSessionSettings {
    pub seed: Option<CavernSeed>,
    pub enemy_density: Option<f32>,
    pub extract_countdown_seconds: Option<f32>,
    pub base_scrap_reward: Option<u32>,
    pub spawn_radius: Option<f32>,
    pub companion_spacing: Option<f32>,
    pub enemy_health_scale: Option<f32>,
    pub enemy_damage_scale: Option<f32>,
    pub elite_health_bonus: Option<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CavernRunPhase {
    Exploring,
    EliteAvailable,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanionBehaviorRole {
    Skirmisher,
    SupportShooter,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSpawnProfile {
    pub is_human: bool,
    pub role: Option<CompanionBehaviorRole>,
    pub spawn_radius: f32,
    pub weapon_cooldown_scale: f32,
    pub projectile_speed_scale: f32,
    pub bonus_health: f32,
}

impl Default for PlayerSpawnProfile {
    fn default() -> Self {
        Self {
            is_human: true,
            role: None,
            spawn_radius: 1.1,
            weapon_cooldown_scale: 1.0,
            projectile_speed_scale: 1.0,
            bonus_health: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunDifficultyProfile {
    pub enemy_health_scale: f32,
    pub enemy_damage_scale: f32,
    pub elite_health_bonus: f32,
}

impl Default for RunDifficultyProfile {
    fn default() -> Self {
        Self {
            enemy_health_scale: 1.0,
            enemy_damage_scale: 1.0,
            elite_health_bonus: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSpawnPolicy {
    pub desired_human_players: u8,
    pub desired_total_participants: u8,
    pub companion_target_count: u8,
    pub spawn_radius: f32,
    pub companion_spacing: f32,
    pub roster_display_names: BTreeMap<u8, String>,
    pub difficulty: RunDifficultyProfile,
}

impl Default for SessionSpawnPolicy {
    fn default() -> Self {
        Self {
            desired_human_players: 1,
            desired_total_participants: 1,
            companion_target_count: 0,
            spawn_radius: 1.1,
            companion_spacing: 1.25,
            roster_display_names: BTreeMap::new(),
            difficulty: RunDifficultyProfile::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomEncounterState {
    Dormant,
    Active,
    Cleared,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomEncounterStatus {
    pub room_id: RoomId,
    pub role: RoomRole,
    pub state: RoomEncounterState,
    pub has_reward: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoomEncounterRegistry {
    pub by_room_id: BTreeMap<RoomId, RoomEncounterStatus>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CavernObjectiveKind {
    Explore,
    HuntElite,
    ReachExtraction,
    ExtractionCountdown,
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernObjectiveState {
    pub kind: CavernObjectiveKind,
    pub title: String,
    pub detail: String,
    pub elite_room: Option<RoomId>,
    pub extraction_room: Option<RoomId>,
}

impl Default for CavernObjectiveState {
    fn default() -> Self {
        Self {
            kind: CavernObjectiveKind::Explore,
            title: "Explore the caverns".to_string(),
            detail: "Find the Nest Guardian".to_string(),
            elite_room: None,
            extraction_room: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionState {
    pub active: bool,
    pub room_id: Option<RoomId>,
    pub countdown_started_at_tick: Option<SimulationTick>,
    pub countdown_remaining_seconds: f32,
    pub occupied_by_alive_player: bool,
}

impl Default for ExtractionState {
    fn default() -> Self {
        Self {
            active: false,
            room_id: None,
            countdown_started_at_tick: None,
            countdown_remaining_seconds: 0.0,
            occupied_by_alive_player: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerStatusPanel {
    pub player_id: u32,
    pub label: String,
    pub alive: bool,
    pub is_companion: bool,
    pub scrap: u32,
    pub health_ratio: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunObjectivePanel {
    pub title: String,
    pub detail: String,
}

impl Default for RunObjectivePanel {
    fn default() -> Self {
        Self {
            title: "Explore the caverns".to_string(),
            detail: "Find the Nest Guardian".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionCountdownPanel {
    pub visible: bool,
    pub seconds_remaining: f32,
}

impl Default for ExtractionCountdownPanel {
    fn default() -> Self {
        Self {
            visible: false,
            seconds_remaining: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernHudState {
    pub visible: bool,
    pub local_health: f32,
    pub local_max_health: f32,
    pub dash_cooldown_remaining: f32,
    pub scrap: u32,
    pub elite_defeated: bool,
    pub extraction_active: bool,
    pub objective: RunObjectivePanel,
    pub extraction: ExtractionCountdownPanel,
    pub teammates: Vec<PlayerStatusPanel>,
    pub status_lines: Vec<String>,
}

impl Default for CavernHudState {
    fn default() -> Self {
        Self {
            visible: true,
            local_health: 0.0,
            local_max_health: 0.0,
            dash_cooldown_remaining: 0.0,
            scrap: 0,
            elite_defeated: false,
            extraction_active: false,
            objective: RunObjectivePanel::default(),
            extraction: ExtractionCountdownPanel::default(),
            teammates: Vec::new(),
            status_lines: Vec::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PlayerCombatTuning {
    pub move_speed: f32,
    pub dash_invulnerability_seconds: f32,
    pub primary_fire_interval_seconds: f32,
    pub projectile_speed: f32,
}

impl Default for PlayerCombatTuning {
    fn default() -> Self {
        Self {
            move_speed: 5.5,
            dash_invulnerability_seconds: 0.15,
            primary_fire_interval_seconds: 0.22,
            projectile_speed: 15.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EnemyCombatTuning {
    pub swarmer_speed: f32,
    pub bruiser_speed: f32,
    pub spitter_speed: f32,
    pub elite_speed: f32,
}

impl Default for EnemyCombatTuning {
    fn default() -> Self {
        Self {
            swarmer_speed: 3.4,
            bruiser_speed: 2.1,
            spitter_speed: 1.6,
            elite_speed: 2.5,
        }
    }
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CavernMetaPersistenceConfig {
    pub enabled: bool,
}

impl Default for CavernMetaPersistenceConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct CavernMetaRewardState {
    pub last_awarded_run_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LocalPlayerRef {
    pub player_id: Option<u32>,
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
            target: [0.0, 1.9, 0.0],
            yaw: std::f32::consts::PI,
            pitch: 1.14,
            distance: 34.0,
            pitch_min: 0.95,
            pitch_max: 1.34,
            distance_min: 18.0,
            distance_max: 48.0,
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
pub struct CavernPredictedFrame {
    pub tick: SimulationTick,
    pub control: CavernControlState,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernPredictionState {
    pub pending_frames: Vec<CavernPredictedFrame>,
    pub corrections_applied: u64,
    pub last_authoritative_tick: SimulationTick,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CavernServerControlMap {
    pub by_player_id: BTreeMap<u32, CavernControlState>,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfAgent {
    pub pos: [f32; 2],
    pub radius: f32,
    pub health_ratio: f32,
    pub team: u32,
    pub kind: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfGeometryPrimitive {
    pub shape_kind: u32,
    pub op_kind: u32,
    pub material_class: u32,
    pub material_instance: u32,
    pub p0: [f32; 4],
    pub p1: [f32; 4],
    pub p2: [f32; 4],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfMaterialProgramHeader {
    pub class_id: u32,
    pub op_offset: u32,
    pub op_count: u32,
    pub const_offset: u32,
    pub const_count: u32,
    pub base_color_slot: u32,
    pub roughness_slot: u32,
    pub metallic_slot: u32,
    pub normal_perturb_slot: u32,
    pub ao_slot: u32,
    pub emissive_slot: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfMaterialOp {
    pub op: u32,
    pub dst: u32,
    pub src_a: u32,
    pub src_b: u32,
    pub src_c: u32,
    pub const_idx: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CavernSdfWorldFrame {
    pub world_bounds: [f32; 4],
    pub floor_height: f32,
    pub rock_height: f32,
    pub camera: CavernCameraState,
    pub render_mode: u32,
    pub gi_mode: u32,
    pub gi_quality: u32,
    pub gi_sample_budget: u32,
    pub material_program_headers: Vec<CavernSdfMaterialProgramHeader>,
    pub material_ops: Vec<CavernSdfMaterialOp>,
    pub material_constants: Vec<[f32; 4]>,
    pub geometry_primitives: Vec<CavernSdfGeometryPrimitive>,
    pub agents: Vec<CavernSdfAgent>,
}

impl Default for CavernSdfWorldFrame {
    fn default() -> Self {
        Self {
            world_bounds: [-24.0, -24.0, 24.0, 24.0],
            floor_height: 0.0,
            rock_height: 3.8,
            camera: CavernCameraState::default(),
            render_mode: crate::domain::CAVERN_RENDER_MODE_MATERIAL_GRAPH,
            gi_mode: crate::domain::CAVERN_GI_MODE_AO_BENT,
            gi_quality: 1,
            gi_sample_budget: 14,
            material_program_headers: Vec::new(),
            material_ops: Vec::new(),
            material_constants: Vec::new(),
            geometry_primitives: Vec::new(),
            agents: Vec::new(),
        }
    }
}
