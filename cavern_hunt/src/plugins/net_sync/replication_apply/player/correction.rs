use anyhow::Result;
use engine::prelude::{FixedTimeConfig, SimulationTick, World};

use super::super::super::legacy::angle_delta;
use super::snapshot_state::apply_non_transform_player_snapshot;
use crate::domain::{
    CavernPredictionState, CorrectionStats, InterpolationConfig, Transform2, Velocity2,
};

pub(super) fn apply_local_player_snapshot_correction(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::domain::CavernPlayerSnapshotV1,
    interpolation: InterpolationConfig,
    dynamic_hard_snap_distance: f32,
    authoritative_tick: Option<SimulationTick>,
    correction_stats: &mut CorrectionStats,
) -> Result<()> {
    let current = world
        .get::<Transform2>(entity)
        .copied()
        .unwrap_or_else(|| Transform2::new(snapshot.x, snapshot.y, snapshot.yaw));
    let current_velocity = world
        .get::<Velocity2>(entity)
        .copied()
        .unwrap_or(Velocity2 {
            x: snapshot.velocity[0],
            y: snapshot.velocity[1],
        });
    let target = [snapshot.x, snapshot.y];
    let dx = target[0] - current.x;
    let dy = target[1] - current.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let local_simulated_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let (has_pending_for_replay, authoritative_tick_is_newer) = authoritative_tick
        .map(|tick| {
            world
                .resource::<CavernPredictionState>()
                .map(|prediction| {
                    let has_pending_for_replay = prediction.pending_frames.iter().any(|frame| {
                        frame.tick.0 > tick.0 && frame.tick.0 <= local_simulated_tick.0
                    });
                    let authoritative_tick_is_newer = tick.0 > prediction.last_authoritative_tick.0;
                    (has_pending_for_replay, authoritative_tick_is_newer)
                })
                .unwrap_or((false, true))
        })
        .unwrap_or((false, true));
    if !authoritative_tick_is_newer {
        correction_stats.last_distance = 0.0;
        apply_non_transform_player_snapshot(world, entity, snapshot);
        return Ok(());
    }

    let should_replay_reconciliation =
        has_pending_for_replay && distance > interpolation.small_error_distance;

    if should_replay_reconciliation {
        if distance <= interpolation.small_error_distance {
            correction_stats.small_corrections =
                correction_stats.small_corrections.saturating_add(1);
        } else if distance <= interpolation.medium_error_distance {
            correction_stats.medium_corrections =
                correction_stats.medium_corrections.saturating_add(1);
        } else {
            correction_stats.large_corrections =
                correction_stats.large_corrections.saturating_add(1);
        }
        correction_stats.last_distance = distance;
        correction_stats.total_distance += distance;
        if correction_stats.ema_distance <= f32::EPSILON {
            correction_stats.ema_distance = distance;
        } else {
            correction_stats.ema_distance += (distance - correction_stats.ema_distance) * 0.1;
        }

        let _ = world.insert(
            entity,
            Transform2::new(snapshot.x, snapshot.y, snapshot.yaw),
        );
        let _ = world.insert(
            entity,
            Velocity2 {
                x: snapshot.velocity[0],
                y: snapshot.velocity[1],
            },
        );
        apply_non_transform_player_snapshot(world, entity, snapshot);

        if let Some(tick) = authoritative_tick {
            let replayed_frames =
                replay_pending_prediction_frames(world, tick, local_simulated_tick)?;
            if replayed_frames > 0 {
                tracing::trace!(
                    replayed_frames,
                    authoritative_tick = tick.0,
                    local_simulated_tick = local_simulated_tick.0,
                    "replayed local prediction after v2 patch"
                );
            }
        }
        if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
            *tick = local_simulated_tick;
        }
        mark_prediction_authoritative_tick(world, authoritative_tick);
        return Ok(());
    }

    if !has_pending_for_replay && distance <= interpolation.medium_error_distance {
        correction_stats.last_distance = 0.0;
        mark_prediction_authoritative_tick(world, authoritative_tick);
        apply_non_transform_player_snapshot(world, entity, snapshot);
        return Ok(());
    }

    let (correction_gain, velocity_blend, hard_snap) = if distance
        <= interpolation.small_error_distance
    {
        correction_stats.small_corrections = correction_stats.small_corrections.saturating_add(1);
        (0.0, 0.1, false)
    } else if distance <= interpolation.medium_error_distance {
        correction_stats.medium_corrections = correction_stats.medium_corrections.saturating_add(1);
        (0.12, 0.18, false)
    } else if distance <= interpolation.large_error_distance {
        correction_stats.large_corrections = correction_stats.large_corrections.saturating_add(1);
        (0.2, 0.26, false)
    } else if distance <= dynamic_hard_snap_distance {
        correction_stats.large_corrections = correction_stats.large_corrections.saturating_add(1);
        (0.28, 0.34, false)
    } else if distance <= dynamic_hard_snap_distance * 2.25 {
        correction_stats.large_corrections = correction_stats.large_corrections.saturating_add(1);
        (0.4, 0.48, false)
    } else {
        correction_stats.hard_snaps = correction_stats.hard_snaps.saturating_add(1);
        (1.0, 1.0, true)
    };

    let corrected = if hard_snap {
        [snapshot.x, snapshot.y, snapshot.yaw]
    } else {
        let fixed_dt = world
            .resource::<FixedTimeConfig>()
            .map(|config| config.step_seconds.max(1.0 / 120.0))
            .unwrap_or(1.0 / 60.0);
        let mut step_x = dx * correction_gain;
        let mut step_y = dy * correction_gain;
        let desired_step = (step_x * step_x + step_y * step_y).sqrt();
        let current_speed = (current_velocity.x * current_velocity.x
            + current_velocity.y * current_velocity.y)
            .sqrt();
        let base_step_budget =
            current_speed * fixed_dt * 0.9 + interpolation.small_error_distance * 0.25;
        let error_bonus = (distance - interpolation.medium_error_distance).max(0.0) * 0.12;
        let max_step = (base_step_budget + error_bonus).clamp(
            interpolation.small_error_distance * 0.08,
            interpolation.large_error_distance * 0.4,
        );
        if desired_step > max_step && desired_step > f32::EPSILON {
            let scale = max_step / desired_step;
            step_x *= scale;
            step_y *= scale;
        }
        let yaw_gain = (correction_gain * 0.85).clamp(0.0, 0.55);
        [
            current.x + step_x,
            current.y + step_y,
            current.yaw + angle_delta(current.yaw, snapshot.yaw) * yaw_gain,
        ]
    };
    correction_stats.last_distance = distance;
    correction_stats.total_distance += distance;
    if correction_stats.ema_distance <= f32::EPSILON {
        correction_stats.ema_distance = distance;
    } else {
        correction_stats.ema_distance += (distance - correction_stats.ema_distance) * 0.1;
    }
    if hard_snap {
        tracing::debug!(
            distance,
            dynamic_hard_snap_distance,
            "local player hard snap correction applied"
        );
    }

    let _ = world.insert(
        entity,
        Transform2::new(corrected[0], corrected[1], corrected[2]),
    );
    let _ = world.insert(
        entity,
        Velocity2 {
            x: current_velocity.x + (snapshot.velocity[0] - current_velocity.x) * velocity_blend,
            y: current_velocity.y + (snapshot.velocity[1] - current_velocity.y) * velocity_blend,
        },
    );
    mark_prediction_authoritative_tick(world, authoritative_tick);
    apply_non_transform_player_snapshot(world, entity, snapshot);
    Ok(())
}

fn mark_prediction_authoritative_tick(
    world: &mut World,
    authoritative_tick: Option<SimulationTick>,
) {
    if let Some(tick) = authoritative_tick
        && let Ok(mut prediction) = world.resource_mut::<CavernPredictionState>()
        && tick.0 > prediction.last_authoritative_tick.0
    {
        prediction.last_authoritative_tick = tick;
    }
}

pub(super) fn replay_pending_prediction_frames(
    world: &mut World,
    authoritative_tick: SimulationTick,
    local_simulated_tick: SimulationTick,
) -> Result<usize> {
    let fixed_dt = world
        .resource::<FixedTimeConfig>()
        .map(|config| config.step_seconds.max(1.0 / 120.0))
        .unwrap_or(1.0 / 60.0);
    let frames_to_replay = {
        let mut prediction = world.resource_mut::<CavernPredictionState>()?;
        prediction
            .pending_frames
            .retain(|frame| frame.tick.0 > authoritative_tick.0);
        let replay = prediction
            .pending_frames
            .iter()
            .copied()
            .filter(|frame| frame.tick.0 <= local_simulated_tick.0)
            .collect::<Vec<_>>();
        if !replay.is_empty() {
            prediction.corrections_applied = prediction.corrections_applied.saturating_add(1);
        }
        if authoritative_tick.0 > prediction.last_authoritative_tick.0 {
            prediction.last_authoritative_tick = authoritative_tick;
        }
        replay
    };

    let replayed_count = frames_to_replay.len();
    for frame in frames_to_replay {
        crate::plugins::combat::replay_predicted_local_frame(world, frame.control, fixed_dt)?;
        if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
            *tick = frame.tick;
        }
    }

    Ok(replayed_count)
}
