use super::*;

fn encode_commands(commands: Vec<ClientCommandEnvelope>) -> Vec<u8> {
    postcard::to_allocvec(&commands).expect("commands should encode")
}

#[test]
fn v2_load_shed_level_accounts_for_connections_and_drops() {
    let cfg = ReplicationLoadShedConfig::default();
    assert_eq!(compute_load_shed_level_v2(10_000, 0, 2, &cfg), 0);
    assert_eq!(compute_load_shed_level_v2(10_000, 0, 3, &cfg), 1);
    assert_eq!(compute_load_shed_level_v2(55_000, 0, 4, &cfg), 2);
    assert_eq!(compute_load_shed_level_v2(30_000, 2, 2, &cfg), 1);
    assert_eq!(compute_load_shed_level_v2(30_000, 30, 2, &cfg), 2);
}

#[test]
fn v2_patch_channel_interval_gating_is_deterministic() {
    assert!(should_emit_patch_channel(7, 1));
    assert!(!should_emit_patch_channel(7, 0));
    assert!(!should_emit_patch_channel(7, 2));
    assert!(should_emit_patch_channel(8, 2));
    assert!(should_emit_patch_channel(12, 3));
}

#[test]
fn env_tuning_overrides_apply_and_clamp() {
    let mut budget = ReplicationBudgetConfig::default();
    let mut cadence = ReplicationCadenceConfig::default();
    let mut diagnostics = Vec::new();
    let vars = std::collections::BTreeMap::from([
        ("CAVERN_NET_BUDGET_ENEMY_L0", "220".to_string()),
        ("CAVERN_NET_BUDGET_PROJECTILE_L2", "99999".to_string()),
        ("CAVERN_NET_CADENCE_ENEMY_L1", "0".to_string()),
        ("CAVERN_NET_CADENCE_PICKUP_L2", "x".to_string()),
    ]);
    apply_replication_tuning_overrides_from_reader(
        &mut budget,
        &mut cadence,
        |key| vars.get(key).cloned(),
        &mut diagnostics,
    );
    assert_eq!(budget.enemy_ops_per_patch_level0, 220);
    assert_eq!(budget.projectile_ops_per_patch_level2, 4096);
    assert_eq!(cadence.enemy_patch_interval_level1, 0);
    assert!(diagnostics.iter().any(|d| d.contains("PROJECTILE_L2")));
    assert!(diagnostics.iter().any(|d| d.contains("PICKUP_L2")));
}

#[test]
fn preset_tuning_applies_four_local_profile() {
    let mut budget = ReplicationBudgetConfig::default();
    let mut cadence = ReplicationCadenceConfig::default();
    let mut diagnostics = Vec::new();
    apply_replication_tuning_preset(
        &mut budget,
        &mut cadence,
        Some("four_local"),
        &mut diagnostics,
    );
    assert!(diagnostics.is_empty());
    assert_eq!(budget.enemy_ops_per_patch_level0, 96);
    assert_eq!(budget.projectile_ops_per_patch_level1, 96);
    assert_eq!(cadence.enemy_patch_interval_level0, 2);
    assert_eq!(cadence.pickup_patch_interval_level2, 12);
}

#[test]
fn server_maps_input_frames_to_stable_player_ids_by_connection() {
    let mut world = server_world();
    world.insert_resource(NetworkInboundQueue::default());

    {
        let mut inbound = world.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.push_client(
            Some(ConnectionId(11)),
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(3),
                payload: encode_commands(vec![ClientCommandEnvelope::Move(MoveCommand {
                    x: 1.0,
                    y: 0.0,
                })]),
            }),
        );
        inbound.push_client(
            Some(ConnectionId(22)),
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(4),
                payload: encode_commands(vec![ClientCommandEnvelope::Move(MoveCommand {
                    x: 0.0,
                    y: 1.0,
                })]),
            }),
        );
    }

    capture_control_input(&mut world).unwrap();
    game::sync_active_player_slots(&mut world).unwrap();

    let ownership = world.resource::<CavernPlayerOwnershipState>().unwrap();
    assert_eq!(ownership.by_connection_id.get(&11), Some(&1));
    assert_eq!(ownership.by_connection_id.get(&22), Some(&2));

    let controls = world.resource::<CavernServerControlMap>().unwrap();
    assert_eq!(
        controls.by_player_id.get(&1).map(|state| state.movement),
        Some([1.0, 0.0])
    );
    assert_eq!(
        controls.by_player_id.get(&2).map(|state| state.movement),
        Some([0.0, 1.0])
    );
}

#[test]
fn server_capture_sanitizes_control_payload_and_drops_far_future_ticks() {
    let mut world = server_world();
    world.insert_resource(NetworkInboundQueue::default());

    {
        let mut inbound = world.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.push_client(
            Some(ConnectionId(11)),
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(2_000),
                payload: encode_commands(vec![ClientCommandEnvelope::Move(MoveCommand {
                    x: 1.0,
                    y: 0.0,
                })]),
            }),
        );
        inbound.push_client(
            Some(ConnectionId(11)),
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(2),
                payload: encode_commands(vec![
                    ClientCommandEnvelope::Move(MoveCommand { x: 5.0, y: 0.0 }),
                    ClientCommandEnvelope::Aim(AimCommand {
                        x: f32::NAN,
                        y: f32::INFINITY,
                    }),
                ]),
            }),
        );
    }

    capture_control_input(&mut world).unwrap();

    let controls = world.resource::<CavernServerControlMap>().unwrap();
    let captured = controls.by_player_id.get(&1).copied().unwrap();
    assert_eq!(captured.source_tick, SimulationTick(2));
    assert_eq!(captured.movement, [1.0, 0.0]);
    assert_eq!(captured.aim_world, [0.0, 0.0]);
}

#[test]
fn server_capture_ignores_frames_without_connection_identity() {
    let mut world = server_world();
    world.insert_resource(NetworkInboundQueue::default());

    {
        let mut inbound = world.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.push_client(
            None,
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(2),
                payload: encode_commands(vec![ClientCommandEnvelope::Move(MoveCommand {
                    x: 1.0,
                    y: 0.0,
                })]),
            }),
        );
    }

    capture_control_input(&mut world).unwrap();

    let controls = world.resource::<CavernServerControlMap>().unwrap();
    assert!(controls.by_player_id.is_empty());
    let ownership = world.resource::<CavernPlayerOwnershipState>().unwrap();
    assert!(ownership.by_connection_id.is_empty());
}
