use crate::{
    AdaptiveSmoothingState, CavernAimState, CavernCameraState, CavernCollisionField,
    CavernControlState, CavernGeometryGraph, CavernGeometryRuntimeState, CavernHudState,
    CavernLayout, CavernMetaPersistenceConfig, CavernMetaProfile, CavernMetaRewardState,
    CavernObjectiveKind, CavernObjectiveState, CavernPlayerOwnershipState, CavernPredictionState,
    CavernRunConfig, CavernRunState, CavernSdfWorldFrame, CavernServerAppliedInputTickMap,
    CavernServerControlMap, CavernSessionSettings, CavernTopology, ClientReplicationMap,
    CorrectionStats, EnemyCombatTuning, ExtractionState, InterpolationConfig, LocalPlayerRef,
    LootTableRegistry, NetDiagnosticsConfigAssetV1, PlayerActive,
    PlayerCombatTuning, PlayerId, PlayerSpawnProfile, ReplicationBudgetConfig,
    ReplicationCadenceConfig, ReplicationKeyframeConfig, ReplicationLoadShedConfig,
    ReplicationRuntimeMetrics, RoomEncounterRegistry, RoomEncounterState, RoomEncounterStatus,
    RoomRole, RunDifficultyProfile, ServerReplicationMap, SessionSpawnPolicy, SpawnDirector,
};
use crate::features::{
    ai::plugin as ai,
    combat::plugin as combat,
    hud::plugin as hud,
    loot::plugin as loot,
    materials::plugin as materials,
    meta::plugin as meta,
    multiplayer::net_config,
    render_sdf::plugin as render_sdf,
    timing::fixed_step_seconds,
    worldgen::plugin as worldgen,
};
use anyhow::Result;
use engine::plugins::ui::domain::UiWorldHudStats;
use engine::prelude::{
    App, AuthorityRole, CoreSet, Plugin, PreUpdate, Res, ResMut, SimulationProfileConfig, Startup,
    SystemConfigExt, Update, World, WorldMut,
};
use engine::state::SessionRuntimeState;
use engine_net::ServerSessionState;
use std::collections::BTreeSet;

#[path = "runtime/mod.rs"]
mod runtime;

pub use runtime::{CavernHuntClientPlugin, CavernHuntPlugin, CavernHuntServerPlugin};
pub(crate) use runtime::sync_active_player_slots;
