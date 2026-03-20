use engine_net::replication::ReplicationProfilePreset;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct CavernReplicationPolicy {
    pub player_state_profile: ReplicationProfilePreset,
    pub player_input_profile: ReplicationProfilePreset,
    pub health_profile: ReplicationProfilePreset,
    pub smoothing_blend_alpha: f32,
    pub correction_hard_snap_distance: f32,
}

impl Default for CavernReplicationPolicy {
    fn default() -> Self {
        Self {
            player_state_profile: ReplicationProfilePreset::PredictedMovement,
            player_input_profile: ReplicationProfilePreset::InputCommand,
            health_profile: ReplicationProfilePreset::ReliableState,
            smoothing_blend_alpha: 0.25,
            correction_hard_snap_distance: 2.0,
        }
    }
}
