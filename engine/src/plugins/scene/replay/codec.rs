use super::super::SceneManager;
use super::super::domain;
use super::super::domain::SceneSlot;
use super::super::snapshot::{
    capture_scene_simulation_snapshot, restore_scene_simulation_snapshot,
};
use anyhow::Result;
use engine_replay::ReplayArchive;
use engine_sim::{SimulationCodec, SimulationTick};
use serde::{Deserialize, Serialize};

// Owner: Engine Scene Plugin - Replay Codec
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEntitySnapshotV1 {
    pub frame_counter: domain::WorldFrameCounter,
    pub debug_position: domain::WorldDebugPosition,
    pub debug_velocity: domain::WorldDebugVelocity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneWorldContextSnapshotV1 {
    pub world: SceneSlot,
    pub overlays: Vec<SceneSlot>,
    pub world_scene_label: String,
    pub overlay_scene_label: String,
    pub gameplay_config: domain::GameplayConfig,
    pub gameplay_config_modified_millis: Option<u64>,
    pub gameplay_config_revision: u64,
    pub overlay_consumed: bool,
    pub player_move_x: f32,
    pub player_move_y: f32,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub delta_seconds: f32,
    pub fixed_step_seconds: f32,
    pub fixed_step_accumulator: f32,
    pub frame_count: u64,
    pub enemy_kills: u32,
    pub session_admitted: bool,
    pub session_lobby_id: Option<String>,
    pub session_roster_player_codes: Vec<String>,
    pub session_max_players: u8,
    pub session_ai_fill_target: u8,
    pub session_settings_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSimulationSnapshotV1 {
    pub context: SceneWorldContextSnapshotV1,
    pub entities: SceneEntitySnapshotV1,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityDeltaV1 {
    pub frame_counter: Option<domain::WorldFrameCounter>,
    pub debug_position: Option<domain::WorldDebugPosition>,
    pub debug_velocity: Option<domain::WorldDebugVelocity>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneWorldContextDeltaV1 {
    pub world: Option<SceneSlot>,
    pub overlays: Option<Vec<SceneSlot>>,
    pub world_scene_label: Option<String>,
    pub overlay_scene_label: Option<String>,
    pub gameplay_config: Option<domain::GameplayConfig>,
    pub gameplay_config_modified_millis: Option<Option<u64>>,
    pub gameplay_config_revision: Option<u64>,
    pub overlay_consumed: Option<bool>,
    pub player_move_x: Option<f32>,
    pub player_move_y: Option<f32>,
    pub camera_yaw: Option<f32>,
    pub camera_pitch: Option<f32>,
    pub camera_distance: Option<f32>,
    pub delta_seconds: Option<f32>,
    pub fixed_step_seconds: Option<f32>,
    pub fixed_step_accumulator: Option<f32>,
    pub frame_count: Option<u64>,
    pub enemy_kills: Option<u32>,
    pub session_admitted: Option<bool>,
    pub session_lobby_id: Option<Option<String>>,
    pub session_roster_player_codes: Option<Vec<String>>,
    pub session_max_players: Option<u8>,
    pub session_ai_fill_target: Option<u8>,
    pub session_settings_json: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneSimulationDeltaV1 {
    pub context: SceneWorldContextDeltaV1,
    pub entities: SceneEntityDeltaV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneReplayCommandFrame {
    pub tick: SimulationTick,
    pub world: SceneSlot,
    pub overlays: Vec<SceneSlot>,
    pub world_scene_label: String,
    pub overlay_scene_label: String,
    pub gameplay_config: domain::GameplayConfig,
    pub gameplay_config_revision: u64,
    pub overlay_consumed: bool,
    pub player_move_x: f32,
    pub player_move_y: f32,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub delta_seconds: f32,
    pub fixed_step_seconds: f32,
    pub session_admitted: bool,
    pub session_lobby_id: Option<String>,
    pub session_roster_player_codes: Vec<String>,
    pub session_max_players: u8,
    pub session_ai_fill_target: u8,
    pub session_settings_json: Option<String>,
}

pub type SceneReplayArchive = ReplayArchive<SceneSimulationSnapshotV1, SceneReplayCommandFrame>;

pub(crate) struct SceneSimulationCodec;

impl SimulationCodec for SceneSimulationCodec {
    type Host = SceneManager;
    type Snapshot = SceneSimulationSnapshotV1;

    fn codec_id() -> &'static str {
        "scene_runtime_v1"
    }

    fn capture(host: &Self::Host) -> Result<Self::Snapshot> {
        capture_scene_simulation_snapshot(host)
    }

    fn restore(host: &mut Self::Host, snapshot: &Self::Snapshot) -> Result<()> {
        restore_scene_simulation_snapshot(host, snapshot)
    }
}
