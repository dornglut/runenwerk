use crate::domain::{
    AdaptiveSmoothingState, CavernAimState, CavernCameraState, CavernCollisionField,
    CavernControlState, CavernGeometryGraph, CavernGeometryRuntimeState, CavernHudState,
    CavernLayout, CavernMetaPersistenceConfig, CavernMetaProfile, CavernMetaRewardState,
    CavernObjectiveKind, CavernObjectiveState, CavernPlayerOwnershipState, CavernPredictionState,
    CavernRunConfig, CavernRunState, CavernSdfWorldFrame, CavernServerAppliedInputTickMap,
    CavernServerControlMap, CavernSessionSettings, CavernTopology, ClientReplicationMap,
    CorrectionStats, EnemyCombatTuning, ExtractionState, InterpolationConfig, LocalPlayerRef,
    LootTableRegistry, NetDiagnosticsConfigAssetV1, NetSyncModeConfig, PlayerActive,
    PlayerCombatTuning, PlayerId, PlayerSpawnProfile, ReplicationBudgetConfig,
    ReplicationCadenceConfig, ReplicationKeyframeConfig, ReplicationLoadShedConfig,
    ReplicationRuntimeMetrics, RoomEncounterRegistry, RoomEncounterState, RoomEncounterStatus,
    RoomRole, RunDifficultyProfile, ServerReplicationMap, SessionSpawnPolicy, SpawnDirector,
};
use crate::plugins::{
    ai, combat, hud, loot, materials, meta, net_config, net_sync, render_sdf,
    timing::fixed_step_seconds, worldgen,
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

include!("game_internal/plugin_wiring.rs");

include!("game_internal/session_sync.rs");

include!("game_internal/setup_and_slots.rs");

include!("game_internal/presentation.rs");

include!("game_internal/tests.rs");
