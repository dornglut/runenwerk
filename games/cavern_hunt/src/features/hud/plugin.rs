use crate::{
    AdaptiveSmoothingState, CavernHudState, CavernObjectiveState, CavernRunState, CorrectionStats,
    DamageFeedbackState, DashState, Health, InventoryRunState, LocalPlayerRef, PlayerCompanion,
    PlayerId, PlayerSpectator, PlayerStatusPanel, ReplicationRuntimeMetrics, RoomEncounterRegistry,
};
use anyhow::Result;
use engine::plugins::ui::domain::UiWorldHudStats;
use engine::prelude::RoundTripMetrics;
use engine::prelude::{App, Plugin, Update, World};

pub struct CavernHuntHudPlugin;

impl Plugin for CavernHuntHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_cavern_hud_system);
    }
}

fn update_cavern_hud_system(mut world: engine::prelude::WorldMut) -> Result<()> {
    update_cavern_hud(&mut world)
}

pub(crate) fn update_cavern_hud(world: &mut World) -> Result<()> {
    let objective = world.resource::<CavernObjectiveState>()?.clone();
    let run_state = world.resource::<CavernRunState>()?.clone();
    let local_player_ref = world.resource::<LocalPlayerRef>()?.clone();
    let local_entity = local_player_ref.entity;

    let (local_health, local_max_health, dash_cooldown_remaining, local_scrap) = local_entity
        .and_then(|entity| {
            let health = world.get::<Health>(entity).copied()?;
            let dash = world.get::<DashState>(entity).copied().unwrap_or_default();
            let inventory =
                world
                    .get::<InventoryRunState>(entity)
                    .cloned()
                    .unwrap_or(InventoryRunState {
                        scrap: 0,
                        weapon_mods: Vec::new(),
                        relics: Vec::new(),
                    });
            Some((
                health.current,
                health.max,
                dash.cooldown_remaining,
                inventory.scrap,
            ))
        })
        .unwrap_or((0.0, 0.0, 0.0, 0));

    let teammates = world
        .query::<(engine::prelude::Entity, &PlayerId)>()
        .iter()
        .filter_map(|(entity, player_id)| {
            if world.get::<crate::PlayerActive>(entity).is_none() {
                return None;
            }
            let health = world.get::<Health>(entity).copied()?;
            let inventory =
                world
                    .get::<InventoryRunState>(entity)
                    .cloned()
                    .unwrap_or(InventoryRunState {
                        scrap: 0,
                        weapon_mods: Vec::new(),
                        relics: Vec::new(),
                    });
            let label = world
                .get::<crate::PlayerRosterIdentity>(entity)
                .map(|identity| identity.player_code.clone())
                .unwrap_or_else(|| format!("hunter_{}", player_id.0));
            Some(PlayerStatusPanel {
                player_id: player_id.0,
                label,
                alive: health.current > 0.0 && world.get::<PlayerSpectator>(entity).is_none(),
                is_companion: world.get::<PlayerCompanion>(entity).is_some(),
                scrap: inventory.scrap,
                health_ratio: health.ratio(),
            })
        })
        .collect::<Vec<_>>();

    let extraction = world.resource::<crate::ExtractionState>()?.clone();
    let encounter_summary = summarize_encounters(world);
    let feedback_summary = summarize_feedback(world);
    let network_summary = summarize_network(world);
    let smoothing_summary = summarize_smoothing(world);
    let player_position = local_entity
        .and_then(|entity| world.get::<crate::Transform2>(entity).copied())
        .map(|transform| [transform.x, transform.y])
        .unwrap_or([0.0, 0.0]);
    let enemies_alive = world.query::<&crate::EnemyKind>().iter().count();

    let mut hud_state = world.resource::<CavernHudState>()?.clone();
    hud_state.visible = true;
    hud_state.local_health = local_health;
    hud_state.local_max_health = local_max_health;
    hud_state.dash_cooldown_remaining = dash_cooldown_remaining;
    hud_state.scrap = local_scrap;
    hud_state.elite_defeated = run_state.elite_defeated;
    hud_state.extraction_active = run_state.extraction_active;
    hud_state.objective = crate::RunObjectivePanel {
        title: objective.title.clone(),
        detail: objective.detail.clone(),
    };
    hud_state.extraction = crate::ExtractionCountdownPanel {
        visible: extraction.active,
        seconds_remaining: extraction.countdown_remaining_seconds,
    };
    hud_state.teammates = teammates.clone();
    hud_state.status_lines = vec![
        format!(
            "health={:.0}/{:.0} dash_cd={:.1}s scrap={}",
            local_health, local_max_health, dash_cooldown_remaining, local_scrap
        ),
        format!(
            "elite={} extraction={}",
            if run_state.elite_defeated {
                "down"
            } else {
                "alive"
            },
            if run_state.extraction_active {
                "live"
            } else {
                "locked"
            }
        ),
        encounter_summary,
        feedback_summary,
        network_summary,
        smoothing_summary,
    ];
    world.insert_resource(hud_state.clone());

    if let Ok(mut stats) = world.resource_mut::<UiWorldHudStats>() {
        stats.visible = true;
        stats.player_x = player_position[0];
        stats.player_y = player_position[1];
        stats.enemies_alive = enemies_alive;
        stats.enemy_kills = run_state.enemy_kills;
        stats.panel_title = "Cavern Hunt".to_string();
        stats.lines = build_ui_lines(&hud_state);
    }

    Ok(())
}

fn build_ui_lines(hud: &CavernHudState) -> Vec<String> {
    let mut lines = vec![
        format!("objective: {}", hud.objective.title),
        hud.objective.detail.clone(),
        format!(
            "health {:.0}/{:.0}  dash {:.1}s  scrap {}",
            hud.local_health, hud.local_max_health, hud.dash_cooldown_remaining, hud.scrap
        ),
    ];
    if hud.extraction.visible {
        lines.push(format!(
            "extraction countdown {:.1}s",
            hud.extraction.seconds_remaining
        ));
    }
    for teammate in &hud.teammates {
        lines.push(format!(
            "{}{}  hp {:.0}%  scrap {}  {}",
            teammate.label,
            if teammate.is_companion { " [AI]" } else { "" },
            teammate.health_ratio * 100.0,
            teammate.scrap,
            if teammate.alive { "alive" } else { "down" }
        ));
    }
    lines.extend(hud.status_lines.iter().cloned());
    lines
}

fn summarize_encounters(world: &World) -> String {
    let encounters = match world.resource::<RoomEncounterRegistry>() {
        Ok(encounters) => encounters,
        Err(_) => return "rooms active 0 cleared 0".to_string(),
    };
    let active = encounters
        .by_room_id
        .values()
        .filter(|status| status.state == crate::RoomEncounterState::Active)
        .count();
    let cleared = encounters
        .by_room_id
        .values()
        .filter(|status| status.state == crate::RoomEncounterState::Cleared)
        .count();
    format!("rooms active {} cleared {}", active, cleared)
}

fn summarize_feedback(world: &World) -> String {
    let local_entity = world
        .resource::<LocalPlayerRef>()
        .ok()
        .and_then(|local| local.entity);
    let Some(entity) = local_entity else {
        return "feedback idle".to_string();
    };
    let feedback =
        world
            .get::<DamageFeedbackState>(entity)
            .copied()
            .unwrap_or(DamageFeedbackState {
                last_damage_taken: 0.0,
                last_damage_dealt: 0.0,
            });
    format!(
        "recent dmg taken {:.1} dealt {:.1}",
        feedback.last_damage_taken, feedback.last_damage_dealt
    )
}

fn summarize_network(world: &World) -> String {
    let metrics = world
        .resource::<ReplicationRuntimeMetrics>()
        .copied()
        .unwrap_or_default();
    format!(
        "net l{} tx {}B(rx {}B) patch_us {} key {} patch {} restore {} ops p/e/pr/pu/ex {}/{}/{}/{}/{} drop e/pr/pu/ex {}/{}/{}/{} smooth n {} err {:.2}/{:.2} a {:.2} ex {:.0}ms",
        metrics.load_shed_level_last_tick,
        metrics.bytes_sent_last_tick,
        metrics.bytes_received_last_frame,
        metrics.patch_apply_micros_last,
        metrics.keyframes_applied,
        metrics.patches_applied,
        metrics.full_world_restores,
        metrics.patch_player_ops_last_tick,
        metrics.patch_enemy_ops_last_tick,
        metrics.patch_projectile_ops_last_tick,
        metrics.patch_pickup_ops_last_tick,
        metrics.patch_extraction_ops_last_tick,
        metrics.dropped_enemy_ops_last_tick,
        metrics.dropped_projectile_ops_last_tick,
        metrics.dropped_pickup_ops_last_tick,
        metrics.dropped_extraction_ops_last_tick,
        metrics.smoothing_samples_last_frame,
        metrics.smoothing_error_mean_last_frame,
        metrics.smoothing_error_max_last_frame,
        metrics.smoothing_alpha_mean_last_frame,
        metrics.smoothing_extrapolation_ms_last_frame
    )
}

fn summarize_smoothing(world: &World) -> String {
    let metrics = world
        .resource::<ReplicationRuntimeMetrics>()
        .copied()
        .unwrap_or_default();
    let rtt = world
        .resource::<RoundTripMetrics>()
        .ok()
        .and_then(|metrics| metrics.last_rtt_millis.map(|millis| millis as f32))
        .unwrap_or(0.0);
    let smoothing = world
        .resource::<AdaptiveSmoothingState>()
        .copied()
        .unwrap_or_default();
    let correction = world
        .resource::<CorrectionStats>()
        .copied()
        .unwrap_or_default();
    format!(
        "rtt {:.0}ms jitter {:.0}ms smooth {:.0}ms corr s/m/l/h {}/{}/{}/{} ({:.2} ema {:.2}) rem err {:.2}/{:.2}",
        if rtt > 0.0 {
            rtt
        } else {
            smoothing.last_rtt_ms
        },
        smoothing.jitter_ms,
        smoothing.effective_delay_ms,
        correction.small_corrections,
        correction.medium_corrections,
        correction.large_corrections,
        correction.hard_snaps,
        correction.last_distance,
        correction.ema_distance,
        metrics.smoothing_error_mean_last_frame,
        metrics.smoothing_error_max_last_frame
    )
}
