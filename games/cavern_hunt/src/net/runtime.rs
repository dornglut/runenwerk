#[cfg(test)]
use super::tuning;
use super::{
    chunking::ClientRunEventChunkState, client_input, diagnostics, apply,
    emit, capture, smoothing,
};
#[cfg(test)]
use super::protocol::CavernRunEventCode;
#[cfg(test)]
use crate::{
    CavernPatchEvent, CavernPlayerPatchOp, InterpolationConfig, ReplicationBudgetConfig,
    ReplicationCadenceConfig, ReplicationLoadShedConfig,
};
use crate::{CavernRunSnapshotV1, ReplicationCursor};
use anyhow::Result;
#[cfg(test)]
use engine::prelude::SimulationTick;
use engine::prelude::{
    App, AuthorityRole, CoreSet, FixedUpdate, Plugin, PreUpdate, SimulationProfileConfig,
    SystemConfigExt, Update, World, WorldMut,
};
use std::collections::BTreeMap;
#[cfg(test)]
const RUN_EVENT_KEYFRAME: &str = CavernRunEventCode::Keyframe.as_str();
#[cfg(test)]
const RUN_EVENT_PATCH: &str = CavernRunEventCode::Patch.as_str();

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct CavernNetRuntimeState {
    pub(super) active_connection_id: Option<u64>,
    pub(super) initial_snapshot_sent: bool,
    pub(super) last_cursor: u64,
    pub(super) last_sent_snapshot: Option<CavernRunSnapshotV1>,
    pub(super) last_sent_geometry_edit_count: usize,
    pub(super) last_received_cursor: u64,
    pub(super) last_received_snapshot: Option<CavernRunSnapshotV1>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ServerReplicationByConnectionState {
    pub(super) cursors_by_connection: BTreeMap<u64, ReplicationCursor>,
    pub(super) latest_cursor: ReplicationCursor,
    pub(super) last_snapshot: Option<CavernRunSnapshotV1>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ClientReplicationState {
    pub(super) last_cursor: ReplicationCursor,
    pub(super) has_keyframe: bool,
    pub(super) remote_targets_by_player_id: BTreeMap<u32, RemotePlayerTarget>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(super) struct RemotePlayerTarget {
    pub(super) pos: [f32; 2],
    pub(super) velocity: [f32; 2],
    pub(super) yaw: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub(super) struct NetSyncDiagnosticsLogState {
    pub(super) last_logged_tick: u64,
}

pub struct CavernHuntNetPlugin;

impl Plugin for CavernHuntNetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::CavernReplicationIntent>();
        app.init_resource::<CavernNetRuntimeState>();
        app.init_resource::<ServerReplicationByConnectionState>();
        app.init_resource::<ClientReplicationState>();
        app.init_resource::<ClientRunEventChunkState>();
        app.init_resource::<NetSyncDiagnosticsLogState>();
        app.add_systems(
            PreUpdate,
            (
                client_send_control_input_system
                    .after(CoreSet::NetReceive)
                    .after(CoreSet::Input),
                capture_control_input_system.after(CoreSet::NetReceive),
                client_apply_replication_events_system.after(CoreSet::NetReceive),
            ),
        );
        app.add_systems(
            Update,
            (client_smoothing_system, net_sync_diagnostics_log_system),
        );
        app.add_systems(FixedUpdate, server_emit_replication_system);
    }
}

fn client_send_control_input_system(mut world: WorldMut) -> Result<()> {
    client_send_control_input(&mut world)
}

fn client_send_control_input(world: &mut World) -> Result<()> {
    client_input::client_send_control_input(world)
}

fn capture_control_input_system(mut world: WorldMut) -> Result<()> {
    capture_control_input(&mut world)
}

fn capture_control_input(world: &mut World) -> Result<()> {
    capture::server_capture_control_input(world)
}

fn server_emit_replication_system(mut world: WorldMut) -> Result<()> {
    server_emit_replication(&mut world)
}

fn server_emit_replication(world: &mut World) -> Result<()> {
    if split_driver_runtime_enabled(world) {
        return Ok(());
    }
    emit::server_emit_replication(world)
}

fn client_apply_replication_events_system(mut world: WorldMut) -> Result<()> {
    client_apply_replication_events_runtime(&mut world)
}

fn client_apply_replication_events_runtime(world: &mut World) -> Result<()> {
    if split_driver_runtime_enabled(world) {
        return Ok(());
    }
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Client) {
        return Ok(());
    }
    // Single runtime protocol path.
    client_apply_replication_events(world)
}

fn split_driver_runtime_enabled(world: &World) -> bool {
    world
        .resource::<engine::plugins::net::SnapshotReplicationState<CavernRunSnapshotV1>>()
        .is_ok()
}

#[cfg(test)]
#[allow(dead_code)]
fn configure_replication_tuning_from_env_system(mut world: WorldMut) -> Result<()> {
    configure_replication_tuning_from_env(&mut world)
}

#[cfg(test)]
#[allow(dead_code)]
fn configure_replication_tuning_from_env(world: &mut World) -> Result<()> {
    tuning::configure_replication_tuning_from_env(world)
}

#[cfg(test)]
fn apply_replication_tuning_preset(
    budget: &mut ReplicationBudgetConfig,
    cadence: &mut ReplicationCadenceConfig,
    preset: Option<&str>,
    diagnostics: &mut Vec<String>,
) {
    tuning::apply_replication_tuning_preset(budget, cadence, preset, diagnostics);
}

#[cfg(test)]
fn apply_replication_tuning_overrides_from_reader<F>(
    budget: &mut ReplicationBudgetConfig,
    cadence: &mut ReplicationCadenceConfig,
    read_var: F,
    diagnostics: &mut Vec<String>,
) where
    F: Fn(&str) -> Option<String>,
{
    tuning::apply_replication_tuning_overrides_from_reader(budget, cadence, read_var, diagnostics);
}

#[cfg(test)]
#[allow(dead_code)]
fn apply_interpolation_overrides_from_reader<F>(
    interpolation: &mut InterpolationConfig,
    read_var: F,
    diagnostics: &mut Vec<String>,
) where
    F: Fn(&str) -> Option<String>,
{
    tuning::apply_interpolation_overrides_from_reader(interpolation, read_var, diagnostics);
}

#[cfg(test)]
fn compute_load_shed_level(
    previous_sent_bytes: u64,
    previous_dropped_ops: u64,
    connection_count: usize,
    config: &ReplicationLoadShedConfig,
) -> u8 {
    emit::compute_load_shed_level(
        previous_sent_bytes,
        previous_dropped_ops,
        connection_count,
        config,
    )
}

#[cfg(test)]
fn should_emit_patch_channel(stream_cursor: u64, interval_ticks: u64) -> bool {
    emit::should_emit_patch_channel(stream_cursor, interval_ticks)
}

fn client_apply_replication_events(world: &mut World) -> Result<()> {
    apply::client_apply_replication_events(world)
}

#[cfg(test)]
fn apply_player_patch_ops(
    world: &mut World,
    ops: Vec<CavernPlayerPatchOp>,
    cursor_authoritative_tick: Option<SimulationTick>,
    apply_local_owned_correction: bool,
) -> Result<()> {
    apply::apply_player_patch_ops(
        world,
        ops,
        cursor_authoritative_tick,
        apply_local_owned_correction,
    )
}

pub(super) fn angle_delta(current: f32, target: f32) -> f32 {
    let mut delta = target - current;
    while delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    while delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

fn client_smoothing_system(mut world: WorldMut) -> Result<()> {
    smoothing::client_smoothing_system(&mut world)
}

fn net_sync_diagnostics_log_system(mut world: WorldMut) -> Result<()> {
    diagnostics::net_sync_diagnostics_log_system(&mut world)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

