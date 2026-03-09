use anyhow::Result;
use engine::prelude::{AuthorityRole, RoundTripMetrics, SimulationProfileConfig, Time, World};

use super::legacy::{ClientReplicationStateV2, angle_delta};
use crate::{
    AdaptiveSmoothingState, ClientReplicationMap, InterpolationConfig, LocalPlayerRef,
    ReplicationRuntimeMetrics, Transform2, Velocity2,
};

pub(super) fn client_smoothing_system(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Client | AuthorityRole::Peer) {
        return Ok(());
    }

    let dt = world
        .resource::<Time>()
        .map(|time| time.delta_seconds.max(0.0))
        .unwrap_or(0.0);
    if dt <= f32::EPSILON {
        return Ok(());
    }
    let interpolation = world
        .resource::<InterpolationConfig>()
        .copied()
        .unwrap_or_default();
    let replication_metrics = world
        .resource::<ReplicationRuntimeMetrics>()
        .copied()
        .unwrap_or_default();
    let dropped_ops_last_tick = replication_metrics
        .dropped_enemy_ops_last_tick
        .saturating_add(replication_metrics.dropped_projectile_ops_last_tick)
        .saturating_add(replication_metrics.dropped_pickup_ops_last_tick)
        .saturating_add(replication_metrics.dropped_extraction_ops_last_tick);
    let load_shed_level = replication_metrics.load_shed_level_last_tick as f32;

    let rtt_ms = world
        .resource::<RoundTripMetrics>()
        .ok()
        .and_then(|metrics| metrics.last_rtt_millis.map(|value| value as f32))
        .unwrap_or(0.0);
    if let Ok(mut smoothing) = world.resource_mut::<AdaptiveSmoothingState>() {
        let measured_rtt_ms = if rtt_ms > 0.0 {
            rtt_ms
        } else {
            smoothing.last_rtt_ms
        };
        if measured_rtt_ms > 0.0 {
            let delta = (measured_rtt_ms - smoothing.last_rtt_ms).abs();
            smoothing.jitter_ms = smoothing.jitter_ms * 0.9 + delta * 0.1;
            smoothing.last_rtt_ms = measured_rtt_ms;
            smoothing.samples = smoothing.samples.saturating_add(1);
        }
        let shed_penalty_ms = load_shed_level * 10.0;
        let drop_penalty_ms = (dropped_ops_last_tick as f32).sqrt().min(20.0);
        let target = (measured_rtt_ms * 0.5
            + smoothing.jitter_ms * 1.5
            + 35.0
            + shed_penalty_ms
            + drop_penalty_ms)
            .clamp(interpolation.min_delay_ms, interpolation.max_delay_ms);
        smoothing.target_delay_ms = target;
        let blend = (dt * 6.0).clamp(0.0, 1.0);
        smoothing.effective_delay_ms += (target - smoothing.effective_delay_ms) * blend;
    }

    let (delay_seconds, extrapolation_seconds) = world
        .resource::<AdaptiveSmoothingState>()
        .map(|state| {
            let delay_seconds = (state.effective_delay_ms / 1000.0).max(0.01);
            let extrapolation_seconds =
                ((state.effective_delay_ms / 1000.0) + load_shed_level * 0.012).clamp(0.0, 0.22);
            (delay_seconds, extrapolation_seconds)
        })
        .unwrap_or((0.08, 0.08));
    let base_alpha = (dt / delay_seconds).clamp(0.0, 1.0);

    let local_player_id = world
        .resource::<LocalPlayerRef>()
        .ok()
        .and_then(|local| local.player_id);
    let client_map = world
        .resource::<ClientReplicationMap>()
        .cloned()
        .unwrap_or_default();
    let state = world
        .resource_mut::<ClientReplicationStateV2>()
        .map(|state| state.clone())
        .unwrap_or_default();

    let mut smoothing_samples = 0_u64;
    let mut smoothing_error_sum = 0.0_f32;
    let mut smoothing_error_max = 0.0_f32;
    let mut smoothing_alpha_sum = 0.0_f32;

    for (player_id, target) in &state.remote_targets_by_player_id {
        if Some(*player_id) == local_player_id {
            continue;
        }
        let Some(network_entity_id) = client_map.by_player_id.get(player_id).copied() else {
            continue;
        };
        let Some(entity) = client_map
            .by_network_entity_id
            .get(&network_entity_id)
            .copied()
        else {
            continue;
        };
        let velocity_alpha = {
            let Some(mut transform) = world.get_mut::<Transform2>(entity) else {
                continue;
            };
            let predicted_x = target.pos[0] + target.velocity[0] * extrapolation_seconds;
            let predicted_y = target.pos[1] + target.velocity[1] * extrapolation_seconds;
            let error_dx = predicted_x - transform.x;
            let error_dy = predicted_y - transform.y;
            let error_distance = (error_dx * error_dx + error_dy * error_dy).sqrt();
            let catch_up = if error_distance <= interpolation.small_error_distance {
                0.8
            } else if error_distance <= interpolation.medium_error_distance {
                1.0
            } else if error_distance <= interpolation.large_error_distance {
                1.35
            } else {
                1.8
            };
            let entity_alpha = (base_alpha * catch_up).clamp(0.0, 1.0);

            let mut step_x = error_dx * entity_alpha;
            let mut step_y = error_dy * entity_alpha;
            let desired_step = (step_x * step_x + step_y * step_y).sqrt();
            let target_speed = (target.velocity[0] * target.velocity[0]
                + target.velocity[1] * target.velocity[1])
                .sqrt();
            let base_step_budget =
                target_speed * dt * 1.4 + interpolation.small_error_distance * 0.4;
            let error_bonus = (error_distance - interpolation.medium_error_distance).max(0.0) * 0.2;
            let max_step = (base_step_budget + error_bonus).clamp(
                interpolation.small_error_distance * 0.35,
                interpolation.large_error_distance * 0.85,
            );
            if desired_step > max_step && desired_step > f32::EPSILON {
                let scale = max_step / desired_step;
                step_x *= scale;
                step_y *= scale;
            }

            transform.x += step_x;
            transform.y += step_y;
            transform.yaw += angle_delta(transform.yaw, target.yaw) * entity_alpha;

            smoothing_samples = smoothing_samples.saturating_add(1);
            smoothing_error_sum += error_distance;
            smoothing_error_max = smoothing_error_max.max(error_distance);
            smoothing_alpha_sum += entity_alpha;

            (entity_alpha * 1.15).clamp(0.0, 1.0)
        };
        if let Some(mut velocity) = world.get_mut::<Velocity2>(entity) {
            velocity.x += (target.velocity[0] - velocity.x) * velocity_alpha;
            velocity.y += (target.velocity[1] - velocity.y) * velocity_alpha;
        }
    }
    if let Ok(mut metrics) = world.resource_mut::<ReplicationRuntimeMetrics>() {
        metrics.smoothing_samples_last_frame = smoothing_samples;
        metrics.smoothing_error_mean_last_frame = if smoothing_samples > 0 {
            smoothing_error_sum / smoothing_samples as f32
        } else {
            0.0
        };
        metrics.smoothing_error_max_last_frame = smoothing_error_max;
        metrics.smoothing_alpha_mean_last_frame = if smoothing_samples > 0 {
            smoothing_alpha_sum / smoothing_samples as f32
        } else {
            0.0
        };
        metrics.smoothing_extrapolation_ms_last_frame = extrapolation_seconds * 1000.0;
    }
    Ok(())
}
