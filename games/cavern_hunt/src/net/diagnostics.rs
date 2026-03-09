use anyhow::Result;
use engine::prelude::{
    AuthorityRole, NetworkSessionStatus, RoundTripMetrics, SimulationProfileConfig, SimulationTick,
    World,
};

use super::legacy::{ClientReplicationStateV2, NetSyncDiagnosticsLogState};
use crate::{
    AdaptiveSmoothingState, CorrectionStats, LocalPlayerRef, NetDiagnosticsConfigAssetV1,
    ReplicationRuntimeMetrics,
};

pub(super) fn net_sync_diagnostics_log_system(world: &mut World) -> Result<()> {
    let diagnostics = world
        .resource::<NetDiagnosticsConfigAssetV1>()
        .copied()
        .unwrap_or_default();
    if !diagnostics.enable_periodic_log {
        return Ok(());
    }
    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default()
        .0;
    let interval_ticks = diagnostics.log_interval_ticks.max(1);
    if interval_ticks == 0 {
        return Ok(());
    }
    {
        let mut state = world.resource_mut::<NetSyncDiagnosticsLogState>()?;
        if tick == 0 || tick < state.last_logged_tick.saturating_add(interval_ticks) {
            return Ok(());
        }
        state.last_logged_tick = tick;
    }

    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    let connected = world
        .resource::<NetworkSessionStatus>()
        .map(|status| status.connected)
        .unwrap_or(false);
    let connection_id = world
        .resource::<NetworkSessionStatus>()
        .ok()
        .and_then(|status| status.connection_id.map(|id| id.0));
    let local_player_id = world
        .resource::<LocalPlayerRef>()
        .ok()
        .and_then(|local| local.player_id);
    let metrics = world
        .resource::<ReplicationRuntimeMetrics>()
        .copied()
        .unwrap_or_default();
    let smoothing = world
        .resource::<AdaptiveSmoothingState>()
        .copied()
        .unwrap_or_default();
    let correction = world
        .resource::<CorrectionStats>()
        .copied()
        .unwrap_or_default();
    let replication_state = world
        .resource::<ClientReplicationStateV2>()
        .cloned()
        .unwrap_or_default();
    let rtt_ms = world
        .resource::<RoundTripMetrics>()
        .ok()
        .and_then(|metrics| metrics.last_rtt_millis.map(|millis| millis as f32))
        .unwrap_or(smoothing.last_rtt_ms);

    tracing::info!(
        tick,
        ?authority,
        connected,
        connection_id,
        local_player_id,
        tx_bytes = metrics.bytes_sent_last_tick,
        rx_bytes = metrics.bytes_received_last_frame,
        rx_keyframes = metrics.keyframes_received_last_frame,
        rx_patches = metrics.patches_received_last_frame,
        load_shed_level = metrics.load_shed_level_last_tick,
        patch_us = metrics.patch_apply_micros_last,
        keyframes = metrics.keyframes_applied,
        patches = metrics.patches_applied,
        patches_frame = metrics.patches_applied_last_frame,
        patch_skip_mismatch = metrics.patches_skipped_base_mismatch_last_frame,
        patch_skip_stale = metrics.patches_stale_ignored_last_frame,
        restores = metrics.full_world_restores,
        patch_players = metrics.patch_player_ops_last_tick,
        patch_enemies = metrics.patch_enemy_ops_last_tick,
        patch_projectiles = metrics.patch_projectile_ops_last_tick,
        patch_pickups = metrics.patch_pickup_ops_last_tick,
        patch_extraction = metrics.patch_extraction_ops_last_tick,
        repl_cursor = replication_state.last_cursor.stream_cursor,
        repl_has_keyframe = replication_state.has_keyframe,
        drop_enemy = metrics.dropped_enemy_ops_last_tick,
        drop_projectile = metrics.dropped_projectile_ops_last_tick,
        drop_pickup = metrics.dropped_pickup_ops_last_tick,
        drop_extraction = metrics.dropped_extraction_ops_last_tick,
        rtt_ms,
        jitter_ms = smoothing.jitter_ms,
        smooth_delay_ms = smoothing.effective_delay_ms,
        smooth_samples = metrics.smoothing_samples_last_frame,
        smooth_err_mean = metrics.smoothing_error_mean_last_frame,
        smooth_err_max = metrics.smoothing_error_max_last_frame,
        smooth_alpha_mean = metrics.smoothing_alpha_mean_last_frame,
        smooth_extrapolation_ms = metrics.smoothing_extrapolation_ms_last_frame,
        correction_last = metrics.local_correction_distance_last,
        correction_ema = correction.ema_distance,
        correction_hard_snaps = metrics.local_correction_hard_snaps_total,
        "cavern net sync diagnostics"
    );

    Ok(())
}
