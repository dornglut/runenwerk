use crate::domain::{
    CavernEnemyPatchOpV2, CavernExtractionPatchOpV2, CavernPatchEventV2, CavernPatchPriorityV2,
    CavernPickupPatchOpV2, CavernPlayerPatchOpV2, CavernProjectilePatchOpV2, CavernRunSnapshotV1,
    CavernRunStatePatchV2, NetworkEntityId, ReplicationBudgetConfig, ReplicationCadenceConfig,
    ReplicationCursor, ServerReplicationMap,
};

use super::should_emit_patch_channel;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub(super) struct PatchBuildStats {
    pub(super) dropped_enemy_ops: u64,
    pub(super) dropped_projectile_ops: u64,
    pub(super) dropped_pickup_ops: u64,
    pub(super) dropped_extraction_ops: u64,
}

pub(super) fn build_patch_event_v2(
    replication_map: &mut ServerReplicationMap,
    cursor: ReplicationCursor,
    previous_snapshot: Option<&CavernRunSnapshotV1>,
    snapshot: &CavernRunSnapshotV1,
    load_shed_level: u8,
    budget_config: &ReplicationBudgetConfig,
    cadence_config: &ReplicationCadenceConfig,
) -> (CavernPatchEventV2, PatchBuildStats) {
    let run_state = previous_snapshot.and_then(|base| {
        let patch = CavernRunStatePatchV2 {
            phase: (base.phase != snapshot.phase).then_some(snapshot.phase),
            elite_defeated: (base.elite_defeated != snapshot.elite_defeated)
                .then_some(snapshot.elite_defeated),
            extraction_active: (base.extraction_active != snapshot.extraction_active)
                .then_some(snapshot.extraction_active),
            extraction_started_at_tick: (base.extraction_started_at_tick
                != snapshot.extraction_started_at_tick)
                .then_some(snapshot.extraction_started_at_tick),
            party_alive_count: (base.party_alive_count != snapshot.party_alive_count)
                .then_some(snapshot.party_alive_count),
            enemy_kills: (base.enemy_kills != snapshot.enemy_kills).then_some(snapshot.enemy_kills),
            objective: (base.objective != snapshot.objective).then_some(snapshot.objective.clone()),
            extraction: (base.extraction != snapshot.extraction)
                .then_some(snapshot.extraction.clone()),
        };
        let has_changes = patch.phase.is_some()
            || patch.elite_defeated.is_some()
            || patch.extraction_active.is_some()
            || patch.extraction_started_at_tick.is_some()
            || patch.party_alive_count.is_some()
            || patch.enemy_kills.is_some()
            || patch.objective.is_some()
            || patch.extraction.is_some();
        has_changes.then_some(patch)
    });

    let mut player_ops = Vec::new();
    let mut previous_by_player_id = previous_snapshot
        .map(|snapshot| {
            snapshot
                .players
                .iter()
                .map(|player| (player.player_id, player))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    for player in &snapshot.players {
        let network_entity_id = replication_map
            .by_player_id
            .entry(player.player_id)
            .or_insert_with(|| NetworkEntityId(0x1000_0000 + u64::from(player.player_id)))
            .to_owned();
        match previous_by_player_id.remove(&player.player_id) {
            None => player_ops.push(CavernPlayerPatchOpV2::Spawn {
                entity_id: network_entity_id,
                priority: CavernPatchPriorityV2::Critical,
                state: player.clone(),
            }),
            Some(previous_state) if previous_state != player => {
                player_ops.push(CavernPlayerPatchOpV2::Patch {
                    entity_id: network_entity_id,
                    priority: CavernPatchPriorityV2::High,
                    state: player.clone(),
                })
            }
            _ => {}
        }
    }
    for (player_id, _) in previous_by_player_id {
        let entity_id = replication_map
            .by_player_id
            .remove(&player_id)
            .unwrap_or(NetworkEntityId(0x1000_0000 + u64::from(player_id)));
        player_ops.push(CavernPlayerPatchOpV2::Despawn {
            entity_id,
            player_id,
        });
    }

    let emit_enemies = should_emit_patch_channel(
        cursor.stream_cursor,
        enemy_patch_interval(cadence_config, load_shed_level),
    );
    let emit_projectiles = should_emit_patch_channel(
        cursor.stream_cursor,
        projectile_patch_interval(cadence_config, load_shed_level),
    );
    let emit_pickups = should_emit_patch_channel(
        cursor.stream_cursor,
        pickup_patch_interval(cadence_config, load_shed_level),
    );
    let emit_extraction = should_emit_patch_channel(
        cursor.stream_cursor,
        extraction_patch_interval(cadence_config, load_shed_level),
    );
    let mut enemy_ops = Vec::new();
    if emit_enemies {
        let mut previous_by_entity = previous_snapshot
            .map(|snapshot| {
                snapshot
                    .enemies
                    .iter()
                    .map(|enemy| (enemy.network_entity_id, enemy))
                    .collect::<std::collections::BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        for enemy in &snapshot.enemies {
            let entity_id = enemy.network_entity_id;
            match previous_by_entity.remove(&entity_id) {
                None => enemy_ops.push(CavernEnemyPatchOpV2::Spawn {
                    entity_id,
                    priority: CavernPatchPriorityV2::High,
                    state: enemy.clone(),
                }),
                Some(previous_state) if previous_state != enemy => {
                    enemy_ops.push(CavernEnemyPatchOpV2::Patch {
                        entity_id,
                        priority: CavernPatchPriorityV2::High,
                        state: enemy.clone(),
                    })
                }
                _ => {}
            }
        }
        for (entity_id, _) in previous_by_entity {
            enemy_ops.push(CavernEnemyPatchOpV2::Despawn { entity_id });
        }
    }

    let mut projectile_ops = Vec::new();
    if emit_projectiles {
        let mut previous_by_entity = previous_snapshot
            .map(|snapshot| {
                snapshot
                    .projectiles
                    .iter()
                    .map(|projectile| (projectile.network_entity_id, projectile))
                    .collect::<std::collections::BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        for projectile in &snapshot.projectiles {
            let entity_id = projectile.network_entity_id;
            match previous_by_entity.remove(&entity_id) {
                None => projectile_ops.push(CavernProjectilePatchOpV2::Spawn {
                    entity_id,
                    priority: CavernPatchPriorityV2::Medium,
                    state: projectile.clone(),
                }),
                Some(previous_state) if previous_state != projectile => {
                    projectile_ops.push(CavernProjectilePatchOpV2::Patch {
                        entity_id,
                        priority: CavernPatchPriorityV2::Medium,
                        state: projectile.clone(),
                    })
                }
                _ => {}
            }
        }
        for (entity_id, _) in previous_by_entity {
            projectile_ops.push(CavernProjectilePatchOpV2::Despawn { entity_id });
        }
    }

    let mut pickup_ops = Vec::new();
    if emit_pickups {
        let mut previous_by_entity = previous_snapshot
            .map(|snapshot| {
                snapshot
                    .pickups
                    .iter()
                    .map(|pickup| (pickup.network_entity_id, pickup))
                    .collect::<std::collections::BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        for pickup in &snapshot.pickups {
            let entity_id = pickup.network_entity_id;
            match previous_by_entity.remove(&entity_id) {
                None => pickup_ops.push(CavernPickupPatchOpV2::Spawn {
                    entity_id,
                    priority: CavernPatchPriorityV2::Low,
                    state: pickup.clone(),
                }),
                Some(previous_state) if previous_state != pickup => {
                    pickup_ops.push(CavernPickupPatchOpV2::Patch {
                        entity_id,
                        priority: CavernPatchPriorityV2::Low,
                        state: pickup.clone(),
                    })
                }
                _ => {}
            }
        }
        for (entity_id, _) in previous_by_entity {
            pickup_ops.push(CavernPickupPatchOpV2::Despawn { entity_id });
        }
    }

    let mut extraction_ops = Vec::new();
    if emit_extraction {
        let mut previous_by_entity = previous_snapshot
            .map(|snapshot| {
                snapshot
                    .extraction_zones
                    .iter()
                    .map(|zone| (zone.network_entity_id, zone))
                    .collect::<std::collections::BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        for zone in &snapshot.extraction_zones {
            let entity_id = zone.network_entity_id;
            match previous_by_entity.remove(&entity_id) {
                None => extraction_ops.push(CavernExtractionPatchOpV2::Spawn {
                    entity_id,
                    priority: CavernPatchPriorityV2::Critical,
                    state: zone.clone(),
                }),
                Some(previous_state) if previous_state != zone => {
                    extraction_ops.push(CavernExtractionPatchOpV2::Patch {
                        entity_id,
                        priority: CavernPatchPriorityV2::Critical,
                        state: zone.clone(),
                    })
                }
                _ => {}
            }
        }
        for (entity_id, _) in previous_by_entity {
            extraction_ops.push(CavernExtractionPatchOpV2::Despawn { entity_id });
        }
    }

    let (enemy_ops, dropped_enemy_ops) =
        cap_enemy_ops(enemy_ops, enemy_ops_budget(budget_config, load_shed_level));
    let (projectile_ops, dropped_projectile_ops) = cap_projectile_ops(
        projectile_ops,
        projectile_ops_budget(budget_config, load_shed_level),
    );
    let (pickup_ops, dropped_pickup_ops) = cap_pickup_ops(
        pickup_ops,
        pickup_ops_budget(budget_config, load_shed_level),
    );
    let (extraction_ops, dropped_extraction_ops) = cap_extraction_ops(
        extraction_ops,
        extraction_ops_budget(budget_config, load_shed_level),
    );

    (
        CavernPatchEventV2 {
            cursor,
            run_state,
            player_ops,
            enemy_ops,
            projectile_ops,
            pickup_ops,
            extraction_ops,
        },
        PatchBuildStats {
            dropped_enemy_ops,
            dropped_projectile_ops,
            dropped_pickup_ops,
            dropped_extraction_ops,
        },
    )
}

fn enemy_patch_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.enemy_patch_interval_level0,
        1 => config.enemy_patch_interval_level1,
        _ => config.enemy_patch_interval_level2,
    }
}

fn projectile_patch_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.projectile_patch_interval_level0,
        1 => config.projectile_patch_interval_level1,
        _ => config.projectile_patch_interval_level2,
    }
}

fn pickup_patch_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.pickup_patch_interval_level0,
        1 => config.pickup_patch_interval_level1,
        _ => config.pickup_patch_interval_level2,
    }
}

fn extraction_patch_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.extraction_patch_interval_level0,
        1 => config.extraction_patch_interval_level1,
        _ => config.extraction_patch_interval_level2,
    }
}

fn enemy_ops_budget(config: &ReplicationBudgetConfig, load_shed_level: u8) -> usize {
    match load_shed_level {
        0 => config.enemy_ops_per_patch_level0,
        1 => config.enemy_ops_per_patch_level1,
        _ => config.enemy_ops_per_patch_level2,
    }
}

fn projectile_ops_budget(config: &ReplicationBudgetConfig, load_shed_level: u8) -> usize {
    match load_shed_level {
        0 => config.projectile_ops_per_patch_level0,
        1 => config.projectile_ops_per_patch_level1,
        _ => config.projectile_ops_per_patch_level2,
    }
}

fn pickup_ops_budget(config: &ReplicationBudgetConfig, load_shed_level: u8) -> usize {
    match load_shed_level {
        0 => config.pickup_ops_per_patch_level0,
        1 => config.pickup_ops_per_patch_level1,
        _ => config.pickup_ops_per_patch_level2,
    }
}

fn extraction_ops_budget(config: &ReplicationBudgetConfig, load_shed_level: u8) -> usize {
    match load_shed_level {
        0 => config.extraction_ops_per_patch_level0,
        1 => config.extraction_ops_per_patch_level1,
        _ => config.extraction_ops_per_patch_level2,
    }
}

fn cap_enemy_ops(
    mut ops: Vec<CavernEnemyPatchOpV2>,
    cap: usize,
) -> (Vec<CavernEnemyPatchOpV2>, u64) {
    if cap == 0 {
        return (Vec::new(), ops.len() as u64);
    }
    if ops.len() <= cap {
        return (ops, 0);
    }
    ops.sort_by_key(enemy_op_sort_key);
    let dropped = ops.len().saturating_sub(cap) as u64;
    ops.truncate(cap);
    (ops, dropped)
}

fn cap_projectile_ops(
    mut ops: Vec<CavernProjectilePatchOpV2>,
    cap: usize,
) -> (Vec<CavernProjectilePatchOpV2>, u64) {
    if cap == 0 {
        return (Vec::new(), ops.len() as u64);
    }
    if ops.len() <= cap {
        return (ops, 0);
    }
    ops.sort_by_key(projectile_op_sort_key);
    let dropped = ops.len().saturating_sub(cap) as u64;
    ops.truncate(cap);
    (ops, dropped)
}

fn cap_pickup_ops(
    mut ops: Vec<CavernPickupPatchOpV2>,
    cap: usize,
) -> (Vec<CavernPickupPatchOpV2>, u64) {
    if cap == 0 {
        return (Vec::new(), ops.len() as u64);
    }
    if ops.len() <= cap {
        return (ops, 0);
    }
    ops.sort_by_key(pickup_op_sort_key);
    let dropped = ops.len().saturating_sub(cap) as u64;
    ops.truncate(cap);
    (ops, dropped)
}

fn cap_extraction_ops(
    mut ops: Vec<CavernExtractionPatchOpV2>,
    cap: usize,
) -> (Vec<CavernExtractionPatchOpV2>, u64) {
    if cap == 0 {
        return (Vec::new(), ops.len() as u64);
    }
    if ops.len() <= cap {
        return (ops, 0);
    }
    ops.sort_by_key(extraction_op_sort_key);
    let dropped = ops.len().saturating_sub(cap) as u64;
    ops.truncate(cap);
    (ops, dropped)
}

fn enemy_op_sort_key(op: &CavernEnemyPatchOpV2) -> (u8, u64) {
    match op {
        CavernEnemyPatchOpV2::Despawn { entity_id } => (0, entity_id.0),
        CavernEnemyPatchOpV2::Spawn { entity_id, .. } => (1, entity_id.0),
        CavernEnemyPatchOpV2::Patch { entity_id, .. } => (2, entity_id.0),
    }
}

fn projectile_op_sort_key(op: &CavernProjectilePatchOpV2) -> (u8, u64) {
    match op {
        CavernProjectilePatchOpV2::Despawn { entity_id } => (0, entity_id.0),
        CavernProjectilePatchOpV2::Spawn { entity_id, .. } => (1, entity_id.0),
        CavernProjectilePatchOpV2::Patch { entity_id, .. } => (2, entity_id.0),
    }
}

fn pickup_op_sort_key(op: &CavernPickupPatchOpV2) -> (u8, u64) {
    match op {
        CavernPickupPatchOpV2::Despawn { entity_id } => (0, entity_id.0),
        CavernPickupPatchOpV2::Spawn { entity_id, .. } => (1, entity_id.0),
        CavernPickupPatchOpV2::Patch { entity_id, .. } => (2, entity_id.0),
    }
}

fn extraction_op_sort_key(op: &CavernExtractionPatchOpV2) -> (u8, u64) {
    match op {
        CavernExtractionPatchOpV2::Despawn { entity_id } => (0, entity_id.0),
        CavernExtractionPatchOpV2::Spawn { entity_id, .. } => (1, entity_id.0),
        CavernExtractionPatchOpV2::Patch { entity_id, .. } => (2, entity_id.0),
    }
}
