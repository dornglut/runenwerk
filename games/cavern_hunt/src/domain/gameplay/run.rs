use crate::CavernSeed;
use crate::RoomId;
use engine::SimulationTick;
use serde::{Deserialize, Serialize};

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
