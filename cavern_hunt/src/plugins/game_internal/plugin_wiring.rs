// Owner: Cavern Hunt Gameplay Plugin - Plugin Wiring
pub struct CavernHuntPlugin;
pub struct CavernHuntClientPlugin;
pub struct CavernHuntServerPlugin;

impl Plugin for CavernHuntPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CavernRunConfig>();
        app.init_resource::<CavernRunState>();
        app.init_resource::<CavernLayout>();
        app.init_resource::<CavernTopology>();
        app.init_resource::<CavernGeometryGraph>();
        app.init_resource::<CavernCollisionField>();
        app.init_resource::<SpawnDirector>();
        app.init_resource::<LootTableRegistry>();
        app.init_resource::<CavernMetaProfile>();
        app.init_resource::<CavernMetaPersistenceConfig>();
        app.init_resource::<CavernMetaRewardState>();
        app.init_resource::<LocalPlayerRef>();
        app.init_resource::<CavernCameraState>();
        app.init_resource::<CavernAimState>();
        app.init_resource::<CavernControlState>();
        app.init_resource::<CavernPredictionState>();
        app.init_resource::<ServerReplicationMap>();
        app.init_resource::<ClientReplicationMap>();
        app.init_resource::<CavernServerControlMap>();
        app.init_resource::<CavernServerAppliedInputTickMap>();
        app.init_resource::<CavernPlayerOwnershipState>();
        app.init_resource::<InterpolationConfig>();
        app.init_resource::<AdaptiveSmoothingState>();
        app.init_resource::<CorrectionStats>();
        app.init_resource::<ReplicationBudgetConfig>();
        app.init_resource::<ReplicationCadenceConfig>();
        app.init_resource::<ReplicationLoadShedConfig>();
        app.init_resource::<ReplicationKeyframeConfig>();
        app.init_resource::<ReplicationRuntimeMetrics>();
        app.init_resource::<NetSyncModeConfig>();
        app.init_resource::<NetDiagnosticsConfigAssetV1>();
        app.init_resource::<CavernSdfWorldFrame>();
        app.init_resource::<CavernGeometryRuntimeState>();
        app.init_resource::<SessionSpawnPolicy>();
        app.init_resource::<RoomEncounterRegistry>();
        app.init_resource::<CavernObjectiveState>();
        app.init_resource::<ExtractionState>();
        app.init_resource::<CavernHudState>();
        app.init_resource::<PlayerCombatTuning>();
        app.init_resource::<EnemyCombatTuning>();
        app.init_resource::<UiWorldHudStats>();
        app.add_plugins((
            materials::CavernHuntMaterialPlugin,
            net_config::CavernHuntNetConfigPlugin,
            combat::CavernHuntCombatPlugin,
            ai::CavernHuntAiPlugin,
            hud::CavernHuntHudPlugin,
            loot::CavernHuntLootPlugin,
            net_sync::CavernHuntNetSyncPlugin,
        ));
        app.add_systems(
            PreUpdate,
            (
                sync_session_runtime_config_system.after(CoreSet::NetReceive),
                sync_session_spawn_policy_system.after(CoreSet::NetReceive),
                sync_active_player_slots_system.after(CoreSet::NetReceive),
            ),
        );
        app.add_systems(Update, sync_run_presentation_state_system);
        app.add_systems(Update, meta::apply_run_meta_rewards_system);
    }
}

impl Plugin for CavernHuntClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, client_setup_system);
        app.add_systems(
            Update,
            (
                render_sdf::update_camera_and_hud_system,
                render_sdf::build_sdf_world_frame_system,
            ),
        );
    }
}

impl Plugin for CavernHuntServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, server_setup_system);
    }
}
