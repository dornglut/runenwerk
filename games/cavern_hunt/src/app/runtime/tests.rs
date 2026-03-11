use super::*;

// Owner: Cavern Hunt Gameplay Plugin - Tests
#[cfg(test)]
mod tests {
    use super::sync_active_player_slots;
    use crate::features::worldgen::plugin as worldgen;
    use crate::{
        CavernAimState, CavernCameraState, CavernLayout, CavernMetaPersistenceConfig,
        CavernMetaProfile, CavernMetaRewardState, CavernPlayerOwnershipState, CavernRunConfig,
        CavernRunState, CavernSdfWorldFrame, CavernServerControlMap, LocalPlayerRef,
        LootTableRegistry, PlayerActive, PlayerCompanion, PlayerId, PlayerRosterIdentity,
        SpawnDirector,
    };
    use engine::plugins::ui::domain::UiWorldHudStats;
    use engine::prelude::{
        AuthorityRole, DeterminismLevel, SimulationProfile, SimulationProfileConfig, World,
    };
    use engine::state::SessionRuntimeState;
    use engine_net::{ConnectionId, ServerSessionConfig, ServerSessionState, SessionPhase};

    #[test]
    fn session_sync_does_not_shrink_default_party_capacity_for_dev_join_state() {
        let session = SessionRuntimeState {
            admitted: true,
            max_players: 1,
            ..SessionRuntimeState::default()
        };
        let mut config = CavernRunConfig {
            max_players: 4,
            ..CavernRunConfig::default()
        };

        super::sync_session_runtime_config(&session, &mut config);

        assert_eq!(config.max_players, 4);
    }

    #[test]
    fn session_settings_json_overrides_game_run_config() {
        let session = SessionRuntimeState {
            admitted: true,
            settings_json: Some(
                r#"{"seed":4242,"enemy_density":1.7,"extract_countdown_seconds":3.5,"base_scrap_reward":19}"#
                    .to_string(),
            ),
            ..SessionRuntimeState::default()
        };
        let mut config = CavernRunConfig::default();

        super::sync_session_runtime_config(&session, &mut config);

        assert_eq!(config.seed.0, 4242);
        assert_eq!(config.enemy_density, 1.7);
        assert_eq!(config.extract_countdown_seconds, 3.5);
        assert_eq!(config.base_scrap_reward, 19);
    }

    #[test]
    fn server_activation_follows_owned_connections() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(CavernMetaPersistenceConfig { enabled: false });
        world.insert_resource(CavernMetaRewardState::default());
        world.insert_resource(SessionRuntimeState {
            admitted: true,
            lobby_id: Some("lobby-test".into()),
            roster_player_codes: vec!["alpha".into(), "beta".into()],
            max_players: 2,
            ai_fill_target: 2,
            settings_json: None,
        });
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(11, 1), (22, 2)].into_iter().collect(),
        });
        world.insert_resource(CavernSdfWorldFrame::default());
        world.insert_resource(UiWorldHudStats::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, false).unwrap();
        let player_query = world.query_state::<(engine::prelude::Entity, &PlayerId), ()>();
        assert_eq!(player_query.iter(&world).count(), 0);

        sync_active_player_slots(&mut world).unwrap();

        let player_query = world.query_state::<(engine::prelude::Entity, &PlayerId), ()>();
        let active_ids = player_query
            .iter(&world)
            .filter_map(|(entity, player_id)| {
                world.get::<PlayerActive>(entity).is_some().then(|| {
                    (
                        player_id.0,
                        world
                            .get::<PlayerRosterIdentity>(entity)
                            .map(|identity| identity.player_code.clone())
                            .unwrap_or_default(),
                    )
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            active_ids,
            vec![(1, "alpha".to_string()), (2, "beta".to_string())]
        );
        assert_eq!(
            world
                .resource::<CavernRunState>()
                .unwrap()
                .party_alive_count,
            2
        );
    }

    #[test]
    fn server_can_fill_missing_party_slots_with_companions() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(CavernMetaPersistenceConfig { enabled: false });
        world.insert_resource(CavernMetaRewardState::default());
        world.insert_resource(SessionRuntimeState {
            admitted: true,
            lobby_id: Some("lobby-fill".into()),
            roster_player_codes: vec!["alpha".into()],
            max_players: 4,
            ai_fill_target: 3,
            settings_json: None,
        });
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(11, 1)].into_iter().collect(),
        });
        world.insert_resource(CavernSdfWorldFrame::default());
        world.insert_resource(UiWorldHudStats::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, false).unwrap();

        sync_active_player_slots(&mut world).unwrap();

        let player_query = world.query_state::<(engine::prelude::Entity, &PlayerId), ()>();
        let active_players = player_query
            .iter(&world)
            .filter_map(|(entity, player_id)| {
                world
                    .get::<PlayerActive>(entity)
                    .is_some()
                    .then_some((entity, player_id.0))
            })
            .collect::<Vec<_>>();
        assert_eq!(active_players.len(), 3);
        assert_eq!(
            active_players
                .iter()
                .filter(|(entity, _)| world.get::<PlayerCompanion>(*entity).is_some())
                .count(),
            2
        );
    }

    #[test]
    fn client_keeps_replicated_remote_players_active() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(CavernMetaPersistenceConfig { enabled: false });
        world.insert_resource(CavernMetaRewardState::default());
        world.insert_resource(SessionRuntimeState {
            admitted: true,
            lobby_id: Some("lobby-client".into()),
            roster_player_codes: vec!["alpha".into(), "beta".into()],
            max_players: 2,
            ai_fill_target: 2,
            settings_json: None,
        });
        world.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: None,
        });
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState::default());
        world.insert_resource(CavernSdfWorldFrame::default());
        world.insert_resource(UiWorldHudStats::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Client,
            determinism: DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, true).unwrap();
        let meta = world.resource::<CavernMetaProfile>().unwrap().clone();
        worldgen::spawn_player_entity(
            &mut world,
            2,
            1,
            true,
            &meta,
            &crate::PlayerSpawnProfile::default(),
            "beta",
            1,
            false,
        );

        sync_active_player_slots(&mut world).unwrap();

        let player_query = world.query_state::<(engine::prelude::Entity, &PlayerId), ()>();
        let mut active_ids = player_query
            .iter(&world)
            .filter_map(|(entity, player_id)| {
                world
                    .get::<PlayerActive>(entity)
                    .is_some()
                    .then_some(player_id.0)
            })
            .collect::<Vec<_>>();
        active_ids.sort_unstable();
        assert_eq!(active_ids, vec![1, 2]);
    }

    #[test]
    fn server_assigns_player_slots_for_active_connections_without_input_frames() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(CavernMetaPersistenceConfig { enabled: false });
        world.insert_resource(CavernMetaRewardState::default());
        world.insert_resource(SessionRuntimeState {
            admitted: true,
            lobby_id: Some("lobby-connections".into()),
            roster_player_codes: vec![
                "alpha".into(),
                "beta".into(),
                "gamma".into(),
                "delta".into(),
            ],
            max_players: 4,
            ai_fill_target: 4,
            settings_json: None,
        });
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState::default());
        world.insert_resource(CavernSdfWorldFrame::default());
        world.insert_resource(UiWorldHudStats::default());
        world.insert_resource(ServerSessionState {
            phase: SessionPhase::Active,
            config: ServerSessionConfig::default(),
            next_connection_id: 5,
            active_connection: Some(ConnectionId(4)),
            active_connections: [
                ConnectionId(1),
                ConnectionId(2),
                ConnectionId(3),
                ConnectionId(4),
            ]
            .into_iter()
            .collect(),
            last_join_request: None,
            last_join_state: None,
            last_disconnect: None,
        });
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, false).unwrap();

        sync_active_player_slots(&mut world).unwrap();

        let ownership = world.resource::<CavernPlayerOwnershipState>().unwrap();
        assert_eq!(ownership.by_connection_id.len(), 4);
        let mut mapped_ids = ownership
            .by_connection_id
            .values()
            .copied()
            .collect::<Vec<_>>();
        mapped_ids.sort_unstable();
        assert_eq!(mapped_ids, vec![1, 2, 3, 4]);

        let player_query = world.query_state::<(engine::prelude::Entity, &PlayerId), ()>();
        let active_players = player_query
            .iter(&world)
            .filter(|(entity, _)| world.get::<PlayerActive>(*entity).is_some())
            .count();
        assert_eq!(active_players, 4);
    }
}
