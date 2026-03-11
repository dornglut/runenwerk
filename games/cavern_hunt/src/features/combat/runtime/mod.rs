use super::*;

mod control_step;
mod movement_fire;
mod plugin_aim;
mod projectiles;
#[cfg(test)]
mod tests;

pub(crate) use control_step::replay_predicted_local_frame;
pub use plugin_aim::CavernHuntCombatPlugin;
pub(crate) use projectiles::{constrained_move, spawn_projectile};

use control_step::{fixed_step_combat_system, run_authoritative_combat_step};
use movement_fire::{
    ProjectileStepMode, fire_player_weapon_with_control, move_player_with_control,
    resolve_local_player_entity,
};
use plugin_aim::update_local_aim;
use projectiles::{
    camera_relative_movement, movement_footprint_radius, normalized_vector, step_projectiles,
};
